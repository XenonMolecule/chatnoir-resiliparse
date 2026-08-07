# 0014 — vBulletin engine handler

- **Date:** 2026-08-07
- **Tag:** 0014-vb2 (baseline compared against: 0013-fences)
- **Status:** landed — largest quality gain since 0004

## Hypothesis
104 dev docs are forum threads (mean F1 0.707 vs corpus 0.782); vBulletin is
the largest engine (44). Gold formats threads as `**user – date**` headers +
post bodies. An engine-gated handler that rebuilds the thread from postbit
DOM should beat the generic walk substantially.

## What changed
- `generator_meta()` engine detection; handler fires only for vBulletin
  generator + main_content + markdown mode (guardrails run plain → untouched).
- Posts from `table[id^=post] / li[id^=post_]` (vB3/vB4); author from
  `a.bigusername / a.username`; date = first date-like `.thead/.postdate/
  .date` text; body via the full generic walk scoped to
  `div[id^=post_message_] / blockquote.postcontent` (reuses markdown
  machinery — new `extract_plain_text_from_node` entry point).
- Emits `**{author} – {date}**\n\n{body}`; ≥2 well-formed posts required,
  else falls back to generic extraction. Posts with unmatched author anchors
  keep their body headerless (dropping whole posts measured −0.38 ×2 on
  train — fixed).

## Results
| set | before | after | per-doc |
|---|---|---|---|
| lpv11 dev | 0.7819 / 0.6712 | **0.7850 / 0.6758** | **18↑ / 0↓** (best +0.26) |
| lpv11 train | 0.7764 | **0.7823** (+0.0059) | **258↑ / 13↓** (worst −0.19) |
| general dev | — | — | 0 changed |

Golden tests pass. Residual train regressions are gold-style variance
(title-included vs post-only openings, `—` vs `–`, hard-break vs blank-line
separators) — gold-inconsistent, not chased (0079 precedent).

## Insights
- The engine-handler pattern transfers to resiliparse exactly as jusText's
  log promised: tight gate (generator meta), rebuild to the gold's shape,
  fall back when unsure — +0.0059 train mean from ONE engine.
- `extract_plain_text_from_node` makes handlers cheap to add: the body walk
  inherits fences/tables/emphasis for free. phpBB (25 docs) is next and
  should be mostly selector work.

## Next
- 0015: phpBB handler; then reply-form pages (newreply.php → near-empty).
- Thread-title emission where the skin carries it (gold-split, needs check).
