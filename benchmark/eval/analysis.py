"""Analysis layer over a resiliparse eval run -- the importable core behind viz.py.

Joins, per doc, the benchmark input (html + gold), the resiliparse prediction, and the
metrics, into a :class:`Doc`. A :class:`Run` is the collection for one (tag, split)
plus ranking / search / breakdown / distribution helpers.

    from analysis import load_run
    run = load_run("v3.0.2-9fb3340", "dev")
    for d in run.worst(10):            # 10 lowest ROUGE-L F1
        print(d.id, d.score, d.primary_tag)
    run.find(pred_empty=True)          # docs resiliparse returned nothing for
    run.breakdown("has_code")          # mean/median score with vs without code

GUARDRAIL: the test split is held out. load_run(..., "test") raises unless you pass
allow_test=True. Day-to-day analysis is dev/train only.
"""

import difflib
import gzip
import json
import os
import re
from statistics import mean, median

_HERE = os.path.dirname(os.path.abspath(__file__))
BENCH_DIR = os.path.dirname(_HERE)
RUNS_DIR = os.path.join(BENCH_DIR, "runs")
DATASETS_DIR = os.environ.get("RESILIPARSE_DATASETS_DIR", os.path.join(BENCH_DIR, "datasets_rawhtml"))
DEFAULT_DATASET = "lpv11"

_NON_ASCII_ALPHA = re.compile(r"[^\x00-\x7f]")
_WS = re.compile(r"\s+")

# Failure tags, in priority order (first match wins for `primary_tag`).
TAG_PRIORITY = [
    "ERROR",          # resiliparse raised
    "EMPTY_PRED",     # returned nothing, gold is non-empty
    "NON_LATIN",      # gold is mostly non-ASCII (stoplist-language mismatch)
    "UNDER_EXTRACT",  # kept far less text than gold
    "OVER_EXTRACT",   # kept far more text than gold (boilerplate leaked in)
    "WHITESPACE",     # content matches; only whitespace/newlines differ
    "PARTIAL",        # everything else imperfect
    "GOOD",           # ROUGE-L F1 >= 0.95
]


def _normalize_ws(text):
    return _WS.sub(" ", text).strip()


def _non_latin_fraction(text):
    letters = [c for c in text if c.isalpha()]
    if not letters:
        return 0.0
    return sum(bool(_NON_ASCII_ALPHA.match(c)) for c in letters) / len(letters)


