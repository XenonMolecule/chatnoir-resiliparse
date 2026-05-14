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

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};
use std::io::{self, Read, Seek, SeekFrom, Write};

// ===========================================================
// Submodules
// ===========================================================

pub mod gzip;
pub mod lz4;

// ===========================================================
// Exported stream parent classes
// ===========================================================

#[pyclass(name = "Reader", subclass)]
pub struct ReaderPy {}

#[pymethods]
impl ReaderPy {
    #[new]
    pub fn __new__() -> Self {
        Self {}
    }

    pub fn read(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }

    pub fn seek(&self, py: Python<'_>, offset: u64) -> PyResult<Py<PyAny>> {
        let _ = offset;
        Ok(py.NotImplemented())
    }

    pub fn tell(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }

    pub fn close(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }

    pub fn __enter__(slf: PyRef<'_, Self>) -> PyResult<PyRef<'_, Self>> {
        Ok(slf)
    }

    // noinspection DuplicatedCode
    pub fn __exit__(
        &self,
        py: Python<'_>,
        _exc_type: Option<Py<PyAny>>,
        _exc_val: Option<Py<PyAny>>,
        _exc_tb: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        self.close(py)?;
        Ok(())
    }
}

#[pyclass(name = "DecompressingReader", extends = ReaderPy, subclass)]
pub struct DecompressingReaderPy {}

#[pymethods]
impl DecompressingReaderPy {
    #[new]
    pub fn __new__() -> (Self, ReaderPy) {
        (Self {}, ReaderPy::__new__())
    }

    pub fn inner_seek(&self, py: Python<'_>, offset: u64) -> PyResult<Py<PyAny>> {
        let _ = offset;
        Ok(py.NotImplemented())
    }

    pub fn inner_tell(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }

    pub fn member_start_position(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }
}

#[pyclass(name = "Writer", subclass)]
pub struct WriterPy {}

#[pymethods]
impl WriterPy {
    #[new]
    pub fn __new__() -> Self {
        Self {}
    }

    pub fn write(&self, py: Python<'_>, data: Py<PyBytes>) -> PyResult<Py<PyAny>> {
        let _ = data;
        Ok(py.NotImplemented())
    }

    pub fn flush(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }

    pub fn close(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }

    pub fn __enter__(slf: PyRef<'_, Self>) -> PyResult<PyRef<'_, Self>> {
        Ok(slf)
    }

    // noinspection DuplicatedCode
    pub fn __exit__(
        &self,
        py: Python<'_>,
        _exc_type: Option<Py<PyAny>>,
        _exc_val: Option<Py<PyAny>>,
        _exc_tb: Option<Py<PyAny>>,
    ) -> PyResult<()> {
        self.close(py)?;
        Ok(())
    }
}

#[pyclass(name = "CompressingWriter", extends = WriterPy, subclass)]
pub struct CompressingWriterPy {}

#[pymethods]
impl CompressingWriterPy {
    #[new]
    pub fn __new__() -> (Self, WriterPy) {
        (Self {}, WriterPy::__new__())
    }

    pub fn finish(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(py.NotImplemented())
    }
}

// ===========================================================
// Adapters for Python file-like objects
// ===========================================================

pub(crate) struct PyReaderAdapter {
    inner: Py<PyAny>,
}

#[allow(unused)]
impl PyReaderAdapter {
    pub fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }
}

// SAFETY: All interaction with `PyReader::inner` must go through `Python::attach`.
unsafe impl Send for PyReaderAdapter {}

impl Read for PyReaderAdapter {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let bound = self.inner.bind(py).call_method1("read", (buf.len(),))?;
            let data = bound
                .cast::<PyBytes>()
                .map_err(|_| PyTypeError::new_err("read() must return bytes"))?
                .as_bytes();
            let len = data.len().min(buf.len());
            buf[..len].copy_from_slice(&data[..len]);
            Ok(len)
        })
    }
}

