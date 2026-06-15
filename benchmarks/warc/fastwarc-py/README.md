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
40636 records/s, 1190.8 MiB/s, 30.0 KiB/rec (20359 total, 596.6 MiB)
31766 records/s, 1382.2 MiB/s, 44.6 KiB/rec (36242 total, 1287.7 MiB)
32784 records/s, 1471.7 MiB/s, 46.0 KiB/rec (52681 total, 2025.7 MiB)
34016 records/s, 1485.7 MiB/s, 44.7 KiB/rec (69691 total, 2768.6 MiB)
28321 records/s, 1555.3 MiB/s, 56.2 KiB/rec (83866 total, 3547.0 MiB)
26753 records/s, 1590.7 MiB/s, 60.9 KiB/rec (97285 total, 4344.9 MiB)
27459 records/s, 1561.6 MiB/s, 58.2 KiB/rec (111019 total, 5126.0 MiB)
Summary: 3.6s, 31567 records/s, 1463.6 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
26237 records/s, 610.1 MiB/s, 23.8 KiB/rec (13164 total, 306.1 MiB)
18449 records/s, 753.1 MiB/s, 41.8 KiB/rec (22389 total, 682.7 MiB)
17145 records/s, 749.7 MiB/s, 44.8 KiB/rec (30969 total, 1057.9 MiB)
17654 records/s, 769.9 MiB/s, 44.7 KiB/rec (39796 total, 1442.8 MiB)
17850 records/s, 793.8 MiB/s, 45.5 KiB/rec (48721 total, 1839.7 MiB)
17107 records/s, 764.1 MiB/s, 45.7 KiB/rec (57309 total, 2223.3 MiB)
17544 records/s, 766.1 MiB/s, 44.7 KiB/rec (66087 total, 2606.7 MiB)
17124 records/s, 819.7 MiB/s, 49.0 KiB/rec (74652 total, 3016.7 MiB)
14760 records/s, 832.3 MiB/s, 57.7 KiB/rec (82032 total, 3432.8 MiB)
13931 records/s, 810.5 MiB/s, 59.6 KiB/rec (88998 total, 3838.1 MiB)
13295 records/s, 822.1 MiB/s, 63.3 KiB/rec (95652 total, 4249.5 MiB)
15326 records/s, 843.7 MiB/s, 56.4 KiB/rec (103315 total, 4671.4 MiB)
13613 records/s, 806.8 MiB/s, 60.7 KiB/rec (110130 total, 5075.3 MiB)
Summary: 6.8s, 16889 records/s, 783.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
32512 records/s, 842.7 MiB/s, 26.5 KiB/rec (16258 total, 421.4 MiB)
22364 records/s, 971.2 MiB/s, 44.5 KiB/rec (27441 total, 907.0 MiB)
22959 records/s, 990.7 MiB/s, 44.2 KiB/rec (38932 total, 1402.9 MiB)
21576 records/s, 961.6 MiB/s, 45.6 KiB/rec (49740 total, 1884.6 MiB)
22688 records/s, 987.8 MiB/s, 44.6 KiB/rec (61087 total, 2378.6 MiB)
21477 records/s, 971.6 MiB/s, 46.3 KiB/rec (71826 total, 2864.4 MiB)
18700 records/s, 1044.9 MiB/s, 57.2 KiB/rec (81178 total, 3387.0 MiB)
17333 records/s, 1006.1 MiB/s, 59.4 KiB/rec (89851 total, 3890.4 MiB)
16857 records/s, 1011.2 MiB/s, 61.4 KiB/rec (98284 total, 4396.3 MiB)
18321 records/s, 1011.8 MiB/s, 56.6 KiB/rec (107449 total, 4902.5 MiB)
Summary: 5.4s, 21248 records/s, 985.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### LZ4:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
43163 records/s, 1293.4 MiB/s, 30.7 KiB/rec (21603 total, 647.4 MiB)
30527 records/s, 1340.0 MiB/s, 44.9 KiB/rec (36867 total, 1317.4 MiB)
34237 records/s, 1531.0 MiB/s, 45.8 KiB/rec (53986 total, 2082.9 MiB)
34186 records/s, 1500.3 MiB/s, 44.9 KiB/rec (71083 total, 2833.3 MiB)
29301 records/s, 1640.6 MiB/s, 57.3 KiB/rec (85735 total, 3653.7 MiB)
28044 records/s, 1636.7 MiB/s, 59.8 KiB/rec (99760 total, 4472.2 MiB)
Summary: 3.5s, 32736 records/s, 1517.8 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

