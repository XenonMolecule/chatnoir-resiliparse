import os
import sys
import time

from warcio.archiveiterator import ArchiveIterator

DEFAULT_BUFFER_SIZE = 4096 << 10


def buffer_size():
    try:
        value = int(os.environ.get('BUFFER_SIZE', ''))
    except ValueError:
        return DEFAULT_BUFFER_SIZE
    return value if value > 0 else DEFAULT_BUFFER_SIZE


def main():
    if len(sys.argv) != 2:
        print(f'Usage: python {sys.argv[0]} WARCFILE')
        return

    path = sys.argv[1]
    start = time.perf_counter()
    last_timer = start
    last_count = 0
    last_bytes = 0
    total_count = 0
    total_bytes = 0

    print(f'Reading WARC file: {path}')
    with open(path, 'rb', buffering=buffer_size()) as stream:
        for record in ArchiveIterator(stream):
            content_length = int(record.rec_headers.get_header('Content-Length') or 0)
            last_count += 1
            last_bytes += content_length
            total_count += 1
            total_bytes += content_length

            now = time.perf_counter()
            elapsed = now - last_timer
            if elapsed >= 0.5:
                print(
                    f'{last_count / elapsed:.0f} records/s, '
                    f'{last_bytes / elapsed / 1024.0 / 1024.0:.1f} MiB/s, '
                    f'{last_bytes / max(last_count, 1) / 1024.0:.1f} KiB/rec '
                    f'({total_count} total, {total_bytes / 1024.0 / 1024.0:.1f} MiB)'
                )
                last_count = 0
                last_bytes = 0
                last_timer = now

    total_elapsed = time.perf_counter() - start
    print(
        f'Summary: {total_elapsed:.1f}s, '
        f'{total_count / total_elapsed:.0f} records/s, '
        f'{total_bytes / total_elapsed / 1024.0 / 1024.0:.1f} MiB/s, '
        f'{total_bytes / max(total_count, 1) / 1024.0:.1f} KiB/rec '
        f'({total_count} total, {total_bytes / 1024.0 / 1024.0:.1f} MiB)'
    )


if __name__ == '__main__':
    main()
