# 0040 — Comment-template probes: small ship + two negatives

- **Date:** 2026-08-07
- **Tag:** 0040-v3 (baseline: 0039-v4)
- **Status:** landed (small); two variants measured negative and reverted

## Context
608 train docs still miss ≥half their gold `**X — TIME**` bylines:
~501 unhandled forum engines (long tail), 56 WordPress-ish, 39
Blogspot-ish, 6 Disqus (content behind iframe — unfixable), 4 MT/TypePad,
2 LiveJournal.

## Shipped (strictly clean: 3 docs improved, zero regressions)
WP rebuild selector fixes: author falls back to bare `.c-head` text minus
"permalink" (Highlander theme); date candidates now require a digit —
otherwise the `.c-head span` permalink label becomes the "date"; `.c-date`
added to candidates; `.datetime` added on the Blogspot threaded branch.
These fire only under the existing native-first gate.

## Negative 1: Highlander force-rebuild
"wordpress.com Highlander gold is always rewritten" is FALSE — forcing the
rebuild past the native-first check cratered docs whose gold keeps the
native rendering (ericalscott −0.28, blowoutbuzz −0.22). The 0020
native-first gate encodes exactly this per-doc gold variance. Reverted.

## Negative 2: old-Blogger blogComment branch
`span.[anon-]comment-author` + `div.blogComment` doc-order pairing
mispaired on nested/mixed archive pages (claudinehellmuth −0.24 — the
very doc it targeted). The 201-vs-47 node-count mismatch was a warning,
not a pairing puzzle. Reverted; family needs per-post scoping, queued.

## Results
dev **0.8077 / 0.7095** (+0 / +0.0001), train 0.7983 / 0.6954
(+0 / +0.0000, 2 docs improved). Goldens pass.

## Insight
Comment-convention rebuilds divide into: (a) gold rewrites ALWAYS
(Blogspot "said..." family — 0039 shipped), (b) gold mirrors native
(generic WP — native-first gate correct), (c) gold varies per doc
(Highlander, old-Blogger) — for (c) neither forcing nor skipping wins;
only a per-doc signal could arbitrate. 10th wall confirmation.
