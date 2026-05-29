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

use crate::stream_io::traits::BufReadSeek;
use std::io;

// ===========================================================
// Submodules
// ===========================================================

pub mod brotli;
pub mod bufread;
pub mod chunked;
pub mod gzip;
pub mod lz4;
pub mod traits;
pub mod zstd;

// ===========================================================
// Helper macros
// ===========================================================

/// Helper macro for implementing `from_path()` and `from_path_with_options()`
/// constructors for Readers and Writers.
macro_rules! impl_stream_from_path {
    ($StreamType:ident, $OptionsType:ident, $open_method:ident) => {
        impl $StreamType<std::fs::File> {
            #[doc = concat!("Create a [`", stringify!($StreamType), "`] from a file path.")]
            #[doc = ""]
            #[doc = "# Arguments"]
            #[doc = ""]
            #[doc = "* `path` - file path"]
            pub fn from_path(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
                Self::from_path_with_options(path, Default::default())
            }

            #[doc = concat!("Create a [`", stringify!($StreamType), "`] from a file path.")]
            #[doc = ""]
            #[doc = "# Arguments"]
            #[doc = ""]
            #[doc = "* `path` - file path"]
            #[doc = "* `options` - constructor options"]
            pub fn from_path_with_options(
                path: impl AsRef<std::path::Path>,
                options: $OptionsType,
            ) -> io::Result<Self> {
                Ok(Self::with_options(std::fs::File::$open_method(path)?, options))
            }
        }
    };
}
pub(super) use impl_stream_from_path;

/// Internal helper for implementing to_any() boilerplate methods.
macro_rules! impl_to_any_methods {
    () => {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    };
}
pub(super) use impl_to_any_methods;

// ===========================================================
// Helper functions
// ===========================================================

/// Internal helper that implements forward seek in compressed streams.
///
/// For this to work, `reader.stream_position()` must report an accurate
/// position after calling `reader.consume()`.
fn _forward_seek(reader: &mut impl BufReadSeek, pos: io::SeekFrom) -> io::Result<u64> {
    let diff = match pos {
        io::SeekFrom::Start(p) => -(reader.stream_position()? as i128) + p as i128,
        io::SeekFrom::Current(p) => p as i128,
        io::SeekFrom::End(_) => {
            return Err(io::Error::new(io::ErrorKind::Unsupported, "Seeking from end not supported"));
        }
    };
    if diff < 0 {
        return Err(io::Error::new(io::ErrorKind::Unsupported, "Backward seeking not supported"));
    }

    let mut remaining =
        usize::try_from(diff).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Seek out of range"))?;

    while remaining > 0 {
        let n = reader.fill_buf()?.len().min(remaining);
        if n == 0 {
            break;
        }
        reader.consume(n);
        remaining -= n;
    }
    reader.stream_position()
}

// ===========================================================
// Tests
// ===========================================================

#[cfg(test)]
#[path = "mod_test.rs"]
mod mod_test;
