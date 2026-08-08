# 0090 — NEGATIVE: smart-quote/dash phantom normalization

- **Date:** 2026-08-08
- **Tags:** 0090-goldenv10 (reverted to v9)

## Hypothesis
Extend the v9 source-absence test to smart quotes/dashes/ellipsis
(byline-exempt): 944 docs carried "phantom" typography.

## Result
Lev −0.0005 — the source-absence test is UNSOUND for cp1252-range
chars: mojibake pages carry real curly quotes/dashes as raw \x92-style
bytes the annotator's browser decoded but a UTF-8 string test misses.
Those chars ARE page typography. v9's U+2011/U+202F normalization
stands (outside cp1252, no such ambiguity); quotes/dashes reverted.

Golden remains v9: **0.8489/0.7624**. Lesson: source-absence proofs
need encoding-aware byte-level tests before extending further.
