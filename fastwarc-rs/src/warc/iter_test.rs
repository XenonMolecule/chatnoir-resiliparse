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
use crate::stream_io::traits::WarcWrite;
use crate::stream_io::zstd::ZstdWriterOptions;
use crate::stream_io::{brotli, chunked, gzip, lz4, zstd};
use crate::warc::iter::{
    ArchiveIterator, ArchiveIteratorOptions, ArchiveIteratorThreadSafe, FilteredArchiveIterator, SharedWarcRecord,
    filter,
};
use crate::warc::mod_test::{
    get_fixture_path, http_response_warc_data_encoded, warc_record_data, warc_record_data_with_headers,
};
use crate::warc::record::{AutoDecode, WarcRecord, WarcRecordType};
use data_encoding::BASE32;
use digest::Digest;
use sha1::Sha1;
use std::fs::File;
use std::io;
use std::io::{Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex};

#[test]
fn archive_iterator_option_setters() -> io::Result<()> {
    let reader = Box::new(io::Cursor::new(Vec::new()));
    let it = ArchiveIterator::new(reader.clone());
    assert_eq!(it.options, ArchiveIteratorOptions::default());

    let new_opts = ArchiveIteratorOptions {
        stream_detect: false,
        parse_http: false,
        decode_http_payload: AutoDecode::All,
        verify_digests: true,
        quirks_mode: true,
    };
    assert_ne!(ArchiveIteratorOptions::default().stream_detect, new_opts.stream_detect);
    assert_ne!(ArchiveIteratorOptions::default().parse_http, new_opts.parse_http);
    assert_ne!(ArchiveIteratorOptions::default().decode_http_payload, new_opts.decode_http_payload);
    assert_ne!(ArchiveIteratorOptions::default().verify_digests, new_opts.verify_digests);
    assert_ne!(ArchiveIteratorOptions::default().quirks_mode, new_opts.quirks_mode);

    // Test constructor with options
    let it = ArchiveIterator::with_options(reader.clone(), new_opts);
    assert_eq!(it.options, new_opts);

    // Test setters
    let mut it = ArchiveIterator::new(reader.clone());
    it.set_stream_detect(new_opts.stream_detect);
    it.set_parse_http(new_opts.parse_http);
    it.set_decode_http_payload(new_opts.decode_http_payload);
    it.set_verify_digests(new_opts.verify_digests);
    it.set_quirks_mode(new_opts.quirks_mode);
    assert_eq!(it.options, new_opts);

    let mut it = ArchiveIterator::new(reader.clone());
    it.set_options(new_opts);
    assert_eq!(it.options, new_opts);

    // Test consuming setters
    let it = ArchiveIterator::new(reader.clone())
        .with_stream_detect(new_opts.stream_detect)
        .with_parse_http(new_opts.parse_http)
        .with_decode_http_payload(new_opts.decode_http_payload)
        .with_verify_digests(new_opts.verify_digests)
        .with_quirks_mode(new_opts.quirks_mode);
    assert_eq!(it.options, new_opts);

    Ok(())
}

