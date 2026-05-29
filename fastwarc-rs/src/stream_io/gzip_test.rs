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
use crate::stream_io::mod_test::*;
use crate::stream_io::traits::IntoWarcReader;
use std::io::{Cursor, Read};
use zlib_rs::{
    DeflateConfig, DeflateFlush, InflateConfig, ReturnCode, compress_bound, compress_slice, decompress_slice,
};

fn compress_member(data: &[u8]) -> io::Result<Vec<u8>> {
    let config = DeflateConfig {
        level: 9,
        window_bits: 15 + 16,
        ..DeflateConfig::default()
    };

    let mut compressed_buf = vec![0u8; compress_bound(data.len())];
    let (compressed, rc) = compress_slice(&mut compressed_buf, data, config);
    assert_eq!(rc, ReturnCode::Ok);

    Ok(compressed.to_vec())
}

fn decompress_member(data: &[u8], expected_len: usize) -> io::Result<Vec<u8>> {
    let config = InflateConfig { window_bits: 15 + 16 };

    let mut decompressed_buf = vec![0u8; expected_len];
    let (decompressed, rc) = decompress_slice(&mut decompressed_buf, data, config);
    assert_eq!(rc, ReturnCode::Ok);

    Ok(decompressed.to_vec())
}

#[test]
fn gzip_compress_decompress_roundtrip_validation() -> io::Result<()> {
    test_compress_decompress_roundtrip_validation(compress_member, decompress_member)
}

// ===========================================================
// Generic tests.
// ===========================================================

#[test]
fn gzip_reader_new_read_seek_and_stream_position() -> io::Result<()> {
    test_reader_new_read_seek_and_stream_position(compress_member, GzipReader::new)
}

#[test]
fn gzip_reader_with_capacity_fill_buf_consume() -> io::Result<()> {
    test_reader_with_capacity_fill_buf_consume(compress_member, GzipReader::with_capacity)
}

#[test]
fn gzip_reader_inner_stream_position_and_into_inner() -> io::Result<()> {
    test_reader_inner_stream_position_and_into_inner(compress_member, GzipReader::with_capacity, GzipReader::into_inner)
}

#[test]
fn gzip_reader_with_tiny_capacity_handles_many_refills() -> io::Result<()> {
    test_reader_with_tiny_capacity_handles_many_refills(compress_member, GzipReader::with_capacity)
}

#[test]
fn gzip_reader_reads_to_eof_after_external_compression() -> io::Result<()> {
    test_reader_reads_to_eof_after_external_compression(compress_member, GzipReader::new)
}

#[test]
fn gzip_reader_inner_seek_inner_stream_position_and_member_tracking() -> io::Result<()> {
    test_reader_inner_seek_inner_stream_position_and_member_tracking(compress_member, GzipReader::new)
}

#[test]
fn gzip_nested_warc_read() -> io::Result<()> {
    test_nested_warc_read(compress_member, |r| GzipReader::new(r).into_warc_reader())
}

#[test]
fn gzip_frame_start_position_in_sync() -> io::Result<()> {
    test_frame_start_position_in_sync(compress_member, GzipReader::new, 0)
}

#[test]
fn gzip_writer_new_write_and_into_inner_roundtrip() -> io::Result<()> {
    test_writer_new_write_and_into_inner_roundtrip(decompress_member, GzipWriter::new, GzipWriter::into_inner)
}

#[test]
fn gzip_writer_drop_finishes_and_flushes_stream() -> io::Result<()> {
    test_writer_drop_finishes_and_flushes_stream(decompress_member, GzipWriter::with_capacity)
}

#[test]
fn gzip_writer_propagates_inner_flush_errors() -> io::Result<()> {
    test_writer_propagates_inner_flush_errors(GzipWriter::with_capacity)
}

#[test]
fn gzip_writer_propagates_inner_write_errors() -> io::Result<()> {
    test_writer_propagates_inner_write_errors(GzipWriter::with_capacity)
}

// ===========================================================
// Specific tests.
// ===========================================================

