# 0120: inline tail-doc rules — golden 0.8879/0.8074

## What (owner-economical: per-doc analysis done inline, no fleet)
Hand-verified rules for deep-tail OVER docs: theday.com (.galleryCats,
.verticalbox gallery-teaser rail), newhampshire.com (.hover-navigation,
.image-crop contest promos), iclassifiedsnetwork.com (.SideModule).
Generic line strips: "[Buy Photo]" labels; ad-targeting machine lines
containing "| adString:"/"| zoneID:".

## Result (`0120-w1` vs `0119-w1`)
golden **0.8879/0.8074** (up 2 / down 0) · dev zero-sum (theday) · train
up 3 / down 0. Plain 1000/1000, extract_golden 7/7.

Note: iclassifieds rules are inert — no og:url and too few absolute links
for the majority-host fallback; its junk is sitewide-template chrome only a
URL-based domain signal could gate (we don't use the eval URL by design —
the extractor sees only the HTML).

## Tail reality check
The remaining −0.0121 F1 gap concentrates in ~35 sub-0.5 docs of which the
largest single-move candidates are OWNER-GATED gold restorations
(menstennisforums 0.046 → ~0.9 if Topic Review posts are restored;
jeepforum 0.475 → ~0.9 with its 7 dropped posts; huskers 0.052) — see
0108 questions. JS-only-content docs (foodandwinechronicles Instagram feed)
are unreachable by construction.
