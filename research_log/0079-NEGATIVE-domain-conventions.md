# 0079 — NEGATIVE (census-only): train→dev domain-convention transfer

- **Date:** 2026-08-08
- **Status:** measured, not built

## Hypothesis
Walls are per-SITE conventions; train gold covers 236 of dev's 980
domains (249 dev docs, 70 with >=3 train examples) — learn conventions
per domain from train, apply on dev.

## Measurements (all three sank)
1. Title-keeping: domain signal is real in aggregate (88% where
   confident) but the ACTIONABLE slice (we dropped the h1, domain says
   titled) collapses to 25% precision — "gold starts with a heading"
   is not "gold uses THIS h1". Source-matched refinement: 6 actionable
   docs, 50%.
2. Byline dash: only 107/1105 byline domains are internally consistent
   (85%+ one dash) — the "convention" is mostly annotator noise WITHIN
   domain, not site style. Actionable mismatches on dev: 1 doc, and
   the convention predicted its gold wrong.
3. (Implication for hard-breaks/blockquote prefixes: same intra-domain
   inconsistency mechanism; not separately measured.)

## Verdict
The walls are not per-site conventions — they are per-DOCUMENT
annotator variance, unlearnable from any side signal. This closes the
last cheap idea for wall arbitration; walls now require either gold
unification (golden-set continuation) or nothing.
