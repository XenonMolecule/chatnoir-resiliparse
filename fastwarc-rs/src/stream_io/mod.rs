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

use crate::stream_io::traits::{BufReadSeek, IntoWarcReader, IntoWarcWriter, WarcRead, WarcWrite};
use std::any::Any;
use std::io;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};

// ===========================================================
// Submodules
// ===========================================================

pub mod brotli;
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
    ($StreamType:ident, $OptionsType:ident) => {
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
                Ok(Self::with_options(std::fs::File::open(path)?, options))
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
// RawReaderAdapter
// ===========================================================

/// Wrapper type for arbitrary [`BufReadSeek`] readers that implements [`WarcRead`].
pub struct RawReaderAdapter<T> {
    inner: T,
}

impl<T> RawReaderAdapter<T> {
    /// Unwrap this [`RawReaderAdapter`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> BufRead for RawReaderAdapter<T>
where
    T: BufReadSeek,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.inner.consume(amount);
    }
}

impl<T> Read for RawReaderAdapter<T>
where
    T: BufReadSeek,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T> Seek for RawReaderAdapter<T>
where
    T: BufReadSeek,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

// ===========================================================
// IntoWarcReader implementations
// ===========================================================

impl<T> WarcRead for RawReaderAdapter<T>
where
    T: BufReadSeek,
{
    impl_to_any_methods!();

    fn inner_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }
}

impl<T> IntoWarcReader for T
where
    T: WarcRead,
{
    type Reader = T;

    fn into_warc_reader(self) -> Self::Reader {
        self
    }
}

impl<T> IntoWarcReader for BufReader<T>
where
    T: Send + 'static + Read + Seek,
{
    type Reader = RawReaderAdapter<BufReader<T>>;

    fn into_warc_reader(self) -> Self::Reader {
        RawReaderAdapter { inner: self }
    }
}

impl<T> IntoWarcReader for io::Cursor<T>
where
    T: Send + 'static + AsRef<[u8]>,
    io::Cursor<T>: Read,
{
    type Reader = RawReaderAdapter<io::Cursor<T>>;

    fn into_warc_reader(self) -> Self::Reader {
        RawReaderAdapter { inner: self }
    }
}

// Forwarding implementation for Box<dyn WarcRead>
impl<T> WarcRead for Box<T>
where
    T: WarcRead + ?Sized,
{
    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        (**self).as_any_mut()
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        let inner: Box<T> = *self;
        inner.into_any()
    }

    fn inner_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        (**self).inner_seek(pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        (**self).inner_stream_position()
    }

    fn frame_start_position(&mut self) -> io::Result<u64> {
        (**self).frame_start_position()
    }
}

// ===========================================================
// RawWriterAdapter
// ===========================================================

/// Wrapper type for arbitrary [`Write`] readers that implements [`WarcWrite`].
pub struct RawWriterAdapter<T> {
    inner: T,
}

impl<T> RawWriterAdapter<T>
where
    T: Write + 'static,
{
    /// Unwrap this [`RawWriterAdapter`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Write for RawWriterAdapter<T>
where
    T: Write + 'static,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T> WarcWrite for RawWriterAdapter<T>
where
    T: Write + 'static,
{
    impl_to_any_methods!();
}

// Forwarding implementation for Box<dyn WarcRead>
impl<T> WarcWrite for Box<T>
where
    T: WarcWrite + ?Sized,
{
    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        (**self).as_any_mut()
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        let inner: Box<T> = *self;
        inner.into_any()
    }

    fn finish(&mut self) -> io::Result<()> {
        (**self).finish()
    }
}

// ===========================================================
// IntoWarcWriter implementations
// ===========================================================

impl<T> IntoWarcWriter for T
where
    T: WarcWrite,
{
    type Writer = T;

    fn into_warc_writer(self) -> Self::Writer {
        self
    }
}

impl<T> IntoWarcWriter for io::BufWriter<T>
where
    T: Send + 'static + Write,
{
    type Writer = RawWriterAdapter<io::BufWriter<T>>;

    fn into_warc_writer(self) -> Self::Writer {
        RawWriterAdapter { inner: self }
    }
}

impl<T> IntoWarcWriter for io::Cursor<T>
where
    T: Send + 'static + AsMut<[u8]>,
    io::Cursor<T>: Write,
{
    type Writer = RawWriterAdapter<io::Cursor<T>>;

    fn into_warc_writer(self) -> Self::Writer {
        RawWriterAdapter { inner: self }
    }
}

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
pub struct LimitedBufReader {
    inner: Box<dyn WarcRead>,
    limit: u64,
    pos: u64,
}

impl LimitedBufReader {
    /// Create a new limited reader from a buffered reader instance.
    ///
    /// # Arguments
    ///
    /// * `reader` - inner reader to put the limit on
    /// * `limit` - limit in bytes at which EOF is returned
    pub fn new(reader: impl IntoWarcReader, limit: Option<u64>) -> Self {
        Self {
            inner: Box::new(reader.into_warc_reader()),
            limit: limit.unwrap_or(u64::MAX),
            pos: 0,
        }
    }

    /// Change the limit of the reader.
    /// Also resets the logical stream position to 0. Use [`Self::real_stream_position()`] to get
    /// the real position on the original stream.
    ///
    /// # Arguments
    ///
    /// * `limit` - new reader limit
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
        self.pos = 0;
    }

    /// Get the current limit.
    pub fn limit(&mut self) -> u64 {
        self.limit
    }

    /// Get the real (not the logical) stream position.
    pub fn real_stream_position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }

    /// Unwrap this [`LimitedBufReadSeek`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> Box<dyn WarcRead> {
        self.inner
    }

    /// Read until a linefeed (LF) is found or `max_line_len` is reached.
    /// The results are appended to the provided buffer.
    ///
    /// Apart from the line length limitation, the behavior is the same as [`io::BufRead::read_line()`].
    /// Unlike `Read::take(max_line_len).read_until(b'\n', buf)`, the read is guaranteed to be complete.
    /// The function will not return until a linefeed is found or EOF is reached.
    ///
    /// # Arguments
    ///
    /// * `buf` - output buffer to append to
    /// * `max_line_len` - maximum line length to read if no LF found
    ///
    /// # Returns
    ///
    /// Number of bytes read
    pub fn read_line(&mut self, buf: &mut Vec<u8>, mut max_line_len: usize) -> io::Result<usize> {
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

impl WarcRead for LimitedBufReader {
    impl_to_any_methods!();

    fn inner_seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.inner_seek(pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.inner.inner_stream_position()
    }
}

impl BufRead for LimitedBufReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let buf = self.inner.fill_buf()?;
        let buf_limited = &buf[..(buf.len() as u64).min(self.limit - self.pos) as usize];
        Ok(buf_limited)
    }

    fn consume(&mut self, amount: usize) {
        let amount = (amount as u64).min(self.limit - self.pos) as usize;
        self.inner.consume(amount);
        self.pos += amount as u64;
    }
}

impl Read for LimitedBufReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let l = buf.len();
        let buf_limited = &mut buf[..(l as u64).min(self.limit - self.pos) as usize];
        let n = self.inner.read(buf_limited)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for LimitedBufReader {
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

        self.inner
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
