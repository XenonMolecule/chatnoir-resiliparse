# Benchmark: jwarc

Benchmark for [Gowarc](https://github.com/internetarchive/gowarc).

## Install Dependencies:

```bash
sudo apt install openjdk-25-jdk maven
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
15971 records/s, 363.9 MiB/s, 23.3 KiB/rec (7996 total, 182.2 MiB)
18034 records/s, 543.4 MiB/s, 30.9 KiB/rec (17014 total, 453.9 MiB)
14919 records/s, 644.2 MiB/s, 44.2 KiB/rec (24502 total, 777.3 MiB)
13204 records/s, 575.1 MiB/s, 44.6 KiB/rec (31138 total, 1066.3 MiB)
16331 records/s, 711.0 MiB/s, 44.6 KiB/rec (39319 total, 1422.5 MiB)
16624 records/s, 730.8 MiB/s, 45.0 KiB/rec (47641 total, 1788.3 MiB)
17728 records/s, 801.9 MiB/s, 46.3 KiB/rec (56505 total, 2189.3 MiB)
13127 records/s, 575.7 MiB/s, 44.9 KiB/rec (63121 total, 2479.4 MiB)
18804 records/s, 828.0 MiB/s, 45.1 KiB/rec (72523 total, 2893.4 MiB)
14186 records/s, 816.2 MiB/s, 58.9 KiB/rec (79616 total, 3301.5 MiB)
12327 records/s, 709.2 MiB/s, 58.9 KiB/rec (85789 total, 3656.7 MiB)
12318 records/s, 722.0 MiB/s, 60.0 KiB/rec (91948 total, 4017.7 MiB)
11240 records/s, 683.0 MiB/s, 62.2 KiB/rec (97606 total, 4361.5 MiB)
13554 records/s, 733.3 MiB/s, 55.4 KiB/rec (104398 total, 4729.0 MiB)
13183 records/s, 790.2 MiB/s, 61.4 KiB/rec (111007 total, 5125.1 MiB)
Summary: 7.8s, 14695 records/s, 681.4 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
9655 records/s, 205.0 MiB/s, 21.7 KiB/rec (4828 total, 102.5 MiB)
12702 records/s, 304.4 MiB/s, 24.5 KiB/rec (11182 total, 254.8 MiB)
10494 records/s, 349.7 MiB/s, 34.1 KiB/rec (16429 total, 429.6 MiB)
8752 records/s, 375.6 MiB/s, 44.0 KiB/rec (20806 total, 617.5 MiB)
8319 records/s, 360.4 MiB/s, 44.4 KiB/rec (24967 total, 797.7 MiB)
9012 records/s, 380.4 MiB/s, 43.2 KiB/rec (29473 total, 987.9 MiB)
8337 records/s, 384.3 MiB/s, 47.2 KiB/rec (33651 total, 1180.5 MiB)
9021 records/s, 379.2 MiB/s, 43.0 KiB/rec (38164 total, 1370.2 MiB)
8605 records/s, 393.9 MiB/s, 46.9 KiB/rec (42469 total, 1567.3 MiB)
8972 records/s, 386.3 MiB/s, 44.1 KiB/rec (46955 total, 1760.4 MiB)
8509 records/s, 392.8 MiB/s, 47.3 KiB/rec (51211 total, 1956.9 MiB)
8980 records/s, 393.7 MiB/s, 44.9 KiB/rec (55702 total, 2153.8 MiB)
9157 records/s, 384.6 MiB/s, 43.0 KiB/rec (60283 total, 2346.2 MiB)
8225 records/s, 375.4 MiB/s, 46.7 KiB/rec (64396 total, 2533.9 MiB)
8526 records/s, 374.4 MiB/s, 45.0 KiB/rec (68662 total, 2721.2 MiB)
8414 records/s, 373.9 MiB/s, 45.5 KiB/rec (72869 total, 2908.2 MiB)
7110 records/s, 417.4 MiB/s, 60.1 KiB/rec (76426 total, 3117.0 MiB)
7156 records/s, 402.1 MiB/s, 57.5 KiB/rec (80005 total, 3318.1 MiB)
6881 records/s, 410.0 MiB/s, 61.0 KiB/rec (83449 total, 3523.3 MiB)
7165 records/s, 404.4 MiB/s, 57.8 KiB/rec (87037 total, 3725.8 MiB)
6966 records/s, 401.1 MiB/s, 59.0 KiB/rec (90520 total, 3926.4 MiB)
6251 records/s, 390.1 MiB/s, 63.9 KiB/rec (93646 total, 4121.4 MiB)
6545 records/s, 401.3 MiB/s, 62.8 KiB/rec (96919 total, 4322.2 MiB)
7298 records/s, 393.5 MiB/s, 55.2 KiB/rec (100570 total, 4519.0 MiB)
6912 records/s, 382.0 MiB/s, 56.6 KiB/rec (104026 total, 4710.0 MiB)
6914 records/s, 389.3 MiB/s, 57.7 KiB/rec (107483 total, 4904.7 MiB)
6762 records/s, 419.6 MiB/s, 63.5 KiB/rec (110864 total, 5114.5 MiB)
Summary: 14.0s, 8189 records/s, 379.7 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

**Note:** Run got stuck reproducibly at the end of the file and could not finish.

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
11524 records/s, 252.3 MiB/s, 22.4 KiB/rec (5762 total, 126.1 MiB)
17138 records/s, 425.2 MiB/s, 25.4 KiB/rec (14331 total, 338.7 MiB)
14459 records/s, 613.3 MiB/s, 43.4 KiB/rec (21561 total, 645.4 MiB)
14647 records/s, 638.2 MiB/s, 44.6 KiB/rec (28888 total, 964.7 MiB)
15917 records/s, 703.5 MiB/s, 45.3 KiB/rec (36847 total, 1316.4 MiB)
16747 records/s, 733.3 MiB/s, 44.8 KiB/rec (45222 total, 1683.2 MiB)
17211 records/s, 782.3 MiB/s, 46.5 KiB/rec (53829 total, 2074.4 MiB)
17825 records/s, 776.9 MiB/s, 44.6 KiB/rec (62779 total, 2464.4 MiB)
13790 records/s, 605.3 MiB/s, 45.0 KiB/rec (69675 total, 2767.2 MiB)
12916 records/s, 661.5 MiB/s, 52.4 KiB/rec (76134 total, 3097.9 MiB)
12630 records/s, 726.3 MiB/s, 58.9 KiB/rec (82455 total, 3461.4 MiB)
12324 records/s, 711.9 MiB/s, 59.2 KiB/rec (88618 total, 3817.5 MiB)
11889 records/s, 724.9 MiB/s, 62.4 KiB/rec (94569 total, 4180.3 MiB)
12046 records/s, 683.8 MiB/s, 58.1 KiB/rec (100593 total, 4522.3 MiB)
12215 records/s, 673.8 MiB/s, 56.5 KiB/rec (106701 total, 4859.2 MiB)
11609 records/s, 706.5 MiB/s, 62.3 KiB/rec (112506 total, 5212.5 MiB)
...
```

### LZ4:

Unsupported.

## Results (RAM)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
22989 records/s, 524.7 MiB/s, 23.4 KiB/rec (11497 total, 262.4 MiB)
36762 records/s, 1486.7 MiB/s, 41.4 KiB/rec (29881 total, 1005.9 MiB)
50476 records/s, 2244.6 MiB/s, 45.5 KiB/rec (55119 total, 2128.2 MiB)
50189 records/s, 2408.0 MiB/s, 49.1 KiB/rec (80214 total, 3332.2 MiB)
46122 records/s, 2673.4 MiB/s, 59.4 KiB/rec (103283 total, 4669.4 MiB)
Summary: 2.7s, 41563 records/s, 1927.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
9449 records/s, 201.3 MiB/s, 21.8 KiB/rec (4729 total, 100.8 MiB)
12162 records/s, 287.8 MiB/s, 24.2 KiB/rec (10810 total, 244.7 MiB)
11092 records/s, 364.2 MiB/s, 33.6 KiB/rec (16356 total, 426.8 MiB)
9199 records/s, 393.3 MiB/s, 43.8 KiB/rec (20971 total, 624.1 MiB)
8825 records/s, 390.5 MiB/s, 45.3 KiB/rec (25387 total, 819.5 MiB)
9412 records/s, 394.3 MiB/s, 42.9 KiB/rec (30094 total, 1016.7 MiB)
8935 records/s, 397.4 MiB/s, 45.5 KiB/rec (34576 total, 1216.0 MiB)
9306 records/s, 406.9 MiB/s, 44.8 KiB/rec (39235 total, 1419.8 MiB)
9281 records/s, 408.1 MiB/s, 45.0 KiB/rec (43876 total, 1623.8 MiB)
8558 records/s, 378.4 MiB/s, 45.3 KiB/rec (48157 total, 1813.1 MiB)
8738 records/s, 409.9 MiB/s, 48.0 KiB/rec (52537 total, 2018.6 MiB)
9498 records/s, 406.1 MiB/s, 43.8 KiB/rec (57289 total, 2221.8 MiB)
9206 records/s, 396.8 MiB/s, 44.1 KiB/rec (61894 total, 2420.2 MiB)
8976 records/s, 395.3 MiB/s, 45.1 KiB/rec (66382 total, 2617.9 MiB)
8501 records/s, 391.6 MiB/s, 47.2 KiB/rec (70633 total, 2813.7 MiB)
7886 records/s, 393.6 MiB/s, 51.1 KiB/rec (74578 total, 3010.6 MiB)
7446 records/s, 418.3 MiB/s, 57.5 KiB/rec (78304 total, 3219.9 MiB)
7175 records/s, 407.9 MiB/s, 58.2 KiB/rec (81892 total, 3423.9 MiB)
7237 records/s, 427.5 MiB/s, 60.5 KiB/rec (85525 total, 3638.5 MiB)
7266 records/s, 419.4 MiB/s, 59.1 KiB/rec (89164 total, 3848.5 MiB)
6795 records/s, 416.4 MiB/s, 62.7 KiB/rec (92575 total, 4057.6 MiB)
6688 records/s, 416.5 MiB/s, 63.8 KiB/rec (95920 total, 4265.9 MiB)
7716 records/s, 415.4 MiB/s, 55.1 KiB/rec (99778 total, 4473.6 MiB)
7440 records/s, 418.4 MiB/s, 57.6 KiB/rec (103504 total, 4683.1 MiB)
7584 records/s, 415.6 MiB/s, 56.1 KiB/rec (107296 total, 4890.9 MiB)
6890 records/s, 427.3 MiB/s, 63.5 KiB/rec (110743 total, 5104.7 MiB)
Summary: 13.5s, 8483 records/s, 393.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

**Note:** Run got stuck reproducibly at the end of the file and could not finish.

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
12978 records/s, 284.6 MiB/s, 22.5 KiB/rec (6490 total, 142.3 MiB)
20050 records/s, 585.8 MiB/s, 29.9 KiB/rec (16515 total, 435.2 MiB)
18505 records/s, 799.7 MiB/s, 44.2 KiB/rec (25768 total, 835.1 MiB)
19083 records/s, 822.8 MiB/s, 44.2 KiB/rec (35310 total, 1246.5 MiB)
18437 records/s, 821.5 MiB/s, 45.6 KiB/rec (44529 total, 1657.2 MiB)
18932 records/s, 851.7 MiB/s, 46.1 KiB/rec (53995 total, 2083.1 MiB)
19582 records/s, 845.7 MiB/s, 44.2 KiB/rec (63787 total, 2506.0 MiB)
15575 records/s, 698.0 MiB/s, 45.9 KiB/rec (71575 total, 2855.0 MiB)
16392 records/s, 909.4 MiB/s, 56.8 KiB/rec (79771 total, 3309.7 MiB)
15105 records/s, 862.6 MiB/s, 58.5 KiB/rec (87325 total, 3741.1 MiB)
14364 records/s, 870.4 MiB/s, 62.1 KiB/rec (94510 total, 4176.5 MiB)
15354 records/s, 861.0 MiB/s, 57.4 KiB/rec (102187 total, 4607.0 MiB)
14456 records/s, 848.8 MiB/s, 60.1 KiB/rec (109416 total, 5031.5 MiB)
...
```

### LZ4:

Unsupported.
