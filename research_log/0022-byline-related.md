# 0022 — Byline anchors + related-modules (mostly a wall, logged)

- **Date:** 2026-08-07
- **Tag:** 0022-final (baseline compared against: 0021-forums3)
- **Status:** landed (small), with two measured negatives

## Hypothesis
Taxonomy #4 (byline anchors stripped, ~10 docs) and #1 (related/recommended
modules on short articles, est. +0.011).

## What changed / measured
- `rel="author"` anchors kept in markdown config **when in byline-context
  ancestry** (author/byline/vcard/posted class within 3 levels). The
  unconditional version measured net-zero (108↑/40↓ — forum member links
  re-admitted); the itemprop=author exemption measured negative and was
  reverted.
- Related-module tokens: the full set (related|recommended|trending|popular…)
  measured dev +0.0009 but train 24 docs < −0.1 vs 28 > +0.1 — the audit's
  ~18%-keep wall in action; **reverted to unambiguous ad-tech tokens only**
  (outbrain|taboola|around-the-web|read-next|you-may-like|sponsored) which
  are clean but tiny.

## Results
dev +0.0001 (41↑/22↓ all small), train +0.0001 (0 docs beyond ±0.1).
Net cycle: ≈ +0.0002. Goldens pass.

## Insights
- Family #1's +0.011 estimate is NOT rule-capturable: gold's related-module
  policy correlates with page type, not class names. This is now the third
  independent confirmation (audit keep-rates, 0018 tradingview, this train
  tail) that the remaining precision mass needs page-type conditioning —
  i.e., the learned classifier. Rule-mining in this region is done.

## Next
- 0023: listing-card rescue + dl-tables (recall side — different physics).
- Begin learned-classifier groundwork (M3 pulled forward): features exist
  (tpl stats, link density, class signals); big_train available.