#[test]
fn archive_iterator() -> io::Result<()> {
    let record_data1 = warc_record_data("request", "<urn:uuid:record1>", None, b"ABC");
    let record_data2 = warc_record_data("response", "<urn:uuid:record2>", None, b"DEFGHI");
    let warc_data = [record_data1.as_slice(), record_data2.as_slice()].concat();

    let reader = Box::new(io::Cursor::new(warc_data));

    // Manual iteration
    let mut record1 = WarcRecord::from_reader(reader.clone())?;
    assert_eq!(record1.stream_pos(), 0);
    assert_eq!(record1.record_id().unwrap(), "<urn:uuid:record1>");
    let mut record2 = record1.next().unwrap()?;
    assert_eq!(record2.record_id().unwrap(), "<urn:uuid:record2>");
    assert_eq!(record2.stream_pos(), warc_record_data("request", "<urn:uuid:record1>", None, b"ABC").len() as u64);
    assert!(record2.next().is_none());

    // ArchiveIterator (without reading payload -> consumed automatically)
    let mut it = ArchiveIterator::new(reader.clone());
    let record1 = it.next().unwrap()?;
    assert_eq!(record1.borrow().record_id().unwrap(), "<urn:uuid:record1>");
    assert_eq!(record1.borrow().stream_pos(), 0);
    let record2 = it.next().unwrap()?;
    assert_eq!(record2.borrow().record_id().unwrap(), "<urn:uuid:record2>");
    assert_eq!(record2.borrow().stream_pos(), record_data1.len() as u64);
    assert!(it.next().is_none());

    // Explicit loop (with reading payload)
    let mut i = 0;
    let mut buf = Vec::with_capacity(9);
    for r in ArchiveIterator::new(reader.clone()) {
        let r = r?;
        if i == 0 {
            assert_eq!(r.borrow().record_id().unwrap(), "<urn:uuid:record1>");
            assert_eq!(r.borrow().stream_pos(), 0);
            r.borrow_mut().reader_mut().unwrap().read_to_end(&mut buf)?;
        } else {
            assert_eq!(r.borrow().record_id().unwrap(), "<urn:uuid:record2>");
            assert_eq!(r.borrow().stream_pos(), record_data1.len() as u64);
            r.borrow_mut().reader_mut().unwrap().read_to_end(&mut buf)?;
        }
        i += 1;
    }
    assert_eq!(i, 2);
    assert_eq!(buf, b"ABCDEFGHI");

    // Trait-derived iterator methods
    assert_eq!(ArchiveIterator::new(reader).count(), 2);

    Ok(())
}

#[test]
fn archive_iterator_into_inner() -> io::Result<()> {
    let record_data = warc_record_data("resource", "<urn:uuid:into-inner>", None, b"ABC");
    let mut reader = ArchiveIterator::new(Box::new(io::Cursor::new(record_data.clone())))
        .into_inner()
        .unwrap();

    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    assert_eq!(buf, record_data);

    Ok(())
}

#[test]
fn archive_iterator_stream_compression_autodetection() -> io::Result<()> {
    let plain_warc = get_fixture_path("warcfile.warc");
    let mut it = ArchiveIterator::new(Box::new(io::BufReader::new(File::open(plain_warc)?)));
    it.next().transpose()?;
    it.next().transpose()?;

    let gzip_warc = get_fixture_path("warcfile.warc.gz");
    let mut it = ArchiveIterator::new(Box::new(io::BufReader::new(File::open(gzip_warc)?)));
    it.next().transpose()?;
    it.next().transpose()?;

    let lz4_warc = get_fixture_path("warcfile.warc.lz4");
    let mut it = ArchiveIterator::new(Box::new(io::BufReader::new(File::open(lz4_warc)?)));
    it.next().transpose()?;
    it.next().transpose()?;

    // Gzip without autodetection
    let gzip_warc = get_fixture_path("warcfile.warc.gz");
    let mut it = ArchiveIterator::new(Box::new(io::BufReader::new(File::open(gzip_warc)?))).with_stream_detect(false);
    assert_eq!(it.next().transpose().unwrap_err().kind(), io::ErrorKind::InvalidData);

    Ok(())
}

#[test]
fn archive_iterator_from_path() -> io::Result<()> {
    let plain_warc = get_fixture_path("warcfile.warc");
    let mut it = ArchiveIterator::from_path(plain_warc)?;
    it.next().transpose()?;
    it.next().transpose()?;

    // Invalid path
    let plain_warc = get_fixture_path("warcfile.warc.doesnotexist");
    assert_eq!(ArchiveIterator::from_path(plain_warc).unwrap_err().kind(), io::ErrorKind::NotFound);

    // With autodetection
    let plain_warc = get_fixture_path("warcfile.warc.gz");
    let mut it = ArchiveIterator::from_path(plain_warc)?;
    it.next().transpose()?;
    it.next().transpose()?;

    let plain_warc = get_fixture_path("warcfile.warc.lz4");
    let mut it = ArchiveIterator::from_path(plain_warc)?;
    it.next().transpose()?;
    it.next().transpose()?;

    // Without autodetection
    let plain_warc = get_fixture_path("warcfile.warc.gz");
    let opts = ArchiveIteratorOptions {
        stream_detect: false,
        ..ArchiveIteratorOptions::default()
    };
    let mut it = ArchiveIterator::from_path_with_options(plain_warc, opts)?;
    assert_eq!(it.next().transpose().unwrap_err().kind(), io::ErrorKind::InvalidData);

    Ok(())
}

