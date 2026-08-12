# Benchmark results

Extraction quality of this fork's Rust engine (`resiliparse._extract_rs`,
build `0177` / commit `850891b`) against upstream resiliparse and published
baselines. Every number below is reproduced from the result files in
`benchmark/runs/`; methodology and per-cycle history live in
[`research_log/`](research_log/) (cycles 0174–0179).

Unless stated otherwise our configuration is
`extract_plain_text(html, main_content=True, preserve_formatting='markdown')`.

---

## 1. External benchmarks — ours vs upstream resiliparse

Same documents, same scorer for both columns (each benchmark's own metric).
Upstream numbers come from the benchmarks' stored baseline predictions, except
WebMainBench which is a live run of upstream's Cython extractor.

| Benchmark | Docs | Metric | Upstream resiliparse | **This fork** | Δ |
|---|--:|---|--:|--:|--:|
| [marin devset v2](#sources) | 420 | token-F1 (4-gram shingle) | 0.8880 | **0.9050** | **+0.0170** |
| [Zyte article-extraction-benchmark](https://github.com/scrapinghub/article-extraction-benchmark) | 181 | token-F1 | 0.8806 | **0.8899** | +0.0093 |
| [WebMainBench](https://github.com/opendatalab/WebMainBench) (en/dev) | 200 | token-F1 | 0.8309 | **0.8633** | **+0.0324** |
| [trafilatura eval set](https://github.com/adbar/trafilatura/tree/master/tests/evaldata) | 960 | F1 (with/without containment) | 0.8104 | **0.8421** | **+0.0317** |
| Extraction unit tests | 100 | tests passed | 90 | **97** | +7 |

Full detail per benchmark (ours, build 0177):

| Benchmark | F1 | Precision | Recall | Levenshtein sim |
|---|--:|--:|--:|--:|
| marin devset | 0.9050 | 0.8673 | 0.9462 | 0.8383 |
| Zyte | 0.8899 | 0.8106 | 0.9864 | — |
| WebMainBench (shingle) | 0.8633 | 0.8051 | 0.9305 | 0.7916 |
| trafilatura eval set | 0.8421 | 0.7656 | 0.9354 | (accuracy 0.8251) |

The recurring shape: **recall 0.93–0.99, precision is the looser side** — our
residual errors are additive chrome, not lost content. That asymmetry is
deliberate and was verified by per-domain content audits (§4).

**Unit-test breakdown (97/100):** code 13/13, math 10/10, tables 7/7,
attribution 12/12, structure 5/5 — every content category perfect. Upstream's
7-test deficit is entirely negatives (chrome that should be dropped) and table
syntax. Our 3 failures: 2 prose-recall misses, 1 non-Latin footer.

**marin per-document distribution** (420 gold docs): mean 0.8927, median
**0.9491**, 283 docs ≥0.90, 356 ≥0.80, 17 below 0.50. The low tail is
dominated by the dataset's *script-loss edge cases* — see §6.

---

## 2. WebMainBench fine-grained metrics

WebMainBench's own structured metrics (edit-distance per content type, plus
TEDS for table structure), en/dev subset, `use_llm=false` for every row so the
comparison is like-for-like. **Overall = mean of the five core metrics.**

| Extractor | overall | text | code | formula | table | TEDS |
|---|--:|--:|--:|--:|--:|--:|
| **This fork (0177)** | **0.6150** | **0.7951** | **0.8229** | **0.3673** | 0.4143 | **0.6756** |
| Dripper | 0.5852 | 0.7174 | 0.7647 | 0.2488 | **0.5343** | 0.6609 |
| LLM stack (mdx-v5think) | 0.5115 | 0.7405 | 0.5908 | 0.3522 | 0.3294 | 0.5447 |
| LLM stack (mdx-v12) | 0.4997 | 0.7614 | 0.4845 | 0.3441 | 0.3577 | 0.5508 |
| LLM stack (mdx-d5+guards) | 0.4989 | 0.7547 | 0.4909 | 0.3497 | 0.3487 | 0.5504 |
| trafilatura | 0.3558 | 0.7733 | 0.1609 | 0.3508 | 0.1828 | 0.3113 |
| extract_head | 0.2948 | 0.7836 | 0.0891 | 0.3069 | 0.1245 | 0.1699 |
| upstream resiliparse | 0.2352 | 0.7561 | 0.0562 | 0.3635 | 0.0000 | 0.0000 |

First place overall and on text, code, formula and TEDS. Two honest caveats:

- **table_edit is the one loss** (0.4143 vs Dripper's 0.5343). The benchmark's
  ground truth serializes tables as raw nested `<table>` HTML echoed from the
  DOM, which Dripper reproduces; we emit GFM pipe tables. Our **TEDS is
  0.95–0.99 on the very documents where table_edit is near zero** — the table
  *structure* matches, only the serialization string differs. Re-encoding our
  tables into their HTML convention scored *worse* overall (0.6065), so we
  keep the readable form. (Cycle 0176.)
- **formula scores are low for everyone** — the metric regex-matches `$…$`
  spans with the currency-filtering LLM disabled, and most formulas on these
  pages are MathML or images no text extractor can reproduce. On the 115
  annotated-equation documents ours scores 0.3838 vs Dripper 0.1579.

Metric denominators (ours): text 200 docs, code 42, table 41, formula 126.

---

## 3. Internal sets (this repo's harness)

`benchmark/eval/run_eval.py`, ROUGE-L F1 + Levenshtein similarity per document.
`lpv11 dev_golden` is the primary optimization target and carries a
zero-regression policy: no change ships if any document regresses.

| Split | Docs | F1 | Precision | Recall | Lev sim |
|---|--:|--:|--:|--:|--:|
| lpv11 dev_golden (primary) | 1000 | **0.8948** | 0.9035 | 0.9120 | **0.8156** |
| lpv11 dev | 1000 | 0.8577 | 0.9160 | 0.8506 | 0.7671 |
| lpv11 train | 9999 | 0.8166 | 0.8570 | 0.8361 | 0.7183 |
| general dev | 1000 | 0.8139 | 0.7611 | 0.9314 | 0.7233 |

---

## 4. Domain sets (code / math / science / tables)

Specialized sets that stress content types where mistakes are unacceptable.
These golds are LLM-authored *plain-text* references, so Levenshtein is
penalized by our markdown formatting — F1 is the meaningful column.

| Domain | dev docs | dev F1 | dev Lev | test docs | test F1 | test Lev |
|---|--:|--:|--:|--:|--:|--:|
| code | 11 | 0.8445 | 0.7666 | 4 | 0.7915 | 0.7377 |
| math | 2 | 0.8240 | 0.7151 | 2 | 0.5620 | 0.3961 |
| science | 3 | 0.9333 | 0.8729 | 2 | 0.5132 | 0.4405 |
| tables | 2 | **0.8870** | 0.4620 | 5 | 0.8259 | 0.6906 |

**Content-fidelity audits found zero content loss.** Across the dev sets:
157 code blocks byte-exact, 26/26 LaTeX spans preserved, no deleted science
paragraphs, 530 table cells with associations intact. Every measured failure
is *additive* chrome.

The low math/science **test** numbers were investigated rather than accepted:
those 2-document splits have golds where **26–27 of ~33 paragraphs do not
appear in the source HTML at all** (LLM rewriting/abridgement). Direct
containment checks confirmed our output carries the source-present content,
including all LaTeX. The single genuine miss anywhere is a link-formatted
reference list on one PubMed-style page (logged as a known limitation).

Tables dev improved **0.6983 → 0.8870** in cycle 0177 (checkbox-widget veto).

---

## 5. Runtime

7-run best-of, 1000 lpv11 dev documents, single-threaded, Apple silicon:

| Configuration | ms/doc |
|---|--:|
| markdown + main_content | **3.00** |
| plain text + main_content | 1.32 |

Reference point at cycle 0116 was 2.76 / 1.30; quality cycles 0161–0177 cost
+8.7% on the markdown path.

---

## 6. Known limitation: JavaScript-hydrated pages

Extraction operates on raw HTML with no JavaScript execution, so content that
exists only inside `<script>` hydration state is unreachable. The marin devset
deliberately includes **27 "script-loss" edge cases** (Quizlet decks, Khan
Academy transcripts, some BBC/Economist articles, answers.com Q&A) where
50–100% of the gold lives in hydration blobs; these dominate our low tail
(e.g. a Khan Academy page scores F1 0.204 — precision 0.984, recall 0.114:
what we emit is correct, most of the gold was never in the DOM).

A proof-of-concept state miner (gated JSON extraction, still no JS execution)
takes that document from **0.204 → 0.785**. The design and its safety gating
are queued in [`research_log/QUEUE.md`](research_log/QUEUE.md); prevalence of
the trigger shape is 4.4% of marin and 5.9% of WebMainBench documents.

---

## Sources

| Resource | Link |
|---|---|
| Zyte article-extraction-benchmark | https://github.com/scrapinghub/article-extraction-benchmark |
| WebMainBench | https://github.com/opendatalab/WebMainBench |
| trafilatura (eval data) | https://github.com/adbar/trafilatura |
| Marin project | https://github.com/marin-community/marin |
| Upstream resiliparse | https://github.com/chatnoir-eu/chatnoir-resiliparse |

The **marin devset v2** (1,934 register-stratified CommonCrawl pages, 420 with
human-verified gold extractions), the **extraction unit-test suite**, and the
**lpv11 / general / domain** sets are curated in a companion research repo and
are not public; their datasheets travel with the data. All extractor code and
evaluation harnesses needed to reproduce the numbers on public benchmarks are
in this repository.

## Reproducing

```bash
# internal sets
python benchmark/eval/run_eval.py --dataset lpv11 --split dev_golden \
    --impl rust --workers 2 --tag mytag --preserve-formatting markdown

# external benchmarks (needs the companion data repo)
python benchmark/external/run_external.py zyte mytag
python benchmark/external/run_external.py marin mytag
```
