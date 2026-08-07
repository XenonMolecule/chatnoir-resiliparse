# 0020 — WordPress comment attribution (native-first)

- **Date:** 2026-08-07
- **Tag:** 0020-wp3 (baseline compared against: 0019-ship)
- **Status:** landed

## Hypothesis
135 dev docs miss ≥2 of gold's `**author — date**` comment/post markers.
Rebuild WP-style comments (`li.comment` + theme selector cascade) as
`**author — date**\n body`, veto the originals, append after the walk.

## What changed
`wp_comment_rebuild` with three hard-won gates:
1. **Native-first**: gold mirrors each theme's own rendering ("1. Alex says:"
   ordered lists score BETTER left alone) — rebuild only when ≥half the
   authors are absent from the native walk output.
2. **Veto-only-rebuilt**: a comment whose selectors failed keeps its native
   rendering (veto-without-rebuild cost 14KB on one doc, −0.47).
3. ≥2 attributed and ≥half of items rebuilt, else no-op; dup-probe guard
   against rescue-swapped duplicates.

## Results
dev +0.0009 (11↑/1↓ sub-threshold); train +0.0011 (34 > +0.1 vs 6 < −0.1,
worst −0.26). Guardrails structurally untouched (markdown-only). Goldens pass.

## Insights
- "Gold mirrors the page" is the deep lesson: canonical-format rebuilds only
  pay where the page's own rendering loses information. The native-first
  check should retrofit onto future handlers.

## Next
- Taxonomy agent's battle plan (see analysis-taxonomy-0019.md): forum
  generic post-stream rebuilder, chrome related-modules tier, listing-card
  rescue, byline-anchor exemption, dl-table serialization, gold-noise
  exclusion list.
