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
use std::cell::RefCell;
use std::io::{self, Read, Seek, SeekFrom, Write};

use fastwarc::stream_io::CompressingStream as _;
use fastwarc::stream_io::gzip::{GzipReader as RustGzipReader, GzipWriter as RustGzipWriter};
use pyo3::exceptions::{PyOSError, PyValueError};

fn io_to_py(err: io::Error) -> PyErr {
    PyOSError::new_err(err.to_string())
}

fn py_to_io(err: PyErr) -> io::Error {
    io::Error::other(err.to_string())
}

struct PyStream {
    inner: Py<PyAny>,
}

impl PyStream {
    fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }

    fn close(&mut self) -> PyResult<()> {
        Python::attach(|py| {
            let stream = self.inner.bind(py);
            match stream.call_method0("close") {
                Ok(_) => Ok(()),
                Err(err) if err.is_instance_of::<PyValueError>(py) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }
}

impl Read for PyStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let data = self
                .inner
                .bind(py)
                .call_method1("read", (buf.len(),))
                .map_err(py_to_io)?;
            let data = data.extract::<Vec<u8>>().map_err(py_to_io)?;
            let len = data.len().min(buf.len());
            buf[..len].copy_from_slice(&data[..len]);
            Ok(len)
        })
    }
}

impl Write for PyStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Python::attach(|py| {
            self.inner
                .bind(py)
                .call_method1("write", (buf,))
                .and_then(|result| result.extract::<usize>())
                .map_err(py_to_io)
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Python::attach(|py| self.inner.bind(py).call_method0("flush").map(|_| ()).map_err(py_to_io))
    }
}

impl Seek for PyStream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        Python::attach(|py| {
            let stream = self.inner.bind(py);
            let result = match pos {
                SeekFrom::Start(offset) => stream.call_method1("seek", (offset, 0)),
                SeekFrom::End(offset) => stream.call_method1("seek", (offset, 2)),
                SeekFrom::Current(offset) => stream.call_method1("seek", (offset, 1)),
            }
            .map_err(py_to_io)?;

            result.extract::<u64>().map_err(py_to_io)
        })
    }
}

#[pymodule]
pub mod stream_io {
    use super::*;

    #[pyclass(unsendable)]
    pub struct GzipReader {
        inner: RefCell<Option<RustGzipReader<PyStream>>>,
    }

    #[pymethods]
    impl GzipReader {
        #[new]
        #[pyo3(signature = (raw_stream, buffer_size=4096))]
        fn __new__(raw_stream: Py<PyAny>, buffer_size: usize) -> Self {
            Self {
                inner: RefCell::new(Some(RustGzipReader::with_capacity(buffer_size, PyStream::new(raw_stream)))),
            }
        }

        fn read(&self, py: Python<'_>, size: usize) -> PyResult<Py<PyAny>> {
            let mut reader = self.inner.borrow_mut();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            let mut buf = vec![0; size];
            let len = reader.read(&mut buf).map_err(io_to_py)?;
            Ok(pyo3::types::PyBytes::new(py, &buf[..len]).into_any().unbind())
        }

        fn seek(&self, offset: u64) -> PyResult<u64> {
            let mut reader = self.inner.borrow_mut();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            reader.seek(SeekFrom::Start(offset)).map_err(io_to_py)
        }

        fn tell(&self) -> PyResult<u64> {
            let mut reader = self.inner.borrow_mut();
            let reader = reader
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            reader.stream_position().map_err(io_to_py)
        }

        fn close(&self) -> PyResult<()> {
            let mut reader = self.inner.borrow_mut();
            if let Some(reader) = reader.take() {
                let mut inner = reader.into_inner();
                inner.close()?;
            }
            Ok(())
        }
    }

    #[pyclass(unsendable)]
    pub struct GzipWriter {
        inner: RefCell<Option<RustGzipWriter<PyStream>>>,
    }

    #[pymethods]
    impl GzipWriter {
        #[new]
        #[pyo3(signature = (raw_stream, compression_level=9, buffer_size=8192))]
        fn __new__(raw_stream: Py<PyAny>, compression_level: i32, buffer_size: usize) -> Self {
            Self {
                inner: RefCell::new(Some(RustGzipWriter::with_capacity_comp_level(
                    buffer_size,
                    PyStream::new(raw_stream),
                    compression_level,
                ))),
            }
        }

        fn write(&self, data: &[u8]) -> PyResult<usize> {
            let mut writer = self.inner.borrow_mut();
            let writer = writer
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            writer.write(data).map_err(io_to_py)
        }

        fn flush(&self) -> PyResult<()> {
            let mut writer = self.inner.borrow_mut();
            let writer = writer
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            writer.flush().map_err(io_to_py)
        }

        fn finish(&self) -> PyResult<()> {
            let mut writer = self.inner.borrow_mut();
            let writer = writer
                .as_mut()
                .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
            writer.finish().map_err(io_to_py)
        }

        fn close(&self) -> PyResult<()> {
            let mut writer = self.inner.borrow_mut();
            if let Some(writer) = writer.take() {
                let mut inner = writer.into_inner().map_err(io_to_py)?;
                inner.close()?;
            }
            Ok(())
        }
    }
}
