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

use crate::stream_io::{CompressingStream, DecompressingStream, ReadSeek};
use lz4_flex::frame::{FrameDecoder, FrameEncoder};
use std::io;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};

// ===========================================================
// Lz4Reader
// ===========================================================

/// Reader for LZ4-compressed streams.
pub struct Lz4Reader<T: ReadSeek> {
    inner: Option<FrameDecoder<BufReader<T>>>,
}

impl<T: ReadSeek> Lz4Reader<T> {
    /// Create a new [`Lz4Reader`].
    ///
    /// Allocates an internal buffer holding chunks of the uncompressed inner stream.
    ///
    /// The default buffer size is 4096 bytes. For custom buffer sizes, use [`Self::with_capacity()`].
    pub fn new(inner: T) -> Self {
        Self::with_capacity(4096, inner)
    }

    /// Create a new [`Lz4Reader`] with a given buffer capacity.
    ///
    /// Allocates an internal buffer holding chunks of the uncompressed inner stream.
    ///
    /// # Arguments
    ///
    /// * `inner` - input (inner) stream to read from
    pub fn with_capacity(capacity: usize, inner: T) -> Self {
        Self {
            inner: Some(FrameDecoder::new(BufReader::with_capacity(capacity, inner))),
        }
    }

    /// Unwraps this [`Lz4Reader`], returning the underlying reader.
    ///
    /// Note that any leftover data in the internal buffer is lost.
    pub fn into_inner(self) -> T {
        self.inner.unwrap().into_inner().into_inner()
    }
}

impl<T: ReadSeek> io::Read for Lz4Reader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.as_mut().unwrap().read(buf)
    }
}

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
        super::_forward_seek(self, pos)
    }

    /// Returns the current seek position from the start of the decompressed output stream.
    fn stream_position(&mut self) -> io::Result<u64> {
        // TODO: implement this
        // Ok(self.stream_pos)
        self.seek(SeekFrom::Current(0))
    }
}

impl<T: ReadSeek> DecompressingStream for Lz4Reader<T> {
    fn inner_seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut inner = self.inner.take().unwrap().into_inner();
        let new_pos = inner.seek(pos)?;
        self.inner = Some(FrameDecoder::new(inner));
        Ok(new_pos)
    }

    fn inner_stream_position(&mut self) -> io::Result<u64> {
        // TODO: implement this
        Ok(0)
    }

    fn member_start_position(&mut self) -> io::Result<u64> {
        // TODO: implement this
        Ok(0)
    }
}

impl<T: ReadSeek> BufRead for Lz4Reader<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.as_mut().unwrap().fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.inner.as_mut().unwrap().consume(amount);
    }
}

// ===========================================================
// Lz4Writer
// ===========================================================

pub struct Lz4Writer<T: Write> {
    inner: Option<FrameEncoder<T>>,
}

impl<T: Write> Lz4Writer<T> {
    /// Create a new [`Lz4Writer`].
    ///
    /// # Arguments
    ///
    /// * `inner` - inner stream to write compressed output to
    pub fn new(inner: T) -> Self {
        Self {
            inner: Some(FrameEncoder::new(inner)),
        }
    }

    /// Unwraps this [`Lz4Writer`], returning the underlying writer.
    ///
    /// Writes out buffer contents before returning the inner reader.
    pub fn into_inner(mut self) -> io::Result<T> {
        self.finish()?;
        self.flush()?;
        Ok(self.inner.take().unwrap().into_inner())
    }
}

impl<T: Write> CompressingStream for Lz4Writer<T> {
    fn finish(&mut self) -> io::Result<usize> {
        let inner = self.inner.take().unwrap().finish()?;
        self.inner = Some(FrameEncoder::new(inner));
        // TODO: implement this
        Ok(0)
    }
}

impl<T: Write> Write for Lz4Writer<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.as_mut().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.as_mut().unwrap().flush()
    }
}

impl<T: Write> Drop for Lz4Writer<T> {
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
#[path = "lz4_test.rs"]
mod lz4_test;
