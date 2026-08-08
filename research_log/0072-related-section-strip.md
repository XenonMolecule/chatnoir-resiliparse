# 0072 — Related/teaser section strip

- **Date:** 2026-08-08
- **Tag:** 0072-v2 (baseline: 0071-social)
- **Status:** landed

## What changed
Post-pass `strip_related_sections`: heading in the related-content
family ("You May Also", "Related Articles/Posts/...", "More From",
"Around the Web", "Trending", ...) plus its section, guarded four ways
(≤25 lines; ≤max(600, 15%-of-doc) chars; section ≤25% of doc — the
quote-site crater fix; no prose line >200 chars — the tampabay guard).
Python-prototyped on both targets before the Rust port (two guard
iterations measured).

## Results
| target | F1 | Lev | vs 0071 |
|---|---|---|---|
| golden | 0.8405 | **0.7526** | 0 / +0.0001 |
| original dev | **0.8282** | **0.7323** | +0.0003 / +0.0005 |
| train | 0.8147 | 0.7161 | +0.0001 / +0.0002 |

Zero craters on any split (177 train docs up). linuxpromagazine golden
both-down −0.25: golden still KEEPS a related section there — fleet
miss by charter rules, queued for golden v4. General plain 1000/1000;
goldens pass.
