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

use crate::stream_io::{
    CompressingWriter, DecompressingReader, ReadSeek, WarcRead, WarcWrite, impl_fastwarc_stream, impl_stream_from_path,
};
use std::any::Any;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom, Write};

// ===========================================================
// Decoder for HTTP Transfer-Encoding: chunked
// ===========================================================

/// Reader for chunked HTTP streams.
pub struct ChunkedReader<T: ReadSeek> {
    inner: Option<BufReader<T>>,
    stream_pos: u64,
    chunk_size: usize,
    chunk_read: usize,
}

/// Options for constructing a new [`ChunkedReader`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
#[derive(Debug, Copy, Clone)]
pub struct ChunkedReaderOptions {
    pub capacity: usize,
}

impl Default for ChunkedReaderOptions {
    fn default() -> Self {
        Self { capacity: 8192 }
    }
}

impl<T: ReadSeek> ChunkedReader<T> {
    /// Create a new [`ChunkedReader`].
    ///
    /// Allocates an internal buffer holding chunks of the chunked inner stream.
    ///
    /// The default buffer size is 8192 bytes. For custom buffer sizes, use [`Self::with_capacity()`].
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, ChunkedReaderOptions::default())
    }

    /// Create a new [`ChunkedReader`] with a given buffer capacity.
    ///
    /// Allocates an internal buffer holding parts of the chunked inner stream.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `capacity` - internal buffer size
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(inner, ChunkedReaderOptions { capacity })
    }

    /// Create a new [`ChunkedReader`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `options` - reader options
    pub fn with_options(inner: T, options: ChunkedReaderOptions) -> Self {
        Self {
            inner: Some(BufReader::with_capacity(options.capacity, inner)),
            stream_pos: 0,
            chunk_size: 0,
            chunk_read: 0,
        }
    }

    /// Unwraps this [`ChunkedReader`], returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost.
    pub fn into_inner(self) -> T {
        self.inner.unwrap().into_inner()
    }
}

impl_fastwarc_stream!(ChunkedReader, WarcRead, ReadSeek);
impl_stream_from_path!(ChunkedReader, ChunkedReaderOptions);

impl<T: ReadSeek> ChunkedReader<T> {
    fn read_chunk(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let inner = self.inner.as_mut().unwrap();

        // New chunk
        if self.chunk_read == 0 && self.chunk_size == 0 {
            let mut header = Vec::with_capacity(8);
            inner.read_until(b'\n', &mut header)?;
            if header.is_empty() {
                return Ok(0);
            }
            let header = String::from_utf8_lossy(&header);
            let header = header.trim_ascii();
            self.chunk_size = usize::from_str_radix(&header, 16)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid chunk header: {}", header)))?;

            // EOF
            if self.chunk_size == 0 {
                return Ok(0);
            }
        }

        // Within chunk
        if self.chunk_read < self.chunk_size {
            let remaining = self.chunk_size - self.chunk_read;
            let in_buf = self.fill_buf()?;
            let n = buf.len().min(remaining).min(in_buf.len());
            buf.copy_from_slice(&in_buf[..n]);
            self.chunk_read += n;
            self.consume(n);
            return Ok(n);
        }

        Ok(0)
    }
}

impl<T: ReadSeek> io::Read for ChunkedReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.read_chunk(buf)?;
        if n > 0 || self.chunk_size == 0 {
            return Ok(n);
        }

        // End of chunk, consume trailing \r\n and read again
        if self.chunk_read == self.chunk_size {
            let l = self.fill_buf()?.len().min(1);
            self.consume(l);
            let l = self.fill_buf()?.len().min(1);
            self.consume(l);
            self.chunk_read = 0;
            self.chunk_size = 0;
        }
        self.read_chunk(buf)
    }
}

impl<T: ReadSeek> Seek for ChunkedReader<T> {
    /// Seek to an offset, in bytes, in the output stream.
    ///
    /// Seeking in a chunked stream is not efficient with O(n) complexity,
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

    /// Returns the current seek position from the start of the output stream.
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.stream_pos)
    }
}

impl<T: ReadSeek> DecompressingReader for ChunkedReader<T> {
    fn inner_seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = self.inner.as_mut().unwrap().seek(pos)?;
        self.stream_pos = 0;
        self.chunk_size = 0;
        Ok(new_pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.inner.as_mut().unwrap().get_mut().stream_position()
    }
}

