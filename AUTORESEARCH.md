# Auto-research playbook: resiliparse extraction quality

This document describes how to run the **auto-research pipeline** on resiliparse's
HTML→text extraction, replicating the loop that took the jusText fork
(https://github.com/XenonMolecule/jusText) from **0.762 → 0.889 ROUGE-L F1** over
~100 logged, committed research cycles. Read this top to bottom before the first
cycle; it encodes both the mechanics (harness, datasets, commit conventions) and
the hard-won methodology (what worked, what repeatedly failed).

**The subject under improvement** is resiliparse's extraction,
`extract_plain_text(html, main_content=True)`. Today the only implementation is
Cython (`resiliparse-py/resiliparse/extract/html2text.pyx`); `resiliparse-rs`
contains only the parse layer (lexbor bindings, DOM, CSS, serialize) — upstream
is actively rewriting the toolkit in Rust but hasn't reached the extract module
yet. The plan is a **timeboxed spike** (see §8): try to port `html2text` to
Rust on top of the Rust DOM (`resiliparse-rs/src/parse/html/`), gated on output
parity with the Cython version. **If parity lands, all iteration happens in
Rust and the Cython module freezes as the reference. If the spike fails the
timebox, fall back to iterating on the Cython implementation** — the whole
harness, cycle process, and speed policy below apply identically either way
(the `--impl` flag makes the extractor a swappable detail).

Two objectives, both first-class:

1. **Quality** — output matching an LLM-distilled gold that is now
   **markdown-flavored** (headings, bold, lists, pipe tables, code fences), so
   "extraction quality" includes producing markdown structure, not just picking
   the right text.
2. **Speed** — resiliparse's reason to exist is throughput. Every cycle reports
   ms/doc next to F1/Lev, and performance regressions block shipping the same
   way quality regressions do (see §6).

---

## 1. The loop in one paragraph

Fork the repo, build an eval harness that runs the current build over a fixed
benchmark and caches per-doc scores under a per-build tag, then iterate in small
preregistered cycles: pick a hypothesis, change the code, measure on train/dev,
compare per-doc against the previous tag, write a short research-log entry
(numbered `NNNN-slug.md`), and **commit the code + log entry together** with a
`NNNN: <description>` commit message. The held-out test split stays vaulted
except at explicit milestones. Negative results get logged and committed too —
the log is the memory that stops you from re-trying dead ends.

## 2. One-time setup (cycle 0)

### 2.1 Fork + remotes

This clone currently has `origin = chatnoir-eu/chatnoir-resiliparse`. Mirror the
jusText setup (fork = origin, original = upstream):

```bash
# creates github.com/<you>/chatnoir-resiliparse and re-points remotes
gh repo fork chatnoir-eu/chatnoir-resiliparse --remote
git remote -v   # verify: origin = your fork, upstream = chatnoir-eu
```

Work on a long-lived branch of the fork (jusText used the fork's `main`;
here branch `autoresearch` off `develop`). Push to the fork after each cycle —
never to upstream.

### 2.2 Build from source

Two builds are needed: the Rust workspace (the port target for the §8 spike)
and the Cython package (the reference baseline — and the iteration target if
the spike falls back).

```bash
# Rust workspace (resiliparse-rs is a default member; lexbor via bindgen/vcpkg)
cargo build --release && cargo test

# Cython reference (see docs/man/installation.rst for vcpkg prerequisites)
python3 -m pip install -e ./resiliparse-py
```

The Rust extractor should be reachable from the eval harness. Follow the
workspace's existing PyO3 pattern (`resiliparse-py/resiliparse/parse/_html_rs`
is a PyO3 crate in the same Cargo workspace): add an `extract` module/crate
exposing the Rust `extract_plain_text` to Python. **Always benchmark and
measure with `--release`** — debug-build timings are meaningless — and rebuild
before every measurement so you never score a stale binary (bake
`cargo build --release` into the eval invocation). Sanity check once bound:

```python
from resiliparse._extract_rs import extract_plain_text   # or wherever it's exposed
extract_plain_text("<html><body><main><h1>t</h1><p>hi</p></main>", main_content=True)
```

### 2.3 Copy the eval harness from jusText

