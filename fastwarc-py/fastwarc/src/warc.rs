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

use crate::stream_io::{
    PyReaderAdapter, PyWriterAdapter, WarcReaderPy, path_like_to_string, python_whence_to_seekfrom, wrap_reader_stream,
};
use fastwarc::stream_io::LimitedBufReader;
use fastwarc::stream_io::traits::{IntoWarcReader, WarcRead};
use fastwarc::warc::iter::{ArchiveIteratorOptions, ArchiveIteratorThreadSafe, SharedWarcRecord, filter};
use fastwarc::warc::record::DigestError::StreamError;
use fastwarc::warc::record::{AutoDecode, HeaderEncoding, HeaderMap, WarcRecord, WarcRecordType};
use pyo3::exceptions::{PyKeyError, PyOSError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDateTime, PyDict, PyIterator, PyString, PyTuple};
use std::io::{self, Read, Seek};
use std::sync::{Arc, Mutex, MutexGuard};

// ===========================================================
// WarcRecordType
// ===========================================================

#[allow(non_camel_case_types)]
#[pyclass(name = "WarcRecordType", module = "fastwarc.warc", eq, eq_int, from_py_object)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
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
    #[default]
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

#[pymethods]
impl WarcRecordTypePy {
    pub fn __int__(&self) -> u16 {
        *self as u16
    }

    pub fn __index__(&self) -> u16 {
        *self as u16
    }

    pub fn __and__(&self, other: u16) -> u16 {
        (*self as u16) & other
    }

    pub fn __rand__(&self, other: u16) -> u16 {
        other & (*self as u16)
    }

    pub fn __or__(&self, other: u16) -> u16 {
        (*self as u16) | other
    }

    pub fn __ror__(&self, other: u16) -> u16 {
        other | (*self as u16)
    }

    pub fn __xor__(&self, other: u16) -> u16 {
        (*self as u16) ^ other
    }

    pub fn __rxor__(&self, other: u16) -> u16 {
        other ^ (*self as u16)
    }

    pub fn __repr__(&self) -> &'static str {
        WarcRecordType::from(*self).as_str()
    }
}

// ===========================================================
// HeaderMap
// ===========================================================

#[pyclass(name = "HeaderMap", module = "fastwarc.warc")]
pub struct HeaderMapPy {
    inner: HeaderMapBacking,
}

enum HeaderMapBacking {
    Owned(HeaderMap),
    Warc(Arc<Mutex<WarcRecord>>),
    Http(Arc<Mutex<WarcRecord>>),
}

impl HeaderMapPy {
    fn from_warc(record: Arc<Mutex<WarcRecord>>) -> Self {
        Self {
            inner: HeaderMapBacking::Warc(record),
        }
    }

    fn from_http(record: Arc<Mutex<WarcRecord>>) -> Self {
        Self {
            inner: HeaderMapBacking::Http(record),
        }
    }

    fn with_headers<F, R>(&self, closure: F) -> R
    where
        F: FnOnce(&HeaderMap) -> R,
    {
        match &self.inner {
            HeaderMapBacking::Owned(header_map) => closure(header_map),
            HeaderMapBacking::Warc(record) => closure(record.lock().unwrap().headers()),
            HeaderMapBacking::Http(record) => closure(
                record
                    .lock()
                    .unwrap()
                    .http_headers()
                    .expect("No HTTP headers in record"),
            ),
        }
    }

    fn with_headers_mut<F, R>(&mut self, closure: F) -> R
    where
        F: FnOnce(&mut HeaderMap) -> R,
    {
        match &mut self.inner {
            HeaderMapBacking::Owned(header_map) => closure(header_map),
            HeaderMapBacking::Warc(record) => closure(record.lock().unwrap().headers_mut()),
            HeaderMapBacking::Http(record) => closure(
                record
                    .lock()
                    .unwrap()
                    .http_headers_mut()
                    .expect("No HTTP headers in record"),
            ),
        }
    }
}

