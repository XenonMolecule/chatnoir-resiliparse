# Copyright 2026 Janek Bevendorff
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import datetime as dt
import hashlib
import io
import pickle
from pathlib import Path

import pytest

import fastwarc
from fastwarc.stream_io import *
from fastwarc.warc import *

HTTP_BODY = b"Hello"


def _http_payload(body: bytes = HTTP_BODY) -> bytes:
    return (
            b"HTTP/1.1 200 OK\r\n"
            b"Date: Tue, 15 Nov 1994 08:12:31 GMT\r\n"
            b"Last-Modified: Tue, 15 Nov 1994 12:45:26 GMT\r\n"
            b"Content-Type: text/plain; charset=utf-8\r\n"
            + f"Content-Length: {len(body)}\r\n".encode()
            + b"\r\n"
            + body
    )


def _make_http_record(record_type=response, *, record_urn: bytes = b"urn:uuid:test-record",
                      body: bytes = HTTP_BODY) -> WarcRecord:
    record = WarcRecord()
    record.init_headers(unknown, record_urn)
    record.record_type = record_type
    record.is_http = True
    record.set_bytes_content(_http_payload(body))
    return record


def _serialize_record(record: WarcRecord) -> bytes:
    stream = io.BytesIO()
    record.write(stream)
    return stream.getvalue()


def test_package_reexports_and_legacy_shims():
    assert WarcRecord is WarcRecord
    assert HeaderMap is HeaderMap
    assert FileStream is FileStream
    assert GZipStream is GZipStream
    assert LZ4Stream is LZ4Stream
    assert "FileStream" in fastwarc.__all__
    assert "GZipStream" in fastwarc.stream_io.__all__


def test_header_write_parse_roundtrip():
    headers = HeaderMap("latin1")
    headers.status_line_bytes = b"HTTP/1.1 204 No Content"
    headers.append("Set-Cookie", "a=1")
    headers.append_bytes(b"Set-Cookie", b"b=2")
    headers.set_bytes(b"X-Test", b"abc")

    assert headers.encoding() == "latin1"
    assert headers.status_code == 204
    assert headers.reason_phrase == "No Content"
    assert headers.get("missing", "fallback") == "fallback"
    assert headers.get_bytes(b"missing", b"fallback") == b"fallback"
    assert headers.get_multiple("Set-Cookie") == ("a=1", "b=2")
    assert headers.get_bytes_multiple(b"Set-Cookie") == (b"a=1", b"b=2")
    assert headers.contains_key("set-cookie")
    assert headers.contains_key_bytes(b"x-test")
    assert tuple(iter(headers)) == headers.items()
    assert headers["X-Test"] == "abc"
    assert "X-Test" in headers

    serialized = io.BytesIO()
    assert headers.write(serialized) > 0
    serialized.seek(0)

    parsed = HeaderMap("latin1")
    assert parsed.parse(serialized, has_status_line=True) > 0
    assert parsed == headers
    assert parsed.to_dict()["Set-Cookie"] == "a=1,b=2"
    assert parsed.items_bytes()[0][0] == b"Set-Cookie"


def test_warc_record_write():
    record = _make_http_record()
    record.record_date = dt.datetime(2025, 5, 1, 12, 30, tzinfo=dt.timezone(dt.timedelta(hours=2)))
    record.headers["WARC-Concurrent-To"] = "<urn:uuid:other>"

    payload_digest = hashlib.sha1(HTTP_BODY).digest()
    stream = io.BytesIO()
    bytes_written = record.write(stream, checksum_data=True, payload_digest=payload_digest)

    assert bytes_written == len(stream.getvalue())
    assert record.record_date == dt.datetime(2025, 5, 1, 12, 30, tzinfo=dt.timezone(dt.timedelta(hours=2)))

    iterator = ArchiveIterator(io.BytesIO(stream.getvalue()), parse_http=False)
    rec = next(iterator)

    assert rec.record_type == response
    assert rec.is_http is True
    assert rec.stream_pos == 0
    assert has_block_digest(rec) is True
    assert has_payload_digest(rec) is True
    assert has_valid_block_digest(rec) is True
    assert has_valid_payload_digest(rec) is False
    assert rec.verify_block_digest() is True
    assert rec.verify_payload_digest() is False
    assert is_http(rec) is True

    rec.parse_http(strict_mode=False)
    assert rec.http_content_type == "text/plain"
    assert rec.http_charset == "utf-8"
    assert rec.http_date == dt.datetime(1994, 11, 15, 8, 12, 31, tzinfo=dt.timezone.utc)
    assert rec.http_last_modified == dt.datetime(1994, 11, 15, 12, 45, 26, tzinfo=dt.timezone.utc)
    assert rec.verify_payload_digest() is True
    assert has_valid_payload_digest(rec) is True

    reader = rec.reader
    assert reader.readline() == HTTP_BODY
    assert reader.tell() == len(HTTP_BODY)
    assert reader.seek(0) == 0
    assert reader.read(2) == HTTP_BODY[:2]
    assert reader.consume() == len(HTTP_BODY) - 2

    restored = pickle.loads(pickle.dumps(rec))
    assert restored.record_id == rec.record_id
    assert restored.record_type == rec.record_type
    assert restored.is_http_parsed is True
    assert restored.http_content_type == "text/plain"


def test_record_and_header_pickle():
    record = WarcRecord()
    record.init_headers(unknown, b"urn:uuid:test-record")
    record.record_type = response
    record.is_http = True
    record.set_bytes_content(_http_payload())
    assert record.is_frozen

    # Pickle headers
    assert pickle.loads(pickle.dumps(record.headers)) == record.headers

    # Pickle entire record
    record_roundtrip = pickle.loads(pickle.dumps(record))
    assert record_roundtrip is not record
    assert record_roundtrip.is_frozen
    assert record_roundtrip.record_id == record.record_id
    assert record_roundtrip.content_length == record.content_length
    assert record_roundtrip.headers is not record.headers
    assert record_roundtrip.headers == record.headers
    record.reader.seek(0)
    assert record_roundtrip.reader.read() == record.reader.read()


def test_archive_iterator_accepts_pathlike_and_filters(tmp_path: Path):
    first = _make_http_record(response, record_urn=b"uuid:first")
    second = _make_http_record(request, record_urn=b"uuid:second", body=b"World")
    warc_path = tmp_path / "sample.warc"
    warc_path.write_bytes(_serialize_record(first) + _serialize_record(second))

    seen_ids = []
    iterator = ArchiveIterator(
        warc_path,
        record_types=response | request,
        parse_http=False,
        min_content_length=1,
        func_filter=lambda rec: rec.record_type == response,
        fsspec_args=False,
    )

    assert iter(iterator) is iterator
    record = next(iterator)
    seen_ids.append(record.record_id)
    with pytest.raises(StopIteration):
        next(iterator)

    assert seen_ids == ["<urn:uuid:first>"]
