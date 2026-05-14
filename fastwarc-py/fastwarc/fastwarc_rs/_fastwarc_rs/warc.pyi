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

from enum import IntFlag
from typing import BinaryIO, Dict, Iterator, Optional, Self, Tuple, Union

from fastwarc.fastwarc_rs._fastwarc_rs.stream_io import _GenericReader


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

    def __new__(cls, encoding: str = "utf-8") -> Self: ...

    def encoding(self) -> str: ...

    def parse(self, reader: Union[BinaryIO, _GenericReader], has_status_line: bool = True) -> int: ...

    def write(self, writer: Union[BinaryIO, _GenericReader]) -> int: ...

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