## Results (RAM)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
129280 records/s, 5086.1 MiB/s, 40.3 KiB/rec (64648 total, 2543.3 MiB)
Summary: 0.9s, 122207 records/s, 5666.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
30078 records/s, 746.0 MiB/s, 25.4 KiB/rec (15045 total, 373.1 MiB)
20292 records/s, 873.6 MiB/s, 44.1 KiB/rec (25197 total, 810.2 MiB)
20492 records/s, 887.8 MiB/s, 44.4 KiB/rec (35457 total, 1254.7 MiB)
20453 records/s, 892.7 MiB/s, 44.7 KiB/rec (45684 total, 1701.1 MiB)
18983 records/s, 857.1 MiB/s, 46.2 KiB/rec (55176 total, 2129.7 MiB)
20248 records/s, 882.7 MiB/s, 44.6 KiB/rec (65300 total, 2571.0 MiB)
19195 records/s, 912.1 MiB/s, 48.7 KiB/rec (74898 total, 3027.1 MiB)
16015 records/s, 931.7 MiB/s, 59.6 KiB/rec (82908 total, 3493.1 MiB)
16272 records/s, 933.9 MiB/s, 58.8 KiB/rec (91044 total, 3960.0 MiB)
15301 records/s, 918.7 MiB/s, 61.5 KiB/rec (98706 total, 4420.1 MiB)
16481 records/s, 905.4 MiB/s, 56.3 KiB/rec (106947 total, 4872.8 MiB)
Summary: 6.0s, 19202 records/s, 890.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.zst
37744 records/s, 1068.2 MiB/s, 29.0 KiB/rec (18873 total, 534.1 MiB)
25433 records/s, 1104.1 MiB/s, 44.5 KiB/rec (31590 total, 1086.2 MiB)
25336 records/s, 1110.2 MiB/s, 44.9 KiB/rec (44259 total, 1641.4 MiB)
24715 records/s, 1104.7 MiB/s, 45.8 KiB/rec (56617 total, 2193.7 MiB)
25329 records/s, 1111.4 MiB/s, 44.9 KiB/rec (69282 total, 2749.4 MiB)
21874 records/s, 1167.9 MiB/s, 54.7 KiB/rec (80220 total, 3333.4 MiB)
20867 records/s, 1199.1 MiB/s, 58.8 KiB/rec (90654 total, 3933.0 MiB)
20196 records/s, 1197.0 MiB/s, 60.7 KiB/rec (100759 total, 4531.9 MiB)
20822 records/s, 1209.5 MiB/s, 59.5 KiB/rec (111170 total, 5136.6 MiB)
Summary: 4.6s, 24639 records/s, 1142.4 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### LZ4:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
59130 records/s, 1982.9 MiB/s, 34.3 KiB/rec (29571 total, 991.6 MiB)
43348 records/s, 1932.2 MiB/s, 45.6 KiB/rec (51246 total, 1957.8 MiB)
44616 records/s, 1990.4 MiB/s, 45.7 KiB/rec (73567 total, 2953.6 MiB)
36658 records/s, 2125.9 MiB/s, 59.4 KiB/rec (91897 total, 4016.6 MiB)
37990 records/s, 2201.0 MiB/s, 59.3 KiB/rec (110893 total, 5117.2 MiB)
Summary: 2.6s, 44109 records/s, 2045.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```
