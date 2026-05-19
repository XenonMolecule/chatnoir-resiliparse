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
use zstd::stream::{decode_all, encode_all};

fn compress(data: &[u8]) -> io::Result<Vec<u8>> {
    encode_all(data, 3)
}

fn decompress(data: &[u8], _expected_len: usize) -> io::Result<Vec<u8>> {
    decode_all(data)
}

#[test]
fn zstd_compress_decompress_roundtrip_validation() -> io::Result<()> {
    test_compress_decompress_roundtrip_validation(compress, decompress)
}

// ===========================================================
// Generic tests.
// ===========================================================

#[test]
fn zstd_reader_new_read_seek_and_stream_position() -> io::Result<()> {
    test_reader_new_read_seek_and_stream_position(compress, ZstdReader::new)
}

#[test]
fn zstd_reader_with_capacity_fill_buf_consume() -> io::Result<()> {
    test_reader_with_capacity_fill_buf_consume(compress, ZstdReader::with_capacity)
}

#[test]
fn zstd_reader_inner_stream_position_and_into_inner() -> io::Result<()> {
    test_reader_inner_stream_position_and_into_inner(compress, ZstdReader::with_capacity, ZstdReader::into_inner)
}

#[test]
fn zstd_reader_with_tiny_capacity_handles_many_refills() -> io::Result<()> {
    test_reader_with_tiny_capacity_handles_many_refills(compress, ZstdReader::with_capacity)
}

#[test]
fn zstd_reader_reads_to_eof_after_external_compression() -> io::Result<()> {
    test_reader_reads_to_eof_after_external_compression(compress, ZstdReader::new)
}

#[test]
fn zstd_reader_inner_seek_inner_stream_position_and_member_tracking() -> io::Result<()> {
    test_reader_inner_seek_inner_stream_position_and_member_tracking(compress, ZstdReader::new, true)
}

#[test]
fn zstd_writer_new_write_and_into_inner_roundtrip() -> io::Result<()> {
    test_writer_new_write_and_into_inner_roundtrip(decompress, ZstdWriter::new, ZstdWriter::into_inner)
}

#[test]
fn zstd_writer_drop_finishes_and_flushes_stream() -> io::Result<()> {
    test_writer_drop_finishes_and_flushes_stream(decompress, ZstdWriter::with_capacity)
}

#[test]
fn zstd_writer_propagates_inner_flush_errors() -> io::Result<()> {
    test_writer_propagates_inner_flush_errors(ZstdWriter::with_capacity)
}

#[test]
fn zstd_writer_propagates_inner_write_errors() -> io::Result<()> {
    test_writer_propagates_inner_write_errors(ZstdWriter::with_capacity)
}
