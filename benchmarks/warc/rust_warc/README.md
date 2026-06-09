# Benchmark: rust_warc

Benchmark for [rust_warc](https://github.com/orottier/rust-warc).

**Note:** `rust_warc` supports compressed WARC files only through external readers. This benchmark uses `flate2` for
Gzip-compressed WARCs.

## Install Dependencies:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build the Benchmark

```bash
make
```

## Run the Benchmark

```bash
sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
./profile WARCFILE.warc
```

## Results (SSD)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with a Samsung 980PRO NVMe SSD (single-core
performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc
38132 records/s, 1085.7 MiB/s, 29.2 KiB/rec (19149 total, 545.2 MiB)
30696 records/s, 1321.9 MiB/s, 44.1 KiB/rec (34509 total, 1206.7 MiB)
28764 records/s, 1278.1 MiB/s, 45.5 KiB/rec (48891 total, 1845.7 MiB)
29839 records/s, 1320.4 MiB/s, 45.3 KiB/rec (63843 total, 2507.3 MiB)
27394 records/s, 1349.9 MiB/s, 50.5 KiB/rec (77541 total, 3182.3 MiB)
23176 records/s, 1326.6 MiB/s, 58.6 KiB/rec (89129 total, 3845.6 MiB)
24503 records/s, 1439.5 MiB/s, 60.2 KiB/rec (101391 total, 4565.9 MiB)
25033 records/s, 1430.9 MiB/s, 58.5 KiB/rec (113955 total, 5284.1 MiB)
Summary: 4.0s, 28473 records/s, 1320.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
17198 records/s, 392.4 MiB/s, 23.4 KiB/rec (8601 total, 196.2 MiB)
14612 records/s, 422.1 MiB/s, 29.6 KiB/rec (15909 total, 407.4 MiB)
10469 records/s, 446.7 MiB/s, 43.7 KiB/rec (21144 total, 630.7 MiB)
9802 records/s, 436.2 MiB/s, 45.6 KiB/rec (26046 total, 848.9 MiB)
10583 records/s, 447.7 MiB/s, 43.3 KiB/rec (31350 total, 1073.3 MiB)
10270 records/s, 451.9 MiB/s, 45.1 KiB/rec (36489 total, 1299.4 MiB)
10194 records/s, 453.7 MiB/s, 45.6 KiB/rec (41589 total, 1526.3 MiB)
10619 records/s, 463.6 MiB/s, 44.7 KiB/rec (46899 total, 1758.1 MiB)
9996 records/s, 463.0 MiB/s, 47.4 KiB/rec (51900 total, 1989.8 MiB)
10528 records/s, 453.7 MiB/s, 44.1 KiB/rec (57165 total, 2216.6 MiB)
10193 records/s, 444.3 MiB/s, 44.6 KiB/rec (62262 total, 2438.8 MiB)
10346 records/s, 461.5 MiB/s, 45.7 KiB/rec (67435 total, 2669.6 MiB)
10297 records/s, 455.1 MiB/s, 45.3 KiB/rec (72588 total, 2897.3 MiB)
8210 records/s, 476.0 MiB/s, 59.4 KiB/rec (76710 total, 3136.3 MiB)
8319 records/s, 465.7 MiB/s, 57.3 KiB/rec (80877 total, 3369.5 MiB)
8061 records/s, 479.5 MiB/s, 60.9 KiB/rec (84915 total, 3609.7 MiB)
8630 records/s, 485.6 MiB/s, 57.6 KiB/rec (89244 total, 3853.3 MiB)
7763 records/s, 472.7 MiB/s, 62.4 KiB/rec (93141 total, 4090.6 MiB)
7667 records/s, 473.0 MiB/s, 63.2 KiB/rec (96975 total, 4327.2 MiB)
8759 records/s, 473.2 MiB/s, 55.3 KiB/rec (101355 total, 4563.8 MiB)
8363 records/s, 451.0 MiB/s, 55.2 KiB/rec (105537 total, 4789.3 MiB)
7582 records/s, 474.5 MiB/s, 64.1 KiB/rec (109338 total, 5027.1 MiB)
8212 records/s, 472.3 MiB/s, 58.9 KiB/rec (113445 total, 5263.3 MiB)
Summary: 11.6s, 9865 records/s, 457.4 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

Unsupported.

## Results (RAM)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
88991 records/s, 3308.8 MiB/s, 38.1 KiB/rec (44496 total, 1654.4 MiB)
84129 records/s, 4085.5 MiB/s, 49.7 KiB/rec (86571 total, 3697.7 MiB)
Summary: 1.4s, 82891 records/s, 3843.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
18548 records/s, 424.7 MiB/s, 23.4 KiB/rec (9276 total, 212.4 MiB)
14604 records/s, 449.9 MiB/s, 31.5 KiB/rec (16584 total, 437.5 MiB)
11408 records/s, 480.4 MiB/s, 43.1 KiB/rec (22290 total, 677.8 MiB)
10640 records/s, 471.1 MiB/s, 45.3 KiB/rec (27612 total, 913.5 MiB)
10897 records/s, 475.7 MiB/s, 44.7 KiB/rec (33063 total, 1151.4 MiB)
11682 records/s, 499.8 MiB/s, 43.8 KiB/rec (38907 total, 1401.4 MiB)
11034 records/s, 497.8 MiB/s, 46.2 KiB/rec (44433 total, 1650.7 MiB)
11520 records/s, 511.5 MiB/s, 45.5 KiB/rec (50193 total, 1906.5 MiB)
11178 records/s, 501.1 MiB/s, 45.9 KiB/rec (55785 total, 2157.1 MiB)
11546 records/s, 488.8 MiB/s, 43.3 KiB/rec (61560 total, 2401.6 MiB)
10707 records/s, 491.3 MiB/s, 47.0 KiB/rec (66915 total, 2647.3 MiB)
10857 records/s, 476.8 MiB/s, 45.0 KiB/rec (72345 total, 2885.8 MiB)
9058 records/s, 515.3 MiB/s, 58.3 KiB/rec (76875 total, 3143.5 MiB)
8877 records/s, 500.6 MiB/s, 57.7 KiB/rec (81315 total, 3393.9 MiB)
8940 records/s, 523.5 MiB/s, 60.0 KiB/rec (85785 total, 3655.7 MiB)
8896 records/s, 505.6 MiB/s, 58.2 KiB/rec (90233 total, 3908.5 MiB)
8074 records/s, 494.1 MiB/s, 62.7 KiB/rec (94272 total, 4155.6 MiB)
8306 records/s, 496.0 MiB/s, 61.2 KiB/rec (98430 total, 4403.9 MiB)
8931 records/s, 487.7 MiB/s, 55.9 KiB/rec (102897 total, 4647.8 MiB)
8633 records/s, 477.5 MiB/s, 56.6 KiB/rec (107220 total, 4887.0 MiB)
7913 records/s, 501.4 MiB/s, 64.9 KiB/rec (111177 total, 5137.7 MiB)
Summary: 10.8s, 10558 records/s, 489.5 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

Unsupported.
