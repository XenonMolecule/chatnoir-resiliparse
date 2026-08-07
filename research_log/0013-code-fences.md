# 0013 — Markdown code fences

- **Date:** 2026-08-07
- **Tag:** 0013-fences (baseline compared against: 0012-final2)
- **Status:** landed (quality-correct at Δmetric≈0, per §6)

## Hypothesis
Gold wraps <pre> content in ``` fences (30 dev docs, language-tagged); the
pre_depth machinery already preserves the verbatim text, so fencing is a
serializer wrap.

## What changed
Markdown mode wraps non-empty `<pre>` blocks (outside tables) in bare ```
fences: margin-flushed opener, newline-guard between opener and code, closer
after rstrip, empty-pre collapse, childless-pre skip (no end event).
Language detection deferred.

## Results
36 dev docs changed: ΣΔLev +0.039 (17↑/14↓ at ±0.0005), ΣΔF1 exactly 0.0000;
aggregates unchanged (0.7819/0.6712). Train flat. Guardrails untouched
(markdown-only path). Golden tests pass. Output verified: fences render as
code blocks.

## Insights
- Gold's fences carry language tags (```python/```vba) and sometimes wrap
  content we extract un-fenced from <code>-only markup (no <pre>) — the
  Lev-down half of the changed docs. Language inference + <code>-block
  fencing (jusText 0056 analog, br-gated) are the follow-ups.

## Next
- Forum chrome family (48 OVER docs) — largest untouched bucket.
- Calendar/empty-cell table veto; ```lang inference; <code>-block fences.
