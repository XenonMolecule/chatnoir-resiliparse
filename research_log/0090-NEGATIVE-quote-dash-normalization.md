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

## v11 addendum (encoding-gated retry): still negative
Even restricted to pages with zero cp1252-suspicious bytes, the
smart-typography normalization measures Lev −0.0005: our own preds
frequently CONTAIN the smart chars (decoded upstream of the stored
html snapshot), so gold's smart typography matches predictions better
as-is. The phantom test is unsound for any char our own pipeline can
produce; U+2011/U+202F (v9) remain the only provable phantoms.
Typography interrogation CLOSED; golden stays v9 (0.8489/0.7624).
