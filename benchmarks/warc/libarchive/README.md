# Benchmark: libarchive

Benchmark for [libarchive](https://github.com/libarchive/libarchive).

**Note:** By default, `libarchive` emits only content records and skips request or metadata records. This benchmark
has to apply a custom patch to make the results comparable. At the moment, the record totals are still off by one,
which probably stems from the `warcinfo` record being skipped (patch welcome!).

## Install Dependencies:

```bash
sudo apt install build-essential
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
39815 records/s, 1159.4 MiB/s, 29.8 KiB/rec (19984 total, 581.9 MiB)
27981 records/s, 1215.3 MiB/s, 44.5 KiB/rec (34020 total, 1191.5 MiB)
27314 records/s, 1198.9 MiB/s, 44.9 KiB/rec (47692 total, 1791.7 MiB)
26241 records/s, 1153.3 MiB/s, 45.0 KiB/rec (60823 total, 2368.8 MiB)
25393 records/s, 1164.5 MiB/s, 47.0 KiB/rec (73534 total, 2951.7 MiB)
19619 records/s, 1133.3 MiB/s, 59.2 KiB/rec (83350 total, 3518.7 MiB)
19045 records/s, 1109.7 MiB/s, 59.7 KiB/rec (92890 total, 4074.5 MiB)
21097 records/s, 1211.2 MiB/s, 58.8 KiB/rec (103459 total, 4681.3 MiB)
21072 records/s, 1211.9 MiB/s, 58.9 KiB/rec (113995 total, 5287.2 MiB)
Summary: 4.5s, 25331 records/s, 1174.5 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### Gzip:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
18211 records/s, 417.8 MiB/s, 23.5 KiB/rec (9111 total, 209.0 MiB)
14460 records/s, 434.7 MiB/s, 30.8 KiB/rec (16374 total, 427.4 MiB)
11181 records/s, 473.5 MiB/s, 43.4 KiB/rec (21966 total, 664.2 MiB)
10749 records/s, 472.5 MiB/s, 45.0 KiB/rec (27348 total, 900.7 MiB)
10949 records/s, 480.1 MiB/s, 44.9 KiB/rec (32823 total, 1140.8 MiB)
11095 records/s, 478.7 MiB/s, 44.2 KiB/rec (38373 total, 1380.3 MiB)
10734 records/s, 478.9 MiB/s, 45.7 KiB/rec (43743 total, 1619.9 MiB)
11002 records/s, 483.2 MiB/s, 45.0 KiB/rec (49245 total, 1861.5 MiB)
10276 records/s, 474.8 MiB/s, 47.3 KiB/rec (54384 total, 2099.0 MiB)
11133 records/s, 464.5 MiB/s, 42.7 KiB/rec (59952 total, 2331.3 MiB)
10363 records/s, 465.2 MiB/s, 46.0 KiB/rec (65136 total, 2564.0 MiB)
10367 records/s, 471.5 MiB/s, 46.6 KiB/rec (70320 total, 2799.8 MiB)
9728 records/s, 487.0 MiB/s, 51.3 KiB/rec (75189 total, 3043.6 MiB)
8672 records/s, 502.2 MiB/s, 59.3 KiB/rec (79536 total, 3295.4 MiB)
8655 records/s, 503.2 MiB/s, 59.5 KiB/rec (83865 total, 3547.0 MiB)
9004 records/s, 510.9 MiB/s, 58.1 KiB/rec (88380 total, 3803.2 MiB)
8182 records/s, 493.7 MiB/s, 61.8 KiB/rec (92475 total, 4050.4 MiB)
7811 records/s, 486.8 MiB/s, 63.8 KiB/rec (96381 total, 4293.8 MiB)
8953 records/s, 486.5 MiB/s, 55.6 KiB/rec (100869 total, 4537.7 MiB)
8634 records/s, 466.2 MiB/s, 55.3 KiB/rec (105207 total, 4771.9 MiB)
8051 records/s, 492.0 MiB/s, 62.6 KiB/rec (109233 total, 5018.0 MiB)
8534 records/s, 495.1 MiB/s, 59.4 KiB/rec (113505 total, 5265.9 MiB)
Summary: 11.1s, 10314 records/s, 478.2 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### Zstandard:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
16174 records/s, 368.1 MiB/s, 23.3 KiB/rec (8088 total, 184.1 MiB)
14374 records/s, 396.1 MiB/s, 28.2 KiB/rec (15276 total, 382.2 MiB)
9134 records/s, 391.1 MiB/s, 43.8 KiB/rec (19845 total, 577.8 MiB)
8158 records/s, 350.4 MiB/s, 44.0 KiB/rec (23925 total, 753.0 MiB)
8622 records/s, 374.7 MiB/s, 44.5 KiB/rec (28239 total, 940.5 MiB)
8029 records/s, 350.9 MiB/s, 44.7 KiB/rec (32256 total, 1116.0 MiB)
8381 records/s, 363.0 MiB/s, 44.4 KiB/rec (36447 total, 1297.6 MiB)
8664 records/s, 377.1 MiB/s, 44.6 KiB/rec (40785 total, 1486.4 MiB)
8923 records/s, 396.7 MiB/s, 45.5 KiB/rec (45249 total, 1684.9 MiB)
8581 records/s, 383.3 MiB/s, 45.7 KiB/rec (49566 total, 1877.7 MiB)
8497 records/s, 392.4 MiB/s, 47.3 KiB/rec (53817 total, 2074.0 MiB)
8952 records/s, 393.1 MiB/s, 45.0 KiB/rec (58293 total, 2270.5 MiB)
9160 records/s, 400.8 MiB/s, 44.8 KiB/rec (62874 total, 2471.0 MiB)
9309 records/s, 405.4 MiB/s, 44.6 KiB/rec (67530 total, 2673.8 MiB)
8841 records/s, 392.5 MiB/s, 45.5 KiB/rec (71955 total, 2870.3 MiB)
8026 records/s, 434.8 MiB/s, 55.5 KiB/rec (75972 total, 3087.9 MiB)
8033 records/s, 458.5 MiB/s, 58.4 KiB/rec (79989 total, 3317.2 MiB)
8629 records/s, 510.0 MiB/s, 60.5 KiB/rec (84315 total, 3572.8 MiB)
9189 records/s, 520.0 MiB/s, 57.9 KiB/rec (88911 total, 3832.9 MiB)
7506 records/s, 457.7 MiB/s, 62.4 KiB/rec (92664 total, 4061.8 MiB)
7552 records/s, 469.2 MiB/s, 63.6 KiB/rec (96441 total, 4296.5 MiB)
9096 records/s, 497.6 MiB/s, 56.0 KiB/rec (100992 total, 4545.4 MiB)
8903 records/s, 477.3 MiB/s, 54.9 KiB/rec (105456 total, 4784.8 MiB)
8897 records/s, 552.2 MiB/s, 63.6 KiB/rec (109911 total, 5061.2 MiB)
Summary: 12.5s, 9143 records/s, 423.9 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### LZ4:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
50069 records/s, 1605.2 MiB/s, 32.8 KiB/rec (25035 total, 802.6 MiB)
39153 records/s, 1715.3 MiB/s, 44.9 KiB/rec (44617 total, 1660.5 MiB)
42925 records/s, 1889.6 MiB/s, 45.1 KiB/rec (66081 total, 2605.3 MiB)
34313 records/s, 1815.7 MiB/s, 54.2 KiB/rec (83256 total, 3514.2 MiB)
37725 records/s, 2176.6 MiB/s, 59.1 KiB/rec (102165 total, 4605.2 MiB)
Summary: 2.8s, 40573 records/s, 1881.2 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

## Results (RAM)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
140971 records/s, 5613.6 MiB/s, 40.8 KiB/rec (70492 total, 2807.1 MiB)
Summary: 0.8s, 134523 records/s, 6237.3 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

k

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
20204 records/s, 455.2 MiB/s, 23.1 KiB/rec (10104 total, 227.6 MiB)
14140 records/s, 471.7 MiB/s, 34.2 KiB/rec (17175 total, 463.5 MiB)
11585 records/s, 493.9 MiB/s, 43.7 KiB/rec (22980 total, 711.0 MiB)
11674 records/s, 498.7 MiB/s, 43.7 KiB/rec (28842 total, 961.4 MiB)
11405 records/s, 500.6 MiB/s, 45.0 KiB/rec (34554 total, 1212.2 MiB)
11518 records/s, 499.1 MiB/s, 44.4 KiB/rec (40317 total, 1461.9 MiB)
11358 records/s, 509.6 MiB/s, 45.9 KiB/rec (46002 total, 1716.9 MiB)
10870 records/s, 497.7 MiB/s, 46.9 KiB/rec (51441 total, 1966.0 MiB)
11437 records/s, 502.2 MiB/s, 45.0 KiB/rec (57168 total, 2217.5 MiB)
11311 records/s, 501.1 MiB/s, 45.4 KiB/rec (62829 total, 2468.3 MiB)
11267 records/s, 490.0 MiB/s, 44.5 KiB/rec (68463 total, 2713.3 MiB)
10595 records/s, 502.9 MiB/s, 48.6 KiB/rec (73761 total, 2964.8 MiB)
9417 records/s, 529.1 MiB/s, 57.5 KiB/rec (78471 total, 3229.4 MiB)
8784 records/s, 520.9 MiB/s, 60.7 KiB/rec (82863 total, 3489.9 MiB)
9412 records/s, 527.3 MiB/s, 57.4 KiB/rec (87570 total, 3753.6 MiB)
8628 records/s, 524.3 MiB/s, 62.2 KiB/rec (91884 total, 4015.7 MiB)
8309 records/s, 513.6 MiB/s, 63.3 KiB/rec (96039 total, 4272.6 MiB)
9408 records/s, 515.9 MiB/s, 56.2 KiB/rec (100743 total, 4530.5 MiB)
9065 records/s, 495.0 MiB/s, 55.9 KiB/rec (105282 total, 4778.3 MiB)
8582 records/s, 526.1 MiB/s, 62.8 KiB/rec (109584 total, 5042.1 MiB)
Summary: 10.5s, 10890 records/s, 504.9 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### Zstandard:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
14184 records/s, 314.7 MiB/s, 22.7 KiB/rec (7096 total, 157.4 MiB)
13161 records/s, 315.4 MiB/s, 24.5 KiB/rec (13680 total, 315.2 MiB)
10013 records/s, 417.9 MiB/s, 42.7 KiB/rec (18687 total, 524.2 MiB)
8722 records/s, 380.8 MiB/s, 44.7 KiB/rec (23052 total, 714.8 MiB)
9850 records/s, 434.7 MiB/s, 45.2 KiB/rec (27978 total, 932.1 MiB)
10120 records/s, 436.5 MiB/s, 44.2 KiB/rec (33042 total, 1150.6 MiB)
9467 records/s, 410.2 MiB/s, 44.4 KiB/rec (37779 total, 1355.8 MiB)
9314 records/s, 419.4 MiB/s, 46.1 KiB/rec (42438 total, 1565.6 MiB)
10657 records/s, 461.1 MiB/s, 44.3 KiB/rec (47772 total, 1796.4 MiB)
8794 records/s, 412.6 MiB/s, 48.0 KiB/rec (52170 total, 2002.8 MiB)
9728 records/s, 419.1 MiB/s, 44.1 KiB/rec (57036 total, 2212.4 MiB)
9527 records/s, 405.6 MiB/s, 43.6 KiB/rec (61800 total, 2415.2 MiB)
9009 records/s, 402.2 MiB/s, 45.7 KiB/rec (66306 total, 2616.4 MiB)
8987 records/s, 408.4 MiB/s, 46.5 KiB/rec (70803 total, 2820.7 MiB)
8974 records/s, 461.8 MiB/s, 52.7 KiB/rec (75297 total, 3052.0 MiB)
9265 records/s, 524.7 MiB/s, 58.0 KiB/rec (79932 total, 3314.4 MiB)
9723 records/s, 576.5 MiB/s, 60.7 KiB/rec (84795 total, 3602.8 MiB)
8860 records/s, 496.7 MiB/s, 57.4 KiB/rec (89226 total, 3851.2 MiB)
8157 records/s, 499.3 MiB/s, 62.7 KiB/rec (93306 total, 4100.9 MiB)
10131 records/s, 600.4 MiB/s, 60.7 KiB/rec (98373 total, 4401.2 MiB)
10536 records/s, 574.1 MiB/s, 55.8 KiB/rec (103650 total, 4688.8 MiB)
8835 records/s, 512.1 MiB/s, 59.4 KiB/rec (108069 total, 4944.9 MiB)
8924 records/s, 539.9 MiB/s, 61.9 KiB/rec (112533 total, 5215.0 MiB)
Summary: 11.7s, 9803 records/s, 454.5 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### LZ4:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
76188 records/s, 2736.5 MiB/s, 36.8 KiB/rec (38095 total, 1368.3 MiB)
66139 records/s, 2938.6 MiB/s, 45.5 KiB/rec (71166 total, 2837.6 MiB)
53726 records/s, 3091.8 MiB/s, 58.9 KiB/rec (98029 total, 4383.5 MiB)
Summary: 1.8s, 63492 records/s, 2943.9 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```
