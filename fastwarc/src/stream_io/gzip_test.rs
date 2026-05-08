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

use super::super::mod_test::{ErrorWriter, SharedVecWriter};
use super::*;
use crate::stream_io::{CompressingStream, DecompressingStream};
use std::io::{self, BufRead, Cursor, Read, Seek, SeekFrom, Write};
use zlib_rs::{
    DeflateConfig, DeflateFlush, InflateConfig, ReturnCode, compress_bound, compress_slice, decompress_slice,
};

fn sample_data() -> Vec<u8> {
    b"The quick brown fox jumps over the lazy dog.\n".repeat(128)
}

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
fn gzip_reader_new_read_seek_and_stream_position() -> io::Result<()> {
    let plain = sample_data();
    let compressed = compress_member(&plain)?;
    let mut reader = GzipReader::new(Cursor::new(compressed));

    let mut prefix = [0; 17];
    assert_eq!(reader.read(&mut prefix)?, prefix.len());
    assert_eq!(prefix, plain[..prefix.len()]);
    assert_eq!(reader.stream_position()?, prefix.len() as u64);

    assert_eq!(reader.seek(SeekFrom::Current(11))?, 28);
    assert_eq!(reader.stream_position()?, 28);

    let mut next = [0; 9];
    reader.read_exact(&mut next)?;
    assert_eq!(next, plain[28..37]);

    assert_eq!(reader.seek(SeekFrom::Start(50))?, 50);
    let mut tail = [0; 12];
    reader.read_exact(&mut tail)?;
    assert_eq!(tail, plain[50..62]);

    // Backward seeking is intentionally unsupported because decompression only advances.
    let err = reader.seek(SeekFrom::Current(-1)).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);

    // Seeking from the end would require scanning the full compressed stream first.
    let err = reader.seek(SeekFrom::End(0)).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::Unsupported);

    Ok(())
}

#[test]
fn gzip_reader_with_capacity_fill_buf_consume_and_into_inner() -> io::Result<()> {
    let plain = sample_data();
    let compressed = compress_member(&plain)?;
    let mut reader = GzipReader::with_capacity(128, Cursor::new(compressed.clone()));

    let buf = reader.fill_buf()?.to_vec();
    assert!(!buf.is_empty());
    assert_eq!(buf, plain[..buf.len()]);

    reader.consume(3);
    assert_eq!(reader.stream_position()?, 3);

    let buf = reader.fill_buf()?.to_vec();
    assert_eq!(buf, plain[3..3 + buf.len()]);

    let fresh_reader = GzipReader::with_capacity(128, Cursor::new(compressed.clone()));
    let mut inner = fresh_reader.into_inner();
    assert_eq!(inner.stream_position()?, 0);

    let mut compressed_roundtrip = Vec::new();
    inner.read_to_end(&mut compressed_roundtrip)?;
    assert_eq!(compressed_roundtrip, compressed);

    Ok(())
}

#[test]
fn gzip_reader_with_tiny_capacity_handles_many_refills() -> io::Result<()> {
    let plain = sample_data();
    let compressed = compress_member(&plain)?;
    let mut reader = GzipReader::with_capacity(8, Cursor::new(compressed));
    let mut out = Vec::with_capacity(plain.len());

    loop {
        let buf = reader.fill_buf()?;
        if buf.is_empty() {
            break;
        }

        // Consume tiny chunks to force repeated refills and cross-buffer bookkeeping.
        let chunk_len = buf.len().min(5);
        out.extend_from_slice(&buf[..chunk_len]);
        reader.consume(chunk_len);
    }

    assert_eq!(out, plain);
    assert_eq!(reader.stream_position()?, plain.len() as u64);
    assert_eq!(reader.read(&mut [0; 1])?, 0);

    Ok(())
}

#[test]
fn gzip_reader_reads_to_eof_after_external_compression() -> io::Result<()> {
    let plain = sample_data();
    let compressed = compress_member(&plain)?;
    let mut reader = GzipReader::new(Cursor::new(compressed));

    let mut out = Vec::new();
    reader.read_to_end(&mut out)?;
    assert_eq!(out, plain);
    assert_eq!(reader.read(&mut [0; 1])?, 0);
    assert_eq!(reader.fill_buf()?, b"");

    Ok(())
}

