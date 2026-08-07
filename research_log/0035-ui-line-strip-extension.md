# 0035 — UI-line strip extension (+ two walls confirmed)

- **Date:** 2026-08-07
- **Tag:** 0035-final (= 0035-v3; baseline: 0034-headings)
- **Status:** landed

## Hypothesis
Cross-doc census of pred-only lines (emitted by us, absent from gold,
repeated across many docs) exposes residual chrome families strippable with
near-zero gold keep-rate; adding them to `strip_ui_label_lines` lifts F1
without per-doc catastrophes.

## What changed
1. **Normalization fix:** the existing curated list only matched plain
   lines — `## Leave a Reply`, `**Leave a Reply**`, and mid-line bold
   (`You **cannot** post…`) all escaped. Now leading `#` is trimmed and
   `*`/`**` markers removed before matching. Guard: heading lines only
   match multi-word or ≥10-char labels (bare `# Reply` is genuine content
   on form pages — abc.net.au regression found and fixed).
2. **List extension** (dev+train gold keep-rate ≤4% each, n≥30):
   phpBB permission band (5 lines), "Who is online", "Author Message",
   "Post subject:", "Display posts from previous: Sort by", topic-nav
   lines, "Leave a Reply Cancel reply", "%d bloggers like this:",
   Akismet notice, "Advertisement(s)", "Skip to (main )content",
   "Newer Post Older Post Home", "Recent Posts", "Advanced Search",
   "Search for:".
3. **Rejected after measurement:** "Posted by at" and "No comments:" —
   Blogspot golds keep both (the gold extractor saw the same degenerate
   bylines); stripping them regressed 25/15 docs. "Comments are closed."
   (35% keep), "Privacy Policy"/"Terms of Service"/"Related Articles"/
   "Most Popular"/"Sort by" (6–19% keep) — left alone.
4. **Ordering experiment (kept the status quo, now documented):** moving
   the post-passes after the rescue ladder (so gates measure unstripped
   length) fixed one junk-rescue doc but cost three beneficial rescues and
   netted worse on train (0.7960 vs 0.7962). Interior ordering is the
   principled one — chrome doesn't count as content for gate purposes,
   same rationale as `content_len` excluding markdown punctuation. Comment
   now states this at the call site.

## Results
| split | F1 | Lev | vs 0034 |
|---|---|---|---|
| dev | **0.8058** | **0.7069** | +0.0009 / +0.0012 |
| train | 0.7962 | 0.6929 | +0.0008 / +0.0009 |

dev: 108 docs improved, 5 both-metrics-down (worst −0.0036 F1, alignment
artifacts on docs whose other metric improved). train: 1068 improved, 59
both-down, worst −0.29 (travelisthenewclub.com: stripped base fell under
the near-empty gate → tier-2 pulled a teaser stream; logged decision —
accepted as the cost of keeping 3 beneficial rescues with the same gate
semantics). General dev plain-config: 1000/1000 byte-identical (plain path
untouched). Goldens pass; fences balanced 1000/1000; median 1.42 ms/doc
(unchanged).

## Walls confirmed this cycle (do not build)
- **Bold coverage:** gold's `**bold**` comes from rendered font-weight
  (CSS classes like `course-header-title`, `span.label`) or annotator
  judgment on plain text — invisible without a CSS engine. 6th wall.
- **Tags/Labels taxonomy lines:** gold keeps them in only 20% of docs
  whose HTML has them — gold-inconsistent. 7th wall.

## Insights
- Under-extraction tail (197 docs, 0.078 F1 mass) is dominated by gold
  noise (raw HTML/entity-map contamination, snapshot mismatch) plus
  gold-kept boilerplate — matches taxonomy-0019.
- P/R globally balanced (0.837/0.836) — no global calibration win exists.
- The pred-only-line census is reusable and cheap; rerun it after any
  selection change to find the next chrome family.