class Doc:
    """One benchmark document with its prediction and scores."""

    def __init__(self, record, prediction, metrics, index=None):
        self.index = index  # position within the split (stable join key)
        # `... or ""` guards against keys present with an explicit null value
        # (the domain datasets carry null snapshot/url/id on some rows).
        self.id = record.get("warc_record_id")
        self.url = record.get("url") or ""
        self.snapshot = record.get("snapshot") or ""
        self.split = record.get("split") or ""
        self.has_table = bool(record.get("has_table", False))
        self.has_code = bool(record.get("has_code", False))
        self.has_list = bool(record.get("has_list", False))
        self.html = record.get("html") or ""
        self.gold = record.get("final_output") or ""
        self.pred = prediction.get("prediction", "")
        self.n_kept = prediction.get("n_kept", 0)
        self.n_paragraphs = prediction.get("n_paragraphs", 0)
        self.error = prediction.get("error") or metrics.get("error")
        self.metrics = metrics
        self.tags = self._classify()

    # -- convenience accessors -------------------------------------------- #
    @property
    def score(self):
        """Primary score = ROUGE-L F1."""
        return self.metrics.get("rougeL_f", 0.0)

    @property
    def lev(self):
        return self.metrics.get("lev_distance", 0)

    @property
    def gold_chars(self):
        return self.metrics.get("gold_chars", len(self.gold))

    @property
    def pred_chars(self):
        return self.metrics.get("pred_chars", len(self.pred))

    @property
    def length_ratio(self):
        return self.pred_chars / self.gold_chars if self.gold_chars else 0.0

    @property
    def primary_tag(self):
        for tag in TAG_PRIORITY:
            if tag in self.tags:
                return tag
        return "PARTIAL"

    @property
    def whitespace_only(self):
        """True if pred and gold differ ONLY in whitespace (the collapse issue)."""
        return self.pred != self.gold and _normalize_ws(self.pred) == _normalize_ws(self.gold)

    # -- classification --------------------------------------------------- #
    def _classify(self):
        tags = set()
        if self.error:
            tags.add("ERROR")
        if self.gold and not self.pred:
            tags.add("EMPTY_PRED")
        if _non_latin_fraction(self.gold) > 0.30:
            tags.add("NON_LATIN")
        if self.pred and self.gold_chars and self.length_ratio < 0.5:
            tags.add("UNDER_EXTRACT")
        if self.gold_chars and self.length_ratio > 1.5:
            tags.add("OVER_EXTRACT")
        if self.whitespace_only:
            tags.add("WHITESPACE")
        if self.score >= 0.95:
            tags.add("GOOD")
        elif not tags:
            tags.add("PARTIAL")
        return tags

    # -- diff ------------------------------------------------------------- #
    def diff_ops(self, granularity="word"):
        """Yield (op, gold_text, pred_text) opcodes.

        op in {'equal','delete','insert','replace'}. 'delete' = in gold, dropped by
        resiliparse (recall miss); 'insert' = resiliparse added, not in gold (precision miss).
        granularity 'word' ignores whitespace differences; 'char' shows them.
        """
        if granularity == "char":
            g, p = list(self.gold), list(self.pred)
            join = ""
        else:
            g, p = self.gold.split(), self.pred.split()
            join = " "
        sm = difflib.SequenceMatcher(a=g, b=p, autojunk=False)
        for op, i1, i2, j1, j2 in sm.get_opcodes():
            yield op, join.join(g[i1:i2]), join.join(p[j1:j2])

    def as_dict(self):
        return {
            "id": self.id, "url": self.url, "snapshot": self.snapshot,
            "has_table": self.has_table, "has_code": self.has_code,
            "has_list": self.has_list, "primary_tag": self.primary_tag,
            "tags": sorted(self.tags), **self.metrics,
        }


