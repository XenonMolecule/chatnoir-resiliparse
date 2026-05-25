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

from datetime import datetime
from collections.abc import Callable, Iterable, Iterator
from enum import IntFlag
from typing import BinaryIO, final, Literal, Self
from typing_extensions import disjoint_base

from .stream_io import _GenericReader, _GenericWriter, _ReaderInput, WarcReader, WarcWriter

__all__ = [
    'WarcRecordType',
    'WarcRecord',
    'ArchiveIterator',
    'HeaderMap',
    'WarcHeaderMap',
    'WarcRecordPayloadReader',

    'warcinfo',
    'response',
    'resource',
    'request',
    'metadata',
    'revisit',
    'conversion',
    'continuation',
    'unknown',
    'any_type',
    'no_type',

    'is_warc_10',
    'is_warc_11',
    'has_block_digest',
    'has_valid_block_digest',
    'has_payload_digest',
    'has_valid_payload_digest',
    'is_http',
    'is_concurrent',
]


@final
class WarcRecordType(IntFlag):
    warcinfo = 2
    response = 4
    resource = 8
    request = 16
    metadata = 32
    revisit = 64
    conversion = 128
    continuation = 256
    unknown = 512
    any_type = 65535
    no_type = 0

    def __int__(self) -> int: ...

    def __index__(self) -> int: ...

    def __and__(self, other: int, /) -> Self: ...

    def __or__(self, other: int, /) -> Self: ...

    def __xor__(self, other: int, /) -> Self: ...

    def __str__(self) -> str:
        return self.name or str(self.value)

    def __repr__(self) -> str:
        return self.name or str(self.value)


warcinfo = WarcRecordType.warcinfo
response = WarcRecordType.response
resource = WarcRecordType.resource
request = WarcRecordType.request
metadata = WarcRecordType.metadata
revisit = WarcRecordType.revisit
conversion = WarcRecordType.conversion
continuation = WarcRecordType.continuation
unknown = WarcRecordType.unknown
no_type = WarcRecordType.no_type
any_type = WarcRecordType.any_type


@final
class HeaderMap:
    def __new__(cls, encoding: str = "utf-8") -> Self: ...

    @property
    def reason_phrase(self) -> str | None: ...

    @property
    def status_code(self) -> int | None: ...

    @property
    def status_line(self) -> str | None: ...

    @status_line.setter
    def status_line(self, status_line: str): ...

    @property
    def status_line_bytes(self) -> bytes | None: ...

    @status_line_bytes.setter
    def status_line_bytes(self, status_line: bytes): ...

    def encoding(self) -> str: ...

    def parse(self, reader: WarcReader | BinaryIO | _GenericReader, has_status_line: bool = True) -> int: ...

    def write(self, writer: WarcWriter | BinaryIO | _GenericWriter) -> int: ...

    def append(self, key: str, value: str): ...

    def to_dict(self) -> dict[str, str]: ...

    # deprecated
    def asdict(self) -> dict[str, str]: ...

    def to_tuples(self) -> tuple[tuple[str, str], ...]: ...

    # deprecated
    def astuples(self) -> tuple[tuple[str, str], ...]: ...

    def is_empty(self) -> bool: ...

    def clear(self): ...

    def get(self, key: str, default: str | None = None) -> str | None: ...

    def get_multiple(self, key: str) -> tuple[str, ...]: ...

    def get_bytes(self, key: bytes, default: bytes | None = None) -> bytes | None: ...

    def get_bytes_multiple(self, key: bytes) -> tuple[bytes, ...]: ...

    def contains_key(self, key: str) -> bool: ...

    def contains_key_bytes(self, key: bytes) -> bool: ...

    def set(self, key: str, value: str): ...

    def set_bytes(self, key: bytes, value: bytes): ...

    def append_bytes(self, key: bytes, value: bytes): ...

    def remove(self, key: str): ...

    def remove_bytes(self, key: bytes): ...

    def items(self) -> tuple[tuple[str, str], ...]: ...

    def items_bytes(self) -> tuple[tuple[bytes, bytes], ...]: ...

    def keys(self) -> tuple[str, ...]: ...

    def keys_bytes(self) -> tuple[bytes, ...]: ...

    def values(self) -> tuple[str, ...]: ...

    def values_bytes(self) -> tuple[bytes, ...]: ...

    def __getitem__(self, item: str, /) -> str: ...

    def __iter__(self) -> Iterator[tuple[str, str]]: ...

    def __len__(self) -> int: ...

    def __setitem__(self, key: str, value: str, /): ...

    def __contains__(self, item: str, /) -> bool: ...

    def __eq__(self, other, /) -> bool: ...

    def __ne__(self, other, /) -> bool: ...

    def __str__(self) -> str: ...

    def __repr__(self) -> str: ...

    def __getnewargs__(self) -> tuple[str]: ...

    def __getstate__(self) -> tuple[bytes, bool]: ...

    def __setstate__(self, state: tuple[bytes, bool]): ...


