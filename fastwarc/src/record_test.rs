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
use crate::stream_io::zstd::ZstdWriterOptions;
use crate::stream_io::{BufReadSeek, CompressingWriter};
use crate::stream_io::{LimitedBufReader, gzip, lz4};
use data_encoding::{BASE32, HEXLOWER};
use md5::Md5;
use pretty_assertions::assert_eq;
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};
use std::borrow::Cow;
use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// Helper for getting path to external test fixtures.
fn get_fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    path
}

/// Test fixture: WARC record as String.
fn warc_record_data(record_type: &str, record_id: &str, content_type: Option<&str>, payload: &[u8]) -> Vec<u8> {
    warc_record_data_with_headers(record_type, record_id, content_type, "", payload)
}

/// Test fixture: WARC record with custom headers as String.
fn warc_record_data_with_headers(
    record_type: &str,
    record_id: &str,
    content_type: Option<&str>,
    extra_headers: &str,
    payload: &[u8],
) -> Vec<u8> {
    let mut bytes = format!(
        "WARC/1.1\r\n\
         WARC-Type: {}\r\n\
         WARC-Record-ID: {}\r\n\
         {}\
         {}\
         Content-Length: {}\r\n\
         \r\n",
        record_type,
        record_id,
        content_type
            .map(|value| format!("Content-Type: {value}\r\n"))
            .unwrap_or_default(),
        extra_headers,
        payload.len()
    )
    .into_bytes();
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(b"\r\n\r\n");
    bytes
}

/// Test fixture: WARC record of an HTTP response record as String.
fn http_response_warc_data(record_id: &str, payload: &str) -> Vec<u8> {
    let http_data = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Server: nginx\r\n\
         \r\n\
         {}",
        payload.len(),
        payload
    );
    warc_record_data("response", record_id, Some("application/http; msgtype=response"), http_data.as_bytes())
}

/// Test fixture: WARC record of an HTTP response record as String.
fn http_response_warc_data_encoded(
    record_id: &str,
    payload: &[u8],
    transfer_encoding: Option<&str>,
    content_encoding: Option<&str>,
) -> Vec<u8> {
    let mut transfer_enc_header = "".to_string();
    let mut content_enc_header = "".to_string();
    let mut content_len_header = format!("Content-Length: {}\r\n", payload.len());

    if let Some(enc) = transfer_encoding {
        transfer_enc_header = format!("Transfer-Encoding: {}\r\n", enc);
        if enc == "chunked" {
            content_len_header = "".to_string();
        }
    };
    if let Some(enc) = content_encoding {
        content_enc_header = format!("Content-Encoding: {}\r\n", enc);
    };
    let http_data = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Server: nginx\r\n\
         {}{}{}\
         \r\n",
        transfer_enc_header, content_enc_header, content_len_header
    );
    let payload = [http_data.as_bytes(), payload].concat();
    warc_record_data("response", record_id, Some("application/http; msgtype=response"), payload.as_slice())
}

#[test]
fn limited_buf_read_seek_limit_seek() -> io::Result<()> {
    let mut limited = LimitedBufReader::new(Box::new(io::Cursor::new(b"abcdef".to_vec())), Some(4));

    assert_eq!(limited.fill_buf()?, b"abcd");
    limited.consume(2);
    assert_eq!(limited.stream_position()?, 2);
    assert_eq!(limited.real_stream_position()?, 2);

    let mut buf = [0u8; 4];
    assert_eq!(limited.read(&mut buf)?, 2);
    assert_eq!(&buf[..2], b"cd");
    assert_eq!(limited.read(&mut buf)?, 0);

    limited.set_limit(3);
    assert_eq!(limited.stream_position()?, 0);

    Ok(())
}

#[test]
fn record_type_try_from_helpers() {
    assert_eq!(WarcRecordType::try_from(4u16), Ok(WarcRecordType::Response));
    assert_eq!(WarcRecordType::try_from("REQUEST"), Ok(WarcRecordType::Request));
    assert_eq!(WarcRecordType::try_from(b"metadata".as_slice()), Ok(WarcRecordType::Metadata));
    assert!(WarcRecordType::try_from(3u16).is_err());
}

