# 0027 — Model v2 (big_train scaling) — NEGATIVE, v1 retained

- **Date:** 2026-08-07
- **Tag:** 0027-v2model measured; reverted to the 0025 model
- **Status:** abandoned (scaling axis); v3 = feature axis

## Hypothesis
10× training data (big_train: 98K docs → 12.2M blocks) should lift the block
model like jusText's fastText-on-100k (+0.027).

## Measured
- HistGBM-300 ceiling: AUC 0.8474 (v1-features ceiling was ~0.85 on train
  too — flat).
- Exportable GBR n60d5 @3M rows: AUC 0.8386 vs v1's 0.8374.
- Live on dev: **−0.0001 (3↑/5↓)** — a wash. Reverted.

## Insights
- **Data scaling is saturated at these 22 features.** jusText's big win came
  from fastText TEXT n-grams — a fundamentally richer representation — not
  from row count. v3 must add features: page-type signals (n_articles,
  comment-block counts, heading density, listing predicates), neighbor
  context (prev/next block stats), and cheap text n-gram hashes.
- The measurement cost was one background pipeline (~1.5h wall, zero
  foreground time) — the parallel-agent/background pattern makes negative
  results cheap.

## Next
- v3 features (the queue's learned-selection items — images, md-links,
  listing/related walls — all wait on exactly this).
