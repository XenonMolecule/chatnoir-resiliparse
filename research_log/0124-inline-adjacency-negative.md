# 0124: inline-adjacency "fixes" — NEGATIVE result, reverted

Band-taxonomy emitter-hygiene candidate: (a) space after closing `**` fused
to a word ("**bargaining**noun"), (b) collapse doubled lines (img alt ==
adjacent link text). Both repro'd and unit-verified.

Battery verdict: **golden down 59 / up 22, train down 636 / up 241** —
nearly all pure-Lev losses at identical F1. The gold pipeline FUSES
adjacent inline elements exactly the way our emitter does (usms heat
sheets, IBM docviews, redhat changelogs all keep "**X**y" fused); the
taxonomy's cambridge/scienceblogs examples where gold has the space are
the minority. Fixing the artifact moves output away from gold. Census≠
action instance #8 — and a new corollary: **gold mirrors source-adjacency;
emitter "hygiene" that renders more faithfully than the annotator pipeline
is anti-progress by construction.**

Also caught during iteration: `str::split_at` on a byte midpoint panics on
non-boundary UTF-8 (first battery: 70 golden craters at −1.0 — panics
zeroed whole docs). Any future line-halving code must guard with
`is_char_boundary`.

Fully reverted; parity vs 0123-v2 verified (0 changed docs).
