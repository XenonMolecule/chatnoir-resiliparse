# 0115: breadcrumb-row strip — golden 0.8861/0.8053

## What
Line-level strip (band-taxonomy nav lexicon): rows with >=2 " >> " or " » "
separators, <100 chars, no sentence punctuation — nav trails
("Moto profi >> Manufacturers list >> Suzuki >> GSF 600 Bandit 2001").

## Battery (`0115-w1` vs `0114-w2`)
dev_golden **0.8861/0.8053** (up 1 / down 0) · dev 0.8548/0.7630 (up 1 /
down 0) · train up 28 / down 7 (4 both-down, all ≤0.027 — original golds
that keep their breadcrumb; charter says chrome, golden-primary policy).
Guardrails: plain 1000/1000, extract_golden 7/7.

## Scoreboard
Original dev **0.8548/0.7630** · golden v12 **0.8861/0.8053** · gap −0.0139 F1.
