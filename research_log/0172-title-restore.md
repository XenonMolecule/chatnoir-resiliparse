# 0172: document-title restore (StackExchange family) — golden 0.89481/0.81540

Math-audit finding: on real SE pages the h1's block is all-link
(link_density 1.0) and the model vetoes it with full-page features — the
page's most important line is dropped across the whole SE/SO family.

Fix: post-extraction restore gated on `h1[itemprop]` (schema.org name
marker) + NORMALIZED absence from output. Two gates were battery-rejected
on the way: title-tag containment (blog site-name h1s are title-backed;
82 golden both-downs) and exact-string containment (re-added a photo
caption present in different form; −0.24 crater).

Battery (vs 0171-w2): golden **0.89481/0.81540** (up 10 / down 4), train
0.81656/0.71813 (up 111 / down 46), math dev +0.0039 (SE doc's title
back). Plain 1000/1000, tests 7/7.
