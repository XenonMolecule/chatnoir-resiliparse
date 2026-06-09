# Benchmark: warcio

Benchmark for [warcio](https://github.com/webrecorder/warcio).

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
8617 records/s, 185.9 MiB/s, 22.1 KiB/rec (4309 total, 93.0 MiB)
7969 records/s, 193.5 MiB/s, 24.9 KiB/rec (8294 total, 189.7 MiB)
8994 records/s, 212.3 MiB/s, 24.2 KiB/rec (12791 total, 295.9 MiB)
7632 records/s, 284.5 MiB/s, 38.2 KiB/rec (16618 total, 438.5 MiB)
7121 records/s, 305.0 MiB/s, 43.9 KiB/rec (20179 total, 591.0 MiB)
7138 records/s, 309.2 MiB/s, 44.4 KiB/rec (23748 total, 745.6 MiB)
7120 records/s, 305.1 MiB/s, 43.9 KiB/rec (27309 total, 898.2 MiB)
7350 records/s, 320.1 MiB/s, 44.6 KiB/rec (30984 total, 1058.3 MiB)
7691 records/s, 337.1 MiB/s, 44.9 KiB/rec (34830 total, 1226.9 MiB)
7721 records/s, 331.5 MiB/s, 44.0 KiB/rec (38691 total, 1392.7 MiB)
7571 records/s, 349.7 MiB/s, 47.3 KiB/rec (42477 total, 1567.6 MiB)
7647 records/s, 333.4 MiB/s, 44.6 KiB/rec (46301 total, 1734.3 MiB)
7527 records/s, 328.6 MiB/s, 44.7 KiB/rec (50068 total, 1898.7 MiB)
7427 records/s, 348.2 MiB/s, 48.0 KiB/rec (53782 total, 2072.9 MiB)
7568 records/s, 322.9 MiB/s, 43.7 KiB/rec (57566 total, 2234.3 MiB)
7897 records/s, 328.0 MiB/s, 42.5 KiB/rec (61515 total, 2398.3 MiB)
7072 records/s, 323.4 MiB/s, 46.8 KiB/rec (65051 total, 2560.0 MiB)
7308 records/s, 324.2 MiB/s, 45.4 KiB/rec (68705 total, 2722.1 MiB)
7703 records/s, 347.1 MiB/s, 46.1 KiB/rec (72571 total, 2896.3 MiB)
7038 records/s, 396.5 MiB/s, 57.7 KiB/rec (76090 total, 3094.6 MiB)
6881 records/s, 399.3 MiB/s, 59.4 KiB/rec (79531 total, 3294.3 MiB)
6717 records/s, 394.8 MiB/s, 60.2 KiB/rec (82890 total, 3491.7 MiB)
7019 records/s, 395.7 MiB/s, 57.7 KiB/rec (86400 total, 3689.6 MiB)
6765 records/s, 389.8 MiB/s, 59.0 KiB/rec (89783 total, 3884.5 MiB)
6710 records/s, 410.2 MiB/s, 62.6 KiB/rec (93138 total, 4089.6 MiB)
6213 records/s, 393.6 MiB/s, 64.9 KiB/rec (96245 total, 4286.4 MiB)
6618 records/s, 352.3 MiB/s, 54.5 KiB/rec (99556 total, 4462.7 MiB)
6757 records/s, 375.6 MiB/s, 56.9 KiB/rec (102935 total, 4650.5 MiB)
6930 records/s, 383.2 MiB/s, 56.6 KiB/rec (106400 total, 4842.1 MiB)
6504 records/s, 407.2 MiB/s, 64.1 KiB/rec (109652 total, 5045.7 MiB)
6646 records/s, 392.4 MiB/s, 60.5 KiB/rec (112975 total, 5241.9 MiB)
Summary: 15.7s, 7294 records/s, 338.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
6501 records/s, 141.5 MiB/s, 22.3 KiB/rec (3251 total, 70.7 MiB)
6397 records/s, 138.6 MiB/s, 22.2 KiB/rec (6450 total, 140.0 MiB)
6137 records/s, 154.6 MiB/s, 25.8 KiB/rec (9520 total, 217.4 MiB)
6179 records/s, 146.5 MiB/s, 24.3 KiB/rec (12610 total, 290.7 MiB)
5326 records/s, 182.8 MiB/s, 35.1 KiB/rec (15273 total, 382.1 MiB)
4660 records/s, 192.7 MiB/s, 42.3 KiB/rec (17603 total, 478.4 MiB)
4392 records/s, 194.7 MiB/s, 45.4 KiB/rec (19799 total, 575.7 MiB)
4658 records/s, 192.1 MiB/s, 42.2 KiB/rec (22128 total, 671.8 MiB)
4473 records/s, 196.5 MiB/s, 45.0 KiB/rec (24365 total, 770.1 MiB)
4473 records/s, 201.0 MiB/s, 46.0 KiB/rec (26605 total, 870.7 MiB)
4697 records/s, 193.7 MiB/s, 42.2 KiB/rec (28954 total, 967.6 MiB)
4638 records/s, 206.4 MiB/s, 45.6 KiB/rec (31273 total, 1070.8 MiB)
4538 records/s, 208.3 MiB/s, 47.0 KiB/rec (33543 total, 1175.0 MiB)
4573 records/s, 192.5 MiB/s, 43.1 KiB/rec (35836 total, 1271.5 MiB)
4561 records/s, 194.7 MiB/s, 43.7 KiB/rec (38118 total, 1368.9 MiB)
4467 records/s, 189.0 MiB/s, 43.3 KiB/rec (40360 total, 1463.7 MiB)
4185 records/s, 206.1 MiB/s, 50.4 KiB/rec (42453 total, 1566.8 MiB)
4338 records/s, 188.5 MiB/s, 44.5 KiB/rec (44623 total, 1661.1 MiB)
4585 records/s, 195.6 MiB/s, 43.7 KiB/rec (46916 total, 1758.9 MiB)
4541 records/s, 199.7 MiB/s, 45.0 KiB/rec (49189 total, 1858.8 MiB)
4226 records/s, 200.8 MiB/s, 48.7 KiB/rec (51303 total, 1959.3 MiB)
4196 records/s, 197.0 MiB/s, 48.1 KiB/rec (53401 total, 2057.8 MiB)
4574 records/s, 190.8 MiB/s, 42.7 KiB/rec (55690 total, 2153.3 MiB)
4245 records/s, 192.3 MiB/s, 46.4 KiB/rec (57813 total, 2249.5 MiB)
4680 records/s, 181.7 MiB/s, 39.8 KiB/rec (60153 total, 2340.4 MiB)
4145 records/s, 194.3 MiB/s, 48.0 KiB/rec (62227 total, 2437.6 MiB)
4637 records/s, 202.6 MiB/s, 44.7 KiB/rec (64549 total, 2539.1 MiB)
4508 records/s, 204.1 MiB/s, 46.4 KiB/rec (66803 total, 2641.1 MiB)
4632 records/s, 200.8 MiB/s, 44.4 KiB/rec (69121 total, 2741.6 MiB)
4383 records/s, 205.0 MiB/s, 47.9 KiB/rec (71313 total, 2844.1 MiB)
4426 records/s, 212.7 MiB/s, 49.2 KiB/rec (73531 total, 2950.7 MiB)
4028 records/s, 238.2 MiB/s, 60.5 KiB/rec (75546 total, 3069.8 MiB)
4066 records/s, 229.1 MiB/s, 57.7 KiB/rec (77581 total, 3184.5 MiB)
3915 records/s, 223.6 MiB/s, 58.5 KiB/rec (79540 total, 3296.4 MiB)
3933 records/s, 212.7 MiB/s, 55.4 KiB/rec (81507 total, 3402.7 MiB)
3874 records/s, 240.3 MiB/s, 63.5 KiB/rec (83446 total, 3523.0 MiB)
4087 records/s, 223.5 MiB/s, 56.0 KiB/rec (85490 total, 3634.8 MiB)
3857 records/s, 228.3 MiB/s, 60.6 KiB/rec (87421 total, 3749.1 MiB)
4025 records/s, 227.5 MiB/s, 57.9 KiB/rec (89434 total, 3862.9 MiB)
3783 records/s, 233.1 MiB/s, 63.1 KiB/rec (91326 total, 3979.4 MiB)
3704 records/s, 223.9 MiB/s, 61.9 KiB/rec (93178 total, 4091.4 MiB)
3630 records/s, 233.1 MiB/s, 65.8 KiB/rec (94993 total, 4208.0 MiB)
3644 records/s, 217.9 MiB/s, 61.2 KiB/rec (96817 total, 4317.0 MiB)
4011 records/s, 219.7 MiB/s, 56.1 KiB/rec (98823 total, 4426.9 MiB)
4135 records/s, 223.4 MiB/s, 55.3 KiB/rec (100891 total, 4538.6 MiB)
3862 records/s, 207.1 MiB/s, 54.9 KiB/rec (102829 total, 4642.5 MiB)
4056 records/s, 222.7 MiB/s, 56.2 KiB/rec (104863 total, 4754.2 MiB)
3977 records/s, 230.9 MiB/s, 59.5 KiB/rec (106852 total, 4869.7 MiB)
3809 records/s, 235.6 MiB/s, 63.3 KiB/rec (108760 total, 4987.7 MiB)
4030 records/s, 238.0 MiB/s, 60.5 KiB/rec (110775 total, 5106.7 MiB)
3842 records/s, 237.5 MiB/s, 63.3 KiB/rec (112696 total, 5225.4 MiB)
Summary: 25.9s, 4419 records/s, 204.9 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
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
10847 records/s, 234.2 MiB/s, 22.1 KiB/rec (5424 total, 117.1 MiB)
10514 records/s, 249.8 MiB/s, 24.3 KiB/rec (10684 total, 242.1 MiB)
9685 records/s, 302.4 MiB/s, 32.0 KiB/rec (15527 total, 393.3 MiB)
9375 records/s, 397.5 MiB/s, 43.4 KiB/rec (20215 total, 592.1 MiB)
8861 records/s, 384.6 MiB/s, 44.4 KiB/rec (24646 total, 784.4 MiB)
9124 records/s, 385.1 MiB/s, 43.2 KiB/rec (29208 total, 977.0 MiB)
8803 records/s, 403.6 MiB/s, 47.0 KiB/rec (33610 total, 1178.8 MiB)
9129 records/s, 383.9 MiB/s, 43.1 KiB/rec (38176 total, 1370.8 MiB)
8751 records/s, 398.0 MiB/s, 46.6 KiB/rec (42552 total, 1569.8 MiB)
8640 records/s, 374.9 MiB/s, 44.4 KiB/rec (46872 total, 1757.2 MiB)
8765 records/s, 402.0 MiB/s, 47.0 KiB/rec (51255 total, 1958.2 MiB)
9415 records/s, 409.4 MiB/s, 44.5 KiB/rec (55963 total, 2162.9 MiB)
9095 records/s, 385.4 MiB/s, 43.4 KiB/rec (60511 total, 2355.7 MiB)
8799 records/s, 398.4 MiB/s, 46.4 KiB/rec (64911 total, 2554.9 MiB)
9073 records/s, 405.2 MiB/s, 45.7 KiB/rec (69450 total, 2757.6 MiB)
9123 records/s, 442.4 MiB/s, 49.7 KiB/rec (74014 total, 2978.9 MiB)
8805 records/s, 495.5 MiB/s, 57.6 KiB/rec (78417 total, 3226.7 MiB)
8370 records/s, 485.6 MiB/s, 59.4 KiB/rec (82604 total, 3469.6 MiB)
8483 records/s, 484.8 MiB/s, 58.5 KiB/rec (86846 total, 3712.0 MiB)
8352 records/s, 492.4 MiB/s, 60.4 KiB/rec (91023 total, 3958.2 MiB)
8058 records/s, 506.8 MiB/s, 64.4 KiB/rec (95052 total, 4211.7 MiB)
8447 records/s, 474.6 MiB/s, 57.5 KiB/rec (99276 total, 4449.0 MiB)
8188 records/s, 453.3 MiB/s, 56.7 KiB/rec (103370 total, 4675.6 MiB)
8377 records/s, 463.9 MiB/s, 56.7 KiB/rec (107559 total, 4907.7 MiB)
8358 records/s, 525.1 MiB/s, 64.3 KiB/rec (111738 total, 5170.2 MiB)
Summary: 12.8s, 8936 records/s, 414.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
6687 records/s, 143.7 MiB/s, 22.0 KiB/rec (3344 total, 71.8 MiB)
6478 records/s, 145.9 MiB/s, 23.1 KiB/rec (6583 total, 144.8 MiB)
6246 records/s, 152.4 MiB/s, 25.0 KiB/rec (9706 total, 221.0 MiB)
6032 records/s, 146.2 MiB/s, 24.8 KiB/rec (12723 total, 294.2 MiB)
5347 records/s, 186.9 MiB/s, 35.8 KiB/rec (15397 total, 387.6 MiB)
4767 records/s, 194.5 MiB/s, 41.8 KiB/rec (17782 total, 484.9 MiB)
4796 records/s, 212.2 MiB/s, 45.3 KiB/rec (20180 total, 591.1 MiB)
4964 records/s, 203.5 MiB/s, 42.0 KiB/rec (22663 total, 692.8 MiB)
4394 records/s, 201.5 MiB/s, 47.0 KiB/rec (24861 total, 793.6 MiB)
4745 records/s, 204.5 MiB/s, 44.1 KiB/rec (27234 total, 895.9 MiB)
4936 records/s, 203.7 MiB/s, 42.3 KiB/rec (29703 total, 997.8 MiB)
4590 records/s, 220.3 MiB/s, 49.2 KiB/rec (31999 total, 1108.0 MiB)
5072 records/s, 201.1 MiB/s, 40.6 KiB/rec (34536 total, 1208.6 MiB)
4564 records/s, 213.5 MiB/s, 47.9 KiB/rec (36819 total, 1315.4 MiB)
4798 records/s, 206.6 MiB/s, 44.1 KiB/rec (39218 total, 1418.7 MiB)
4641 records/s, 211.2 MiB/s, 46.6 KiB/rec (41539 total, 1524.3 MiB)
4885 records/s, 206.2 MiB/s, 43.2 KiB/rec (43983 total, 1627.5 MiB)
4639 records/s, 213.8 MiB/s, 47.2 KiB/rec (46303 total, 1734.4 MiB)
4719 records/s, 206.8 MiB/s, 44.9 KiB/rec (48663 total, 1837.9 MiB)
4362 records/s, 209.9 MiB/s, 49.3 KiB/rec (50848 total, 1943.0 MiB)
4433 records/s, 194.2 MiB/s, 44.9 KiB/rec (53065 total, 2040.1 MiB)
4761 records/s, 203.0 MiB/s, 43.7 KiB/rec (55446 total, 2141.6 MiB)
4667 records/s, 211.3 MiB/s, 46.4 KiB/rec (57784 total, 2247.5 MiB)
4893 records/s, 192.3 MiB/s, 40.3 KiB/rec (60231 total, 2343.7 MiB)
4235 records/s, 198.6 MiB/s, 48.0 KiB/rec (62350 total, 2443.0 MiB)
4665 records/s, 202.8 MiB/s, 44.5 KiB/rec (64683 total, 2544.5 MiB)
4506 records/s, 208.7 MiB/s, 47.4 KiB/rec (66936 total, 2648.8 MiB)
4750 records/s, 204.5 MiB/s, 44.1 KiB/rec (69311 total, 2751.1 MiB)
4580 records/s, 209.2 MiB/s, 46.8 KiB/rec (71602 total, 2855.7 MiB)
4322 records/s, 218.2 MiB/s, 51.7 KiB/rec (73765 total, 2964.9 MiB)
4200 records/s, 234.2 MiB/s, 57.1 KiB/rec (75865 total, 3082.0 MiB)
4105 records/s, 238.5 MiB/s, 59.5 KiB/rec (77918 total, 3201.3 MiB)
4117 records/s, 231.1 MiB/s, 57.5 KiB/rec (79977 total, 3316.9 MiB)
4097 records/s, 231.5 MiB/s, 57.9 KiB/rec (82026 total, 3432.7 MiB)
3925 records/s, 242.4 MiB/s, 63.2 KiB/rec (83992 total, 3554.1 MiB)
4094 records/s, 231.1 MiB/s, 57.8 KiB/rec (86040 total, 3669.7 MiB)
4166 records/s, 229.7 MiB/s, 56.5 KiB/rec (88123 total, 3784.6 MiB)
3989 records/s, 237.6 MiB/s, 61.0 KiB/rec (90118 total, 3903.4 MiB)
3870 records/s, 242.2 MiB/s, 64.1 KiB/rec (92053 total, 4024.5 MiB)
4025 records/s, 236.7 MiB/s, 60.2 KiB/rec (94069 total, 4143.1 MiB)
3731 records/s, 246.4 MiB/s, 67.6 KiB/rec (95935 total, 4266.3 MiB)
4217 records/s, 235.5 MiB/s, 57.2 KiB/rec (98044 total, 4384.1 MiB)
4257 records/s, 223.5 MiB/s, 53.8 KiB/rec (100173 total, 4495.9 MiB)
4239 records/s, 231.9 MiB/s, 56.0 KiB/rec (102293 total, 4611.8 MiB)
4160 records/s, 227.6 MiB/s, 56.0 KiB/rec (104373 total, 4725.7 MiB)
4029 records/s, 232.1 MiB/s, 59.0 KiB/rec (106390 total, 4841.9 MiB)
3890 records/s, 239.5 MiB/s, 63.0 KiB/rec (108335 total, 4961.6 MiB)
3902 records/s, 241.0 MiB/s, 63.2 KiB/rec (110286 total, 5082.1 MiB)
4153 records/s, 240.4 MiB/s, 59.3 KiB/rec (112363 total, 5202.3 MiB)
Summary: 24.9s, 4581 records/s, 212.4 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

Unsupported.
