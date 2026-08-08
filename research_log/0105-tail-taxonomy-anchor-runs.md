# 0105: tail taxonomy + anchor-run/template vetoes — golden 0.8714/0.7893

## Fresh tail taxonomy (the input for the next several cycles)
6-agent fleet classified all 54 dev_golden docs with F1<0.5 on 0104-v3
(loss mass 0.0362 — more than the whole remaining F1 gap):
**over_chrome 22 · under_missed_content 17 · gold_noise 7 ·
hidden_content_extracted 3 · near_empty 2 · engine_missing 1 ·
css-hidden lanes ≤5 total.** Full per-doc reports:
`scratchpad/…/tasks/wmeehals2.output` (workflow wf_5e1ca161-68d).

Rendering-aware verdict: **weak lane.** In-page census confirms it — only
7/1000 docs leak stylesheet-hidden text (~3.7KB total), most hiding lives in
*external* CSS we don't have, and gold sometimes KEEPS in-page-hidden content
(caiso, tradingview i-hidden chips, ifood overlays). Dropped as a lane;
the tail is classifier patterns.

## Shipped this cycle (markdown config only)
1. **Inline anchor-run veto** (`is_anchor_run`): `<p>/<small>/<font>/<dd>`
   blocks with ≥25 anchors, ≥300B text. Two safety rails found by battery,
   not census (the law holds):
   - avg link length ≤ 60 — anchor-wrapped story teasers are content
     (wfrn.com news archive, −0.75 train crater at first cut);
   - non-link non-space bytes ≤ 32 (absolute, not ratio) — thread lists and
     product cards interleave dates/prices between anchors (xoxohth −0.35
     crater at ratio 0.8; ratio 0.9 missed rosinstrument because short
     labels dilute the ratio with separators). A pure link index has
     essentially zero non-link ink; that is the real signature.
2. **Template-token line strip** (post-pass): lines matching
   `\$obj.method()` (Velocity) or `translation_missing` — unrendered
   client-side placeholders. Line-level, indented code exempt. First cut as
   a per-element veto was reverted for runtime (get_node_text on every node).

## Battery (`0105-w4` vs `0104-v3`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8714** (+0.0004) | **0.7893** (+0.0004) | up 2 / down 0 |
| dev | **0.8460** | **0.7525** | up 2 / down 0 |
| train | 0.8154 | 0.7170 | up 2 / down 1 (pcworld −0.069 F1, Lev flat — original-gold shopping widget, accepted) |

Movers: rosinstrument +0.354 F1, carfax +0.056 F1. Guardrails: plain
1000/1000 identical, extract_golden 7/7. Runtime (7-run best-of, 1000 docs):
markdown 0.95 ms/doc, plain 0.81 ms/doc.

## 0104 postscript — bisect-vs-HEAD lesson
Wave-7 bisect initially regex-nuked ALL rows for offender domains including
shipped wave-5/6 rows; a lone foodily train doc −0.06 was the tell. Bisects
must only remove rows absent from HEAD.

## Queued from taxonomy (next cycles)
- Related-teaser tables/lists: "Similar Threads" (vBulletin), "Similar
  Tracks", recent-gallery teasers, related-flashcards fieldsets (≥5 docs).
- Link-list rescues: sole-entry-content link lists (gascu 0-byte), catalog
  `ul.stampspec`, cast-crew card lists, `<ul>` with bare text + no `<li>`,
  hotel-deal lists under kept headings (≥6 docs).
- Footer copyright/disclaimer rescue (charter C4; sedo, tradingview,
  genealogy) — battery-risky, needs its own cycle.
- Modal/dialog + popup vocabulary vetoes (print/email/login modals).
- Gold-edit proposals for owner review: 7 gold_noise + 1 snapshot-mismatch
  (bikeforums 7 dropped posts), incl. census.gov/beckett megaMenu nav (within
  the approved nav-chrome family) and foodily triplicated no-results block.