#[test]
fn archive_iterator_thread_safe() -> io::Result<()> {
    let record_data1 = warc_record_data("resource", "<urn:uuid:threadsafe-1>", None, b"ABC");
    let record_data2 = warc_record_data("metadata", "<urn:uuid:threadsafe-2>", None, b"XYZ");
    let reader = Box::new(io::Cursor::new([record_data1.as_slice(), record_data2.as_slice()].concat()));

    let mut it = ArchiveIteratorThreadSafe::new(reader);
    let record1 = it.next().unwrap()?;
    let _: Arc<Mutex<WarcRecord>> = record1.clone();
    assert_eq!(record1.lock().unwrap().record_id().unwrap(), "<urn:uuid:threadsafe-1>");

    let record2 = it.next().unwrap()?;
    assert_eq!(record2.lock().unwrap().record_id().unwrap(), "<urn:uuid:threadsafe-2>");
    assert!(it.next().is_none());

    Ok(())
}

#[test]
fn archive_iterator_quirks_mode() -> io::Result<()> {
    let mut record_data = warc_record_data("resource", "<urn:uuid:a>", None, b"ABC");
    record_data.extend_from_slice(b"corrupted-data-without-linefeed");
    record_data.extend_from_slice(&warc_record_data("resource", "<urn:uuid:b>", None, b"ABC"));
    record_data.extend_from_slice(b"foo\n");
    record_data.extend_from_slice(&warc_record_data("resource", "<urn:uuid:c>", None, b"ABC"));
    record_data.extend_from_slice(b"bar\r\n\r\n");
    record_data.extend_from_slice(&warc_record_data("resource", "<urn:uuid:d>", None, b"ABC"));

    let mut reader = ArchiveIterator::new(Box::new(io::Cursor::new(record_data.clone())));
    assert_eq!(reader.next().unwrap()?.borrow().record_id().unwrap(), "<urn:uuid:a>");
    assert!(reader.next().unwrap().is_err());

    let mut reader = ArchiveIterator::new(Box::new(io::Cursor::new(record_data))).with_quirks_mode(true);
    assert_eq!(reader.next().unwrap()?.borrow().record_id().unwrap(), "<urn:uuid:a>");
    // b skipped due to corrupted WARC/1.1 header start
    assert_eq!(reader.next().unwrap()?.borrow().record_id().unwrap(), "<urn:uuid:c>");
    assert_eq!(reader.next().unwrap()?.borrow().record_id().unwrap(), "<urn:uuid:d>");
    assert!(reader.next().is_none());

    Ok(())
}

#[test]
fn archive_iterator_with_encoded_http_payloads() -> io::Result<()> {
    let payload_raw = [b"ABCDEF".repeat(2000), b"UVWXYZ".repeat(2000), b"ABCXYZ".repeat(2000)];
    let mut data = Vec::new();

    // Record 1
    let mut w = zstd::ZstdWriter::new(Vec::new());
    w.write_all(&payload_raw[0])?;
    let encoded = w.into_inner()?;
    let mut w = brotli::BrotliWriter::new(Vec::new());
    w.write_all(&encoded)?;
    let encoded = w.into_inner()?;
    let mut w = chunked::ChunkedWriter::new(Vec::new());
    w.write_all(&encoded)?;
    let encoded = w.into_inner()?;
    data.push(http_response_warc_data_encoded("<urn:uuid:abc>", &encoded, Some("zstd, br, chunked"), None));

    // Record 2
    let mut w = brotli::BrotliWriter::new(Vec::new());
    w.write_all(&payload_raw[1])?;
    let encoded = w.into_inner()?;
    data.push(http_response_warc_data_encoded("<urn:uuid:abc>", &encoded, None, Some("br")));

    // Record 3
    data.push(http_response_warc_data_encoded("<urn:uuid:abc>", &payload_raw[2], None, None));

    let mut count = 0;
    for (i, rec) in ArchiveIterator::new(Box::new(io::Cursor::new(data.concat())))
        .with_decode_http_payload(AutoDecode::All)
        .enumerate()
    {
        rec?.with_mut(|r| -> io::Result<()> {
            let mut buf = Vec::with_capacity(payload_raw[i].len());
            r.reader_mut().unwrap().read_to_end(&mut buf)?;
            assert_eq!(buf, payload_raw[i]);
            Ok(())
        })?;
        count += 1;
    }
    assert_eq!(count, data.len());
    Ok(())
}