impl From<HeaderMap> for HeaderMapPy {
    fn from(inner: HeaderMap) -> Self {
        Self {
            inner: HeaderMapBacking::Owned(inner),
        }
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
            inner: HeaderMapBacking::Owned(HeaderMap::new(enc)),
        })
    }

    pub fn __getnewargs__(&self) -> (&'static str,) {
        (self.encoding(),)
    }

    pub fn __getstate__<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, PyBytes>, bool)> {
        self.with_headers(|headers| {
            let mut buf = Vec::with_capacity(headers.len() * 40);
            headers.write(&mut buf)?;
            Ok((PyBytes::new(py, buf.as_ref()), self.status_line(py).is_some()))
        })
    }

    pub fn __setstate__<'py>(&mut self, state: (Bound<'py, PyBytes>, bool)) -> PyResult<()> {
        let (header_bytes, has_status_line) = state;
        let mut headers = HeaderMap::new(match self.encoding() {
            "utf-8" => HeaderEncoding::Unicode,
            "latin1" => HeaderEncoding::Latin1,
            e => return Err(PyValueError::new_err(format!("Unsupported encoding: {}", e))),
        });
        headers.parse(&mut io::Cursor::new(header_bytes.as_bytes().to_vec()), has_status_line)?;
        self.inner = HeaderMapBacking::Owned(headers);
        Ok(())
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        PyTuple::new(
            py,
            [
                self.status_line(py)
                    .unwrap_or_else(|| PyString::new(py, "<no status line>"))
                    .as_any(),
                self.to_tuples(py)?.as_any(),
            ],
        )?
        .call_method0("__repr__")
    }

    pub fn __str__(&self) -> String {
        self.with_headers(|headers| headers.to_string())
    }

    pub fn encoding(&self) -> &'static str {
        match self.with_headers(|h| h.encoding()) {
            HeaderEncoding::Unicode => "utf-8",
            HeaderEncoding::Latin1 => "latin1",
        }
    }

    #[pyo3(signature = (reader, has_status_line=true))]
    pub fn parse(&mut self, reader: Py<PyAny>, has_status_line: bool) -> PyResult<usize> {
        let mut reader = PyReaderAdapter::new(reader)?;
        Ok(self.with_headers_mut(|h| h.parse(&mut reader, has_status_line))?)
    }

    pub fn write(&self, writer: Py<PyAny>) -> PyResult<usize> {
        let mut writer = PyWriterAdapter::new(writer)?;
        Ok(self.with_headers(|h| h.write(&mut writer))?)
    }

    #[getter]
    pub fn reason_phrase<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.with_headers(|h| h.reason_phrase().map(|s| PyString::new(py, s.as_ref())))
    }

    #[getter]
    pub fn status_code(&self) -> Option<u16> {
        self.with_headers(HeaderMap::status_code)
    }

    #[getter]
    pub fn status_line<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.with_headers(|h| h.status_line().map(|s| PyString::new(py, s.as_ref())))
    }

    #[setter]
    pub fn set_status_line(&mut self, status_line: &str) {
        self.with_headers_mut(|h| h.set_status_line(status_line));
    }

    #[getter]
    pub fn status_line_bytes<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyBytes>> {
        self.with_headers(|h| h.status_line_bytes().as_ref().map(|s| PyBytes::new(py, s)))
    }

    #[setter]
    pub fn set_status_line_bytes(&mut self, status_line: &[u8]) {
        self.with_headers_mut(|h| h.set_status_line_bytes(status_line));
    }

    pub fn append(&mut self, key: &str, value: &str) {
        self.with_headers_mut(|h| h.append(key, value));
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        let items = self.with_headers(|h| {
            h.to_map()
                .into_iter()
                .map(|(k, v)| (k.as_ref().to_string(), v))
                .collect::<Vec<_>>()
        });
        for (key, value) in items {
            dict.set_item(key.as_str(), value.as_str())?;
        }
        Ok(dict)
    }

    pub fn asdict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        self.to_dict(py)
    }

    pub fn to_tuples<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        self.items(py)
    }

    pub fn astuples<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        self.to_tuples(py)
    }

    pub fn is_empty(&self) -> bool {
        self.with_headers(HeaderMap::is_empty)
    }

    pub fn clear(&mut self) {
        self.with_headers_mut(HeaderMap::clear);
    }

    #[pyo3(signature = (key, default=None))]
    pub fn get<'py>(
        &self,
        py: Python<'py>,
        key: &str,
        default: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        Ok(self.with_headers(|h| h.get(key).map(|s| s.into_owned())).map_or_else(
            || default.unwrap_or_else(|| py.None().bind(py).clone()),
            |s| PyString::new(py, &s).into_any(),
        ))
    }

    pub fn get_multiple<'py>(&self, py: Python<'py>, key: &str) -> PyResult<Bound<'py, PyTuple>> {
        let values = self.with_headers(|h| {
            h.get_multiple(key)
                .into_iter()
                .map(|s| s.into_owned())
                .collect::<Vec<_>>()
        });
        let items = values.into_iter().map(|s| PyString::new(py, &s));
        PyTuple::new(py, items)
    }

    pub fn get_bytes<'py>(
        &self,
        py: Python<'py>,
        key: &[u8],
        default: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        Ok(self
            .with_headers(|h| h.get_bytes(key).map(|s| s.into_owned()))
            .map_or_else(|| default.unwrap_or_else(|| py.None().bind(py).clone()), |s| PyBytes::new(py, &s).into_any()))
    }

    pub fn get_bytes_multiple<'py>(&self, py: Python<'py>, key: &[u8]) -> PyResult<Bound<'py, PyTuple>> {
        let values = self.with_headers(|h| {
            h.get_bytes_multiple(key)
                .into_iter()
                .map(|s| s.into_owned())
                .collect::<Vec<_>>()
        });
        let items = values.into_iter().map(|s| PyBytes::new(py, &s));
        PyTuple::new(py, items)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.with_headers(|h| h.contains_key(key))
    }

    pub fn contains_key_bytes(&self, key: &[u8]) -> bool {
        self.with_headers(|h| h.contains_key_bytes(key))
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.with_headers_mut(|h| h.set(key, value));
    }

    pub fn set_bytes(&mut self, key: &[u8], value: &[u8]) {
        self.with_headers_mut(|h| h.set_bytes(key, value));
    }

    pub fn append_bytes(&mut self, key: &[u8], value: &[u8]) {
        self.with_headers_mut(|h| h.append_bytes(key, value));
    }

    pub fn remove(&mut self, key: &str) {
        self.with_headers_mut(|h| h.remove(key));
    }

    pub fn remove_bytes(&mut self, key: &[u8]) {
        self.with_headers_mut(|h| h.remove_bytes(key));
    }

    pub fn items<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self.with_headers(|h| {
            h.items()
                .map(|(k, v)| PyTuple::new(py, [k.as_ref(), v.as_ref()]))
                .collect::<PyResult<Vec<_>>>()
        })?;
        PyTuple::new(py, items)
    }

    pub fn items_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items = self.with_headers(|h| {
            h.items_bytes()
                .map(|(k, v)| PyTuple::new(py, [k, v]))
                .collect::<PyResult<Vec<_>>>()
        })?;
        PyTuple::new(py, items)
    }

    pub fn keys<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let keys = self.with_headers(|h| h.keys().map(|k| PyString::new(py, &k)).collect::<Vec<_>>());
        PyTuple::new(py, keys)
    }

    pub fn keys_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let keys = self.with_headers(|h| h.keys_bytes().map(|k| PyBytes::new(py, &k)).collect::<Vec<_>>());
        PyTuple::new(py, keys)
    }

    pub fn values<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let values = self.with_headers(|h| h.values().map(|k| PyString::new(py, &k)).collect::<Vec<_>>());
        PyTuple::new(py, values)
    }

    pub fn values_bytes<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let values = self.with_headers(|h| h.values_bytes().map(|k| PyBytes::new(py, &k)).collect::<Vec<_>>());
        PyTuple::new(py, values)
    }

    pub fn __getitem__<'py>(&self, py: Python<'py>, item: &str) -> PyResult<Bound<'py, PyString>> {
        self.with_headers(|h| h.get(item).map(|s| s.into_owned()))
            .map(|s| PyString::new(py, &s))
            .ok_or_else(|| PyKeyError::new_err(item.to_string()))
    }

    pub fn __iter__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyIterator>> {
        PyIterator::from_object(self.items(py)?.as_any())
    }

    pub fn __len__(&self) -> usize {
        self.with_headers(HeaderMap::len)
    }

    pub fn __setitem__(&mut self, key: &str, value: &str) {
        self.with_headers_mut(|h| h.set(key, value));
    }

    pub fn __contains__(&self, item: &str) -> bool {
        self.with_headers(|h| h.contains_key(item))
    }

    fn __eq__(&self, other: &HeaderMapPy) -> bool {
        self.with_headers(|left| other.with_headers(|right| left == right))
    }

    fn __ne__(&self, other: &HeaderMapPy) -> bool {
        !self.__eq__(other)
    }
}

