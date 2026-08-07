# 0045 — Tab normalization

- **Date:** 2026-08-07
- **Tag:** 0045-v2 (baseline: 0044-movabletype)
- **Status:** landed

## Hypothesis
Gold is tab-free in 99.6% of docs; we emit tabs in 7.4% (white-space:pre
contexts and layout tables that survive the walk). Collapsing them to
gold's convention is a broad mechanical Lev win.

## What changed
`normalize_tabs` post-pass: interior whitespace runs containing a tab
collapse to one space. Three protections, all measured in (v1 cratered
source-code pages −0.36): docs where >25% of non-empty lines carry tabs
are code listings and opt out wholesale; fenced code keeps its tabs
(gold does); leading indentation is never touched.

## Results
| split | F1 | Lev | vs 0044 |
|---|---|---|---|
| dev | 0.8090 | **0.7118** | 0 / +0.0002 |
| train | 0.7988 | 0.6965 | 0 / +0.0001 |

dev 57 improved / 4 down (Lev-only, worst −0.0008); train 630 / 48
(worst −0.005). Zero both-down on either split. General dev plain
1000/1000; goldens pass; 1.5 ms/doc band.
