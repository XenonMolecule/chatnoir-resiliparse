# 0055 — v5 page features (nav share, generator kind, block count)

- **Date:** 2026-08-07
- **Tag:** 0055-ship (= 0055-v5; baseline: 0054-final)
- **Status:** landed

## What changed
Three more page-type features: semantic-nav text share (nav/aside/
header/footer subtree text, accumulated bottom-up), coarse generator
kind (none/blogger/wordpress/forum from the meta tag), and
substantial-block count (log1p). 66 dims; GBR n120d6 retrained,
AUC **0.8592** (axis ledger: 0.8503 → 0.8550 → 0.8592). Thresholds:
0.40/0.60 kept — the dev-swept 0.38 point measured WORSE on train
(0.8140/0.7148 vs 0.8142/0.7154): dev-sweep overfit caught by the
train cross-check, which is now a standing step for threshold picks.

## Ops note
Background trainings were being killed by harness lifecycle (three
losses misdiagnosed as OOM); nohup+disown with a Monitor watch is the
reliable pattern.

## Results
| split | F1 | Lev | vs 0054 |
|---|---|---|---|
| dev | **0.8223** | **0.7260** | +0.0017 / +0.0013 |
| train | 0.8142 | 0.7154 | +0.0006 / +0.0005 |

General dev plain 1000/1000 identical; goldens pass; fences balanced.
Next v6 candidates: title/h1 overlap, per-block distance-to-densest-
region, listing-grid share.
