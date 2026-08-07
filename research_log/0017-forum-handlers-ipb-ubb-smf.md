# 0017 — Invision / UBB.threads / SMF handlers (agent-built, verified)

- **Date:** 2026-08-07
- **Tag:** 0017-forums (baseline compared against: 0016-nbsp)
- **Status:** landed
- **Provenance:** built by a background worktree agent following the
  0014/0015 handler pattern; selectors derived from actual dev-doc markup;
  patch applied to main and re-verified with the full battery here.

## What changed
Three engine handlers, each with a disjoint exact gate and generic fallback:
- **Invision** (IPB3 `body#ipboard_body` / IPS4 `body.ipsApp` +
  `data-pagecontroller=topic`): posts, authors, dates, bodies + **member
  signatures** (gold keeps them); thread title on IPB3 only (gold policy
  differs by skin generation); photo-only posts keep their authored header.
- **UBB.threads** (generator meta): per-post table climb for author/date;
  ≥1 authored post (exact gate permits single-post threads).
- **SMF 2.0** (`div#forumposts`): `« Reply #N on: DATE »` parsing, title from
  "Author Topic: … (Read N)", drops "Quote from:" attribution lines while
  keeping quoted text (gold policy).

## Results (verified in main repo)
| set | before | after | per-doc |
|---|---|---|---|
| lpv11 dev | 0.7856 / 0.6785 | **0.7878 / 0.6820** | **14↑ / 0↓** |
| lpv11 train | 0.7829 | **0.7842** (+0.0013) | 78↑ / 7↓ |
| general dev | — | — | 0 changed |

Golden tests pass; off-target docs byte-identical (agent verified 986/9900,
guardrail run confirms). One documented −0.10-rule exception: a UBB train doc
whose gold is an uncleaned page dump (contains "Powered by UBB.threads®" and
the forum-jump menu verbatim); the same skin's clean-gold doc gains +0.83 —
gold noise, logged per 0006 precedent.

## Insights
- The parallel-agent pattern works for handler expansion: ~29 min of agent
  time produced three shippable handlers with honest per-doc accounting; the
  main-thread battery caught nothing the agent missed.
- Forum arc cumulative (0014–0017): dev F1 +0.0058, Lev +0.0108; forums have
  gone from the worst family to near-solved for the five major engines.

## Next
- 0018: audit-guided safe-chrome drops; 0019: template-subtraction port.
- Remaining forum long tail: jcink/custom skins (deliberately ungated),
  phpBB2 legacy, reply-form gold-wall.