#[test]
fn gzip_reader_inner_seek_inner_stream_position_and_member_tracking() -> io::Result<()> {
    let first_plain = b"first member data\n".repeat(32);
    let second_plain = b"second member payload\n".repeat(24);
    let first_member = compress_member(&first_plain)?;
    let second_member = compress_member(&second_plain)?;

    let mut combined = first_member.clone();
    combined.extend_from_slice(&second_member);

    let mut reader = GzipReader::new(Cursor::new(combined));
    let mut first_out = vec![0; first_plain.len()];
    reader.read_exact(&mut first_out)?;
    assert_eq!(first_out, first_plain);
    assert_eq!(reader.member_start_position()?, 0);

    let mut first_byte_second_member = [0; 1];
    reader.read_exact(&mut first_byte_second_member)?;
    assert_eq!(first_byte_second_member[0], second_plain[0]);
    // The member offset should jump once the second gzip header becomes active.
    assert_eq!(reader.member_start_position()?, first_member.len() as u64);
    assert!(reader.inner_stream_position()? >= first_member.len() as u64);

    assert_eq!(reader.inner_seek(SeekFrom::Start(first_member.len() as u64))?, first_member.len() as u64);
    assert_eq!(reader.member_start_position()?, first_member.len() as u64);

    let mut second_out = Vec::new();
    reader.read_to_end(&mut second_out)?;
    assert_eq!(second_out, second_plain);

    Ok(())
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
    let mut reader = GzipReader::with_capacity(128, Cursor::new(Vec::new()));
    let initial_len = reader.buf.len();

    // A very high output-to-input ratio should force the decompressed buffer to grow.
    reader._update_buf_size(128, 1, 64);
    let grown_len = reader.buf.len();
    assert!(grown_len > initial_len);

    // Repeated low-ratio updates should decay the moving average enough to shrink again.
    for _ in 0..16 {
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
fn gzip_writer_new_write_and_into_inner_roundtrip() -> io::Result<()> {
    let plain = sample_data();
    let mut writer = GzipWriter::new(Vec::new());

    assert_eq!(writer.write(&plain[..23])?, 23);
    writer.write_all(&plain[23..])?;

    let compressed = writer.into_inner()?;
    assert_eq!(decompress_member(&compressed, plain.len())?, plain);

    Ok(())
}

#[test]
fn gzip_writer_returns_error_when_deflate_is_left_in_finish_state() {
    let mut writer = GzipWriter::with_capacity(1, Vec::new());
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
    let mut writer = GzipWriter::with_capacity(8, inner.clone());

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

    let mut writer = GzipWriter::with_capacity_comp_level(9, Vec::new(), 1);
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

#[test]
fn gzip_writer_drop_finishes_and_flushes_stream() -> io::Result<()> {
    let plain = sample_data();
    let inner = SharedVecWriter::new();
    let shared_data = inner.data();

    {
        let mut writer = GzipWriter::with_capacity(11, inner.clone());
        writer.write_all(&plain)?;
    }

    // Drop should behave like finish() + flush() when the writer still owns an inner stream.
    assert!(!shared_data.borrow().is_empty());
    assert_eq!(decompress_member(&shared_data.borrow(), plain.len())?, plain);

    Ok(())
}

#[test]
fn gzip_writer_propagates_inner_write_errors() {
    let plain = sample_data();
    let mut writer = GzipWriter::with_capacity(
        8,
        ErrorWriter {
            fail_on_write: true,
            fail_on_flush: false,
        },
    );

    let err = writer.write_with_flush_opt(&plain, DeflateFlush::Finish).unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::Other);
    assert!(err.to_string().contains("injected write failure"));
}

#[test]
fn gzip_writer_propagates_inner_flush_errors() -> io::Result<()> {
    let plain = sample_data();
    let mut writer = GzipWriter::with_capacity(
        64,
        ErrorWriter {
            fail_on_write: false,
            fail_on_flush: true,
        },
    );

    writer.write_all(&plain)?;

    // flush() must forward the inner stream failure after draining buffered output.
    let err = writer.flush().unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::Other);
    assert!(err.to_string().contains("injected flush failure"));

    Ok(())
}
