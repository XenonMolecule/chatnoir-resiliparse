# 0156: stale FALLBACK_EXCLUDE entries — golden 0.8928/0.8131

`FALLBACK_EXCLUDE` (0109) opts domains out of the majority-link-host
`page_domain` fallback. It was fitted against golden **v12**; the gold has
since moved to v16 (four edit batches plus the owner's rulings). Nobody had
re-tested it — a config fitted to a moving target.

A/B with the list emptied, then re-tested per entry:
| domain | effect now | verdict |
|---|---|---|
| iclassifiedsnetwork.com | **0.087 → 1.000** | REMOVED |
| pt.usc.edu / usc.edu | 0.794 → 0.873 | REMOVED |
| theserverside.com | 0.920 → 0.055 | kept |
| menstennisforums.com | 0.948 → 0.008 | kept (its gold is now the owner-ruled version — the exclusion protects it) |
| bimmerwerkz.com | 0.453 → 0.096 | kept |
| motoprofi / cricketarchive / convertunits | −0.04 … −0.13 | kept |

## Battery (`0156-w1` vs `0154-parity`)
golden **0.89283 / 0.81314** (+0.0010/+0.0010), **up 2 / down 0**.
train unchanged (up 0 / down 0). dev shows iclassifieds −0.17 — the
original noisy gold for that doc disagrees with the audited one, the
0093 zero-sum family; golden-primary applies.
Plain 1000/1000, tests 7/7.

## Lesson
Blocklists and exclusion tables are fitted artifacts and **decay when the
gold basis changes**. Every such table in the codebase (site_rule_blocklist
.json, FALLBACK_EXCLUDE) should be re-A/B'd after a gold-basis bump, not
treated as permanent.
