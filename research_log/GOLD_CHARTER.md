# Gold Annotation Charter (lpv11 golden devset)
Ratified by the dataset owner 2026-08-07. The single source of truth for
what counts as content vs noise in gold annotations.

## CONTENT — keep in gold
1. The page's main article/post/product/thread, in markdown.
2. **ALL fully-rendered articles on the page** — lazy-loaded feeds,
   multi-post blog pages, archive pages with complete post bodies.
   (If the body text renders, it is content, even if unrelated to the
   primary article.)
3. **All comment sections** (with `**author — date**` bylines).
4. **Copyright and legal text rendered on the page** — including footer
   copyright lines and full terms documents (needed for downstream
   filtering purposes).
5. Data tables, code blocks, forum threads, captions belonging to the
   page's subject.

## NOISE — remove from gold
1. Raw HTML/script fragments: literal tags, iframe markup, JS, entity
   maps, base64/__VIEWSTATE blobs, <option> dumps.
2. Machine junk: mass-repeated phrases, whitespace floods,
   annotator meta-commentary.
3. Navigation menus and breadcrumbs.
4. Sidebar widgets WITHOUT article bodies: popular-posts lists,
   subscribe/mailing-list blocks, archive link lists, blogrolls,
   news-headline tickers.
5. Related-article/teaser chrome: title-as-link lists, "More from X",
   thumbnails-with-headlines — anything pointing AT content without
   carrying its body.
6. Share bars, UI labels (Reply/Like/Report), pagination controls,
   login/search forms.

## Boundary rule
Sidebar/secondary placement does NOT make text noise; absence of body
text does. A sidebar containing a full second article = content (rule
C2). A sidebar listing ten headlines = noise (rule N4/N5).

## Process
- The original dev.jsonl.gz is NEVER modified; edits build
  dev_golden.jsonl.gz alongside it.
- Every fleet edit spec must be mechanically verified (apply → check
  article + comments survive) before inclusion.

## Refinement (owner ruling, 2026-08-08 — biotech-capital case)
Dated timeline/newswire sections whose entries carry SUMMARY BODY TEXT
(headline + at least a sentence of prose per entry) are CONTENT, even
when they link onward — the body-text rule (C2/N5 boundary) governs,
not the section's teaser-like shape. Bare headline-link lists remain
noise (N5).

## Refinement (owner ruling, 2026-08-08 — weatherbug case)
An edit must not reduce a document to a husk. If applying a spec leaves
<300 chars or <15% of the original, the edit is suspect: legitimate
ONLY when the removed mass is unambiguous junk (N1/N2 dumps) or pure
no-body-text chrome. Data/listing/widget sections on a page ABOUT that
data (weather modules on a weather page) are content. When in doubt,
the edit is the error, not the doc.
