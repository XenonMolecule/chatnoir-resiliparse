# 0138: wave 15 (partial, crash-salvaged) — golden 0.8902/0.8106

## What
Wave 15 ran twice into failures (network switch → 8 agents ENOTFOUND;
then the Fable-5 session limit → 7 agents). **86/150 docs and 52 verified
rules were salvaged from the checkpoint files** and shipped without
re-running anything: +25 vetoes / +16 whitelists after filtering.

Third crash absorbed by the checkpoint protocol (2 limits + 1 network
drop, ~1.5M tokens of rework avoided cumulatively).

## Battery (`0138-w1` vs `0135-v2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8902** | **0.8106** | up 5 / down 1 (mixed-sign) |
| dev | **0.8569** | **0.7659** | up 5 / down 1 |
| train | 0.8161 | 0.7179 | up 1 / down 0 |

Guardrails: plain 1000/1000, extract_golden 7/7.

## Lane decision (owner: token budget)
Fleet waves are **stopped**. Final rate was +0.0002 golden F1 per ~800k
tokens; the remaining reachable residue does not justify the spend.
Future work should be inline/scripted (censuses, lexicon batches, gold
edits) or wait on the owner-gated items.

## Scoreboard
Original dev **0.8569/0.7659** · golden v14 **0.8902/0.8106** · goal gap
**−0.0098 F1** (Lev in-band).
