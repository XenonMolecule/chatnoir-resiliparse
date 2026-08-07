# 0041 — phpBB subSilver2 handler

- **Date:** 2026-08-07
- **Tag:** 0041-subsilver (baseline: 0040-v3)
- **Status:** landed — zero regressions on any split

## Hypothesis
The dash-agnostic byline-miss census (the em/en-dash census artifact is
itself logged: gold is 61/39 en/em on vB docs — annotator variance, do
not chase) shows 12 dev / 84 train phpBB docs our prosilver handler
misses: the subSilver2 table skin (`b.postauthor` + `div.postbody` +
"Posted:" dates in `td.gensmall`). Board-configured date strings are
copied verbatim by gold — no reformatting needed.

## What changed
`extract_phpbb_subsilver2`: merged doc-order query over b.postauthor /
div.postbody / td.gensmall; per author, date = text after "Posted:"
(split before "Post subject:", ≤48B, digit-bearing), body = first
following postbody; `**author — date**` + body; ≥2 posts and ≥25% page
coverage guard (0021 pattern). Dispatched after the prosilver handler.
Byline-exact on the probe doc (worldworksgames).

## Results
| split | F1 | Lev | vs 0040 |
|---|---|---|---|
| dev | **0.8086** | **0.7109** | +0.0009 / +0.0014 |
| train | 0.7985 | 0.6958 | +0.0002 / +0.0004 |

dev 6 improved / 0 regressed; train 18 / 0. General dev plain
identical; goldens pass.

## Queue from census (dash-agnostic true misses)
vBulletin variants 12 dev / 85 train (incl. printthread.php print
views); XenForo 4 dev / 53 train (jusText 0058 map: blockquote
.messageText + title-attr times); Invision variants 5/68; generic
comment platforms (Lithium/lenovo etc.) 19 dev / 249 train.
