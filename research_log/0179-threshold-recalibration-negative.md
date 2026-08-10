# 0179: threshold recalibration for the 120k model — recovers ⅓, still loses

## What
The 0178 open rescue: coordinate-descent over the three model operating
points with the 120k control model (gbr_v5_120k, 66 features), judged on
lpv11 dev_golden. 13 grid points, each a full export→build→1000-doc run.

## Grid (F1 on dev_golden; shipped bar = 0.89482)
veto axis (big=0.10, keep=0.60):
| 0.30 | 0.35 | **0.40** | 0.45 | 0.50 |
|---|---|---|---|---|
| 0.88861 | 0.89191 | **0.89292** | 0.88637 | 0.87878 |

keep axis (veto=0.40, big=0.10):
| 0.50 | 0.55 | 0.60 | **0.65** | 0.70 | 0.75 |
|---|---|---|---|---|---|
| 0.88870 | 0.89183 | 0.89292 | **0.89345** | 0.89012 | 0.88891 |

big axis (veto=0.40, keep=0.60): 0.05 → 0.89292 (identical), 0.15/0.20 →
0.89290 — inert; almost no big blocks score in that band.

## Verdict: NEGATIVE, lane closed
Best recalibrated point (0.40/0.10/0.65) = **0.89345/0.81369** — the
stricter keep threshold does recover the tag-page whitelisting losses
(≈⅓ of the 120k deficit, +0.0005 over its own default), exactly the
mechanism 0178 predicted. But it remains **−0.0014 F1 / −0.0019 Lev
below shipped**. Both axes are cleanly unimodal around the incumbent
values, so no finer step or interaction is going to find +0.0014.

The 120k data lever is now measured-closed in all three forms: as-trained
title variant (0178), as-trained control (0178), and threshold-
recalibrated control (this cycle). The shipped model + 0.40/0.10/0.60
stands. Reverted; parity re-verified (0 changed docs vs 0177-w3 on
dev_golden, extract_golden 7/7).

## What would be different next time
The models themselves are stronger rankers (AUC +0.008); the transfer
failure is distribution shift against fixed *global* operating points.
A per-page adaptive threshold (e.g., quantile-of-page-scores) is the
untested idea that survives this closure — but it is a mechanism change,
not a calibration, and belongs behind the next big feature idea, not
ahead of it.
