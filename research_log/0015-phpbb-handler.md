# 0015 — phpBB3 engine handler

- **Date:** 2026-08-07
- **Tag:** 0015-phpbb3 (baseline compared against: 0014-vb2)
- **Status:** landed

## Hypothesis
Second-largest forum engine (25 dev docs). Same handler pattern as 0014:
`body#phpbb` gate, posts from `div.post`, author/date parsed from `p.author`
("by USER » DATE" + post-icon and date-first skin variants — jusText 0080/
0072 ported preemptively), body via subtree walk of `div.content`.

## What changed
Handler + three hardening rounds driven by train regressions:
- author parse handles "Post by USER » DATE" (post-icon skins) and
  "DATE by USER" (WP-integrated skins);
- gate requires `section-viewtopic` when the skin emits section classes
  (search.php results pages share postbody markup — bogleheads −0.23);
- ≥2 *authored* posts required, else generic fallback (an author-less rebuild
  only subtracts titles/attribution — dhammawheel −0.27);
- thread title emitted as `# {h2}` (gold keeps it).

## Results
dev +0.0006 (5↑/1↓, worst −0.043); train +0.0006 (47↑/8↓, worst −0.094).
Guardrails untouched (markdown-only). Golden tests pass.
Cumulative forum arc (0014+0015): dev 0.7819 → **0.7856**, train 0.7764 →
**0.7829**.

## Insights
- The jusText forum-log entries are a usable *bug encyclopedia*: both shipped
  hardening fixes (post-icon author, search-page misfire) were documented
  there before we hit them.
- "Rebuild must beat the generic walk or abstain" is the right frame: every
  regression came from the handler winning the gate but producing less than
  the walk would have.

## Next
- Reply-form pages (newreply.php), Invision/UBB/SMF, phpBB2 legacy skins.
- Parallel-agent wave (user suggestion): gold-boilerplate audit, wild-idea
  exploration, handler expansion — main thread keeps serial cycles.
