# 0159: do old NEGATIVES flip on a new gold basis? — no

0156/0157 showed per-domain tables decay when the gold basis moves
(+0.0017). Natural follow-up: do the near-miss NEGATIVE results also flip?
Re-tested the closest one — 0121's link-list-after-heading rescue, which
lost by a single doc at golden v12 — on v16, where gold has since gained
substantial forum-post content.

Verdict unchanged: golden 0.89354 → 0.89347 with **4 both-down docs**
(scoop.co.nz −0.048, ema.europa.eu −0.041). Train F1 rose (+0.0003) but
with 13 both-downs. Reverted; parity verified.

## The refined rule (completing 0156-0158)
A gold-basis change invalidates judgements whose UNIT matches what the
audit changed:
- **per-domain verdicts** (blocklist, FALLBACK_EXCLUDE) — decayed, worth
  +0.0017 on re-test, because the audit changed which docs are right;
- **global operating points** (model thresholds, UL gates) — unchanged;
- **structural rule negatives** (0121 and its family) — unchanged, because
  they failed on mechanism (widget headings introduce nav lists), and the
  mechanism is independent of which docs are in the gold.

Only the first class is worth re-testing after a basis bump. That is the
cheap, repeatable check to run whenever the gold moves again.
