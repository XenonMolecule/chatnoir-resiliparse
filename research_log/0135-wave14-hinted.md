# 0135: wave 14 (hinted, checkpointed) — golden 0.8900/0.8104

## What
Verified re-extraction over the 114 band-79 census docs flagged
site_veto/site_whitelist, with the census hints in the agent prompts and
instructions to skip existing table rows. **First live validation of the
checkpoint protocol**: a session limit killed all 8 agents mid-run; 60
per-doc result lines survived on disk and the resumed agents skipped that
work. 87 raw rules → **+67 vetoes / +13 whitelists**.

## Bisect (wave rows only — 0104/0132 lesson applied)
Removed + blocklisted: yahoo.com (train −0.12 crater), blurb.com,
cbssports.com, smallbizpages.co.uk, lybrate.com (golden both-downs).

## Battery (`0135-v2` vs `0134-g15b`/`0133-w1`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8900** (+0.0004) | **0.8104** (+0.0007) | up 15 / down 4 (0 both) |
| dev | **0.8568** | **0.7658** | up 14 / down 6 |
| train | 0.8161 | 0.7179 | up 1 / down 0 |

Guardrails: plain 1000/1000, extract_golden 7/7.

## Scoreboard — 0.89/0.81 crossed
Original dev **0.8568/0.7658** · golden v14 **0.8900/0.8104** · goal gap
**−0.0100 F1** (Lev in-band).
