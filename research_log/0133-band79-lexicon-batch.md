# 0133: band-79 lexicon batch — golden 0.8895/0.8096

## What (24 lexicon_line routes from the band-79 census, distilled to the
## cross-engine recurring lines)
LABELS: LikeLike (WP like-widget), "Please Login to Remove!", "(permalink)",
"Find all posts by this user", "Quote this message in a reply", Adobe Flash
upgrade nags, "Taking too long? Try again or cancel this request.".
Skeleton strips: "View Full Version : <t>" (vB archive), "[Date Prev][Date
Next]" (mailman), "Quote Originally Posted by X … View Post", "Thanked N
Times in N Posts", "slide N of M", "Users browsing this forum…".

Ablation: the quote-header strip was tested out (train shows 14 small
both-downs from original golds that keep it) — but means are HIGHER with it
on every split (golden Lev 0.8096 vs 0.8095, train 0.8161/0.7179 vs
0.8160/0.7178); kept per golden-primary + mean dominance.

## Battery (`0133-w1` vs `0132-v3`/`0125-w2`)
| split | F1 | Lev | per-doc |
|---|---|---|---|
| dev_golden | **0.8895** (+0.0007) | **0.8096** (+0.0011) | up 25 / down 1 (−0.002) |
| dev | **0.8565** | **0.7654** | up 25 / down 3 |
| train | **0.8161** | **0.7179** | **up 185** / down 29 (all ≤0.043, original-gold quote-keeps) |

Guardrails: plain 1000/1000, extract_golden 7/7. Generic lane — transfers.

## Scoreboard
Original dev **0.8565/0.7654** · golden v13 **0.8895/0.8096** · goal gap
**−0.0105 F1** (Lev in-band).
