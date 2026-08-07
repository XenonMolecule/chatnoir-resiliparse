# Template subtraction — wild-idea agent report (2026-08-07)

**Idea:** intra-page structural template subtraction — boilerplate is built
from repeated sibling subtrees (nav lists, card grids); main content is
structurally diverse. Score containers by sibling-signature repetition, drop
when repetition ∧ link-density.

## Measured (prototype over naive text dump, NOT our extractor)

- 200 dev docs: **F1 +0.0804** (P +0.130, R −0.042), 162↑/20↓.
- Held-out 200 docs (untuned): **+0.0617**, 155↑/27↓ — real, not overfit.
- Cross-page variant (same-host verbatim-line subtraction): +0.0674 on
  covered docs, cleaner tail, but only 15–27% of train has ≥2 docs/host —
  complement, not substitute.

## The formula
Depth-2 structural hash per child (tag + digit-normalized classes + child
sigs); drop container iff ≥3 repeated-sig children AND ≥50% of children
repeated AND link-density ≥0.6 AND container ≤50% of body text.
**Link-density conjunct is load-bearing**: a short-text alternative path
flipped the result to −0.03 (kills content lists/tables). Signature depth
barely matters; sibling repetition count does the work.

## Why it matters for us
Catches what class regexes structurally cannot: classless `<ul>`s (blogroll
case from cycle 0005!), BEM classes (`panel__container--b`), site-idiosyncratic
ids — each example +0.37..+0.58 F1 on its doc. Engine-agnostic; this is the
answer to the per-engine handler treadmill's long tail.

## Failure mode
Pages whose gold content IS repeated link-dense structure (course modules,
speaker indexes, link-FAQ pages) — needs a page-level guard (skip when body
link-density >0.7) and/or the audit's "article-ness" condition.

## Recommended port (agent's + my assessment)
Run only on containers **no regex matched**, behind the compound gate, as an
additional `is_main_content_node` veto — measured through the standard
zero-regression battery. Prototype: benchmark/experiments/template_subtraction_prototype.py.