#[test]
fn filtered_archive_iterator() -> io::Result<()> {
    let mut filtered: FilteredArchiveIterator<_> = ArchiveIterator::with_filter(
        Box::new(io::Cursor::new(filter_test_warc_data())),
        filter::has_record_type(WarcRecordType::Resource),
    );
    filtered.set_parse_http(false);
    let _: &ArchiveIterator = &filtered;

    let record = filtered.next().unwrap()?;
    assert_eq!(record.borrow().record_id().as_deref(), Some("<urn:uuid:filter-block>"));
    assert_eq!(record.borrow().record_type(), WarcRecordType::Resource);
    assert!(filtered.next().is_none());

    Ok(())
}

/// Test fixture: WARC records for testing filter predicates.
fn filter_test_warc_data() -> Vec<u8> {
    let warc10 =
        String::from_utf8_lossy(warc_record_data("warcinfo", "<urn:uuid:filter-warc10>", None, b"INFO").as_slice())
            .replacen("WARC/1.1", "WARC/1.0", 1)
            .into_bytes();

    let http_payload = "Hello";
    let http_data = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        http_payload.len(),
        http_payload
    )
    .into_bytes();
    let payload_digest = BASE32.encode(&Sha1::digest(http_payload.as_bytes()));
    let http = warc_record_data_with_headers(
        "response",
        "<urn:uuid:filter-http>",
        Some("application/http; msgtype=response"),
        &format!("WARC-Payload-Digest: sha1:{payload_digest}\r\n"),
        http_data.as_slice(),
    );

    let block_payload = b"BLOCK";
    let block_digest = BASE32.encode(&Sha1::digest(block_payload));
    let block = warc_record_data_with_headers(
        "resource",
        "<urn:uuid:filter-block>",
        None,
        &format!(
            "WARC-Block-Digest: sha1:{block_digest}\r\n\
             WARC-Concurrent-To: <urn:uuid:filter-http>\r\n"
        ),
        block_payload,
    );

    let metadata = warc_record_data("metadata", "<urn:uuid:filter-metadata>", None, b"LONGER");

    [
        warc10.as_slice(),
        http.as_slice(),
        block.as_slice(),
        metadata.as_slice(),
    ]
    .concat()
}

/// Helper for running iterator checks on both [`ArchiveIterator`] and [`ArchiveIteratorThreadSafe`].
fn run_archive_iterator_variants<R, M, F>(mut make_reader: M, mut check: F) -> io::Result<()>
where
    R: BufReadSeek + Send + 'static,
    M: FnMut() -> io::Result<R>,
    F: FnMut(&mut WarcRecord) -> io::Result<()>,
{
    let opts = ArchiveIteratorOptions {
        parse_http: false,
        ..Default::default()
    };
    for r in ArchiveIterator::with_options(Box::new(make_reader()?), opts) {
        r?.with_mut(|rm| check(rm))?;
    }

    for r in ArchiveIteratorThreadSafe::with_options(Box::new(make_reader()?), opts) {
        r?.with_mut(|rm| check(rm))?;
    }

    Ok(())
}

/// Test fixture: IDs of records matching a filter predicate
fn filtered_record_ids<F>(filter: F) -> io::Result<Vec<String>>
where
    F: Fn(&mut WarcRecord) -> bool,
{
    let mut ids = Vec::new();
    run_archive_iterator_variants(
        || Ok(io::Cursor::new(filter_test_warc_data())),
        |record| {
            if filter(record) {
                ids.push(record.record_id().unwrap().to_string());
            }
            Ok(())
        },
    )?;

    Ok(ids)
}

