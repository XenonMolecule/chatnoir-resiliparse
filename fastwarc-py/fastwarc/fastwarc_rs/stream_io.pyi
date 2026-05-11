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

from types import TracebackType
from typing import Any, ContextManager, Dict, Literal, Optional, Type, Union, BinaryIO, Protocol


class _GenericIOStream(Protocol):
    def write(self, data: bytes) -> int: ...

    def flush(self) -> None: ...

    def read(self, size: int) -> bytes: ...

    def seek(self, offset: int) -> int: ...

    def close(self) -> None: ...

    def tell(self) -> int: ...


class IOStream(ContextManager[IOStream]):
    def read(self, size: int) -> bytes: ...

    def write(self, data: bytes) -> int: ...

    def close(self) -> None: ...

    def flush(self) -> None: ...

    def seek(self, offset: int) -> None: ...

    def tell(self) -> int: ...

    def __enter__(self) -> IOStream: ...

    def __exit__(
            self,
            exc_type: Optional[Type[BaseException]],
            exc: Optional[BaseException],
            traceback: Optional[TracebackType]
    ) -> None: ...


def wrap_stream(
        raw_stream: Union[IOStream, BinaryIO, _GenericIOStream, str],
        mode: str = 'rb',
        fsspec_args: Optional[Union[Dict[Any, Any], Literal[False]]] = None
) -> IOStream: ...


class BufferedReader:
    def __init__(
            self, stream: IOStream, buf_size: int = 65536, negotiate_stream: bool = True
    ) -> None: ...

    def close(self) -> None: ...

    def consume(self, size: int = -1) -> int: ...

    def read(self, size: int = -1) -> bytes: ...

    def readline(self, crlf: bool = True, max_line_len: int = 8192) -> bytes: ...

    def tell(self) -> int: ...


class BytesIOStream(IOStream):
    def __init__(self, initial_data: Union[bytes, None] = None) -> None: ...

    def getvalue(self) -> bytes: ...


class FileStream(IOStream):
    def __init__(self, filename: str, mode: str = 'rb') -> None: ...


class CompressingStream(IOStream):
    def begin_member(self) -> int: ...

    def end_member(self) -> int: ...


class BrotliStream(CompressingStream):
    def __init__(
            self,
            raw_stream: Union[IOStream, BinaryIO, _GenericIOStream, str],
            quality: int = 11,
            lgwin: int = 22,
            lgblock: int = 0,
            fsspec_args: Optional[Union[Dict[Any, Any], Literal[False]]] = None
    ) -> None: ...


class GZipStream(CompressingStream):
    def __init__(
            self, raw_stream: Union[IOStream, BinaryIO, _GenericIOStream, str],
            compression_level: int = 9,
            zlib: bool = False,
            fsspec_args: Optional[Union[Dict[Any, Any], Literal[False]]] = None
    ) -> None: ...


class LZ4Stream(CompressingStream):
    def __init__(
            self,
            raw_stream: Union[IOStream, BinaryIO, _GenericIOStream, str],
            compression_level: int = 12,
            favor_dec_speed: bool = True,
            fsspec_args: Optional[Union[Dict[Any, Any], Literal[False]]] = None
    ) -> None: ...

    def prepopulate(self, initial_data: bytes) -> None: ...


class PythonIOStreamAdapter(IOStream):
    def __init__(self, py_stream: _GenericIOStream) -> None: ...


class FastWARCError(Exception):
    pass


class ReaderStaleError(FastWARCError):
    pass


class StreamError(FastWARCError):
    pass
