// Copyright 2026 Janek Bevendorff
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::stream_io::{PyReaderAdapter, PyWriterAdapter};
use fastwarc::record::DigestError::StreamError;
use fastwarc::record::{ArchiveIterator as RustArchiveIterator, HeaderEncoding, HeaderMap, WarcRecord, WarcRecordType};
use pyo3::exceptions::{PyKeyError, PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyIterator, PyString, PyTuple};
use std::cell::RefCell;
use std::io::BufReader;
use std::rc::Rc;

// ===========================================================
// WarcRecordType
// ===========================================================

#[allow(non_camel_case_types)]
#[pyclass(name = "WarcRecordType", eq, eq_int, from_py_object)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WarcRecordTypePy {
    warcinfo = 2,
    response = 4,
    resource = 8,
    request = 16,
    metadata = 32,
    revisit = 64,
    conversion = 128,
    continuation = 256,
    unknown = 512,
    any_type = 65535,
    no_type = 0,
}

impl From<WarcRecordTypePy> for WarcRecordType {
    fn from(value: WarcRecordTypePy) -> Self {
        match value {
            WarcRecordTypePy::warcinfo => WarcRecordType::WarcInfo,
            WarcRecordTypePy::response => WarcRecordType::Response,
            WarcRecordTypePy::resource => WarcRecordType::Resource,
            WarcRecordTypePy::request => WarcRecordType::Request,
            WarcRecordTypePy::metadata => WarcRecordType::Metadata,
            WarcRecordTypePy::revisit => WarcRecordType::Revisit,
            WarcRecordTypePy::conversion => WarcRecordType::Conversion,
            WarcRecordTypePy::continuation => WarcRecordType::Continuation,
            WarcRecordTypePy::unknown => WarcRecordType::Unknown,
            WarcRecordTypePy::any_type => WarcRecordType::AnyType,
            WarcRecordTypePy::no_type => WarcRecordType::NoType,
        }
    }
}

impl From<WarcRecordType> for WarcRecordTypePy {
    fn from(value: WarcRecordType) -> Self {
        match value {
            WarcRecordType::WarcInfo => WarcRecordTypePy::warcinfo,
            WarcRecordType::Response => WarcRecordTypePy::response,
            WarcRecordType::Resource => WarcRecordTypePy::resource,
            WarcRecordType::Request => WarcRecordTypePy::request,
            WarcRecordType::Metadata => WarcRecordTypePy::metadata,
            WarcRecordType::Revisit => WarcRecordTypePy::revisit,
            WarcRecordType::Conversion => WarcRecordTypePy::conversion,
            WarcRecordType::Continuation => WarcRecordTypePy::continuation,
            WarcRecordType::Unknown => WarcRecordTypePy::unknown,
            WarcRecordType::AnyType => WarcRecordTypePy::any_type,
            WarcRecordType::NoType => WarcRecordTypePy::no_type,
        }
    }
}

// ===========================================================
// HeaderMap
// ===========================================================

#[pyclass(name = "HeaderMap")]
#[derive(Default)]
pub struct HeaderMapPy {
    inner: HeaderMap,
}

impl From<HeaderMap> for HeaderMapPy {
    fn from(inner: HeaderMap) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl HeaderMapPy {
    #[new]
    #[pyo3(signature = (encoding="utf-8"))]
    pub fn __new__(encoding: &str) -> PyResult<Self> {
        let enc = match encoding.to_ascii_lowercase().as_str() {
            "utf-8" => HeaderEncoding::Unicode,
            "latin1" => HeaderEncoding::Latin1,
            _ => return Err(PyValueError::new_err(format!("Unsupported encoding: {}", encoding))),
        };
        Ok(Self {
            inner: HeaderMap::new(enc),
        })
    }

    pub fn encoding(&self) -> &'static str {
        match self.inner.encoding() {
            HeaderEncoding::Unicode => "utf-8",
            HeaderEncoding::Latin1 => "latin1",
        }
    }

