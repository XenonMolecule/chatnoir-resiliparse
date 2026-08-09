# 0118: related-section family extension — golden 0.8878/0.8073

## What (cheap inline cycle — no fleet, per owner token concern)
`strip_related_sections` heading vocabulary extended with taxonomy families:
similar tracks, latest stories/news/headlines, more stories, most
read/popular/viewed, sponsored links, from our partners, editor's picks,
gallery links, related medicine news.

Battery-trimmed three candidates:
- "register now" — content on registration pages (ldsmingle −0.15);
- "similar threads/topics" section-form — original train golds keep them
  (plantedtank −0.125; the 0107 line-level table strip already covers the
  golden-relevant cases);
- "sponsored content" — weatherbug's owner-ruled husk gold keeps it as an
  orphan heading (golden −0.004 → dropped the token, golden now clean).

## Battery (`0118-w3` vs `0117-v2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8878** | **0.8073** | up 5 / down 0 |
| dev | 0.8557 | 0.7643 | up 5 / down 1 (mixed-sign) |
| train | **0.8159** | **0.7175** | up 69 / down 26 (11 both, all ≤0.038 — original-gold teaser-keeps, charter noise) |

Guardrails: plain 1000/1000 identical, extract_golden 7/7. Generic lane —
transfers to test.

## Scoreboard
Original dev **0.8557/0.7643** · golden v12 **0.8878/0.8073** · goal gap
**−0.0122 F1** (Lev in-band).
