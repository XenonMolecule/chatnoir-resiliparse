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

use crate::stream_io::traits::{ReadSeek, WarcRead, WarcWrite, Write as _Write};
use crate::stream_io::{impl_stream_from_path, impl_to_any_methods};
use std::any::Any;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};
use zlib_rs::{Deflate, DeflateFlush, Inflate, InflateFlush};

// ===========================================================
// GzipReader
// ===========================================================

/// Reader for Gzip-compressed streams.
pub struct GzipReader<T: ReadSeek> {
    inner: BufReader<T>,
    deflate: Inflate,
    stream_pos: u64,
    member_pos: u64,
    next_member_pos: u64,
    buf: Vec<u8>,
    buf_pos: usize,
    buf_len: usize,
    window_bits: u8,
    decomp_ratio: f32,
}

pub const MAX_WBITS: u8 = 15;

/// Options for constructing a new [`GzipReader`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
/// * `window_bits` - history buffer size, behavior compatible with zlib:
///   Values between `9..15` are for zlib streams. Values above `15` ([`MAX_WBITS`])
///   enable gzip header decoding:
///     * `+16` enables gzip header decoding only,
///     * `+32` enables gzip/zlib header autodetection.
/// * `expect_header` - specifies whether a zlib or gzip header is expected or
///   whether this is a raw stream (equivalent to negative `window_bits` in zlib).
#[derive(Debug, Copy, Clone)]
pub struct GzipReaderOptions {
    pub capacity: usize,
    pub window_bits: u8,
    pub expect_header: bool,
}

impl Default for GzipReaderOptions {
    fn default() -> Self {
        Self {
            window_bits: MAX_WBITS + 16,
            expect_header: true,
            capacity: 4096,
        }
    }
}

impl<T: ReadSeek> GzipReader<T> {
    /// Create a new [`GzipReader`].
    ///
    /// Allocates an internal buffer holding chunks of the inner stream.
    /// A second, larger buffer is allocated for the decompressed data.
    /// Initially, the decompressed buffer will be twice the size of the
    /// uncompressed buffer, but its size can change based on demand.
    ///
    /// The default buffer size is 4096 bytes. For custom buffer sizes, use
    /// [`Self::with_capacity()`].
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, GzipReaderOptions::default())
    }

    /// Create a new [`GzipReader`] with a given buffer capacity.
    ///
    /// Allocates an internal buffer holding chunks of the inner stream.
    /// A second, larger buffer is allocated for the decompressed data.
    /// Initially, the decompressed buffer will be twice the size of the
    /// uncompressed buffer, but its size can change based on demand.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `capacity` - buffer capacity
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(
            inner,
            GzipReaderOptions {
                capacity,
                ..GzipReaderOptions::default()
            },
        )
    }

    /// Create a new [`GzipReader`] with the supplied options.
    ///
    /// Allocates an internal buffer holding chunks of the inner stream.
    /// A second, larger buffer is allocated for the decompressed data.
    /// Initially, the decompressed buffer will be twice the size of the
    /// uncompressed buffer, but its size can change based on demand.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `options` - reader options
    pub fn with_options(mut inner: T, options: GzipReaderOptions) -> Self {
        let window_bits = options.window_bits;
        let decomp_ratio = 2.0;
        let inner_pos = inner.stream_position().unwrap_or(0);
        Self {
            inner: BufReader::with_capacity(options.capacity, inner),
            deflate: Inflate::new(options.expect_header, window_bits),
            stream_pos: 0,
            member_pos: inner_pos,
            next_member_pos: inner_pos,
            buf: vec![0; options.capacity * decomp_ratio as usize],
            buf_pos: 0,
            buf_len: 0,
            window_bits,
            decomp_ratio,
        }
    }

    /// Unwrap this [`GzipReader`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    /// Dynamically update the output buffer size using a moving average of the
    /// output to input size.
    ///
    /// TODO: Benchmark the parameters!
    ///
    /// # Arguments
    ///
    /// * `buf_len` - length of the uncompressed input buffer
    /// * `consumed` - number of bytes consumed from the input buffer
    /// * `produced` - number of bytes produced on the output buffer
    fn _update_buf_size(&mut self, read_len: usize, consumed: usize, produced: usize) {
        self.decomp_ratio = 0.9 * self.decomp_ratio + 0.1 * (produced as f32 / consumed as f32);
        let target_buf_size = ((read_len as f32 * self.decomp_ratio).ceil() as usize)
            .clamp(2 * self.inner.capacity(), 1 << 16)
            .max(self.buf_len)
            .next_power_of_two();

        if target_buf_size > self.buf.len() {
            // println!("Increasing output buffer to {}", target_buf_size);
            self.buf.resize(target_buf_size, 0);
        } else if target_buf_size * 2 < self.buf.len() {
            // println!("Shrinking output buffer to {}", target_buf_size);
            self.buf.truncate(target_buf_size);
        }
    }
}

impl_stream_from_path!(GzipReader, GzipReaderOptions, open);