class Run:
    """All docs for one (tag, split), with analysis helpers."""

    def __init__(self, tag, split, docs, dataset=DEFAULT_DATASET):
        self.tag = tag
        self.dataset = dataset
        self.split = split
        self.docs = docs
        # Only non-null, non-colliding ids are addressable by id; everything is
        # always addressable by position (#index), which is collision-proof.
        seen = {}
        for d in docs:
            if d.id:
                seen[d.id] = None if d.id in seen else d
        self._by_id = {k: v for k, v in seen.items() if v is not None}

    def __len__(self):
        return len(self.docs)

    # -- lookup ----------------------------------------------------------- #
    def get(self, doc_id):
        """Fetch by position (#N), exact id, or unique id-prefix.

        Use #N for null/duplicate-id rows (the domain datasets have them).
        """
        if isinstance(doc_id, int):
            return self.docs[doc_id]
        if isinstance(doc_id, str) and doc_id.startswith("#"):
            return self.docs[int(doc_id[1:])]
        if doc_id in self._by_id:
            return self._by_id[doc_id]
        hits = [d for d in self.docs if d.id and d.id.startswith(doc_id)]
        if len(hits) == 1:
            return hits[0]
        if not hits:
            raise KeyError(f"no doc matching {doc_id!r}")
        raise KeyError(f"{doc_id!r} is ambiguous ({len(hits)} matches)")

    # -- ranking ---------------------------------------------------------- #
    def sorted_by(self, metric="rougeL_f", reverse=False):
        return sorted(self.docs, key=lambda d: d.metrics.get(metric, 0), reverse=reverse)

    def worst(self, n=10, metric="rougeL_f"):
        return self.sorted_by(metric)[:n]

    def best(self, n=10, metric="rougeL_f"):
        return self.sorted_by(metric, reverse=True)[:n]

    def around_median(self, n=10, metric="rougeL_f"):
        ordered = self.sorted_by(metric)
        mid = len(ordered) // 2
        half = n // 2
        return ordered[max(0, mid - half): mid - half + n]

    # -- search ----------------------------------------------------------- #
    def find(self, gold=None, pred=None, url=None, diff_added=None, diff_dropped=None,
             tag=None, pred_empty=None, has_code=None, has_table=None, has_list=None,
             snapshot=None, predicate=None):
        """Filter docs. Text args are regexes; diff_* match only changed spans."""
        gold_re = re.compile(gold) if gold else None
        pred_re = re.compile(pred) if pred else None
        url_re = re.compile(url) if url else None
        add_re = re.compile(diff_added) if diff_added else None
        drop_re = re.compile(diff_dropped) if diff_dropped else None

        out = []
        for d in self.docs:
            if gold_re and not gold_re.search(d.gold):
                continue
            if pred_re and not pred_re.search(d.pred):
                continue
            if url_re and not url_re.search(d.url):
                continue
            if tag and tag.upper() not in d.tags:
                continue
            if pred_empty is not None and bool(not d.pred) != pred_empty:
                continue
            if has_code is not None and d.has_code != has_code:
                continue
            if has_table is not None and d.has_table != has_table:
                continue
            if has_list is not None and d.has_list != has_list:
                continue
            if snapshot and d.snapshot != snapshot:
                continue
            if add_re or drop_re:
                added = " ".join(p for op, g, p in d.diff_ops() if op in ("insert", "replace"))
                dropped = " ".join(g for op, g, p in d.diff_ops() if op in ("delete", "replace"))
                if add_re and not add_re.search(added):
                    continue
                if drop_re and not drop_re.search(dropped):
                    continue
            if predicate and not predicate(d):
                continue
            out.append(d)
        return out

    # -- aggregation ------------------------------------------------------ #
    def values(self, metric="rougeL_f"):
        return [d.metrics.get(metric, 0) for d in self.docs]

    def percentiles(self, metric="rougeL_f", ps=(0, 10, 25, 50, 75, 90, 100)):
        vals = sorted(self.values(metric))
        if not vals:
            return {}
        out = {}
        for p in ps:
            idx = min(len(vals) - 1, max(0, round(p / 100 * (len(vals) - 1))))
            out[p] = vals[idx]
        return out

    def tag_counts(self):
        counts = {}
        for d in self.docs:
            counts[d.primary_tag] = counts.get(d.primary_tag, 0) + 1
        return dict(sorted(counts.items(), key=lambda kv: -kv[1]))

    def breakdown(self, by, metric="rougeL_f"):
        """Group docs by an attribute; return {value: {n, mean, median}}."""
        groups = {}
        for d in self.docs:
            key = getattr(d, by, None) if hasattr(d, by) else d.metrics.get(by)
            groups.setdefault(key, []).append(d.metrics.get(metric, 0))
        return {
            k: {"n": len(v), "mean": mean(v), "median": median(v)}
            for k, v in sorted(groups.items(), key=lambda kv: mean(kv[1]))
        }


# --------------------------------------------------------------------------- #
# Loading
# --------------------------------------------------------------------------- #
def _read_jsonl(path):
    with open(path, encoding="utf-8") as handle:
        return [json.loads(line) for line in handle if line.strip()]


def list_datasets():
    """Return [dataset names] available under benchmark/datasets/."""
    if not os.path.isdir(DATASETS_DIR):
        return []
    return sorted(d for d in os.listdir(DATASETS_DIR)
                  if os.path.isdir(os.path.join(DATASETS_DIR, d)))


