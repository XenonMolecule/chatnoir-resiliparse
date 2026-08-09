# 0143: owner restoration rulings applied — golden v15 = 0.8915/0.8116

The owner ruled on all three disputed docs via the ruling tool:

| doc | ruling | gold | doc F1 |
|---|---|---|---|
| **menstennisforums** | "I like our extraction better" → replace the 3x-duplicated 298B header with the rendered Topic Review posts | 298B → 6,650B | 0.047 → **0.948** |
| **jeepforum** | "our extraction is better" → restore the omitted rendered posts | 4,296B → 9,455B | 0.475 → **0.713** |
| **huskers** | "We should keep the schedule … I don't mind if we keep [the FAQ]" → keep carousel AND add the visible FAQ body | 6,254B → 6,994B | 0.052 → **0.247** |

Two extractor defects the owner spotted alongside the rulings were fixed
first (0141 empty-cell form rows + reply-form labels; 0142 quote
block-break), so the restored golds are built from clean extraction.

Golden basis v14 → **v15: 0.8915/0.8116** (+0.0013/+0.0010), zero down
docs. Extractor unchanged in this cycle.

## Charter refinements from the rulings (apply to the ~300 unlabeled docs)
1. A gold that is a duplicated stub while the page renders a full thread is
   BROKEN — the rendered posts are the target.
2. Rendered forum posts are content even when the annotator omitted them.
3. CSS-hidden widgets the annotator kept may STAY (huskers schedule); the
   test is what the gold already commits to, not our hidden-content rule.
4. Loosely-related on-page sections (huskers FAQ) are acceptable content —
   relatedness is not the charter's test, rendering is.

## Scoreboard
Original dev 0.8569/0.7659 · golden v15 **0.8915/0.8116** · goal gap
**−0.0085 F1** (Lev in-band).
