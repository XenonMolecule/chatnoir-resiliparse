# FastWARC Benchmarks

This directory contains benchmarks of FastWARC and third-party WARC reading tools. The benchmarks are all designed in a
very similar fashion to allow for as much of an apples-to-apples comparison as possible.

The following benchmarks are available:

FastWARC:

- `fastwarc` (Rust)
- `fastwarc-py` (Python bindings)

Third-party:

- `gowarc` (Go) - [[repository](https://github.com/internetarchive/gowarc)]
- `jwarc` (Java) - [[repository](https://github.com/iipc/jwarc)]
- `libarchive` (C) - [[repository](https://github.com/libarchive/libarchive)]
- `node-warc` (JavaScript) - [[repository](https://github.com/N0taN3rd/node-warc)]
- `rust_warc` (Rust) - [[repository](https://github.com/orottier/rust-warc)]
- `slyrz_warc` (Go) - [[repository](https://github.com/slyrz/warc)]
- `warc-rs` (Rust) - [[repository](https://github.com/jedireza/warc)]
- `warcio` (Python) - [[repository](https://github.com/webrecorder/warcio)]
- `warcio.js` (TypeScript) - [[repository](https://github.com/webrecorder/warcio.js)]
- `warcpp` (C++) - [[repository](https://github.com/pisa-engine/warcpp)]
- `warcprotocol` (.NET) - [[repository](https://github.com/toimik/WarcProtocol)]

## Running the Benchmarks

All benchmarks come with a `Makefile` that produces a `./profile` executable. The executable takes the path to a WARC
file (uncompressed or compressed) and prints timing and throughput statistics.

All benchmarked parsers support uncompressed `.warc` files, and most support also compressed `.warc.gz` files. At the
moment, FastWARC and libarchive are the only parsers that also (fully) support `.warc.zst` and `.warc.lz4` files.

**IMPORTANT:** Before running a benchmark, you should drop the page cache with
`echo 3 | sudo tee /proc/sys/vm/drop_caches` for more realistic results.

Build a benchmark:

```console
$ cd fastwarc
$ make
cargo build --release
   Compiling libc v0.2.186
   Compiling shlex v2.0.1
   Compiling find-msvc-tools v0.1.9
...
```

Run a benchmark:

```
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc
40216 records/s, 1175.6 MiB/s, 29.9 KiB/rec (20143 total, 588.8 MiB)
37786 records/s, 1638.5 MiB/s, 44.4 KiB/rec (39046 total, 1408.5 MiB)
34980 records/s, 1562.6 MiB/s, 45.7 KiB/rec (56587 total, 2192.0 MiB)
34982 records/s, 1587.0 MiB/s, 46.5 KiB/rec (74122 total, 2987.5 MiB)
27994 records/s, 1596.1 MiB/s, 58.4 KiB/rec (88183 total, 3789.2 MiB)
26794 records/s, 1570.6 MiB/s, 60.0 KiB/rec (101584 total, 4574.8 MiB)
Summary: 3.4s, 33208 records/s, 1539.7 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)

```

Reading local WARC files benefits a lot from a large (but not too large) input buffer size. The optimum depends
completely on your SSD and CPU. By default, all benchmarks use an input buffer of 1 MiB. You can adjust the buffer size
by running the benchmarks with a custom `BUFFER_SIZE` environment variable:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ BUFFER_SIZE=$((768 << 10)) ./profile CC-MAIN-20231005012006-20231005042006-00899.warc  # 768 KiB
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc
38094 records/s, 1081.4 MiB/s, 29.1 KiB/rec (19057 total, 541.0 MiB)
28996 records/s, 1268.9 MiB/s, 44.8 KiB/rec (33556 total, 1175.5 MiB)
28979 records/s, 1265.3 MiB/s, 44.7 KiB/rec (48121 total, 1811.4 MiB)
29380 records/s, 1312.1 MiB/s, 45.7 KiB/rec (62830 total, 2468.3 MiB)
28245 records/s, 1359.4 MiB/s, 49.3 KiB/rec (77002 total, 3150.4 MiB)
24893 records/s, 1425.9 MiB/s, 58.7 KiB/rec (89470 total, 3864.6 MiB)
25105 records/s, 1466.3 MiB/s, 59.8 KiB/rec (102028 total, 4598.0 MiB)
Summary: 4.0s, 28883 records/s, 1339.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

To measure the raw parsing throughput without disk latency, you can read the WARC file directly from a tmpfs:

```console
$ mkdir tmpfs
$ sudo mount -t tmpfs none tmpfs
$ cp CC-MAIN-20231005012006-20231005042006-00899.warc tmpfs
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
148273 records/s, 5979.5 MiB/s, 41.3 KiB/rec (74155 total, 2990.5 MiB)
Summary: 0.8s, 137433 records/s, 6372.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

## Results

Following is a summary of the benchmarking results as a table. More details can be found in the individual subfolders.

The rows are sorted by best uncompressed throughput. Compressed times can vary greatly based on the compressor
implementation used. FastWARC generally has the fastest compressors and is only beaten by `libarchive`'s LZ4
implementation.

FastWARC and `libarchive` both have the overall fastest parsers, but FastWARC comes out on top when the WARC file is
read from a cold disk instead of from the page cache (or directly from RAM).

The speedup of `fastwarc-py` over `warcio` lies between 4.5x and 13x (uncompressed) or 3.8x and 4x (Gzip-compressed).

Keep in mind that in a real-world scenario, the raw parser throughput is usually not the deciding factor (at least not
if the parser is reasonably fast to begin with). Real-world performance is often dominated by disk or network latency
and decompressor throughput. The behaviour of a parser in how it reads bytes from a saturated disk and how it skips over
records also matters a lot.

### WARC Read From SSD

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with a Samsung 980PRO NVMe SSD (single-core
performance, read buffer size: 1 MiB).

| Parser         | Time (Uncompressed) | Time (Gzip) | Time (Zstd) | Time (LZ4) | MiB/s (Uncompressed) | MiB/s (Gzip) | MiB/s (Zstd) | MiB/s (LZ4) |
|----------------|--------------------:|------------:|------------:|-----------:|---------------------:|-------------:|-------------:|------------:|
| `fastwarc`     |                3.4s |        6.6s |        5.1s |       3.3s |               1539.7 |        797.0 |       1030.0 |      1586.9 |
| `fastwarc-py`  |                3.5s |        6.9s |        5.4s |       3.5s |               1502.0 |        763.7 |        980.5 |      1523.2 |
| `rust_warc`    |                4.0s |       11.6s |           - |          - |               1320.2 |        457.4 |            - |           - |
| `libarchive`   |                4.5s |       11.1s |       12.5s |       2.8s |               1174.5 |        478.2 |        423.9 |      1881.2 |
| `slyrz_warc`   |                5.1s |       23.0s |           - |          - |               1031.1 |        230.7 |            - |           - |
| `warcio.js`    |                5.6s |       43.8s |           - |          - |                952.0 |        119.8 |            - |           - |
| `gowarc`       |                6.0s |       35.1s |        9.1s |          - |                876.2 |        150.8 |        584.9 |           - |
| `warc-rs`      |                6.1s |       13.7s |           - |          - |                874.0 |        386.3 |            - |           - |
| `jwarc`        |                7.8s |       14.0s |          -* |          - |                681.4 |        379.7 |           -* |           - |
| `warcpp`       |                8.0s |        4.6s |           - |          - |                652.5 |       1139.3 |            - |           - |
| `nodewarc`     |               13.4s |       33.9s |           - |          - |                396.3 |        156.3 |            - |           - |
| `warcio`       |               15.7s |       25.9s |           - |          - |                338.2 |        204.9 |            - |           - |
| `warcprotocol` |               20.6s |      137.5s |           - |          - |                257.8 |         38.5 |            - |           - |

\* `jwarc` generally supports reading Zstd WARCs, but the iterator could not finish the test file.

### WARC Read From RAM

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB). This measures the raw parser speed, but is not a particularly
realistic benchmark.

| Parser         | Time (Uncompressed) | Time (Gzip) | Time (Zstd) | Time (LZ4) | MiB/s (Uncompressed) | MiB/s (Gzip) | MiB/s (Zstd) | MiB/s (LZ4) |
|----------------|--------------------:|------------:|------------:|-----------:|---------------------:|-------------:|-------------:|------------:|
| `fastwarc`     |                0.8s |        6.0s |        4.6s |       2.4s |               6372.2 |        879.6 |       1160.6 |      2184.6 |
| `libarchive`   |                0.8s |       10.5s |       11.7s |       1.8s |               6237.3 |        504.9 |        454.5 |      2943.9 |
| `fastwarc-py`  |                1.0s |        6.1s |        4.7s |       2.6s |               5256.8 |        870.6 |       1128.1 |      2072.1 |
| `rust_warc`    |                1.4s |       10.8s |           - |          - |               3843.3 |        489.5 |            - |           - |
| `slyrz_warc`   |                1.9s |       22.5s |           - |          - |               2860.6 |        235.6 |            - |           - |
| `gowarc`       |                2.0s |       34.3s |        8.2s |          - |               2659.3 |        154.3 |        644.1 |           - |
| `warcio.js`    |                2.1s |       43.8s |           - |          - |               2466.1 |        120.0 |            - |           - |
| `warc-rs`      |                3.6s |       13.4s |           - |          - |               1461.1 |        394.7 |            - |           - |
| `jwarc`        |                2.7s |       13.5s |          -* |          - |               1927.1 |        393.3 |           -* |           - |
| `warcpp`       |                4.6s |           - |           - |          - |               1139.3 |            - |            - |           - |
| `nodewarc`     |                9.7s |       34.4s |           - |          - |                546.5 |        154.0 |            - |           - |
| `warcio`       |               12.8s |       24.9s |           - |          - |                414.3 |        212.4 |            - |           - |
| `warcprotocol` |               18.2s |      141.2s |           - |          - |                291.6 |         37.5 |            - |           - |

\* `jwarc` generally supports reading Zstd WARCs, but the iterator could not finish the test file. 