// ===========================================================
// WarcRecord
// ===========================================================

#[pyclass(name = "WarcRecordPayloadReader", module = "fastwarc.warc", extends = WarcReaderPy)]
struct WarcRecordPayloadReaderPy {
    record: Arc<Mutex<WarcRecord>>,
}

impl WarcRecordPayloadReaderPy {
    fn with_payload_reader<F, T>(&self, closure: F) -> PyResult<T>
    where
        F: FnOnce(&mut LimitedBufReader) -> PyResult<T>,
    {
        let mut record = self.record.lock().unwrap();
        let reader = record
            .reader_mut()
            .ok_or_else(|| PyOSError::new_err("WarcRecord has no active reader"))?;
        closure(reader)
    }
}

#[pymethods]
impl WarcRecordPayloadReaderPy {
    #[pyo3(signature = (size=-1))]
    pub fn read<'py>(&self, py: Python<'py>, size: i128) -> PyResult<Bound<'py, PyBytes>> {
        self.with_payload_reader(|reader| {
            let mut buf;
            let n;
            if size < 0 {
                buf = Vec::with_capacity(2048);
                n = reader.read_to_end(&mut buf)?;
            } else {
                buf = Vec::with_capacity(size as usize);
                n = reader.take(size as u64).read_to_end(&mut buf)?;
            }
            Ok(PyBytes::new(py, &buf[..n]))
        })
    }

    #[pyo3(signature = (max_line_len=8192))]
    pub fn readline<'py>(&self, py: Python<'py>, max_line_len: usize) -> PyResult<Bound<'py, PyBytes>> {
        self.with_payload_reader(|reader| {
            let mut buf = Vec::with_capacity(max_line_len.min(128));
            reader.read_line(&mut buf, max_line_len)?;
            Ok(PyBytes::new(py, &buf))
        })
    }

    #[pyo3(signature = (size=-1))]
    pub fn consume(&self, size: i128) -> PyResult<usize> {
        let mut record = self.record.lock().unwrap();
        if size < 0 {
            Ok(record.consume()?)
        } else {
            Ok(record.consume_n(size as usize)?)
        }
    }

    pub fn tell(&self) -> PyResult<u64> {
        self.with_payload_reader(|reader| Ok(reader.stream_position()?))
    }

    #[pyo3(signature = (offset, whence=0))]
    pub fn seek(&self, offset: i128, whence: u8) -> PyResult<u64> {
        self.with_payload_reader(|reader| Ok(reader.seek(python_whence_to_seekfrom(offset, whence)?)?))
    }

    pub fn close(&self) {}
}

