# 0114: forum user-panel vocab strips — golden 0.8860/0.8053

## What (from the 0113 band taxonomy; generic, line-level)
Postbit family (0082) extended with the vBulletin per-post user-panel
skeletons: "<name> is offline/online", "<name>'s Avatar", "iTrader:",
"Mentioned: N Post(s)", "Tagged: N Thread(s)", "Quoted: N Post(s)",
"Liked N Times in N Posts", "Appreciate N". New LABELS: "Garage List",
"Reply With Quote", "View Options", "Report a problem".

Battery-corrected: "Cite This Source" was in the first cut — the
dictionary.com/thesaurus.com family KEEPS it in gold (−0.025..−0.039 there);
removed from LABELS. (Even hand-picked "obvious chrome" labels need the
battery — lexicon entries are cheap to test, one label at a time.)

## Battery (`0114-w2` vs `0113-w3`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8860** | **0.8053** | up 11 / down 1 (raptors mixed-sign) |
| dev | **0.8548** | **0.7630** | up 10 / down 3 (all ≤0.008 F1) |
| train | **0.8158** | **0.7174** | **up 80** / down 5 (0 both-down) |

Guardrails: plain 1000/1000 identical, extract_golden 7/7. The train-wide
up-count confirms this is a GENERIC lane (should transfer to test, unlike
site rules — the M2 lesson).

## Scoreboard
Original dev **0.8548/0.7630** · golden v12 **0.8860/0.8053** · goal gap
−0.0140 F1 (Lev in-band).
