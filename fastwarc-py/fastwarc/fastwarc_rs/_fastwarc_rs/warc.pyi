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
from enum import IntFlag
from typing import BinaryIO, Callable, Dict, Iterable, Iterator, Literal, Optional, Self, Tuple, Union

from .stream_io import _GenericReader, _GenericWriter, Reader, Writer


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


class HeaderMap:
    reason_phrase: Optional[str]
    status_code: Optional[int]
    status_line: Optional[str]
    status_line_bytes: Optional[bytes]

    def __new__(cls, encoding: str = 'utf-8') -> Self: ...

    def encoding(self) -> str: ...

    def parse(self, reader: Union[Reader, BinaryIO, _GenericReader], has_status_line: bool = True) -> int: ...

    def write(self, writer: Union[Reader, BinaryIO, _GenericReader]) -> int: ...

    def append(self, key: str, value: str): ...

    def asdict(self) -> Dict[str, str]: ...

    def astuples(self) -> Tuple[Tuple[str, str], ...]: ...

    def is_empty(self) -> bool: ...

    def clear(self): ...

    def get(self, key: str, default: Optional[str] = None) -> Optional[str]: ...

    def get_multiple(self, key: str) -> Tuple[str, ...]: ...

    def get_bytes(self, key: bytes, default: Optional[bytes] = None) -> Optional[bytes]: ...

    def get_bytes_multiple(self, key: bytes) -> Tuple[bytes, ...]: ...

    def contains_key(self, key: str) -> bool: ...

    def contains_key_bytes(self, key: bytes) -> bool: ...

    def set(self, key: str, value: str): ...

    def set_bytes(self, key: bytes, value: bytes): ...

    def append_bytes(self, key: bytes, value: bytes): ...

    def remove(self, key: str): ...

    def remove_bytes(self, key: bytes): ...

    def items(self) -> Tuple[Tuple[str, str], ...]: ...

    def items_bytes(self) -> Tuple[Tuple[bytes, bytes], ...]: ...

    def keys(self) -> Tuple[str, ...]: ...

    def keys_bytes(self) -> Tuple[bytes, ...]: ...

    def values(self) -> Tuple[str, ...]: ...

    def values_bytes(self) -> Tuple[bytes, ...]: ...

    def __getitem__(self, item: str) -> str: ...

    def __iter__(self) -> Iterator[Tuple[str, str]]: ...

    def __len__(self) -> int: ...

    def __setitem__(self, key: str, value: str): ...

    def __contains__(self, item: str) -> bool: ...


class WarcRecordPayloadReaderPy(Reader):

    def readline(self, max_line_len: int = 8192) -> bytes: ...

    def consume(self, size: int = -1) -> int: ...


class WarcRecord:
    record_id: str
    record_type: WarcRecordType
    content_length: int
    record_date: Optional[datetime]
    headers: HeaderMap
    is_http: bool
    is_http_parsed: bool
    http_headers: Optional[HeaderMap]
    http_content_type: Optional[str]
    http_charset: Optional[str]
    http_date: Optional[datetime]
    http_last_modified: Optional[datetime]
    reader: WarcRecordPayloadReaderPy
    stream_pos: int
    is_frozen: bool

    def init_headers(
            self, content_length: int = 0, record_type: WarcRecordType = no_type, record_urn: Optional[bytes] = None
    ) -> None: ...

    def freeze(self) -> bool: ...

    def set_bytes_content(self, content: bytes) -> None: ...

    def set_bytes_payload(self, content: bytes) -> None: ...

    def consume(self, n: Optional[int] = None) -> int: ...

    def parse_warc_headers(self, quirks_mode: bool = False) -> int: ...

    def parse_http(self, strict_mode: bool = True,
                   auto_decode: Literal['none', 'content', 'transfer', 'all'] = 'none') -> None: ...

    def verify_block_digest(self, consume: bool = False) -> bool: ...

    def verify_payload_digest(self, consume: bool = False) -> bool: ...

    def write(
            self,
            stream: Union[Writer, BinaryIO, _GenericWriter],
            checksum_data: bool = False,
            payload_digest: Optional[bytes] = None,
            chunk_size: int = 16384
    ) -> int: ...


class ArchiveIterator(Iterable[WarcRecord]):
    def __new__(
            cls,
            stream: Union[Reader, BinaryIO, _GenericReader],
            record_types: WarcRecordType = any_type,
            parse_http: bool = True,
            min_content_length: int = -1,
            max_content_length: int = -1,
            func_filter: Optional[Callable[[WarcRecord], bool]] = None,
            verify_digests: bool = False,
            strict_mode: bool = True,
            auto_decode: Literal['none', 'content', 'transfer', 'all'] = 'none',
    ) -> Self: ...

    def __iter__(self) -> Iterator[WarcRecord]: ...

    def __next__(self) -> WarcRecord: ...
