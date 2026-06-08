import sys
import time

from fastwarc.warc import ArchiveIterator

BUFFER_SIZE = 4096 << 10


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

    print(f"Reading WARC file: {path}")
    for record in ArchiveIterator(path, parse_http=False, buffer_size=BUFFER_SIZE):
        content_length = record.content_length
        last_count += 1
        last_bytes += content_length
        total_count += 1
        total_bytes += content_length

        now = time.perf_counter()
        elapsed = now - last_timer
        if elapsed >= 0.5:
            print(
                f"{last_count / elapsed:.0f} records/s, "
                f"{last_bytes / elapsed / 1024 / 1024:.1f} MiB/s, "
                f"{last_bytes / max(last_count, 1) / 1024:.1f} KiB/rec "
                f"({total_count} total, "
                f"{total_bytes / 1024 / 1024:.1f} MiB)"
            )

            last_count = 0
            last_bytes = 0
            last_timer = now

    print(f"Time elapsed: {time.perf_counter() - start:.1f}s")


if __name__ == "__main__":
    main()