// noinspection DuplicatedCode
impl<T: ReadSeek> io::Read for GzipReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.fill_buf()?.read(buf)?;
        self.consume(n);
        Ok(n)
    }
}

// noinspection DuplicatedCode
impl<T: ReadSeek> Seek for GzipReader<T> {
    /// Seek to an offset, in bytes, in the decompressed output stream.
    ///
    /// Seeking in a compressed stream is not efficient with O(n) complexity,
    /// and backwards seeking and seeking from the end are not supported.
    ///
    /// # Arguments
    ///
    /// `pos` - seek position
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        if pos == SeekFrom::Current(0) {
            return Ok(self.stream_pos);
        }
        super::_forward_seek(self, pos)
    }

    /// Returns the current seek position from the start of the decompressed output stream.
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.stream_pos)
    }
}

impl<T: ReadSeek> WarcRead for GzipReader<T> {
    impl_to_any_methods!();

    fn inner_seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.deflate = Inflate::new(true, self.window_bits);
        self.buf_pos = 0;
        self.buf_len = 0;
        let new_pos = self.inner.seek(pos)?;
        self.next_member_pos = new_pos;
        self.stream_pos = 0;
        Ok(new_pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.inner.stream_position()
    }

    fn frame_start_position(&mut self) -> io::Result<Option<u64>> {
        Ok(Some(self.member_pos))
    }

    fn is_stream_decoder(&self) -> bool {
        true
    }
}

impl<T: ReadSeek> BufRead for GzipReader<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.buf_pos < self.buf_len {
            return Ok(&self.buf[self.buf_pos..self.buf_len]);
        }

        self.buf_pos = 0;
        self.buf_len = 0;

        loop {
            let total_out = self.deflate.total_out();
            let total_in = self.deflate.total_in();

            let in_buf = self.inner.fill_buf()?;
            let in_buf_len = in_buf.len();

            // New member started and not EOF
            if self.next_member_pos != self.member_pos && in_buf_len > 0 {
                self.member_pos = self.next_member_pos;
            }

            let status = self
                .deflate
                .decompress(in_buf, &mut self.buf[self.buf_len..], InflateFlush::NoFlush)
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("Gzip decompression error: {}", e.as_str()))
                })?;

            let in_delta = self.deflate.total_in() - total_in;
            let out_delta = self.deflate.total_out() - total_out;
            self.inner.consume(in_delta as usize);
            self.buf_len += out_delta as usize;

            // Member end or EOF
            if matches!(status, zlib_rs::Status::StreamEnd) {
                self.deflate = Inflate::new(true, self.window_bits);
                self.next_member_pos = self.inner.stream_position()?;
                break;
            }

            if in_delta > 0 {
                // Adjust output buffer size if needed
                self._update_buf_size(in_buf_len, in_delta as usize, out_delta as usize);
            }

            // Buffer full or no progress
            if self.buf_len == self.buf.len() || (in_delta == 0 && out_delta == 0) {
                break;
            }
        }

        Ok(&self.buf[..self.buf_len])
    }

    fn consume(&mut self, amount: usize) {
        let old_buf_os = self.buf_pos;
        self.buf_pos = self.buf.len().min(self.buf_pos + amount);
        self.stream_pos += (self.buf_pos - old_buf_os) as u64;
    }
}

// ===========================================================
// GzipWriter
// ===========================================================

/// Writer for Gzip-compressed streams.
pub struct GzipWriter<T: _Write> {
    inner: Option<T>,
    deflate: Deflate,
    member_started: bool,
    buf: Vec<u8>,
    buf_pos: usize,
    level: i32,
    window_bits: u8,
}

/// Options for constructing a new [`GzipWriter`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
/// * `window_bits` - history buffer size, behavior compatible with zlib:
///   Values between `9..15` are for zlib streams. Values above `15` ([`MAX_WBITS`])
///   enable gzip header encoding:
///     * `+16` enables gzip header encoding only,
///     * `+32` enables gzip/zlib header autodetection.
/// * `expect_header` - specifies whether a zlib or gzip header is expected or
///   whether this is a raw stream (equivalent to negative `window_bits` in zlib).
/// * `compression_level` - gzip / DEFLATE compression level to use (`9` is best)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct GzipWriterOptions {
    pub capacity: usize,
    pub window_bits: u8,
    pub expect_header: bool,
    pub compression_level: i32,
}

impl Default for GzipWriterOptions {
    fn default() -> Self {
        Self {
            window_bits: MAX_WBITS + 16,
            expect_header: true,
            capacity: 8192,
            compression_level: 9,
        }
    }
}

