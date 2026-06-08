import sys
import time

from warcio.archiveiterator import ArchiveIterator


def main():
    if len(sys.argv) != 2:
        print(f'Usage: python {sys.argv[0]} WARCFILE')
        return

    path = sys.argv[1]
    start = time.monotonic()
    last_timer = start
    last_count = 0
    last_bytes = 0
    total_count = 0
    total_bytes = 0

    print(f'Reading WARC file: {path}')
    with open(path, 'rb', buffering=4096 << 10) as stream:
        for record in ArchiveIterator(stream, arc2warc=False):
            content_length = int(record.rec_headers.get_header('Content-Length') or 0)
            last_count += 1
            last_bytes += content_length
            total_count += 1
            total_bytes += content_length

            now = time.monotonic()
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

    print(f'Time elapsed: {time.monotonic() - start:.1f}s')


if __name__ == '__main__':
    main()
