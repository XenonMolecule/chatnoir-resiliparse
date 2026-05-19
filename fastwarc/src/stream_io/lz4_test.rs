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

use super::*;
use crate::stream_io::mod_test::helpers::*;
use lz4_flex::frame::{FrameDecoder, FrameEncoder};
use std::io::Read;

fn compress_frame(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut comp = FrameEncoder::new(Vec::new());
    comp.write_all(data)?;
    Ok(comp.finish()?)
}

fn decompress_frame(data: &[u8], expected_len: usize) -> io::Result<Vec<u8>> {
    let mut decomp = FrameDecoder::new(data);
    let mut out = vec![0u8; expected_len];
    decomp.read_exact(&mut out)?;
    Ok(out)
}

#[test]
fn lz4_compress_decompress_roundtrip_validation() -> io::Result<()> {
    test_compress_decompress_roundtrip_validation(compress_frame, decompress_frame)
}

// ===========================================================
// Generic tests.
// ===========================================================

#[test]
fn lz4_reader_new_read_seek_and_stream_position() -> io::Result<()> {
    test_reader_new_read_seek_and_stream_position(compress_frame, Lz4Reader::new)
}

#[test]
fn lz4_reader_with_capacity_fill_buf_consume_and_into_inner() -> io::Result<()> {
    test_reader_with_capacity_fill_buf_consume_and_into_inner(
        compress_frame,
        Lz4Reader::with_capacity,
        Lz4Reader::into_inner,
    )
}

#[test]
fn lz4_reader_with_tiny_capacity_handles_many_refills() -> io::Result<()> {
    test_reader_with_tiny_capacity_handles_many_refills(compress_frame, Lz4Reader::with_capacity)
}

#[test]
fn lz4_reader_reads_to_eof_after_external_compression() -> io::Result<()> {
    test_reader_reads_to_eof_after_external_compression(compress_frame, Lz4Reader::new)
}

#[test]
fn lz4_reader_inner_seek_inner_stream_position_and_member_tracking() -> io::Result<()> {
    test_reader_inner_seek_inner_stream_position_and_member_tracking(compress_frame, Lz4Reader::new, true)
}

#[test]
fn lz4_writer_new_write_and_into_inner_roundtrip() -> io::Result<()> {
    test_writer_new_write_and_into_inner_roundtrip(decompress_frame, Lz4Writer::new, Lz4Writer::into_inner)
}

#[test]
fn lz4_writer_drop_finishes_and_flushes_stream() -> io::Result<()> {
    test_writer_drop_finishes_and_flushes_stream(decompress_frame, Lz4Writer::with_capacity)
}

#[test]
fn lz4_writer_propagates_inner_flush_errors() -> io::Result<()> {
    test_writer_propagates_inner_flush_errors(Lz4Writer::with_capacity)
}

#[test]
fn lz4_writer_propagates_inner_write_errors() -> io::Result<()> {
    test_writer_propagates_inner_write_errors(Lz4Writer::with_capacity)
}
