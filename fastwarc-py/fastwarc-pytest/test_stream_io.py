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

import io
import pytest
from fastwarc.stream_io import *


@pytest.mark.parametrize(
    ("writer_cls", "reader_cls", "reader_supports_frame_position"),
    [
        (GzipWriter, GzipReader, True),
        (BrotliWriter, BrotliReader, False),
        (Lz4Writer, Lz4Reader, True),
        (ZstdWriter, ZstdReader, True),
    ],
)
def test_stream_reader_writer(writer_cls, reader_cls, reader_supports_frame_position):
    payload = b"alpha\nbeta\n"
    raw = io.BytesIO()

    writer = writer_cls(raw)
    assert writer.write(payload) == len(payload)
    writer.flush()
    writer.finish()
    writer.close()

    encoded = raw.getvalue()
    assert encoded

    with reader_cls(io.BytesIO(encoded)) as reader:
        assert reader.tell() == 0
        assert reader.seek(0) == 0
        assert reader.read(6) == b"alpha\n"
        assert reader.tell() == 6
        assert reader.read() == b"beta\n"
        assert isinstance(reader.inner_tell(), int)
        if reader_supports_frame_position:
            assert reader.frame_start_position() == 0
        else:
            assert reader.frame_start_position() is None
        assert reader.inner_seek(0) == 0


def test_chunked_reader_writer():
    payload = b"alpha\nbeta\n"
    raw = io.BytesIO()

    with ChunkedWriter(raw, min_chunk_size=1) as writer:
        assert writer.write(payload[:5]) == 5
        assert writer.write(payload[5:]) == len(payload) - 5
        writer.flush()
        writer.finish()

    encoded = raw.getvalue()
    assert encoded.startswith(b"5\r\nalpha\r\n")

    with ChunkedReader(io.BytesIO(encoded)) as reader:
        assert reader.seek(0) == 0
        assert reader.read(6) == b"alpha\n"
        assert reader.tell() == 6
        assert reader.inner_seek(0) == 0
        assert isinstance(reader.inner_tell(), int)


def test_stream_io_context_manager():
    payload = b"context manager payload"
    raw = io.BytesIO()

    with GzipWriter(raw) as writer:
        assert writer.write(payload) == len(payload)
        writer.flush()
        writer.finish()

    with GzipReader(io.BytesIO(raw.getvalue())) as reader:
        assert reader.read() == payload