#[test]
fn gzip_reader_from_path() {
    let warc_file = get_fixture_path("warcfile.warc.gz");
    let mut reader = GzipReader::from_path(warc_file).unwrap();
    let mut buf = vec![0; 5];
    reader.read_exact(&mut buf).unwrap();
    assert_eq!(buf, b"WARC/");
}

#[test]
fn gzip_reader_returns_error_for_invalid_gzip_data() {
    let mut reader = GzipReader::new(Cursor::new(vec![0, 1, 2, 3, 4, 5, 6, 7]));
    let err = reader.fill_buf().unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    // The wrapped message proves we hit the explicit decompression error branch.
    assert!(err.to_string().contains("Gzip decompression error"));
}

#[test]
fn gzip_reader_update_buf_size_grows_and_shrinks_buffer() {
    let mut reader = GzipReader::with_capacity(Cursor::new(Vec::new()), 128);
    let initial_len = reader.buf.len();

    // A very high output-to-input ratio should force the decompressed buffer to grow.
    reader._update_buf_size(128, 1, 128);
    let grown_len = reader.buf.len();
    assert!(grown_len > initial_len);

    // Repeated low-ratio updates should decay the moving average enough to shrink again.
    for _ in 0..128 {
        reader._update_buf_size(128, 128, 128);
        if reader.buf.len() < grown_len {
            break;
        }
    }
    assert!(reader.buf.len() < grown_len);
    // The buffer must never shrink below the minimum size derived from the input buffer.
    assert!(reader.buf.len() >= 256);
}

#[test]
fn gzip_writer_returns_error_when_deflate_is_left_in_finish_state() {
    let mut writer = GzipWriter::with_capacity(Vec::new(), 1);
    let mut out = [0u8; 64];

    // Finish the raw deflater directly so GzipWriter does not get a chance to reset it.
    let status = writer.deflate.compress(b"seed", &mut out, DeflateFlush::Finish);
    assert!(matches!(status, Ok(zlib_rs::Status::StreamEnd)));

    let err = writer.write_with_flush_opt(b"more", DeflateFlush::NoFlush).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    // The wrapped message proves we hit the explicit compression error branch.
    assert!(err.to_string().contains("Gzip compression error"));
}

#[test]
fn gzip_writer_with_capacity_flush_and_write_with_flush_opt() -> io::Result<()> {
    let plain = sample_data();
    let inner = SharedVecWriter::new();
    let shared_data = inner.data();
    let mut writer = GzipWriter::with_capacity(inner.clone(), 8);

    assert_eq!(writer.write_with_flush_opt(&plain[..64], DeflateFlush::SyncFlush)?, 64);
    writer.flush()?;

    // SyncFlush plus flush() should already emit a readable partial member.
    assert!(!shared_data.borrow().is_empty());

    writer.write_all(&plain[64..])?;
    writer.finish()?;
    writer.flush()?;

    assert_eq!(decompress_member(&shared_data.borrow(), plain.len())?, plain);

    Ok(())
}

#[test]
fn gzip_writer_with_capacity_comp_level_finish_and_set_level_create_members() -> io::Result<()> {
    let first_plain = b"level-one member\n".repeat(40);
    let second_plain = b"level-nine member\n".repeat(40);

    let options = GzipWriterOptions {
        capacity: 1,
        compression_level: 9,
        ..Default::default()
    };
    let mut writer = GzipWriter::with_options(Vec::new(), options);
    writer.write_all(&first_plain)?;

    // The level change only applies to the next member after finish() resets the compressor.
    writer.set_level(9);
    writer.write_all(&second_plain)?;

    let compressed = writer.into_inner()?;
    let mut expected = first_plain.clone();
    expected.extend_from_slice(&second_plain);
    // The one-shot zlib-rs helpers stop after one gzip member.
    let mut reader = GzipReader::new(Cursor::new(compressed));
    let mut decompressed = Vec::new();
    reader.read_to_end(&mut decompressed)?;
    assert_eq!(decompressed, expected);

    Ok(())
}
