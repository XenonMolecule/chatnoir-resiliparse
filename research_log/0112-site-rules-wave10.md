# 0112: site-rules wave 10 — golden 0.8857/0.8048; lane yield decaying again

## What
Tenth wave on 0111-v2 diffs (367 targets, 8 agents; run interrupted by a
session limit — all 8 shards resumed cleanly via workflow resume). 564 raw
rules → **+136 SITE_VETOES / +65 SITE_WHITELIST**.

## Bisect
Removed + blocklisted: kesq.com (−0.41 golden crater + train −0.11),
jetcost.com.sg (down both dev bases), linuxpromagazine.com (train
both-down), thorax.bmj.com (dev repeat offender), juanmanuelsara.com
(F1 down on both dev bases).

## Battery (`0112-v2` vs `0111-v2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8857** (+0.0007) | **0.8048** (+0.0007) | up 32 / down 5 (0 both) / craters 0 |
| dev | **0.8545** (+0.0005) | **0.7627** (+0.0007) | up 32 / down 9 (0 both) / craters 0 |
| train | **0.8156** | **0.7172** | up 5 / down 0 |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Lane status
Wave yields on golden Lev: w8 +0.0040 → w9 +0.0032 → w10 **+0.0007**.
Second decay floor hit (the 0109 domain-fallback reset bought waves 8-9).
Remaining goal gap: **−0.0143 F1** (Lev in-band at 0.8048). The residual
tail is increasingly: docs needing gold restoration rulings (owner-gated),
JS-only content, and sub-container mixed chrome — not clean selectors.
