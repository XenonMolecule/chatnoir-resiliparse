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

use crate::stream_io::traits::{ReadSeek, WarcRead, WarcWrite};
use crate::stream_io::{impl_stream_from_path, impl_to_any_methods};
use lz4_flex::frame::{FrameDecoder, FrameEncoder};
use std::any::Any;
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};

// ===========================================================
// Lz4Reader
// ===========================================================

/// Reader for LZ4-compressed streams.
pub struct Lz4Reader<T: ReadSeek> {
    inner: Option<FrameDecoder<BufReader<T>>>,
    stream_pos: u64,
    frame_start_pos: Option<u64>,
}

/// Options for constructing a new [`Lz4Reader`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Lz4ReaderOptions {
    pub capacity: usize,
}

impl Default for Lz4ReaderOptions {
    fn default() -> Self {
        Self { capacity: 4096 }
    }
}

impl<T: ReadSeek> Lz4Reader<T> {
    /// Create a new [`Lz4Reader`].
    ///
    /// Allocates an internal buffer holding chunks of the inner stream.
    ///
    /// The default buffer size is 4096 bytes. For custom buffer sizes, use [`Self::with_capacity()`].
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, Lz4ReaderOptions::default())
    }

    /// Create a new [`Lz4Reader`] with a given buffer capacity.
    ///
    /// Allocates an internal buffer holding chunks of the inner stream.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `capacity` - internal buffer size
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(inner, Lz4ReaderOptions { capacity })
    }

    /// Create a new [`Lz4Reader`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    /// * `options` - reader options
    pub fn with_options(inner: T, options: Lz4ReaderOptions) -> Self {
        Self {
            inner: Some(FrameDecoder::new(BufReader::with_capacity(options.capacity, inner))),
            stream_pos: 0,
            frame_start_pos: None,
        }
    }

    /// Unwrap this [`Lz4Reader`], returning the underlying reader.
    ///
    /// Discards input buffers, so continued reads on the unwrapped stream may fail.
    pub fn into_inner(self) -> T {
        self.inner.unwrap().into_inner().into_inner()
    }

    /// Internal: Lazily load the frame start position from the inner stream (needs to be run as early as possible!)
    fn ensure_frame_start_pos(&mut self) -> io::Result<u64> {
        if self.frame_start_pos.is_none() {
            self.frame_start_pos = Some(self.inner.as_mut().unwrap().get_mut().stream_position()?);
        }
        Ok(self.frame_start_pos.unwrap())
    }

    /// Internal: Helper for syncing decoder state with LZ4 frame boundaries.
    /// This needs to be called in `fill_buf()` to not terminate early and in
    /// `stream_position()` to not get incorrect values (usually off by the four
    /// bytes at the end of a frame).
    fn sync_next_frame(&mut self) -> io::Result<()> {
        self.ensure_frame_start_pos()?;
        if self.stream_pos == 0 {
            return Ok(());
        }

        let buf_empty = self.inner.as_mut().unwrap().fill_buf()?.is_empty();
        if !buf_empty {
            return Ok(());
        }

        // Frame end, but more data available in the underlying stream.
        // Reset the decoder and advance member_pos.
        if buf_empty && !self.inner.as_mut().unwrap().get_mut().fill_buf()?.is_empty() {
            let old_pos = self.stream_pos;
            let new_pos = self.inner_seek(SeekFrom::Current(0))?;
            self.stream_pos = old_pos;
            self.frame_start_pos = Some(new_pos);
        }

        Ok(())
    }
}

impl_stream_from_path!(Lz4Reader, Lz4ReaderOptions, open);

// noinspection DuplicatedCode
impl<T: ReadSeek> io::Read for Lz4Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.fill_buf()?.read(buf)?;
        self.consume(n);
        Ok(n)
    }
}

// noinspection DuplicatedCode
impl<T: ReadSeek> Seek for Lz4Reader<T> {
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

impl<T: ReadSeek> WarcRead for Lz4Reader<T> {
    impl_to_any_methods!();

