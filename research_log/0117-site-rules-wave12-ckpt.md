# 0117: site-rules wave 12 (checkpointed fleet) — golden 0.8878/0.8072

## Fleet protocol change (owner flagged token burn)
Session limits killed waves 10 and 12 mid-flight (~1.1M subagent tokens
unrecoverable — workflow resume only caches COMPLETED agents). New protocol,
first used this wave: agents append per-doc results to
`wave12_ckpt_<shard>.jsonl` IMMEDIATELY (zero-yield docs get a "none"
marker), read their own file on restart to skip done docs; 10×20-doc shards
instead of 8×44; targets trimmed to bottom-200 by F1 (loss-mass core) —
roughly half the token cost per wave. All 200 docs completed; 440 raw
rules from checkpoints → **+79 SITE_VETOES / +45 SITE_WHITELIST**.

## Bisect
recordnet.com removed + blocklisted (train both-down; byline/author vetoes
fought original train gold). cinephonix dev −0.35 KEPT: golden-primary
zero-sum (its golden doc 0.287→0.547 — the wave-9/11 whitelists finally
land as one family).

## Battery (`0117-v2` vs `0116-v2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8878** (+0.0006) | **0.8072** (+0.0004) | up 20 / down 5 (0 both) / craters 0 |
| dev | 0.8557 (−0.0002) | 0.7643 (−0.0001) | zero-sum vs golden (cinephonix/blip) |
| train | 0.8158 | 0.7174 | up 5 / down 1 (−0.0001) |

Guardrails: plain 1000/1000 identical, extract_golden 7/7.

## Scoreboard
Original dev **0.8557/0.7643** · golden v12 **0.8878/0.8072** · goal gap
**−0.0122 F1** (Lev in-band).
