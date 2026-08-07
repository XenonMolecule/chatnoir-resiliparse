# 0037 — Byline microdata recovery + tier-1 keep-test hardening

- **Date:** 2026-08-07
- **Tag:** 0037-v4 (baseline: 0036-v6)
- **Status:** landed

## Hypothesis
The train census's #1 pred-only line, "Posted by at" (300 docs, gold keep
0%), is not chrome — it is a DEGENERATE byline. Gold keeps "Posted by
NAME at TIME"; we emit the bare glue words because the inherited itemprop
veto (`author`/`datePublished`/`url` → drop, Cython reference behavior)
kills the visible `span[itemprop=author]` / `abbr[itemprop=datePublished]`
carrying the name and time. Recovering them aligns 300+ docs.

## What changed
1. **itemprop exemption** (mirrors 0022's rel=author fix): in markdown
   config, visible (non-meta/link) `itemprop=author|datePublished` nodes
   with byline ancestry and text ≤80 bytes are kept. `itemprop=url`, plain
   config, and meta/link carriers keep reference behavior. The size cap
   keeps author BIOS dead (bonsaifinance regression found and fixed —
   the 0033 veto family must not resurrect through this door).
2. **BYLINE_CLS extended** with timestamp|published|post-footer|
   entry-date|post-date so timestamp containers qualify as byline context.
3. **Tier-1 keep test hardened** (the cycle's structural find): raw-length
   OR content-length clearing the 20x factor accepts the rescue. Each test
   alone flipped catastrophically at the margin when a few byline bytes
   landed in a tiny base output (happysadlola −0.76 dev, blogtalkradio
   −0.37 train — both resolved by the OR-frontier; third instance of the
   0012/0036 gate-instability class, now closed on both sides).

## Results
| split | F1 | Lev | vs 0036 |
|---|---|---|---|
| dev | **0.8061** | **0.7072** | +0.0003 / +0.0001 |
| train | 0.7965 | 0.6933 | +0.0003 / +0.0002 |

dev 52 improved / 21 down; train 474 improved / 211 down (98 both-down,
worst −0.08). Logged decision: the residual downs are docs where gold
DROPS bylines — listing/archive pages with per-teaser bylines and
scattered per-doc inconsistency (page-type wall family); improvements
dominate 2.3:1 and both aggregates rise on both splits. General dev
plain: 1000/1000 identical. Goldens pass; fences balanced; 1.44 ms/doc.

## Insights
- "Gold keep-rate 0%" on a SHORT line can mean degenerate-fragment, not
  chrome: gold keeps the full version our extraction truncated. Check
  whether gold contains a superstring before classifying as strippable.
- Rescue-gate margins are now dual-sided (gate: content_len with |#*-
  excluded + content-table veto; keep: raw OR content). Serializer and
  selection changes should stop flipping tiny-output docs.
