# 0039 — Blogspot comment rebuild (+ effective-veto-set bug fix)

- **Date:** 2026-08-07
- **Tag:** 0039-v4 (baseline: 0038-final)
- **Status:** landed — biggest cycle since template subtraction

## Hypothesis
Gold rewrites Blogger's native comment rendering ("NAME said..." + a
separate footer timestamp) as `**NAME — TIMESTAMP**` + body (em-dash
joiner dominates 3093:558 across 228 marked-up docs). A rebuild in the
engine-handler style — NOT native-first, since the native walk keeps the
author in the wrong shape — converts the whole family.

## What changed
1. `blogspot_comment_rebuild`: classic dl template (document-order
   dt.comment-author / dd.comment-body / dd.comment-footer triples) and
   threaded template (div.comment-block with cite + .comment-timestamp/
   .datetime + .comment-content). Emits `**NAME — TIME**  \n body`;
   strips "said..." suffixes; author sanity guard (>48B, `;`,
   `document.` → abort that comment — a script tag inside the author
   node was emitted as attribution on crazyhousereviews). Fires at ≥2
   attributed and ≥half coverage; vetoes BOTH templates' nodes (some
   blogs render comments through both — leaving the mirror alive
   duplicated every comment).
2. **Effective-veto-set bug (latent since 0020, exposed here):** after a
   comment rebuild re-ran the walk with comment vetoes, the rescue-ladder
   retries still used the PRE-rebuild veto set — a rescue firing after a
   rebuild resurrected the native comments and site shell next to the
   rebuilt block (carolinescrayons −0.24, noagela −0.15). All extraction
   passes now share one `effective_tpl` that rebuilds extend.

## Results
| split | F1 | Lev | vs 0038 |
|---|---|---|---|
| dev | **0.8077** | **0.7094** | +0.0014 / +0.0018 |
| train | 0.7983 | 0.6954 | +0.0015 / +0.0018 |

dev: 22 improved / 2 down / ZERO both-down. train: 232 / 17 (8
both-down; worst johnysimple −0.088, a bold-dateline difference outside
this family — logged). General dev plain 1000/1000 identical; goldens
pass; fences balanced; 1.54 ms/doc (noise band).

## Insights
- Gold also rewrites the POST byline as `**Author — TIME**` on some
  Blogspot docs (carolinescrayons gold starts `**Crayons — 7:35 PM**`) —
  a possible follow-on family ("post-byline reformat").
- Rebuild-then-rescue interactions are now structurally safe; any future
  rebuild must extend `effective_tpl`, not fork its own set.
