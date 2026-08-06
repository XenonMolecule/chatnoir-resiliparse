# 0005 — `<ul>` rule repair (gated text-heavy-list rescue)

- **Date:** 2026-08-06
- **Tag:** 0005-ship3 (baseline compared against: 0004-rescue)
- **Status:** landed (deliberately conservative variant; bigger variant queued)
- **Machine:** cloud VM (e2-standard-4, x86).

## Hypothesis

The `<ul>` blacklist rule (reject if depth<4 OR link-ratio>0.2) was 0003's
largest catastrophic bucket (13 docs): it breaks on list-structured main
content. An exemption for text-heavy lists should recover that bucket without
readmitting nav menus.

## What changed

Tier-2 rescue in the Rust extractor (after the 0004 near-empty tier): when a
rescue-eligible `<ul>` was dropped AND output < 30% of the collapsed body
text, re-extract with the exemption active; keep only if >2× larger and not a
duplication (mid-output probe appearing twice — Blogspot renders posts inside
list widgets). Exemption = list with ≥1000 bytes collapsed text, link ratio
<0.5, **at most ~1 link per item**, and ≥150 bytes non-link text per item.
Dropped-UL eligibility is tested lazily (only when the gate fires) and
full-page text is materialized lazily — the fast path pays only a pointer
push per dropped list.

## Results

| set | 0004 F1 | 0005 F1 | per-doc |
|---|---|---|---|
| lpv11 dev | 0.7782 | **0.7789** | +1 / −0 |
| general dev | 0.8107 | **0.8113** | +1 / −0 |
| general dev2 | 0.8131 | **0.8136** | +1 / −0 |

Speed (interleaved vs frozen Cython yardstick): rust/cython ratio 0.818 vs
0.811 pre-rescue → both rescue tiers together cost ~1% relative throughput.
(Wall-clock best-of-3 readings of +5–10% were machine drift; the interleaved
ratio is the trustworthy instrument on this VM.) Golden tests pass.

**The journey matters more than the delta.** Successively rejected variants,
all measured (tags kept):
1. Exemption always-on: dev +0.0030 but 6 regressions (worst −0.28) — text
   mass alone can't tell list-content from text-heavy junk lists.
2. Gated (output<30% body, keep>2×): dev +0.0025, 6↑/0↓ — but guardrails
   regressed (−0.44/−0.53): a FAQ *index* (link+keywords per item) and a
   Blogspot *blogroll with snippets* both pass the text-mass test.
3. +per-item density (≥150B non-link/item): killed the FAQ index; blogroll
   survived (snippets are dense).
4. +duplication guard: caught template-duplicated posts, blogroll still
   passed (its content is unique text).
5. +≤1 link per item: kills blogroll and FAQ index — but also 9 of the 12
   would-be wins (real list-content often has multiple links per item too).

Variant 2+3+4 (F1 +0.0019/+0.0026/+0.0013, 12↑/1↓ total, single −0.46 junk
admission) is strictly better on aggregate but violates the zero-regression
rule; shipped 5 instead and queued the relaxation.

## Insights

- "Text-heavy list" is not a sufficient main-content signal: FAQ indexes,
  blogrolls-with-snippets, and link directories are all text-heavy. The
  missing signal is *container context* (Blogger/WordPress `widget` classes
  on ancestors) — the current classifier has no widget concept at all.
- The rescue-ladder architecture (tier-1 near-empty → tier-2 targeted retry)
  composes cleanly and keeps each fix's blast radius measurable; new
  rule-repairs can slot in as further tiers or exemptions behind gates.
- Lazy gating matters at this speed scale: materializing full-page text on
  every doc costs ~5–8% — always order gates cheapest-first and cache.

## Next

- Queued: widget-class junk signal (would unlock variant 2+3+4's +12 docs).
- 0006 candidate: `<article>` teaser link-cluster (8-doc bucket from 0003),
  same rescue-tier pattern.
- Also queued from 0003: landmark_id/footer regex tightening
  (`wsite-not-footer`), forum over-extraction track.
