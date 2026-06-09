# Benchmark: FastWARC

Benchmark for [FastWARC](https://github.com/chatnoir-eu/chatnoir-resiliparse/) (Rust).

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
echo 3 | sudo tee /proc/sys/vm/drop_caches
./profile WARCFILE.warc
```

## Results (SSD)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with a Samsung 980PRO NVMe SSD (single-core
performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc
41941 records/s, 1248.2 MiB/s, 30.5 KiB/rec (20971 total, 624.1 MiB)
30965 records/s, 1347.8 MiB/s, 44.6 KiB/rec (36454 total, 1298.0 MiB)
33014 records/s, 1477.2 MiB/s, 45.8 KiB/rec (52978 total, 2037.4 MiB)
36470 records/s, 1603.1 MiB/s, 45.0 KiB/rec (71257 total, 2840.9 MiB)
28781 records/s, 1617.0 MiB/s, 57.5 KiB/rec (85648 total, 3649.4 MiB)
26010 records/s, 1534.9 MiB/s, 60.4 KiB/rec (98707 total, 4420.1 MiB)
28122 records/s, 1623.4 MiB/s, 59.1 KiB/rec (112807 total, 5234.0 MiB)
Summary: 3.5s, 32210 records/s, 1493.4 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
27509 records/s, 634.0 MiB/s, 23.6 KiB/rec (13758 total, 317.1 MiB)
18001 records/s, 765.3 MiB/s, 43.5 KiB/rec (22797 total, 701.3 MiB)
17315 records/s, 754.1 MiB/s, 44.6 KiB/rec (31494 total, 1080.2 MiB)
17904 records/s, 775.7 MiB/s, 44.4 KiB/rec (40449 total, 1468.1 MiB)
18292 records/s, 820.8 MiB/s, 45.9 KiB/rec (49596 total, 1878.6 MiB)
17889 records/s, 797.5 MiB/s, 45.7 KiB/rec (58563 total, 2278.3 MiB)
18201 records/s, 802.8 MiB/s, 45.2 KiB/rec (67698 total, 2681.2 MiB)
16735 records/s, 825.1 MiB/s, 50.5 KiB/rec (76071 total, 3094.1 MiB)
14252 records/s, 830.1 MiB/s, 59.6 KiB/rec (83238 total, 3511.5 MiB)
14864 records/s, 846.0 MiB/s, 58.3 KiB/rec (90672 total, 3934.7 MiB)
13618 records/s, 843.4 MiB/s, 63.4 KiB/rec (97482 total, 4356.4 MiB)
15298 records/s, 823.0 MiB/s, 55.1 KiB/rec (105132 total, 4768.0 MiB)
13847 records/s, 833.7 MiB/s, 61.7 KiB/rec (112056 total, 5184.8 MiB)
Summary: 6.6s, 17190 records/s, 797.0 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
33969 records/s, 902.4 MiB/s, 27.2 KiB/rec (16990 total, 451.4 MiB)
22451 records/s, 976.1 MiB/s, 44.5 KiB/rec (28267 total, 941.6 MiB)
22185 records/s, 964.8 MiB/s, 44.5 KiB/rec (39361 total, 1424.1 MiB)
23169 records/s, 1045.6 MiB/s, 46.2 KiB/rec (50947 total, 1947.0 MiB)
23797 records/s, 1044.9 MiB/s, 45.0 KiB/rec (62847 total, 2469.5 MiB)
22488 records/s, 1030.8 MiB/s, 46.9 KiB/rec (74091 total, 2984.9 MiB)
19270 records/s, 1104.5 MiB/s, 58.7 KiB/rec (83727 total, 3537.1 MiB)
18971 records/s, 1112.1 MiB/s, 60.0 KiB/rec (93213 total, 4093.3 MiB)
17954 records/s, 1029.1 MiB/s, 58.7 KiB/rec (102199 total, 4608.3 MiB)
18126 records/s, 1071.5 MiB/s, 60.5 KiB/rec (111262 total, 5144.1 MiB)
Summary: 5.1s, 22215 records/s, 1030.0 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### LZ4:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
43748 records/s, 1317.9 MiB/s, 30.8 KiB/rec (21897 total, 659.7 MiB)
34951 records/s, 1530.7 MiB/s, 44.8 KiB/rec (39373 total, 1425.0 MiB)
35895 records/s, 1597.8 MiB/s, 45.6 KiB/rec (57324 total, 2224.1 MiB)
34624 records/s, 1581.6 MiB/s, 46.8 KiB/rec (74638 total, 3015.0 MiB)
30554 records/s, 1757.1 MiB/s, 58.9 KiB/rec (89920 total, 3893.8 MiB)
28055 records/s, 1620.3 MiB/s, 59.1 KiB/rec (103951 total, 4704.1 MiB)
Summary: 3.3s, 34226 records/s, 1586.9 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

## Results (RAM)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
142470 records/s, 5679.6 MiB/s, 40.8 KiB/rec (71239 total, 2839.9 MiB)
Summary: 0.9s, 130491 records/s, 6050.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
30160 records/s, 749.5 MiB/s, 25.4 KiB/rec (15081 total, 374.8 MiB)
19434 records/s, 831.6 MiB/s, 43.8 KiB/rec (24798 total, 790.6 MiB)
20176 records/s, 877.0 MiB/s, 44.5 KiB/rec (34886 total, 1229.1 MiB)
19892 records/s, 880.1 MiB/s, 45.3 KiB/rec (44835 total, 1669.2 MiB)
19752 records/s, 886.6 MiB/s, 46.0 KiB/rec (54711 total, 2112.5 MiB)
19738 records/s, 855.0 MiB/s, 44.4 KiB/rec (64581 total, 2540.1 MiB)
19103 records/s, 898.9 MiB/s, 48.2 KiB/rec (74142 total, 2990.0 MiB)
16366 records/s, 915.1 MiB/s, 57.3 KiB/rec (82329 total, 3447.7 MiB)
16173 records/s, 940.5 MiB/s, 59.5 KiB/rec (90420 total, 3918.2 MiB)
14388 records/s, 888.4 MiB/s, 63.2 KiB/rec (97614 total, 4362.4 MiB)
16600 records/s, 896.0 MiB/s, 55.3 KiB/rec (105915 total, 4810.5 MiB)
15759 records/s, 936.7 MiB/s, 60.9 KiB/rec (113796 total, 5278.9 MiB)
Summary: 6.0s, 18971 records/s, 879.6 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.zst
36232 records/s, 995.5 MiB/s, 28.1 KiB/rec (18117 total, 497.8 MiB)
25741 records/s, 1123.2 MiB/s, 44.7 KiB/rec (30991 total, 1059.5 MiB)
26888 records/s, 1182.6 MiB/s, 45.0 KiB/rec (44436 total, 1650.9 MiB)
26677 records/s, 1192.4 MiB/s, 45.8 KiB/rec (57784 total, 2247.5 MiB)
25759 records/s, 1136.2 MiB/s, 45.2 KiB/rec (70669 total, 2815.8 MiB)
22172 records/s, 1197.9 MiB/s, 55.3 KiB/rec (81760 total, 3415.0 MiB)
20526 records/s, 1215.4 MiB/s, 60.6 KiB/rec (92029 total, 4023.1 MiB)
20708 records/s, 1193.8 MiB/s, 59.0 KiB/rec (102384 total, 4620.0 MiB)
20639 records/s, 1210.9 MiB/s, 60.1 KiB/rec (112704 total, 5225.5 MiB)
Summary: 4.6s, 25031 records/s, 1160.6 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### LZ4:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
60780 records/s, 2059.5 MiB/s, 34.7 KiB/rec (30390 total, 1029.8 MiB)
49757 records/s, 2209.5 MiB/s, 45.5 KiB/rec (55270 total, 2134.6 MiB)
46191 records/s, 2178.5 MiB/s, 48.3 KiB/rec (78367 total, 3223.9 MiB)
38631 records/s, 2281.0 MiB/s, 60.5 KiB/rec (97683 total, 4364.4 MiB)
Summary: 2.4s, 47117 records/s, 2184.6 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```
