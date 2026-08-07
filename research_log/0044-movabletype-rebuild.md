# 0044 — MovableType comment rebuild

- **Date:** 2026-08-07
- **Tag:** 0044-movabletype (baseline: 0043-v5)
- **Status:** landed (small, surgical)

## What changed
`movabletype_comment_rebuild` (jusText-0085 family): source renders
body-first (`div.commentText`) with attribution after (`p.posted`:
"Posted by: NAME | DATE | ..."); gold puts `**NAME — DATE**` first.
Doc-order body→byline pairing, always-rebuild semantics, chained after
the Blogspot rebuild. Byline-exact on the probe doc (146 bylines).

## Results
dev **0.8090 / 0.7116**: 1 doc transformed (wizardsinsider
F1 0.030→0.549, Lev 0.014→0.487), 0 regressions. train: 1 tiny
regression (−0.003, a sibling blog with a variant p.posted format).
General dev plain 1000/1000 identical; goldens pass.
