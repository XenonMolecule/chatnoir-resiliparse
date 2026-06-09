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
moment, FastWARC is the only parser that also supports `.warc.zst` and `.warc.lz4` files.

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
38395 records/s, 1095.3 MiB/s, 29.2 KiB/rec (19249 total, 549.1 MiB)
27857 records/s, 1215.3 MiB/s, 44.7 KiB/rec (33223 total, 1158.7 MiB)
27041 records/s, 1185.2 MiB/s, 44.9 KiB/rec (46774 total, 1752.7 MiB)
26963 records/s, 1184.5 MiB/s, 45.0 KiB/rec (60283 total, 2346.2 MiB)
26216 records/s, 1186.8 MiB/s, 46.4 KiB/rec (73402 total, 2940.1 MiB)
23962 records/s, 1380.2 MiB/s, 59.0 KiB/rec (85387 total, 3630.4 MiB)
24310 records/s, 1455.6 MiB/s, 61.3 KiB/rec (97585 total, 4360.8 MiB)
26023 records/s, 1470.9 MiB/s, 57.9 KiB/rec (110638 total, 5098.6 MiB)
Summary: 4.1s, 27643 records/s, 1281.7 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)

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
142470 records/s, 5679.6 MiB/s, 40.8 KiB/rec (71239 total, 2839.9 MiB)
Summary: 0.9s, 130491 records/s, 6050.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```
