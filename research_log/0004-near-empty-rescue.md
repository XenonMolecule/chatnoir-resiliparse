# 0004 — Near-empty rescue (fallback to full extraction)

- **Date:** 2026-08-06
- **Tag:** 0004-rescue (baseline compared against: v1.0.9-cacde72)
- **Status:** landed
- **Machine:** cloud VM (e2-standard-4, x86); VM Rust baseline 2.152 ms/doc (0003).

## Hypothesis

0003 showed the catastrophic tail is main-content classifier false negatives
whose content survives with `main_content=False`. A gated self-correcting
rescue — when main-content extraction is near-empty relative to page text,
re-extract unfiltered and keep the result only if it is much larger — should
recover those docs and be structurally unable to regress working pages.
Expected +0.5–1.5 F1, zero regressions, negligible mean speed cost.

## What changed

In the Rust `extract_plain_text`: after a `main_content=True` extraction,
if `output < 200 bytes` AND `collapsed body text > 30× output`, re-extract
with `main_content=False` and keep the fallback iff it is `> 20× output`.
Thresholds were chosen by an exhaustive offline sweep (4×3×3 grid) over
cached dev predictions — the shipped combo is the F1-maximizing one under the
**zero-regression constraint**; looser gates reach F1 0.7799 but regress 3–26
docs (worst −0.77 on docs whose small output was *correct*), so they were
rejected per policy.

## Results

| set | baseline F1/Lev | 0004 F1/Lev | per-doc |
|---|---|---|---|
| lpv11 dev | 0.7619 / 0.6540 | **0.7782 / 0.6656** | **+29 / −0** (971 unchanged) |
| general dev (raw) | 0.7983 / 0.7114 | **0.8107 / 0.7208** | +17 / −0 |
| general dev2 (raw) | 0.8013 / 0.7152 | **0.8131 / 0.7236** | +18 / −0 |

Speed: 2.021 ms/doc best-of-3 vs 2.152 baseline — no cost (rescue fires on
~3% of docs; second extraction only paid there). Golden parity tests still
pass (fixture outputs are large; gate can't fire).

Byte-level effect: 12 of 13 EMPTY_PRED docs recovered (median recovered-doc
F1 ≈ 0.9); the rest of the improvements are severe UNDER_EXTRACTs.

## Insights

- The jusText "self-correcting rescue" pattern transfers perfectly: +1.6 F1
  points for a ~30-line, provably-safe change. First quality delta of the fork.
- The offline gate sweep (cache both branches' predictions once, then compose
  and score every threshold combo in seconds) is far better methodology than
  tune-rebuild-rerun; reuse it for every future gated heuristic.
- KEEP_FACTOR=20 does the protective work: on genuinely thin pages the
  unfiltered output is boilerplate of comparable size, not 20× larger, so the
  swap never happens. ABS=500 looked harmless but broke docs where a
  200–500-byte output was the whole correct answer.
- Remaining catastrophic tail after this: rule-level fixes from 0003
  (`<ul>` rule 13 docs, article-teaser 8, nav/landmark/footer ~9) — those
  need actual classifier repairs, not rescues.

## Next

- 0005: `<ul>` rule repair (the 13-doc bucket) — only drop shallow/link-heavy
  lists when they don't carry the page's dominant text mass.
- Then article-teaser threshold, landmark/footer regex tightening
  (`wsite-not-footer`!).
- Forum over-extraction track (131 OVER docs) still untouched.
