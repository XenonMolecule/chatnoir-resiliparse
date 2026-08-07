# 0057 — Golden devset v1 (charter-based gold-noise removal)

- **Date:** 2026-08-07
- **Status:** built (dev_golden.jsonl.gz alongside the untouched original)

## Process
Owner-ratified charter (GOLD_CHARTER.md) → 34-agent read-only fleet over
all 1000 dev docs → 366 verified removal specs (27 docs flagged for
owner review, packet pending) → mechanical rebuild via
benchmark/eval/build_golden.py from research_log/dev_golden_specs.jsonl.
365 docs edited, ~1.01M noise chars removed (families: teaser chrome
N5/N6 380, nav N3 111, sidebar widgets N4 102, machine junk N2 45,
raw HTML N1 26). Random-sample base rate that motivated the fleet:
11/30 docs major noise, mean 18% noise share.

## Current pipeline (0055) scored on both
| gold | F1 | Lev |
|---|---|---|
| original dev | 0.8223 | 0.7260 |
| **golden dev** | **0.8330** | **0.7450** |

+0.0107 F1 / +0.0190 Lev of already-earned performance was hidden by
target noise. Charter protections held: comments, rendered multi-article
content, and copyright text untouched by every spec.

## Standing policy
Report BOTH numbers each cycle until the owner blesses golden as
primary. The stop-hook goal remains scored on the original.
