# 0036 — Table separator convention + cell-space collapse + rescue/table stability

- **Date:** 2026-08-07
- **Tag:** 0036-v6 (baseline: 0035-final)
- **Status:** landed

## Hypothesis
A pred-only-line census showed `| --- |` rows as the top artifact. Initial
read ("we emit tables gold drops") was half wrong: the census regex missed
that gold DOES emit pipe tables — in a different separator style. Gold is
99.8% tight (`|----|`), split between header-width-padded (75/119 dev),
minimal `|---|` (35), and column-max (9); it never uses our spaced
`| --- |`. Gold also single-spaces cell interiors where our cells leaked
source whitespace (`|   WK |`).

## What changed
1. **Separator:** spaced `| --- |` → minimal tight `|---|`. Width-padded
   header-matching separators were tried first and measured WORSE
   (dev Lev 0.7066): docs where gold drops the table entirely pay for
   every extra byte. Minimal is strictly shorter for those docs and
   exactly matches gold's minimal-style tables.
2. **Cell-space collapse** at row end: runs of spaces inside a finished
   table row collapse to one (`|   WK |` → `| WK |`).
3. **Gate stability (the real find of the cycle):** two train docs
   flipped catastrophically (−0.92, −0.86 F1) during iteration because
   formatting-byte changes moved outputs across the near-empty-rescue
   keep-factor boundary — the 0012 instability class. Closed with:
   - `content_len` now excludes `-` (separator dashes) alongside `|#*`;
   - tier-1 rescue is vetoed when the base output contains a pipe table
     with a substantive cell (≥16 alphabetic chars): a compact
     acronym/stat table is real content, not a wiped page
     (thefreedictionary). Number-only tables (calendars — metrorock) are
     nav chrome and still rescue.

## Results
| split | F1 | Lev | vs 0035 |
|---|---|---|---|
| dev | 0.8058 | **0.7071** | 0 / +0.0002 |
| train | 0.7962 | 0.6931 | 0 / +0.0002 |

dev 60 improved / 17 down (all Lev-only, worst −0.0037, chrome-table docs
that keep paying for tables gold drops — selection question, queued).
train 763 improved / 148 down, **zero both-metrics-down**. General dev
plain: 1000/1000 byte-identical; dev2 above baseline. Goldens pass;
median 1.43 ms/doc (unchanged).

## Queued from this cycle
- Chrome-table selection: docs where gold drops the whole table
  (fleaflicker calendar, patents infobox, photo grids) — needs a
  table-level keep/drop decision, likely page-type-adjacent.
- Pre-existing serializer defect found by render check: first cell
  starting with a block element orphans the row's leading `|`
  (mcafee doc), adjacent to a fence-then-table interplay. One dev doc.

## Insights
- Formatting-byte changes are NOT metric-neutral around rescue gates;
  any serializer change needs the gate-stability lens. Now structurally
  hardened (dash exclusion + content-table veto).
- When a census claim implies "gold never does X", validate the regex
  against a doc where gold plainly does X.
