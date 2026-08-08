# 0106: tier-0a rooted near-empty rescue — golden 0.8722/0.7899

## What
New rescue tier ahead of the model-rollback/unfiltered tiers (markdown only):
when the base main-content pass comes back near-empty (<200B content) and the
page names exactly one article container (`.entry-content`, `.article-body`,
`.articleBody`, `.article-text`, `.post-content`, `.postcontent`,
`[itemprop=articleBody]`), rescue from THAT container instead of the whole
body. The body-wide tiers resurrect the site shell on husk pages — gascu
(F1 0.000, tier-0 rollback returned pure nav) is the type specimen; the
rooted crop returns the actual benefits list.

## Two battery-found guards (census≠action, again)
1. **Engine pages are excluded** (generator meta present): Blogger/Typepad/
   WordPress golds keep title + comments OUTSIDE the article container —
   first battery cratered prefersimplicity.blogspot −0.62, scrabble66.typepad
   −0.36. Their rescues stay with the engine handlers/body-wide tiers.
2. **Generator meta can be in `<body>`**: fraudswatch's theme puts an `<img>`
   inside `<head>`, which force-closes head at parse time and dumps the
   WordPress generator meta into body — the head-only scan missed it and the
   doc kept regressing through the "gated" build. The gate now also queries
   `meta[name=generator]` under body. Worth remembering: **any head-scoped
   heuristic silently fails on head-broken markup.**

## Battery (`0106-w3` vs `0105-w4`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8722** (+0.0008) | **0.7899** (+0.0006) | up 2 / down 0 |
| dev | **0.8468** (+0.0008) | **0.7531** (+0.0006) | up 2 / down 0 |
| train | **0.8155** | **0.7170** | up 3 / down 0 |

Zero regressions on all splits. Guardrails: plain 1000/1000 identical,
extract_golden 7/7.

## Scoreboard
Original dev **0.8468/0.7531** · golden **0.8722/0.7899** · goal gap
−0.0278 F1 / −0.0101 Lev (golden basis).
