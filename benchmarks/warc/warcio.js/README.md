# Benchmark: warcpp

Benchmark for [warcpp](https://github.com/pisa-engine/warcpp).

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
27350 records/s, 630.2 MiB/s, 23.6 KiB/rec (13675 total, 315.1 MiB)
20214 records/s, 862.7 MiB/s, 43.7 KiB/rec (23782 total, 746.4 MiB)
20296 records/s, 885.2 MiB/s, 44.7 KiB/rec (33930 total, 1189.0 MiB)
20940 records/s, 918.6 MiB/s, 44.9 KiB/rec (44412 total, 1648.9 MiB)
21804 records/s, 974.9 MiB/s, 45.8 KiB/rec (55314 total, 2136.3 MiB)
22307 records/s, 972.8 MiB/s, 44.7 KiB/rec (66480 total, 2623.3 MiB)
20104 records/s, 1001.6 MiB/s, 51.0 KiB/rec (76560 total, 3125.5 MiB)
18260 records/s, 1051.1 MiB/s, 58.9 KiB/rec (85740 total, 3653.9 MiB)
18173 records/s, 1084.5 MiB/s, 61.1 KiB/rec (94827 total, 4196.2 MiB)
18524 records/s, 1035.5 MiB/s, 57.2 KiB/rec (104089 total, 4714.0 MiB)
17401 records/s, 1036.8 MiB/s, 61.0 KiB/rec (112800 total, 5233.0 MiB)
Summary: 5.6s, 20532 records/s, 952.0 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
2203 records/s, 49.7 MiB/s, 23.1 KiB/rec (1104 total, 24.9 MiB)
3990 records/s, 84.0 MiB/s, 21.6 KiB/rec (3099 total, 66.9 MiB)
4301 records/s, 86.5 MiB/s, 20.6 KiB/rec (5250 total, 110.2 MiB)
4204 records/s, 102.3 MiB/s, 24.9 KiB/rec (7374 total, 161.8 MiB)
4361 records/s, 106.8 MiB/s, 25.1 KiB/rec (9555 total, 215.3 MiB)
4353 records/s, 100.8 MiB/s, 23.7 KiB/rec (11733 total, 265.7 MiB)
4532 records/s, 106.3 MiB/s, 24.0 KiB/rec (14007 total, 319.0 MiB)
2664 records/s, 122.0 MiB/s, 46.9 KiB/rec (15339 total, 380.1 MiB)
2798 records/s, 117.5 MiB/s, 43.0 KiB/rec (16740 total, 438.9 MiB)
3016 records/s, 120.4 MiB/s, 40.9 KiB/rec (18255 total, 499.4 MiB)
2677 records/s, 122.6 MiB/s, 46.9 KiB/rec (19602 total, 561.1 MiB)
2676 records/s, 109.3 MiB/s, 41.8 KiB/rec (20940 total, 615.7 MiB)
2887 records/s, 118.9 MiB/s, 42.2 KiB/rec (22386 total, 675.3 MiB)
2603 records/s, 118.2 MiB/s, 46.5 KiB/rec (23691 total, 734.6 MiB)
2760 records/s, 122.3 MiB/s, 45.4 KiB/rec (25071 total, 795.7 MiB)
2861 records/s, 123.9 MiB/s, 44.4 KiB/rec (26502 total, 857.7 MiB)
2849 records/s, 122.1 MiB/s, 43.9 KiB/rec (27939 total, 919.3 MiB)
3071 records/s, 117.4 MiB/s, 39.1 KiB/rec (29478 total, 978.1 MiB)
2717 records/s, 124.5 MiB/s, 46.9 KiB/rec (30837 total, 1040.4 MiB)
2359 records/s, 114.1 MiB/s, 49.5 KiB/rec (32019 total, 1097.6 MiB)
2707 records/s, 118.2 MiB/s, 44.7 KiB/rec (33375 total, 1156.8 MiB)
2977 records/s, 118.1 MiB/s, 40.6 KiB/rec (34866 total, 1215.9 MiB)
2769 records/s, 119.2 MiB/s, 44.1 KiB/rec (36255 total, 1275.8 MiB)
2788 records/s, 118.9 MiB/s, 43.7 KiB/rec (37650 total, 1335.3 MiB)
2761 records/s, 117.0 MiB/s, 43.4 KiB/rec (39045 total, 1394.3 MiB)
2760 records/s, 115.9 MiB/s, 43.0 KiB/rec (40425 total, 1452.3 MiB)
2462 records/s, 122.0 MiB/s, 50.7 KiB/rec (41656 total, 1513.3 MiB)
2718 records/s, 119.6 MiB/s, 45.1 KiB/rec (43015 total, 1573.1 MiB)
2810 records/s, 120.8 MiB/s, 44.0 KiB/rec (44421 total, 1633.6 MiB)
2936 records/s, 121.2 MiB/s, 42.3 KiB/rec (45891 total, 1694.2 MiB)
2855 records/s, 121.1 MiB/s, 43.5 KiB/rec (47319 total, 1754.8 MiB)
2596 records/s, 126.5 MiB/s, 49.9 KiB/rec (48621 total, 1818.3 MiB)
2806 records/s, 120.7 MiB/s, 44.1 KiB/rec (50025 total, 1878.7 MiB)
2459 records/s, 120.7 MiB/s, 50.2 KiB/rec (51255 total, 1939.0 MiB)
2638 records/s, 122.0 MiB/s, 47.4 KiB/rec (52574 total, 2000.1 MiB)
2624 records/s, 115.3 MiB/s, 45.0 KiB/rec (53886 total, 2057.7 MiB)
2669 records/s, 107.3 MiB/s, 41.2 KiB/rec (55221 total, 2111.4 MiB)
2600 records/s, 113.3 MiB/s, 44.6 KiB/rec (56521 total, 2168.0 MiB)
2456 records/s, 108.7 MiB/s, 45.3 KiB/rec (57753 total, 2222.6 MiB)
2895 records/s, 114.2 MiB/s, 40.4 KiB/rec (59202 total, 2279.7 MiB)
3013 records/s, 121.1 MiB/s, 41.2 KiB/rec (60711 total, 2340.4 MiB)
2433 records/s, 120.2 MiB/s, 50.6 KiB/rec (61932 total, 2400.7 MiB)
2668 records/s, 120.0 MiB/s, 46.1 KiB/rec (63266 total, 2460.7 MiB)
2888 records/s, 119.1 MiB/s, 42.2 KiB/rec (64713 total, 2520.3 MiB)
2712 records/s, 117.7 MiB/s, 44.4 KiB/rec (66081 total, 2579.7 MiB)
2547 records/s, 122.6 MiB/s, 49.3 KiB/rec (67356 total, 2641.1 MiB)
2893 records/s, 116.3 MiB/s, 41.2 KiB/rec (68820 total, 2700.0 MiB)
2561 records/s, 122.5 MiB/s, 49.0 KiB/rec (70101 total, 2761.2 MiB)
2697 records/s, 121.4 MiB/s, 46.1 KiB/rec (71457 total, 2822.2 MiB)
2817 records/s, 114.8 MiB/s, 41.7 KiB/rec (72867 total, 2879.7 MiB)
2163 records/s, 129.0 MiB/s, 61.1 KiB/rec (73950 total, 2944.2 MiB)
2307 records/s, 129.3 MiB/s, 57.4 KiB/rec (75105 total, 3009.0 MiB)
2256 records/s, 130.9 MiB/s, 59.4 KiB/rec (76233 total, 3074.4 MiB)
2138 records/s, 126.3 MiB/s, 60.5 KiB/rec (77304 total, 3137.7 MiB)
2332 records/s, 122.2 MiB/s, 53.6 KiB/rec (78474 total, 3199.0 MiB)
2094 records/s, 127.2 MiB/s, 62.2 KiB/rec (79521 total, 3262.6 MiB)
2234 records/s, 127.3 MiB/s, 58.3 KiB/rec (80652 total, 3327.0 MiB)
2389 records/s, 122.8 MiB/s, 52.6 KiB/rec (81849 total, 3388.5 MiB)
1920 records/s, 132.3 MiB/s, 70.6 KiB/rec (82809 total, 3454.7 MiB)
2241 records/s, 126.0 MiB/s, 57.6 KiB/rec (83931 total, 3517.8 MiB)
2370 records/s, 132.0 MiB/s, 57.0 KiB/rec (85116 total, 3583.8 MiB)
2275 records/s, 127.6 MiB/s, 57.4 KiB/rec (86259 total, 3647.9 MiB)
2244 records/s, 128.3 MiB/s, 58.6 KiB/rec (87384 total, 3712.2 MiB)
2274 records/s, 129.7 MiB/s, 58.4 KiB/rec (88521 total, 3777.1 MiB)
2297 records/s, 127.9 MiB/s, 57.0 KiB/rec (89670 total, 3841.1 MiB)
2148 records/s, 125.1 MiB/s, 59.6 KiB/rec (90750 total, 3904.0 MiB)
1886 records/s, 131.5 MiB/s, 71.4 KiB/rec (91695 total, 3969.8 MiB)
2127 records/s, 122.2 MiB/s, 58.8 KiB/rec (92760 total, 4031.0 MiB)
2293 records/s, 127.5 MiB/s, 57.0 KiB/rec (93912 total, 4095.1 MiB)
1834 records/s, 128.8 MiB/s, 71.9 KiB/rec (94839 total, 4160.2 MiB)
1993 records/s, 126.1 MiB/s, 64.8 KiB/rec (95853 total, 4224.4 MiB)
2218 records/s, 124.7 MiB/s, 57.6 KiB/rec (96963 total, 4286.8 MiB)
2304 records/s, 122.1 MiB/s, 54.3 KiB/rec (98115 total, 4347.8 MiB)
2299 records/s, 121.6 MiB/s, 54.2 KiB/rec (99276 total, 4409.2 MiB)
2386 records/s, 126.3 MiB/s, 54.2 KiB/rec (100470 total, 4472.4 MiB)
2282 records/s, 125.6 MiB/s, 56.4 KiB/rec (101613 total, 4535.3 MiB)
2299 records/s, 120.8 MiB/s, 53.8 KiB/rec (102765 total, 4595.9 MiB)
2114 records/s, 121.3 MiB/s, 58.8 KiB/rec (103822 total, 4656.5 MiB)
2326 records/s, 121.2 MiB/s, 53.4 KiB/rec (104985 total, 4717.1 MiB)
2243 records/s, 128.7 MiB/s, 58.7 KiB/rec (106107 total, 4781.5 MiB)
2221 records/s, 125.1 MiB/s, 57.7 KiB/rec (107223 total, 4844.4 MiB)
1970 records/s, 127.9 MiB/s, 66.5 KiB/rec (108213 total, 4908.7 MiB)
2038 records/s, 130.3 MiB/s, 65.4 KiB/rec (109233 total, 4973.8 MiB)
2133 records/s, 129.1 MiB/s, 62.0 KiB/rec (110304 total, 5038.7 MiB)
2046 records/s, 131.0 MiB/s, 65.6 KiB/rec (111327 total, 5104.2 MiB)
2330 records/s, 121.4 MiB/s, 53.3 KiB/rec (112497 total, 5165.2 MiB)
2242 records/s, 122.8 MiB/s, 56.1 KiB/rec (113625 total, 5226.9 MiB)
Summary: 43.8s, 2607 records/s, 119.8 MiB/s, 47.1 KiB/rec (114274 total, 5252.1 MiB)
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
53349 records/s, 1748.8 MiB/s, 33.6 KiB/rec (26682 total, 874.6 MiB)
45974 records/s, 2012.1 MiB/s, 44.8 KiB/rec (49669 total, 1880.7 MiB)
57131 records/s, 2671.2 MiB/s, 47.9 KiB/rec (78237 total, 3216.4 MiB)
55628 records/s, 3209.4 MiB/s, 59.1 KiB/rec (106053 total, 4821.3 MiB)
Summary: 2.1s, 53188 records/s, 2466.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
2230 records/s, 51.3 MiB/s, 23.6 KiB/rec (1116 total, 25.7 MiB)
4108 records/s, 84.6 MiB/s, 21.1 KiB/rec (3171 total, 68.0 MiB)
4336 records/s, 88.6 MiB/s, 20.9 KiB/rec (5339 total, 112.3 MiB)
4141 records/s, 105.3 MiB/s, 26.0 KiB/rec (7425 total, 165.3 MiB)
4354 records/s, 102.1 MiB/s, 24.0 KiB/rec (9602 total, 216.4 MiB)
4293 records/s, 99.4 MiB/s, 23.7 KiB/rec (11751 total, 266.2 MiB)
4395 records/s, 101.3 MiB/s, 23.6 KiB/rec (13950 total, 316.9 MiB)
2654 records/s, 122.4 MiB/s, 47.2 KiB/rec (15282 total, 378.3 MiB)
2786 records/s, 116.2 MiB/s, 42.7 KiB/rec (16677 total, 436.5 MiB)
3016 records/s, 116.1 MiB/s, 39.4 KiB/rec (18185 total, 494.5 MiB)
2619 records/s, 121.0 MiB/s, 47.3 KiB/rec (19497 total, 555.1 MiB)
2682 records/s, 113.6 MiB/s, 43.4 KiB/rec (20838 total, 611.9 MiB)
2894 records/s, 118.6 MiB/s, 42.0 KiB/rec (22299 total, 671.8 MiB)
2609 records/s, 117.0 MiB/s, 45.9 KiB/rec (23604 total, 730.4 MiB)
2775 records/s, 122.7 MiB/s, 45.3 KiB/rec (25002 total, 792.2 MiB)
2673 records/s, 124.4 MiB/s, 47.7 KiB/rec (26340 total, 854.4 MiB)
3023 records/s, 120.4 MiB/s, 40.8 KiB/rec (27852 total, 914.7 MiB)
3057 records/s, 119.0 MiB/s, 39.8 KiB/rec (29382 total, 974.2 MiB)
2757 records/s, 125.6 MiB/s, 46.6 KiB/rec (30762 total, 1037.1 MiB)
2432 records/s, 118.4 MiB/s, 49.9 KiB/rec (31978 total, 1096.3 MiB)
2844 records/s, 123.5 MiB/s, 44.5 KiB/rec (33402 total, 1158.1 MiB)
3045 records/s, 120.1 MiB/s, 40.4 KiB/rec (34926 total, 1218.2 MiB)
2773 records/s, 122.0 MiB/s, 45.0 KiB/rec (36315 total, 1279.3 MiB)
2839 records/s, 120.3 MiB/s, 43.4 KiB/rec (37737 total, 1339.6 MiB)
2821 records/s, 122.0 MiB/s, 44.3 KiB/rec (39150 total, 1400.7 MiB)
2830 records/s, 117.7 MiB/s, 42.6 KiB/rec (40565 total, 1459.6 MiB)
2568 records/s, 124.0 MiB/s, 49.4 KiB/rec (41850 total, 1521.6 MiB)
2828 records/s, 121.1 MiB/s, 43.9 KiB/rec (43269 total, 1582.4 MiB)
2675 records/s, 122.5 MiB/s, 46.9 KiB/rec (44607 total, 1643.7 MiB)
2957 records/s, 121.7 MiB/s, 42.2 KiB/rec (46086 total, 1704.6 MiB)
2856 records/s, 119.0 MiB/s, 42.6 KiB/rec (47517 total, 1764.2 MiB)
2676 records/s, 124.1 MiB/s, 47.5 KiB/rec (48864 total, 1826.7 MiB)
2668 records/s, 123.6 MiB/s, 47.4 KiB/rec (50199 total, 1888.5 MiB)
2555 records/s, 120.0 MiB/s, 48.1 KiB/rec (51477 total, 1948.6 MiB)
2547 records/s, 120.8 MiB/s, 48.6 KiB/rec (52752 total, 2009.0 MiB)
2632 records/s, 111.6 MiB/s, 43.4 KiB/rec (54069 total, 2064.9 MiB)
2698 records/s, 110.0 MiB/s, 41.7 KiB/rec (55419 total, 2119.9 MiB)
2579 records/s, 116.3 MiB/s, 46.2 KiB/rec (56709 total, 2178.1 MiB)
2488 records/s, 109.6 MiB/s, 45.1 KiB/rec (57953 total, 2232.9 MiB)
3038 records/s, 116.4 MiB/s, 39.2 KiB/rec (59472 total, 2291.1 MiB)
2938 records/s, 117.6 MiB/s, 41.0 KiB/rec (60942 total, 2350.0 MiB)
2344 records/s, 119.7 MiB/s, 52.3 KiB/rec (62115 total, 2409.9 MiB)
2892 records/s, 121.7 MiB/s, 43.1 KiB/rec (63564 total, 2470.8 MiB)
2787 records/s, 121.0 MiB/s, 44.5 KiB/rec (64959 total, 2531.4 MiB)
2686 records/s, 118.2 MiB/s, 45.1 KiB/rec (66303 total, 2590.5 MiB)
2662 records/s, 123.4 MiB/s, 47.5 KiB/rec (67635 total, 2652.2 MiB)
2872 records/s, 118.4 MiB/s, 42.2 KiB/rec (69072 total, 2711.5 MiB)
2532 records/s, 123.7 MiB/s, 50.0 KiB/rec (70344 total, 2773.6 MiB)
2789 records/s, 118.1 MiB/s, 43.3 KiB/rec (71739 total, 2832.6 MiB)
2832 records/s, 124.6 MiB/s, 45.1 KiB/rec (73155 total, 2895.0 MiB)
1973 records/s, 131.9 MiB/s, 68.4 KiB/rec (74142 total, 2960.9 MiB)
2307 records/s, 123.4 MiB/s, 54.8 KiB/rec (75300 total, 3022.9 MiB)
2240 records/s, 128.9 MiB/s, 58.9 KiB/rec (76428 total, 3087.8 MiB)
2140 records/s, 121.0 MiB/s, 57.9 KiB/rec (77498 total, 3148.2 MiB)
2248 records/s, 117.2 MiB/s, 53.4 KiB/rec (78624 total, 3206.9 MiB)
1955 records/s, 124.9 MiB/s, 65.4 KiB/rec (79602 total, 3269.4 MiB)
2340 records/s, 125.9 MiB/s, 55.1 KiB/rec (80775 total, 3332.5 MiB)
2363 records/s, 124.3 MiB/s, 53.8 KiB/rec (81957 total, 3394.7 MiB)
1894 records/s, 130.7 MiB/s, 70.7 KiB/rec (82908 total, 3460.3 MiB)
2207 records/s, 126.0 MiB/s, 58.5 KiB/rec (84012 total, 3523.3 MiB)
2287 records/s, 129.0 MiB/s, 57.8 KiB/rec (85161 total, 3588.1 MiB)
2278 records/s, 122.8 MiB/s, 55.2 KiB/rec (86300 total, 3649.6 MiB)
2170 records/s, 125.3 MiB/s, 59.1 KiB/rec (87385 total, 3712.2 MiB)
2263 records/s, 128.2 MiB/s, 58.0 KiB/rec (88518 total, 3776.4 MiB)
2244 records/s, 125.5 MiB/s, 57.3 KiB/rec (89646 total, 3839.5 MiB)
2189 records/s, 127.1 MiB/s, 59.4 KiB/rec (90741 total, 3903.0 MiB)
1845 records/s, 128.5 MiB/s, 71.3 KiB/rec (91665 total, 3967.4 MiB)
2040 records/s, 121.9 MiB/s, 61.2 KiB/rec (92697 total, 4029.0 MiB)
2281 records/s, 123.5 MiB/s, 55.4 KiB/rec (93840 total, 4090.9 MiB)
1949 records/s, 131.3 MiB/s, 69.0 KiB/rec (94815 total, 4156.6 MiB)
1941 records/s, 125.0 MiB/s, 65.9 KiB/rec (95787 total, 4219.2 MiB)
2234 records/s, 125.6 MiB/s, 57.6 KiB/rec (96906 total, 4282.1 MiB)
2319 records/s, 126.8 MiB/s, 56.0 KiB/rec (98067 total, 4345.6 MiB)
2375 records/s, 123.6 MiB/s, 53.3 KiB/rec (99258 total, 4407.6 MiB)
2424 records/s, 129.7 MiB/s, 54.8 KiB/rec (100470 total, 4472.4 MiB)
2283 records/s, 125.7 MiB/s, 56.4 KiB/rec (101613 total, 4535.3 MiB)
2333 records/s, 125.0 MiB/s, 54.9 KiB/rec (102789 total, 4598.3 MiB)
2218 records/s, 123.0 MiB/s, 56.8 KiB/rec (103898 total, 4659.8 MiB)
2303 records/s, 125.8 MiB/s, 55.9 KiB/rec (105051 total, 4722.8 MiB)
2296 records/s, 128.7 MiB/s, 57.4 KiB/rec (106200 total, 4787.3 MiB)
2259 records/s, 127.1 MiB/s, 57.6 KiB/rec (107331 total, 4850.9 MiB)
1886 records/s, 126.7 MiB/s, 68.8 KiB/rec (108279 total, 4914.6 MiB)
2045 records/s, 129.2 MiB/s, 64.7 KiB/rec (109311 total, 4979.8 MiB)
2165 records/s, 125.5 MiB/s, 59.4 KiB/rec (110394 total, 5042.6 MiB)
1963 records/s, 129.6 MiB/s, 67.6 KiB/rec (111381 total, 5107.7 MiB)
2296 records/s, 123.7 MiB/s, 55.2 KiB/rec (112536 total, 5169.9 MiB)
2483 records/s, 124.3 MiB/s, 51.3 KiB/rec (113778 total, 5232.1 MiB)
Summary: 43.8s, 2612 records/s, 120.0 MiB/s, 47.1 KiB/rec (114274 total, 5252.1 MiB)
```

### Zstandard:

Unsupported.

### LZ4:

Unsupported.
