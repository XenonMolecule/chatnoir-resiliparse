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

use pyo3::prelude::*;
use pyo3::types::PyModule;

#[pymodule]
pub mod _fastwarc {
    use super::*;

    #[pymodule_init]
    pub fn __init__(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let stream_io = m.getattr("stream_io")?;
        let warc = m.getattr("warc")?;

        // Top-level exports
        m.add("ArchiveIterator", warc.getattr("ArchiveIterator")?)?;
        m.add("WarcRecord", warc.getattr("WarcRecord")?)?;
        m.add("WarcRecordType", warc.getattr("WarcRecordType")?)?;

        m.add("GzipReader", stream_io.getattr("GzipReader")?)?;
        m.add("GzipWriter", stream_io.getattr("GzipWriter")?)?;
        m.add("Lz4Reader", stream_io.getattr("Lz4Reader")?)?;
        m.add("Lz4Writer", stream_io.getattr("Lz4Writer")?)?;
        m.add("ZstdReader", stream_io.getattr("ZstdReader")?)?;
        m.add("ZstdWriter", stream_io.getattr("ZstdWriter")?)?;

        Ok(())
    }

    #[pymodule(module = "fastwarc")]
    pub mod stream_io {
        #[pymodule_export]
        pub use crate::stream_io::{WarcReaderPy, WarcWriterPy};

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

    #[pymodule(module = "fastwarc")]
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
