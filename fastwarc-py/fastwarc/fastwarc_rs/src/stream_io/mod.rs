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

use self::gzip::{GzipReaderPy, GzipWriterPy};
use self::lz4::{Lz4ReaderPy, Lz4WriterPy};
use pyo3::exceptions::{PyModuleNotFoundError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyString};
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};

// ===========================================================
// Submodules
// ===========================================================

pub mod brotli;
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

    #[pyo3(signature = (offset, whence=0))]
    pub fn seek(&self, py: Python<'_>, offset: i128, whence: u8) -> PyResult<Py<PyAny>> {
        let _ = (offset, whence);
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

    #[pyo3(signature = (offset, whence=0))]
    pub fn inner_seek(&self, py: Python<'_>, offset: i128, whence: u8) -> PyResult<Py<PyAny>> {
        let _ = (offset, whence);
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

/// Python object stream adapter trait.
pub(crate) trait PyStreamAdapter: Sized {
    fn new(inner: Py<PyAny>) -> Self;
}

enum PyReaderType {
    GzipReader(Py<PyAny>),
    Lz4Reader(Py<PyAny>),
    Other(Py<PyAny>),
}

/// Reader adapter for Python file-like objects.
/// Wrapped types from `stream_io` are optimized with static dispatch. Other
/// `BytesIO` types will use dynamic Python `read()` / `seek()` calls.
pub(crate) struct PyReaderAdapter {
    inner: PyReaderType,
    pos: u64,
    buf: Vec<u8>,
    buf_pos: usize,
    buf_len: usize,
}

impl PyStreamAdapter for PyReaderAdapter {
    fn new(inner: Py<PyAny>) -> Self {
        PyReaderAdapter::new(inner)
    }
}

#[allow(unused)]
impl PyReaderAdapter {
    pub fn new(inner: Py<PyAny>) -> Self {
        Self::with_capacity(inner, 8192)
    }

    pub fn with_capacity(inner: Py<PyAny>, capacity: usize) -> Self {
        let inner = Python::attach(|py| {
            let bound = inner.bind(py);
            if bound.is_exact_instance_of::<GzipReaderPy>() {
                PyReaderType::GzipReader(inner)
            } else if bound.is_exact_instance_of::<Lz4ReaderPy>() {
                PyReaderType::Lz4Reader(inner)
            } else {
                PyReaderType::Other(inner)
            }
        });
        Self {
            inner,
            pos: 0,
            buf: vec![0; capacity],
            buf_pos: 0,
            buf_len: 0,
        }
    }
}

impl Read for PyReaderAdapter {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let in_buf = self.fill_buf()?;
        if in_buf.is_empty() {
            return Ok(0);
        }
        let n = in_buf.len().min(buf.len());
        buf[..n].copy_from_slice(&in_buf[..n]);
        self.consume(n);
        Ok(n)
    }
}

macro_rules! native_seek_call {
    ($inner: ident, $py: ident, $ReaderType: ident, $pos: ident, $buffered: expr) => {{
        let inner = &$inner.bind($py).cast::<$ReaderType>().unwrap().borrow().inner;
        let pos = match $pos {
            SeekFrom::Start(offset) => SeekFrom::Start(offset),
            SeekFrom::Current(offset) => SeekFrom::Current(offset - $buffered),
            SeekFrom::End(offset) => SeekFrom::End(offset),
        };
        Ok(inner.lock().unwrap().as_mut().unwrap().seek(pos)?)
    }};
}

impl Seek for PyReaderAdapter {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        if pos == SeekFrom::Current(0) {
            return Ok(self.pos);
        }
        let new_pos = match pos {
            SeekFrom::Start(p) => SeekFrom::Start(p),
            SeekFrom::Current(p) => SeekFrom::Start(
                u64::try_from(p as i128 + self.pos as i128)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Seek out of range"))?,
            ),
            SeekFrom::End(p) => SeekFrom::End(p),
        };
        if let SeekFrom::Start(p) = new_pos {
            if p >= self.pos - self.buf_pos as u64 && p < self.pos + self.buf_len as u64 {
                self.buf_pos += (p - self.pos) as usize;
                self.pos = p;
                return Ok(p);
            }
        }

        let buffered = (self.buf_len - self.buf_pos) as i64;
        let new_pos = Python::attach(|py| -> io::Result<u64> {
            match &self.inner {
                PyReaderType::GzipReader(inner) => native_seek_call!(inner, py, GzipReaderPy, pos, buffered),
                PyReaderType::Lz4Reader(inner) => native_seek_call!(inner, py, Lz4ReaderPy, pos, buffered),
                PyReaderType::Other(inner) => {
                    let stream = inner.bind(py);
                    let result = match pos {
                        SeekFrom::Start(offset) => stream.call_method1("seek", (offset, 0)),
                        SeekFrom::Current(offset) => stream.call_method1("seek", (offset - buffered, 1)),
                        SeekFrom::End(offset) => stream.call_method1("seek", (offset, 2)),
                    }?;
                    Ok(result.extract::<u64>()?)
                }
            }
        })?;
        self.pos = new_pos;
        self.buf_pos = 0;
        self.buf_len = 0;
        Ok(new_pos)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
    }
}

