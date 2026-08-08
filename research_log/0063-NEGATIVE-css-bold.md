# 0063 — NEGATIVE: stylesheet-bold span emphasis

- **Date:** 2026-08-08
- **Tags:** 0063-cssbold; reverted

## Hypothesis
Additive rendering-awareness: `.class { font-weight:bold }` spans emit
`**` in markdown — attacking wall #6 (visual bold) with no veto risk.

## Result
Net negative everywhere: dev 0 up / 6 down, train 14 up / 61 down
(Lev-only, worst −0.10). Implementation worked exactly as designed
(synthetic verified, no craters, lawyerfox unaffected) — the SIGNAL is
wrong: gold bolds a curated subset of visually-bold text (bylines,
labels it deems semantic), while CSS-bold classes cover far more styled
UI text that gold renders plain. Wall #6 now confirmed from BOTH
directions: we can neither recover gold's bold from markup we drop
(0035) nor from CSS we can read (0063).

## Rendering program verdict (with 0062)
Both static-CSS probes measured negative. Simple rendering proxies do
not clear the bar; the direction's real entry cost is computed styles
from a layout engine, plus a learned mapping from visual emphasis to
gold's conventions. No further static-CSS attempts.
