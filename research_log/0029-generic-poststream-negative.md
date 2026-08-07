# 0029 — Generic post-stream rebuilder — NEGATIVE, disabled

- **Date:** 2026-08-07
- **Tag:** 0029-generic{,2} measured; 0029-reverted (= 0026 output)
- **Status:** abandoned (code kept behind `if false` for per-engine reuse)

## Measured
| gates | dev ΔF1 | per-doc |
|---|---|---|
| repeated blocks + short link + date anywhere | **−0.0197** | 14↑/85↓ (policy libraries, race results rebuilt as fake forums) |
| + head-anchored author/date, ≥2 distinct authors, date required | −0.0019 | 1↑/8↓ (worst −0.68: slideshow with datelines) |

## Insights
- Fifth confirmation of the page-type discriminator wall: "repeated
  containers with author-ish links and dates" describes forum threads AND
  slideshows AND datelined article grids. Only per-engine signatures (exact
  markup gates, the 0014–0021 pattern) separate them reliably — the ten
  one-off engines are each a 15-minute agent task with a proper gate, not a
  generic rule.
- The helper's post-parsing internals (head-anchored author, date regex,
  line-dedup) are sound and reusable by future per-engine handlers.

## Next
- Batch the one-off engines (Nabble, PerlMonks, Slash, WebBBS, …) to a
  worktree agent with per-engine gates, 0017-style.
- Content-chunk residual diagnosis (20% of Lev diff mass) remains open.