macro_rules! native_read_call {
    ($inner: ident, $py: ident, $ReaderType: ident, $buf: expr) => {{
        let inner = &$inner.bind($py).cast::<$ReaderType>().unwrap().borrow().inner;
        Ok(inner.lock().unwrap().as_mut().unwrap().read(&mut $buf)?)
    }};
}

impl BufRead for PyReaderAdapter {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.buf_pos < self.buf_len {
            return Ok(&self.buf[self.buf_pos..self.buf_len]);
        }

        let n = Python::attach(|py| -> io::Result<usize> {
            match &self.inner {
                PyReaderType::GzipReader(inner) => native_read_call!(inner, py, GzipReaderPy, self.buf),
                PyReaderType::Lz4Reader(inner) => native_read_call!(inner, py, Lz4ReaderPy, self.buf),
                PyReaderType::Other(inner) => {
                    let bound = inner.bind(py).call_method1("read", (self.buf.len(),))?;
                    let data = bound
                        .cast::<PyBytes>()
                        .map_err(|_| PyTypeError::new_err("read() must return bytes"))?
                        .as_bytes();
                    self.buf[..data.len()].copy_from_slice(data);
                    Ok(data.len())
                }
            }
        })?;

        self.buf_len = n;
        self.buf_pos = 0;
        Ok(&self.buf[..self.buf_len])
    }

    fn consume(&mut self, amount: usize) {
        self.pos += amount as u64;
        self.buf_pos += amount;
    }
}

enum PyWriterType {
    GzipWriter(Py<PyAny>),
    Lz4Writer(Py<PyAny>),
    Other(Py<PyAny>),
}

/// Writer adapter for Python file-like objects.
/// Wrapped types from `stream_io` are optimized with static dispatch. Other
/// `BytesIO` types will use dynamic Python `write()` / `flush()` calls.
pub(crate) struct PyWriterAdapter {
    inner: PyWriterType,
}

impl PyStreamAdapter for PyWriterAdapter {
    fn new(inner: Py<PyAny>) -> Self {
        PyWriterAdapter::new(inner)
    }
}

impl PyWriterAdapter {
    pub fn new(inner: Py<PyAny>) -> Self {
        let inner = Python::attach(|py| {
            let bound = inner.bind(py);
            if bound.is_exact_instance_of::<GzipReaderPy>() {
                PyWriterType::GzipWriter(inner)
            } else if bound.is_exact_instance_of::<Lz4ReaderPy>() {
                PyWriterType::Lz4Writer(inner)
            } else {
                PyWriterType::Other(inner)
            }
        });
        Self { inner }
    }
}

macro_rules! native_write_call {
    ($inner: ident, $py: ident, $WriterType: ident, $buf: ident) => {{
        let inner = &$inner.bind($py).cast::<$WriterType>().unwrap().borrow().inner;
        Ok(inner.lock().unwrap().as_mut().unwrap().write($buf)?)
    }};
}

macro_rules! native_flush_call {
    ($inner: ident, $py: ident, $WriterType: ident) => {{
        let inner = &$inner.bind($py).cast::<$WriterType>().unwrap().borrow().inner;
        Ok(inner.lock().unwrap().as_mut().unwrap().flush()?)
    }};
}

impl Write for PyWriterAdapter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(Python::attach(|py| -> io::Result<usize> {
            match &self.inner {
                PyWriterType::GzipWriter(inner) => native_write_call!(inner, py, GzipWriterPy, buf),
                PyWriterType::Lz4Writer(inner) => native_write_call!(inner, py, Lz4WriterPy, buf),
                PyWriterType::Other(inner) => Ok(inner
                    .bind(py)
                    .call_method1("write", (buf,))
                    .and_then(|result| result.extract::<usize>())?),
            }
        })?)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(Python::attach(|py| -> io::Result<()> {
            match &self.inner {
                PyWriterType::GzipWriter(inner) => native_flush_call!(inner, py, GzipWriterPy),
                PyWriterType::Lz4Writer(inner) => native_flush_call!(inner, py, Lz4WriterPy),
                PyWriterType::Other(inner) => Ok(inner.bind(py).call_method0("flush").map(|_| ())?),
            }
        })?)
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

