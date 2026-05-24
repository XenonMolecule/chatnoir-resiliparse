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

from os import PathLike
from types import TracebackType
from typing import BinaryIO, ContextManager, Optional, Protocol, Type, Self, Union

# Legacy shims
from fastwarc.legacy.stream_io import *


class _GenericReader(Protocol):
    def read(self, size: int = -1) -> bytes: ...

    def seek(self, offset: int, whence: int = 0) -> int: ...

    def tell(self) -> int: ...

    def close(self) -> None: ...


class _GenericWriter(Protocol):
    def write(self, data: bytes) -> int: ...

    def flush(self) -> None: ...

    def close(self) -> None: ...


class WarcReader(ContextManager[WarcReader]):
    def __new__(cls) -> Self: ...

    def read(self, size: int = -1) -> bytes: ...

    def seek(self, offset: int, whence: int = 0) -> int: ...

    def tell(self) -> int: ...

    def close(self) -> None: ...

    def __enter__(self) -> Self: ...

    def __exit__(
            self,
            exc_type: Optional[Type[BaseException]],
            exc: Optional[BaseException],
            traceback: Optional[TracebackType]
    ) -> None: ...

    def inner_seek(self, offset: int, whence: int = 0) -> int: ...

    def inner_tell(self) -> int: ...

    def frame_start_position(self) -> Optional[int]: ...


class WarcWriter(ContextManager[WarcWriter]):
    def __new__(cls) -> Self: ...

    def write(self, data: bytes) -> int: ...

    def flush(self) -> None: ...

    def close(self) -> None: ...

    def __enter__(self) -> Self: ...

    def __exit__(
            self,
            exc_type: Optional[Type[BaseException]],
            exc: Optional[BaseException],
            traceback: Optional[TracebackType]
    ) -> None: ...

    def finish(self) -> None: ...


class GzipReader(WarcReader):
    def __new__(cls, inner: Union[WarcReader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, zlib=False, fsspec_args=None) -> Self: ...


class GzipWriter(WarcWriter):
    def __new__(cls, inner: Union[WarcWriter, BinaryIO, _GenericWriter, PathLike, str],
                compression_level=9, buffer_size=8192, zlib=False, fsspec_args=None) -> Self: ...


class ZstdReader(WarcReader):
    def __new__(cls, inner: Union[WarcReader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, fsspec_args=None, dictionary=None) -> Self: ...


class ZstdWriter(WarcWriter):
    def __new__(cls, inner: Union[WarcWriter, BinaryIO, _GenericWriter, PathLike, str],
                buffer_size=8192, fsspec_args=None, dictionary=None, compress_dictionary_frame=False) -> Self: ...


class Lz4Reader(WarcReader):
    def __new__(cls, inner: Union[WarcReader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, fsspec_args=None) -> Self: ...


class Lz4Writer(WarcWriter):
    def __new__(cls, inner: Union[WarcWriter, BinaryIO, _GenericWriter, PathLike, str],
                buffer_size=8192, fsspec_args=None) -> Self: ...


class BrotliReader(WarcReader):
    def __new__(cls, inner: Union[WarcReader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, fsspec_args=None) -> Self: ...


class BrotliWriter(WarcWriter):
    def __new__(cls, inner: Union[WarcWriter, BinaryIO, _GenericWriter, PathLike, str],
                buffer_size=8192, fsspec_args=None) -> Self: ...


class ChunkedReader(WarcReader):
    def __new__(cls, inner: Union[WarcReader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=1024, fsspec_args=None) -> Self: ...


class ChunkedWriter(WarcWriter):
    def __new__(cls, inner: Union[WarcWriter, BinaryIO, _GenericWriter, PathLike, str],
                min_chunk_size=512, fsspec_args=None) -> Self: ...
