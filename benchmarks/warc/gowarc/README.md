# Benchmark: gowarc

Benchmark for [gowarc](https://github.com/internetarchive/gowarc).

## Install Dependencies:

```bash
sudo apt install golang
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
29786 records/s, 734.2 MiB/s, 25.2 KiB/rec (14895 total, 367.2 MiB)
18685 records/s, 797.0 MiB/s, 43.7 KiB/rec (24264 total, 766.8 MiB)
19381 records/s, 844.6 MiB/s, 44.6 KiB/rec (33981 total, 1190.2 MiB)
18740 records/s, 825.1 MiB/s, 45.1 KiB/rec (43353 total, 1602.9 MiB)
18447 records/s, 835.3 MiB/s, 46.4 KiB/rec (52602 total, 2021.7 MiB)
20055 records/s, 870.2 MiB/s, 44.4 KiB/rec (62634 total, 2456.9 MiB)
18355 records/s, 813.7 MiB/s, 45.4 KiB/rec (71832 total, 2864.7 MiB)
15876 records/s, 890.0 MiB/s, 57.4 KiB/rec (79770 total, 3309.7 MiB)
17054 records/s, 972.8 MiB/s, 58.4 KiB/rec (88311 total, 3796.9 MiB)
16427 records/s, 1012.4 MiB/s, 63.1 KiB/rec (96528 total, 4303.3 MiB)
17833 records/s, 963.3 MiB/s, 55.3 KiB/rec (105459 total, 4785.8 MiB)
15654 records/s, 939.9 MiB/s, 61.5 KiB/rec (113286 total, 5255.7 MiB)
Summary: 6.0s, 18898 records/s, 876.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
5860 records/s, 127.9 MiB/s, 22.4 KiB/rec (3036 total, 66.3 MiB)
5497 records/s, 121.1 MiB/s, 22.6 KiB/rec (5787 total, 126.9 MiB)
4794 records/s, 117.6 MiB/s, 25.1 KiB/rec (8184 total, 185.7 MiB)
5130 records/s, 114.5 MiB/s, 22.9 KiB/rec (10749 total, 243.0 MiB)
4910 records/s, 127.7 MiB/s, 26.6 KiB/rec (13204 total, 306.8 MiB)
4135 records/s, 150.5 MiB/s, 37.3 KiB/rec (15273 total, 382.1 MiB)
3189 records/s, 130.2 MiB/s, 41.8 KiB/rec (16869 total, 447.2 MiB)
3341 records/s, 143.4 MiB/s, 44.0 KiB/rec (18540 total, 519.0 MiB)
3622 records/s, 154.7 MiB/s, 43.7 KiB/rec (20352 total, 596.3 MiB)
3132 records/s, 129.4 MiB/s, 42.3 KiB/rec (21918 total, 661.1 MiB)
2788 records/s, 133.3 MiB/s, 48.9 KiB/rec (23331 total, 728.6 MiB)
3479 records/s, 150.8 MiB/s, 44.4 KiB/rec (25071 total, 804.0 MiB)
3553 records/s, 155.2 MiB/s, 44.7 KiB/rec (26850 total, 881.7 MiB)
3703 records/s, 146.9 MiB/s, 40.6 KiB/rec (28704 total, 955.2 MiB)
3274 records/s, 144.6 MiB/s, 45.2 KiB/rec (30342 total, 1027.6 MiB)
3143 records/s, 151.2 MiB/s, 49.3 KiB/rec (31914 total, 1103.2 MiB)
3407 records/s, 152.6 MiB/s, 45.9 KiB/rec (33621 total, 1179.7 MiB)
3885 records/s, 159.8 MiB/s, 42.1 KiB/rec (35565 total, 1259.6 MiB)
3424 records/s, 147.7 MiB/s, 44.2 KiB/rec (37278 total, 1333.5 MiB)
3641 records/s, 159.2 MiB/s, 44.8 KiB/rec (39099 total, 1413.1 MiB)
3314 records/s, 142.5 MiB/s, 44.0 KiB/rec (40758 total, 1484.4 MiB)
3096 records/s, 152.1 MiB/s, 50.3 KiB/rec (42309 total, 1560.6 MiB)
3751 records/s, 153.2 MiB/s, 41.8 KiB/rec (44187 total, 1637.3 MiB)
3317 records/s, 142.5 MiB/s, 44.0 KiB/rec (45846 total, 1708.6 MiB)
3349 records/s, 147.0 MiB/s, 45.0 KiB/rec (47526 total, 1782.3 MiB)
3418 records/s, 157.7 MiB/s, 47.3 KiB/rec (49236 total, 1861.2 MiB)
3189 records/s, 159.8 MiB/s, 51.3 KiB/rec (50835 total, 1941.4 MiB)
3215 records/s, 146.3 MiB/s, 46.6 KiB/rec (52443 total, 2014.6 MiB)
3611 records/s, 157.0 MiB/s, 44.5 KiB/rec (54249 total, 2093.1 MiB)
3665 records/s, 150.6 MiB/s, 42.1 KiB/rec (56088 total, 2168.7 MiB)
3014 records/s, 137.0 MiB/s, 46.6 KiB/rec (57600 total, 2237.4 MiB)
3545 records/s, 144.1 MiB/s, 41.6 KiB/rec (59373 total, 2309.5 MiB)
3474 records/s, 139.9 MiB/s, 41.3 KiB/rec (61110 total, 2379.5 MiB)
2950 records/s, 151.1 MiB/s, 52.4 KiB/rec (62586 total, 2455.1 MiB)
3650 records/s, 158.7 MiB/s, 44.5 KiB/rec (64413 total, 2534.5 MiB)
3449 records/s, 147.8 MiB/s, 43.9 KiB/rec (66138 total, 2608.5 MiB)
3352 records/s, 155.8 MiB/s, 47.6 KiB/rec (67815 total, 2686.4 MiB)
3633 records/s, 158.2 MiB/s, 44.6 KiB/rec (69633 total, 2765.6 MiB)
3102 records/s, 145.3 MiB/s, 48.0 KiB/rec (71184 total, 2838.2 MiB)
3473 records/s, 145.4 MiB/s, 42.9 KiB/rec (72921 total, 2911.0 MiB)
2801 records/s, 172.0 MiB/s, 62.9 KiB/rec (74322 total, 2997.0 MiB)
2838 records/s, 159.6 MiB/s, 57.6 KiB/rec (75741 total, 3076.8 MiB)
2816 records/s, 168.0 MiB/s, 61.1 KiB/rec (77149 total, 3160.8 MiB)
2813 records/s, 147.5 MiB/s, 53.7 KiB/rec (78570 total, 3235.3 MiB)
2609 records/s, 155.0 MiB/s, 60.9 KiB/rec (79875 total, 3312.8 MiB)
2709 records/s, 153.1 MiB/s, 57.9 KiB/rec (81231 total, 3389.5 MiB)
2723 records/s, 159.0 MiB/s, 59.8 KiB/rec (82593 total, 3469.0 MiB)
2566 records/s, 156.6 MiB/s, 62.5 KiB/rec (83876 total, 3547.3 MiB)
2953 records/s, 162.6 MiB/s, 56.4 KiB/rec (85353 total, 3628.6 MiB)
2766 records/s, 156.1 MiB/s, 57.8 KiB/rec (86748 total, 3707.3 MiB)
2831 records/s, 161.2 MiB/s, 58.3 KiB/rec (88164 total, 3787.9 MiB)
2813 records/s, 167.1 MiB/s, 60.8 KiB/rec (89577 total, 3871.9 MiB)
2640 records/s, 156.6 MiB/s, 60.7 KiB/rec (90897 total, 3950.2 MiB)
2417 records/s, 156.6 MiB/s, 66.3 KiB/rec (92106 total, 4028.5 MiB)
2550 records/s, 152.7 MiB/s, 61.3 KiB/rec (93381 total, 4104.8 MiB)
2669 records/s, 164.5 MiB/s, 63.1 KiB/rec (94716 total, 4187.1 MiB)
2226 records/s, 147.3 MiB/s, 67.8 KiB/rec (95835 total, 4261.1 MiB)
2490 records/s, 145.0 MiB/s, 59.6 KiB/rec (97080 total, 4333.7 MiB)
2908 records/s, 153.0 MiB/s, 53.9 KiB/rec (98535 total, 4410.2 MiB)
2968 records/s, 154.9 MiB/s, 53.4 KiB/rec (100019 total, 4487.7 MiB)
2785 records/s, 158.7 MiB/s, 58.3 KiB/rec (101412 total, 4567.0 MiB)
2844 records/s, 151.1 MiB/s, 54.4 KiB/rec (102834 total, 4642.6 MiB)
2705 records/s, 149.8 MiB/s, 56.7 KiB/rec (104190 total, 4717.7 MiB)
2993 records/s, 158.8 MiB/s, 54.3 KiB/rec (105690 total, 4797.3 MiB)
2672 records/s, 159.0 MiB/s, 60.9 KiB/rec (107028 total, 4876.9 MiB)
2480 records/s, 160.4 MiB/s, 66.2 KiB/rec (108276 total, 4957.6 MiB)
2547 records/s, 165.1 MiB/s, 66.4 KiB/rec (109557 total, 5040.6 MiB)
2919 records/s, 171.2 MiB/s, 60.1 KiB/rec (111027 total, 5126.9 MiB)
2843 records/s, 159.7 MiB/s, 57.5 KiB/rec (112449 total, 5206.7 MiB)
3100 records/s, 161.4 MiB/s, 53.3 KiB/rec (114000 total, 5287.5 MiB)
Summary: 35.1s, 3252 records/s, 150.8 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
19964 records/s, 451.1 MiB/s, 23.1 KiB/rec (9982 total, 225.5 MiB)
15771 records/s, 525.4 MiB/s, 34.1 KiB/rec (17868 total, 488.3 MiB)
13041 records/s, 566.0 MiB/s, 44.4 KiB/rec (24390 total, 771.3 MiB)
13709 records/s, 596.3 MiB/s, 44.5 KiB/rec (31245 total, 1069.5 MiB)
13439 records/s, 588.1 MiB/s, 44.8 KiB/rec (37965 total, 1363.6 MiB)
11504 records/s, 508.5 MiB/s, 45.3 KiB/rec (43717 total, 1617.8 MiB)
14044 records/s, 636.5 MiB/s, 46.4 KiB/rec (50739 total, 1936.1 MiB)
12154 records/s, 532.7 MiB/s, 44.9 KiB/rec (56820 total, 2202.6 MiB)
13182 records/s, 575.1 MiB/s, 44.7 KiB/rec (63411 total, 2490.2 MiB)
13078 records/s, 581.1 MiB/s, 45.5 KiB/rec (69954 total, 2780.9 MiB)
10535 records/s, 530.6 MiB/s, 51.6 KiB/rec (75222 total, 3046.2 MiB)
10281 records/s, 593.4 MiB/s, 59.1 KiB/rec (80364 total, 3342.9 MiB)
11246 records/s, 647.5 MiB/s, 59.0 KiB/rec (85992 total, 3667.0 MiB)
11318 records/s, 671.5 MiB/s, 60.8 KiB/rec (91651 total, 4002.7 MiB)
10431 records/s, 633.1 MiB/s, 62.2 KiB/rec (96867 total, 4319.3 MiB)
11332 records/s, 613.5 MiB/s, 55.4 KiB/rec (102534 total, 4626.1 MiB)
11387 records/s, 655.4 MiB/s, 58.9 KiB/rec (108237 total, 4954.4 MiB)
10780 records/s, 637.5 MiB/s, 60.6 KiB/rec (113628 total, 5273.2 MiB)
Summary: 9.1s, 12616 records/s, 584.9 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
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
59548 records/s, 2000.5 MiB/s, 34.4 KiB/rec (29774 total, 1000.2 MiB)
63426 records/s, 2791.9 MiB/s, 45.1 KiB/rec (61487 total, 2396.2 MiB)
54649 records/s, 2862.7 MiB/s, 53.6 KiB/rec (88815 total, 3827.8 MiB)
Summary: 2.0s, 57356 records/s, 2659.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
6116 records/s, 133.9 MiB/s, 22.4 KiB/rec (3058 total, 67.0 MiB)
5835 records/s, 127.5 MiB/s, 22.4 KiB/rec (5976 total, 130.7 MiB)
5490 records/s, 135.4 MiB/s, 25.3 KiB/rec (8724 total, 198.5 MiB)
5471 records/s, 126.2 MiB/s, 23.6 KiB/rec (11496 total, 262.4 MiB)
5343 records/s, 131.8 MiB/s, 25.3 KiB/rec (14169 total, 328.3 MiB)
3422 records/s, 154.9 MiB/s, 46.4 KiB/rec (15882 total, 405.9 MiB)
3502 records/s, 147.3 MiB/s, 43.1 KiB/rec (17634 total, 479.6 MiB)
3420 records/s, 148.7 MiB/s, 44.5 KiB/rec (19359 total, 554.6 MiB)
3743 records/s, 159.4 MiB/s, 43.6 KiB/rec (21231 total, 634.3 MiB)
3502 records/s, 153.4 MiB/s, 44.9 KiB/rec (22982 total, 711.0 MiB)
3079 records/s, 136.0 MiB/s, 45.2 KiB/rec (24522 total, 779.0 MiB)
3548 records/s, 162.4 MiB/s, 46.9 KiB/rec (26298 total, 860.3 MiB)
3785 records/s, 156.7 MiB/s, 42.4 KiB/rec (28191 total, 938.7 MiB)
3593 records/s, 145.1 MiB/s, 41.3 KiB/rec (29988 total, 1011.3 MiB)
3318 records/s, 154.2 MiB/s, 47.6 KiB/rec (31647 total, 1088.4 MiB)
3284 records/s, 154.7 MiB/s, 48.2 KiB/rec (33291 total, 1165.8 MiB)
3928 records/s, 157.0 MiB/s, 40.9 KiB/rec (35259 total, 1244.5 MiB)
3407 records/s, 151.9 MiB/s, 45.6 KiB/rec (36966 total, 1320.6 MiB)
3601 records/s, 151.1 MiB/s, 43.0 KiB/rec (38769 total, 1396.2 MiB)
3438 records/s, 148.5 MiB/s, 44.2 KiB/rec (40491 total, 1470.6 MiB)
3140 records/s, 156.7 MiB/s, 51.1 KiB/rec (42061 total, 1548.9 MiB)
3952 records/s, 162.5 MiB/s, 42.1 KiB/rec (44037 total, 1630.2 MiB)
3429 records/s, 148.3 MiB/s, 44.3 KiB/rec (45753 total, 1704.4 MiB)
3617 records/s, 159.8 MiB/s, 45.2 KiB/rec (47562 total, 1784.4 MiB)
3640 records/s, 164.8 MiB/s, 46.4 KiB/rec (49386 total, 1867.0 MiB)
2865 records/s, 148.9 MiB/s, 53.2 KiB/rec (50838 total, 1942.4 MiB)
3369 records/s, 151.1 MiB/s, 45.9 KiB/rec (52536 total, 2018.6 MiB)
3785 records/s, 166.2 MiB/s, 45.0 KiB/rec (54429 total, 2101.7 MiB)
3523 records/s, 145.1 MiB/s, 42.2 KiB/rec (56193 total, 2174.4 MiB)
3425 records/s, 158.3 MiB/s, 47.3 KiB/rec (57906 total, 2253.6 MiB)
3845 records/s, 148.3 MiB/s, 39.5 KiB/rec (59829 total, 2327.7 MiB)
3389 records/s, 141.6 MiB/s, 42.8 KiB/rec (61527 total, 2398.7 MiB)
3207 records/s, 162.3 MiB/s, 51.8 KiB/rec (63132 total, 2479.9 MiB)
3713 records/s, 155.0 MiB/s, 42.7 KiB/rec (64989 total, 2557.4 MiB)
3429 records/s, 156.7 MiB/s, 46.8 KiB/rec (66708 total, 2635.9 MiB)
3523 records/s, 154.7 MiB/s, 45.0 KiB/rec (68475 total, 2713.5 MiB)
2963 records/s, 134.8 MiB/s, 46.6 KiB/rec (69957 total, 2781.0 MiB)
3381 records/s, 151.5 MiB/s, 45.9 KiB/rec (71649 total, 2856.8 MiB)
3557 records/s, 168.4 MiB/s, 48.5 KiB/rec (73428 total, 2941.0 MiB)
2762 records/s, 164.5 MiB/s, 61.0 KiB/rec (74814 total, 3023.5 MiB)
2875 records/s, 164.2 MiB/s, 58.5 KiB/rec (76254 total, 3105.8 MiB)
2724 records/s, 163.4 MiB/s, 61.4 KiB/rec (77616 total, 3187.5 MiB)
3010 records/s, 162.3 MiB/s, 55.2 KiB/rec (79122 total, 3268.7 MiB)
2489 records/s, 150.2 MiB/s, 61.8 KiB/rec (80370 total, 3344.0 MiB)
3016 records/s, 158.9 MiB/s, 53.9 KiB/rec (81879 total, 3423.5 MiB)
2516 records/s, 166.2 MiB/s, 67.6 KiB/rec (83139 total, 3506.7 MiB)
3089 records/s, 178.9 MiB/s, 59.3 KiB/rec (84684 total, 3596.2 MiB)
2992 records/s, 161.2 MiB/s, 55.2 KiB/rec (86187 total, 3677.2 MiB)
2810 records/s, 154.3 MiB/s, 56.2 KiB/rec (87594 total, 3754.5 MiB)
2801 records/s, 166.8 MiB/s, 61.0 KiB/rec (88998 total, 3838.1 MiB)
2666 records/s, 151.6 MiB/s, 58.2 KiB/rec (90331 total, 3913.9 MiB)
2433 records/s, 162.4 MiB/s, 68.4 KiB/rec (91563 total, 3996.1 MiB)
2510 records/s, 148.0 MiB/s, 60.4 KiB/rec (92823 total, 4070.4 MiB)
2827 records/s, 165.4 MiB/s, 59.9 KiB/rec (94239 total, 4153.2 MiB)
2349 records/s, 160.6 MiB/s, 70.0 KiB/rec (95415 total, 4233.6 MiB)
2510 records/s, 153.6 MiB/s, 62.6 KiB/rec (96672 total, 4310.5 MiB)
3065 records/s, 162.8 MiB/s, 54.4 KiB/rec (98205 total, 4392.0 MiB)
2775 records/s, 146.3 MiB/s, 54.0 KiB/rec (99594 total, 4465.2 MiB)
2809 records/s, 161.7 MiB/s, 58.9 KiB/rec (101013 total, 4546.8 MiB)
2891 records/s, 152.3 MiB/s, 53.9 KiB/rec (102462 total, 4623.2 MiB)
2698 records/s, 148.9 MiB/s, 56.5 KiB/rec (103812 total, 4697.7 MiB)
2915 records/s, 158.6 MiB/s, 55.7 KiB/rec (105276 total, 4777.4 MiB)
2709 records/s, 151.5 MiB/s, 57.3 KiB/rec (106644 total, 4853.9 MiB)
2527 records/s, 155.4 MiB/s, 63.0 KiB/rec (107916 total, 4932.1 MiB)
2578 records/s, 169.0 MiB/s, 67.1 KiB/rec (109206 total, 5016.6 MiB)
2723 records/s, 158.3 MiB/s, 59.5 KiB/rec (110568 total, 5095.8 MiB)
2713 records/s, 168.1 MiB/s, 63.4 KiB/rec (111927 total, 5180.0 MiB)
2771 records/s, 153.6 MiB/s, 56.8 KiB/rec (113313 total, 5256.9 MiB)
Summary: 34.3s, 3327 records/s, 154.3 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
23098 records/s, 527.9 MiB/s, 23.4 KiB/rec (11550 total, 264.0 MiB)
16042 records/s, 604.0 MiB/s, 38.6 KiB/rec (19602 total, 567.1 MiB)
15481 records/s, 665.0 MiB/s, 44.0 KiB/rec (27345 total, 899.7 MiB)
14964 records/s, 654.2 MiB/s, 44.8 KiB/rec (34827 total, 1226.8 MiB)
14368 records/s, 636.3 MiB/s, 45.3 KiB/rec (42011 total, 1545.0 MiB)
14223 records/s, 623.2 MiB/s, 44.9 KiB/rec (49125 total, 1856.7 MiB)
13852 records/s, 620.3 MiB/s, 45.9 KiB/rec (56052 total, 2166.9 MiB)
12718 records/s, 561.6 MiB/s, 45.2 KiB/rec (62411 total, 2447.7 MiB)
13884 records/s, 612.2 MiB/s, 45.2 KiB/rec (69354 total, 2753.9 MiB)
13579 records/s, 688.8 MiB/s, 51.9 KiB/rec (76146 total, 3098.4 MiB)
12362 records/s, 698.4 MiB/s, 57.9 KiB/rec (82329 total, 3447.7 MiB)
11777 records/s, 685.4 MiB/s, 59.6 KiB/rec (88218 total, 3790.5 MiB)
11373 records/s, 682.5 MiB/s, 61.4 KiB/rec (93906 total, 4131.8 MiB)
11380 records/s, 666.7 MiB/s, 60.0 KiB/rec (99596 total, 4465.2 MiB)
12204 records/s, 664.5 MiB/s, 55.8 KiB/rec (105699 total, 4797.5 MiB)
11214 records/s, 701.0 MiB/s, 64.0 KiB/rec (111306 total, 5148.0 MiB)
Summary: 8.2s, 13891 records/s, 644.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### LZ4:

Unsupported.
