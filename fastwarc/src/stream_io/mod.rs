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

use std::any::Any;
use std::io;

// ===========================================================
// Submodules
// ===========================================================

pub mod brotli;
pub mod chunked;
pub mod gzip;
pub mod lz4;
pub mod zstd;

// ===========================================================
// Global trait definitions
// ===========================================================

pub trait ReadSeek: io::Read + io::Seek + Send + 'static {}
impl<T: io::Read + io::Seek + Send + ?Sized + 'static> ReadSeek for T {}

pub trait BufReadSeek: io::BufRead + io::Seek + Send + 'static {}
impl<T: io::BufRead + io::Seek + Send + ?Sized + 'static> BufReadSeek for T {}

// ===========================================================
// General Reader / Writer trait
// ===========================================================

pub trait WarcRead: BufReadSeek + Any {
    /// Get an [`Any`] reference to this [`DecompressingReader`].
    fn as_any(&self) -> &dyn Any;

    /// Get a mutable [`Any`] reference to this [`DecompressingReader`].
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert the [`DecompressingReader`] into [`Any`].
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

pub trait WarcWrite: io::Write + Any + 'static {
    /// Get an [`Any`] reference to this [`DecompressingReader`].
    fn as_any(&self) -> &dyn Any;

    /// Get a mutable [`Any`] reference to this [`DecompressingReader`].
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert the [`DecompressingReader`] into [`Any`].
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

/// Internal helper for implementing boilerplate methods for casting to [`Any`].
macro_rules! impl_fastwarc_stream {
    ($StreamType: ident, $Trait: ident, $($TraitBounds: tt)+) => {
        impl<T: $($TraitBounds)+> $Trait for $StreamType<T> {
            fn as_any(&self) -> &dyn Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                self
            }

            fn into_any(self: Box<Self>) -> Box<dyn Any> {
                self
            }
        }
    };
}
pub(crate) use impl_fastwarc_stream;

// ===========================================================
// Compressors and decompressors
// ===========================================================

/// Trait for [`io::Read`] stream implementations reading from
/// compressed input streams.
pub trait DecompressingReader: WarcRead {
    /// Seek to an offset, in bytes, in the compressed inner stream.
    /// The semantics are the same as [`io::Seek::seek()`].
    ///
    /// Seeking on the inner stream may reset the state of the decompressor.
    /// It is up to the user to seek valid positions from which decompression
    /// can be resumed.
    fn inner_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>;

    /// Return the current seek position from the start of the compressed inner stream.
    /// The semantics are the same as [`io::Seek::stream_position()`].
    ///
    /// The returned position is the logical position on the compressed inner stream,
    /// which does not consider input buffer sizes. The physical reader position may be
    /// larger than this.
    fn inner_stream_position(&mut self) -> io::Result<u64>;

    /// Return the start position, in bytes, of the current member / frame
    /// in the compressed inner stream. If the compression format does not support
    /// multi-member streams, this is always the beginning of the stream.
    ///
    /// # Returns
    ///
    /// Position, in bytes, of the current member
    fn member_start_position(&mut self) -> io::Result<u64> {
        Ok(0)
    }
}

/// Trait for [`io::Write`] stream implementations that write compressed data
/// onto an output stream.
pub trait CompressingWriter: WarcWrite {
    /// Finish a compression member / frame and reset the compressor state.
    ///
    /// If the compressor supports multi-member streams, the writer can be
    /// used again after this to start a new member / frame. Otherwise, writing
    /// further bytes may yield an error. Calling this method without an open
    /// member / frame is a no-op.
    ///
    /// Does not necessarily flush buffer contents to the inner stream.
    /// Users should call [`io::Write::flush()`] afterward to ensure that
    /// all pending data is safely written.
    ///
    /// The behavior is implementation-specific and may do nothing.
    fn finish(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// ===========================================================
// Helper macros
// ===========================================================

/// Helper macro for implementing `from_path()` and `from_path_with_options()`
/// constructors for Readers and Writers.
macro_rules! impl_stream_from_path {
    ($StreamType: ident, $OptionsType: ident) => {
        impl $StreamType<std::fs::File> {
            #[doc = concat!("Create a [`", stringify!($StreamType), "`] from a file path.")]
            #[doc = ""]
            #[doc = "# Arguments"]
            #[doc = ""]
            #[doc = "* `path` - file path"]
            pub fn from_path(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
                Ok(Self::new(std::fs::File::open(path)?))
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
                Ok(Self::with_options(std::fs::File::open(path)?, options))
            }
        }
    };
}
pub(crate) use impl_stream_from_path;

// ===========================================================
// Limited buffered reader
// ===========================================================

/// A limited seekable buffered reader.
///
/// Wraps an existing [`BufReadSeek`] reader, setting the current reader position to be the new
/// stream start, and terminating with a premature, virtual EOF when the set limit is reached.
/// Calls to [`Self::stream_position()`] and [`Self::seek()`] work with and return the new
/// logical stream positions.
///
/// Does not allocate a new buffer. All calls are passed directly to the underlying reader.
pub trait LimitedBufReadSeek: WarcRead + 'static {
    /// Change the limit of the reader.
    /// Also resets the logical stream position to 0. Use [`Self::real_stream_position()`] to get
    /// the real position on the original stream.
    ///
    /// # Arguments
    ///
    /// * `limit` - new reader limit
    fn set_limit(&mut self, limit: u64);

    /// Get the current limit.
    fn limit(&mut self) -> u64;

    /// Get the real (not the logical) stream position.
    fn real_stream_position(&mut self) -> io::Result<u64>;

    /// Unwrap this [`LimitedBufReadSeek`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    fn into_inner(self) -> Box<dyn BufReadSeek>;

    /// Read until a linefeed (LF) is found or `max_line_len` is reached.
    /// The results are appended to the provided buffer.
    ///
    /// # Arguments
    ///
    /// * `buf` - output buffer to append to
    /// * `max_line_len` - maximum line length to read if no LF found
    ///
    /// # Returns
    ///
    /// Number of bytes read
    fn read_line(&mut self, buf: &mut Vec<u8>, mut max_line_len: usize) -> io::Result<usize> {
        max_line_len = max_line_len.min(self.limit() as usize);
        while buf.len() < max_line_len {
            let chunk = self.fill_buf()?;
            if chunk.is_empty() {
                break;
            }
            let remaining = max_line_len - buf.len();
            let limit = chunk.len().min(remaining);

            if let Some(pos) = chunk[..limit].iter().position(|&b| b == b'\n') {
                buf.extend_from_slice(&chunk[..=pos]);
                self.consume(pos + 1);
                return Ok(buf.len());
            }
            buf.extend_from_slice(&chunk[..limit]);
            self.consume(limit);
        }
        Ok(buf.len())
    }
}

pub type LimitedBufReader = LimitedBufReaderImpl<Box<dyn BufReadSeek>>;

impl LimitedBufReadSeek for LimitedBufReader {
    fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
        self.pos = 0;
    }

