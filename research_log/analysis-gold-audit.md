# Gold boilerplate audit — lpv11/dev (agent report, 2026-08-07)

Requested by the user ("this model let through a disproportionate amount of
boilerplate"). 150-doc forward audit (seed 42) + 30-doc reverse audit
(seed 123), keep-rates conditioned on the page actually having the element.

## Keep-rates and verdicts

| Category | keep-rate given present | corpus recall mass | verdict |
|---|---|---|---|
| Nav menus / sidebars | 14.7% | 0.66% | **wall** (keeps concentrate on catalog/profile pages; near-zero on articles) |
| Site-wide footer/© | 5.1% | 0.24% | **safe to drop** |
| Source-attribution © credits | high | ~0.5% | **keep (gold policy)** — NPR/wire credits, image rights, dictionary credits |
| Cookie/consent banners | 0% | 0 | **safe to drop** |
| Share/social rows | 2.2% | 0.02% | **safe to drop** |
| Breadcrumbs | 5.3% | 0.09% | **safe to drop** |
| Login/search/subscribe chrome | ~6% | 0.14% | **safe to drop** |
| Related-posts blocks | ~18% | ≤0.1% | **wall (leans safe)** |
| Pagination / prev-next | 7.3% | 0.04% | **safe to drop** |
| Tag/category rows | low | — | **safe to drop** (even when adjacent meta kept) |

Dropping every safe category costs ≤ ~0.25 F1 loss-side; the precision upside
on the 85–98% of element-bearing docs where gold dropped it is far larger.

## Gold policies to encode (not fight)
- User comments and author/date metadata: KEEP (confirmed again).
- Source-attribution copyright/credit lines attached to content: KEEP.
- "0 comments" counts on index pages: kept by gold.
- Nav-drop decisions should be conditioned on article-ness — gold keeps chrome
  on catalog/directory/profile pages where chrome IS the content.

## Reverse audit (recall side)
No mid-article truncation in 30/30 docs; all 12 gold-dropped paragraphs were
genuine boilerplate. Gold recall quality is high — precision-side noise only
(~1–2% of gold tokens). The metric can be trusted for recall work; precision
work should follow the table above rather than raw per-doc F1 on wall
categories.