# Legacy name
WarcHeaderMap = HeaderMap


@final
class WarcRecordPayloadReader(WarcReader):
    def readline(self, max_line_len: int = 8192) -> bytes: ...

    def consume(self, size: int = ...) -> int: ...


@disjoint_base
class WarcRecord:
    def __new__(cls) -> Self: ...

    @property
    def record_id(self) -> str | None: ...

    @record_id.setter
    def record_id(self, record_id: str): ...

    @property
    def record_type(self) -> WarcRecordType: ...

    @record_type.setter
    def record_type(self, record_type: WarcRecordType | int): ...

    @property
    def content_length(self) -> int: ...

    @property
    def record_date(self) -> datetime: ...

    @record_date.setter
    def record_date(self, record_date: datetime): ...

    @property
    def headers(self) -> HeaderMap: ...

    @property
    def is_http(self) -> bool: ...

    @is_http.setter
    def is_http(self, is_http: bool): ...

    @property
    def is_http_parsed(self) -> bool: ...

    @property
    def http_headers(self) -> HeaderMap: ...

    @property
    def http_content_type(self) -> str | None: ...

    @property
    def http_charset(self) -> str | None: ...

    @property
    def http_date(self) -> datetime: ...

    @property
    def http_last_modified(self) -> datetime: ...

    @property
    def reader(self) -> WarcRecordPayloadReader: ...

    @property
    def stream_pos(self) -> int: ...

    @property
    def is_frozen(self) -> bool: ...

    def init_headers(self, record_type: WarcRecordType = ..., record_urn: bytes | None = None, *,
                     content_length: int | None = None): ...

    def freeze(self) -> bool: ...

    def set_bytes_content(self, content: bytes): ...

    def set_bytes_payload(self, content: bytes): ...

    def consume(self, n: int | None = None) -> int: ...

    def parse_warc_headers(self, quirks_mode: bool = False) -> int: ...

    def parse_http(self, auto_decode: Literal['none', 'content', 'transfer', 'all'] = 'none', *,
                   strict_mode: bool = True): ...

    def verify_block_digest(self, consume: bool = False) -> bool: ...

    def verify_payload_digest(self, consume: bool = False) -> bool: ...

    def write(
            self,
            stream: WarcWriter | BinaryIO | _GenericWriter,
            checksum_data: bool = False,
            payload_digest: bytes | None = None,
            chunk_size: int = 16384
    ) -> int: ...

    def __str__(self) -> str: ...

    def __repr__(self) -> str: ...

    def __getnewargs__(self) -> tuple[()]: ...

    def __getstate__(self) -> tuple[bytes, bool]: ...

    def __setstate__(self, state: tuple[bytes, bool]): ...


@final
class ArchiveIterator(Iterable[WarcRecord]):
    def __new__(
            cls,
            stream: _ReaderInput,
            record_types: WarcRecordType = any_type,
            parse_http: bool = True,
            min_content_length: int | None = None,
            max_content_length: int | None = None,
            func_filter: Callable[[WarcRecord], bool] | None = None,
            verify_digests: bool = False,
            quirks_mode: bool = False,
            auto_decode: Literal['none', 'content', 'transfer', 'all'] = 'none',
            stream_detect: bool = True,
            fsspec_args=None,
            *,
            strict_mode: bool = True
    ) -> Self: ...

    def __iter__(self) -> Iterator[WarcRecord]: ...

    def __next__(self) -> WarcRecord: ...


# ArchiveIterator filter predicates

def is_warc_10(record: WarcRecord) -> bool: ...


def is_warc_11(record: WarcRecord) -> bool: ...


def has_block_digest(record: WarcRecord) -> bool: ...


def has_valid_block_digest(record: WarcRecord) -> bool: ...


def has_payload_digest(record: WarcRecord) -> bool: ...


def has_valid_payload_digest(record: WarcRecord) -> bool: ...


def is_http(record: WarcRecord) -> bool: ...


def is_concurrent(record: WarcRecord) -> bool: ...
