# 0054 — v4 page-level features (forms / articles / comment-classes)

- **Date:** 2026-08-07
- **Tag:** 0054-final (baseline: 0053-final)
- **Status:** landed

## Hypothesis
Every confirmed wall is a page-type ambiguity the block model cannot
see. First page-type features: page-wide counts of `<form>`,
`<article>`, and class*="comment" elements (log1p), threaded through
TplNode bottom-up — DOM-only (the API has no URL). Feature v4 = 63 dims.

## What changed
Scan counts + debug export + exporter layout committed as 0054-prep;
model GBR n120d6 retrained on the re-extracted 100k-doc v4 cache
(AUC **0.8550** vs 0.8503 — the 3 features add as much as doubling
trees did); thresholds re-swept → veto 0.40 / keep 0.60.

## Results
| split | F1 | Lev | vs 0053 |
|---|---|---|---|
| dev | **0.8206** | **0.7247** | +0.0013 / +0.0014 |
| train | 0.8136 | 0.7149 | +0.0018 / +0.0025 |

Model-churn craters continue (34 train, replacing 0053's 23 — the wall
docs reshuffle under each new model; logged). General dev plain
1000/1000 identical; goldens pass; fences balanced.

## Direction confirmed
Page-type features move BOTH AUC and live metrics — the axis extends:
generator meta flags, title/h1 overlap, nav-text share, block-count
shape. Queue for v5.
