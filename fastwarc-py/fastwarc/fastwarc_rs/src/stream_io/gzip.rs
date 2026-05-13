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
use crate::stream_io::{CompressingStreamPy, DecompressingStreamPy, PyReader, PyWriter};
use fastwarc::stream_io::gzip;
use fastwarc::stream_io::{CompressingStream, DecompressingStream};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

#[pyclass(name = "GzipReader", extends = DecompressingStreamPy, subclass)]
pub struct GzipReaderPy {
    inner: Mutex<Option<gzip::GzipReader<PyReader>>>,
}

// noinspection DuplicatedCode
#[pymethods]
impl GzipReaderPy {
    #[new]
    #[pyo3(signature = (inner, buffer_size=4096, zlib=false))]
    pub fn __new__(inner: Py<PyAny>, buffer_size: usize, zlib: bool) -> (Self, DecompressingStreamPy) {
        let options = gzip::GzipReaderOptions {
            capacity: buffer_size,
            window_bits: if zlib { 15 } else { 15 + 16 },
            expect_header: true,
        };
        let slf = Self {
            inner: Mutex::new(Some(gzip::GzipReader::with_options(PyReader::new(inner), options))),
        };
        (slf, DecompressingStreamPy::default())
    }

    #[pyo3(signature = (size=-1))]
    pub fn read(&self, py: Python<'_>, size: i128) -> PyResult<Py<PyBytes>> {
        impl_reader_read!(self, py, size)
    }

    #[pyo3(signature = (offset, whence=0))]
    pub fn seek(&self, offset: u64, whence: u8) -> PyResult<u64> {
        impl_reader_seek!(self, offset, whence, seek)
    }

    #[pyo3(signature = (offset, whence=0))]
    pub fn inner_seek(&self, offset: u64, whence: u8) -> PyResult<u64> {
        impl_reader_seek!(self, offset, whence, inner_seek)
    }

    pub fn tell(&self) -> PyResult<u64> {
        forward_fn_call!(self, stream_position)
    }

    pub fn inner_tell(&self) -> PyResult<u64> {
        forward_fn_call!(self, inner_stream_position)
    }

    pub fn member_start_position(&mut self) -> io::Result<u64> {
        forward_fn_call!(self, member_start_position)
    }

    pub fn close(&self) -> PyResult<()> {
        impl_reader_close!(self)
    }
}

#[pyclass(name = "GzipWriter", extends = CompressingStreamPy, subclass)]
pub struct GzipWriterPy {
    inner: Mutex<Option<gzip::GzipWriter<PyWriter>>>,
}

// noinspection DuplicatedCode
#[pymethods]
impl GzipWriterPy {
    #[new]
    #[pyo3(signature = (inner, compression_level=9, buffer_size=8192, zlib=false))]
    pub fn __new__(
        inner: Py<PyAny>,
        compression_level: i32,
        buffer_size: usize,
        zlib: bool,
    ) -> (Self, CompressingStreamPy) {
        let options = gzip::GzipWriterOptions {
            capacity: buffer_size,
            window_bits: if zlib { 15 } else { 15 + 16 },
            expect_header: true,
            compression_level,
        };
        let slf = Self {
            inner: Mutex::new(Some(gzip::GzipWriter::with_options(PyWriter::new(inner), options))),
        };
        (slf, CompressingStreamPy::default())
    }

    pub fn write(&self, data: &[u8]) -> PyResult<usize> {
        impl_writer_write!(self, data)
    }

    pub fn flush(&self) -> PyResult<()> {
        forward_fn_call!(self, flush)
    }

    pub fn finish(&self) -> PyResult<()> {
        forward_fn_call!(self, finish)
    }

    pub fn close(&self) -> PyResult<()> {
        impl_writer_close!(self)
    }
}