impl<T: ReadSeek> BufRead for ChunkedReader<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.as_mut().unwrap().fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.stream_pos += amount as u64;
        self.inner.as_mut().unwrap().consume(amount);
    }
}

// ===========================================================
// ChunkedWriter
// ===========================================================

/// Writer for chunked HTTP streams.
pub struct ChunkedWriter<T: Write + 'static> {
    inner: Option<T>,
    chunk_buffer: Vec<u8>,
    min_chunk_size: usize,
    stream_started: bool,
}

/// Options for constructing a new [`ChunkedWriter`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
#[derive(Debug, Copy, Clone)]
pub struct ChunkedWriterOptions {
    pub min_chunk_size: usize,
}

impl Default for ChunkedWriterOptions {
    fn default() -> Self {
        Self { min_chunk_size: 512 }
    }
}

impl<T: Write + 'static> ChunkedWriter<T> {
    /// Create a new [`ChunkedWriter`].
    ///
    /// Maintains a small write buffer to temporarily accumulate data in a chunk before flushing it
    /// to the underlying stream. The default minimum chunk size is 512 bytes. Use [`Self::with_chunk_size()`]
    /// for custom buffer sizes.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write chunked output to
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, ChunkedWriterOptions::default())
    }

    /// Create a new [`ChunkedWriter`] a custom write buffer size.
    ///
    /// Maintains a small write buffer to temporarily accumulate data in a chunk before flushing it
    /// to the underlying stream. The default minimum chunk size is 512 bytes.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write chunked output to
    /// * `min_chunk_size` - chunk write buffer size
    pub fn with_chunk_size(inner: T, min_chunk_size: usize) -> Self {
        Self::with_options(inner, ChunkedWriterOptions { min_chunk_size })
    }

    /// Create a new [`ChunkedWriter`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write chunked output to
    /// * `options` - writer options
    pub fn with_options(inner: T, options: ChunkedWriterOptions) -> Self {
        Self {
            inner: Some(inner),
            chunk_buffer: Vec::with_capacity(options.min_chunk_size),
            min_chunk_size: options.min_chunk_size,
            stream_started: false,
        }
    }

    /// Unwraps this [`ChunkedWriter`], returning the underlying writer.
    ///
    /// Writes out buffer contents before returning the inner reader.
    pub fn into_inner(mut self) -> io::Result<T> {
        self.finish()?;
        self.flush()?;
        Ok(self.inner.take().unwrap())
    }
}

impl_fastwarc_stream!(ChunkedWriter, WarcWrite, Write + 'static);
impl_stream_from_path!(ChunkedWriter, ChunkedWriterOptions);

impl<T: Write + 'static> ChunkedWriter<T> {
    fn write_chunk_buffer(&mut self) -> io::Result<usize> {
        let inner = self.inner.as_mut().unwrap();
        let header = format!("{:x}\r\n", self.chunk_buffer.len()).into_bytes();
        inner.write_all(&header)?;
        self.chunk_buffer.extend_from_slice(b"\r\n");
        inner.write_all(&self.chunk_buffer)?;
        let n = header.len() + self.chunk_buffer.len();
        self.chunk_buffer.clear();
        Ok(n)
    }
}

impl<T: Write + 'static> CompressingWriter for ChunkedWriter<T> {
    fn finish(&mut self) -> io::Result<()> {
        if self.chunk_buffer.is_empty() && !self.stream_started {
            return Ok(());
        }
        self.write_chunk_buffer()?;
        self.inner.as_mut().unwrap().write_all(b"0\r\n\r\n")?;
        self.stream_started = false;
        Ok(())
    }
}

impl<T: Write + 'static> Write for ChunkedWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream_started = true;
        self.chunk_buffer.extend_from_slice(buf);
        if self.chunk_buffer.len() >= self.min_chunk_size {
            self.write_chunk_buffer()?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.as_mut().unwrap().flush()
    }
}

impl<T: Write + 'static> Drop for ChunkedWriter<T> {
    // noinspection ALL
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

// #[cfg(test)]
// #[path = "chunked_test.rs"]
// mod chunked_test;
