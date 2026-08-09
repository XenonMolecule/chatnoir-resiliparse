# 0166: zero-gold lexicon batch 2 — golden 0.89465/0.81523

Second pass of the 0165 method at a lower threshold (>=2 docs, lines <70
chars): 112 candidates. Shipped 13, excluding bare words that are
plausible content elsewhere (view, search, source, writer, language,
admin, back, next, comment, cancel, logo, hide) — the corpus test says
they never appear in THESE golds, but a single-word line is weak evidence
and the exclusions cost nothing.

Shipped: liked by 1 person, reply ↓, more..., related articles, please
reload, more news, forum jump, print article, email article, welcome!,
facebook, twitter, edit.

| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | 0.89457 → **0.89465** | 0.81507 → **0.81523** | up 24 / down 4 |
| dev | 0.85755 → **0.85763** | 0.76667 → **0.76684** | up 25 / down 6 |
| train | 0.81636 → **0.81640** | 0.71824 → **0.71828** | up 145 / down 66 |

All downs ≤0.012. Plain 1000/1000, tests 7/7.
