# 0024 — dl serialization + learned-classifier groundwork

- **Date:** 2026-08-07
- **Tag:** 0024-dl (baseline compared against: 0023-reverted/0022-final)
- **Status:** landed (dl); classifier groundwork in progress

## Part A — definition lists as `**label:** value` (taxonomy #5)
Markdown mode renders `<dt>` bold with a folded colon and keeps `<dd>` on the
same line. dev: +5 docs / 0 regressions, aggregate ≈ flat (quality-correct
ship per §6; renders verified). Empty-dt collapse guard.

## Part B — learned block classifier v0 (task #10)
Architecture decision: **features are computed in Rust** and exported via a
debug PyO3 entry point (`block_features`), so train and inference agree by
construction. v0: 18 structural features (tag, depth, text/link stats, page
link-density, 11 regex-family bits), labels = gold token-containment per
block, HistGradientBoosting on 3,000 train docs (370K blocks):
- **AUC 0.840**
- veto@0.05 pred: 0.17% coverage, **0.00% false-veto** (safe veto tier)
- keep@0.9 pred: 7.2% coverage, **1.3% false-keep** (whitelist tier — can
  override rule vetoes, i.e. rescue rule false-negatives)
v1 retraining now on all 9,999 docs with text-shape features (punctuation/
digit/upper densities, avg word length).

## Insights
- The four walls (nav/related/listing/catalog) all live in the model's
  mid-range — as expected — but the confident ends already offer deployable
  tiers with measured error rates, something no rule ever gave us.

## Next
- v1 results → threshold policy → Rust integration (tree export or runtime
  scoring via exported thresholds on the same features).
