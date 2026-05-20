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
use brotli::{CompressorWriter, Decompressor};
use std::any::Any;
use std::io::{self, BufRead, Seek, SeekFrom, Write};

// ===========================================================
// BrotliReader
// ===========================================================

/// Reader for Brotli-compressed streams.
pub struct BrotliReader<T: ReadSeek> {
    inner: Option<io::BufReader<Decompressor<T>>>,
    stream_pos: u64,
}

/// Options for constructing a new [`BrotliReader`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BrotliReaderOptions {
    pub capacity: usize,
}

impl Default for BrotliReaderOptions {
    fn default() -> Self {
        Self { capacity: 4096 }
    }
}

impl<T: ReadSeek> BrotliReader<T> {
    /// Create a new [`BrotliReader`].
    ///
    /// Allocates an internal buffer holding chunks of the inner stream.
    ///
    /// The default buffer size is 4096 bytes. For custom buffer sizes, use [`Self::with_capacity()`].
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, BrotliReaderOptions::default())
    }

    /// Create a new [`BrotliReader`] with a given buffer capacity.
    ///
    /// Allocates an internal buffer holding chunks of the inner stream.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `capacity` - internal buffer size
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(inner, BrotliReaderOptions { capacity })
    }

    /// Create a new [`BrotliReader`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `options` - reader options
    pub fn with_options(inner: T, options: BrotliReaderOptions) -> Self {
        Self {
            inner: Some(io::BufReader::with_capacity(options.capacity, Decompressor::new(inner, options.capacity))),
            stream_pos: 0,
        }
    }

    /// Unwraps this [`BrotliReader`], returning the underlying reader.
    ///.
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> T {
        self.inner.unwrap().into_inner().into_inner()
    }
}

impl_fastwarc_stream!(BrotliReader, WarcRead, ReadSeek);
impl_stream_from_path!(BrotliReader, BrotliReaderOptions);

impl<T: ReadSeek> io::Read for BrotliReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.as_mut().unwrap().read(buf)?;
        self.stream_pos += n as u64;
        Ok(n)
    }
}

impl<T: ReadSeek> Seek for BrotliReader<T> {
    /// Seek to an offset, in bytes, in the decompressed output stream.
    ///
    /// Seeking in a compressed stream is not efficient with O(n) complexity,
    /// and backwards seeking and seeking from the end are not supported.
    ///
    /// # Arguments
    ///
    /// `pos` - seek position
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.stream_pos = super::_forward_seek(self, pos)?;
        Ok(self.stream_pos)
    }

    /// Returns the current seek position from the start of the decompressed output stream.
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.stream_pos)
    }
}

impl<T: ReadSeek + 'static> DecompressingReader for BrotliReader<T> {
    fn inner_seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let capacity = self.inner.as_ref().unwrap().capacity();
        let mut inner = self.inner.take().unwrap().into_inner().into_inner();
        let new_pos = inner.seek(pos)?;
        self.inner = Some(io::BufReader::with_capacity(capacity, Decompressor::new(inner, capacity)));
        Ok(new_pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.inner.as_mut().unwrap().get_mut().get_mut().stream_position()
    }
}

impl<T: ReadSeek> BufRead for BrotliReader<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.as_mut().unwrap().fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        if let Some(inner) = self.inner.as_mut() {
            inner.consume(amount);
            self.stream_pos += amount as u64;
        }
    }
}

// ===========================================================
// BrotliWriter
// ===========================================================

/// Writer for Brotli-compressed streams.
pub struct BrotliWriter<T: Write + 'static> {
    inner: Option<CompressorWriter<T>>,
}

/// Options for constructing a new [`BrotliWriter`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BrotliWriterOptions {
    pub capacity: usize,
    pub quality: u32,
    pub lg_window_size: u32,
}

impl Default for BrotliWriterOptions {
    fn default() -> Self {
        Self {
            capacity: 8192,
            quality: 4,
            lg_window_size: 22,
        }
    }
}

impl_fastwarc_stream!(BrotliWriter, WarcWrite, Write + 'static);

impl<T: Write + 'static> BrotliWriter<T> {
    /// Create a new [`BrotliWriter`].
    ///
    /// Maintains a small write buffer to temporarily store compressed data before flushing them
    /// to the underlying stream. The default buffer size is 8192 bytes. Use [`Self::with_capacity()`]
    /// for custom buffer sizes.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, BrotliWriterOptions::default())
    }

    /// Create a new [`BrotliWriter`] a custom write buffer size.
    ///
    /// Maintains a small write buffer to temporarily store compressed data before flushing them
    /// to the underlying stream.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    /// * `capacity` - write buffer size
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(
            inner,
            BrotliWriterOptions {
                capacity,
                ..BrotliWriterOptions::default()
            },
        )
    }

    /// Create a new [`BrotliWriter`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    /// * `options` - writer options
    pub fn with_options(inner: T, options: BrotliWriterOptions) -> Self {
        Self {
            inner: Some(CompressorWriter::new(inner, options.capacity, options.quality, options.lg_window_size)),
        }
    }

    /// Unwraps this [`BrotliWriter`], returning the underlying writer.
    ///
    /// Writes out buffer contents before returning the inner reader.
    pub fn into_inner(mut self) -> io::Result<T> {
        let mut writer = self.inner.take().unwrap().into_inner();
        writer.flush()?;
        Ok(writer)
    }
}

impl_stream_from_path!(BrotliWriter, BrotliWriterOptions);

impl<T: Write + 'static> CompressingWriter for BrotliWriter<T> {
    fn finish(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: Write + 'static> Write for BrotliWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.as_mut().unwrap().flush()?;
        Ok(())
    }
}

impl<T: Write + 'static> Drop for BrotliWriter<T> {
    // noinspection ALL
    fn drop(&mut self) {
        if self.inner.is_some() {
            self.flush().ok();
        }
    }
}

// ===========================================================
// Tests
// ===========================================================

#[cfg(test)]
#[path = "brotli_test.rs"]
mod brotli_test;
