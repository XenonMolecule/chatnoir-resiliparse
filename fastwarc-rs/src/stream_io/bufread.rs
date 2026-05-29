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

use crate::stream_io::impl_to_any_methods;
use crate::stream_io::traits::{
    BufReadSeek, IntoWarcReader, IntoWarcWriter, ReadSeek, WarcRead, WarcWrite, Write as _Write,
};
use std::any::Any;
use std::io;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};

// ===========================================================
// RawReaderAdapter
// ===========================================================

/// Wrapper type for arbitrary [`BufReadSeek`] readers that implements [`WarcRead`].
pub struct RawReaderAdapter<T> {
    inner: T,
    pos: u64,
}

impl<T> RawReaderAdapter<T> {
    /// Unwrap this [`RawReaderAdapter`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Borrow the wrapped reader.
    pub fn get_ref(&self) -> &T {
        &self.inner
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
        self.pos += amount as u64;
        self.inner.consume(amount);
    }
}

impl<T> Read for RawReaderAdapter<T>
where
    T: BufReadSeek,
{
    // noinspection DuplicatedCode
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.fill_buf()?.read(buf)?;
        self.consume(n);
        Ok(n)
    }
}

impl<T> Seek for RawReaderAdapter<T>
where
    T: BufReadSeek,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        if pos == io::SeekFrom::Current(0) {
            return Ok(self.pos);
        }
        self.pos = self.inner.seek(pos)?;
        Ok(self.pos)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.pos)
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
        self.seek(pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.stream_position()
    }
}

impl<T> IntoWarcReader for T
where
    T: WarcRead,
{
    fn into_warc_reader(self) -> Box<dyn WarcRead> {
        Box::new(self)
    }
}

impl IntoWarcReader for Box<dyn WarcRead> {
    fn into_warc_reader(self) -> Box<dyn WarcRead> {
        self
    }
}

impl IntoWarcReader for Box<dyn WarcRead + Send> {
    fn into_warc_reader(self) -> Box<dyn WarcRead> {
        self
    }
}

impl IntoWarcReader for Box<dyn WarcRead + Send + Sync> {
    fn into_warc_reader(self) -> Box<dyn WarcRead> {
        self
    }
}

impl<T> IntoWarcReader for BufReader<T>
where
    T: ReadSeek,
{
    fn into_warc_reader(mut self) -> Box<dyn WarcRead> {
        let pos = self.stream_position().unwrap_or(0);
        Box::new(RawReaderAdapter { inner: self, pos })
    }
}

impl IntoWarcReader for std::fs::File {
    fn into_warc_reader(mut self) -> Box<dyn WarcRead> {
        let pos = self.stream_position().unwrap_or(0);
        Box::new(RawReaderAdapter {
            inner: BufReader::new(self),
            pos,
        })
    }
}

impl<T> IntoWarcReader for io::Cursor<T>
where
    T: Send + AsRef<[u8]> + 'static,
    io::Cursor<T>: Read,
{
    fn into_warc_reader(self) -> Box<dyn WarcRead> {
        let pos = self.position();
        Box::new(RawReaderAdapter { inner: self, pos })
    }
}

// ===========================================================
// RawWriterAdapter
// ===========================================================

/// Wrapper type for arbitrary [`Write`] readers that implements [`WarcWrite`].
pub struct RawWriterAdapter<T> {
    inner: T,
}

impl<T: _Write> RawWriterAdapter<T> {
    /// Unwrap this [`RawWriterAdapter`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: _Write> Write for RawWriterAdapter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: _Write> WarcWrite for RawWriterAdapter<T> {
    impl_to_any_methods!();
}

// ===========================================================
// IntoWarcWriter implementations
// ===========================================================

impl<T> IntoWarcWriter for T
where
    T: WarcWrite,
{
    fn into_warc_writer(self) -> Box<dyn WarcWrite> {
        Box::new(self)
    }
}

impl IntoWarcWriter for Box<dyn WarcWrite> {
    fn into_warc_writer(self) -> Box<dyn WarcWrite> {
        self
    }
}

impl IntoWarcWriter for Box<dyn WarcWrite + Send> {
    fn into_warc_writer(self) -> Box<dyn WarcWrite> {
        self
    }
}

impl IntoWarcWriter for Box<dyn WarcWrite + Send + Sync> {
    fn into_warc_writer(self) -> Box<dyn WarcWrite> {
        self
    }
}

impl<T> IntoWarcWriter for io::BufWriter<T>
where
    T: _Write,
{
    fn into_warc_writer(self) -> Box<dyn WarcWrite> {
        Box::new(RawWriterAdapter { inner: self })
    }
}

impl<T> IntoWarcWriter for io::Cursor<T>
where
    T: AsMut<[u8]> + Send + 'static,
    io::Cursor<T>: _Write,
{
    fn into_warc_writer(self) -> Box<dyn WarcWrite> {
        Box::new(RawWriterAdapter { inner: self })
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
        let mut reader = reader.into_warc_reader();
        let pos = reader.stream_position().unwrap_or(0);
        Self {
            inner: reader,
            limit: limit.unwrap_or(u64::MAX),
            pos,
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

    /// Reset the reader to being unlimited.
    pub fn reset_limit(&mut self) {
        self.limit = u64::MAX;
        self.pos = self.inner.stream_position().unwrap_or(0);
    }

    /// Get the current limit.
    pub fn limit(&mut self) -> u64 {
        self.limit
    }

    /// Get the real (not the logical) stream position in the inner stream.
    /// This method is non-recursive. Use [`Self::inner_stream_position()`] if you need
    /// to reach through a decoding inner reader stack.
    pub fn real_stream_position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }

    /// Unwrap this [`LimitedBufReadSeek`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> Box<dyn WarcRead> {
        self.inner
    }

    /// Borrow the wrapped reader as [`Any`] for type inspection.
    pub fn inner_as_any(&self) -> &dyn Any {
        self.inner.as_any()
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

    fn frame_start_position(&mut self) -> io::Result<Option<u64>> {
        self.inner.frame_start_position()
    }

    fn is_stream_decoder(&self) -> bool {
        self.inner.is_stream_decoder()
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
