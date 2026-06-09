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
38787 records/s, 1112.9 MiB/s, 29.4 KiB/rec (19414 total, 557.0 MiB)
30960 records/s, 1344.8 MiB/s, 44.5 KiB/rec (34909 total, 1230.1 MiB)
33861 records/s, 1512.2 MiB/s, 45.7 KiB/rec (51841 total, 1986.3 MiB)
34281 records/s, 1493.2 MiB/s, 44.6 KiB/rec (68983 total, 2732.9 MiB)
27507 records/s, 1499.2 MiB/s, 55.8 KiB/rec (82756 total, 3483.6 MiB)
25857 records/s, 1534.8 MiB/s, 60.8 KiB/rec (95728 total, 4253.6 MiB)
26817 records/s, 1518.7 MiB/s, 58.0 KiB/rec (109189 total, 5015.9 MiB)
Summary: 3.7s, 30959 records/s, 1435.4 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.gz
27624 records/s, 636.3 MiB/s, 23.6 KiB/rec (13866 total, 319.4 MiB)
17966 records/s, 768.7 MiB/s, 43.8 KiB/rec (22851 total, 703.8 MiB)
17281 records/s, 752.4 MiB/s, 44.6 KiB/rec (31494 total, 1080.2 MiB)
17769 records/s, 769.6 MiB/s, 44.4 KiB/rec (40425 total, 1467.0 MiB)
17019 records/s, 762.6 MiB/s, 45.9 KiB/rec (48942 total, 1848.6 MiB)
16356 records/s, 732.5 MiB/s, 45.9 KiB/rec (57156 total, 2216.5 MiB)
16930 records/s, 733.4 MiB/s, 44.4 KiB/rec (65625 total, 2583.4 MiB)
16319 records/s, 764.7 MiB/s, 48.0 KiB/rec (73785 total, 2965.7 MiB)
13328 records/s, 764.2 MiB/s, 58.7 KiB/rec (80449 total, 3347.8 MiB)
13784 records/s, 790.7 MiB/s, 58.7 KiB/rec (87348 total, 3743.5 MiB)
13364 records/s, 793.2 MiB/s, 60.8 KiB/rec (94032 total, 4140.2 MiB)
13848 records/s, 806.5 MiB/s, 59.6 KiB/rec (100959 total, 4543.7 MiB)
14010 records/s, 785.4 MiB/s, 57.4 KiB/rec (107970 total, 4936.7 MiB)
Summary: 6.9s, 16470 records/s, 763.7 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.zst
31872 records/s, 818.8 MiB/s, 26.3 KiB/rec (15939 total, 409.5 MiB)
21612 records/s, 938.3 MiB/s, 44.5 KiB/rec (26746 total, 878.7 MiB)
21770 records/s, 939.5 MiB/s, 44.2 KiB/rec (37633 total, 1348.5 MiB)
22496 records/s, 994.0 MiB/s, 45.2 KiB/rec (48882 total, 1845.5 MiB)
21850 records/s, 963.6 MiB/s, 45.2 KiB/rec (59808 total, 2327.3 MiB)
21818 records/s, 981.6 MiB/s, 46.1 KiB/rec (70717 total, 2818.2 MiB)
18121 records/s, 983.9 MiB/s, 55.6 KiB/rec (79780 total, 3310.2 MiB)
18525 records/s, 1058.9 MiB/s, 58.5 KiB/rec (89043 total, 3839.7 MiB)
17423 records/s, 1060.7 MiB/s, 62.3 KiB/rec (97758 total, 4370.3 MiB)
19154 records/s, 1049.9 MiB/s, 56.1 KiB/rec (107337 total, 4895.3 MiB)
Summary: 5.4s, 21148 records/s, 980.5 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### LZ4:

