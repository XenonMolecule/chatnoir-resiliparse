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

use crate::stream_io::{PyReader, PyWriter, io_err_to_py};
use fastwarc::stream_io::CompressingStream;
use fastwarc::stream_io::gzip::{GzipReader as RustGzipReader, GzipWriter as RustGzipWriter};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

#[pyclass]
pub struct GzipReader {
    inner: Mutex<Option<RustGzipReader<PyReader>>>,
}

#[pymethods]
impl GzipReader {
    #[new]
    #[pyo3(signature = (raw_stream, buffer_size=4096))]
    fn __new__(raw_stream: Py<PyAny>, buffer_size: usize) -> Self {
        Self {
            inner: Mutex::new(Some(RustGzipReader::with_capacity(buffer_size, PyReader::new(raw_stream)))),
        }
    }

    fn read(&self, py: Python<'_>, size: usize) -> PyResult<Py<PyAny>> {
        let mut reader = self.inner.lock().unwrap();
        let reader = reader
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
        let mut buf = vec![0; size];
        let len = reader.read(&mut buf).map_err(io_err_to_py)?;
        Ok(pyo3::types::PyBytes::new(py, &buf[..len]).into_any().unbind())
    }

    fn seek(&self, offset: u64) -> PyResult<u64> {
        let mut reader = self.inner.lock().unwrap();
        let reader = reader
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
        reader.seek(SeekFrom::Start(offset)).map_err(io_err_to_py)
    }

    fn tell(&self) -> PyResult<u64> {
        let mut reader = self.inner.lock().unwrap();
        let reader = reader
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
        reader.stream_position().map_err(io_err_to_py)
    }

    fn close(&self) -> PyResult<()> {
        let mut reader = self.inner.lock().unwrap();
        if let Some(reader) = reader.take() {
            let mut inner = reader.into_inner();
            inner.close()?;
        }
        Ok(())
    }
}

#[pyclass]
pub struct GzipWriter {
    inner: Mutex<Option<RustGzipWriter<PyWriter>>>,
}

#[pymethods]
impl GzipWriter {
    #[new]
    #[pyo3(signature = (raw_stream, compression_level=9, buffer_size=8192))]
    fn __new__(raw_stream: Py<PyAny>, compression_level: i32, buffer_size: usize) -> Self {
        Self {
            inner: Mutex::new(Some(RustGzipWriter::with_capacity_comp_level(
                buffer_size,
                PyWriter::new(raw_stream),
                compression_level,
            ))),
        }
    }

    fn write(&self, data: &[u8]) -> PyResult<usize> {
        let mut writer = self.inner.lock().unwrap();
        let writer = writer
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
        writer.write(data).map_err(io_err_to_py)
    }

    fn flush(&self) -> PyResult<()> {
        let mut writer = self.inner.lock().unwrap();
        let writer = writer
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
        writer.flush().map_err(io_err_to_py)
    }

    fn finish(&self) -> PyResult<()> {
        let mut writer = self.inner.lock().unwrap();
        let writer = writer
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Trying I/O on closed file."))?;
        writer.finish().map_err(io_err_to_py)
    }

    fn close(&self) -> PyResult<()> {
        let mut writer = self.inner.lock().unwrap();
        if let Some(writer) = writer.take() {
            let mut inner = writer.into_inner().map_err(io_err_to_py)?;
            inner.close()?;
        }
        Ok(())
    }
}
