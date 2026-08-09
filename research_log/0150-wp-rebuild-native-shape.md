# 0150: WP comment rebuild fires on native shape — golden 0.8915/0.8117

## The bug (found via the 68-doc header-recall census)
`wp_comment_rebuild` produces gold's `**Name – date**` comment headers, but
it only ran when the authors were MISSING from the main extraction
(`missing * 2 >= authors.len()`). On WordPress.com themes the authors ARE
present — in WordPress's own `**Name** says:` rendering, which gold
rewrites. So the correct rebuild was computed and then discarded, leaving
the inferior shape.

Fix: also trigger when the native rendering is detected (>=4 "says:"
markers with >=4 attributed comments). The >=2 threshold was tried first
and was mixed (golden 1 both-down, train 4); >=4 keeps the large threads
where the rebuild clearly wins and leaves short ones alone.

## Battery (`0150-w2` vs `0148-w1`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | 0.89150 → **0.89153** | 0.81168 → **0.81170** | up 1 / down 1 / **both 0** |
| dev | 0.85687 → 0.85689 | 0.76603 → 0.76599 | — |
| train | 0.81612 → **0.81615** | 0.71805 → 0.71803 | up 7 / down 6 / both 1 |

Golden improves on both metrics with zero both-down docs; shipped under the
golden-primary policy. Plain 1000/1000, tests 7/7.

## Note on the census that found it
The 68-doc "gold has post headers we don't produce" census also covers
41 no-generator forum templates and several one-off engines — each would
need its own handler. This WP case was the only one where the machinery
already existed and was merely mis-gated.