// noinspection DuplicatedCode
impl<T: _Write> GzipWriter<T> {
    /// Create a new [`GzipWriter`].
    ///
    /// Maintains a small write buffer to temporarily store compressed data before flushing them
    /// to the underlying stream. The default buffer size is 8192 bytes. Use [`Self::with_capacity()`]
    /// for custom buffer sizes.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, GzipWriterOptions::default())
    }

    /// Create a new [`GzipWriter`] in DEFLATE mode (i.e., with zlib header).
    ///
    /// This is equivalent to calling [`Self::with_options()`] and setting `window_flags`
    /// to [`MAX_WBITS`].
    pub fn new_deflate(inner: T) -> Self {
        Self::with_options(
            inner,
            GzipWriterOptions {
                window_bits: MAX_WBITS,
                ..GzipWriterOptions::default()
            },
        )
    }

    /// Create a new [`GzipWriter`] a custom write buffer size.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    /// * `capacity` - write buffer size
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(
            inner,
            GzipWriterOptions {
                capacity,
                ..GzipWriterOptions::default()
            },
        )
    }

    /// Create a new [`GzipWriter`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    /// * `options` - writer options
    pub fn with_options(inner: T, options: GzipWriterOptions) -> Self {
        Self {
            inner: Some(inner),
            deflate: Deflate::new(options.compression_level, options.expect_header, options.window_bits),
            member_started: false,
            buf: vec![0; options.capacity],
            buf_pos: 0,
            level: options.compression_level,
            window_bits: options.window_bits,
        }
    }

    /// Unwrap this [`GzipWriter`], returning the underlying writer.
    ///
    /// Writes out buffer contents before returning the inner reader.
    pub fn into_inner(mut self) -> io::Result<T> {
        self.finish()?;
        self.flush()?;
        Ok(self.inner.take().unwrap())
    }

    /// Write compressed bytes with explicit Deflate flush options.
    ///
    /// Changing the default flush options can mess with the compression ratios
    /// and internal buffer states. In almost all cases, you should rather use
    /// [`Self::write()`], unless you know what you are doing.
    ///
    /// Using [`DeflateFlush::Finish`] with an empty input buffer is equivalent to
    /// calling [`Self::finish()`]. Using [`DeflateFlush::NoFlush`] is the same as
    /// simply calling [`Self::write()`].
    ///
    /// Does not call [`Self::flush()`].
    ///
    /// # Arguments
    ///
    /// * `buf` - bytes to compress
    /// * `flush` - Deflate flush options
    ///
    /// # Returns
    ///
    /// Number of bytes consumed from `buf`.
    pub fn write_with_flush_opt(&mut self, buf: &[u8], flush: DeflateFlush) -> io::Result<usize> {
        let mut consumed = 0usize;
        let inner = self.inner.as_mut().unwrap();
        self.member_started = true;

        loop {
            let total_in = self.deflate.total_in();
            let total_out = self.deflate.total_out();

            let status = self
                .deflate
                .compress(&buf[consumed..], &mut self.buf[self.buf_pos..], flush)
                .map_err(|e| {
                    io::Error::new(io::ErrorKind::InvalidData, format!("Gzip compression error: {}", e.as_str()))
                })?;

            let in_delta = (self.deflate.total_in() - total_in) as usize;
            let out_delta = (self.deflate.total_out() - total_out) as usize;

            consumed += in_delta;
            self.buf_pos += out_delta;
            let stream_end = matches!(status, zlib_rs::Status::StreamEnd);

            // Write buffer stream if full or stream end
            if self.buf_pos == self.buf.len() || stream_end {
                inner.write_all(&self.buf[..self.buf_pos])?;
                self.buf_pos = 0;
            }

            // Break if all bytes consumed or stream end
            if stream_end {
                self.deflate = Deflate::new(self.level, true, self.window_bits);
                break;
            } else if consumed == buf.len() && !matches!(flush, DeflateFlush::Finish) {
                break;
            } else if in_delta == 0 && out_delta == 0 {
                return Err(io::Error::new(io::ErrorKind::WriteZero, "Deflate made no progress"));
            }
        }

        Ok(consumed)
    }

    /// Change the compression level for the next stream member.
    ///
    /// Has no effect until [`Self::finish()`] is called.
    ///
    /// # Arguments
    ///
    /// * `level` - new compression level (1-9)
    pub fn set_level(&mut self, level: i32) {
        self.level = level;
    }
}

impl_stream_from_path!(GzipWriter, GzipWriterOptions, create);

impl<T: _Write> WarcWrite for GzipWriter<T> {
    impl_to_any_methods!();

    fn finish(&mut self) -> io::Result<()> {
        if !self.member_started {
            return Ok(());
        }
        self.write_with_flush_opt(&[], DeflateFlush::Finish)?;
        self.member_started = false;
        Ok(())
    }
}

impl<T: _Write> io::Write for GzipWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_with_flush_opt(buf, DeflateFlush::NoFlush)
    }

    fn flush(&mut self) -> io::Result<()> {
        let inner = self.inner.as_mut().unwrap();
        if self.buf_pos > 0 {
            inner.write_all(&self.buf[..self.buf_pos])?;
            self.buf_pos = 0;
        }
        inner.flush()
    }
}

// noinspection DuplicatedCode
impl<T: _Write> Drop for GzipWriter<T> {
    fn drop(&mut self) {
        if self.inner.is_some() {
            self.finish().ok();
            self.flush().ok();
        }
    }
}

// ===========================================================
// Tests
// ===========================================================

#[cfg(test)]
#[path = "gzip_test.rs"]
mod gzip_test;
