# 0152: machine-junk sweep — NEGATIVE (false positives), 1 doc flagged for owner

Second approved-family sweep (after 0151's dedupe). Scanned all 1000 golds
for raw HTML tags, JS, CSS and HTML entities. 10 docs matched; inspection
shows the family is almost entirely FALSE POSITIVES:

- **encycolorpedia** — `` `<p>#fcd9ae primeiro plano</p>` `` is inside
  backticks: a code sample on a colour-reference page. Content.
- **postgresql docs / coursereport / dennis2society** — JS snippets are
  documented code. Content.
- **ithemes codex** — MediaWiki page whose subject IS markup. Content.
- **wowdigsite / wildsolutions** — literal `&nbsp;` inside prose the
  annotator kept.

No autonomous edits made. The charter's "raw HTML dump" rule was written
for machine junk, and every match here is markup-as-subject-matter — a
distinction only inspection reveals, which is why this ran as a census
first rather than an edit.

## Flagged for owner ruling (not applied)
**bhagpuss.blogspot** (F1 0.818): 28 of 54 gold lines are raw
`<table>`/`<img>` HTML the annotator preserved to represent image layout.
Removing them would raise our score (we do not emit images), but that is a
content judgement about how images should be represented in gold, not
chrome removal — the same class of question as the three rulings in 0143.
Deliberately left for the owner.
