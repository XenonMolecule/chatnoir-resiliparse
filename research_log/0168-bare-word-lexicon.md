# 0168: held-back bare words — golden 0.89468, train up 390

0166 shipped 13 multi-word zero-gold lines but held back 14 bare words
(view, search, print, next, back, cancel, comment, source, writer,
language, admin, logo, hide, videos) on the reasoning that a single word
is weak evidence even with a zero corpus count. Tested as a batch:

| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | 0.89465 → **0.89468** | 0.81523 → **0.81528** | up 28 / down 9 |
| dev | 0.85763 | 0.76684 | up 27 / down 10 |
| train | 0.81640 → **0.81645** | 0.71828 → **0.71835** | **up 390** / down 97 |

All downs ≤0.016. Plain 1000/1000, tests 7/7.

The caution was unnecessary: a zero count across 1000 golds is strong
evidence regardless of line length, because these words only ever appear
as standalone UI affordances. Worth remembering — the corpus test is more
trustworthy than the intuition that overrode it.
