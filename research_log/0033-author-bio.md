# 0033 — Author-bio boxes + page-relative chrome guard

- **Date:** 2026-08-07
- **Tag:** 0033-v3 (baseline compared against: 0032-oneoff)
- **Status:** landed

## What changed
- Gold-consistency check first: 30 dev pages carry author-bio classes, gold
  keeps the bio on 3 (10%) → safe per audit threshold. Tokens
  `author-bio|author-box|about-(the-)author|bio-box` added to the chrome
  veto (`author-info` measured riskier — matches content containers — and
  was dropped from the set).
- **Page-relative chrome guard**: the veto now also abstains when the
  candidate holds >40% of the page's text (author PAGES where the bio IS the
  content: oreilly /pub/au was −0.70 without it). Generalizes the absolute
  1500B cap.

## Results
dev 0.8046 → **0.8049 / 0.7055** (+0.0003/+0.0005); train +0.0002 (13 > +0.1
vs 7 < −0.1; one persistent −0.78 attachment-page outlier, gold-noise-
adjacent). Goldens pass.

## Next
- M1 milestone at 0.81/0.72: F1 needs +0.005, Lev +0.015.
