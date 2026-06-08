#include <algorithm>
#include <chrono>
#include <cstddef>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <variant>
#include <vector>
#include <warcpp/warcpp.hpp>

int main(int argc, char** argv)
{
    if (argc < 2) {
        std::cerr << "Usage: " << argv[0] << " WARCFILE" << std::endl;
        return 2;
    }

    auto const path = std::string(argv[1]);
    auto file_buffer = std::vector<char>(4096 << 10);
    auto file = std::ifstream();
    file.rdbuf()->pubsetbuf(file_buffer.data(),
        static_cast<std::streamsize>(file_buffer.size()));
    file.open(path, std::ios::binary);
    if (!file) {
        std::cerr << "Failed to open WARC file: " << path << '\n';
        return 1;
    }

    using clock = std::chrono::steady_clock;
    auto const start = clock::now();
    auto last_timer = start;
    std::size_t last_count = 0;
    std::size_t last_bytes = 0;
    std::size_t total_count = 0;
    std::size_t total_bytes = 0;

    std::cout << "Reading WARC file: " << path << '\n';
    while (!file.eof()) {
        auto result = warcpp::read_subsequent_record(file);
        if (!warcpp::holds_record(result)) {
            continue;
        }

        auto const& record = std::get<warcpp::Record>(result);
        auto const content_length = record.content_length();
        last_count += 1;
        last_bytes += content_length;
        total_count += 1;
        total_bytes += content_length;

        auto const now = clock::now();
        std::chrono::duration<double> const elapsed = now - last_timer;
        if (elapsed >= std::chrono::milliseconds(500)) {
            std::cout << std::fixed << std::setprecision(1) << std::setprecision(0)
                      << last_count / elapsed.count() << " records/s, "
                      << std::setprecision(1)
                      << last_bytes / elapsed.count() / 1024.0 / 1024.0 << " MiB/s, "
                      << last_bytes / static_cast<double>(std::max<std::size_t>(last_count, 1)) / 1024.0
                      << " KiB/rec (" << total_count << " total, "
                      << total_bytes / 1024.0 / 1024.0 << " MiB)"
                      << std::endl;
            last_count = 0;
            last_bytes = 0;
            last_timer = now;
        }
    }

    std::chrono::duration<double> const total_elapsed = clock::now() - start;
    std::cout << std::fixed << std::setprecision(1)
              << "Time elapsed: " << total_elapsed.count() << "s" << std::endl;

    return 0;
}
