# 0104: site-rules wave 7 — golden 0.8710/0.7889; yields at the decay floor

## What
Seventh wave of domain-gated site rules. 440 targets (all sub-0.93 dev_golden
docs with |P−R|>0.01, blocklist-excluded), 8 workflow agents diffing against
the 0103-v2 extractor at the finest grain yet (50-char runs, 0.01 deficit
threshold). Raw haul: 969 vetoes + 450 whitelists → after the standard filter
(GENERIC tokens, `[\w-]+` selectors, blocklist, veto∩whitelist conflict drop,
dedupe vs existing tables): **+101 SITE_VETOES, +57 SITE_WHITELIST** appended.

## Battery + bisect
- First battery (`0104-wave7`): golden flat 0.8704/0.7880 — gains fully offset
  by 5 offender domains, incl. a −0.49 F1 crater on glassdoor.com.
- Bisected out: glassdoor.com, foodandwinechronicles.com, skinet.com,
  foodily.com, nutriacultivation.ru → added to `site_rule_blocklist.json`.
- **Bisect bug caught by the train scan**: the removal regex stripped ALL rows
  for offender domains — including wave-5/6 rows that were part of the shipped
  0103 baseline (foodily `#cards`, the glassdoor block, …). A lone foodily
  train doc moving −0.06 F1 was the tell. Fix: restored the 21 HEAD rows
  (15 veto + 6 whitelist) for those domains so the bisect removes only the
  wave-7 additions. Lesson for future bisects: **diff removals against HEAD,
  never regex-nuke a whole domain** — prior waves may own rows there.

## Result (`0104-v3` vs `0103-v2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | 0.8704 → **0.8710** | 0.7880 → **0.7889** | up 22 / down 4 / both 1 / craters 0 |
| dev (original) | 0.8450 → **0.8455** | 0.7513 → **0.7520** | up 21 / down 5 / both 2 / craters 0 |
| train | 0.8154 / 0.7170 | | up 2 / down 0 |

Worst residual: sparkpeople.com dev doc −0.018/−0.036 (sub-crater, accepted).
Guardrails: general/dev plain **1000/1000 byte-identical** vs 0030-respond;
`extract_golden` 7/7.

## Runtime (periodic check)
7-run best-of, single process, full 1000-doc lpv11 dev:
**markdown 0.95 ms/doc, plain 0.81 ms/doc**. The ~1800-row site tables are
domain-gated string compares — effectively free.

## Lane status
Wave yields: w5 +0.0043 → w6 +0.0025 → w7 **+0.0009** Lev (golden). The
site-rule lane has hit its decay floor — the remaining loss is not per-site
chrome at this grain. Goal gap after 0104: **−0.0290 F1 / −0.0111 Lev**
(golden basis). Next lanes: rendering-aware features (user-picked direction),
mid-band extractor work, and the user-gated train-gold review.
