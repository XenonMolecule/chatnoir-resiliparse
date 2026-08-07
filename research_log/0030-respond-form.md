# 0030 — WordPress respond-form veto (the residual's hidden giant)

- **Date:** 2026-08-07
- **Tag:** 0030-respond (baseline compared against: 0029-fixed)
- **Status:** landed — largest cycle since 0004; F1 crossed 0.80, Lev crossed 0.70

## Hypothesis
The content-chunk residual diagnosis (76% pred-side junk) surfaced one
dominant family: the WordPress reply form ("Leave a Reply / Fill in your
details below / Gravatar / WordPress.com…" via `#respond`/`#commentform`/
`.comment-form`) surviving on comment-bearing pages — ubiquitous because
WordPress is.

## What changed
`respond|comment-?respond|comment-?form|commentform` added to the chrome
veto (all existing guards apply: content-marker exemption, size cap,
negation gluing, body_depth>2).

## Results
| set | before | after | per-doc |
|---|---|---|---|
| lpv11 dev F1 | 0.7978 | **0.8013** (+0.0035) | **101↑ / 8↓** |
| lpv11 dev Lev | 0.6961 | **0.7010** (+0.0049) | |
| lpv11 train | — | +0.0022 / +0.0030 | 72 > +0.1 vs 19 < −0.1 |
| general dev | — | +0.0001 | 1↑ / 0↓ |

Goldens pass. Tail: compose/NewMessage pages whose gold IS the form (the
documented reply-form gold-wall) — accepted per 0016/0029 precedent.

## Insights
- The biggest win in 26 cycles came from READING THE RESIDUAL, not from new
  machinery — the audit→diagnose→one-line-fix loop remains the highest-ROI
  motion in the playbook, exactly as jusText's log claimed.
- Milestone: **M1 (0.81 F1 / 0.72 Lev) is within reach** — currently
  0.8013 / 0.7010.

## Next
- Remaining pred-extra families from the same diagnosis: "Comment and
  Contribute" character-count forms, article tool blocks ("Share this
  article" + tools), Stats-copyright lines.
- One-off engine agent batch in flight.