    fn limit(&mut self) -> u64 {
        self.limit
    }

    fn real_stream_position(&mut self) -> io::Result<u64> {
        self.reader.stream_position()
    }

    fn into_inner(self) -> Box<dyn BufReadSeek> {
        self.reader
    }
}

/// Internal type for wrapped payload readers.
/// This is only there to fulfil the type contracts on `WarcRecord.reader` and it implements an
/// `into_inner()` method that doesn't erase the [`WarcRead`] trait (important for unwrapping via [`Any`]).
pub(crate) type LimitedBufReaderOuterWrap = LimitedBufReaderImpl<Box<dyn WarcRead>>;

impl LimitedBufReadSeek for LimitedBufReaderOuterWrap {
    fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
        self.pos = 0;
    }

    fn limit(&mut self) -> u64 {
        self.limit
    }

    fn real_stream_position(&mut self) -> io::Result<u64> {
        self.reader.stream_position()
    }

    fn into_inner(self) -> Box<dyn BufReadSeek> {
        self.reader
    }
}

impl LimitedBufReaderOuterWrap {
    pub(crate) fn into_inner_warc(self) -> Box<dyn WarcRead> {
        self.reader
    }
}

pub struct LimitedBufReaderImpl<T>
where
    T: BufReadSeek,
{
    reader: T,
    limit: u64,
    pos: u64,
}

impl<T> LimitedBufReaderImpl<T>
where
    T: BufReadSeek,
{
    /// Create a new limited reader from a buffered reader instance.
    ///
    /// # Arguments
    ///
    /// * `reader` - inner reader to put the limit on
    /// * `limit` - limit in bytes at which EOF is returned
    pub fn new(reader: T, limit: Option<u64>) -> Self {
        Self {
            reader,
            limit: limit.unwrap_or(u64::MAX),
            pos: 0,
        }
    }
}

impl<T> WarcRead for LimitedBufReaderImpl<T>
where
    T: BufReadSeek,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

impl<T> io::Read for LimitedBufReaderImpl<T>
where
    T: BufReadSeek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let l = buf.len();
        let buf_limited = &mut buf[..(l as u64).min(self.limit - self.pos) as usize];
        let n = self.reader.read(buf_limited)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<T> io::BufRead for LimitedBufReaderImpl<T>
where
    T: BufReadSeek,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let buf = self.reader.fill_buf()?;
        let buf_limited = &buf[..(buf.len() as u64).min(self.limit - self.pos) as usize];
        Ok(buf_limited)
    }

    fn consume(&mut self, amount: usize) {
        let amount = (amount as u64).min(self.limit - self.pos) as usize;
        self.reader.consume(amount);
        self.pos += amount as u64;
    }
}

impl<T> io::Seek for LimitedBufReaderImpl<T>
where
    T: BufReadSeek,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        if pos == io::SeekFrom::Current(0) {
            return Ok(self.pos);
        }

        let mut new_pos = match pos {
            io::SeekFrom::Start(p) => p as i128,
            io::SeekFrom::End(p) => self.limit as i128 + p as i128,
            io::SeekFrom::Current(p) => self.pos as i128 + p as i128,
        };

        if new_pos < 0 || new_pos > i64::MAX as i128 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Seek out of range"));
        } else if new_pos > self.limit as i128 {
            new_pos = self.limit as i128;
        }

        self.reader
            .seek(io::SeekFrom::Current(new_pos as i64 - self.pos as i64))?;
        self.pos = new_pos as u64;
        Ok(self.pos)
    }
}

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
