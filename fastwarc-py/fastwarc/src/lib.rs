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

mod stream_io;
mod warc;

use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use pyo3::types::{PyList, PyModule};
use std::ffi::CString;
use std::path::PathBuf;

#[pymodule]
pub mod fastwarc {
    use super::*;

    #[pymodule_init]
    pub fn __init__(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let stream_io = m.getattr("stream_io")?;
        let warc = m.getattr("warc")?;

        // Register submodules to make them importable.
        // https://github.com/PyO3/pyo3/issues/759#issuecomment-2282197848
        let parent_name: String = m.getattr("__name__")?.extract()?;
        let sys_modules = m.py().import("sys")?.getattr("modules")?;
        sys_modules.set_item(format!("{parent_name}.stream_io"), &stream_io)?;
        sys_modules.set_item(format!("{parent_name}.warc"), &warc)?;

        enable_submodule_imports(m)?;

        // Top-level exports
        m.add("GzipReader", stream_io.getattr("GzipReader")?)?;
        m.add("GzipWriter", stream_io.getattr("GzipWriter")?)?;
        m.add("Lz4Reader", stream_io.getattr("Lz4Reader")?)?;
        m.add("Lz4Writer", stream_io.getattr("Lz4Writer")?)?;
        m.add("ZstdReader", stream_io.getattr("ZstdReader")?)?;
        m.add("ZstdWriter", stream_io.getattr("ZstdWriter")?)?;

        // Legacy exports
        m.add("FileStream", stream_io.getattr("FileStream")?)?;
        m.add("GZipStream", stream_io.getattr("GZipStream")?)?;
        m.add("LZ4Stream", stream_io.getattr("LZ4Stream")?)?;
        m.add("FastWARCError", stream_io.getattr("FastWARCError")?)?;
        m.add("StreamError", stream_io.getattr("StreamError")?)?;

        Ok(())
    }

    #[pymodule]
    pub mod stream_io {
        use super::*;

        #[pymodule_init]
        fn __init__(m: &Bound<'_, PyModule>) -> PyResult<()> {
            create_legacy_shims(m, m)
        }

        #[pymodule_export]
        pub use crate::stream_io::{CompressingWriterPy, DecompressingReaderPy, ReaderPy, WriterPy};

        #[pymodule_export]
        pub use crate::stream_io::gzip::{GzipReaderPy, GzipWriterPy};

        #[pymodule_export]
        pub use crate::stream_io::lz4::{Lz4ReaderPy, Lz4WriterPy};

        #[pymodule_export]
        pub use crate::stream_io::brotli::{BrotliReaderPy, BrotliWriterPy};

        #[pymodule_export]
        pub use crate::stream_io::zstd::{ZstdReaderPy, ZstdWriterPy};

        #[pymodule_export]
        pub use crate::stream_io::chunked::{ChunkedReaderPy, ChunkedWriterPy};
    }

    #[pymodule]
    pub mod warc {
        use super::*;

        #[pymodule_init]
        fn __init__(m: &Bound<'_, PyModule>) -> PyResult<()> {
            // Re-export enum members as individual constants.
            m.add("warcinfo", WarcRecordTypePy::warcinfo)?;
            m.add("response", WarcRecordTypePy::response)?;
            m.add("resource", WarcRecordTypePy::resource)?;
            m.add("request", WarcRecordTypePy::request)?;
            m.add("metadata", WarcRecordTypePy::metadata)?;
            m.add("revisit", WarcRecordTypePy::revisit)?;
            m.add("conversion", WarcRecordTypePy::conversion)?;
            m.add("continuation", WarcRecordTypePy::continuation)?;
            m.add("unknown", WarcRecordTypePy::unknown)?;
            m.add("no_type", WarcRecordTypePy::no_type)?;
            m.add("any_type", WarcRecordTypePy::any_type)?;

            Ok(())
        }

        #[pymodule_export]
        pub use crate::warc::{ArchiveIteratorPy, HeaderMapPy, WarcRecordPy, WarcRecordTypePy};
    }
}

/// Helper for making on-disk submodules importable by making the extension behave like a package..
fn enable_submodule_imports(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    let module_file: String = m.getattr("__file__")?.extract()?;
    let Some(package_dir) = PathBuf::from(module_file).parent().map(|p| p.to_path_buf()) else {
        return Ok(());
    };
    let package_dir = package_dir.to_string_lossy().into_owned();
    let submodule_search_locations = PyList::new(py, [&package_dir])?;

    // Mark extension as package-like by adding __path__ and __spec__.
    m.setattr("__path__", &submodule_search_locations)?;
    let spec = m.getattr("__spec__")?;
    if !spec.is_none() {
        spec.setattr("submodule_search_locations", &submodule_search_locations)?;
    }

    Ok(())
}