#[test]
fn archive_iterator_filter_predicates() -> io::Result<()> {
    macro_rules! assert_filtered_ids {
        ($name:literal, $predicate:expr, $expected:expr) => {
            assert_eq!(filtered_record_ids($predicate)?, $expected.repeat(2), "{}", $name);
        };
    }

    assert_filtered_ids!("is_warc_10", filter::is_warc_10, ["<urn:uuid:filter-warc10>"]);
    assert_filtered_ids!(
        "is_warc_11",
        filter::is_warc_11,
        [
            "<urn:uuid:filter-http>",
            "<urn:uuid:filter-block>",
            "<urn:uuid:filter-metadata>"
        ]
    );
    assert_filtered_ids!("has_block_digest", filter::has_block_digest, ["<urn:uuid:filter-block>"]);
    assert_filtered_ids!("has_valid_block_digest", filter::has_valid_block_digest, ["<urn:uuid:filter-block>"]);
    assert_filtered_ids!("has_payload_digest", filter::has_payload_digest, ["<urn:uuid:filter-http>"]);
    assert_filtered_ids!(
        "has_valid_payload_digest",
        |record: &mut WarcRecord| {
            record.parse_http().unwrap();
            filter::has_valid_payload_digest(record)
        },
        ["<urn:uuid:filter-http>"]
    );
    assert_filtered_ids!("is_http", filter::is_http, ["<urn:uuid:filter-http>"]);
    assert_filtered_ids!("is_concurrent", filter::is_concurrent, ["<urn:uuid:filter-block>"]);
    assert_filtered_ids!(
        "has_record_type",
        filter::has_record_type(WarcRecordType::Metadata),
        ["<urn:uuid:filter-metadata>"]
    );
    assert_filtered_ids!("has_content_length_lte", filter::has_content_length_lte(4), ["<urn:uuid:filter-warc10>"]);
    assert_filtered_ids!(
        "has_content_length_gte",
        filter::has_content_length_gte(6),
        ["<urn:uuid:filter-http>", "<urn:uuid:filter-metadata>"]
    );
    // Custom closure filter.
    assert_filtered_ids!(
        "custom_closure",
        |record: &mut WarcRecord| record.record_id().is_some_and(|id| id.contains("metadata")),
        ["<urn:uuid:filter-metadata>"]
    );

    Ok(())
}

#[test]
fn archive_iterator_read_clipped_warc_file() -> io::Result<()> {
    let clipped = get_fixture_path("clipped.warc.gz");

    for parse_http in [true, false] {
        let mut rec_count = 0;
        run_archive_iterator_variants(
            || Ok(gzip::GzipReader::new(File::open(clipped.clone())?)),
            |r| -> io::Result<()> {
                let mut content = Vec::with_capacity(r.content_length() as usize);
                r.reader_mut().unwrap().read_to_end(&mut content)?;

                if parse_http {
                    r.parse_http()?;
                    assert!(r.http_headers().is_some());
                } else {
                    assert!(content.starts_with(b"HTTP/"));
                }
                // Content-Length is larger than the actual clipped payload.
                assert!(r.content_length() as usize > content.len());

                // Should fail, since we already read the contents.
                assert!(!r.verify_block_digest(true).unwrap());
                rec_count += 1;

                Ok(())
            },
        )?;

        // Contains exactly one record (one per iteration)
        assert_eq!(rec_count, 2);
    }

    Ok(())
}

/// Helper for testing whether iterating over a WARC reports the correct record offsets,
/// and whether we can restart a new iterator from those record / compression member boundaries.
fn iterate_archive_members_with_offsets<R, M>(mut make_reader: M) -> io::Result<()>
where
    R: BufReadSeek + Send + 'static,
    M: FnMut() -> io::Result<R>,
{
    let mut iterator_variant_runs = Vec::new();
    let mut offsets = Vec::new();
    let mut record_ids = Vec::new();

    // First, iterate over all records and collect their offsets and record IDs.
    run_archive_iterator_variants(&mut make_reader, |record| {
        let stream_pos = record.stream_pos();

        if !offsets.is_empty() && stream_pos == 0 {
            // First iteration of second variant
            iterator_variant_runs.push((std::mem::take(&mut offsets), std::mem::take(&mut record_ids)));
        } else if let Some(&previous) = offsets.last() {
            assert!(stream_pos > previous);
        } else {
            // First overall
            assert_eq!(stream_pos, 0);
        }

        offsets.push(stream_pos);
        record_ids.push(record.record_id().unwrap().to_string());

        Ok(())
    })?;
    iterator_variant_runs.push((offsets, record_ids));

    // Recorded two variant runs (non-thread-safe and thread-safe).
    assert_eq!(iterator_variant_runs.len(), 2);
    // Both must be identical.
    assert_eq!(iterator_variant_runs[0], iterator_variant_runs[1]);

    // Discard the second variant run.
    let (offsets, record_ids) = &iterator_variant_runs[0];
    let num_records = offsets.len();
    assert!(num_records > 0);

    // Test whether we can restart the iterator from any of the previously recorded offsets.
    for (i, &offset) in offsets.iter().enumerate() {
        let num_expected_records = num_records - i;
        let expected_id = &record_ids[i];
        let mut iterator_variant_counts = Vec::new();
        let mut count = 0usize;
        let mut first_record = true;

        let make_reader = || {
            let mut reader = make_reader()?;
            reader.seek(SeekFrom::Start(offset))?;
            Ok(reader)
        };
        run_archive_iterator_variants(make_reader, |record| {
            // First run of second variant
            if count > 0 && record.stream_pos() == offset {
                iterator_variant_counts.push(count);
                count = 0;
                first_record = true;
            }

            // Check whether first record matches the previously recorded ID and verify block digest.
            if first_record {
                assert_eq!(record.record_id().as_deref(), Some(expected_id.as_str()));
                if record.record_type() == WarcRecordType::Response {
                    assert!(record.verify_block_digest(false).unwrap());
                }
                first_record = false;
            }

            count += 1;
            Ok(())
        })?;

        iterator_variant_counts.push(count);
        assert_eq!(iterator_variant_counts, vec![num_expected_records, num_expected_records]);
    }

    Ok(())
}

