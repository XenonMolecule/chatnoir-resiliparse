# 0066 — vBulletin 5 handler + golden v3

- **Date:** 2026-08-08
- **Tag:** 0066-vb5b (baselines: 0065 / golden-v3)
- **Status:** landed — zero regressions on all three targets

## Golden v3 (owner-flavor re-review)
Two agents re-reviewed all ~300 unlabeled specs against the four
rejection flavors distilled from the owner's 75 labels (calibrated on
the owner's actual rejected specs from git history): 269 confirmed /
7 dropped / 24 narrowed, all narrows mechanically verified.
Golden v3: 0.8374/0.7490 before this cycle's extractor work.

## vBulletin 5 handler
`li.b-post` posts (class token on <li>, not <div> — first fire attempt
returned zero), `div.author strong` author, `time` visible text,
`div.js-post__content-text` body with post-chrome skips (count chip,
duplicated OLD__post-date, h2 title). Announcement modules
(`div.announcement-tabs`) emitted before the thread — gold keeps them
(pacersdigest regression found and fixed in-cycle). Single-dated-post
exemption shared with 0065.

## Results
| target | F1 | Lev | Δ |
|---|---|---|---|
| golden v3 | **0.8383** | **0.7502** | +0.0009 / +0.0012 |
| original dev | **0.8257** | **0.7296** | +0.0009 / +0.0012 |
| train | 0.8145 | 0.7159 | +0.0001 / +0.0002 (8 up / 0 down) |

Zero regressions anywhere; general plain 1000/1000; goldens pass.
Golden Lev crossed 0.75.
