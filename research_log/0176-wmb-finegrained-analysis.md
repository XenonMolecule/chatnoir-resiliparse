# 0176: WebMainBench fine-grained — #1 on en-dev, beats Dripper +0.030

## Result (WebMainBench's own metrics, en-dev n=200, regex-only config)
All extractors scored by the same runner on the same subset; "ours-original"
= the extract_head row from the reference table, confirming the reference
numbers ARE en-dev and directly comparable.

| extractor | overall | text | code | formula | table_edit | TEDS |
|---|---:|---:|---:|---:|---:|---:|
| **ours-v0175** | **0.6150** | **0.7951** | **0.8229** | **0.3673** | 0.4143 | **0.6756** |
| Dripper | 0.5852 | 0.7174 | 0.7647 | 0.2488 | **0.5343** | 0.6609 |
| mdx-v12 (best LLM stack) | 0.4997 | 0.7614 | 0.4845 | 0.3441 | 0.3577 | 0.5508 |
| mdx-d5-guards | 0.4989 | 0.7547 | 0.4909 | 0.3497 | 0.3487 | 0.5504 |
| trafilatura (en-dev) | 0.3558 | 0.7733 | 0.1609 | 0.3508 | 0.1828 | 0.3113 |
| ours-original / extract_head | 0.2948 | 0.7836 | 0.0891 | 0.3069 | 0.1245 | 0.1699 |
| upstream resiliparse (en-dev) | 0.2352 | 0.7561 | 0.0562 | 0.3635 | 0.0000 | 0.0000 |

We are #1 overall and #1 on text, code, formula, and TEDS. Only loss:
table_edit vs Dripper (0.4143 vs 0.5343).

## Why we lose table_edit (and why it is not a content problem)
1. The WMB gold serializes tables as RAW HTML echoed from the DOM —
   `<table><tbody><tr><td>` trees including the page's nested LAYOUT tables.
   Dripper emits the same raw-HTML convention, so its string edit distance
   is near-free. We emit GFM pipe tables.
2. The tell: on our biggest table_edit losses (0aa9f0da 0.007, abc5d680
   0.018, 90e52c1a 0.039) our **TEDS is 0.95–0.99** — the parsed structure
   matches; only the serialization string doesn't.
3. The 3 docs where both our metrics are 0 (f99894fb, 9903c770, 7b3a4421)
   are old table-LAYOUT pages: gold kept the layout `<table>` wrapper; we
   correctly linearize layout tables to prose. Article text verified fully
   present in our output on all three. **Zero genuine content loss across
   all 36/41 table-scored docs.**
4. Counter-experiment: converting our pipe tables to the gold's HTML
   convention (adapter-side, `ours-v0175-htmltab`) scored WORSE — 0.3822
   table_edit / 0.6065 overall — because gold tables are nested layout
   trees; nothing short of echoing the raw DOM wins this string metric.
   Native pipe output is our best presentation. Convention-chasing rejected.

## Formula metric decoded
`$...$` regex with the currency-filtering LLM DISABLED (all runs,
use_llm=false) — two dollar prices in a paragraph count as a "formula",
which is why our denominator is 126 docs vs Dripper's 73 (we preserve more
text, including prices). On the 115 annotated-equation docs: **ours 0.3838
vs Dripper 0.1579** (2.4×); we zero on 66/115 vs their 95/115. The low
absolute ceiling is the metric: most en-dev formulas exist as MathML/images
that no text extractor can reproduce, so everyone zeroes on most docs.

## Is WMB en-dev worth adopting as an iteration devset?
**No, not for table_edit** — the only metric we lose is dominated by a
raw-DOM-echo convention that is antithetical to clean extraction (and the
counter-experiment shows partial alignment makes it worse). **Already won
everywhere else** — text/code/formula/TEDS are #1 and the remaining
headroom is metric noise (currency-formulas, unreproducible MathML).
The shingle-F1 track (0.8633, also #1) stays our WMB health check; the
en/test split (md5 parity) remains untouched/clean if we ever revisit.

## Artifacts
small-rephraser: results/webmainbench/WebMainBench-545/en-dev/
{ours-v0175, ours-v0175-htmltab}.{jsonl,summary.json}; predictions at
/tmp/wmb_ours_preds.jsonl (correct schema: metadata.record_index).
Extractor unchanged (v0175 = cycle 0175 build); no battery needed.
