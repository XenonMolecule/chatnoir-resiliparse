# 0034 — Heading-level promotion (+ hard-break gate re-confirmed)

- **Date:** 2026-08-07
- **Tag:** 0034-headings (baseline compared against: 0033-v3)
- **Status:** landed (small)

## What changed
Gold re-levels each page so its top heading is `#` (98% of heading-bearing
golds; our preds were shifted down on 151 docs). Post-serialization
promotion: shift all headings up so the doc minimum is `#`; fenced code
untouched. Also re-tested ungated two-space hard breaks on the current
baseline: still Lev-negative (−0.0009) — the 0009 BR-origin gate is correct;
reverted, twice-confirmed.

## Results
Lev +0.0002 (F1 unchanged — hashes are invisible to ROUGE); dev
**0.8049 / 0.7057**, train 0.7950/0.6972. Goldens pass.

## Insights
- The whitespace census ranks remaining mechanics: gold-side hard breaks
  (context-specific, partially wall), bold-coverage gaps (304+225 `**` ops —
  label-styled bolding, needs the deferred heuristics),   typography
  (133 ops, gold-side narrow-NBSP in numbers).
