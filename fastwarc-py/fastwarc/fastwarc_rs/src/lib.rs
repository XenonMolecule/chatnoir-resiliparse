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

#[pymodule]
pub mod _fastwarc_rs {
    use super::*;

    #[pymodule_init]
    pub fn __init__(m: &Bound<'_, PyModule>) -> PyResult<()> {
        // Register submodules to make them importable.
        // https://github.com/PyO3/pyo3/issues/759#issuecomment-2282197848
        let parent_name: String = m.getattr("__name__")?.extract()?;
        let sys_modules = m.py().import("sys")?.getattr("modules")?;
        sys_modules.set_item(format!("{parent_name}.stream_io"), m.getattr("stream_io")?)?;
        sys_modules.set_item(format!("{parent_name}.warc"), m.getattr("warc")?)?;

        Ok(())
    }

    #[pymodule]
    pub mod stream_io {
        #[pymodule_export]
        pub use crate::stream_io::{CompressingWriterPy, DecompressingReaderPy, ReaderPy, WriterPy};

        #[pymodule_export]
        pub use crate::stream_io::gzip::{GzipReaderPy, GzipWriterPy};

        #[pymodule_export]
        pub use crate::stream_io::lz4::{Lz4ReaderPy, Lz4WriterPy};
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
        pub use crate::warc::{HeaderMapPy, WarcRecordTypePy};
    }
}