The harness is deliberately extractor-agnostic; ~95% copies over unchanged from
the jusText repo (local path: `/Users/michaelryan/Documents/School/Stanford/Research/jusText`,
remote: https://github.com/XenonMolecule/jusText):

```bash
JT=/Users/michaelryan/Documents/School/Stanford/Research/jusText
mkdir -p benchmark/eval benchmark/runs benchmark/datasets
cp $JT/benchmark/eval/{run_eval.py,metrics.py,analysis.py,viz.py,report.py} benchmark/eval/
```

Then adapt `run_eval.py`:

- Replace the jusText worker with resiliparse. The whole extractor surface is
  one function:

  ```python
  def _extract(item):
      index, html = item
      start = perf_counter()
      try:
          prediction = extract_plain_text(html, main_content=True)
          error = None
      except Exception as exc:      # one bad doc must never kill the run
          prediction, error = "", repr(exc)
      return {"index": index, "prediction": prediction,
              "runtime_ms": (perf_counter() - start) * 1000.0, "error": error}
  ```

- Drop the stoplist/`--model` machinery (jusText-specific). Add an
  `--impl {rust,cython}` flag that selects which extractor the worker calls, so
  the Cython reference and the Rust build score through the identical pipeline
  (that's what makes parity and speedup claims trustworthy). Add flags for
  extraction options as you start experimenting with them.
- Keep everything else **exactly as is**: the run cache
  (`benchmark/runs/<tag>/<dataset>/<split>.{predictions,metrics}.jsonl` +
  `summary.json`), the auto-tag `v<version>-<gitsha>[-dirty]`, `--tag` for named
  experiments, `--skip-run` re-scoring, `--limit` smoke tests, and the
  **`--allow-test` vault** that refuses to touch the test split.
- Point the default datasets dir at the raw-HTML sets (see §3):
  `DATASETS_DIR = os.environ.get("RESILIPARSE_DATASETS_DIR", ...)`, default
  dataset `lpv11`.

`metrics.py` needs no changes (rapidfuzz-based, see §4). `analysis.py`/`viz.py`
are the analysis layer you will live in: `overview`, `rank --worst`, `tags`
(failure taxonomy), `search --dropped <regex>`, `diff <id> --only dropped`, and
crucially `compare <tagA> <tagB>` which joins per-doc by id and prints mean delta
plus biggest regressions/improvements — that per-doc join is what makes a
zero-regression policy enforceable.

### 2.4 Link the datasets

Datasets are large and stay **untracked** (add `benchmark/datasets*` and
`benchmark/runs` to `.gitignore`; commit only the small `manifest.json`s if
desired). Symlink from the jusText repo rather than copying ~4GB:

```bash
JT=/Users/michaelryan/Documents/School/Stanford/Research/jusText
ln -s $JT/benchmark/datasets_rawhtml benchmark/datasets_rawhtml
ln -s $JT/benchmark/datasets benchmark/datasets_preprocessed
```

### 2.5 Create the research log

```bash
mkdir research_log
cp $JT/research_log/README.md research_log/   # conventions + cycle definition, edit names
```

Start `research_log/0001-baseline.md` (baseline = stock
`extract_plain_text(main_content=True)` on lpv11 dev, both raw-HTML and
preprocessed variants) and an empty `research_log/QUEUE.md` for the deferred-idea
backlog. Keep a short index table at the bottom of the README, one line per
entry, updated every cycle.

---

## 3. Datasets — pointers and priority

All local paths are under `/Users/michaelryan/Documents/School/Stanford/Research/jusText/benchmark/`.

### 3.1 PRIMARY: `lpv11` (use this to kick off — higher priority than everything else)

The new benchmark, distilled with the `llm_pipeline_v1_1` spec. Its gold **is the
target markdown flavor**. Splits are WARC-disjoint from each other **and** from
the old benchmark, spanning CC-MAIN-2013-20 … 2022-49.

| variant | path | `html` field |
|---|---|---|
| **raw HTML (default)** | `datasets_rawhtml/lpv11/` | `raw_html` — CC bytes decoded exactly as production sees them (utf-8, errors=replace) |
| preprocessed | `datasets/lpv11/` | `preprocess_html_for_extraction(raw_html)` — the lpv11 pipeline's own input transform |

Splits (same doc ids in both variants): **dev 1,000 · train 9,999 · test 1,000 ·
big_train 100,000** (big_train is per-WARC-capped, for any learned-model work).
Source of truth: `gs://marin-us-central2/datasets/extractor_eval_set/lpv11_source/joined`.

Record schema (one JSON object per line, gzip):

```
html, final_output (gold), url, warc_record_id, snapshot,
has_table, has_code, has_list, split, warc_hash
```

**Iterate against the raw-HTML variant.** jusText cycle 0073 measured this
directly: the pre-stripped/preprocessed html masks data-capture failures as
extraction failures, and raw HTML is what production feeds the extractor. Keep
the preprocessed variant for diagnosis: if a doc fails on raw but not
preprocessed, the bug is in handling full-page noise, not in content selection.

Markdown prevalence in lpv11 dev gold (of 1,000 docs): headings 748, `**bold**`
747, bullet lists 315, numbered lists 80, pipe tables 72, code fences 30. This is
not a plain-text target.

### 3.2 SECONDARY: the old LLM-pipeline benchmark (guardrails + domain probes)

The previous distillation (`gs://marin-us-central2/datasets/high_quality_3000_distill/data`),
kept as regression guardrails and domain-targeted probes — **lower priority than
lpv11**, but still measured because we still care about math/code/science/table
performance:

- `datasets/{general,math,code,science,table}/` — `{train,dev,test}.jsonl.gz`
  each, pre-stripped html. The domain splits are tiny: trust direction, not
  decimals.
- `datasets_rawhtml/general/` — raw-HTML general set, plus **`dev2`** and
  **`dev3`** (fresh 1,000/2,000-doc samples, disjoint from everything, added when
  dev started feeling overfit). jusText's per-cycle regression check ran on
  **dev2 + dev** — cheap and un-tuned-on — rather than re-running the 10k train
  every cycle. Do the same here once lpv11 dev has been iterated on for a while.
- `big_train[_rawhtml].jsonl.gz` + manifests at `benchmark/` root — the old 100k
  distill sets (used to train jusText's fastText classifier).

### 3.3 Split discipline

- **test** — vaulted. The harness refuses it without `--allow-test`. Touch it
  only at explicit milestones (jusText: the v4.2.0 release, cycle 0097), and
  record the milestone in the log.
- **dev** — the daily driver; every cycle reports it.
- **train** (10k) — pattern mining at scale and confirming a fix generalizes;
  not re-run every cycle.
- **big_train** (100k) — learned models only.

---

## 4. Metrics

From `metrics.py` (rapidfuzz-backed, fast enough for 1,000 docs/second-ish):

- **ROUGE-L F1 / P / R** — LCS over lowercase `\w+` tokens, computed exactly via
  Indel distance. The headline number. Note markdown punctuation (`**`, `|`, `#`)
  is *invisible* to this metric.
- **Levenshtein similarity** — char-level on the raw strings; whitespace, case,
  and markdown syntax all count. This is the metric that sees formatting quality.
  Report both; a formatting change that moves Lev but not F1 is real.

Predictions are the extractor output as a single string; if you build output from
paragraphs, join with `"\n\n"` (the gold's paragraph separator — jusText cycle
0102 exists because a downstream consumer joined with `"\n"` and mashed
everything together).

**Speed metrics** (first-class, reported every cycle alongside F1/Lev):

- **ms/doc (mean + p50/p95) and docs/s** on lpv11 dev — the harness already
  records `runtime_ms` per doc and aggregates it in `summary.json`; surface it
  in the per-cycle results table and in `viz.py compare` (a per-doc runtime
  join catches "one pathological doc got 100× slower" the same way it catches
  quality regressions).
- **Measurement discipline:** release/optimized builds only (`cargo build
  --release` on the Rust path; on the Cython path never measure a stale
  extension — rebuild after every `.pyx` edit); single-worker (`--workers 1`)
  for official timing so pool scheduling doesn't pollute per-doc numbers; same
  machine across compared tags (note the machine in the log entry the first
  time and on any change). Per-doc timing through the Python binding includes
  binding overhead — that's fine as the standard number (production also pays
  it). For micro-optimizing hot paths on the Rust path, add **criterion
  benches in `resiliparse-rs`** (`cargo bench`) on a fixed set of ~20
  representative dev pages checked into the bench fixtures.
- **The speed baseline is the stock Cython implementation** on the same docs,
  same machine, via the same harness (`--impl cython`, tag 0001). If the Rust
  spike (§8) wins, track the Rust/Cython ratio — the port should start at
  parity or faster and widen from there. If iterating on Cython, tag 0001 is
  still the ms/doc reference every cycle compares against.

## 5. The iteration cycle (per-cycle checklist)

One cycle ≈ one hypothesis ≈ one research-log entry ≈ one commit. From
`research_log/README.md` in the jusText repo, verbatim in spirit:

1. **Backfill** — add the git sha of the previous cycle's commit to the previous
   entry (it didn't exist when the entry was written).
2. **Review** — skim recent entries + `QUEUE.md`; note anything that informs today.
3. **Preregister** — write the Hypothesis section of a stub entry *before*
   coding. Keeps you honest about what you were actually testing.
4. **Revert (rare)** — if a past change should be undone, revert it as its own
   step first.
5. **Change** — implement. Multiple edits per hypothesis is normal; not every
   edit is a commit.
6. **Measure** — `python benchmark/eval/run_eval.py --dataset lpv11 --split dev`
   (auto-tags from git; use `--tag <name>` for experiments while dirty). Never
   touch test mid-cycle.
7. **Iterate** — refine and re-run freely.
8. **Log** — once the result is clear (win, quality-neutral ship, or ruled out),
   fill in: Results (vs. previous tag via `viz.py compare`), Insights, Next.
9. **Commit** — code + log entry together, message `NNNN: <short description>`
   (e.g. `0009: stop fragmenting rows/lists`). Follow-ups within the same idea:
   `NNNN fix:` / `NNNN polish:`. Queue-only commits: `Queue: <note>`.
10. **Repeat.**

Entry template:

```markdown
# NNNN — <title>

- **Date:** YYYY-MM-DD
- **Tag:** vX.Y.Z-<sha>   (baseline compared against: <prevTag>)
- **Status:** idea | in progress | landed | abandoned

## Hypothesis
## What changed
## Results        (small table vs. comparison tag; dev/train only)
## Insights       (what we learned, even/especially when it failed)
## Next
```

Keep entries under a 5-minute read. Log negatives with the same care as wins —
roughly a third of jusText's 100+ entries are NEGATIVE results, and they paid for
themselves every time an old dead end resurfaced looking shiny.

## 6. Shipping policy (guardrails)

- **Zero-regression rule.** A change ships only if `viz.py compare` shows no
  meaningful per-doc regressions on the guardrail sets. "Aggregate +0.001 but 39
  docs got worse" does not ship (jusText 0079). Per-doc comparison, not means.
- **Speed is a shipping gate, not a footnote.** Every cycle's results table
  includes ms/doc vs. the comparison tag. A quality win that costs meaningful
  throughput (rule of thumb: >5% mean ms/doc, or any new per-doc pathological
  blowup at p95+) doesn't ship as-is — either optimize it, gate it so the slow
  path only runs where it can matter (the jusText "self-correcting rescue"
  pattern is naturally cheap: it only fires on near-empty extractions), or log
  the trade-off explicitly and get a deliberate decision. Pure speed wins at
  quality Δ≈0 are shippable cycles in their own right.
- **Ship quality wins at Δmetric ≈ 0.** If output is genuinely better (correct
  indentation, proper attribution, a table that renders) and nothing regresses,
  ship it even when the aggregate doesn't move. The metric is a proxy.
- **The gold is imperfect.** In the old benchmark ~25%+ of apparent precision
  errors were *gold* errors (gold under-extracts). Never chase the metric by
  dropping content you can see is good. When gold and quality conflict, record
  the decision in the log and pick quality (jusText 0059/0086 precedent).
- **Verify markdown renders.** For any table/formatting change, run the output
  through a markdown renderer (`python-markdown`) and check it actually renders
  as a table — token-level metrics can't tell a valid pipe table from a broken one.
- **Tiny splits: direction, not decimals.** The domain sets and per-engine doc
  groups are small; a ±0.001 there is noise.

## 7. Methodology that worked (and what didn't) — jusText's 100 cycles, condensed

Carry these priors over; they are about web data and gold-matching generally, not
about jusText's classifier:

**Worked, in rough order of value:**

1. **Look at the data relentlessly.** The single highest-ROI move, every time it
   was tried: sample ~5 partial-F1 docs, read the diffs (`viz.py diff <id> --only
   dropped`), find the shared pathology, fix that. Wins repeatedly came from
   pathologies that sounded impossible until a specific doc made them concrete
   (mojibake repair, glued words at inline boundaries, `<br>`-split list markers).
2. **Structural/segmentation fixes beat threshold tuning.** "Stop fragmenting
   table rows and list items" was worth more than every classifier-threshold
   sweep combined. Where resiliparse splits or merges blocks is the first place
   to look.
3. **Platform-specific handlers with tight gates.** Forum engines (vBulletin,
   phpBB, XenForo, SMF, bbPress, JForum, MetaFilter…), DSpace, CONTENTdm: each a
   recognizable DOM signature → a handler that restructures to the gold's shape
   (`**username** (date):` before each post). Every attempt at a *general* forum
   detector was net-negative; every tightly-gated engine handler shipped clean.
   Gate on a signature that fires on zero non-target docs, verify with a
   train-wide fire count.
4. **Self-correcting rescues.** For catastrophic failures (near-empty output):
   try a fallback (re-merge concatenated `<html>` docs, unwrap content-in-links,
   pull JSON-LD/JS-state-blob content) and keep it **only if it yields more
   content**. Gated on "extraction was near-empty," these can't regress working
   pages — the pattern that made them all shippable.
5. **Whitespace/encoding hygiene.** Mojibake (ftfy, signature-gated), U+FFFD
   infill, double-encoded entities, `<br>`→newline, spaces at inline-element
   boundaries, preserving `<pre>` indentation. Individually small, collectively
   a large chunk of the Levenshtein gains, and near-zero regression risk.
6. **Dedup** of repeated paragraphs (quote-normalized, containment-aware) — but
   make it code-safe; code legitimately repeats lines.

**Failed repeatedly — don't re-try without new evidence:**

- Blanket heuristics tuned on aggregates (thresholds, keep-quotes-everywhere,
  general forum detection, punctuation-based code detection): they always
  regressed a long tail even when the mean moved up.
- Chasing gold typography that's *inconsistent in the gold* (which sections a
  wiki page keeps, quote-keeping in forums). Check gold consistency across ~5
  docs **before** building; if the gold disagrees with itself, it's a wall.
  Caveat: jusText-era "markdown bold is unpredictable" (0024) was a property of
  the *old* gold — lpv11's markdown is systematic, so formatting is now a real
  lever, not a wall.
- Fixing what's actually upstream data capture (JS-rendered SPAs, bad WARC
  charset decoding baked into the dataset). Detect and route around
  (needs-JS detector), don't try to extract what isn't in the bytes. See
  `WARC-decoding-recommendations.md` in the jusText repo.

## 8. Resiliparse-specific starting agenda

Suggested first cycles (revise after the baseline):

1. **0001 — Cython baseline (quality + speed reference).**
   `extract_plain_text(main_content=True)` via `--impl cython` on lpv11 dev,
   raw-HTML and preprocessed variants; also the old general/math/code/science/
   table devs for reference. Log F1/Lev, **ms/doc + docs/s**, failure-tag
   distribution, and the machine specs. Sweep the existing knobs
   (`preserve_formatting`, `list_bullets`, `alt_texts`, `links`) — the best
   stock config is the real baseline. This tag is the reference every later
   Rust cycle compares against; the Cython module is frozen after this.
2. **0002 — Rust port spike (TIMEBOXED: ~4 hours of focused work).** Attempt to
   port the extraction walk (`_extract_plain_text_impl` + the `ExtractOpts`/
   `FormattingOpts` machinery, main-content scoring, list/table handling) onto
   the Rust DOM in `resiliparse-rs`, exposed to Python via the workspace's PyO3
   pattern (like `parse/_html_rs`). Since we're building on upstream's active
   Rust rewrite, plan to merge `upstream/develop` periodically if this path wins.

   The spike has **two gates in sequence** — feasibility at the timebox, then a
   measured speed verdict — and iterating on Rust requires passing both.

   **Parity is measured against the Cython outputs, not the gold.** The cached
   predictions from tag 0001
   (`benchmark/runs/<0001-tag>/lpv11/dev.predictions.jsonl`) are the oracle:
   diff the Rust output doc-by-doc against the Cython output on the same docs.
   The harness gives this for free — score the Rust predictions with the
   *Cython predictions* in the gold slot, and mean Levenshtein similarity is
   the parity meter (1.0 = byte-identical port). No gold, no judgment calls
   about "within noise" — two implementations of the same algorithm should
   agree exactly.

   **Gate 1 — feasibility (at the ~4h timebox):** the spike *continues* past
   the timebox only if the port runs end-to-end on lpv11 dev (no crashes,
   `--release`) and matches the Cython output byte-for-byte on the large
   majority of docs (roughly ≥90%) with the remaining diffs enumerable and
   understood — i.e., parity is clearly a matter of finishing, not of fighting
   the DOM API or the build. Otherwise **stop, log the spike honestly as its
   own entry** (what blocked it: missing DOM affordances, build friction,
   logic too entangled), park "Rust port" in `QUEUE.md`, and fall back to
   Cython.

   **Parity bar (finish this before gate 2):** byte-identical output vs. the
   Cython reference across lpv11 dev (parity similarity ≈ 1.0), with any
   residual diffs individually documented and defensible (e.g. a Cython bug
   the port deliberately doesn't reproduce). Freeze a handful of these docs as
   golden tests in the Rust crate so future refactors can't silently drift.

   **Gate 2 — speed verdict (the point of the port):** with parity landed, run
   a head-to-head on lpv11 dev — same harness, same docs, same machine,
   `--workers 1`, `--impl rust` vs `--impl cython` — plus criterion benches on
   the fixture pages. Do not assume Rust wins: the Cython implementation is
   already mostly nogil C++ over the same lexbor parser, so a wash is a real
   possibility. Rules of thumb for the verdict (log the numbers and the
   decision either way):
   - **≥ ~1.2× faster mean ms/doc (and no worse p95):** adopt Rust as the
     iteration base.
   - **~parity (0.9–1.2×):** speed alone doesn't decide it; adopting Rust is
     then a strategic call (upstream is heading there; a clean port is
     upstreamable) — surface the numbers to the user for the call rather than
     deciding unilaterally.
   - **Slower and not quickly fixable:** iterate on Cython; keep the port
     parked with the measurements attached.

   **Fallback path (either gate fails): iterate on the Cython
   `html2text.pyx`** — editable install, remember Cython needs a rebuild
   before every measurement. Every later section of this playbook applies
   unchanged on the fallback path; the speed gate (§6) still holds.

   Either way: no quality iteration during the spike — a port bug must never be
   confusable with a quality change.
3. **Failure taxonomy.** Port `viz.py tags`; read the worst 20 docs. In jusText
   this immediately reoriented the roadmap (cycle 0006).
4. **Markdown output mode** (in whichever implementation won the spike). The
   single biggest quality gap: the gold
   has headings/bold/lists/tables/fences; `extract_plain_text` emits plain
   text. Extend the ported `FormattingOpts` (`FORMAT_OFF / FORMAT_BASIC /
   FORMAT_MINIMAL_HTML`) with a markdown flavor: `h1..h6 → #…######`,
   `b/strong → **`, `i/em → *`, `li → -` / `1.`, uniform `<table>` → pipe
   tables, `<pre>/<code>` → fenced blocks with indentation preserved. Verify
   renders (§6). Expect most of the win on Levenshtein first.
5. **Main-content selection gap-mining + speed cycles.** Then iterate §7-style:
   segmentation, hygiene, engine handlers, rescues — in whatever order the
   failure taxonomy says — interleaved with pure-performance cycles (criterion
   profiles, allocation pressure, single-pass restructuring) once quality
   plateaus between ideas.

Also keep in mind: math content (LaTeX-as-image transcription, MediaWiki `class="tex"`
images — jusText 0065/0098) and code formatting (line-numbered code tables → fenced
blocks, `&nbsp;` indentation — 0055/0056) both still matter under the markdown
flavor; the jusText cycles named here are the reference implementations.

## 9. Reference map (jusText repo)

| what | where |
|---|---|
| full research log + index | `research_log/README.md`, `research_log/NNNN-*.md` |
| idea backlog / deferred fixes | `research_log/QUEUE.md` |
| harness | `benchmark/eval/{run_eval,metrics,analysis,viz,report}.py` |
| cached runs (every tag ever measured) | `benchmark/runs/<tag>/` |
| WARC/charset decoding guidance | `WARC-decoding-recommendations.md` |
| fork usage/setup doc (end-state example) | `SETUP.md` |

Local: `/Users/michaelryan/Documents/School/Stanford/Research/jusText` ·
remote: https://github.com/XenonMolecule/jusText (upstream miso-belica/jusText).
