# 0141: owner-flagged form junk — golden 0.8902/0.8106 (up 10 / down 1)

Owner review of the ruling tool surfaced two concrete extractor defects:
- **Empty-cell markdown rows** `| | | | |` (vBulletin post-icon grids,
  spacer rows) — 15 instances across 10 dev docs, zero information.
  Generic line strip: a row whose every cell is empty.
- **New-reply form labels**: "Post Icons", "Trackback:", "Send Trackbacks
  to (Separate multiple URLs with spaces) :", "Confirm Password:",
  "Password:", "User Name:".

Battery vs 0138-w1: golden up 10 / down 1 (−0.0007 Lev), dev up 8 / down 3,
**train up 137 / down 21**. Plain 1000/1000, tests 7/7. Generic lane.
