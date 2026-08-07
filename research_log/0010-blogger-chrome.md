# 0010 — Blogger chrome handler (precision track opener)

- **Date:** 2026-08-07
- **Tag:** 0010-blogger (baseline compared against: 0009-markdown-v3)
- **Status:** landed

## Hypothesis

OVER_EXTRACT (155 docs) decomposes into forum chrome (48), table pages (42),
tag/archive (15), other (50) — the latter led by platform chrome escaping the
class regexes (Blogger's `post-share-buttons`: the social regex's boundary
list lacks plain `-`). A Blogger-gated chrome skip should recover precision
with zero effect elsewhere.

## What changed

- `is_blogger_doc`: `<meta name="generator" content="blogger…">` head check.
- When it fires (main_content only), the skip-selector set gains
  `.post-share-buttons, .feed-links, .blog-pager, #blog-pager, .post-labels`.
- **Gold-consistency check before building (playbook §7):** on 62 Blogger dev
  docs, gold keeps "Posted by X at" 28/62 (inconsistent → wall, NOT touched),
  Labels 7/62 and "No comments" 1/62 (consistent → dropped).
- Golden `table` fixture updated (it is a blogspot page; the diff is exactly
  the intended feed-links removal — first deliberate fixture change, logged).

## Results

| set | Δ mean F1 | per-doc |
|---|---|---|
| lpv11 dev (markdown cfg) | +0.0003, Lev +0.0004 | **49↑ / 2↓** (−0.011/−0.001 on already-broken docs) |
| general dev | +0.0001 | **25↑ / 0↓** |
| general dev2 | +0.0001 | **36↑ / 0↓** |
| lpv11 train | +0.0003 | 559↑ / 74↓ (7.6:1) |

Train's two worst regressions (−0.42/−0.29) are **rescue interplay**, not
handler errors: chrome removal pushed tiny image-blog pages under the tier-1
near-empty gate, which then swapped in the shell. Same accepted-tail class as
0006; dev and guardrails are clean.

## Insights

- Blogspot is everywhere: 62/1000 dev docs, and the handler improved 110 docs
  across the three dev-tier sets with zero meaningful regressions — platform
  handlers with signature gates remain the highest-precision tool in the kit.
- The aggregate F1 barely moves (+0.0003) while 600+ docs improve — chrome
  lines are short, so per-doc deltas are small. Precision work will look like
  this: many small wins, visible in per-doc counts, not the mean.
- Gold renders images as markdown `![alt](src)`! Spotted on image-blog golds.
  Our alt_texts currently emit bare alt text — a markdown-v2 item with real
  Lev/F1 upside on image-heavy pages (image URLs contain many tokens).

## Next

- Markdown v2: `![alt](src)` images, pipe tables, code fences, md links.
- Forum chrome family (48 docs) — needs per-engine handlers (the jusText M2
  arc); start with the phpBB/vBulletin signatures.
- Rescue-interplay guard idea (queue): don't rescue when the pre-rescue
  output already matches gold-like shape (heading + image lines).