#[test]
fn record_type_from_helpers() {
    assert_eq!(<&'static str>::from(WarcRecordType::Continuation), "continuation");
    assert_eq!(<&'static str>::from(WarcRecordType::NoType), "unknown");
}

#[test]
fn record_type_conversions() {
    let named_types = [
        (WarcRecordType::WarcInfo, 2u16, "warcinfo"),
        (WarcRecordType::Response, 4u16, "response"),
        (WarcRecordType::Resource, 8u16, "resource"),
        (WarcRecordType::Request, 16u16, "request"),
        (WarcRecordType::Metadata, 32u16, "metadata"),
        (WarcRecordType::Revisit, 64u16, "revisit"),
        (WarcRecordType::Conversion, 128u16, "conversion"),
        (WarcRecordType::Continuation, 256u16, "continuation"),
        (WarcRecordType::Unknown, 512u16, "unknown"),
    ];

    for (record_type, value, name) in named_types {
        assert_eq!(WarcRecordType::try_from(value), Ok(record_type));
        assert_eq!(WarcRecordType::try_from(name), Ok(record_type));
        assert_eq!(WarcRecordType::try_from(name.as_bytes()), Ok(record_type));
        assert_eq!(<&'static str>::from(record_type), name);
    }

    assert_eq!(WarcRecordType::try_from(WarcRecordType::AnyType as u16), Ok(WarcRecordType::AnyType));
    assert_eq!(WarcRecordType::try_from(WarcRecordType::NoType as u16), Ok(WarcRecordType::NoType));
    assert_eq!(<&'static str>::from(WarcRecordType::AnyType), "unknown");
    assert_eq!(<&'static str>::from(WarcRecordType::NoType), "unknown");
    assert!(WarcRecordType::try_from(3u16).is_err());
    assert!(WarcRecordType::try_from("not-a-record-type").is_err());
    assert!(WarcRecordType::try_from(b"not-a-record-type".as_slice()).is_err());
}

#[test]
fn record_type_bitmask_helpers() {
    let http_exchange = WarcRecordType::Request as u16 | WarcRecordType::Response as u16;
    assert!(WarcRecordType::Request.matches_bitmask(http_exchange));
    assert!(WarcRecordType::Response.matches_bitmask(http_exchange));
    assert!(!WarcRecordType::Resource.matches_bitmask(http_exchange));

    let metadata_or_revisit = WarcRecordType::Metadata as u16 | WarcRecordType::Revisit as u16;
    assert!(WarcRecordType::Metadata.matches_bitmask(metadata_or_revisit));
    assert!(WarcRecordType::Revisit.matches_bitmask(metadata_or_revisit));
    assert!(!WarcRecordType::Conversion.matches_bitmask(metadata_or_revisit));

    assert!(WarcRecordType::Response.matches_bitmask(WarcRecordType::AnyType as u16));
}

#[test]
fn record_type_and_header_map_helpers() {
    let mut headers = HeaderMap::new(HeaderEncoding::Unicode);
    headers.set_status_line("HTTP/1.1 204");
    headers.append("Set-Cookie", "a=1");
    headers.append("set-cookie", "b=2");

    assert_eq!(headers.encoding(), HeaderEncoding::Unicode);
    assert_eq!(headers.status_code(), Some(204));
    assert_eq!(headers.reason_phrase(), None);
    assert!(headers.contains_key_bytes(b"SET-cookie"));

    let merged = headers.to_map();
    assert_eq!(merged.get(&ci("set-cookie")), Some(&"a=1,b=2".to_string()));

    headers.remove_bytes(b"set-cookie");
    assert!(!headers.contains_key("Set-Cookie"));
    headers.clear();
    assert!(headers.is_empty());
    assert_eq!(headers.status_line(), None);
}

#[test]
fn new_empty_header_map() {
    let headers = HeaderMap::new(HeaderEncoding::Unicode);
    assert_eq!(headers.encoding(), HeaderEncoding::Unicode);
    assert_eq!(headers.len(), 0);
    assert!(headers.is_empty());
    assert_eq!(headers.status_code(), None);
}

#[test]
fn set_get_remove_header() {
    let mut headers = HeaderMap::new(HeaderEncoding::Latin1);
    headers.set("Content-Type", "text/plain");
    assert_eq!(headers.encoding(), HeaderEncoding::Latin1);
    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/plain"));
    assert_eq!(headers.len(), 1);

    // Override existing
    headers.set("Content-Type", "text/html");
    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/html"));
    assert_eq!(headers.len(), 1);

    // Add new
    headers.set("Content-Length", "10");
    assert_eq!(headers.get("Content-Length").as_deref(), Some("10"));
    assert_eq!(headers.len(), 2);

    // Set and get as bytes
    headers.set_bytes(b"Content-Type", b"text/plain");
    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/plain"));
    assert_eq!(headers.get_bytes(b"Content-Type").as_deref(), Some(b"text/plain".as_slice()));
    assert_eq!(headers.len(), 2);

    // Header does not exist
    assert_eq!(headers.get("Missing-Header"), None);

    // Remove (case-insensitive)
    headers.remove("CONTENT-TYPE");
    assert_eq!(headers.get("Content-Type"), None);
    assert_eq!(headers.len(), 1);
}

#[test]
fn duplicate_header() {
    let mut headers = HeaderMap::new(HeaderEncoding::Latin1);
    assert_eq!(headers.len(), 0);
    assert_eq!(headers.get_multiple("Content-Type"), Vec::<&str>::new());

    // Set
    headers.set("Content-Type", "text/plain");
    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/plain"));
    assert_eq!(headers.len(), 1);
    assert_eq!(headers.get_multiple("Content-Type"), vec!["text/plain"]);

    // Set again
    headers.set("Content-Type", "text/html");
    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/html"));
    assert_eq!(headers.len(), 1);
    assert_eq!(headers.get_multiple("Content-Type"), vec!["text/html"]);

    // Append duplicate
    headers.append("Content-Type", "text/plain");
    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/html"));
    assert_eq!(headers.len(), 2);
    assert_eq!(headers.get_multiple("Content-Type"), vec!["text/html", "text/plain"]);
    assert_eq!(headers.get_bytes_multiple(b"Content-Type"), vec![b"text/html".as_slice(), b"text/plain".as_slice()]);

    // Remove (case-insensitive)
    headers.remove("CONTENT-TYPE");
    assert_eq!(headers.get("Content-Type"), None);
    assert_eq!(headers.len(), 0);
}

#[test]
fn header_case_insensitive_key() {
    let mut headers = HeaderMap::new(HeaderEncoding::Unicode);
    headers.set("Content-Type", "text/html");
    assert!(headers.keys().any(|k| k == "Content-Type"));
    assert_eq!(headers.get("CONTENT-TYPE").as_deref(), Some("text/html"));
    assert_eq!(headers.get("content-type").as_deref(), Some("text/html"));
    assert_eq!(headers.get("CoNtEnT-TyPe").as_deref(), Some("text/html"));

    headers.set("CONTENT-TYPE", "text/html");
    assert!(headers.keys().any(|k| k == "CONTENT-TYPE"));
}

#[test]
fn header_case_insensitive_key_from_helpers() {
    let borrowed_key = CaseInsensitiveKey::from("Content-Type");
    let owned_key = CaseInsensitiveKey::from("content-type".to_string());
    let cow_key = CaseInsensitiveKey::from(Cow::Borrowed("CONTENT-TYPE"));
    assert_eq!(borrowed_key, "content-type");
    assert_eq!("CONTENT-TYPE", owned_key);
    assert_eq!(String::from(cow_key), "CONTENT-TYPE");
}

#[test]
fn header_case_insensitive_key_eq_and_deref() {
    let borrowed_key = CaseInsensitiveKey::from("Content-Type");
    let owned_key = CaseInsensitiveKey::from("content-type".to_string());

    assert_eq!(borrowed_key, owned_key);
    assert_eq!(borrowed_key, *"CONTENT-TYPE");
    assert_eq!(borrowed_key, "content-type");
    assert_eq!(*"CONTENT-TYPE", borrowed_key);
    assert_eq!("content-type", borrowed_key);
    assert_ne!(borrowed_key, "Accept");

    assert_eq!(borrowed_key.as_ref(), "Content-Type");
}

#[test]
fn iterate_headers() {
    let tuples = [
        ("Content-Type", "text/html"),
        ("Content-Length", "1234"),
        ("Set-Cookie", "cookie1=value1"),
        ("Set-Cookie", "cookie2=value2"),
    ];

    let mut headers = HeaderMap::new(HeaderEncoding::Latin1);
    headers.set(tuples[0].0, tuples[0].1);
    headers.set(tuples[1].0, tuples[1].1);
    headers.append(tuples[2].0, tuples[2].1);
    headers.append(tuples[3].0, tuples[3].1);

    let mut count = 0;
    for ((k, v), (kb, vb)) in headers.items().zip(headers.items_bytes()) {
        assert_eq!(k, tuples[count].0);
        assert_eq!(kb, tuples[count].0.as_bytes());
        assert_eq!(v, tuples[count].1);
        assert_eq!(vb, tuples[count].1.as_bytes());
        count += 1;
    }
    assert_eq!(count, 4);

    count = 0;
    for (k, kb) in headers.keys().zip(headers.keys_bytes()) {
        assert_eq!(k, tuples[count].0);
        assert_eq!(kb, tuples[count].0.as_bytes());
        count += 1;
    }
    assert_eq!(count, 4);

    count = 0;
    for (v, vb) in headers.values().zip(headers.values_bytes()) {
        assert_eq!(v, tuples[count].1);
        assert_eq!(vb, tuples[count].1.as_bytes());
        count += 1;
    }
    assert_eq!(count, 4);
}

#[test]
fn header_sanitization() {
    let mut headers = HeaderMap::new(HeaderEncoding::Unicode);
    headers.set("Content-Type:", "text/html; charset=utf-8");
    headers.set("Foo:bar", "bar:baz");
    headers.set("new\r\nline", "new\t\nline");

    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/html; charset=utf-8"));
    assert_eq!(headers.get("Foobar").as_deref(), Some("bar:baz"));
    assert_eq!(headers.get("new  line").as_deref(), Some("new\t line"));
}

#[test]
fn parse_headers_with_continuation_lines() -> io::Result<()> {
    let http_data = b"HTTP/1.1 200 OK\r\n\
                              Content-Length: 123\r\n\
                              Content-Encoding     :     gzip    \r\n\
                              Content-Type: text/html;\r\n  charset=utf-8\r\n\
                              Invalid-Header-Ignored\r\n\
                              Accept: text/html,\r\n\tapplication/json,\r\n\ttext/plain\r\n\
                              \r\n";

    let mut headers = HeaderMap::new(HeaderEncoding::Latin1);
    let mut reader = io::Cursor::new(http_data);
    headers.parse(&mut reader, true)?;

    assert_eq!(headers.get("Content-Length").as_deref(), Some("123"));
    assert_eq!(headers.get("Content-Encoding").as_deref(), Some("gzip"));
    assert_eq!(headers.get("Content-Type").as_deref(), Some("text/html; charset=utf-8"));
    assert_eq!(headers.get("Accept").as_deref(), Some("text/html, application/json, text/plain"));

    assert!(!headers.keys().any(|k| k == "Invalid-Header-Ignored"));
    assert!(!headers.contains_key("Invalid-Header-Ignored"));
    assert!(!headers.values().any(|k| k == "Invalid-Header-Ignored"));

    Ok(())
}

#[test]
fn new_empty_header_encoding() -> io::Result<()> {
    let mut headers_unicode = HeaderMap::new(HeaderEncoding::Unicode);
    let mut headers_latin1 = HeaderMap::new(HeaderEncoding::Latin1);

    let utf8_value = "abcäöü";
    let latin1_bytes = WINDOWS_1252.encode(utf8_value, EncoderTrap::Ignore).unwrap_or_default();

    // Test Unicode encoding
    headers_unicode.set("X-Utf8", utf8_value);
    assert_eq!(headers_unicode.get("X-Utf8").as_deref(), Some(utf8_value));
    assert_eq!(headers_unicode.get_bytes(b"X-Utf8").as_deref(), Some(utf8_value.as_bytes()));

    // Test Latin1 encoding
    headers_latin1.set("X-Latin1", utf8_value);
    assert_eq!(headers_latin1.get("X-Latin1").as_deref(), Some(utf8_value));
    assert_eq!(headers_latin1.get_bytes(b"X-Latin1").as_deref(), Some(latin1_bytes.as_slice()));

    // Incorrect decodings
    let latin_value_utf8_dec_lossy = "abc���";
    let utf8_value_latin_dec = "abcÃ¤Ã¶Ã¼";

    // Test incorrect UTF-8 decoding of Latin bytes (irreversible)
    headers_unicode.set_bytes(b"X-Latin1-Utf8", latin1_bytes.as_slice());
    assert_eq!(headers_unicode.get("X-Latin1-Utf8").as_deref(), Some(latin_value_utf8_dec_lossy));
    assert_eq!(headers_unicode.get_bytes(b"X-Latin1-Utf8").as_deref(), Some(latin1_bytes.as_slice()));

    // Test incorrect Latin1 decoding of UTF-8 bytes (reversible)
    headers_latin1.set_bytes(b"X-Utf8-Latin1", utf8_value.as_bytes());
    assert_eq!(headers_latin1.get("X-Utf8-Latin1").as_deref(), Some(utf8_value_latin_dec));
    assert_eq!(headers_latin1.get_bytes(b"X-Utf8-Latin1").as_deref(), Some(utf8_value.as_bytes()));

    // Invalid UTF-8 sequence
    let invalid_utf8 = b"abc\xff\xfedef";
    let invalid_utf8_dec_lossy = "abc��def";
    let invalid_utf8_latin_dec = "abcÿþdef";

    // Test UTF-8 decoding with invalid UTF-8 sequence
    headers_unicode.set_bytes(b"X-Invalid", invalid_utf8);
    // Bytes should be the same
    assert_eq!(headers_unicode.get_bytes(b"X-Invalid").as_deref(), Some(invalid_utf8.as_ref()));
    // Decoding is lossy
    assert_eq!(headers_unicode.get("X-Invalid").as_deref(), Some(invalid_utf8_dec_lossy));

    // Test Latin decoding with invalid UTF-8 sequence
    headers_latin1.set_bytes(b"X-Invalid", invalid_utf8);
    // Bytes should be the same
    assert_eq!(headers_latin1.get_bytes(b"X-Invalid").as_deref(), Some(invalid_utf8.as_ref()));
    // Decodes to strange characters
    assert_eq!(headers_latin1.get("X-Invalid").as_deref(), Some(invalid_utf8_latin_dec));

    Ok(())
}

#[test]
fn parse_warc_headers() -> io::Result<()> {
    let record_data1 = warc_record_data("request", "<urn:uuid:record1>", None, b"ABC");
    let record_data2 = warc_record_data("response", "<urn:uuid:record2>", None, b"DEFGHI");
    let warc_data = [record_data1.as_slice(), record_data2.as_slice()].concat();

    let reader = Box::new(io::Cursor::new(warc_data));
    let mut record1 = WarcRecord::new();

    assert_eq!(record1.content_length(), 0);
    assert_eq!(record1.record_id(), None);
    assert_eq!(record1.record_type(), WarcRecordType::NoType);

    // Parse first record (construct manually)
    record1.attach_reader(reader);
    record1.parse_warc_headers()?;
    assert_eq!(record1.stream_pos(), 0);
    assert_eq!(record1.content_length(), 3);
    assert_eq!(record1.record_type(), WarcRecordType::Request);

    let headers = record1.headers();
    assert_eq!(headers.status_line().as_deref(), Some("WARC/1.1"));
    assert_eq!(headers.status_line_bytes().as_deref(), Some(b"WARC/1.1".as_slice()));
    assert!(!record1.is_http());
    assert_eq!(headers.get("WARC-Type").as_deref(), Some("request"));
    assert_eq!(headers.get_bytes(b"WARC-Type").as_deref(), Some(b"request".as_slice()));
    assert_eq!(record1.record_id().as_deref(), Some("<urn:uuid:record1>"));
    assert_eq!(headers.get("WARC-Record-ID").as_deref(), Some("<urn:uuid:record1>"));
    assert_eq!(headers.get_bytes(b"WARC-Record-ID").as_deref(), Some(b"<urn:uuid:record1>".as_slice()));
    assert_eq!(headers.get("Content-Length").as_deref(), Some("3"));
    assert_eq!(headers.get_bytes(b"Content-Length").as_deref(), Some(b"3".as_slice()));

    // Change headers
    let headers = record1.headers_mut();
    headers.set_status_line("WARC/1.0");
    assert_eq!(headers.status_line().as_deref(), Some("WARC/1.0"));
    headers.set("X-Foo", "Bar");
    assert_eq!(headers.get("X-Foo").as_deref(), Some("Bar"));

    // Read payload
    let mut buf = Vec::new();
    record1.reader_mut().unwrap().read_to_end(&mut buf)?;
    assert_eq!(String::from_utf8_lossy(&buf), "ABC");

    // Parse second record (construct directly from stream)
    let reader = record1.detach_reader().unwrap();
    let mut record2 = WarcRecord::from_reader(reader)?;

    assert_eq!(record2.stream_pos(), record_data1.len() as u64);
    assert_eq!(record2.content_length(), 6);
    assert_eq!(record2.record_type(), WarcRecordType::Response);

    buf.clear();
    record2.reader_mut().unwrap().read_to_end(&mut buf)?;
    assert_eq!(String::from_utf8_lossy(&buf), "DEFGHI");

    Ok(())
}

#[test]
fn parse_http_headers() -> io::Result<()> {
    let payload = "Hello World";
    let warc_data = http_response_warc_data("<urn:uuid:record-id>", payload);

    let reader = Box::new(io::Cursor::new(warc_data));
    let mut record = WarcRecord::new();
    record.attach_reader(reader);
    record.parse_warc_headers()?;

    let warc_headers = record.headers();
    assert_eq!(warc_headers.status_line().as_deref(), Some("WARC/1.1"));
    assert!(record.is_http());
    assert!(!record.is_http_parsed());
    assert!(record.http_headers().is_none());

    record.parse_http()?;
    assert!(record.is_http_parsed());
    let http_headers = record.http_headers().unwrap();
    assert_eq!(http_headers.status_line().as_deref(), Some("HTTP/1.1 200 OK"));
    assert_eq!(http_headers.status_code(), Some(200));
    assert_eq!(http_headers.reason_phrase().as_deref(), Some("OK"));
    assert_eq!(http_headers.get("Content-Type").as_deref(), Some("text/plain; charset=utf-8"));
    assert_eq!(record.http_charset().as_deref(), Some("utf-8"));
    assert_eq!(record.http_content_type().as_deref(), Some("text/plain"));

    let mut buf = Vec::new();
    record.reader_mut().unwrap().read_to_end(&mut buf)?;
    assert_eq!(String::from_utf8_lossy(&buf), payload);

    Ok(())
}

#[test]
fn parse_warc_headers_quirks_and_payload_replacement() -> io::Result<()> {
    let warc_data = b"garbage before header\r\n\
                      \r\n\
                      WARC/1.1\r\n\
                      WARC-Type: resource\r\n\
                      Content-Length: 3\r\n\
                      \r\n\
                      ABC\r\n\r\nrest"
        .to_vec();

    let mut strict = WarcRecord::new();
    strict.attach_reader(Box::new(io::Cursor::new(warc_data.clone())));
    let err = strict.parse_warc_headers().unwrap_err();
    assert_eq!(err.kind(), io::ErrorKind::InvalidData);

    let mut record = WarcRecord::new();
    record.attach_reader(Box::new(io::Cursor::new(warc_data)));
    let bytes_read = record.parse_warc_headers_quirks(true)?;
    assert!(bytes_read > 0);
    assert_eq!(record.headers().status_line().as_deref(), Some("WARC/1.1"));
    assert_eq!(record.record_type(), WarcRecordType::Resource);
    assert_eq!(record.content_length(), 3);

    record.set_bytes_payload(b"XYZ".to_vec());
    assert!(record.is_frozen());
    assert_eq!(record.content_length(), 3);
    assert_eq!(record.headers().get("Content-Length").as_deref(), Some("3"));

    let mut payload = Vec::new();
    record.reader_mut().unwrap().read_to_end(&mut payload)?;
    assert_eq!(payload, b"XYZ");

    let detached = record.detach_reader().unwrap();
    let mut detached = detached;
    assert!(detached.stream_position()? > 0);

    Ok(())
}

#[test]
fn record_init_from_bytes() -> io::Result<()> {
    let record_bytes = warc_record_data("response", "<urn:uuid:42e5b12c-3396-4b7e-b4b3-c88b7000cf43>", None, b"ABC");
    let record = WarcRecord::from_bytes(record_bytes)?;
    assert!(record.is_frozen());
    assert_eq!(record.record_type(), WarcRecordType::Response);
    assert_eq!(record.record_id().as_deref(), Some("<urn:uuid:42e5b12c-3396-4b7e-b4b3-c88b7000cf43>"));
    assert_eq!(record.content_length(), 3);
    Ok(())
}

#[test]
fn record_init_headers_http() -> io::Result<()> {
    let mut record = WarcRecord::new();
    record.init_headers(4, Some(WarcRecordType::AnyType), Some(b"uuid:494749ad-b14a-4f22-b143-0bab4347884b"));
    assert_eq!(record.record_type(), WarcRecordType::Unknown);
    assert_eq!(record.record_id().as_deref(), Some("<urn:uuid:494749ad-b14a-4f22-b143-0bab4347884b>"));
    assert_eq!(record.headers().status_line().as_deref(), Some("WARC/1.1"));
    assert_eq!(record.content_length(), 4);

    record.parse_http()?;
    assert!(!record.is_http_parsed());

    record.set_record_type(WarcRecordType::Request);
    record.set_is_http(true);
    assert!(record.is_http());
    assert_eq!(record.headers().get("Content-Type").as_deref(), Some("application/http; msgtype=request"));
    assert_eq!(record.http_content_type(), None);
    assert_eq!(record.http_charset(), None);

    Ok(())
}

#[test]
fn warc_record_debug_format() -> io::Result<()> {
    let mut non_http = WarcRecord::new();
    non_http.init_headers(3, Some(WarcRecordType::Resource), Some(b"urn:uuid:debug-resource"));
    non_http.set_bytes_payload(b"ABC".to_vec());

    let non_http_debug = format!("{non_http:?}");
    assert!(non_http_debug.contains("WarcRecord"));
    assert!(non_http_debug.contains("record_type: Resource"));
    assert!(non_http_debug.contains("content_length: 3"));
    assert!(non_http_debug.contains("is_http: false"));
    assert!(!non_http_debug.contains("http_charset"));
    assert!(!non_http_debug.contains("http_headers"));

    let mut http = WarcRecord::from_bytes(http_response_warc_data("<urn:uuid:debug-http>", "Hello"))?;
    http.parse_http()?;

    let http_debug = format!("{http:?}");
    assert!(http_debug.contains("record_type: Response"));
    assert!(http_debug.contains("is_http: true"));
    assert!(http_debug.contains("http_charset: Some(\"utf-8\")"));
    assert!(http_debug.contains("http_headers: Some"));

    Ok(())
}

#[test]
fn write_headers() -> io::Result<()> {
    let http_data = "HTTP/1.1 200 OK\r\n\
                              Content-Length: 123\r\n\
                              Content-Encoding: gzip\r\n\
                              Content-Type: text/html; charset=utf-8\r\n\
                              \r\n";

    let mut headers = HeaderMap::new(HeaderEncoding::Latin1);
    headers.set_status_line("HTTP/1.1 200 OK");
    headers.set("Content-Length", "456");
    headers.set("Content-Encoding", "gzip");
    headers.set("Content-Type", "text/html; charset=utf-8");
    headers.set("Content-Length", "123");

    let mut buf = Vec::with_capacity(http_data.len());
    headers.write(&mut buf)?;
    assert_eq!(String::from_utf8_lossy(&buf), http_data);

    Ok(())
}

#[test]
fn write_record() -> io::Result<()> {
    let payload = b"ABCDE".to_vec();

    let mut default_write = WarcRecord::new();
    default_write.init_headers(payload.len() as u64, Some(WarcRecordType::Resource), Some(b"urn:uuid:write-default"));
    default_write.set_bytes_payload(payload.clone());

    let mut default_serialized = Vec::new();
    let default_bytes_written = default_write.write(&mut default_serialized)?;
    assert_eq!(default_bytes_written, default_serialized.len());
    assert!(String::from_utf8_lossy(&default_serialized).contains("WARC-Type: resource\r\n"));
    assert!(default_serialized.ends_with(b"ABCDE\r\n\r\n"));
    assert!(!String::from_utf8_lossy(&default_serialized).contains("WARC-Block-Digest"));

    Ok(())
}

#[test]
fn write_record_with_block_size() -> io::Result<()> {
    let payload = b"ABCDE".to_vec();

    let mut chunked_write = WarcRecord::new();
    chunked_write.init_headers(payload.len() as u64, Some(WarcRecordType::Resource), Some(b"urn:uuid:write-chunked"));
    chunked_write.set_bytes_payload(payload);

    let mut chunked_serialized = Vec::new();
    let chunked_bytes_written = chunked_write.write_with_block_size(&mut chunked_serialized, 2)?;
    assert_eq!(chunked_bytes_written, chunked_serialized.len());
    assert!(chunked_serialized.ends_with(b"ABCDE\r\n\r\n"));
    assert!(!String::from_utf8_lossy(&chunked_serialized).contains("WARC-Block-Digest"));

    Ok(())
}

#[test]
fn write_record_with_checksum() -> io::Result<()> {
    let mut checksummed = WarcRecord::from_bytes(http_response_warc_data("<urn:uuid:write-checksum>", "Hello"))?;
    checksummed.parse_http()?;

    let http_headers = b"HTTP/1.1 200 OK\r\n\
                         Content-Type: text/plain; charset=utf-8\r\n\
                         Content-Length: 5\r\n\
                         Server: nginx\r\n\
                         \r\n";
    let expected_payload_digest = format!("sha1:{}", BASE32.encode(&Sha1::digest(http_headers)));

    let mut block_data = http_headers.to_vec();
    block_data.extend_from_slice(b"Hello");
    let expected_block_digest = format!("sha1:{}", BASE32.encode(&Sha1::digest(&block_data)));

    let mut checksummed_serialized = Vec::new();
    let checksummed_bytes_written = checksummed.write_with_checksum(&mut checksummed_serialized)?;
    let checksummed_text = String::from_utf8_lossy(&checksummed_serialized);
    assert_eq!(checksummed_bytes_written, checksummed_serialized.len());
    assert!(checksummed_text.contains(&format!("WARC-Block-Digest: {expected_block_digest}\r\n")));
    assert!(checksummed_text.contains(&format!("WARC-Payload-Digest: {expected_payload_digest}\r\n")));
    assert!(checksummed_text.contains("HTTP/1.1 200 OK\r\n"));
    assert!(checksummed_serialized.ends_with(b"Hello\r\n\r\n"));

    Ok(())
}

#[test]
fn verify_record_digests() -> io::Result<()> {
    let payload = b"ABC".to_vec();
    let mut record = WarcRecord::new();
    record.init_headers(payload.len() as u64, Some(WarcRecordType::Resource), Some(b"urn:uuid:digest-test"));
    record.set_bytes_payload(payload.clone());

    let digest = BASE32.encode(&Sha1::digest(&payload));
    record.headers_mut().set("WARC-Block-Digest", format!("sha1:{digest}"));

    // `consume = false` should leave the frozen payload reader rewound for later use.
    assert!(record.verify_block_digest(false).unwrap());
    assert_eq!(record.reader_mut().unwrap().stream_position()?, 0);

    let md5_digest = BASE32.encode(&Md5::digest(&payload));
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("md5:{md5_digest}"));
    assert!(record.verify_block_digest(false).unwrap());
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("md5:{}", BASE32.encode(b"xxxxxx")));
    assert!(!record.verify_block_digest(false).unwrap());

    let sha256_digest = BASE32.encode(&Sha256::digest(&payload));
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("sha256:{sha256_digest}"));
    assert!(record.verify_block_digest(false).unwrap());
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("sha256:{}", BASE32.encode(b"xxxxxx")));
    assert!(!record.verify_block_digest(false).unwrap());

    let sha512_digest = BASE32.encode(&Sha512::digest(&payload));
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("sha512:{sha512_digest}"));
    assert!(record.verify_block_digest(false).unwrap());
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("sha512:{}", BASE32.encode(b"xxxxxx")));
    assert!(!record.verify_block_digest(false).unwrap());

    let sha1_hex_digest = HEXLOWER.encode(&Sha1::digest(&payload));
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("sha1:{sha1_hex_digest}"));
    assert!(record.verify_block_digest(false).unwrap());
    record
        .headers_mut()
        .set("WARC-Block-Digest", format!("sha1:{}", "0".repeat(sha1_hex_digest.len())));
    assert!(!record.verify_block_digest(false).unwrap());

    // Cover formatting failures.
    record.headers_mut().remove("WARC-Block-Digest");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::Missing(_))));
    record.headers_mut().set("WARC-Block-Digest", "sha999:AAAA");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::Unsupported(_))));
    record.headers_mut().set("WARC-Block-Digest", "bad-format");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::FormatError(_))));
    record.headers_mut().set("WARC-Block-Digest", "sha1:_____");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::FormatError(_))));
    assert!(matches!(record.verify_payload_digest(false), Err(DigestError::NoPayload(_))));

    let mut http_without_payload_digest =
        WarcRecord::from_bytes(http_response_warc_data("<urn:uuid:missing-payload-digest>", "Hello"))?;
    http_without_payload_digest.parse_http()?;
    assert!(matches!(http_without_payload_digest.verify_payload_digest(false), Err(DigestError::Missing(_))));

    let http_payload = "Hello";
    let http_data = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         \r\n\
         {}",
        http_payload.len(),
        http_payload
    );
    let mut http_with_digests = WarcRecord::from_bytes(warc_record_data_with_headers(
        "response",
        "<urn:uuid:payload-digest-consume>",
        Some("application/http; msgtype=response"),
        &format!(
            "WARC-Block-Digest: sha1:{}\r\n\
                 WARC-Payload-Digest: sha1:{}\r\n",
            BASE32.encode(&Sha1::digest(http_data.as_bytes())),
            BASE32.encode(&Sha1::digest(http_payload.as_bytes()))
        ),
        http_data.as_bytes(),
    ))?;
    assert!(http_with_digests.verify_block_digest(false).unwrap());
    http_with_digests.parse_http()?;
    assert!(http_with_digests.verify_payload_digest(true).unwrap());
    assert!(!http_with_digests.verify_block_digest(false).unwrap());

    let mut writable = WarcRecord::new();
    writable.init_headers(payload.len() as u64, Some(WarcRecordType::Resource), Some(b"urn:uuid:write-test"));
    writable.set_bytes_payload(payload);

    // Serialize and reparse.
    let mut serialized = Vec::new();
    let bytes_written = writable.write_with_block_size_checksum(&mut serialized, 2, true)?;
    assert_eq!(bytes_written, serialized.len());

    let mut reparsed = WarcRecord::from_reader(Box::new(io::Cursor::new(serialized)))?;
    assert!(reparsed.headers().contains_key("WARC-Block-Digest"));
    assert!(reparsed.verify_block_digest(false).unwrap());

    let mut reparsed_payload = Vec::new();
    reparsed.reader_mut().unwrap().read_to_end(&mut reparsed_payload)?;
    assert_eq!(reparsed_payload, b"ABC");

    Ok(())
}

