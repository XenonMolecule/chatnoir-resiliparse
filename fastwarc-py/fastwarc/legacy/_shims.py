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

# Legacy shims for making the new Rust extension work with the old type names

__all__ = [
    'wrap_stream',
    'BufferedReader',
    'IOStream',
    'PythonIOStreamAdapter',
    'CompressingStream',
    'FileStream',
    'BytesIOStream',
    'GZipStream',
    'LZ4Stream',
    'BrotliStream',
    'FastWARCError',
    'StreamError',
]

FastWARCError = OSError

StreamError = OSError


def _is_writer_stream(raw_stream):
    mode = getattr(raw_stream, "mode", None)
    if isinstance(mode, str):
        if any(flag in mode for flag in ("w", "a", "x")):
            return True
        if "r" in mode:
            return False

    has_read = callable(getattr(raw_stream, "read", None))
    has_write = callable(getattr(raw_stream, "write", None))
    return has_write and not has_read


def wrap_stream(raw_stream, mode='rb', fsspec_args=None):
    if isinstance(raw_stream, str):
        if fsspec_args is not False and '://' in raw_stream:
            try:
                import fsspec
                return fsspec.open(raw_stream, mode, **(fsspec_args or {})).open()
            except ModuleNotFoundError:
                pass
        return FileStream(raw_stream, mode)

    return raw_stream


class BufferedReader:
    def __new__(cls, first, *args, **kwargs):
        return first


class IOStream:
    pass


class PythonIOStreamAdapter(IOStream):
    def __new__(cls, first, *args, **kwargs):
        return first


class CompressingStream(IOStream):
    pass


class FileStream(IOStream):
    def __new__(cls, filename, mode='rb'):
        if 'b' not in mode:
            mode += 'b'
        return open(filename, mode)


class BytesIOStream(IOStream):
    def __new__(cls, initial_data):
        import io
        return io.BytesIO(initial_data)


class GZipStream(CompressingStream):
    def __new__(cls, raw_stream, mode='r', compression_level=9, zlib=False, fsspec_args=None):
        from fastwarc.stream_io import GzipReader, GzipWriter

        is_writer = any(flag in mode for flag in ('w', 'a', 'x'))
        if isinstance(raw_stream, str):
            if is_writer:
                return GzipWriter(
                    raw_stream,
                    compression_level=compression_level,
                    zlib=zlib,
                    fsspec_args=fsspec_args,
                )
            return GzipReader(raw_stream, zlib=zlib, fsspec_args=fsspec_args)

        if is_writer or _is_writer_stream(raw_stream):
            return GzipWriter(
                raw_stream,
                compression_level=compression_level,
                zlib=zlib,
                fsspec_args=fsspec_args,
            )
        return GzipReader(raw_stream, zlib=zlib, fsspec_args=fsspec_args)


class LZ4Stream(CompressingStream):
    def __new__(cls, raw_stream, mode='r', compression_level=12, favor_dec_speed=True, fsspec_args=None):
        from fastwarc.stream_io import Lz4Reader, Lz4Writer

        is_writer = any(flag in mode for flag in ('w', 'a', 'x'))
        if isinstance(raw_stream, str):
            if is_writer:
                return Lz4Writer(raw_stream, fsspec_args=fsspec_args)
            return Lz4Reader(raw_stream, fsspec_args=fsspec_args)

        if is_writer or _is_writer_stream(raw_stream):
            return Lz4Writer(raw_stream, fsspec_args=fsspec_args)
        return Lz4Reader(raw_stream, fsspec_args=fsspec_args)


class BrotliStream(CompressingStream):
    def __new__(cls, raw_stream, mode='r', fsspec_args=None):
        from fastwarc.stream_io import BrotliReader, BrotliWriter

        is_writer = any(flag in mode for flag in ('w', 'a', 'x'))

        if isinstance(raw_stream, str):
            if is_writer:
                return BrotliWriter(raw_stream, fsspec_args=fsspec_args)
            return BrotliReader(raw_stream, fsspec_args=fsspec_args)

        if is_writer or _is_writer_stream(raw_stream):
            return BrotliWriter(raw_stream, fsspec_args=fsspec_args)
        return BrotliReader(raw_stream, fsspec_args=fsspec_args)
