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
use fastwarc::stream_io::lz4;
use fastwarc::stream_io::{CompressingStream, DecompressingStream};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io;
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

#[pyclass(name = "Lz4Reader", extends = DecompressingStreamPy, subclass)]
pub struct Lz4ReaderPy {
    inner: Mutex<Option<lz4::Lz4Reader<PyReader>>>,
}

// noinspection DuplicatedCode
#[pymethods]
impl Lz4ReaderPy {
    #[new]
    #[pyo3(signature = (inner, buffer_size=4096))]
    pub fn __new__(inner: Py<PyAny>, buffer_size: usize) -> (Self, DecompressingStreamPy) {
        (
            Self {
                inner: Mutex::new(Some(lz4::Lz4Reader::with_capacity(PyReader::new(inner), buffer_size))),
            },
            DecompressingStreamPy::default(),
        )
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

#[pyclass(name = "Lz4Writer", extends = CompressingStreamPy, subclass)]
pub struct Lz4WriterPy {
    inner: Mutex<Option<lz4::Lz4Writer<PyWriter>>>,
}

// noinspection DuplicatedCode
#[pymethods]
impl Lz4WriterPy {
    #[new]
    #[pyo3(signature = (inner, buffer_size=8192))]
    pub fn __new__(inner: Py<PyAny>, buffer_size: usize) -> (Self, CompressingStreamPy) {
        (
            Self {
                inner: Mutex::new(Some(lz4::Lz4Writer::with_capacity(PyWriter::new(inner), buffer_size))),
            },
            CompressingStreamPy::default(),
        )
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