```console
$ echo 3 | sudo tee /proc/sys/vm/drop_caches
$ ./profile CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
43768 records/s, 1318.6 MiB/s, 30.8 KiB/rec (21897 total, 659.7 MiB)
33446 records/s, 1461.2 MiB/s, 44.7 KiB/rec (38623 total, 1390.4 MiB)
32469 records/s, 1455.5 MiB/s, 45.9 KiB/rec (54858 total, 2118.2 MiB)
33254 records/s, 1465.5 MiB/s, 45.1 KiB/rec (71536 total, 2853.2 MiB)
27441 records/s, 1544.1 MiB/s, 57.6 KiB/rec (85257 total, 3625.2 MiB)
27160 records/s, 1604.6 MiB/s, 60.5 KiB/rec (98839 total, 4427.7 MiB)
27484 records/s, 1578.8 MiB/s, 58.8 KiB/rec (112582 total, 5217.2 MiB)
Summary: 3.6s, 32141 records/s, 1490.2 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

## Results (RAM)

These results were measured on an AMD Ryzen Threadripper 2920X 12-Core CPU with the WARC read directly from RAM
(single-core performance, read buffer size: 1 MiB).

### Uncompressed:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc
119701 records/s, 4656.5 MiB/s, 39.8 KiB/rec (59851 total, 2328.3 MiB)
101647 records/s, 5545.9 MiB/s, 55.9 KiB/rec (110683 total, 5101.7 MiB)
Summary: 1.0s, 110170 records/s, 5108.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Gzip:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.gz
29881 records/s, 739.1 MiB/s, 25.3 KiB/rec (14943 total, 369.6 MiB)
19975 records/s, 853.4 MiB/s, 43.7 KiB/rec (24933 total, 796.4 MiB)
20182 records/s, 875.2 MiB/s, 44.4 KiB/rec (35024 total, 1234.0 MiB)
20057 records/s, 888.1 MiB/s, 45.3 KiB/rec (45057 total, 1678.3 MiB)
19590 records/s, 879.2 MiB/s, 46.0 KiB/rec (54855 total, 2118.0 MiB)
19574 records/s, 850.2 MiB/s, 44.5 KiB/rec (64647 total, 2543.3 MiB)
18587 records/s, 860.5 MiB/s, 47.4 KiB/rec (73953 total, 2974.2 MiB)
15578 records/s, 880.0 MiB/s, 57.8 KiB/rec (81744 total, 3414.3 MiB)
15660 records/s, 914.1 MiB/s, 59.8 KiB/rec (89574 total, 3871.3 MiB)
14503 records/s, 892.4 MiB/s, 63.0 KiB/rec (96837 total, 4318.2 MiB)
16392 records/s, 889.0 MiB/s, 55.5 KiB/rec (105042 total, 4763.2 MiB)
15384 records/s, 926.4 MiB/s, 61.7 KiB/rec (112734 total, 5226.4 MiB)
Summary: 6.1s, 18776 records/s, 870.6 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### Zstandard:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.zst
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.zst
37401 records/s, 1050.0 MiB/s, 28.7 KiB/rec (18702 total, 525.0 MiB)
24974 records/s, 1085.2 MiB/s, 44.5 KiB/rec (31192 total, 1067.8 MiB)
24633 records/s, 1083.5 MiB/s, 45.0 KiB/rec (43509 total, 1609.5 MiB)
25158 records/s, 1119.1 MiB/s, 45.5 KiB/rec (56095 total, 2169.4 MiB)
25103 records/s, 1101.9 MiB/s, 44.9 KiB/rec (68647 total, 2720.4 MiB)
22265 records/s, 1179.7 MiB/s, 54.3 KiB/rec (79780 total, 3310.2 MiB)
20789 records/s, 1190.6 MiB/s, 58.6 KiB/rec (90175 total, 3905.5 MiB)
19647 records/s, 1162.5 MiB/s, 60.6 KiB/rec (99999 total, 4486.8 MiB)
20128 records/s, 1167.2 MiB/s, 59.4 KiB/rec (110064 total, 5070.5 MiB)
Summary: 4.7s, 24331 records/s, 1128.1 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```

### LZ4:

```console
$ ./profile tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
Reading WARC file: tmpfs/CC-MAIN-20231005012006-20231005042006-00899.warc.lz4
55557 records/s, 1842.2 MiB/s, 34.0 KiB/rec (27781 total, 921.2 MiB)
44798 records/s, 1969.0 MiB/s, 45.0 KiB/rec (50182 total, 1905.7 MiB)
44793 records/s, 1981.9 MiB/s, 45.3 KiB/rec (72586 total, 2897.0 MiB)
36112 records/s, 2070.2 MiB/s, 58.7 KiB/rec (90645 total, 3932.3 MiB)
36679 records/s, 2143.5 MiB/s, 59.8 KiB/rec (108988 total, 5004.2 MiB)
Summary: 2.6s, 43366 records/s, 2010.7 MiB/s, 47.5 KiB/rec (114274 total, 5298.4 MiB)
```