    #[pyo3(signature = (reader, has_status_line=true))]
    pub fn parse(&mut self, reader: Py<PyAny>, has_status_line: bool) -> PyResult<usize> {
        let mut reader = BufReader::new(PyReaderAdapter::new(reader));
        Ok(self.inner.parse(&mut reader, has_status_line)?)
    }

    pub fn write(&self, writer: Py<PyAny>) -> PyResult<usize> {
        let mut writer = PyWriterAdapter::new(writer);
        Ok(self.inner.write(&mut writer)?)
    }

    #[getter]
    pub fn reason_phrase<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.inner.reason_phrase().map(|s| PyString::new(py, s.as_ref()))
    }

    #[getter]
    pub fn status_code(&self) -> Option<u16> {
        self.inner.status_code()
    }

    #[getter]
    pub fn status_line<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.inner.status_line().map(|s| PyString::new(py, s.as_ref()))
    }

    #[setter]
    pub fn set_status_line(&mut self, status_line: &str) {
        self.inner.set_status_line(status_line);
    }

    #[getter]
    pub fn status_line_bytes<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.inner.status_line_bytes().map(|s| PyBytes::new(py, s))
    }

    #[setter]
    pub fn set_status_line_bytes(&mut self, status_line: &[u8]) {
        self.inner.set_status_line_bytes(status_line);
    }

    pub fn append(&mut self, key: &str, value: &str) {
        self.inner.append(key, value);
    }

    pub fn asdict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (key, value) in self.inner.to_map() {
            dict.set_item(key.as_ref(), value)?;
        }
        Ok(dict)
    }

    pub fn astuples<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        self.items(py)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[pyo3(signature = (key, default=None))]
    pub fn get<'py>(
        &self,
        py: Python<'py>,
        key: &str,
        default: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        Ok(self.inner.get(key).map_or_else(
            || default.unwrap_or_else(|| py.None().bind(py).clone()),
            |s| PyString::new(py, s.as_ref()).into_any(),
        ))
    }

    pub fn get_multiple<'py>(&self, py: Python<'py>, key: &str) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .get_multiple(key)
            .into_iter()
            .map(|s| PyString::new(py, s.as_ref()));
        PyTuple::new(py, items)
    }

    pub fn get_bytes<'py>(
        &self,
        py: Python<'py>,
        key: &[u8],
        default: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        Ok(self
            .inner
            .get_bytes(key)
            .map_or_else(|| default.unwrap_or_else(|| py.None().bind(py).clone()), |s| PyBytes::new(py, s).into_any()))
    }

    pub fn get_bytes_multiple<'py>(&self, py: Python<'py>, key: &[u8]) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .get_bytes_multiple(key)
            .into_iter()
            .map(|s| PyBytes::new(py, s.as_ref()));
        PyTuple::new(py, items)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    pub fn contains_key_bytes(&self, key: &[u8]) -> bool {
        self.inner.contains_key_bytes(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.inner.set(key, value);
    }

    pub fn set_bytes(&mut self, key: &[u8], value: &[u8]) {
        self.inner.set_bytes(key, value);
    }

    pub fn append_bytes(&mut self, key: &[u8], value: &[u8]) {
        self.inner.append_bytes(key, value);
    }

    pub fn remove(&mut self, key: &str) {
        self.inner.remove(key);
    }

    pub fn remove_bytes(&mut self, key: &[u8]) {
        self.inner.remove_bytes(key);
    }

    pub fn items<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .items()
            .map(|(k, v)| PyTuple::new(py, [k.as_ref(), v.as_ref()]))
            .collect::<PyResult<Vec<_>>>()?;
        PyTuple::new(py, items)
    }

    pub fn items_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .items_bytes()
            .map(|(k, v)| PyTuple::new(py, [k, v]))
            .collect::<PyResult<Vec<_>>>()?;
        PyTuple::new(py, items)
    }

    pub fn keys<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .keys()
            .map(|k| PyString::new(py, k.as_ref()))
            .collect::<Vec<_>>();
        PyTuple::new(py, items)
    }

    pub fn keys_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self.inner.keys_bytes().map(|k| PyBytes::new(py, k)).collect::<Vec<_>>();
        PyTuple::new(py, items)
    }

    pub fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .values()
            .map(|k| PyString::new(py, k.as_ref()))
            .collect::<Vec<_>>();
        PyTuple::new(py, items)
    }

    pub fn values_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self
            .inner
            .values_bytes()
            .map(|k| PyBytes::new(py, k))
            .collect::<Vec<_>>();
        PyTuple::new(py, items)
    }

    pub fn __getitem__<'py>(&self, py: Python<'py>, item: &str) -> PyResult<Bound<'py, PyString>> {
        self.inner
            .get(item)
            .map(|s| PyString::new(py, s.as_ref()))
            .ok_or_else(|| PyKeyError::new_err(item.to_string()))
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        Ok(PyIterator::from_object(self.items(py)?.as_any())?)
    }

    pub fn __len__(&self) -> usize {
        self.inner.len()
    }

    pub fn __setitem__(&mut self, key: &str, value: &str) {
        self.inner.set(key, value);
    }

    pub fn __contains__(&self, item: &str) -> bool {
        self.inner.contains_key(item)
    }
}

