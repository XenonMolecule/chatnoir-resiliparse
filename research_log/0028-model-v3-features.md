# 0028 — Model v3 feature axis (in progress)

- **Date:** 2026-08-07
- **Status:** in progress — features built, calibration pending

## Built (committed at 30fbbe2)
32-dim FNV-hashed word bag accumulated bottom-up through the DOM, per-block
heading counts, prev/next-block context (link-density + log-length),
page-heading density — 60 features total, train/export column order
canonicalized against the Rust code generator.

## First measurement (HistGBM on 10k train, held-out)
- **AUC 0.8551** vs 0.8497 (v1 features): the feature axis moves where the
  data axis (0027) did not — consistent with the jusText n-gram precedent,
  though the sketch-sized bag buys +0.005, not +0.03.
- Tier shift: keep@0.9 steady (9.3% cov / 2.2% fk); the veto end DEGRADED
  (veto@0.05 now 10% fv — the wordbag makes some low-end family confidently
  wrong). Do NOT blind-swap: next step is training the exportable n60d5 on
  v3 features and choosing thresholds by live harness sweeps, not offline
  tiers.

## Next (exact steps)
1. Retrain exportable GBR n60d5 on v3 features (train_block_classifier.py
   already emits the right order; small_gbm.py's inline order is STALE — do
   not use without updating).
2. Export via export_gbm_rust.py (names list already v3).
3. Live threshold sweep: VETO ∈ {0.05, 0.08, disabled}, KEEP ∈ {0.85, 0.9},
   dev + train battery each.
4. If keep-tier wins hold: revisit the queued learned-selection families
   (images, md-links) with v3 scores.
