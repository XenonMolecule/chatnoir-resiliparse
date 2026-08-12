# Benchmark results — tables only

Build `0177` (commit `850891b`), `resiliparse._extract_rs`,
`main_content=True, preserve_formatting='markdown'`.
Annotated version with methodology and caveats: [`BENCHMARKS.md`](BENCHMARKS.md).

## External benchmarks vs upstream resiliparse

| Benchmark | Docs | Metric | Upstream | This fork | Δ |
|---|--:|---|--:|--:|--:|
| marin devset v2 | 420 | token-F1 | 0.8880 | **0.9050** | +0.0170 |
| Zyte article-extraction-benchmark | 181 | token-F1 | 0.8806 | **0.8899** | +0.0093 |
| WebMainBench en/dev | 200 | token-F1 | 0.8309 | **0.8633** | +0.0324 |
| trafilatura eval set | 960 | F1 | 0.8104 | **0.8421** | +0.0317 |
| Extraction unit tests | 100 | passed | 90 | **97** | +7 |

| Benchmark | F1 | P | R | Lev sim |
|---|--:|--:|--:|--:|
| marin devset | 0.9050 | 0.8673 | 0.9462 | 0.8383 |
| Zyte | 0.8899 | 0.8106 | 0.9864 | — |
| WebMainBench (shingle) | 0.8633 | 0.8051 | 0.9305 | 0.7916 |
| trafilatura eval set | 0.8421 | 0.7656 | 0.9354 | acc 0.8251 |

marin per-doc: mean 0.8927 · median 0.9491 · ≥0.90: 283/420 · ≥0.80: 356/420 · <0.50: 17/420.

Unit tests 97/100: code 13/13 · math 10/10 · tables 7/7 · attribution 12/12 · structure 5/5.

## WebMainBench fine-grained (en/dev, n=200, use_llm=false)

| Extractor | overall | text | code | formula | table | TEDS |
|---|--:|--:|--:|--:|--:|--:|
| **This fork (0177)** | **0.6150** | **0.7951** | **0.8229** | **0.3673** | 0.4143 | **0.6756** |
| Dripper | 0.5852 | 0.7174 | 0.7647 | 0.2488 | **0.5343** | 0.6609 |
| mdx-v5think | 0.5115 | 0.7405 | 0.5908 | 0.3522 | 0.3294 | 0.5447 |
| mdx-v12 | 0.4997 | 0.7614 | 0.4845 | 0.3441 | 0.3577 | 0.5508 |
| mdx-d5+guards | 0.4989 | 0.7547 | 0.4909 | 0.3497 | 0.3487 | 0.5504 |
| trafilatura | 0.3558 | 0.7733 | 0.1609 | 0.3508 | 0.1828 | 0.3113 |
| extract_head | 0.2948 | 0.7836 | 0.0891 | 0.3069 | 0.1245 | 0.1699 |
| upstream resiliparse | 0.2352 | 0.7561 | 0.0562 | 0.3635 | 0.0000 | 0.0000 |

Denominators: text 200 · code 42 · table 41 · formula 126.
Formula on the 115 annotated-equation docs: ours 0.3838 · Dripper 0.1579.

## Internal sets

| Split | Docs | F1 | P | R | Lev sim |
|---|--:|--:|--:|--:|--:|
| lpv11 dev_golden | 1000 | **0.8948** | 0.9035 | 0.9120 | **0.8156** |
| lpv11 dev | 1000 | 0.8577 | 0.9160 | 0.8506 | 0.7671 |
| lpv11 train | 9999 | 0.8166 | 0.8570 | 0.8361 | 0.7183 |
| general dev | 1000 | 0.8139 | 0.7611 | 0.9314 | 0.7233 |

## Domain sets

| Domain | dev n | dev F1 | dev Lev | test n | test F1 | test Lev |
|---|--:|--:|--:|--:|--:|--:|
| code | 11 | 0.8445 | 0.7666 | 4 | 0.7915 | 0.7377 |
| math | 2 | 0.8240 | 0.7151 | 2 | 0.5620 | 0.3961 |
| science | 3 | 0.9333 | 0.8729 | 2 | 0.5132 | 0.4405 |
| tables | 2 | **0.8870** | 0.4620 | 5 | 0.8259 | 0.6906 |

## Runtime

7-run best-of, 1000 lpv11 dev docs, single-threaded, Apple silicon.

| Configuration | ms/doc |
|---|--:|
| markdown + main_content | 3.00 |
| plain + main_content | 1.32 |

## Sources

| Resource | Link |
|---|---|
| Zyte article-extraction-benchmark | https://github.com/scrapinghub/article-extraction-benchmark |
| WebMainBench | https://github.com/opendatalab/WebMainBench |
| trafilatura | https://github.com/adbar/trafilatura |
| Marin project | https://github.com/marin-community/marin |
| Upstream resiliparse | https://github.com/chatnoir-eu/chatnoir-resiliparse |

marin devset v2, the unit-test suite, and the lpv11/general/domain sets are
curated in a companion research repo and are not public.
