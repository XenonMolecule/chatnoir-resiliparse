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

use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io::{self, Read, Seek, SeekFrom, Write};

// ===========================================================
// Submodules
// ===========================================================

pub mod gzip;

// ===========================================================
// Exported stream parent classes
// ===========================================================

#[pyclass(name = "DecompressingStream", subclass)]
#[derive(Default)]
pub struct DecompressingStreamPy {}

#[pymethods]
impl DecompressingStreamPy {
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

#[pyclass(name = "CompressingStream", subclass)]
#[derive(Default)]
pub struct CompressingStreamPy {}

#[pymethods]
impl CompressingStreamPy {
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

    pub fn finish(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
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

// ===========================================================
// Adapter for Python file-like objects
// ===========================================================

// #[derive(FromPyObject)]
// enum ReaderType<'py> {
//     GzipReader(PyRef<'py, gzip::GzipReader>),
//     Py(Bound<'py, PyAny>),
// }
//
// #[derive(FromPyObject)]
// enum WriterType<'py> {
//     GzipWriter(PyRef<'py, gzip::GzipWriter>),
//     Py(Bound<'py, PyAny>),
// }

struct PyReader {
    inner: Py<PyAny>,
}

impl PyReader {
    fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }
}

// SAFETY: All interaction with `PyReader::inner` must go through `Python::attach`.
unsafe impl Send for PyReader {}

impl Read for PyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let data = self.inner.bind(py).call_method1("read", (buf.len(),))?;
            let data = data.extract::<Vec<u8>>()?;
            let len = data.len().min(buf.len());
            buf[..len].copy_from_slice(&data[..len]);
            Ok(len)
        })
    }
}

impl Seek for PyReader {
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

pub(crate) struct PyWriter {
    inner: Py<PyAny>,
}

impl PyWriter {
    fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }
}

// SAFETY: All interaction with `PyWriter::inner` must go through `Python::attach`.
unsafe impl Send for PyWriter {}

impl Write for PyWriter {
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
// Helper macros for redundant Reader/Writer implementations
// ===========================================================

pub(crate) mod impl_macros {
    macro_rules! impl_reader_read {
        ($self: ident, $py: ident, $size: ident) => {{
            let mut reader = $self.inner.lock().unwrap();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            let mut buf = vec![0; $size];
            let len = reader.read(&mut buf)?;
            Ok(PyBytes::new($py, &buf[..len]).unbind())
        }};
    }
    pub(crate) use impl_reader_read;

    macro_rules! impl_reader_seek {
        ($self: ident, $offset: ident) => {{
            let mut reader = $self.inner.lock().unwrap();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            Ok(reader.seek(SeekFrom::Start($offset))?)
        }};
    }
    pub(crate) use impl_reader_seek;

    macro_rules! impl_reader_tell {
        ($self: ident) => {{
            let mut reader = $self.inner.lock().unwrap();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            Ok(reader.stream_position()?)
        }};
    }
    pub(crate) use impl_reader_tell;

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

    macro_rules! impl_writer_flush {
        ($self: ident) => {{
            let mut writer = $self.inner.lock().unwrap();
            let writer = writer
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            Ok(writer.flush()?)
        }};
    }
    pub(crate) use impl_writer_flush;

    macro_rules! impl_writer_finish {
        ($self: ident) => {{
            let mut writer = $self.inner.lock().unwrap();
            let writer = writer
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            Ok(writer.finish()?)
        }};
    }
    pub(crate) use impl_writer_finish;

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