/// Convert the `whence` argument of `BinaryIO.seek()` to a `SeekFrom`.
pub(crate) fn python_whence_to_seekfrom(offset: i128, whence: u8) -> PyResult<SeekFrom> {
    Ok(match whence {
        0 => SeekFrom::Start(u64::try_from(offset).map_err(|_| PyValueError::new_err("Seek offset out of range."))?),
        1 => SeekFrom::Current(i64::try_from(offset).map_err(|_| PyValueError::new_err("Seek offset out of range."))?),
        2 => SeekFrom::End(i64::try_from(offset).map_err(|_| PyValueError::new_err("Seek offset out of range."))?),
        _ => return Err(PyValueError::new_err("Invalid value for `whence` argument. Must be 0, 1, or 2.")),
    })
}

/// Wrap a Python stream object or file path / URL into a reader adapter.
/// Returns an `fsspec` stream object if `raw_stream` is a string starting with a
/// protocol prefix and `fsspec_args` is not `False`.
///
/// # Arguments
///
/// * `raw_stream` - input python stream / file-like object
/// * `fsspec_open_mode` - open mode for `fsspec.open()`
/// * `fsspec_args` - dict of arguments to pass to `fsspec` or `None` for defaults (set to `False` to disable `fsspec`)
/// * `wrap_stream_fn` - closure for constructing a native reader from a [`PyStreamAdapter`]
/// * `wrap_path_fn` - closure for constructing a native reader from a `String` file path
pub(crate) fn wrap_stream<T, A, FStream, FPath>(
    py: Python<'_>,
    raw_stream: Py<PyAny>,
    fsspec_open_mode: &str,
    fsspec_args: Option<Py<PyAny>>,
    wrap_stream_fn: FStream,
    wrap_path_fn: FPath,
) -> PyResult<T>
where
    A: PyStreamAdapter,
    FStream: FnOnce(A) -> io::Result<T>,
    FPath: FnOnce(String) -> io::Result<T>,
{
    // Check whether `raw_stream` is a string or path-like object or wrap in adapter.
    let Ok(path) = path_like_to_string(raw_stream.bind(py)) else {
        return Ok(wrap_stream_fn(A::new(raw_stream))?);
    };

    let use_fsspec = path.split_once("://").is_some()
        && fsspec_args
            .as_ref()
            .is_none_or(|a| a.bind(py).is(PyBool::new(py, false)));
    if use_fsspec {
        match py.import("fsspec") {
            Ok(fsspec) => {
                let handle = if let Some(args) = fsspec_args {
                    fsspec
                        .getattr("open")?
                        .call((path.as_str(), fsspec_open_mode), Some(args.bind(py).cast::<PyDict>()?))?
                } else {
                    fsspec.getattr("open")?.call1((path.as_str(), fsspec_open_mode))?
                }
                .call_method0("open")?;
                return Ok(wrap_stream_fn(A::new(handle.unbind()))?);
            }
            Err(err) => {
                if err.matches(py, py.get_type::<PyModuleNotFoundError>())? {
                    // fall-through
                } else {
                    return Err(err);
                }
            }
        }
    }

    // String argument, and we're not using fsspec: Construct native reader.
    Ok(wrap_path_fn(path)?)
}

/// Shorthand for [`wrap_stream::<_, PyReaderAdapter, _, _>(...)`](wrap_stream).
pub(crate) fn wrap_reader_stream<T, FStream, FPath>(
    py: Python<'_>,
    raw_stream: Py<PyAny>,
    fsspec_args: Option<Py<PyAny>>,
    wrap_stream_fn: FStream,
    wrap_path_fn: FPath,
) -> PyResult<T>
where
    FStream: FnOnce(PyReaderAdapter) -> io::Result<T>,
    FPath: FnOnce(String) -> io::Result<T>,
{
    wrap_stream::<T, PyReaderAdapter, _, _>(py, raw_stream, "rb", fsspec_args, wrap_stream_fn, wrap_path_fn)
}

/// Shorthand for [`wrap_stream::<_, PyWriterAdapter, _, _>(...)`](wrap_stream).
pub(crate) fn wrap_writer_stream<T, FStream, FPath>(
    py: Python<'_>,
    raw_stream: Py<PyAny>,
    fsspec_args: Option<Py<PyAny>>,
    wrap_stream_fn: FStream,
    wrap_path_fn: FPath,
) -> PyResult<T>
where
    FStream: FnOnce(PyWriterAdapter) -> io::Result<T>,
    FPath: FnOnce(String) -> io::Result<T>,
{
    wrap_stream::<T, PyWriterAdapter, _, _>(py, raw_stream, "wb", fsspec_args, wrap_stream_fn, wrap_path_fn)
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
            use crate::stream_io::python_whence_to_seekfrom;
            Ok(reader.$seek_fn_name(python_whence_to_seekfrom($offset, $whence)?)?)
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
