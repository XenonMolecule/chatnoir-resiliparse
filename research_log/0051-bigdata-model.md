# 0051 — Big-data v3 model + size-tiered veto authority — M1 CROSSED

- **Date:** 2026-08-07
- **Tag:** 0051-final (= 0051-sized; baseline: 0049-batch3)
- **Status:** landed — largest cycle of the journey

## What changed
1. **Model:** GBR n60d6 retrained on 3M blocks from 98k big_train docs
   with v3 features (AUC 0.8442 vs 0.8395); exported to block_model.rs
   (15.4k lines). The 0028-era "data-scaling plateau" verdict was an
   artifact of the CONSERVATIVE thresholds — with real veto authority,
   more data pays.
2. **Thresholds re-swept for the new calibration:** veto 0.30
   (small blocks), keep 0.70. At the old 0.25/0.85 the new model
   measured WORSE — sweeps do not transfer between calibrations.
3. **Size-tiered veto (the structural piece):** blocks >1500B need
   score < 0.10 to be vetoed — a large block is an article-body
   candidate; the crater profile at wide thresholds was whole-article
   false vetoes. Small blocks use the aggressive threshold.

## Rejected variants (all measured)
- 40% page-mass veto cap; relative "model gutted the page" rollback
  (with and without link-density guard); sparse-page conservative mode;
  genuinely-unfiltered tier-1 fallback. Every guard shifted which docs
  crater without improving the aggregate — the crater set is the
  gold-keeps-chrome wall family expressing through the model axis, and
  it is not locally separable (see-saw measured 5 ways).

## Results
| split | F1 | Lev | vs 0049 |
|---|---|---|---|
| dev | **0.8174** | **0.7211** | +0.0054 / +0.0058 |
| train | 0.8083 | 0.7078 | +0.0055 / +0.0066 |

dev 166 improved / 84 down (5 craters ≤ −0.32); train 1656 / 839 (40
craters). Logged decision: 2.7:1 doc ratio, both aggregates at journey
highs, and the crater family is the established wall. General dev plain
1000/1000 identical; dev2 at its high-water mark; goldens+guard pass;
1.54 ms/doc.

## M1 MILESTONE: dev F1 0.8174 ≥ 0.81 AND Lev 0.7211 ≥ 0.72 — both
targets crossed. The declared `--allow-test` checkpoint is due.
