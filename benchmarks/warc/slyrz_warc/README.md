# Benchmark: slyrz_warc

Benchmark for [slyrz_warc](https://github.com/slyrz/warc).

## Install Dependencies:

```bash
apt install golang
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
31441 records/s, 801.9 MiB/s, 26.1 KiB/rec (15795 total, 402.8 MiB)
22690 records/s, 978.5 MiB/s, 44.2 KiB/rec (27140 total, 892.1 MiB)
24846 records/s, 1085.3 MiB/s, 44.7 KiB/rec (39594 total, 1436.1 MiB)
23680 records/s, 1059.6 MiB/s, 45.8 KiB/rec (51444 total, 1966.3 MiB)
25525 records/s, 1111.8 MiB/s, 44.6 KiB/rec (64209 total, 2522.3 MiB)
20592 records/s, 969.9 MiB/s, 48.2 KiB/rec (74511 total, 3007.6 MiB)
18568 records/s, 1070.6 MiB/s, 59.0 KiB/rec (83795 total, 3542.8 MiB)
18538 records/s, 1087.4 MiB/s, 60.1 KiB/rec (93066 total, 4086.7 MiB)
18480 records/s, 1052.6 MiB/s, 58.3 KiB/rec (102306 total, 4613.0 MiB)
17771 records/s, 1052.7 MiB/s, 60.7 KiB/rec (111192 total, 5139.4 MiB)
Summary: 5.1s, 22240 records/s, 1031.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
8621 records/s, 186.3 MiB/s, 22.1 KiB/rec (4323 total, 93.4 MiB)
8264 records/s, 198.3 MiB/s, 24.6 KiB/rec (8455 total, 192.6 MiB)
8060 records/s, 189.7 MiB/s, 24.1 KiB/rec (12504 total, 287.9 MiB)
5824 records/s, 202.8 MiB/s, 35.6 KiB/rec (15417 total, 389.3 MiB)
5196 records/s, 209.2 MiB/s, 41.2 KiB/rec (18015 total, 493.9 MiB)
4979 records/s, 220.9 MiB/s, 45.4 KiB/rec (20505 total, 604.3 MiB)
4741 records/s, 202.3 MiB/s, 43.7 KiB/rec (22878 total, 705.6 MiB)
4883 records/s, 221.0 MiB/s, 46.3 KiB/rec (25320 total, 816.1 MiB)
5524 records/s, 237.5 MiB/s, 44.0 KiB/rec (28083 total, 934.9 MiB)
5352 records/s, 225.2 MiB/s, 43.1 KiB/rec (30759 total, 1047.5 MiB)
4622 records/s, 209.7 MiB/s, 46.5 KiB/rec (33078 total, 1152.7 MiB)
5654 records/s, 246.7 MiB/s, 44.7 KiB/rec (35907 total, 1276.1 MiB)
5280 records/s, 224.3 MiB/s, 43.5 KiB/rec (38550 total, 1388.4 MiB)
5222 records/s, 232.7 MiB/s, 45.6 KiB/rec (41166 total, 1505.0 MiB)
5187 records/s, 231.6 MiB/s, 45.7 KiB/rec (43761 total, 1620.8 MiB)
5701 records/s, 245.3 MiB/s, 44.1 KiB/rec (46614 total, 1743.6 MiB)
5476 records/s, 242.9 MiB/s, 45.4 KiB/rec (49353 total, 1865.0 MiB)
4600 records/s, 223.7 MiB/s, 49.8 KiB/rec (51654 total, 1976.9 MiB)
5309 records/s, 241.4 MiB/s, 46.6 KiB/rec (54309 total, 2097.7 MiB)
5807 records/s, 243.2 MiB/s, 42.9 KiB/rec (57213 total, 2219.3 MiB)
5721 records/s, 236.8 MiB/s, 42.4 KiB/rec (60075 total, 2337.7 MiB)
4996 records/s, 233.4 MiB/s, 47.9 KiB/rec (62574 total, 2454.5 MiB)
5473 records/s, 234.6 MiB/s, 43.9 KiB/rec (65313 total, 2571.9 MiB)
4601 records/s, 212.4 MiB/s, 47.3 KiB/rec (67617 total, 2678.3 MiB)
5338 records/s, 240.1 MiB/s, 46.1 KiB/rec (70287 total, 2798.4 MiB)
5212 records/s, 220.7 MiB/s, 43.4 KiB/rec (72894 total, 2908.8 MiB)
4057 records/s, 239.6 MiB/s, 60.5 KiB/rec (74925 total, 3028.7 MiB)
4229 records/s, 248.5 MiB/s, 60.2 KiB/rec (77049 total, 3153.5 MiB)
4383 records/s, 245.7 MiB/s, 57.4 KiB/rec (79254 total, 3277.1 MiB)
4126 records/s, 233.6 MiB/s, 58.0 KiB/rec (81318 total, 3394.0 MiB)
4074 records/s, 250.0 MiB/s, 62.9 KiB/rec (83355 total, 3519.0 MiB)
4388 records/s, 242.5 MiB/s, 56.6 KiB/rec (85551 total, 3640.4 MiB)
3982 records/s, 223.0 MiB/s, 57.3 KiB/rec (87543 total, 3751.9 MiB)
4219 records/s, 248.4 MiB/s, 60.3 KiB/rec (89655 total, 3876.2 MiB)
3949 records/s, 250.8 MiB/s, 65.0 KiB/rec (91665 total, 4003.9 MiB)
4407 records/s, 251.2 MiB/s, 58.4 KiB/rec (93870 total, 4129.6 MiB)
3795 records/s, 253.0 MiB/s, 68.3 KiB/rec (95778 total, 4256.8 MiB)
4269 records/s, 241.8 MiB/s, 58.0 KiB/rec (97914 total, 4377.8 MiB)
4222 records/s, 222.0 MiB/s, 53.8 KiB/rec (100026 total, 4488.8 MiB)
4444 records/s, 242.7 MiB/s, 55.9 KiB/rec (102249 total, 4610.2 MiB)
3676 records/s, 207.4 MiB/s, 57.8 KiB/rec (104088 total, 4714.0 MiB)
4115 records/s, 225.1 MiB/s, 56.0 KiB/rec (106152 total, 4826.9 MiB)
3622 records/s, 218.8 MiB/s, 61.9 KiB/rec (107970 total, 4936.7 MiB)
4011 records/s, 257.0 MiB/s, 65.6 KiB/rec (109977 total, 5065.3 MiB)
4204 records/s, 244.8 MiB/s, 59.6 KiB/rec (112110 total, 5189.5 MiB)
Summary: 23.0s, 4975 records/s, 230.7 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
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
67521 records/s, 2368.2 MiB/s, 35.9 KiB/rec (33761 total, 1184.1 MiB)
68052 records/s, 3002.3 MiB/s, 45.2 KiB/rec (67787 total, 2685.3 MiB)
55840 records/s, 3133.5 MiB/s, 57.5 KiB/rec (95709 total, 4252.1 MiB)
Summary: 1.9s, 61697 records/s, 2860.6 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
8724 records/s, 188.5 MiB/s, 22.1 KiB/rec (4362 total, 94.2 MiB)
8207 records/s, 196.9 MiB/s, 24.6 KiB/rec (8466 total, 192.7 MiB)
8087 records/s, 190.5 MiB/s, 24.1 KiB/rec (12516 total, 288.1 MiB)
5891 records/s, 206.7 MiB/s, 35.9 KiB/rec (15462 total, 391.5 MiB)
5593 records/s, 226.8 MiB/s, 41.5 KiB/rec (18261 total, 505.0 MiB)
5366 records/s, 235.1 MiB/s, 44.9 KiB/rec (20946 total, 622.7 MiB)
5226 records/s, 227.0 MiB/s, 44.5 KiB/rec (23562 total, 736.3 MiB)
5182 records/s, 235.5 MiB/s, 46.5 KiB/rec (26153 total, 854.0 MiB)
5736 records/s, 233.2 MiB/s, 41.6 KiB/rec (29022 total, 970.7 MiB)
5034 records/s, 226.1 MiB/s, 46.0 KiB/rec (31545 total, 1084.0 MiB)
5945 records/s, 245.9 MiB/s, 42.3 KiB/rec (34521 total, 1207.1 MiB)
5220 records/s, 240.3 MiB/s, 47.1 KiB/rec (37134 total, 1327.3 MiB)
5536 records/s, 239.9 MiB/s, 44.4 KiB/rec (39906 total, 1447.5 MiB)
4851 records/s, 228.0 MiB/s, 48.1 KiB/rec (42333 total, 1561.6 MiB)
5934 records/s, 251.4 MiB/s, 43.4 KiB/rec (45303 total, 1687.4 MiB)
5267 records/s, 233.7 MiB/s, 45.4 KiB/rec (47937 total, 1804.2 MiB)
5321 records/s, 249.1 MiB/s, 47.9 KiB/rec (50598 total, 1928.8 MiB)
5160 records/s, 237.3 MiB/s, 47.1 KiB/rec (53181 total, 2047.6 MiB)
5458 records/s, 225.4 MiB/s, 42.3 KiB/rec (55910 total, 2160.3 MiB)
5168 records/s, 230.1 MiB/s, 45.6 KiB/rec (58494 total, 2275.3 MiB)
5559 records/s, 228.4 MiB/s, 42.1 KiB/rec (61284 total, 2390.0 MiB)
5300 records/s, 242.7 MiB/s, 46.9 KiB/rec (63936 total, 2511.4 MiB)
5195 records/s, 230.3 MiB/s, 45.4 KiB/rec (66537 total, 2626.7 MiB)
5249 records/s, 233.6 MiB/s, 45.6 KiB/rec (69162 total, 2743.6 MiB)
5278 records/s, 239.7 MiB/s, 46.5 KiB/rec (71808 total, 2863.7 MiB)
4612 records/s, 249.4 MiB/s, 55.4 KiB/rec (74142 total, 2990.0 MiB)
4580 records/s, 256.6 MiB/s, 57.4 KiB/rec (76432 total, 3118.2 MiB)
4691 records/s, 255.5 MiB/s, 55.8 KiB/rec (78792 total, 3246.8 MiB)
3928 records/s, 233.5 MiB/s, 60.9 KiB/rec (80757 total, 3363.6 MiB)
4277 records/s, 256.6 MiB/s, 61.4 KiB/rec (82896 total, 3491.9 MiB)
4383 records/s, 249.1 MiB/s, 58.2 KiB/rec (85089 total, 3616.5 MiB)
4598 records/s, 262.1 MiB/s, 58.4 KiB/rec (87396 total, 3748.0 MiB)
4438 records/s, 251.8 MiB/s, 58.1 KiB/rec (89616 total, 3874.0 MiB)
3881 records/s, 242.0 MiB/s, 63.9 KiB/rec (91557 total, 3995.0 MiB)
3786 records/s, 226.6 MiB/s, 61.3 KiB/rec (93450 total, 4108.3 MiB)
4038 records/s, 255.7 MiB/s, 64.9 KiB/rec (95472 total, 4236.4 MiB)
3910 records/s, 235.7 MiB/s, 61.7 KiB/rec (97428 total, 4354.3 MiB)
4600 records/s, 233.1 MiB/s, 51.9 KiB/rec (99729 total, 4470.9 MiB)
4301 records/s, 237.3 MiB/s, 56.5 KiB/rec (101880 total, 4589.6 MiB)
4237 records/s, 237.8 MiB/s, 57.5 KiB/rec (104004 total, 4708.8 MiB)
4211 records/s, 231.6 MiB/s, 56.3 KiB/rec (106110 total, 4824.6 MiB)
3743 records/s, 231.7 MiB/s, 63.4 KiB/rec (108006 total, 4941.9 MiB)
3881 records/s, 244.3 MiB/s, 64.4 KiB/rec (109950 total, 5064.3 MiB)
4442 records/s, 255.9 MiB/s, 59.0 KiB/rec (112173 total, 5192.3 MiB)
Summary: 22.5s, 5081 records/s, 235.6 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

Unsupported.
