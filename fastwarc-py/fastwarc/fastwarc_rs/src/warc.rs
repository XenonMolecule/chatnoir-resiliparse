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
use fastwarc::record::{HeaderEncoding, HeaderMap};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyIterator, PyString, PyTuple};
use std::io::BufReader;

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

#[pyclass(name = "HeaderMap")]
#[derive(Default)]
pub struct HeaderMapPy {
    inner: HeaderMap,
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
    pub fn reason_phrase(&self) -> Option<String> {
        self.inner.reason_phrase().map(|s| s.to_string())
    }

    #[getter]
    pub fn status_code(&self) -> Option<u16> {
        self.inner.status_code()
    }

    #[getter]
    pub fn status_line(&self) -> Option<String> {
        self.inner.status_line().map(|s| s.to_string())
    }

    #[setter]
    pub fn set_status_line(&mut self, status_line: &str) {
        self.inner.set_status_line(status_line);
    }

    #[getter]
    pub fn status_line_bytes(&self, py: Python<'_>) -> Option<Py<PyBytes>> {
        self.inner.status_line_bytes().map(|s| PyBytes::new(py, s).unbind())
    }

    #[setter]
    pub fn set_status_line_bytes(&mut self, status_line: &[u8]) {
        self.inner.set_status_line_bytes(status_line);
    }

    pub fn append(&mut self, key: &str, value: &str) {
        self.inner.append(key, value);
    }

    pub fn asdict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        for (key, value) in self.inner.to_map() {
            dict.set_item(key.as_ref(), value)?;
        }
        Ok(dict.unbind())
    }

    pub fn astuples(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        self.items(py)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[pyo3(signature = (key, default=None))]
    pub fn get(&self, py: Python<'_>, key: &str, default: Option<Py<PyAny>>) -> PyResult<Py<PyAny>> {
        self.__getitem__(py, key)
            .map(|v| v.into_any())
            .or_else(|_| Ok(default.unwrap_or_else(|| py.None())))
    }

    pub fn get_multiple(&self, py: Python<'_>, key: &str) -> PyResult<Py<PyTuple>> {
        let items = self
            .inner
            .get_multiple(key)
            .into_iter()
            .map(|s| PyString::new(py, s.as_ref()));
        Ok(PyTuple::new(py, items)?.unbind())
    }

    pub fn get_bytes(&self, py: Python<'_>, key: &[u8], default: Option<Py<PyAny>>) -> PyResult<Py<PyAny>> {
        Ok(self
            .inner
            .get_bytes(key)
            .map_or_else(|| default.unwrap_or_else(|| py.None()), |s| PyBytes::new(py, &s).into_any().unbind()))
    }

    pub fn get_bytes_multiple(&self, py: Python<'_>, key: &[u8]) -> PyResult<Py<PyTuple>> {
        let items = self
            .inner
            .get_bytes_multiple(key)
            .into_iter()
            .map(|s| PyBytes::new(py, s.as_ref()));
        Ok(PyTuple::new(py, items)?.unbind())
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

    pub fn items(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let items = self
            .inner
            .items()
            .map(|(k, v)| PyTuple::new(py, [k.as_ref(), v.as_ref()]))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyTuple::new(py, items)?.unbind())
    }

    pub fn items_bytes(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let items = self
            .inner
            .items_bytes()
            .map(|(k, v)| PyTuple::new(py, [k, v]))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyTuple::new(py, items)?.unbind())
    }

    pub fn keys(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let items = self
            .inner
            .keys()
            .map(|k| PyString::new(py, k.as_ref()))
            .collect::<Vec<_>>();
        Ok(PyTuple::new(py, items)?.unbind())
    }

    pub fn keys_bytes(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let items = self.inner.keys_bytes().map(|k| PyBytes::new(py, k)).collect::<Vec<_>>();
        Ok(PyTuple::new(py, items)?.unbind())
    }

    pub fn values(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let items = self
            .inner
            .values()
            .map(|k| PyString::new(py, k.as_ref()))
            .collect::<Vec<_>>();
        Ok(PyTuple::new(py, items)?.unbind())
    }

    pub fn values_bytes(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let items = self
            .inner
            .values_bytes()
            .map(|k| PyBytes::new(py, k))
            .collect::<Vec<_>>();
        Ok(PyTuple::new(py, items)?.unbind())
    }

    pub fn __getitem__(&self, py: Python<'_>, item: &str) -> PyResult<Py<PyString>> {
        self.inner
            .get(item)
            .map(|s| PyString::new(py, &s).unbind())
            .ok_or_else(|| PyKeyError::new_err(format!("Unknown key {}", item)))
    }

    pub fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyIterator>> {
        let iter = PyIterator::from_object(self.items(py)?.bind(py))?;
        Ok(iter.unbind())
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