    fn inner_seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut inner = self.inner.take().unwrap().into_inner();
        let new_pos = inner.seek(pos)?;
        self.inner = Some(FrameDecoder::new(inner));
        self.frame_start_pos = Some(new_pos);
        self.stream_pos = 0;
        Ok(new_pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        self.sync_next_frame()?;
        self.inner.as_mut().unwrap().get_mut().stream_position()
    }

    fn frame_start_position(&mut self) -> io::Result<Option<u64>> {
        Ok(self.frame_start_pos)
    }

    fn is_stream_decoder(&self) -> bool {
        true
    }
}

impl<T: ReadSeek> BufRead for Lz4Reader<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.sync_next_frame()?;
        self.inner.as_mut().unwrap().fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.stream_pos += amount as u64;
        self.inner.as_mut().unwrap().consume(amount);
    }
}

// ===========================================================
// Lz4Writer
// ===========================================================

/// Writer for LZ4-compressed streams.
pub struct Lz4Writer<T: Write + 'static> {
    inner: Option<FrameEncoder<BufWriter<T>>>,
    frame_started: bool,
}

/// Options for constructing a new [`Lz4Writer`].
///
/// # Options
///
/// * `capacity` - sets the internal buffer size.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Lz4WriterOptions {
    pub capacity: usize,
}

impl Default for Lz4WriterOptions {
    fn default() -> Self {
        Self { capacity: 8192 }
    }
}

impl<T: Write + 'static> Lz4Writer<T> {
    /// Create a new [`Lz4Writer`].
    ///
    /// Maintains a small write buffer to temporarily store compressed data before flushing them
    /// to the underlying stream. The default buffer size is 8192 bytes. Use [`Self::with_capacity()`]
    /// for custom buffer sizes.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    pub fn new(inner: T) -> Self {
        Self::with_options(inner, Lz4WriterOptions::default())
    }

    /// Create a new [`Lz4Writer`] a custom write buffer size.
    ///
    /// Maintains a small write buffer to temporarily store compressed data before flushing them
    /// to the underlying stream.=
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    /// * `capacity` - write buffer size
    pub fn with_capacity(inner: T, capacity: usize) -> Self {
        Self::with_options(inner, Lz4WriterOptions { capacity })
    }

    /// Create a new [`Lz4Writer`] with the supplied options.
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    /// * `options` - writer options
    pub fn with_options(inner: T, options: Lz4WriterOptions) -> Self {
        Self {
            inner: Some(FrameEncoder::new(BufWriter::with_capacity(options.capacity, inner))),
            frame_started: false,
        }
    }

    /// Unwrap this [`Lz4Writer`], returning the underlying writer.
    ///
    /// Writes out buffer contents before returning the inner reader.
    pub fn into_inner(mut self) -> io::Result<T> {
        self.finish()?;
        let mut writer = self.inner.take().unwrap().into_inner();
        writer.flush()?;
        Ok(writer.into_inner()?)
    }
}

impl_stream_from_path!(Lz4Writer, Lz4WriterOptions, create);

impl<T: Write + 'static> WarcWrite for Lz4Writer<T> {
    impl_to_any_methods!();

    fn finish(&mut self) -> io::Result<()> {
        if !self.frame_started {
            return Ok(());
        }
        let inner = self.inner.take().unwrap().finish()?;
        self.frame_started = false;
        self.inner = Some(FrameEncoder::new(inner));
        Ok(())
    }
}

impl<T: Write + 'static> Write for Lz4Writer<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.frame_started = true;
        self.inner.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let inner = self.inner.as_mut().unwrap();
        inner.flush()?;
        inner.get_mut().flush()?;
        Ok(())
    }
}

// noinspection DuplicatedCode
impl<T: Write + 'static> Drop for Lz4Writer<T> {
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
#[path = "lz4_test.rs"]
mod lz4_test;
