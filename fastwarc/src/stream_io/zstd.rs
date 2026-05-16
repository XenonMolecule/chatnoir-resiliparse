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

use crate::stream_io::{CompressingWriter, DecompressingReader, ReadSeek, impl_stream_from_path};
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use zstd::stream::{Decoder, Encoder};

// ===========================================================
// ZstdReader
// ===========================================================

/// Reader for Brotli-compressed streams.
pub struct ZstdReader<T: ReadSeek> {
    inner: Option<BufReader<Decoder<'static, BufReader<T>>>>,
    stream_pos: u64,
    member_pos: u64,
}

/// Options for constructing a new [`ZstdReader`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
#[derive(Debug, Copy, Clone)]
pub struct ZstdReaderOptions {
    pub capacity: usize,
}

impl Default for ZstdReaderOptions {
    fn default() -> Self {
        Self { capacity: 4096 }
    }
}

impl<T: ReadSeek> ZstdReader<T> {
    /// Create a new [`ZstdReader`].
    ///
    /// Allocates an internal buffer holding chunks of the uncompressed inner stream.
    ///
    /// The default buffer size is 4096 bytes. For custom buffer sizes, use [`Self::with_capacity()`].
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, ZstdReaderOptions::default())
    }

    /// Create a new [`ZstdReader`] with a given buffer capacity.
    ///
    /// Allocates an internal buffer holding chunks of the uncompressed inner stream.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `capacity` - internal buffer size
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(inner, ZstdReaderOptions { capacity })
    }

    /// Create a new [`ZstdReader`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `options` - reader options
    pub fn with_options(mut inner: T, options: ZstdReaderOptions) -> Self {
        let member_pos = inner.stream_position().unwrap_or(0);
        let decoder = BufReader::with_capacity(
            options.capacity,
            Decoder::new(inner)
                .expect("Failed to create Zstd decoder.")
                .single_frame(),
        );
        Self {
            inner: Some(decoder),
            stream_pos: 0,
            member_pos,
        }
    }

    /// Unwraps this [`ZstdReader`], returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost.
    pub fn into_inner(self) -> T {
        self.inner.unwrap().into_inner().finish().into_inner()
    }
}

impl_stream_from_path!(ZstdReader, ZstdReaderOptions);

impl<T: ReadSeek> io::Read for ZstdReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = match self.inner.as_mut().unwrap().read(buf) {
            Ok(0) if !self.inner.as_mut().unwrap().get_ref().get_ref().buffer().is_empty() => {
                // Frame end: Reset Decoder and read again (keep self.stream_pos counting up).
                let old_pos = self.stream_pos;
                self.inner_seek(SeekFrom::Current(0))?;
                self.stream_pos = old_pos;
                self.inner.as_mut().unwrap().read(buf)?
            }
            Ok(b) => b,
            Err(e) => return Err(e),
        };
        self.stream_pos += n as u64;
        Ok(n)
    }
}

impl<T: ReadSeek> Seek for ZstdReader<T> {
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

impl<T: ReadSeek> DecompressingReader for ZstdReader<T> {
    fn inner_seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut inner = self.inner.take().unwrap().into_inner().finish();
        let new_pos = inner.seek(pos)?;
        self.inner = Some(BufReader::new(Decoder::with_buffer(inner)?));
        self.member_pos = new_pos;
        self.stream_pos = 0;
        Ok(new_pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.inner.as_mut().unwrap().get_mut().get_mut().stream_position()
    }

    fn member_start_position(&mut self) -> io::Result<u64> {
        Ok(self.member_pos)
    }
}

impl<T: ReadSeek> BufRead for ZstdReader<T> {
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
// ZstdWriter
// ===========================================================

/// Writer for LZ4-compressed streams.
pub struct ZstdWriter<T: Write> {
    inner: Option<Encoder<'static, BufWriter<T>>>,
    options: ZstdWriterOptions,
    frame_started: bool,
}

/// Options for constructing a new [`ZstdWriter`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
/// * `level` - compression level
/// * `include_checksum` - include checksums at the end of frames
/// * `multithread_workers` - number of threads to use for compression (0 to disable)
#[derive(Debug, Copy, Clone)]
pub struct ZstdWriterOptions {
    pub capacity: usize,
    pub level: i32,
    pub include_checksum: bool,
    pub multithread_workers: u32,
}

impl Default for ZstdWriterOptions {
    fn default() -> Self {
        Self {
            capacity: 8192,
            level: 3,
            include_checksum: false,
            multithread_workers: 4,
        }
    }
}

impl<T: Write> ZstdWriter<T> {
    /// Create a new [`ZstdWriter`].
    ///
    /// Maintains a small write buffer to temporarily store compressed data before flushing them
    /// to the underlying stream. The default buffer size is 8192 bytes. Use [`Self::with_capacity()`]
    /// for custom buffer sizes.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, ZstdWriterOptions::default())
    }

    /// Create a new [`ZstdWriter`] a custom write buffer size.
    ///
    /// Maintains a small write buffer to temporarily store compressed data before flushing them
    /// to the underlying stream.
    ///
    /// The default compression level is 9 (best). Use [`Self::with_capacity_comp_level()`] for custom
    /// compression levels.
    ///
    /// # Arguments
    ///
    /// * `capacity` - write buffer size
    /// * `inner` - inner stream to write compressed output to
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(
            inner,
            ZstdWriterOptions {
                capacity,
                ..ZstdWriterOptions::default()
            },
        )
    }

    fn new_with_options(inner: BufWriter<T>, options: ZstdWriterOptions) -> io::Result<Encoder<'static, BufWriter<T>>> {
        let mut encoder = Encoder::new(inner, options.level)?;
        encoder.multithread(options.multithread_workers)?;
        encoder.include_checksum(options.include_checksum)?;
        Ok(encoder)
    }

    /// Create a new [`ZstdWriter`] a the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    /// * `options` - writer options
    pub fn with_options(inner: T, options: ZstdWriterOptions) -> Self {
        Self {
            inner: Some(
                Self::new_with_options(BufWriter::new(inner), options).expect("Failed to set Zstd encoder options."),
            ),
            options,
            frame_started: false,
        }
    }

    /// Unwraps this [`ZstdWriter`], returning the underlying writer.
    ///
    /// Writes out buffer contents before returning the inner reader.
    pub fn into_inner(mut self) -> io::Result<T> {
        let mut writer = self.inner.take().unwrap().finish()?.into_inner()?;
        writer.flush()?;
        Ok(writer)
    }
}

impl_stream_from_path!(ZstdWriter, ZstdWriterOptions);

impl<T: Write> CompressingWriter for ZstdWriter<T> {
    fn finish(&mut self) -> io::Result<()> {
        if !self.frame_started {
            return Ok(());
        }
        let inner = self.inner.take().unwrap().finish()?;
        self.inner = Some(Self::new_with_options(inner, self.options)?);
        self.frame_started = false;
        Ok(())
    }
}

impl<T: Write> Write for ZstdWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.frame_started = true;
        self.inner.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.as_mut().unwrap().flush()?;
        Ok(())
    }
}

impl<T: Write> Drop for ZstdWriter<T> {
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

#[cfg(test)]
#[path = "zstd_test.rs"]
mod zstd_test;
