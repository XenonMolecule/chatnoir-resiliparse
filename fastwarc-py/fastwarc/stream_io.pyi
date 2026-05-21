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


class Reader(ContextManager[Reader]):
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


class DecompressingReader(Reader):
    def inner_seek(self, offset: int, whence: int = 0) -> int: ...

    def inner_tell(self) -> int: ...

    def member_start_position(self) -> int: ...


class Writer(ContextManager[Writer]):
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


class CompressingWriter(Writer):
    def finish(self) -> None: ...


class GzipReader(DecompressingReader):
    def __new__(cls, inner: Union[Reader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, zlib=False, fsspec_args=None) -> Self: ...


class GzipWriter(CompressingWriter):
    def __new__(cls, inner: Union[Writer, BinaryIO, _GenericWriter, PathLike, str],
                compression_level=9, buffer_size=8192, zlib=False, fsspec_args=None) -> Self: ...


class ZstdReader(DecompressingReader):
    def __new__(cls, inner: Union[Writer, Reader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, fsspec_args=None, dictionary=None) -> Self: ...


class ZstdWriter(CompressingWriter):
    def __new__(cls, inner: Union[Writer, BinaryIO, _GenericWriter, PathLike, str],
                buffer_size=8192, fsspec_args=None, dictionary=None, compress_dictionary_frame=False) -> Self: ...


class Lz4Reader(DecompressingReader):
    def __new__(cls, inner: Union[Writer, Reader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, fsspec_args=None) -> Self: ...


class Lz4Writer(CompressingWriter):
    def __new__(cls, inner: Union[Writer, BinaryIO, _GenericWriter, PathLike, str],
                buffer_size=8192, fsspec_args=None) -> Self: ...


class BrotliReader(DecompressingReader):
    def __new__(cls, inner: Union[Writer, Reader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=4096, fsspec_args=None) -> Self: ...


class BrotliWriter(CompressingWriter):
    def __new__(cls, inner: Union[Writer, BinaryIO, _GenericWriter, PathLike, str],
                buffer_size=8192, fsspec_args=None) -> Self: ...


class ChunkedReader(DecompressingReader):
    def __new__(cls, inner: Union[Writer, Reader, BinaryIO, _GenericReader, PathLike, str],
                buffer_size=1024, fsspec_args=None) -> Self: ...


class ChunkedWriter(CompressingWriter):
    def __new__(cls, inner: Union[Writer, BinaryIO, _GenericWriter, PathLike, str],
                min_chunk_size=512, fsspec_args=None) -> Self: ...
