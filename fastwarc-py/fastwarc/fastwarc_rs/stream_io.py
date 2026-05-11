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

from __future__ import annotations

from io import BytesIO
from typing import Any, Dict, Literal, Optional, Union

from ._fastwarc_rs.stream_io import GzipReader, GzipWriter


class FastWARCError(Exception):
    pass


class ReaderStaleError(FastWARCError):
    pass


class StreamError(FastWARCError):
    pass


class IOStream:
    def __enter__(self) -> "IOStream":
        return self

    def __exit__(self, exc_type, exc, traceback) -> None:
        self.close()

    def read(self, size: int) -> bytes:
        raise NotImplementedError

    def write(self, data: bytes) -> int:
        raise NotImplementedError

    def close(self) -> None:
        raise NotImplementedError

    def flush(self) -> None:
        raise NotImplementedError

    def seek(self, offset: int, whence: int = 0) -> int:
        raise NotImplementedError

    def tell(self) -> int:
        raise NotImplementedError


class PythonIOStreamAdapter(IOStream):
    def __init__(self, py_stream: Any) -> None:
        self.py_stream = py_stream

    def read(self, size: int) -> bytes:
        return self.py_stream.read(size)

    def write(self, data: bytes) -> int:
        return self.py_stream.write(data)

    def close(self) -> None:
        try:
            self.py_stream.close()
        except ValueError:
            pass

    def flush(self) -> None:
        self.py_stream.flush()

    def seek(self, offset: int, whence: int = 0) -> int:
        return self.py_stream.seek(offset, whence)

    def tell(self) -> int:
        return self.py_stream.tell()


class BytesIOStream(PythonIOStreamAdapter):
    def __init__(self, initial_data: Optional[bytes] = None) -> None:
        super().__init__(BytesIO(initial_data or b""))

    def getvalue(self) -> bytes:
        return self.py_stream.getvalue()


class FileStream(PythonIOStreamAdapter):
    def __init__(self, filename: str, mode: str = "rb") -> None:
        super().__init__(open(filename, mode))


def wrap_stream(
        raw_stream: Union[IOStream, Any, str],
        mode: str = "rb",
        fsspec_args: Optional[Union[Dict[Any, Any], Literal[False]]] = None,
) -> IOStream:
    if isinstance(raw_stream, IOStream):
        return raw_stream

    if isinstance(raw_stream, str):
        if fsspec_args is not False:
            try:
                import fsspec

                return PythonIOStreamAdapter(fsspec.open(raw_stream, mode, **(fsspec_args or {})).open())
            except ModuleNotFoundError:
                pass
        return FileStream(raw_stream, mode=mode)

    return PythonIOStreamAdapter(raw_stream)


class CompressingStream(IOStream):
    def begin_member(self) -> int:
        return 0

    def end_member(self) -> int:
        return 0


class GZipStream(CompressingStream):
    def __init__(
            self,
            raw_stream: Union[IOStream, Any, str],
            compression_level: int = 9,
            zlib: bool = False,
            fsspec_args: Optional[Union[Dict[Any, Any], Literal[False]]] = None,
    ) -> None:
        if zlib:
            raise NotImplementedError("zlib/raw deflate mode is not implemented in fastwarc_rs yet.")

        self.raw_stream = wrap_stream(raw_stream, mode="rb", fsspec_args=fsspec_args)
        self._compression_level = compression_level
        self._reader: Optional[GzipReader] = None
        self._writer: Optional[GzipWriter] = None

    def _ensure_reader(self) -> GzipReader:
        if self._writer is not None:
            raise StreamError("Compression in progress.")
        if self._reader is None:
            self._reader = GzipReader(self.raw_stream)
        return self._reader

    def _ensure_writer(self) -> GzipWriter:
        if self._reader is not None:
            raise StreamError("Decompression in progress")
        if self._writer is None:
            self._writer = GzipWriter(self.raw_stream, compression_level=self._compression_level)
        return self._writer

    def read(self, size: int) -> bytes:
        return self._ensure_reader().read(size)

    def write(self, data: bytes) -> int:
        return self._ensure_writer().write(data)

    def seek(self, offset: int, whence: int = 0) -> None:
        if self._writer is not None:
            raise StreamError("Compression in progress.")
        self.raw_stream.seek(offset, whence)
        if self._reader is not None:
            self._reader.close()
            self._reader = GzipReader(self.raw_stream)

    def tell(self) -> int:
        return self.raw_stream.tell()

    def begin_member(self) -> int:
        self._ensure_writer()
        return 0

    def end_member(self) -> int:
        if self._writer is None:
            return 0
        before = self.raw_stream.tell()
        self._writer.finish()
        self._writer.flush()
        return self.raw_stream.tell() - before

    def flush(self) -> None:
        if self._writer is not None:
            self._writer.flush()
        self.raw_stream.flush()

    def close(self) -> None:
        if self._reader is not None:
            self._reader.close()
            self._reader = None
            return

        if self._writer is not None:
            self._writer.close()
            self._writer = None
            return

        self.raw_stream.close()
