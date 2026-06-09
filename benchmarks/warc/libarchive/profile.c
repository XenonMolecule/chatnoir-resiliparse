#include <archive.h>
#include <archive_entry.h>

#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#define DEFAULT_BUFFER_SIZE (4096 << 10)

static size_t buffer_size(void)
{
    const char* value = getenv("BUFFER_SIZE");
    if (value == NULL || *value == '\0') {
        return DEFAULT_BUFFER_SIZE;
    }

    char* end = NULL;
    unsigned long long parsed = strtoull(value, &end, 10);
    if (end == value || *end != '\0' || parsed == 0) {
        return DEFAULT_BUFFER_SIZE;
    }

    return (size_t)parsed;
}

static double elapsed_seconds(struct timespec start, struct timespec end)
{
    return (double)(end.tv_sec - start.tv_sec) + (double)(end.tv_nsec - start.tv_nsec) / 1000000000.0;
}

int main(int argc, char** argv)
{
    if (argc != 2) {
        fprintf(stderr, "Usage: %s WARCFILE\n", argv[0]);
        return 2;
    }

    const char* path = argv[1];
    struct timespec start;
    struct timespec last_timer;
    clock_gettime(CLOCK_MONOTONIC, &start);
    last_timer = start;

    size_t last_count = 0;
    size_t last_bytes = 0;
    size_t total_count = 0;
    size_t total_bytes = 0;

    printf("Reading WARC file: %s\n", path);
    size_t buffer_size_value = buffer_size();

    struct archive* archive = archive_read_new();
    if (archive == NULL) {
        fprintf(stderr, "failed to allocate archive reader\n");
        return 1;
    }

    archive_read_support_filter_all(archive);
    archive_read_support_format_warc(archive);

    int result = archive_read_open_filename(archive, path, buffer_size_value);
    if (result != ARCHIVE_OK) {
        fprintf(stderr, "failed to open WARC file: %s\n", archive_error_string(archive));
        archive_read_free(archive);
        return 1;
    }

    struct archive_entry* entry = NULL;
    while ((result = archive_read_next_header(archive, &entry)) == ARCHIVE_OK) {
        la_int64_t entry_size = archive_entry_size(entry);
        size_t content_length = entry_size > 0 ? (size_t)entry_size : 0;

        result = archive_read_data_skip(archive);
        if (result != ARCHIVE_OK && result != ARCHIVE_WARN) {
            fprintf(stderr, "failed to skip entry data: %s\n", archive_error_string(archive));
            archive_read_free(archive);
            return 1;
        }

        last_count += 1;
        last_bytes += content_length;
        total_count += 1;
        total_bytes += content_length;

        struct timespec now;
        clock_gettime(CLOCK_MONOTONIC, &now);
        double elapsed = elapsed_seconds(last_timer, now);
        if (elapsed >= 0.5) {
            printf(
                "%.0f records/s, %.1f MiB/s, %.1f KiB/rec (%zu total, %.1f MiB)\n",
                (double)last_count / elapsed,
                (double)last_bytes / elapsed / 1024.0 / 1024.0,
                (double)last_bytes / (double)(last_count > 0 ? last_count : 1) / 1024.0,
                total_count,
                (double)total_bytes / 1024.0 / 1024.0);
            last_count = 0;
            last_bytes = 0;
            last_timer = now;
        }
    }

    if (result != ARCHIVE_EOF) {
        fprintf(stderr,
            "error while reading archive after %zu records at offset %lld: %s\n",
            total_count,
            (long long)archive_filter_bytes(archive, 0),
            archive_error_string(archive));
        archive_read_free(archive);
        return 1;
    }

    archive_read_free(archive);

    struct timespec end;
    clock_gettime(CLOCK_MONOTONIC, &end);
    double total_elapsed = elapsed_seconds(start, end);
    printf(
        "Summary: %.1fs, %.0f records/s, %.1f MiB/s, %.1f KiB/rec (%zu total, %.1f MiB)\n",
        total_elapsed,
        (double)total_count / total_elapsed,
        (double)total_bytes / total_elapsed / 1024.0 / 1024.0,
        (double)total_bytes / (double)(total_count > 0 ? total_count : 1) / 1024.0,
        total_count,
        (double)total_bytes / 1024.0 / 1024.0);

    return 0;
}
