# Benchmark: node-warc

Benchmark for [node-warc](https://github.com/N0taN3rd/node-warc).

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
9851 records/s, 211.4 MiB/s, 22.0 KiB/rec (4926 total, 105.7 MiB)
10589 records/s, 248.7 MiB/s, 24.1 KiB/rec (10230 total, 230.3 MiB)
9466 records/s, 280.5 MiB/s, 30.3 KiB/rec (14982 total, 371.1 MiB)
8091 records/s, 337.8 MiB/s, 42.7 KiB/rec (19068 total, 541.7 MiB)
8444 records/s, 368.8 MiB/s, 44.7 KiB/rec (23316 total, 727.3 MiB)
9810 records/s, 424.4 MiB/s, 44.3 KiB/rec (28236 total, 940.1 MiB)
8875 records/s, 390.7 MiB/s, 45.1 KiB/rec (32682 total, 1135.8 MiB)
8710 records/s, 376.4 MiB/s, 44.3 KiB/rec (37056 total, 1324.8 MiB)
9268 records/s, 410.5 MiB/s, 45.4 KiB/rec (41703 total, 1530.6 MiB)
9424 records/s, 411.0 MiB/s, 44.7 KiB/rec (46446 total, 1737.5 MiB)
8805 records/s, 411.1 MiB/s, 47.8 KiB/rec (50856 total, 1943.4 MiB)
9158 records/s, 395.7 MiB/s, 44.2 KiB/rec (55437 total, 2141.3 MiB)
9296 records/s, 393.5 MiB/s, 43.3 KiB/rec (60087 total, 2338.2 MiB)
9721 records/s, 435.7 MiB/s, 45.9 KiB/rec (64965 total, 2556.8 MiB)
9673 records/s, 438.3 MiB/s, 46.4 KiB/rec (69816 total, 2776.6 MiB)
8703 records/s, 430.6 MiB/s, 50.7 KiB/rec (74175 total, 2992.3 MiB)
7974 records/s, 440.3 MiB/s, 56.5 KiB/rec (78189 total, 3213.9 MiB)
8089 records/s, 458.9 MiB/s, 58.1 KiB/rec (82242 total, 3443.8 MiB)
7589 records/s, 451.1 MiB/s, 60.9 KiB/rec (86046 total, 3669.9 MiB)
7816 records/s, 452.1 MiB/s, 59.2 KiB/rec (89967 total, 3896.7 MiB)
7436 records/s, 452.4 MiB/s, 62.3 KiB/rec (93696 total, 4123.6 MiB)
7123 records/s, 439.4 MiB/s, 63.2 KiB/rec (97284 total, 4344.9 MiB)
8000 records/s, 430.5 MiB/s, 55.1 KiB/rec (101334 total, 4562.9 MiB)
7159 records/s, 385.4 MiB/s, 55.1 KiB/rec (104925 total, 4756.2 MiB)
7071 records/s, 427.9 MiB/s, 62.0 KiB/rec (108471 total, 4970.8 MiB)
6385 records/s, 387.7 MiB/s, 62.2 KiB/rec (111690 total, 5166.2 MiB)
Summary: 13.4s, 8548 records/s, 396.3 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### Gzip:

```console
$ sync && echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
4596 records/s, 107.2 MiB/s, 23.9 KiB/rec (2316 total, 54.0 MiB)
5717 records/s, 112.9 MiB/s, 20.2 KiB/rec (5187 total, 110.7 MiB)
4496 records/s, 122.9 MiB/s, 28.0 KiB/rec (7443 total, 172.4 MiB)
5449 records/s, 113.0 MiB/s, 21.2 KiB/rec (10200 total, 229.6 MiB)
4666 records/s, 119.1 MiB/s, 26.1 KiB/rec (12576 total, 290.2 MiB)
4249 records/s, 136.5 MiB/s, 32.9 KiB/rec (14724 total, 359.2 MiB)
3477 records/s, 147.2 MiB/s, 43.4 KiB/rec (16476 total, 433.4 MiB)
3669 records/s, 151.9 MiB/s, 42.4 KiB/rec (18318 total, 509.7 MiB)
3396 records/s, 148.8 MiB/s, 44.9 KiB/rec (20034 total, 584.9 MiB)
3593 records/s, 144.1 MiB/s, 41.1 KiB/rec (21837 total, 657.2 MiB)
3118 records/s, 147.7 MiB/s, 48.5 KiB/rec (23400 total, 731.3 MiB)
3559 records/s, 156.0 MiB/s, 44.9 KiB/rec (25182 total, 809.4 MiB)
3626 records/s, 154.8 MiB/s, 43.7 KiB/rec (27012 total, 887.5 MiB)
3928 records/s, 162.1 MiB/s, 42.3 KiB/rec (28977 total, 968.6 MiB)
3611 records/s, 160.5 MiB/s, 45.5 KiB/rec (30789 total, 1049.1 MiB)
3435 records/s, 161.2 MiB/s, 48.0 KiB/rec (32514 total, 1130.1 MiB)
3867 records/s, 145.7 MiB/s, 38.6 KiB/rec (34452 total, 1203.1 MiB)
3312 records/s, 158.5 MiB/s, 49.0 KiB/rec (36126 total, 1283.2 MiB)
3587 records/s, 154.6 MiB/s, 44.1 KiB/rec (37923 total, 1360.7 MiB)
3717 records/s, 163.6 MiB/s, 45.1 KiB/rec (39783 total, 1442.5 MiB)
3372 records/s, 155.8 MiB/s, 47.3 KiB/rec (41481 total, 1521.0 MiB)
3578 records/s, 156.2 MiB/s, 44.7 KiB/rec (43275 total, 1599.3 MiB)
3571 records/s, 158.7 MiB/s, 45.5 KiB/rec (45081 total, 1679.6 MiB)
3599 records/s, 155.4 MiB/s, 44.2 KiB/rec (46902 total, 1758.2 MiB)
3834 records/s, 167.8 MiB/s, 44.8 KiB/rec (48822 total, 1842.2 MiB)
3232 records/s, 160.4 MiB/s, 50.8 KiB/rec (50442 total, 1922.6 MiB)
3208 records/s, 146.9 MiB/s, 46.9 KiB/rec (52062 total, 1996.8 MiB)
3633 records/s, 161.6 MiB/s, 45.6 KiB/rec (53880 total, 2077.7 MiB)
3745 records/s, 156.3 MiB/s, 42.7 KiB/rec (55767 total, 2156.5 MiB)
3453 records/s, 148.2 MiB/s, 44.0 KiB/rec (57510 total, 2231.3 MiB)
3580 records/s, 148.4 MiB/s, 42.5 KiB/rec (59322 total, 2306.4 MiB)
3686 records/s, 149.6 MiB/s, 41.5 KiB/rec (61203 total, 2382.7 MiB)
3103 records/s, 161.1 MiB/s, 53.1 KiB/rec (62766 total, 2463.9 MiB)
3343 records/s, 143.1 MiB/s, 43.8 KiB/rec (64455 total, 2536.1 MiB)
3604 records/s, 155.7 MiB/s, 44.2 KiB/rec (66264 total, 2614.3 MiB)
3379 records/s, 160.0 MiB/s, 48.5 KiB/rec (67956 total, 2694.4 MiB)
3518 records/s, 149.7 MiB/s, 43.6 KiB/rec (69729 total, 2769.9 MiB)
3481 records/s, 160.7 MiB/s, 47.3 KiB/rec (71481 total, 2850.8 MiB)
3670 records/s, 166.1 MiB/s, 46.4 KiB/rec (73332 total, 2934.6 MiB)
2707 records/s, 168.4 MiB/s, 63.7 KiB/rec (74693 total, 3019.2 MiB)
3027 records/s, 167.5 MiB/s, 56.7 KiB/rec (76215 total, 3103.5 MiB)
2723 records/s, 161.3 MiB/s, 60.7 KiB/rec (77577 total, 3184.1 MiB)
2870 records/s, 158.0 MiB/s, 56.4 KiB/rec (79035 total, 3264.4 MiB)
2829 records/s, 166.4 MiB/s, 60.2 KiB/rec (80466 total, 3348.6 MiB)
3010 records/s, 159.2 MiB/s, 54.2 KiB/rec (81975 total, 3428.4 MiB)
2615 records/s, 172.9 MiB/s, 67.7 KiB/rec (83289 total, 3515.3 MiB)
2995 records/s, 174.3 MiB/s, 59.6 KiB/rec (84789 total, 3602.6 MiB)
3051 records/s, 162.7 MiB/s, 54.6 KiB/rec (86334 total, 3685.0 MiB)
2786 records/s, 155.3 MiB/s, 57.1 KiB/rec (87738 total, 3763.2 MiB)
2882 records/s, 172.7 MiB/s, 61.4 KiB/rec (89181 total, 3849.7 MiB)
2835 records/s, 161.1 MiB/s, 58.2 KiB/rec (90609 total, 3930.8 MiB)
2681 records/s, 174.3 MiB/s, 66.6 KiB/rec (91953 total, 4018.2 MiB)
2641 records/s, 162.2 MiB/s, 62.9 KiB/rec (93294 total, 4100.6 MiB)
2697 records/s, 165.9 MiB/s, 63.0 KiB/rec (94650 total, 4184.0 MiB)
2495 records/s, 161.9 MiB/s, 66.5 KiB/rec (95907 total, 4265.6 MiB)
2643 records/s, 151.0 MiB/s, 58.5 KiB/rec (97236 total, 4341.5 MiB)
3035 records/s, 164.1 MiB/s, 55.4 KiB/rec (98757 total, 4423.8 MiB)
3071 records/s, 159.9 MiB/s, 53.3 KiB/rec (100311 total, 4504.7 MiB)
3208 records/s, 174.5 MiB/s, 55.7 KiB/rec (101928 total, 4592.6 MiB)
2945 records/s, 168.2 MiB/s, 58.5 KiB/rec (103404 total, 4676.9 MiB)
3071 records/s, 159.7 MiB/s, 53.2 KiB/rec (104949 total, 4757.2 MiB)
2765 records/s, 161.9 MiB/s, 60.0 KiB/rec (106338 total, 4838.6 MiB)
2754 records/s, 162.8 MiB/s, 60.5 KiB/rec (107733 total, 4921.1 MiB)
2818 records/s, 183.6 MiB/s, 66.7 KiB/rec (109155 total, 5013.7 MiB)
2995 records/s, 171.6 MiB/s, 58.7 KiB/rec (110655 total, 5099.7 MiB)
2712 records/s, 166.5 MiB/s, 62.9 KiB/rec (112011 total, 5182.9 MiB)
3101 records/s, 173.9 MiB/s, 57.4 KiB/rec (113568 total, 5270.2 MiB)
Summary: 33.9s, 3371 records/s, 156.3 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
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
12870 records/s, 279.4 MiB/s, 22.2 KiB/rec (6450 total, 140.0 MiB)
12138 records/s, 298.3 MiB/s, 25.2 KiB/rec (12525 total, 289.3 MiB)
11755 records/s, 447.5 MiB/s, 39.0 KiB/rec (18414 total, 513.5 MiB)
11543 records/s, 500.1 MiB/s, 44.4 KiB/rec (24198 total, 764.1 MiB)
11882 records/s, 510.9 MiB/s, 44.0 KiB/rec (30153 total, 1020.1 MiB)
11936 records/s, 525.7 MiB/s, 45.1 KiB/rec (36126 total, 1283.2 MiB)
12658 records/s, 566.6 MiB/s, 45.8 KiB/rec (42495 total, 1568.3 MiB)
12848 records/s, 557.7 MiB/s, 44.4 KiB/rec (48930 total, 1847.6 MiB)
11740 records/s, 536.7 MiB/s, 46.8 KiB/rec (54816 total, 2116.7 MiB)
12436 records/s, 517.5 MiB/s, 42.6 KiB/rec (61047 total, 2376.0 MiB)
12384 records/s, 570.4 MiB/s, 47.2 KiB/rec (67251 total, 2661.7 MiB)
12616 records/s, 583.8 MiB/s, 47.4 KiB/rec (73569 total, 2954.1 MiB)
10962 records/s, 622.9 MiB/s, 58.2 KiB/rec (79065 total, 3266.4 MiB)
11144 records/s, 652.8 MiB/s, 60.0 KiB/rec (84645 total, 3593.2 MiB)
11539 records/s, 649.4 MiB/s, 57.6 KiB/rec (90420 total, 3918.2 MiB)
10741 records/s, 678.9 MiB/s, 64.7 KiB/rec (95805 total, 4258.6 MiB)
10652 records/s, 587.0 MiB/s, 56.4 KiB/rec (101136 total, 4552.3 MiB)
11240 records/s, 620.3 MiB/s, 56.5 KiB/rec (106758 total, 4862.6 MiB)
10909 records/s, 664.8 MiB/s, 62.4 KiB/rec (112218 total, 5195.3 MiB)
Summary: 9.7s, 11786 records/s, 546.5 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
4350 records/s, 101.7 MiB/s, 23.9 KiB/rec (2196 total, 51.3 MiB)
5505 records/s, 109.9 MiB/s, 20.4 KiB/rec (4974 total, 106.8 MiB)
4844 records/s, 117.9 MiB/s, 24.9 KiB/rec (7422 total, 166.4 MiB)
5768 records/s, 130.4 MiB/s, 23.2 KiB/rec (10335 total, 232.2 MiB)
4716 records/s, 122.3 MiB/s, 26.6 KiB/rec (12705 total, 293.7 MiB)
4153 records/s, 138.6 MiB/s, 34.2 KiB/rec (14796 total, 363.5 MiB)
3494 records/s, 144.3 MiB/s, 42.3 KiB/rec (16560 total, 436.4 MiB)
3575 records/s, 148.0 MiB/s, 42.4 KiB/rec (18351 total, 510.5 MiB)
3344 records/s, 147.8 MiB/s, 45.2 KiB/rec (20034 total, 584.9 MiB)
3527 records/s, 141.1 MiB/s, 41.0 KiB/rec (21810 total, 655.9 MiB)
3275 records/s, 154.2 MiB/s, 48.2 KiB/rec (23457 total, 733.5 MiB)
3430 records/s, 150.9 MiB/s, 45.0 KiB/rec (25182 total, 809.4 MiB)
3515 records/s, 150.3 MiB/s, 43.8 KiB/rec (26940 total, 884.5 MiB)
3750 records/s, 151.0 MiB/s, 41.2 KiB/rec (28833 total, 960.8 MiB)
3317 records/s, 147.4 MiB/s, 45.5 KiB/rec (30510 total, 1035.3 MiB)
3318 records/s, 157.3 MiB/s, 48.5 KiB/rec (32172 total, 1114.1 MiB)
3617 records/s, 152.3 MiB/s, 43.1 KiB/rec (33981 total, 1190.2 MiB)
3330 records/s, 144.8 MiB/s, 44.5 KiB/rec (35646 total, 1262.6 MiB)
3451 records/s, 149.5 MiB/s, 44.4 KiB/rec (37377 total, 1337.6 MiB)
3421 records/s, 148.4 MiB/s, 44.4 KiB/rec (39090 total, 1411.9 MiB)
3379 records/s, 147.7 MiB/s, 44.8 KiB/rec (40782 total, 1485.9 MiB)
3297 records/s, 159.2 MiB/s, 49.5 KiB/rec (42447 total, 1566.3 MiB)
3612 records/s, 150.3 MiB/s, 42.6 KiB/rec (44277 total, 1642.4 MiB)
3707 records/s, 164.0 MiB/s, 45.3 KiB/rec (46140 total, 1724.9 MiB)
3497 records/s, 155.8 MiB/s, 45.6 KiB/rec (47892 total, 1802.9 MiB)
3408 records/s, 151.4 MiB/s, 45.5 KiB/rec (49626 total, 1879.9 MiB)
3064 records/s, 150.6 MiB/s, 50.3 KiB/rec (51159 total, 1955.3 MiB)
3375 records/s, 154.7 MiB/s, 46.9 KiB/rec (52866 total, 2033.5 MiB)
3660 records/s, 156.6 MiB/s, 43.8 KiB/rec (54702 total, 2112.1 MiB)
3612 records/s, 153.8 MiB/s, 43.6 KiB/rec (56526 total, 2189.7 MiB)
3306 records/s, 151.0 MiB/s, 46.8 KiB/rec (58197 total, 2266.1 MiB)
3963 records/s, 149.5 MiB/s, 38.6 KiB/rec (60183 total, 2341.0 MiB)
3287 records/s, 152.0 MiB/s, 47.4 KiB/rec (61845 total, 2417.9 MiB)
3359 records/s, 151.5 MiB/s, 46.2 KiB/rec (63543 total, 2494.5 MiB)
3390 records/s, 148.5 MiB/s, 44.9 KiB/rec (65262 total, 2569.8 MiB)
3485 records/s, 163.6 MiB/s, 48.1 KiB/rec (67009 total, 2651.8 MiB)
3737 records/s, 154.6 MiB/s, 42.4 KiB/rec (68898 total, 2729.9 MiB)
3259 records/s, 156.6 MiB/s, 49.2 KiB/rec (70551 total, 2809.4 MiB)
3660 records/s, 158.9 MiB/s, 44.5 KiB/rec (72396 total, 2889.5 MiB)
2994 records/s, 160.8 MiB/s, 55.0 KiB/rec (73899 total, 2970.2 MiB)
2918 records/s, 169.2 MiB/s, 59.4 KiB/rec (75378 total, 3056.0 MiB)
2876 records/s, 170.9 MiB/s, 60.8 KiB/rec (76830 total, 3142.3 MiB)
3059 records/s, 162.5 MiB/s, 54.4 KiB/rec (78366 total, 3223.9 MiB)
2688 records/s, 167.1 MiB/s, 63.7 KiB/rec (79734 total, 3308.9 MiB)
3003 records/s, 161.2 MiB/s, 55.0 KiB/rec (81249 total, 3390.3 MiB)
2812 records/s, 167.7 MiB/s, 61.1 KiB/rec (82662 total, 3474.5 MiB)
2686 records/s, 164.2 MiB/s, 62.6 KiB/rec (84015 total, 3557.2 MiB)
3089 records/s, 168.5 MiB/s, 55.8 KiB/rec (85566 total, 3641.8 MiB)
2866 records/s, 163.2 MiB/s, 58.3 KiB/rec (87000 total, 3723.5 MiB)
2808 records/s, 161.6 MiB/s, 58.9 KiB/rec (88413 total, 3804.8 MiB)
2767 records/s, 161.7 MiB/s, 59.8 KiB/rec (89805 total, 3886.2 MiB)
2710 records/s, 160.5 MiB/s, 60.7 KiB/rec (91179 total, 3967.6 MiB)
2581 records/s, 164.5 MiB/s, 65.3 KiB/rec (92472 total, 4050.0 MiB)
2718 records/s, 155.8 MiB/s, 58.7 KiB/rec (93840 total, 4128.4 MiB)
2300 records/s, 160.1 MiB/s, 71.3 KiB/rec (94995 total, 4208.8 MiB)
2373 records/s, 145.7 MiB/s, 62.9 KiB/rec (96192 total, 4282.3 MiB)
2896 records/s, 160.4 MiB/s, 56.7 KiB/rec (97659 total, 4363.5 MiB)
2979 records/s, 158.2 MiB/s, 54.4 KiB/rec (99153 total, 4442.9 MiB)
2937 records/s, 162.2 MiB/s, 56.5 KiB/rec (100626 total, 4524.2 MiB)
2999 records/s, 155.6 MiB/s, 53.1 KiB/rec (102135 total, 4602.5 MiB)
2630 records/s, 156.6 MiB/s, 61.0 KiB/rec (103458 total, 4681.3 MiB)
2978 records/s, 151.7 MiB/s, 52.2 KiB/rec (104949 total, 4757.2 MiB)
2901 records/s, 168.6 MiB/s, 59.5 KiB/rec (106416 total, 4842.5 MiB)
2868 records/s, 168.0 MiB/s, 60.0 KiB/rec (107856 total, 4926.9 MiB)
2674 records/s, 178.6 MiB/s, 68.4 KiB/rec (109194 total, 5016.2 MiB)
2903 records/s, 167.2 MiB/s, 59.0 KiB/rec (110658 total, 5100.6 MiB)
2784 records/s, 169.3 MiB/s, 62.3 KiB/rec (112062 total, 5186.0 MiB)
3008 records/s, 168.3 MiB/s, 57.3 KiB/rec (113568 total, 5270.2 MiB)
Summary: 34.4s, 3320 records/s, 154.0 MiB/s, 47.5 KiB/rec (114273 total, 5298.4 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

Unsupported.
