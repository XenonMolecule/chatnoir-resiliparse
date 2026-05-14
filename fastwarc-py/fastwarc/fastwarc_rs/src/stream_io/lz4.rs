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
use crate::stream_io::{
    CompressingWriterPy, DecompressingReaderPy, PyReaderAdapter, PyWriterAdapter, path_like_to_string,
};
use fastwarc::stream_io::{CompressingWriter, DecompressingReader, lz4};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::sync::Mutex;

#[pyclass(name = "Lz4Reader", extends = DecompressingReaderPy, subclass)]
pub struct Lz4ReaderPy {
    inner: Mutex<Option<Box<dyn DecompressingReader + Send>>>,
}

// noinspection DuplicatedCode
#[pymethods]
impl Lz4ReaderPy {
    #[new]
    #[pyo3(signature = (inner, buffer_size=4096))]
    pub fn __new__(py: Python<'_>, inner: Py<PyAny>, buffer_size: usize) -> PyResult<PyClassInitializer<Self>> {
        let options = lz4::Lz4ReaderOptions { capacity: buffer_size };
        let inner: Box<dyn DecompressingReader + Send> = if let Ok(p) = path_like_to_string(inner.bind(py)) {
            Box::new(lz4::Lz4Reader::from_path_with_options(p, options)?)
        } else {
            Box::new(lz4::Lz4Reader::with_options(PyReaderAdapter::new(inner), options))
        };
        Ok(PyClassInitializer::from(DecompressingReaderPy::__new__()).add_subclass(Self {
            inner: Mutex::new(Some(inner)),
        }))
    }

    #[pyo3(signature = (size=-1))]
    pub fn read<'py>(&self, py: Python<'py>, size: i128) -> PyResult<Bound<'py, PyBytes>> {
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

#[pyclass(name = "Lz4Writer", extends = CompressingWriterPy, subclass)]
pub struct Lz4WriterPy {
    inner: Mutex<Option<Box<dyn CompressingWriter + Send>>>,
}

// noinspection DuplicatedCode
#[pymethods]
impl Lz4WriterPy {
    #[new]
    #[pyo3(signature = (inner, buffer_size=8192))]
    pub fn __new__(py: Python<'_>, inner: Py<PyAny>, buffer_size: usize) -> PyResult<PyClassInitializer<Self>> {
        let options = lz4::Lz4WriterOptions { capacity: buffer_size };
        let inner: Box<dyn CompressingWriter + Send> = if let Ok(p) = path_like_to_string(inner.bind(py)) {
            Box::new(lz4::Lz4Writer::from_path_with_options(p, options)?)
        } else {
            Box::new(lz4::Lz4Writer::with_options(PyWriterAdapter::new(inner), options))
        };
        Ok(PyClassInitializer::from(CompressingWriterPy::__new__()).add_subclass(Self {
            inner: Mutex::new(Some(inner)),
        }))
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
