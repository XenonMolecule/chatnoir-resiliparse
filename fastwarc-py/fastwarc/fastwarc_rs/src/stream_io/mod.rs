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

use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use std::io::{self, Read, Seek, SeekFrom, Write};

// ===========================================================
// Submodules
// ===========================================================

pub mod gzip;

// ===========================================================
// Helpers
// ===========================================================

fn io_err_to_py(err: io::Error) -> PyErr {
    PyOSError::new_err(err.to_string())
}

fn py_to_io_err(err: PyErr) -> io::Error {
    io::Error::other(err.to_string())
}

// ===========================================================
// Adapter for Python file-like objects
// ===========================================================

// #[derive(FromPyObject)]
// enum StreamType<'py> {
//     GzipReader(PyRef<'py, stream_io::GzipReader>),
//     GzipWriter(PyRef<'py, stream_io::GzipWriter>),
//     Py(Bound<'py, PyAny>),
// }

struct PyReader {
    inner: Py<PyAny>,
}

impl PyReader {
    fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }

    // noinspection ALL
    fn close(&mut self) -> PyResult<()> {
        Python::attach(|py| {
            let reader = self.inner.bind(py);
            match reader.call_method0("close") {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }
}

impl Read for PyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Python::attach(|py| {
            let data = self
                .inner
                .bind(py)
                .call_method1("read", (buf.len(),))
                .map_err(py_to_io_err)?;
            let data = data.extract::<Vec<u8>>().map_err(py_to_io_err)?;
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
            }
            .map_err(py_to_io_err)?;

            result.extract::<u64>().map_err(py_to_io_err)
        })
    }
}

struct PyWriter {
    inner: Py<PyAny>,
}

impl PyWriter {
    fn new(inner: Py<PyAny>) -> Self {
        Self { inner }
    }

    // noinspection ALL
    fn close(&mut self) -> PyResult<()> {
        Python::attach(|py| {
            let writer = self.inner.bind(py);
            match writer.call_method0("close") {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }
        })
    }
}

impl Write for PyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Python::attach(|py| {
            self.inner
                .bind(py)
                .call_method1("write", (buf,))
                .and_then(|result| result.extract::<usize>())
                .map_err(py_to_io_err)
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Python::attach(|py| {
            self.inner
                .bind(py)
                .call_method0("flush")
                .map(|_| ())
                .map_err(py_to_io_err)
        })
    }
}