#[test]
fn archive_iterator_record_offsets() -> io::Result<()> {
    let warc = get_fixture_path("warcfile.warc");
    iterate_archive_members_with_offsets(|| Ok(io::BufReader::new(File::open(warc.clone())?)))?;

    let warc_gz = get_fixture_path("warcfile.warc.gz");
    iterate_archive_members_with_offsets(|| Ok(gzip::GzipReader::new(File::open(warc_gz.clone())?)))?;

    let warc_lz4 = get_fixture_path("warcfile.warc.lz4");
    iterate_archive_members_with_offsets(|| Ok(lz4::Lz4Reader::new(File::open(warc_lz4.clone())?)))?;

    Ok(())
}

#[test]
fn archive_iterator_zstd_dict() -> io::Result<()> {
    let dict = zstd::train_dictionary_from_samples(
        &[&b"The quick brown fox jumps over the lazy dog."].repeat(100),
        32 * 1024,
    )?;

    // Frame 1 (+ Dictionary)
    let mut writer = zstd::ZstdWriter::with_dictionary(io::Cursor::new(Vec::new()), dict.clone(), None);
    writer.write_all(&warc_record_data("response", "<urn:uuid:rec0>", None, b"ABC"))?;
    writer.finish()?;

    // Frame 2: skippable frame with arbitrary content (allowed by spec, must be skipped)
    // Zstd frames with magic numbers 0x184D2A50..0x184D2A5F are skippable.
    let mut writer = writer.into_inner()?;
    writer.write_all(&0x184D2A5Fu32.to_le_bytes())?;
    writer.write_all(&20u32.to_le_bytes())?;
    writer.write_all(&b"A".repeat(20))?;

    // Frame 3
    let opts = Some(ZstdWriterOptions {
        write_dictionary_frame: false,
        ..ZstdWriterOptions::default()
    });
    let mut writer = zstd::ZstdWriter::with_dictionary(writer, dict.clone(), opts);
    writer.write_all(&warc_record_data("response", "<urn:uuid:rec1>", None, b"ABC"))?;
    writer.finish()?;

    // Frame 4: empty (allowed by spec, must be skipped)
    let _ = writer.write(b"")?;
    writer.finish()?;

    // Frame 5
    writer.write_all(&warc_record_data("response", "<urn:uuid:rec2>", None, b"ABC"))?;
    writer.finish()?;

    let mut encoded = writer.into_inner()?;
    encoded.rewind()?;

    // Confirm that decompression without dictionary fails.
    assert_eq!(::zstd::decode_all(encoded.clone()).unwrap_err().to_string(), "Dictionary mismatch");

    // ZstdReader should load dictionary frame automatically and keep it for all iterations.
    let reader = Box::new(zstd::ZstdReader::new(encoded));
    let mut count = 0;
    for (i, rec) in ArchiveIterator::new(reader).enumerate() {
        let rec = rec?;
        assert_eq!(rec.borrow().record_id().unwrap().to_string(), format!("<urn:uuid:rec{}>", i));
        count += 1;
    }
    assert_eq!(count, 3);

    Ok(())
}