#[test]
fn verify_record_digest_error_kinds() -> io::Result<()> {
    let payload = b"ABC".to_vec();
    let mut record = WarcRecord::new();
    record.init_headers(payload.len() as u64, Some(WarcRecordType::Resource), Some(b"urn:uuid:digest-errors"));
    record.set_bytes_payload(payload);

    record.headers_mut().remove("WARC-Block-Digest");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::Missing(_))));

    record.headers_mut().set("WARC-Block-Digest", "sha999:AAAA");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::Unsupported(_))));

    record.headers_mut().set("WARC-Block-Digest", "bad-format");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::FormatError(_))));

    record.headers_mut().set("WARC-Block-Digest", "sha1:_____");
    assert!(matches!(record.verify_block_digest(false), Err(DigestError::FormatError(_))));

    assert!(matches!(record.verify_payload_digest(false), Err(DigestError::NoPayload(_))));

    let mut stream_error_record = WarcRecord::new();
    stream_error_record.init_headers(3, Some(WarcRecordType::Resource), Some(b"urn:uuid:digest-stream-error"));
    stream_error_record.headers_mut().set("WARC-Block-Digest", "sha1:AAAA");
    assert!(matches!(stream_error_record.verify_block_digest(false), Err(DigestError::StreamError(_))));

    Ok(())
}

