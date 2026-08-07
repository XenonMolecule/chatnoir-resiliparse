# 0023 — Listing-card rescue — NEGATIVE, disabled (plumbing kept)

- **Date:** 2026-08-07
- **Tag:** 0023-listing{,2,3} measured; 0023-reverted (= 0022-final output)
- **Status:** abandoned for the rules era; knowledge banked

## Hypothesis
Taxonomy family #3 (+0.010 est.): tag/search/archive pages where gold keeps
every teaser card but the link-cluster rules keep 1 item. Retry with card
rules relaxed when output is a deep loss (<15% of body text).

## Measured variants (dev / train)
| gate | dev ΔF1 | train ΔF1 | train tail |
|---|---|---|---|
| deep-loss only, keep 2× | +0.0035 | — | one dev −0.62 (chrome-heavy article) |
| keep 4× | +0.0013 | +0.0011 | **32 docs < −0.3** (worst −0.82) |
| + card-grid page signal | −0.0003 | −0.0001 | 29 < −0.3; blocked the wins too |

## Insights
- The listing/article discriminator is NOT expressible from deep-loss +
  structural-repetition signals: chrome-heavy articles and true card grids
  produce identical shapes at the gate. Fourth confirmation this week that
  the remaining selection mass needs page-type awareness (0018 catalog wall,
  0022 related wall, audit nav wall, now listing).
- `RelaxFlags::listing_cards` and the rule-skip plumbing stay in place —
  the classifier era gets the gate for free.

## Next
- dl-table `**label:** value` serialization (family #5) — different physics,
  still rules-era viable.
- Learned page-type + node classifier groundwork begins (features exist:
  tpl stats, link densities, chrome vetoes, byline signals).
