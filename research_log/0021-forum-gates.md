# 0021 — vBulletin markup-gate fallback + phpBB2 handler

- **Date:** 2026-08-07
- **Tag:** 0021-forums3 (baseline compared against: 0020-wp3)
- **Status:** landed

## Hypothesis
Taxonomy family #2: vB installs without generator meta and phpBB 2.x table
skins never reach their handlers.

## What changed
- vBulletin dispatch: markup fallback (≥2 `div[id^=post_message_]`) besides
  the generator meta; the handler's own ≥2-authored-posts gate protects.
- `extract_phpbb2`: `span.name`/`span.postbody` paired by order (counts must
  match), date parsed from `span.postdetails` "Posted: …", plus a **coverage
  guard** — the rebuild must carry ≥25% of the page's collapsed text or
  abstain (PNphpBB2 skins match signatures, not bodies: one −0.72 train
  catastrophe before the guard, zero after).

## Results
dev **+0.0014 (7↑/0↓)**; train +0.0008 (28 > +0.1, ZERO < −0.1, worst
−0.034). Goldens pass; guardrails structurally untouched.
Cumulative forum arc (0014–0021): dev ≈ +0.0086 F1.

## Insights
- The coverage guard generalizes: "rebuild must beat a meaningful fraction of
  the page or abstain" is now the standard third gate alongside authored-posts
  and disjoint signatures. Candidates for retrofit onto 0014/0015 if their
  tails ever resurface.

## Next
- Generic post-stream rebuilder for the ten one-off engines (taxonomy #2b).
- 0022: byline-anchor exemption + related-modules tier.
