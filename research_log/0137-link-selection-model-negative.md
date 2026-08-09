# 0137: learned per-anchor link selection — NEGATIVE, selection wall quantified

The last untested autonomous lane (0136 follow-up). Built the full
per-anchor dataset from the TRAIN split (9,999 docs, 614,321 http anchors,
10 DOM features): label = train gold keeps the anchor as [text](url).
Base rate **1.11%**. GBM (n120 d5): **AUC 0.8652** — decent ranking, but
the precision-recall floor is fatal: precision 0.70 only at **2.5% recall**
(~170 correct links corpus-wide against ~70 wrong ones; each wrong link
injects URL tokens that damage P on an otherwise-clean doc). No operating
point is net-positive. Top features: sibling-text mass, parent tag,
sibling-anchor count.

This quantifies the selection wall for links exactly as 0011/0113 did for
images (3.4%/13% keep rates): the annotator's keep decisions depend on
rendered context invisible to DOM features. AUC-is-not-a-go-signal (0056)
holds again. No code changed.

With this, every autonomous lane in the ledger has a terminal
measurement. Remaining −0.0100: ~0.006 walls-by-construction, ~0.002
owner-gated gold rulings, ~0.002 diffuse hinted-wave residue at
~+0.0005/800k-token round.
