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
use brotli::enc::BrotliEncoderParams;

fn compress(mut data: &[u8]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(data.len());
    brotli::BrotliCompress(&mut data, &mut out, &BrotliEncoderParams::default()).map_err(|e| io::Error::other(e))?;
    Ok(out)
}

fn decompress(mut data: &[u8], expected_len: usize) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(expected_len);
    brotli::BrotliDecompress(&mut data, &mut out).map_err(|e| io::Error::other(e))?;
    Ok(out)
}

#[test]
fn brotli_compress_decompress_roundtrip_validation() -> io::Result<()> {
    test_compress_decompress_roundtrip_validation(compress, decompress)
}

// ===========================================================
// Generic tests.
// ===========================================================

#[test]
fn brotli_reader_new_read_seek_and_stream_position() -> io::Result<()> {
    test_reader_new_read_seek_and_stream_position(compress, BrotliReader::new)
}

#[test]
fn brotli_reader_with_capacity_fill_buf_consume_and_into_inner() -> io::Result<()> {
    test_reader_with_capacity_fill_buf_consume_and_into_inner(
        compress,
        BrotliReader::with_capacity,
        BrotliReader::into_inner,
    )
}

#[test]
fn brotli_reader_with_tiny_capacity_handles_many_refills() -> io::Result<()> {
    test_reader_with_tiny_capacity_handles_many_refills(compress, BrotliReader::with_capacity)
}

#[test]
fn brotli_reader_reads_to_eof_after_external_compression() -> io::Result<()> {
    test_reader_reads_to_eof_after_external_compression(compress, BrotliReader::new)
}

#[test]
fn gzip_reader_inner_seek_inner_stream_position_and_member_tracking() -> io::Result<()> {
    test_reader_inner_seek_inner_stream_position_and_member_tracking(compress, BrotliReader::new, false)
}

#[test]
fn brotli_writer_new_write_and_into_inner_roundtrip() -> io::Result<()> {
    test_writer_new_write_and_into_inner_roundtrip(decompress, BrotliWriter::new, BrotliWriter::into_inner)
}

#[test]
fn brotli_writer_drop_finishes_and_flushes_stream() -> io::Result<()> {
    test_writer_drop_finishes_and_flushes_stream(decompress, BrotliWriter::with_capacity)
}

#[test]
fn brotli_writer_propagates_inner_flush_errors() -> io::Result<()> {
    test_writer_propagates_inner_flush_errors(BrotliWriter::with_capacity)
}

#[test]
fn brotli_writer_propagates_inner_write_errors() -> io::Result<()> {
    test_writer_propagates_inner_write_errors(BrotliWriter::with_capacity)
}