#[pyclass(name = "WarcRecord", module = "fastwarc.warc")]
pub struct WarcRecordPy {
    inner: Arc<Mutex<WarcRecord>>,
}

impl WarcRecordPy {
    fn lock(&self) -> MutexGuard<'_, WarcRecord> {
        self.inner.lock().unwrap()
    }

    fn from_record(record: WarcRecord) -> Self {
        Self {
            inner: Arc::new(Mutex::new(record)),
        }
    }
}

#[pymethods]
impl WarcRecordPy {
    #[new]
    pub fn __new__() -> Self {
        Self::from_record(WarcRecord::new())
    }

    pub fn __getnewargs__<'py>(&self, py: Python<'py>) -> Bound<'py, PyTuple> {
        PyTuple::empty(py)
    }

    pub fn __getstate__(&mut self) -> PyResult<(Vec<u8>, bool)> {
        let mut record = self.lock();
        let http_parsed = record.is_http_parsed();
        let mut serialized = Vec::with_capacity(record.content_length() as usize + 400);
        record.write_with_block_size(&mut serialized, 16384)?;
        Ok((serialized, http_parsed))
    }

    pub fn __setstate__(&mut self, state: (Vec<u8>, bool)) -> PyResult<()> {
        let (serialized, http_parsed) = state;
        let mut record = WarcRecord::from_bytes(serialized)?;
        if http_parsed {
            record.parse_http()?;
        }
        self.inner = Arc::new(Mutex::new(record));
        Ok(())
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        PyString::from_fmt(
            py,
            format_args!(
                "<WarcRecord type='{:?}', id={:?}>",
                self.record_type(),
                self.record_id(py).unwrap_or_else(|| PyString::new(py, "<no_id>"))
            ),
        )
    }

    pub fn __str__(&self) -> String {
        self.inner.with_mut(|r| r.to_string())
    }

    #[getter]
    pub fn record_id<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.lock().record_id().map(|s| PyString::new(py, s.as_ref()))
    }

    #[setter]
    pub fn set_record_id<'py>(&self, record_id: &str) {
        self.lock().set_record_id(record_id);
    }

    #[getter]
    pub fn record_type(&self) -> WarcRecordTypePy {
        self.lock().record_type().into()
    }

    #[setter]
    pub fn set_record_type(&mut self, record_type: WarcRecordTypePy) {
        self.lock().set_record_type(record_type.into());
    }

    #[getter]
    pub fn content_length(&self) -> u64 {
        self.lock().content_length()
    }

    #[getter]
    pub fn record_date<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyDateTime>>> {
        let inner = self.lock();
        let Some(value) = inner.headers().get("WARC-Date") else {
            return Ok(None);
        };
        let datetime_module = py.import("datetime")?;
        let datetime = datetime_module.getattr("datetime")?;
        let normalized = value.replace('Z', "+00:00");
        match datetime.call_method1("fromisoformat", (normalized,)) {
            Ok(obj) => Ok(Some(obj.cast_into::<PyDateTime>()?)),
            Err(_) => Ok(None),
        }
    }

    #[setter]
    pub fn set_record_date<'py>(&mut self, record_date: Bound<'_, PyDateTime>) -> PyResult<()> {
        if record_date.getattr("tzinfo")?.is_none() {
            return Err(PyValueError::new_err("Trying to set naive datetime without timezone info."));
        }
        let formatted: String = record_date.call_method1("isoformat", ())?.extract::<String>()?;
        self.inner
            .lock()
            .unwrap()
            .headers_mut()
            .set("WARC-Date", formatted.replace("+00:00", "Z"));
        Ok(())
    }

    #[getter]
    pub fn headers(&self) -> HeaderMapPy {
        HeaderMapPy::from_warc(self.inner.clone())
    }

    #[getter]
    pub fn is_http(&self) -> bool {
        self.lock().is_http()
    }

    #[setter]
    pub fn set_is_http(&mut self, is_http: bool) {
        self.lock().set_is_http(is_http);
    }

    #[getter]
    pub fn is_http_parsed(&self) -> bool {
        self.lock().is_http_parsed()
    }

    #[getter]
    pub fn http_headers(&self) -> Option<HeaderMapPy> {
        self.lock()
            .http_headers()
            .map(|_| HeaderMapPy::from_http(self.inner.clone()))
    }

    #[getter]
    pub fn http_content_type<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.lock().http_content_type().as_deref().map(|s| PyString::new(py, s))
    }

    #[getter]
    pub fn http_charset<'py>(&self, py: Python<'py>) -> Option<Bound<'py, PyString>> {
        self.lock().http_charset().as_ref().map(|s| PyString::new(py, s))
    }

    #[getter]
    pub fn http_date<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        http_datetime_to_py(py, self.lock().http_headers().and_then(|h| h.get("Date")).as_deref())
    }

    #[getter]
    pub fn http_last_modified<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        http_datetime_to_py(
            py,
            self.lock()
                .http_headers()
                .and_then(|h| h.get("Last-Modified"))
                .as_deref(),
        )
    }

    #[getter]
    pub fn reader<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        if !self.inner.lock().unwrap().reader_mut().is_some() {
            return Ok(py.None().into_bound(py));
        }
        let reader = Py::new(
            py,
            PyClassInitializer::from(WarcReaderPy::__new__()).add_subclass(WarcRecordPayloadReaderPy {
                record: self.inner.clone(),
            }),
        )?;
        Ok(reader.into_bound(py).into_any())
    }

    #[getter]
    pub fn stream_pos(&self) -> u64 {
        self.lock().stream_pos()
    }

    #[getter]
    pub fn is_frozen(&self) -> bool {
        self.lock().is_frozen()
    }

    #[pyo3(signature = (content_length=0, record_type=WarcRecordTypePy::no_type, record_urn=None))]
    pub fn init_headers(&mut self, content_length: u64, record_type: WarcRecordTypePy, record_urn: Option<&[u8]>) {
        self.lock()
            .init_headers(content_length, Some(record_type.into()), record_urn);
    }

    pub fn freeze(&mut self) -> PyResult<bool> {
        self.lock().freeze()?;
        Ok(true)
    }

    pub fn set_bytes_content(&mut self, content: &[u8]) {
        self.set_bytes_payload(content);
    }

    pub fn set_bytes_payload(&mut self, content: &[u8]) {
        self.lock().set_bytes_payload(content.to_vec());
    }

    #[pyo3(signature = (n=None))]
    pub fn consume(&mut self, n: Option<usize>) -> PyResult<usize> {
        match n {
            Some(n) => Ok(self.lock().consume_n(n)?),
            None => Ok(self.lock().consume()?),
        }
    }

    #[pyo3(signature = (quirks_mode=false))]
    pub fn parse_warc_headers(&mut self, quirks_mode: bool) -> PyResult<usize> {
        if quirks_mode {
            Ok(self.lock().parse_warc_headers_quirks(quirks_mode)?)
        } else {
            Ok(self.lock().parse_warc_headers()?)
        }
    }

    #[pyo3(signature = (auto_decode="none"))]
    pub fn parse_http(&mut self, auto_decode: &str) -> PyResult<()> {
        self.lock()
            .parse_http_with_decode_opts(auto_decode_str_to_enum(auto_decode)?)?;
        Ok(())
    }

    #[pyo3(signature = (consume=false))]
    pub fn verify_block_digest(&mut self, consume: bool) -> PyResult<bool> {
        match self.lock().verify_block_digest(consume) {
            Ok(valid) => Ok(valid),
            Err(StreamError(e)) => Err(PyOSError::new_err(e.to_string())),
            Err(_) => Ok(false),
        }
    }

    #[pyo3(signature = (consume=false))]
    pub fn verify_payload_digest(&mut self, consume: bool) -> PyResult<bool> {
        match self.lock().verify_payload_digest(consume) {
            Ok(valid) => Ok(valid),
            Err(StreamError(e)) => Err(PyOSError::new_err(e.to_string())),
            Err(_) => Ok(false),
        }
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
        if let Some(payload_digest) = payload_digest {
            let encoded = py
                .import("base64")?
                .getattr("b32encode")?
                .call1((PyBytes::new(py, payload_digest),))?
                .extract::<Bound<'_, PyBytes>>()?;
            let mut digest_header = b"sha1:".to_vec();
            digest_header.extend_from_slice(encoded.as_bytes());
            self.lock()
                .headers_mut()
                .set_bytes(b"WARC-Payload-Digest", &digest_header);
        }

        // Record correct before stream position for legacy shims.
        // TODO: Remove when shims are removed.
        let raw_stream = stream.getattr("_raw_stream").ok();
        let pos_before = match &raw_stream {
            Some(raw_stream) => raw_stream.call_method0("tell").and_then(|r| r.extract::<usize>()).ok(),
            None => None,
        };

        let mut writer = PyWriterAdapter::new(stream.clone().unbind())?;
        let bytes_written = if checksum_data {
            self.lock()
                .write_with_block_size_checksum(&mut writer, chunk_size, true)?
        } else {
            self.lock().write_with_block_size(&mut writer, chunk_size)?
        };

        // Call legacy shim end_member and calculate bytes written.
        // TODO: Remove when shims are removed.
        if stream.hasattr("end_member")? {
            stream.call_method0("end_member")?;
        }
        Ok(match (pos_before, raw_stream) {
            (Some(pos_before), Some(raw_stream)) => raw_stream
                .call_method0("tell")
                .and_then(|r| r.extract::<usize>())
                .map(|pos_after| pos_after - pos_before)
                .unwrap_or(bytes_written),
            _ => bytes_written,
        })
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

#[pyclass(name = "ArchiveIterator", module = "fastwarc.warc")]
pub struct ArchiveIteratorPy {
    inner: ArchiveIteratorThreadSafe,
    record_types: u16,
    min_content_length: Option<u64>,
    max_content_length: Option<u64>,
    func_filter: Option<Py<PyAny>>,
}

fn auto_decode_str_to_enum(value: &str) -> PyResult<AutoDecode> {
    Ok(match value {
        "none" => AutoDecode::None,
        "transfer" => AutoDecode::TransferEncoding,
        "content" => AutoDecode::ContentEncoding,
        "all" => AutoDecode::All,
        _ => return Err(PyValueError::new_err(format!("Invalid value for auto_decode: '{}'", value))),
    })
}

#[pymethods]
impl ArchiveIteratorPy {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (
        stream,
        record_types=WarcRecordTypePy::any_type as u16,
        parse_http=true,
        min_content_length=-1,
        max_content_length=-1,
        func_filter=None,
        verify_digests=false,
        quirks_mode=false,
        auto_decode="none",
        stream_detect=true,
        fsspec_args=None,
    ))]
    pub fn __new__(
        py: Python<'_>,
        stream: Py<PyAny>,
        record_types: u16,
        parse_http: bool,
        min_content_length: i64,
        max_content_length: i64,
        func_filter: Option<Py<PyAny>>,
        verify_digests: bool,
        quirks_mode: bool,
        auto_decode: &str,
        stream_detect: bool,
        fsspec_args: Option<Py<PyAny>>,
    ) -> PyResult<Self> {
        // Check if fsspec is `False`
        let use_fsspec = fsspec_args
            .as_ref()
            .and_then(|f| f.cast_bound::<PyBool>(py).ok())
            .map(|b| b.is_true())
            .unwrap_or(true);

        let mut iterator;
        if !use_fsspec && let Ok(path) = path_like_to_string(stream.bind(py)) {
            let opts = ArchiveIteratorOptions {
                stream_detect,
                ..ArchiveIteratorOptions::default()
            };
            iterator = ArchiveIteratorThreadSafe::from_path_with_options(path, opts)?;
        } else {
            let reader = wrap_reader_stream(
                py,
                stream,
                fsspec_args,
                |reader| -> io::Result<Box<dyn WarcRead + Send>> { Ok(Box::new(reader)) },
                |path| Ok(Box::new(io::BufReader::new(std::fs::File::open(path)?).into_warc_reader())),
            )?;
            iterator = ArchiveIteratorThreadSafe::new(reader).with_stream_detect(stream_detect);
        }
        iterator = iterator
            .with_quirks_mode(quirks_mode)
            .with_parse_http(parse_http)
            .with_verify_digests(verify_digests)
            .with_decode_http_payload(auto_decode_str_to_enum(auto_decode)?);

        Ok(Self {
            inner: iterator,
            record_types,
            min_content_length: u64::try_from(min_content_length).ok(),
            max_content_length: u64::try_from(max_content_length).ok(),
            func_filter,
        })
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
            let record_ref = record.lock().unwrap();

            // TODO: Proper implementation with filters
            let content_length = record_ref.content_length();
            if !record_ref.record_type().matches_bitmask(self.record_types)
                || self.min_content_length.is_some_and(|min| content_length < min)
                || self.max_content_length.is_some_and(|max| content_length > max)
            {
                continue;
            }
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

// ===========================================================
// ArchiveIterator filter predicates
// ===========================================================

fn apply_filter(record: &WarcRecordPy, predicate: impl FnOnce(&mut WarcRecord) -> bool) -> bool {
    let mut record = record.lock();
    predicate(&mut record)
}

#[pyfunction(name = "is_warc_10")]
pub fn is_warc_10_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::is_warc_10)
}

#[pyfunction(name = "is_warc_11")]
pub fn is_warc_11_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::is_warc_11)
}

#[pyfunction(name = "has_block_digest")]
pub fn has_block_digest_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::has_block_digest)
}

#[pyfunction(name = "has_valid_block_digest")]
pub fn has_valid_block_digest_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::has_valid_block_digest)
}

#[pyfunction(name = "has_payload_digest")]
pub fn has_payload_digest_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::has_payload_digest)
}

#[pyfunction(name = "has_valid_payload_digest")]
pub fn has_valid_payload_digest_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::has_valid_payload_digest)
}

#[pyfunction(name = "is_http")]
pub fn is_http_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::is_http)
}

#[pyfunction(name = "is_concurrent")]
pub fn is_concurrent_py(record: &WarcRecordPy) -> bool {
    apply_filter(record, filter::is_concurrent)
}
