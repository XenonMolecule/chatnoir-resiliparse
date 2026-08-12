# Benchmark results

Extraction quality of this fork's Rust engine (`resiliparse._extract_rs`,
build `0177` / commit `850891b`) against upstream resiliparse and published
baselines. Every number below is reproduced from the result files in
`benchmark/runs/`; methodology and per-cycle history live in
[`research_log/`](research_log/) (cycles 0174–0179).

Unless stated otherwise our configuration is
`extract_plain_text(html, main_content=True, preserve_formatting='markdown')`.

---

## 1. External benchmarks — five extractors, one scorer per benchmark

Every row is scored by `benchmark/external/score_all_extractors.py` from each
extractor's raw predictions, so nothing here mixes published numbers with
locally measured ones. Baseline predictions are the stored outputs of Dripper,
marin v1.1, upstream resiliparse and upstream trafilatura, except the two
WebMainBench baselines, which are live runs (upstream resiliparse 1.0.9,
trafilatura 2.2.0 with `output_format='markdown'`).

**The honest summary: we lead the two broad-web benchmarks (marin,
WebMainBench) and place third on the two article-centric ones (Zyte,
trafilatura eval), where trafilatura and Dripper cut chrome more
aggressively. We beat upstream resiliparse on all four.**

### 1.1 marin devset v2 — 420 gold docs, token-F1

Register-stratified CommonCrawl (news, forums, science, code, commerce…).

| Extractor | F1 | Precision | Recall |
|---|--:|--:|--:|
| **This fork (0177)** | **0.9050** | 0.8673 | 0.9462 |
| Dripper | 0.8943 | 0.8917 | 0.8969 |
| resiliparse (upstream) | 0.8880 | 0.8451 | 0.9355 |
| trafilatura (upstream) | 0.8518 | 0.9075 | 0.8026 |

### 1.2 WebMainBench en/dev — 200 docs, token-F1

Diverse modern web including code, tables and formulas.

| Extractor | F1 | Precision | Recall |
|---|--:|--:|--:|
| **This fork (0177)** | **0.8633** | 0.8051 | 0.9305 |
| resiliparse (upstream) | 0.8309 | 0.7693 | 0.9033 |
| trafilatura (upstream) | 0.8073 | 0.8459 | 0.7721 |
| Dripper | 0.7482 | 0.7328 | 0.7643 |

(WebMainBench's own fine-grained metrics are in §2.)

### 1.3 Zyte article-extraction-benchmark — 181 news articles, token-F1

| Extractor | F1 | Precision | Recall |
|---|--:|--:|--:|
| trafilatura (upstream) | **0.9520** | 0.9278 | 0.9775 |
| Dripper | 0.9461 | 0.9073 | 0.9884 |
| **This fork (0177)** | 0.8899 | 0.8106 | **0.9864** |
| resiliparse (upstream) | 0.8806 | 0.7997 | 0.9799 |
| marin v1.1 | 0.7766 | 0.7169 | 0.8471 |

### 1.4 trafilatura eval set — 960 docs, containment F1

Upstream methodology: each document lists strings that must appear (`with`)
and strings that must not (`without`).

| Extractor | F1 | Precision | Recall | Accuracy |
|---|--:|--:|--:|--:|
| Dripper | **0.9335** | 0.9166 | 0.9511 | 0.9325 |
| trafilatura (upstream) | 0.9130 | 0.9119 | 0.9141 | 0.9132 |
| **This fork (0177)** | 0.8421 | 0.7656 | 0.9354 | 0.8251 |
| resiliparse (upstream) | 0.8104 | 0.7142 | 0.9365 | 0.7816 |
| marin v1.1 | 0.2353 | 0.8801 | 0.1358 | 0.5601 |

### 1.5 Extraction unit tests — 100 tests

| Extractor | Passed |
|---|--:|
| **This fork (0177)** | **97** |
| resiliparse (upstream) | 90 |

Breakdown (ours): code 13/13 · math 10/10 · tables 7/7 · attribution 12/12 ·
structure 5/5 — every content category perfect. Failures: 2 prose-recall
misses, 1 non-Latin footer.

### Reading the spread

Our recall is the highest or near-highest everywhere (0.93–0.99, and the top
recall of all five on Zyte); **precision is consistently our weak side**. On
article benchmarks with a single clean body, aggressive boilerplate cutting is
almost free, which is where trafilatura and Dripper gain — trafilatura's own
eval set and Zyte are both that shape. On mixed-register and structured
content (marin, WebMainBench) the same aggression costs them recall and we
lead. Our residual errors are additive chrome rather than lost content, which
the per-domain audits in §4 verify directly.

marin per-document distribution (ours): mean 0.8927, median **0.9491**,
283/420 ≥0.90, 356/420 ≥0.80, 17 below 0.50 — the low tail is the
script-loss family described in §6.

Upstream's 7-test deficit on the unit suite is entirely negatives (chrome that
should be dropped) and table syntax — no content-category difference.

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
