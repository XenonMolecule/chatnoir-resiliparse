# Benchmark: FastWARC (Python)

Benchmark for [FastWARC](https://github.com/chatnoir-eu/chatnoir-resiliparse/) (Python bindings).

## Install Dependencies:

```bash
sudo apt install python3
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
36249 records/s, 996.8 MiB/s, 28.2 KiB/rec (18163 total, 499.4 MiB)
28853 records/s, 1266.5 MiB/s, 44.9 KiB/rec (32590 total, 1132.7 MiB)
30977 records/s, 1353.2 MiB/s, 44.7 KiB/rec (48172 total, 1813.4 MiB)
30852 records/s, 1367.0 MiB/s, 45.4 KiB/rec (63628 total, 2498.3 MiB)
27782 records/s, 1365.9 MiB/s, 50.3 KiB/rec (77569 total, 3183.7 MiB)
23658 records/s, 1355.5 MiB/s, 58.7 KiB/rec (89398 total, 3861.4 MiB)
24803 records/s, 1447.9 MiB/s, 59.8 KiB/rec (101803 total, 4585.6 MiB)
Summary: 4.0s, 28551 records/s, 1323.8 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
15789 records/s, 356.7 MiB/s, 23.1 KiB/rec (7897 total, 178.4 MiB)
13844 records/s, 371.1 MiB/s, 27.5 KiB/rec (14821 total, 364.0 MiB)
9452 records/s, 399.6 MiB/s, 43.3 KiB/rec (19552 total, 564.1 MiB)
9327 records/s, 401.2 MiB/s, 44.0 KiB/rec (24216 total, 764.7 MiB)
9687 records/s, 414.3 MiB/s, 43.8 KiB/rec (29077 total, 972.6 MiB)
9027 records/s, 411.4 MiB/s, 46.7 KiB/rec (33591 total, 1178.3 MiB)
9677 records/s, 409.8 MiB/s, 43.4 KiB/rec (38434 total, 1383.4 MiB)
9070 records/s, 407.4 MiB/s, 46.0 KiB/rec (42969 total, 1587.1 MiB)
9430 records/s, 410.0 MiB/s, 44.5 KiB/rec (47704 total, 1792.9 MiB)
8918 records/s, 417.2 MiB/s, 47.9 KiB/rec (52168 total, 2001.8 MiB)
9510 records/s, 409.8 MiB/s, 44.1 KiB/rec (56929 total, 2206.9 MiB)
9395 records/s, 396.8 MiB/s, 43.3 KiB/rec (61627 total, 2405.3 MiB)
9195 records/s, 415.3 MiB/s, 46.2 KiB/rec (66225 total, 2613.0 MiB)
9146 records/s, 415.4 MiB/s, 46.5 KiB/rec (70810 total, 2821.3 MiB)
8465 records/s, 429.1 MiB/s, 51.9 KiB/rec (75046 total, 3036.0 MiB)
7661 records/s, 435.1 MiB/s, 58.2 KiB/rec (78877 total, 3253.6 MiB)
7430 records/s, 430.7 MiB/s, 59.4 KiB/rec (82592 total, 3468.9 MiB)
7304 records/s, 424.0 MiB/s, 59.4 KiB/rec (86248 total, 3681.2 MiB)
7179 records/s, 416.0 MiB/s, 59.3 KiB/rec (89839 total, 3889.3 MiB)
6759 records/s, 408.3 MiB/s, 61.9 KiB/rec (93219 total, 4093.5 MiB)
6814 records/s, 429.0 MiB/s, 64.5 KiB/rec (96631 total, 4308.3 MiB)
7695 records/s, 409.0 MiB/s, 54.4 KiB/rec (100479 total, 4512.8 MiB)
7367 records/s, 407.4 MiB/s, 56.6 KiB/rec (104163 total, 4716.5 MiB)
7365 records/s, 419.6 MiB/s, 58.3 KiB/rec (107848 total, 4926.5 MiB)
6868 records/s, 438.9 MiB/s, 65.4 KiB/rec (111283 total, 5146.0 MiB)
Summary: 12.9s, 8876 records/s, 411.5 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
46565 records/s, 1451.5 MiB/s, 31.9 KiB/rec (23284 total, 725.8 MiB)
39014 records/s, 1707.8 MiB/s, 44.8 KiB/rec (42791 total, 1579.7 MiB)
40014 records/s, 1771.1 MiB/s, 45.3 KiB/rec (62799 total, 2465.2 MiB)
35681 records/s, 1784.8 MiB/s, 51.2 KiB/rec (80640 total, 3357.6 MiB)
32746 records/s, 1945.0 MiB/s, 60.8 KiB/rec (97015 total, 4330.3 MiB)
33426 records/s, 1893.4 MiB/s, 58.0 KiB/rec (113728 total, 5277.0 MiB)
Summary: 3.0s, 37971 records/s, 1760.5 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

## Results (RAM)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
80610 records/s, 2922.4 MiB/s, 37.1 KiB/rec (40306 total, 1461.2 MiB)
75337 records/s, 3486.9 MiB/s, 47.4 KiB/rec (77977 total, 3204.8 MiB)
71281 records/s, 4134.2 MiB/s, 59.4 KiB/rec (113618 total, 5271.9 MiB)
Summary: 1.5s, 75777 records/s, 3513.4 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
16175 records/s, 368.1 MiB/s, 23.3 KiB/rec (8088 total, 184.1 MiB)
13924 records/s, 379.2 MiB/s, 27.9 KiB/rec (15051 total, 373.7 MiB)
9449 records/s, 402.8 MiB/s, 43.7 KiB/rec (19777 total, 575.2 MiB)
9677 records/s, 416.0 MiB/s, 44.0 KiB/rec (24616 total, 783.2 MiB)
10257 records/s, 432.2 MiB/s, 43.1 KiB/rec (29745 total, 999.3 MiB)
9629 records/s, 429.4 MiB/s, 45.7 KiB/rec (34570 total, 1214.5 MiB)
9756 records/s, 426.7 MiB/s, 44.8 KiB/rec (39450 total, 1427.9 MiB)
9641 records/s, 428.3 MiB/s, 45.5 KiB/rec (44272 total, 1642.1 MiB)
9734 records/s, 429.5 MiB/s, 45.2 KiB/rec (49140 total, 1856.9 MiB)
9222 records/s, 430.2 MiB/s, 47.8 KiB/rec (53752 total, 2072.1 MiB)
9928 records/s, 425.7 MiB/s, 43.9 KiB/rec (58716 total, 2284.9 MiB)
9857 records/s, 429.3 MiB/s, 44.6 KiB/rec (63645 total, 2499.6 MiB)
9577 records/s, 424.9 MiB/s, 45.4 KiB/rec (68434 total, 2712.1 MiB)
9638 records/s, 434.7 MiB/s, 46.2 KiB/rec (73253 total, 2929.5 MiB)
7715 records/s, 458.6 MiB/s, 60.9 KiB/rec (77113 total, 3158.9 MiB)
7742 records/s, 439.3 MiB/s, 58.1 KiB/rec (80986 total, 3378.7 MiB)
7804 records/s, 458.7 MiB/s, 60.2 KiB/rec (84888 total, 3608.0 MiB)
8120 records/s, 454.7 MiB/s, 57.3 KiB/rec (88951 total, 3835.6 MiB)
7431 records/s, 452.6 MiB/s, 62.4 KiB/rec (92667 total, 4061.9 MiB)
7034 records/s, 441.3 MiB/s, 64.2 KiB/rec (96196 total, 4283.3 MiB)
8417 records/s, 452.5 MiB/s, 55.1 KiB/rec (100405 total, 4509.6 MiB)
7948 records/s, 435.0 MiB/s, 56.0 KiB/rec (104395 total, 4728.0 MiB)
7604 records/s, 446.7 MiB/s, 60.2 KiB/rec (108198 total, 4951.4 MiB)
7523 records/s, 459.0 MiB/s, 62.5 KiB/rec (111960 total, 5180.9 MiB)
Summary: 12.3s, 9313 records/s, 431.8 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
62576 records/s, 2142.8 MiB/s, 35.1 KiB/rec (31300 total, 1071.8 MiB)
49908 records/s, 2211.8 MiB/s, 45.4 KiB/rec (56260 total, 2178.0 MiB)
49430 records/s, 2397.5 MiB/s, 49.7 KiB/rec (80980 total, 3377.0 MiB)
44814 records/s, 2598.4 MiB/s, 59.4 KiB/rec (103387 total, 4676.2 MiB)
Summary: 2.3s, 50561 records/s, 2344.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```
