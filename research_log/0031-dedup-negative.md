# 0031 — Paragraph dedup — NEGATIVE (naive port), disabled

- **Date:** 2026-08-07
- **Tag:** 0031-dedup{,2} measured; 0031-reverted (= 0030 output)
- **Status:** abandoned pending a containment-aware port

## Measured
| floor | dev | train |
|---|---|---|
| 60B exact-dup | +0.0011 (40↑/22↓, worst −0.156) | not run |
| 150B exact-dup | +0.0006 (16↑/12↓) | **−0.0004** (22 < −0.1, worst −0.82) |

## Insights
- Gold keeps verbatim repeats on some templates (pull-quotes, intro-as-body)
  — naive exact-dup dedup doesn't generalize. jusText's shipped version
  (0018/0030 there) was quote-normalized + containment-aware and shipped
  with careful gating; that's the port to do, not this one.
- Dev-positive/train-negative is the canonical overfit signature — train
  runs on every subtractive cycle remain mandatory.

## Next
- Residual families still open: sponsor blocks (newsletter pages),
  author-bio boxes, product-menu chrome.
