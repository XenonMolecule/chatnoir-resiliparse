# 0107: forum/attachment boilerplate strips — golden 0.8725/0.7900

## What (all line-level post-pass strips, the safe action class)
- **"Similar Threads" teaser table**: a single-cell `| Similar Threads |`
  header row opens the block; rows stripped until the first non-table line
  (vBulletin family — bimmerwerkz, futures.io).
- **"Currently Active Users Viewing This Thread: …"** status line (<90 chars).
- **WordPress attachment suffix**: "… | Full size is 640 × 425 pixels" is cut
  from the byline line, keeping the byline itself (thelosangelesbeat family).
- New exact labels: "Bookmark the permalink(.)", "LinkBack".

## Battery (`0107-w1` vs `0106-w3`)
| split | F1 | Lev | notes |
|---|---|---|---|
| dev_golden | **0.8725** (+0.0003) | **0.7900** (+0.0001) | up 3 / down 1 (losangelesbeat F1 +0.12, Lev −0.04 mixed) |
| dev | 0.8467 (−0.0001) | 0.7530 (−0.0001) | zero-sum: original gold keeps the teaser tables golden cuts |
| train | 0.8155 | 0.7170 | futures.io −0.095 = stripped Similar Threads table its original gold keeps |

Guardrails: plain 1000/1000 identical, extract_golden 7/7. Golden-primary
conflict policy applied (0093 precedent).

## Scoreboard
Original dev **0.8467/0.7530** · golden **0.8725/0.7900** · goal gap
−0.0275 F1 / −0.0100 Lev (golden basis).
