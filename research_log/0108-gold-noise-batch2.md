# 0108: gold-noise batch 2 — golden v12 = 0.8746/0.7916

## Gold edits (from the 0105 tail taxonomy; all within approved families)
Specs (nav-chrome removal, family approved by owner):
- **beckett.com forum**: feedback form + login box + full megaMenu dropdown
  (~300 lines) removed; gold now starts at the first post. +0.259 F1.
- **census.gov**: the display:none mega-menu rollover panels (~130/153
  lines) removed; gold is now the NAICS trade article. (Doc F1 drops
  0.226→0.173 because our own extraction still emits the OTHER rollover
  panels — now measured honestly; see negative result below.)
- **multichannelmerchant.com**: 522-heading mega-menu block (2085 lines)
  before the actual search results removed. +0.807 F1.

Repairs (source-absence proof, phantom-typography precedent):
- **foodily.com**: verbatim TRIPLICATED no-results block deduped (source
  HTML contains it exactly once). +0.253 F1.
- **complex.com**: escaped-JSON app-state blob echoes (~15KB of other
  articles + a duplicated body paragraph) removed; gold is the rendered
  article. +0.839 F1.

Golden basis bump v11 → **v12**: 0.8725/0.7900 → **0.8746/0.7916** with the
unchanged 0107 extractor. Also removed 15 inert census.gov SITE_WHITELIST
rows (fitted to the old noisy gold; removal verified zero-effect).

## Negative result: model whitelist must keep honoring inline-hidden
Census's leftover failure looked like "model keep-score overrides
display:none" — a blanket filter dropping inline-hidden nodes from the model
whitelist fixed census but cratered train: **39 craters, 213 down** (goconqr
flashcards −0.87, nujournal −0.85, nassauda FAQ −0.75 …). Rendered pages
routinely REVEAL inline-hidden content via JS (tabs/accordions/lazy blocks)
and gold keeps it. Reverted to byte-parity. Census-style hidden-nav emission
is a per-site problem, not a global rule. (Fourth entry in the
hidden-content-is-bidirectional ledger.)

## Owner questions queued (need your ruling — content ADDITIONS)
1. **jeepforum** (998b734e): gold omits 7 fully-rendered posts our extractor
   correctly finds (one header even has an EMPTY body in gold). Restore them
   per charter "all rendered comments are content"?
2. **menstennisforums** (9c0edc63): gold is a 3× duplicated 298-byte thread
   header; the page renders ~15 full posts in "Topic Review". Restore?
3. **huskers.com** (e736c20c): gold keeps a CSS-hidden schedule carousel but
   drops the visible FAQ article body — remove carousel AND add FAQ, or
   leave as-is? (Both directions needed; husk risk if only removing.)

## Battery
Extractor unchanged from 0107 (parity verified: 0 changed docs on all
splits, plain 1000/1000, extract_golden 7/7).

## Scoreboard
Original dev **0.8467/0.7530** · golden v12 **0.8746/0.7916** · goal gap
−0.0254 F1 / −0.0084 Lev (golden basis).
