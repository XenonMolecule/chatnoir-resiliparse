# Benchmark: warcio.js

Benchmark for [warcio.js](https://github.com/webrecorder/warcio.js).

## Install Dependencies:

```bash
sudo apt install npm
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
13572 records/s, 295.0 MiB/s, 22.3 KiB/rec (6786 total, 147.5 MiB)
16995 records/s, 461.5 MiB/s, 27.8 KiB/rec (15284 total, 378.3 MiB)
14926 records/s, 623.7 MiB/s, 42.8 KiB/rec (22747 total, 690.1 MiB)
15797 records/s, 683.8 MiB/s, 44.3 KiB/rec (30690 total, 1034.0 MiB)
17003 records/s, 735.5 MiB/s, 44.3 KiB/rec (39192 total, 1401.7 MiB)
15074 records/s, 660.9 MiB/s, 44.9 KiB/rec (46729 total, 1732.2 MiB)
13660 records/s, 625.2 MiB/s, 46.9 KiB/rec (53559 total, 2044.8 MiB)
15822 records/s, 655.6 MiB/s, 42.4 KiB/rec (61500 total, 2373.9 MiB)
16799 records/s, 756.3 MiB/s, 46.1 KiB/rec (69900 total, 2752.1 MiB)
14002 records/s, 726.7 MiB/s, 53.1 KiB/rec (76901 total, 3115.4 MiB)
12677 records/s, 726.4 MiB/s, 58.7 KiB/rec (83240 total, 3478.6 MiB)
12834 records/s, 723.9 MiB/s, 57.8 KiB/rec (89657 total, 3840.5 MiB)
11327 records/s, 699.5 MiB/s, 63.2 KiB/rec (95346 total, 4191.9 MiB)
13340 records/s, 729.7 MiB/s, 56.0 KiB/rec (102016 total, 4556.7 MiB)
11228 records/s, 627.7 MiB/s, 57.3 KiB/rec (107630 total, 4870.6 MiB)
11704 records/s, 696.3 MiB/s, 60.9 KiB/rec (113482 total, 5218.8 MiB)
Summary: 8.0s, 14197 records/s, 652.5 MiB/s, 47.1 KiB/rec (114274 total, 5252.1 MiB)
```

### Gzip:

Unsupported.

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
20286 records/s, 451.0 MiB/s, 22.8 KiB/rec (10143 total, 225.5 MiB)
27894 records/s, 1053.8 MiB/s, 38.7 KiB/rec (24099 total, 752.7 MiB)
25973 records/s, 1119.2 MiB/s, 44.1 KiB/rec (37086 total, 1312.4 MiB)
21629 records/s, 946.0 MiB/s, 44.8 KiB/rec (47905 total, 1785.5 MiB)
22417 records/s, 982.5 MiB/s, 44.9 KiB/rec (59114 total, 2276.8 MiB)
28698 records/s, 1274.5 MiB/s, 45.5 KiB/rec (73463 total, 2914.1 MiB)
25882 records/s, 1482.6 MiB/s, 58.7 KiB/rec (86404 total, 3655.4 MiB)
26837 records/s, 1561.8 MiB/s, 59.6 KiB/rec (99823 total, 4436.3 MiB)
23185 records/s, 1344.6 MiB/s, 59.4 KiB/rec (111416 total, 5108.6 MiB)
Summary: 4.6s, 24789 records/s, 1139.3 MiB/s, 47.1 KiB/rec (114274 total, 5252.1 MiB)
```

### Gzip:

Unsupported.

### Zstandard:

Unsupported.

### LZ4:

Unsupported.
