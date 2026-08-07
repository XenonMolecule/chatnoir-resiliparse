# 0043 — XenForo handler (both generations)

- **Date:** 2026-08-07
- **Tag:** 0043-final (= 0043-v3; baseline: 0042-vbvariants)
- **Status:** landed

## What changed
`extract_xenforo` (jusText-0058 map, adapted): XF1 `li[data-author]` +
`blockquote.messageText` + `span.DateTime` title-attr time (" at " → " ");
XF2 `article[data-author]` + `div.bbWrapper` + `<time>` where gold uses
the visible date only (data-date-string). En-dash joiner per gold.
Quote blocks stripped: gold strips them in 36/49 quote-bearing XF docs —
the OPPOSITE of jusText's gold. A duplicate-only-strip variant (keep
novel quotes) measured worse — whitespace/emoji normalization made real
reply-quotes look novel; blanket strip shipped, smogon (train, −0.50,
a quote-keeper doc) logged as the 27%-side cost against 24 improved.

## Results
| split | F1 | Lev | vs 0042 |
|---|---|---|---|
| dev | **0.8089** | **0.7115** | +0.0003 / +0.0006 |
| train | 0.7988 | 0.6962 | +0.0002 / +0.0003 |

dev 2 improved / 0 regressed (byline-exact on both probe docs). train
24 / 3 (2 both-down: smogon + army.ca −0.035). General dev plain
1000/1000 identical; goldens pass.

## Lesson re-learned (3rd time)
Patch-script heredocs MUST use raw strings — a `\u{2013}` in a non-raw
triple-quote silently failed the patch while the eval measured the old
binary as if it were the new one. Caught by metric identity.

## Follow-up (0043b, same day)
The blanket quote-strip reduced `total` below the 25% coverage guard on
quote-heavy threads — the handler located the thread, then disqualified
itself (homebrewtalk fell back to the generic walk in the harness while
the pre-strip spot-check passed). Guard now counts each post's FULL
pre-strip mass. dev **0.8090 / 0.7116** (+1 doc); train 0.7988/0.6964
(8 up / 3 down; gen3insight −0.15 — a quote-keeper gold where the guard
rejection had been accidentally protective; 27%-side cost, logged).