impl Seek for PyReaderAdapter {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        Python::attach(|py| {
            let stream = self.inner.bind(py);
            let result = match pos {
                SeekFrom::Start(offset) => stream.call_method1("seek", (offset, 0)),
                SeekFrom::Current(offset) => stream.call_method1("seek", (offset, 1)),
                SeekFrom::End(offset) => stream.call_method1("seek", (offset, 2)),
            }?;
            Ok(result.extract::<u64>()?)
        })
    }
}

pub(crate) struct PyWriterAdapter {
    inner: Py<PyAny>,
}

#[allow(unused)]
impl PyWriterAdapter {
    pub fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }
}

// SAFETY: All interaction with `PyWriter::inner` must go through `Python::attach`.
unsafe impl Send for PyWriterAdapter {}

impl Write for PyWriterAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Python::attach(|py| {
            Ok(self
                .inner
                .bind(py)
                .call_method1("write", (buf,))
                .and_then(|result| result.extract::<usize>())?)
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Python::attach(|py| Ok(self.inner.bind(py).call_method0("flush").map(|_| ())?))
    }
}

// ===========================================================
// Helper functions
// ===========================================================

/// Convert a path-like object into a String.
pub(crate) fn path_like_to_string(obj: &Bound<'_, PyAny>) -> PyResult<String> {
    if let Ok(s) = obj.cast::<PyString>() {
        return Ok(s.to_str()?.to_owned());
    }

    let os_fspath = obj.py().import("os")?.getattr("fspath")?;
    os_fspath.call1((obj,))?.extract()
}

// ===========================================================
// Helper macros for redundant Reader/Writer implementations
// ===========================================================

pub(crate) mod impl_macros {
    macro_rules! impl_reader_read {
        ($self: ident, $py: ident, $size: ident) => {{
            let mut reader = $self.inner.lock().unwrap();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            let mut buf;
            let n;
            if $size < 0 {
                buf = Vec::new();
                n = reader.read_to_end(&mut buf)?;
            } else {
                buf = vec![0; $size as usize];
                n = reader.read(&mut buf)?;
            }
            Ok(PyBytes::new($py, &buf[..n]))
        }};
    }

    pub(crate) use impl_reader_read;

    macro_rules! impl_reader_seek {
        ($self: ident, $offset: ident, $whence: ident, $seek_fn_name: ident) => {{
            let mut reader = $self.inner.lock().unwrap();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            match $whence {
                0 => Ok(reader.$seek_fn_name(SeekFrom::Start(
                    u64::try_from($offset).map_err(|_| PyValueError::new_err("Seek offset out of range."))?,
                ))?),
                1 => Ok(reader.$seek_fn_name(SeekFrom::Current(
                    i64::try_from($offset).map_err(|_| PyValueError::new_err("Seek offset out of range."))?,
                ))?),
                2 => Ok(reader.$seek_fn_name(SeekFrom::End(
                    i64::try_from($offset).map_err(|_| PyValueError::new_err("Seek offset out of range."))?,
                ))?),
                _ => Err(PyValueError::new_err("Invalid value for `whence` argument. Must be 0, 1, or 2.")),
            }
        }};
    }
    pub(crate) use impl_reader_seek;

    macro_rules! forward_fn_call {
        ($self: ident, $fn_name: ident) => {{
            let mut reader = $self.inner.lock().unwrap();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            Ok(reader.$fn_name()?)
        }};
    }
    pub(crate) use forward_fn_call;

    macro_rules! impl_reader_close {
        ($self: ident) => {{
            let mut reader = $self.inner.lock().unwrap();
            reader.take();
            Ok(())
        }};
    }
    pub(crate) use impl_reader_close;

    macro_rules! impl_writer_write {
        ($self: ident, $data: ident) => {{
            let mut writer = $self.inner.lock().unwrap();
            let writer = writer
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            Ok(writer.write($data)?)
        }};
    }
    pub(crate) use impl_writer_write;

    macro_rules! impl_writer_close {
        ($self: ident) => {{
            let mut writer = $self.inner.lock().unwrap();
            if let Some(w) = writer.as_mut() {
                w.flush()?;
            }
            writer.take();
            Ok(())
        }};
    }
    pub(crate) use impl_writer_close;
}