def list_runs():
    """Return {tag: {dataset: [splits...]}} for every cached run (test split hidden)."""
    if not os.path.isdir(RUNS_DIR):
        return {}
    out = {}
    for tag in sorted(os.listdir(RUNS_DIR)):
        tag_dir = os.path.join(RUNS_DIR, tag)
        if not os.path.isdir(tag_dir):
            continue
        for dataset in sorted(os.listdir(tag_dir)):
            ds_dir = os.path.join(tag_dir, dataset)
            if not os.path.isdir(ds_dir):
                continue
            splits = sorted({f.split(".")[0] for f in os.listdir(ds_dir)
                             if f.endswith(".metrics.jsonl") and not f.startswith("test")})
            if splits:
                out.setdefault(tag, {})[dataset] = splits
    return out


def compare(run_a, run_b, metric="rougeL_f"):
    """Per-doc deltas (run_b minus run_a) joined by position. Returns list of dicts
    sorted by delta ascending (biggest regressions first).

    Joins by position (not id) so it is robust to null/duplicate ids; both runs must
    cover the same dataset/split, which run_eval writes in a stable order.
    """
    if len(run_a.docs) != len(run_b.docs):
        raise ValueError(
            f"Cannot compare: A has {len(run_a.docs)} docs, B has {len(run_b.docs)}. "
            "Both runs must be on the same dataset/split."
        )
    rows = []
    for d_a, d_b in zip(run_a.docs, run_b.docs):
        va, vb = d_a.metrics.get(metric, 0), d_b.metrics.get(metric, 0)
        rows.append({"id": d_b.id or f"#{d_b.index}", "url": d_b.url, "a": va, "b": vb,
                     "delta": vb - va, "doc_a": d_a, "doc_b": d_b})
    return sorted(rows, key=lambda r: r["delta"])


def latest_tag():
    if not os.path.isdir(RUNS_DIR):
        return None
    tags = [d for d in os.listdir(RUNS_DIR) if os.path.isdir(os.path.join(RUNS_DIR, d))]
    if not tags:
        return None
    return max(tags, key=lambda t: os.path.getmtime(os.path.join(RUNS_DIR, t)))


def load_run(tag=None, split="dev", dataset=DEFAULT_DATASET, allow_test=False):
    """Load a Run for one (tag, dataset, split). Refuses test unless allow_test=True."""
    if split == "test" and not allow_test:
        raise PermissionError(
            "The test split is held out for final evaluation. Analyze on 'dev' or "
            "'train'. Pass allow_test=True only for the final, one-shot test run."
        )
    tag = tag or latest_tag()
    if not tag:
        raise FileNotFoundError(f"No runs found under {RUNS_DIR}; run run_eval.py first.")

    run_dir = os.path.join(RUNS_DIR, tag, dataset)
    predictions = _read_jsonl(os.path.join(run_dir, f"{split}.predictions.jsonl"))
    metric_rows = _read_jsonl(os.path.join(run_dir, f"{split}.metrics.jsonl"))

    split_path = os.path.join(DATASETS_DIR, dataset, f"{split}.jsonl.gz")
    with gzip.open(split_path, "rt", encoding="utf-8") as handle:
        records = [json.loads(l) for l in handle if l.strip()]

    # Join by position, NOT by warc_record_id: the domain datasets contain rows with
    # null/duplicate ids, and run_eval writes predictions + metrics in split order.
    if not (len(records) == len(predictions) == len(metric_rows)):
        raise ValueError(
            f"Row-count mismatch for {tag}/{dataset}/{split}: "
            f"data={len(records)} preds={len(predictions)} metrics={len(metric_rows)}. "
            "The cached run is stale -- re-run run_eval.py for this dataset/split."
        )
    docs = [Doc(records[i], predictions[i], metric_rows[i], index=i)
            for i in range(len(records))]
    return Run(tag, split, docs, dataset=dataset)
