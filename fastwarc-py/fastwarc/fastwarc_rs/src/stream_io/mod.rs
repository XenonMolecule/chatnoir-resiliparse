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
use std::io::{self, Read, Seek, SeekFrom, Write};

// ===========================================================
// Submodules
// ===========================================================

pub mod gzip;

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
