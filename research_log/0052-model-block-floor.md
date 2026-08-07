# 0052 — Model block-floor sweep

- **Date:** 2026-08-07
- **Tag:** 0052-floor40 (baseline: 0051-final)
- **Status:** landed

## Hypothesis
The 150-byte model floor ("smaller blocks can't move either tier") is a
0028-era assumption; with real veto authority, small chrome crumbs
(share buttons, nav scraps, byline fragments) are worth scoring.

## Sweep (dev)
floor 150: 0.8174/0.7211 → 100: 0.8181/0.7221 → 80: 0.8185/0.7224 →
60: 0.8185/0.7228 → **40: 0.8185/0.7232** → 20: 0.8192/0.7244 →
10: 0.8194/0.7247. Monotone quality gains, BUT interleaved timing:
floor40 = 2.60 ms/doc (baseline 2.67 — free), floor10 = 3.37 (+26%,
breaches the 5% throughput gate). Shipped 40; floor 10-20 queued behind
per-block feature-cost profiling (~+0.0009 F1 available there).

## Results
| split | F1 | Lev | vs 0051 |
|---|---|---|---|
| dev | **0.8185** | **0.7232** | +0.0011 / +0.0021 |
| train | 0.8101 | 0.7107 | +0.0018 / +0.0029 |

train 1906 improved / 925 down, 14 craters (welcome_page interstitials
and store/product pages — tiny-block pages fully vetoed; same wall
ledger as 0051, net strongly positive). dev: zero craters. General dev
plain 1000/1000 identical; goldens pass.