// ===========================================================
// WarcRecord
// ===========================================================

#[pyclass(name = "WarcRecord", unsendable)]
pub struct WarcRecordPy {
    inner: Rc<RefCell<WarcRecord>>,
}

#[pymethods]
impl WarcRecordPy {
    #[new]
    pub fn __new__() -> Self {
        Self {
            inner: Rc::new(RefCell::new(WarcRecord::new())),
        }
    }

    #[getter]
    pub fn record_id<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.inner.borrow().record_id().map(|s| PyString::new(py, s.as_ref()))
    }

    #[getter]
    pub fn record_type(&self) -> WarcRecordTypePy {
        self.inner.borrow().record_type().into()
    }

    #[setter]
    pub fn set_record_type(&mut self, record_type: WarcRecordTypePy) {
        self.inner.borrow_mut().set_record_type(record_type.into());
    }

    #[getter]
    pub fn content_length(&self) -> u64 {
        self.inner.borrow().content_length()
    }

    #[getter]
    pub fn record_date<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        let inner = self.inner.borrow();
        let Some(value) = inner.headers().get("WARC-Date") else {
            return Ok(None);
        };
        let datetime_module = py.import("datetime")?;
        let datetime = datetime_module.getattr("datetime")?;
        let normalized = value.replace('Z', "+00:00");
        match datetime.call_method1("fromisoformat", (normalized,)) {
            Ok(obj) => Ok(Some(obj)),
            Err(_) => Ok(None),
        }
    }

    #[getter]
    pub fn headers(&self) -> HeaderMapPy {
        self.inner.borrow().headers().clone().into()
    }

    #[getter]
    pub fn is_http(&self) -> bool {
        self.inner.borrow().is_http()
    }

    #[setter]
    pub fn set_is_http(&mut self, is_http: bool) {
        self.inner.borrow_mut().set_is_http(is_http);
    }

    #[getter]
    pub fn is_http_parsed(&self) -> bool {
        self.inner.borrow().is_http_parsed()
    }

    #[getter]
    pub fn http_headers(&self) -> Option<HeaderMapPy> {
        self.inner.borrow().http_headers().cloned().map(Into::into)
    }

    #[getter]
    pub fn http_content_type<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.inner
            .borrow()
            .http_content_type()
            .as_deref()
            .map(|s| PyString::new(py, s))
    }

    #[getter]
    pub fn http_charset<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.inner.borrow().http_charset().map(|s| PyString::new(py, s))
    }

    #[getter]
    pub fn http_date<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        http_datetime_to_py(
            py,
            self.inner
                .borrow()
                .http_headers()
                .and_then(|h| h.get("Date"))
                .as_deref(),
        )
    }

    #[getter]
    pub fn http_last_modified<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        http_datetime_to_py(
            py,
            self.inner
                .borrow()
                .http_headers()
                .and_then(|h| h.get("Last-Modified"))
                .as_deref(),
        )
    }

    #[getter]
    pub fn reader<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        py.None().bind(py).clone()
    }

    #[getter]
    pub fn stream_pos(&self) -> u64 {
        self.inner.borrow().stream_pos()
    }

    #[getter]
    pub fn is_frozen(&self) -> bool {
        self.inner.borrow().is_frozen()
    }

    #[pyo3(signature = (content_length=0, record_type=WarcRecordTypePy::no_type, record_urn=None))]
    pub fn init_headers(&mut self, content_length: u64, record_type: WarcRecordTypePy, record_urn: Option<&[u8]>) {
        self.inner
            .borrow_mut()
            .init_headers(content_length, Some(record_type.into()), record_urn);
    }

    pub fn freeze(&mut self) -> PyResult<bool> {
        self.inner.borrow_mut().freeze()?;
        Ok(true)
    }

    pub fn set_bytes_content(&mut self, content: &[u8]) {
        self.set_bytes_payload(content);
    }

    pub fn set_bytes_payload(&mut self, content: &[u8]) {
        self.inner.borrow_mut().set_bytes_payload(content.to_vec());
    }

    #[pyo3(signature = (n=None))]
    pub fn consume(&mut self, n: Option<usize>) -> PyResult<usize> {
        match n {
            Some(n) => Ok(self.inner.borrow_mut().consume_n(n)?),
            None => Ok(self.inner.borrow_mut().consume()?),
        }
    }

    #[pyo3(signature = (quirks_mode=false))]
    pub fn parse_warc_headers(&mut self, quirks_mode: bool) -> PyResult<usize> {
        if quirks_mode {
            Ok(self.inner.borrow_mut().parse_warc_headers_quirks(quirks_mode)?)
        } else {
            Ok(self.inner.borrow_mut().parse_warc_headers()?)
        }
    }

    #[pyo3(signature = (strict_mode=true, auto_decode="none"))]
    pub fn parse_http(&mut self, strict_mode: bool, auto_decode: &str) -> PyResult<()> {
        // TODO: Implement parameters
        let _ = (strict_mode, auto_decode);
        self.inner.borrow_mut().parse_http()?;
        Ok(())
    }

    #[pyo3(signature = (consume=false))]
    pub fn verify_block_digest(&mut self, consume: bool) -> PyResult<bool> {
        self.inner
            .borrow_mut()
            .verify_block_digest(consume)
            .map_err(|e| match e {
                StreamError(_) => PyOSError::new_err(e.to_string()),
                _ => PyValueError::new_err(e.to_string()),
            })
    }

    #[pyo3(signature = (consume=false))]
    pub fn verify_payload_digest(&mut self, consume: bool) -> PyResult<bool> {
        self.inner
            .borrow_mut()
            .verify_payload_digest(consume)
            .map_err(|e| match e {
                StreamError(_) => PyOSError::new_err(e.to_string()),
                _ => PyValueError::new_err(e.to_string()),
            })
    }

    #[pyo3(signature = (stream, checksum_data=false, payload_digest=None, chunk_size=16384))]
    pub fn write<'py>(
        &mut self,
        py: Python<'py>,
        stream: Bound<'py, PyAny>,
        checksum_data: bool,
        payload_digest: Option<&[u8]>,
        chunk_size: usize,
    ) -> PyResult<usize> {
        if let Some(payload_digest) = payload_digest
            && !checksum_data
        {
            let encoded = py
                .import("base64")?
                .getattr("b32encode")?
                .call1((PyBytes::new(py, payload_digest),))?
                .extract::<Bound<'_, PyBytes>>()?;
            let mut digest_header = b"sha1:".to_vec();
            digest_header.extend_from_slice(encoded.as_bytes());
            self.inner
                .borrow_mut()
                .headers_mut()
                .set_bytes(b"WARC-Payload-Digest", &digest_header);
        }

        let mut writer = PyWriterAdapter::new(stream.unbind());
        if checksum_data {
            Ok(self
                .inner
                .borrow_mut()
                .write_with_block_size_checksum(&mut writer, chunk_size, true)?)
        } else {
            Ok(self.inner.borrow_mut().write_with_block_size(&mut writer, chunk_size)?)
        }
    }
}

