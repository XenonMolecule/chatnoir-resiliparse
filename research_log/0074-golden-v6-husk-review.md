# 0074 — Golden v6 (husk review) + widget/vB-alias NEGATIVE

- **Date:** 2026-08-08
- **Status:** gold-side ship + extractor negative (reverted)

## Golden v6
Owner rejected the weatherbug husk edit → charter husk rule (<300
chars or <15% → suspect) → tripwire sweep found 35 husk-risk specs →
review agent (weatherbug-calibrated): 29 legit (N1/N2 junk dominates
the doc), 5 NARROWED with verification (webkinz forum posts, use.perl
copyright, stampedia catalog, meals.com recipe, barnesjewish
summaries). Root cause of all five: range END-LITERAL matching its
last occurrence far past the intended stop — a fleet-wide spec-format
hazard now documented. Golden v6: **0.8453 / 0.7572** (+0.0029/+0.0026
measurement correction).

## NEGATIVE: Blogspot widget selectors + vB td_post_ alias (0074-widgets)
Both triage suggestions measured strongly negative together (train
−0.0041 F1, 35 craters — forums mispaired via the td_post_ container
zip; Blogspot .widget.Profile family hit content on some templates).
Reverted; per-suggestion isolation required before retry (triage
est_gain is not a measurement — reconfirmed).

## Scoreboard
original dev 0.8282/0.7323 · **golden v6 0.8453/0.7572**.
