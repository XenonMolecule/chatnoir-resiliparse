# 0075 — Safe batch: UI strips + unambiguous Blogspot widgets

- **Date:** 2026-08-08
- **Tag:** 0075-v3 (baselines: golden-v6 / 0072-v2)
- **Status:** landed (golden-primary decision)

## What changed
Seven strip lines (wheretoget UI trio, "Please follow and like us:",
protected-content error, census promos) + Blogspot widget subset
(LinkList/BlogArchive/FollowByEmail/Attribution/Followers — Image/
Profile/Label excluded after the 0074 crater isolation). "More like
this" related-heading tried and dropped (gold-kept on djangosnippets).

## Results
golden **0.8454/0.7572** (9 up / 0 down). original dev 0.8281/0.7322
(7 up / 2 down — BOTH regressions are docs whose ORIGINAL gold keeps
the chrome and whose GOLDEN score improves: original-gold noise, first
formally logged golden-primary divergence). train flat (27 up / 20
down, worst −0.01 legit). General plain 1000/1000; goldens pass.

## Also
huskers triage entry was stale+hallucinated selectors (already 0.93 on
v6; `div.event` absent from html) — engine lane confirmed drained.
