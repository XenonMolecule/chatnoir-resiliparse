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
    /// Get an [`Any`] reference to this [`DecompressingRead`].
    fn as_any(&self) -> &dyn Any;

    /// Get a mutable [`Any`] reference to this [`DecompressingRead`].
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert the [`DecompressingRead`] into [`Any`].
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Seek to an offset, in bytes, in a wrapped inner stream.
    /// If the stream isn't wrapped, or if the inner stream doesn't perform
    /// decompression or buffering, then this is the same as [`io::Seek::seek()`].
    ///
    /// Seeking on the inner stream may reset the state of the decompressor.
    /// It is up to the user to seek valid positions from which decompression
    /// can be resumed.
    fn inner_seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>;

    /// Return the current seek position from the start of a wrapped inner stream.
    /// If the stream isn't wrapped, or if the inner stream doesn't perform
    /// decompression or buffering, then this is the same as [`io::Seek::stream_position()`].
    ///
    /// The returned position is the logical position on the compressed inner stream,
    /// which does not consider input buffer sizes. The physical reader position may be
    /// larger than this.
    fn inner_stream_position(&mut self) -> io::Result<u64>;
}

pub trait WarcWrite: io::Write + Any + 'static {
    /// Get an [`Any`] reference to this [`DecompressingRead`].
    fn as_any(&self) -> &dyn Any;

    /// Get a mutable [`Any`] reference to this [`DecompressingRead`].
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Convert the [`DecompressingRead`] into [`Any`].
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    /// Finish the stream and reset the inner writer.
    ///
    /// If the writer uses a frame format and supports multi-frame streams, it
    /// can be used again after this to start a new frame. Otherwise, writing
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
// Compressors and decompressors
// ===========================================================

/// Trait for [`io::Read`] stream implementations reading from
/// compressed input streams.
pub trait DecompressingRead: WarcRead {
    /// Return the start position, in bytes, of the current member / frame
    /// in the compressed inner stream. If the compression format does not support
    /// multi-member streams, this is always the beginning of the stream.
    ///
    /// # Returns
    ///
    /// Position, in bytes, of the current member
    fn frame_start_position(&mut self) -> io::Result<u64> {
        Ok(0)
    }
}

/// Trait for [`io::Write`] stream implementations that write compressed data
/// onto an output stream.
pub trait CompressingWrite: WarcWrite {}
