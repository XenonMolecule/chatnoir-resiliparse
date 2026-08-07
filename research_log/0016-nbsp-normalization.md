# 0016 — NBSP normalization (markdown mode)

- **Date:** 2026-08-07
- **Tag:** 0016-nbsp (baseline compared against: 0015-phpbb3)
- **Status:** landed

## Hypothesis
Predictions carried 11,518 U+00A0 chars across 632 dev docs; the gold keeps
only 307 (normalizes NBSP to plain space). Treating NBSP as collapsible
whitespace in markdown mode should be a pure Lev win.

## What changed
`get_collapsed_string_nbsp` (C2 A0 treated as whitespace, merged into space
runs), used in the serializer text path for markdown mode only. `<pre>`
content is untouched (code indentation legitimately uses NBSP — jusText
0056), and plain/minimal modes are byte-identical (guardrails unaffected).

## Results
Lev 0.6769 → **0.6785** (+0.0016); F1 exactly flat, 0 per-doc F1 changes.
Per-doc Lev strictly one-sided. Golden tests pass.

## Insights
- First pure-hygiene cycle; the family (double-encoded entities, mojibake,
  U+FFFD, gold's   number-typography) remains queued and is
  measurement-cheap now that the NBSP plumbing exists.

## Next
- Reply-form pages measured as a gold wall (gold repeats blocks 3×) — queued
  as such, not chased.
- Review incoming agent reports (gold audit, template subtraction, handler
  expansion) and integrate winners.
