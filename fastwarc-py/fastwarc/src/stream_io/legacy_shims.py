# from fastwarc.stream_io import *


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
        is_writer = any(flag in mode for flag in ('w', 'a', 'x'))
        if isinstance(raw_stream, str):
            if is_writer:
                return BrotliWriter(raw_stream, fsspec_args=fsspec_args)
            return BrotliReader(raw_stream, fsspec_args=fsspec_args)
        if is_writer or _is_writer_stream(raw_stream):
            return BrotliWriter(raw_stream, fsspec_args=fsspec_args)
        return BrotliReader(raw_stream, fsspec_args=fsspec_args)
