# Failure taxonomy at 0019 — all 175 docs below F1 0.6 (agent report, 2026-08-07)

Total addressable ≈ +0.049 F1; realistic near-term ≈ +0.025–0.03.
Ranked families (docs / est. value / fix):

1. **CHROME on short articles** (32 / +0.011): related/recommended modules,
   subscribe walls drown 200–2000-char articles (recall 1.0, precision 0.2).
   Fix: related/recommended class signatures in the chrome veto + a
   template-subtraction tier that fires even for large containers when one
   high-density article container dominates.
2. **FORUM unhandled/mis-gated** (29 / +0.010): phpBB2 (table skins — our
   gate is phpBB3-only), vBulletin WITHOUT generator meta (add markup gate:
   ≥2 `table[id^=post]`/`div[id^=post_message_]`), showpost/newreply
   sub-pages, and ten one-off engines (Nabble, PerlMonks, Slash, WebBBS…) —
   best served by a GENERIC post-stream rebuilder: repeated same-class
   containers each holding user-link + date + body → `**user — date**`.
3. **LISTING pages truncated** (30 / +0.010): tag/search/archive/schedule
   pages — gold keeps every teaser card; card grids (div/article, not ul)
   get link-density-vetoed. Fix: listing-shaped-page predicate (already in
   tpl_scan) should ENABLE card extraction via a rescue tier.
4. **FORMAT divergence** (25 / +0.005): heading levels off-by-one, `•` vs
   `-`, and **byline anchors stripped as nav links** ("Posted by at" —
   exempt anchors inside author/byline/vcard containers). Quick win.
5. **METADATA fields lost** (21 / +0.005): `**Field:** value` lines from
   dl/2-col tables — add a definition-table branch to the 0012 pre-pass.
6. **COMMENTS extensions** (9 / +0.003): Blogger comment-blocks, Disqus-in-
   DOM, tweet streams — extend 0020's machinery.
7. **GOLDNOISE** (17 / +0.003 real): gold contains raw JS/__VIEWSTATE/LLM-leak
   lines ("Let's parse the HTML snippet:"). Mark and exclude from sweeps:
   thecut, crazydaysandnights, stambia, barnesjewish, bluffcountry,
   conferenceboard, disneyandmore, bearalley, happysadlola,
   iclassifiedsnetwork, + 7 more (see 0019 run diffs).
8. **NEAREMPTY misc** (12 / +0.004): rescue-sweep candidates + form pages.

Priority order: 2 (precedented, low risk) → 1 (needs sweep) → 3 → 4 → 5/6.
