# 0019 — Structural template subtraction (the wild idea lands)

- **Date:** 2026-08-07
- **Tag:** 0019-ship (baseline compared against: 0018-chrome7)
- **Status:** landed — largest single-cycle quality gain of the project

## Hypothesis
Boilerplate is built from repeated sibling subtrees; main content is
structurally diverse. The agent prototype measured +0.06–0.08 F1 (held-out
confirmed) with repetition∧link-density as the load-bearing conjunct.

## What changed
Bottom-up `tpl_scan` over the body (single pass, once per document, reused by
rescue retries): per-element depth-2 structural signature (tag +
digit-normalized classes + child sigs, concat-free hashing). A container is
vetoed (joins the skip set) iff ≥3 repeated-signature children, ≥50% of
children repeated, link-density ≥0.7, ≤2500 collapsed bytes, ≤30% of body
text. Page-level guards: skip on thin pages (<1500B — repeats ARE the
content there) and listing-like pages (body link-density >0.7). Markdown
config only; applied to primary, fallback, and retry extractions.
Rejected variants (measured): 0.6 link-density (island-card listings),
no size caps (photo blogs, verse lists, instruction pages — 33 catastrophes),
image-dominance guard (cost 0.0008 mean for 1 doc of tail).

## Results
| set | 0018 | 0019 | per-doc |
|---|---|---|---|
| lpv11 dev F1 | 0.7895 | **0.7935** (+0.0041) | 193↑ / 70↓ |
| lpv11 dev Lev | 0.6843 | **0.6917** (+0.0075) | |
| lpv11 train F1 | 0.7854 | **0.7891** (+0.0037) | 175 > +0.1 vs 94 < −0.1, 20 < −0.3 |
| lpv11 train Lev | — | +0.0062 | |
| general dev/dev2 | — | — | 0 changed |

**Logged decisions:** (1) the residual tail (20 train docs < −0.3) is the
structure-blind wall — pages whose gold content is itself repeated link-dense
structure (personal-blog sidebars gold keeps, photo pages, catalog cards);
listed for the learned-classifier/article-ness era. (2) Speed: the scan costs
~10% relative (ratio vs frozen Cython 1.19× → 1.05–1.08× after single-pass +
concat-free-hash optimization) — **markdown-config only; the plain-text
default path is untouched at 1.19×**. Further criterion profiling queued.

## Insights
- The wild-idea pipeline (agent prototype → gate spec → port → guard rounds)
  went measurement-first the whole way; every guard was bought with a named
  failure family, not intuition.
- Rescue-gate/formatting interactions strike every subtractive change; the
  content-length gates (0012) and once-per-doc veto reuse are now standard
  plumbing.
- Regex chrome work (0018) and template subtraction overlap ~30% — the
  structural signal catches classless/BEM/site-idiosyncratic containers the
  audit predicted regexes never would.

## Next
- Criterion profile of tpl_scan; possible SmallVec/arena pass.
- Learned per-node classifier remains the endgame for the walls (M3).
