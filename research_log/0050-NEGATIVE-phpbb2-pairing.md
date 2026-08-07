# 0050 — NEGATIVE: phpBB2 doc-order pairing + dated coverage bypass

- **Date:** 2026-08-07
- **Tag:** 0050-phpbb2 (vs 0049-batch3); reverted

## Hypothesis
The equal-count gate (`span.name == span.postbody`) misses threads whose
posts carry signature postbody spans (murga-linux family); doc-order
first-following pairing plus a "all posts dated → skip coverage guard"
bypass should widen coverage.

## Result: strictly negative, reverted
dev −0.0001/−0.0001 with the murga dev doc itself DOWN −0.11; train 10
up / 9 down with craters −0.55/−0.39/−0.32 (sinfest, eslcafe,
footballsfuture — classic skins where the old zip pairing was right).
phpBB2 table skins put name and body in table COLUMNS: document order
interleaves posts' cells unpredictably, so "first following postbody"
mispairs bodies across posts on exactly the installs the equal-count
gate exists for.

## Lesson
Doc-order pairing is safe within row-sequential markup (subSilver2,
old-Blogger... which also failed in 0040!). Column-based table layouts
need the count invariant. Two failures now — treat doc-order pairing on
mixed-cell forum tables as an anti-pattern.
