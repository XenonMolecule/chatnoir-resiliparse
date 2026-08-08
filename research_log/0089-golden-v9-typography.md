# 0089 — Golden v9: phantom-typography normalization

- **Date:** 2026-08-08
- **Status:** gold-side ship (source-gated; owner review invited)

## Finding
479 golden docs carry U+2011 (non-breaking hyphen) and 146 carry
U+202F (narrow NBSP) that their source pages provably do NOT contain
(neither raw char nor entity) — annotator-tool artifacts, confirmed by
the owner-reviewed sirstevesguide repair. This retroactively explains
the early-session "  typography wall — do not chase": it was
gold noise all along, unprovable before the source-absence test.

## Change
build_golden.py normalizes U+2011 → '-' and U+202F → ' ' ONLY when the
source lacks the char (and its entity). Reversible one-liner; docs
flagged gold_typo_fixed.

## Result
Golden v9: **0.8489 / 0.7624** (+0.0001 F1 / +0.0012 Lev — pure
measurement correction; Lev because these were per-char edit ops
sprinkled across half the devset). Original dev unchanged.
Scoreboard: original 0.8288/0.7328 · golden v9 0.8489/0.7624.
