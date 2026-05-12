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

use super::impl_macros::*;
use crate::stream_io::{PyReader, PyWriter};
use fastwarc::stream_io::CompressingStream;
use fastwarc::stream_io::gzip;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

#[pyclass]
pub struct GzipReader {
    inner: Mutex<Option<gzip::GzipReader<PyReader>>>,
}

#[pymethods]
impl GzipReader {
    #[new]
    #[pyo3(signature = (raw_stream, buffer_size=4096))]
    pub fn __new_(raw_stream: Py<PyAny>, buffer_size: usize) -> Self {
        Self {
            inner: Mutex::new(Some(gzip::GzipReader::with_capacity(buffer_size, PyReader::new(raw_stream)))),
        }
    }

    pub fn read(&self, py: Python<'_>, size: usize) -> PyResult<Py<PyBytes>> {
        impl_reader_read!(self, py, size)
    }

    pub fn seek(&self, offset: u64) -> PyResult<u64> {
        impl_reader_seek!(self, offset)
    }

    pub fn tell(&self) -> PyResult<u64> {
        impl_reader_tell!(self)
    }

    pub fn close(&self) -> PyResult<()> {
        impl_reader_close!(self)
    }
}

#[pyclass]
pub struct GzipWriter {
    inner: Mutex<Option<gzip::GzipWriter<PyWriter>>>,
}

#[pymethods]
impl GzipWriter {
    #[new]
    #[pyo3(signature = (raw_stream, compression_level=9, buffer_size=8192))]
    pub fn __new__(raw_stream: Py<PyAny>, compression_level: i32, buffer_size: usize) -> Self {
        Self {
            inner: Mutex::new(Some(gzip::GzipWriter::with_capacity_comp_level(
                buffer_size,
                PyWriter::new(raw_stream),
                compression_level,
            ))),
        }
    }

    pub fn write(&self, data: &[u8]) -> PyResult<usize> {
        impl_writer_write!(self, data)
    }

    pub fn flush(&self) -> PyResult<()> {
        impl_writer_flush!(self)
    }

    pub fn finish(&self) -> PyResult<()> {
        impl_writer_finish!(self)
    }

    pub fn close(&self) -> PyResult<()> {
        impl_writer_close!(self)
    }
}
