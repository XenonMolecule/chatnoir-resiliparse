# 0155: post-ruling tail re-check — no new gold lanes; extractor tail priced

Hypothesis after the 0143 rulings and 0151 sweep: the worst-20 tail might
hold more approved-family GOLD problems (menstennisforums was worth +0.90
on its doc). Re-examined all 22 worst docs with gold/pred size ratios.

**Result: the tail is extractor-side, not gold-side.**
- iclassifiedsnetwork (gold 22B, R=1.000) — gold is correct: the page has
  no article, only chrome. Our 619B of login/search widgets is OUR bug, and
  it is unreachable by site rules (no og:url, too few absolute links for the
  0109 majority-host fallback).
- wheretoget.it (gold 30B) — same shape; domain blocklisted.
- lafollettepress (gold 246B) — gold is the article; we add font-size and
  toolbar chrome.
- read718 (gold 2,722B vs our 316B) — the one genuine FEATURE gap: gold
  renders the registration form (labels + `______` blanks); we drop form
  controls.

Form-rendering family census: 11 golds with >=3 blank runs, but only
read718 is a real form — the rest are forum signature separators. Total
headroom if perfected: **0.0018 mean-F1**, concentrated in one doc.

## Closing position
No autonomous lane remaining is worth more than ~0.002, and most are
~0.0001/cycle. The −0.0082 F1 gap to 0.90 cannot be closed by grinding;
it needs the owner-gated items (train-gold authorization, image-in-gold
ruling) or research-scale work (computed-style formatting, JS rendering),
all scoped in 0149.
