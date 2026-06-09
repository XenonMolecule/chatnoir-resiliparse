# Benchmark: warc-rs

Benchmark for [warc-rs](https://github.com/slyrz/warc).

**Note:** `warc-rs` has built-in Gzip support. The constructor, however, does not allow specifying a custom buffer size.
This benchmark therefore uses `flate2` for a fairer comparison.

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
26769 records/s, 621.8 MiB/s, 23.8 KiB/rec (13389 total, 311.0 MiB)
18626 records/s, 768.3 MiB/s, 42.2 KiB/rec (22713 total, 695.6 MiB)
19023 records/s, 838.1 MiB/s, 45.1 KiB/rec (32232 total, 1115.0 MiB)
19451 records/s, 851.9 MiB/s, 44.9 KiB/rec (41958 total, 1541.0 MiB)
20409 records/s, 921.1 MiB/s, 46.2 KiB/rec (52167 total, 2001.8 MiB)
21004 records/s, 914.5 MiB/s, 44.6 KiB/rec (62670 total, 2459.1 MiB)
20378 records/s, 897.0 MiB/s, 45.1 KiB/rec (72859 total, 2907.5 MiB)
16598 records/s, 955.7 MiB/s, 59.0 KiB/rec (81159 total, 3385.4 MiB)
16328 records/s, 943.8 MiB/s, 59.2 KiB/rec (89325 total, 3857.4 MiB)
16023 records/s, 981.0 MiB/s, 62.7 KiB/rec (97338 total, 4348.0 MiB)
17182 records/s, 926.3 MiB/s, 55.2 KiB/rec (105929 total, 4811.2 MiB)
14398 records/s, 878.3 MiB/s, 62.5 KiB/rec (113130 total, 5250.4 MiB)
Summary: 6.1s, 18849 records/s, 874.0 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
13938 records/s, 306.7 MiB/s, 22.5 KiB/rec (6981 total, 153.6 MiB)
13585 records/s, 327.4 MiB/s, 24.7 KiB/rec (13776 total, 317.4 MiB)
8938 records/s, 371.3 MiB/s, 42.5 KiB/rec (18249 total, 503.2 MiB)
8603 records/s, 373.2 MiB/s, 44.4 KiB/rec (22551 total, 689.8 MiB)
8098 records/s, 361.5 MiB/s, 45.7 KiB/rec (26604 total, 870.7 MiB)
8694 records/s, 371.7 MiB/s, 43.8 KiB/rec (30954 total, 1056.7 MiB)
8688 records/s, 378.6 MiB/s, 44.6 KiB/rec (35298 total, 1246.0 MiB)
8552 records/s, 378.3 MiB/s, 45.3 KiB/rec (39582 total, 1435.5 MiB)
8574 records/s, 376.0 MiB/s, 44.9 KiB/rec (43869 total, 1623.5 MiB)
8762 records/s, 386.5 MiB/s, 45.2 KiB/rec (48252 total, 1816.8 MiB)
8197 records/s, 389.0 MiB/s, 48.6 KiB/rec (52353 total, 2011.4 MiB)
9137 records/s, 389.2 MiB/s, 43.6 KiB/rec (56925 total, 2206.2 MiB)
8918 records/s, 374.3 MiB/s, 43.0 KiB/rec (61386 total, 2393.4 MiB)
8605 records/s, 387.7 MiB/s, 46.1 KiB/rec (65700 total, 2587.8 MiB)
8468 records/s, 385.0 MiB/s, 46.6 KiB/rec (69936 total, 2780.4 MiB)
8076 records/s, 390.7 MiB/s, 49.5 KiB/rec (73974 total, 2975.8 MiB)
7121 records/s, 412.2 MiB/s, 59.3 KiB/rec (77541 total, 3182.3 MiB)
7004 records/s, 397.5 MiB/s, 58.1 KiB/rec (81048 total, 3381.3 MiB)
7114 records/s, 414.9 MiB/s, 59.7 KiB/rec (84606 total, 3588.8 MiB)
7323 records/s, 411.9 MiB/s, 57.6 KiB/rec (88269 total, 3794.9 MiB)
6746 records/s, 415.3 MiB/s, 63.0 KiB/rec (91665 total, 4003.9 MiB)
6683 records/s, 410.9 MiB/s, 63.0 KiB/rec (95007 total, 4209.4 MiB)
6952 records/s, 396.1 MiB/s, 58.3 KiB/rec (98490 total, 4407.9 MiB)
7516 records/s, 404.9 MiB/s, 55.2 KiB/rec (102252 total, 4610.5 MiB)
7243 records/s, 397.1 MiB/s, 56.1 KiB/rec (105879 total, 4809.3 MiB)
6601 records/s, 410.8 MiB/s, 63.7 KiB/rec (109182 total, 5014.9 MiB)
6803 records/s, 405.5 MiB/s, 61.0 KiB/rec (112596 total, 5218.4 MiB)
Summary: 13.7s, 8331 records/s, 386.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
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
40623 records/s, 1190.2 MiB/s, 30.0 KiB/rec (20313 total, 595.1 MiB)
31869 records/s, 1385.7 MiB/s, 44.5 KiB/rec (36252 total, 1288.2 MiB)
31365 records/s, 1406.5 MiB/s, 45.9 KiB/rec (51936 total, 1991.5 MiB)
32681 records/s, 1428.6 MiB/s, 44.8 KiB/rec (68277 total, 2705.8 MiB)
29250 records/s, 1573.8 MiB/s, 55.1 KiB/rec (82908 total, 3493.1 MiB)
26720 records/s, 1588.9 MiB/s, 60.9 KiB/rec (96270 total, 4287.6 MiB)
28547 records/s, 1614.3 MiB/s, 57.9 KiB/rec (110544 total, 5094.8 MiB)
Summary: 3.6s, 31513 records/s, 1461.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
14696 records/s, 325.0 MiB/s, 22.6 KiB/rec (7359 total, 162.7 MiB)
13840 records/s, 346.0 MiB/s, 25.6 KiB/rec (14279 total, 335.7 MiB)
8869 records/s, 381.6 MiB/s, 44.1 KiB/rec (18717 total, 526.7 MiB)
8804 records/s, 383.4 MiB/s, 44.6 KiB/rec (23121 total, 718.5 MiB)
9082 records/s, 397.2 MiB/s, 44.8 KiB/rec (27666 total, 917.3 MiB)
9013 records/s, 393.6 MiB/s, 44.7 KiB/rec (32175 total, 1114.2 MiB)
9129 records/s, 392.2 MiB/s, 44.0 KiB/rec (36741 total, 1310.3 MiB)
8764 records/s, 385.3 MiB/s, 45.0 KiB/rec (41124 total, 1503.0 MiB)
8620 records/s, 381.4 MiB/s, 45.3 KiB/rec (45434 total, 1693.8 MiB)
8771 records/s, 389.3 MiB/s, 45.5 KiB/rec (49821 total, 1888.5 MiB)
8549 records/s, 395.7 MiB/s, 47.4 KiB/rec (54099 total, 2086.5 MiB)
8822 records/s, 378.3 MiB/s, 43.9 KiB/rec (58512 total, 2275.7 MiB)
8615 records/s, 381.9 MiB/s, 45.4 KiB/rec (62820 total, 2466.7 MiB)
8712 records/s, 383.2 MiB/s, 45.0 KiB/rec (67176 total, 2658.3 MiB)
8589 records/s, 384.2 MiB/s, 45.8 KiB/rec (71472 total, 2850.5 MiB)
7569 records/s, 396.4 MiB/s, 53.6 KiB/rec (75258 total, 3048.8 MiB)
7401 records/s, 418.6 MiB/s, 57.9 KiB/rec (78960 total, 3258.1 MiB)
7217 records/s, 419.2 MiB/s, 59.5 KiB/rec (82572 total, 3468.0 MiB)
7268 records/s, 423.0 MiB/s, 59.6 KiB/rec (86208 total, 3679.6 MiB)
7179 records/s, 412.4 MiB/s, 58.8 KiB/rec (89799 total, 3885.9 MiB)
6826 records/s, 414.6 MiB/s, 62.2 KiB/rec (93213 total, 4093.3 MiB)
6497 records/s, 409.6 MiB/s, 64.6 KiB/rec (96462 total, 4298.1 MiB)
7608 records/s, 404.0 MiB/s, 54.4 KiB/rec (100275 total, 4500.6 MiB)
7291 records/s, 404.4 MiB/s, 56.8 KiB/rec (103923 total, 4702.9 MiB)
7349 records/s, 421.0 MiB/s, 58.7 KiB/rec (107601 total, 4913.6 MiB)
6754 records/s, 421.7 MiB/s, 63.9 KiB/rec (110979 total, 5124.6 MiB)
Summary: 13.4s, 8513 records/s, 394.7 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

Unsupported.
