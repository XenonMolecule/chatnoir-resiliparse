# 0042 — vBulletin variant coverage (vB4 gate + author fallback)

- **Date:** 2026-08-07
- **Tag:** 0042-vbvariants (baseline: 0041-subsilver)
- **Status:** landed (small)

## What changed
1. Markup dispatch gate now also accepts vB4 pages with
   `li[id^=post_] blockquote.postcontent` >=2 (no `post_message_` divs,
   generator meta stripped — javaprogrammingforums family).
2. Author fallback: skins with an empty-class profile anchor (rcgroups)
   attribute via the first `a[href^="member.php"]` with 1–40 char text.

## Results
dev **0.8086 / 0.7109** (flat aggregate; 2 docs improved, 1 mixed-sign).
train 0.7986 / 0.6959 (+0.0001/+0.0001; 12 improved, 2 both-down worst
−0.005 — a wrong-author pick on redszone, logged). Goldens pass.

## Notes
Remaining vB misses are vB5 (`b-post` structure, no legacy ids) and
printthread.php print views — different families, queued. Byline dash
(em vs en) is per-doc annotator variance (61/39 on vB docs) — do not
chase; census must stay dash-agnostic.