fn http_datetime_to_py<'py>(py: Python<'py>, value: Option<&str>) -> PyResult<Option<Bound<'py, PyAny>>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let parsed = py
        .import("email.utils")?
        .getattr("parsedate_to_datetime")?
        .call1((value,));
    match parsed {
        Ok(obj) => Ok(Some(obj)),
        Err(_) => Ok(None),
    }
}

// ===========================================================
// ArchiveIterator
// ===========================================================

#[pyclass(name = "ArchiveIterator", unsendable)]
pub struct ArchiveIteratorPy {
    inner: RustArchiveIterator,
    record_types: u16,
    parse_http: bool,
    min_content_length: Option<u64>,
    max_content_length: Option<u64>,
    func_filter: Option<Py<PyAny>>,
    verify_digests: bool,
    strict_mode: bool,
    auto_decode: String,
}

#[pymethods]
impl ArchiveIteratorPy {
    #[new]
    #[pyo3(signature = (
        stream,
        record_types=WarcRecordTypePy::any_type,
        parse_http=true,
        min_content_length=-1,
        max_content_length=-1,
        func_filter=None,
        verify_digests=false,
        strict_mode=true,
        auto_decode="none"
    ))]
    pub fn __new__(
        stream: Py<PyAny>,
        record_types: WarcRecordTypePy,
        parse_http: bool,
        min_content_length: i64,
        max_content_length: i64,
        func_filter: Option<Py<PyAny>>,
        verify_digests: bool,
        strict_mode: bool,
        auto_decode: &str,
    ) -> Self {
        let mut inner = RustArchiveIterator::new(Box::new(BufReader::new(PyReaderAdapter::new(stream))));
        inner.set_parse_http(parse_http);
        inner.set_verify_digests(verify_digests);
        Self {
            inner,
            record_types: record_types as u16,
            parse_http,
            min_content_length: u64::try_from(min_content_length).ok(),
            max_content_length: u64::try_from(max_content_length).ok(),
            func_filter,
            verify_digests,
            strict_mode,
            auto_decode: auto_decode.to_owned(),
        }
    }

    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__<'py>(&mut self, py: Python<'py>) -> PyResult<Option<Py<WarcRecordPy>>> {
        loop {
            let Some(next) = self.inner.next() else {
                return Ok(None);
            };
            let record = next?;
            let record_ref = record.borrow_mut();

            let content_length = record_ref.content_length();
            if !record_ref.record_type().matches_bitmask(self.record_types)
                || self.min_content_length.is_some_and(|min| content_length < min)
                || self.max_content_length.is_some_and(|max| content_length > max)
            {
                let _ = &self.strict_mode;
                continue;
            }
            let _ = (self.parse_http, self.verify_digests, &self.auto_decode, self.strict_mode);
            drop(record_ref);

            let record_obj = Py::new(py, WarcRecordPy { inner: record.clone() })?;
            if let Some(func_filter) = &self.func_filter {
                let keep = func_filter.bind(py).call1((record_obj.bind(py),))?.is_truthy()?;
                if !keep {
                    continue;
                }
            }
            return Ok(Some(record_obj));
        }
    }
}
