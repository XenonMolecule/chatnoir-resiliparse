# 0154: shape-based WP rebuild trigger — NEGATIVE, reverted

Follow-on to 0150/0153: fire the WP comment rebuild whenever it produces
>=3 gold-style `**Name – date**` headers and the current result has fewer
than half as many — i.e. trigger on OUTPUT SHAPE rather than on a
theme-specific "says:" marker. Intent: reach the 24 remaining WP comment
docs in the header-recall census.

Battery: golden F1 0.89184 → 0.89169, train F1 0.81616 → 0.81614 with
**36 both-down docs** (ericalscott −0.28, blowoutbuzz −0.22). Ups were real
(lifeconfusions +0.073, lifeinbetween +0.221) but the rule also fires on
docs whose native comment rendering already matches gold, and the rebuild
then drops surrounding content.

Verdict: header shape is not sufficient evidence that the rebuild is
better — the 0150 marker trigger's precision is what makes it safe.
Reverted; parity verified vs 0153.

Remaining header-recall census after 0153: 67 docs (26 "other" engines,
24 WP, 11 vBulletin, 5 phpBB, 1 Invision). Each needs template-specific
evidence, not a global rule.
