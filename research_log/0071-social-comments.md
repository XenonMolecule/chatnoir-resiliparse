# 0071 — WP Social-plugin comment selectors

- **Date:** 2026-08-08
- **Tag:** 0071-social (baseline: 0070-onecomment)
- **Status:** landed — zero regressions (8th consecutive)

## What changed
WP rebuild selector set: `li.social-comment` items, `cite.social-fn`
author, `.social-comment-meta a` date, `.social-comment-body` body
(the Social plugin theme; thebruinsblog probe F1 0.43 → ~0.95).

## Results
golden **0.8405/0.7525** (+0.0006/+0.0007) · original dev
**0.8279/0.7318** (+0.0006/+0.0007). Zero regressions; general plain
1000/1000; goldens pass. Golden crossed 0.84 F1.