/// Helper for creating legacy shims in stream_io module
pub(crate) fn create_legacy_shims(m: &Bound<'_, PyModule>, stream_io: &Bound<'_, PyAny>) -> PyResult<()> {
    let py = m.py();

    m.add("FastWARCError", py.get_type::<PyOSError>())?;
    m.add("StreamError", py.get_type::<PyOSError>())?;

    let code = CString::new(
        r#"
def _is_writer_stream(raw_stream):
    mode = getattr(raw_stream, "mode", None)
    if isinstance(mode, str):
        if any(flag in mode for flag in ("w", "a", "x")):
            return True
        if "r" in mode:
            return False

    has_read = callable(getattr(raw_stream, "read", None))
    has_write = callable(getattr(raw_stream, "write", None))
    return has_write and not has_read

def wrap_stream(raw_stream, mode='rb', fsspec_args=None):
    return raw_stream

class IOStream:
    pass

class PythonIOStreamAdapter(IOStream):
    def __new__(cls, first, *args, **kwargs):
        return first

class CompressingStream(IOStream):
    pass

class FileStream(IOStream):
    def __new__(cls, filename, mode='rb'):
        if 'b' not in mode:
            mode += 'b'
        return open(filename, mode)


class BytesIOStream(IOStream):
    def __new__(cls, initial_data):
        import io
        return io.BytesIO(initial_data)


class GZipStream(CompressingStream):
    def __new__(cls, raw_stream, mode='r', compression_level=9, zlib=False, fsspec_args=None):
        is_writer = any(flag in mode for flag in ('w', 'a', 'x'))
        if isinstance(raw_stream, str):
            if is_writer:
                return GzipWriter(
                    raw_stream,
                    compression_level=compression_level,
                    zlib=zlib,
                    fsspec_args=fsspec_args,
                )
            return GzipReader(raw_stream, zlib=zlib, fsspec_args=fsspec_args)
        if is_writer or _is_writer_stream(raw_stream):
            return GzipWriter(
                raw_stream,
                compression_level=compression_level,
                zlib=zlib,
                fsspec_args=fsspec_args,
            )
        return GzipReader(raw_stream, zlib=zlib, fsspec_args=fsspec_args)


class LZ4Stream(CompressingStream):
    def __new__(cls, raw_stream, mode='r', compression_level=12, favor_dec_speed=True, fsspec_args=None):
        is_writer = any(flag in mode for flag in ('w', 'a', 'x'))
        if isinstance(raw_stream, str):
            if is_writer:
                return Lz4Writer(raw_stream, fsspec_args=fsspec_args)
            return Lz4Reader(raw_stream, fsspec_args=fsspec_args)
        if is_writer or _is_writer_stream(raw_stream):
            return Lz4Writer(raw_stream, fsspec_args=fsspec_args)
        return Lz4Reader(raw_stream, fsspec_args=fsspec_args)

class BrotliStream(CompressingStream):
    def __new__(cls, raw_stream, mode='r', fsspec_args=None):
        is_writer = any(flag in mode for flag in ('w', 'a', 'x'))
        if isinstance(raw_stream, str):
            if is_writer:
                return BrotliWriter(raw_stream, fsspec_args=fsspec_args)
            return BrotliReader(raw_stream, fsspec_args=fsspec_args)
        if is_writer or _is_writer_stream(raw_stream):
            return BrotliWriter(raw_stream, fsspec_args=fsspec_args)
        return BrotliReader(raw_stream, fsspec_args=fsspec_args)
"#,
    )?;
    let filename = CString::new("<fastwarc_legacy_shims>")?;
    let module_name = CString::new("fastwarc_legacy_shims")?;
    let legacy = PyModule::from_code(py, code.as_c_str(), filename.as_c_str(), module_name.as_c_str())?;

    legacy.add("GzipReader", stream_io.getattr("GzipReader")?)?;
    legacy.add("GzipWriter", stream_io.getattr("GzipWriter")?)?;
    legacy.add("Lz4Reader", stream_io.getattr("Lz4Reader")?)?;
    legacy.add("Lz4Writer", stream_io.getattr("Lz4Writer")?)?;
    legacy.add("BrotliReader", stream_io.getattr("BrotliReader")?)?;
    legacy.add("BrotliWriter", stream_io.getattr("BrotliWriter")?)?;

    m.add("IOStream", legacy.getattr("IOStream")?)?;
    m.add("PythonIOStreamAdapter", legacy.getattr("PythonIOStreamAdapter")?)?;
    m.add("CompressingStream", legacy.getattr("CompressingStream")?)?;
    m.add("FileStream", legacy.getattr("FileStream")?)?;
    m.add("BytesIOStream", legacy.getattr("BytesIOStream")?)?;
    m.add("GZipStream", legacy.getattr("GZipStream")?)?;
    m.add("LZ4Stream", legacy.getattr("LZ4Stream")?)?;
    m.add("BrotliStream", legacy.getattr("BrotliStream")?)?;
    m.add("wrap_stream", legacy.getattr("wrap_stream")?)?;

    Ok(())
}