#[test]
fn archive_iterator_option_setters() -> io::Result<()> {
    let reader = Box::new(io::Cursor::new(Vec::new()));
    let it = ArchiveIterator::new(reader.clone());
    assert_eq!(it.options, ArchiveIteratorOptions::default());

    let new_opts = ArchiveIteratorOptions {
        parse_http: false,
        decode_http_payload: AutoDecode::All,
        verify_digests: true,
        quirks_mode: true,
    };
    assert_ne!(ArchiveIteratorOptions::default().parse_http, new_opts.parse_http);
    assert_ne!(ArchiveIteratorOptions::default().decode_http_payload, new_opts.decode_http_payload);
    assert_ne!(ArchiveIteratorOptions::default().verify_digests, new_opts.verify_digests);
    assert_ne!(ArchiveIteratorOptions::default().quirks_mode, new_opts.quirks_mode);

    // Test constructor with options
    let it = ArchiveIterator::with_options(reader.clone(), new_opts);
    assert_eq!(it.options, new_opts);

    // Test setters
    let mut it = ArchiveIterator::new(reader.clone());
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
fn record_consume_and_freeze_stream_payload() -> io::Result<()> {
    let record_data1 = warc_record_data("resource", "<urn:uuid:consume-freeze-1>", None, b"ABCDEF");
    let record_data2 = warc_record_data("metadata", "<urn:uuid:consume-freeze-2>", None, b"XYZ");

    let mut consumed = WarcRecord::from_reader(Box::new(io::Cursor::new(
        [record_data1.as_slice(), record_data2.as_slice()].concat(),
    )))?;
    assert_eq!(consumed.consume_n(2)?, 2);
    assert_eq!(consumed.reader_mut().unwrap().stream_position()?, 2);
    assert_eq!(consumed.consume()?, 4);
    assert_eq!(consumed.reader_mut().unwrap().stream_position()?, 6);

    let next = consumed.next().unwrap()?;
    assert_eq!(next.record_id().as_deref(), Some("<urn:uuid:consume-freeze-2>"));

    let mut frozen = WarcRecord::from_reader(Box::new(io::Cursor::new(record_data1)))?;
    let mut prefix = [0u8; 2];
    frozen.reader_mut().unwrap().read_exact(&mut prefix)?;
    assert_eq!(&prefix, b"AB");

    frozen.freeze()?;
    assert!(frozen.is_frozen());
    assert_eq!(frozen.content_length(), 4);

    let mut remaining = Vec::new();
    frozen.reader_mut().unwrap().read_to_end(&mut remaining)?;
    assert_eq!(remaining, b"CDEF");

    Ok(())
}

#[test]
fn record_encoded_http_payload() -> io::Result<()> {
    let payload_raw = b"ABCDEF".repeat(2000);

    let read_record = |payload_encoded: &[u8], tenc, cenc, dec_opts| -> io::Result<Vec<u8>> {
        let data = http_response_warc_data_encoded("<urn:uuid:abc>", payload_encoded, tenc, cenc);
        let mut rec = WarcRecord::from_reader(Box::new(io::Cursor::new(data)))?;
        rec.parse_http_with_decode_opts(dec_opts)?;
        let mut buf = Vec::with_capacity(payload_raw.len());
        rec.reader_mut().unwrap().read_to_end(&mut buf)?;
        Ok(buf)
    };

    // Transfer-Encoding: gzip (decode: None)
    let mut w = gzip::GzipWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let encoded = w.into_inner()?;
    let decoded = read_record(&encoded, Some("gzip"), None, AutoDecode::None)?;
    assert_eq!(decoded, encoded);

    // Transfer-Encoding: gzip, Content-Encoding: gzip (decode: All)
    let mut w = gzip::GzipWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let encoded = w.into_inner()?;
    let mut w = gzip::GzipWriter::new(Vec::new());
    w.write_all(&encoded)?;
    let encoded = w.into_inner()?;
    let decoded = read_record(&encoded, Some("gzip"), Some("gzip"), AutoDecode::All)?;
    assert_eq!(decoded, payload_raw);

    // Transfer-Encoding: gzip, Content-Encoding: br (decode: Transfer)
    let mut w = brotli::BrotliWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let encoded = w.into_inner()?;
    let mut w = gzip::GzipWriter::new(Vec::new());
    w.write_all(&encoded)?;
    let encoded = w.into_inner()?;
    let mut w = brotli::BrotliWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let encoded_content = w.into_inner()?;
    let decoded = read_record(&encoded, Some("gzip"), Some("br"), AutoDecode::TransferEncoding)?;
    assert_eq!(decoded, encoded_content);

    // Transfer-Encoding: zstd, deflate (decode: Transfer)
    let mut w = zstd::ZstdWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let encoded = w.into_inner()?;
    let mut w = gzip::GzipWriter::new_deflate(Vec::new());
    w.write_all(&encoded)?;
    let encoded = w.into_inner()?;
    let decoded = read_record(&encoded, Some("zstd, deflate"), None, AutoDecode::TransferEncoding)?;
    assert_eq!(decoded, payload_raw);

    // Transfer-Encoding: chunked (decode: Transfer)
    let mut w = chunked::ChunkedWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let encoded = w.into_inner()?;
    let decoded = read_record(&encoded, Some("chunked"), None, AutoDecode::TransferEncoding)?;
    assert_eq!(decoded, payload_raw);

    // Transfer-Encoding: chunked, Content-Encoding: gzip (decode: All)
    let mut w = gzip::GzipWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let encoded = w.into_inner()?;
    let mut w = chunked::ChunkedWriter::new(Vec::new());
    w.write_all(&encoded)?;
    let encoded = w.into_inner()?;
    let decoded = read_record(&encoded, Some("chunked"), Some("gzip"), AutoDecode::All)?;
    assert_eq!(decoded, payload_raw);

    // Transfer-Encoding: identity (decode: Transfer)
    let decoded = read_record(payload_raw.as_slice(), Some("identity"), None, AutoDecode::TransferEncoding)?;
    assert_eq!(decoded, payload_raw);

    // Content-Encoding: unsupported (decode: Transfer)
    let decoded = read_record(payload_raw.as_slice(), None, Some("unsupported"), AutoDecode::TransferEncoding)?;
    assert_eq!(decoded, payload_raw);

    // Content-Encoding: unsupported (decode: Content)
    let decoded = read_record(payload_raw.as_slice(), None, Some("unsupported"), AutoDecode::ContentEncoding);
    assert!(decoded.is_err());

    Ok(())
}

#[test]
fn record_encoded_http_payload_frozen_record() -> io::Result<()> {
    let payload_raw = b"ABCDEF".repeat(2000);

    // Encode payload and check that output starts with Gzip magic bytes.
    let mut w = gzip::GzipWriter::new(Vec::new());
    w.write_all(&payload_raw)?;
    let payload_encoded = w.into_inner()?;
    assert_eq!(payload_encoded[..2], [0x1F, 0x8B]);

    // Create HTTP WARC record with encoded payload.
    let record_data = http_response_warc_data_encoded("<urn:uuid:abc>", &payload_encoded, Some("gzip"), None);
    let mut rec = WarcRecord::from_reader(Box::new(io::Cursor::new(record_data.clone())))?;

    // Freeze record
    rec.freeze()?;

    // Parse HTTP and decode content.
    rec.parse_http_with_decode_opts(AutoDecode::TransferEncoding)?;
    let mut decoded = Vec::with_capacity(payload_raw.len());
    rec.reader_mut().unwrap().read_to_end(&mut decoded)?;
    assert_eq!(decoded, payload_raw);

    // Rewind frozen stream and read again (same result)
    decoded.clear();
    rec.reader_mut().unwrap().rewind()?;
    rec.reader_mut().unwrap().read_to_end(&mut decoded)?;
    assert_eq!(decoded, payload_raw);

    // Detach and rewind reader.
    let mut detached = rec.detach_reader().unwrap();
    detached.rewind()?;

    // Create new record from stream and HTTP without content decoding.
    let mut rec = WarcRecord::from_reader(detached)?;
    rec.parse_http_with_decode_opts(AutoDecode::None)?;

    // Test that stream begins with Gzip magic bytes
    let mut magic_bytes = [0u8; 2];
    rec.reader_mut().unwrap().read_exact(&mut magic_bytes)?;
    assert_eq!(magic_bytes, payload_encoded[..2]);
    assert_ne!(magic_bytes, payload_raw[..2]);

    // Test freezing and eager decoding don't mess with record boundaries.
    let record_data = [record_data.clone(), record_data].concat();
    let mut count = 0;
    for r in ArchiveIterator::new(Box::new(io::Cursor::new(record_data))).with_parse_http(false) {
        r?.with_mut(|r| -> io::Result<()> {
            r.freeze()?;
            r.parse_http_with_decode_opts(AutoDecode::TransferEncoding)?;
            let mut buf = Vec::with_capacity(r.content_length() as usize);
            r.reader_mut().unwrap().read_to_end(&mut buf)?;
            assert_eq!(buf, payload_raw);
            count += 1;
            Ok(())
        })?;
    }
    assert_eq!(count, 2);

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
