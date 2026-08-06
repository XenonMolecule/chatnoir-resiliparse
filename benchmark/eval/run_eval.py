#!/usr/bin/env python3
"""Run a local resiliparse build over a benchmark split and score it.

Two stages, written to ``benchmark/runs/<tag>/<dataset>/``:

    1. run    -> <split>.predictions.jsonl   (extractor output per doc + timing)
    2. score  -> <split>.metrics.jsonl       (ROUGE-L + Levenshtein per doc)
                 <split>.summary.json        (aggregate scores, timing, config)

``<tag>`` identifies the build/config being evaluated; it defaults to
``v<version>-<git-sha>`` so different builds land in separate folders and stay
comparable. Use ``--skip-run`` to re-score existing predictions without
re-running the extractor.

``--impl`` selects the extractor implementation: ``cython`` (the reference
``resiliparse.extract.html2text``) or ``rust`` (the PyO3 port in
``resiliparse._extract_rs``). Both run through the identical pipeline, which is
what makes parity and speedup claims trustworthy. ``--impl rust`` triggers a
``cargo build --release`` first so a stale binary can never be scored.

Examples
--------
    python benchmark/eval/run_eval.py --split dev                       # lpv11 raw-HTML
    python benchmark/eval/run_eval.py --split dev --impl rust
    python benchmark/eval/run_eval.py --split dev --tag exp --workers 8
    python benchmark/eval/run_eval.py --split dev --skip-run            # re-score only
    python benchmark/eval/run_eval.py --split dev --limit 50            # quick smoke test
    python benchmark/eval/run_eval.py --split dev --gold-from-run <tag> # parity vs cached preds
"""

import argparse
import gzip
import json
import os
import subprocess
import sys
from concurrent.futures import ProcessPoolExecutor
from statistics import mean, median
from time import perf_counter

# Make this dir (for `metrics`) importable regardless of invocation cwd.
_HERE = os.path.dirname(os.path.abspath(__file__))
BENCH_DIR = os.path.dirname(_HERE)
REPO_DIR = os.path.dirname(BENCH_DIR)
sys.path.insert(0, _HERE)

from metrics import score_pair  # noqa: E402

DATASETS_DIR = os.environ.get("RESILIPARSE_DATASETS_DIR",
                              os.path.join(BENCH_DIR, "datasets_rawhtml"))
DEFAULT_DATASET = "lpv11"
PARAGRAPH_SEPARATOR = "\n\n"

# --------------------------------------------------------------------------- #
# resiliparse worker (runs in a process pool)
# --------------------------------------------------------------------------- #
_EXTRACT_FN = None
_EXTRACT_KWARGS = None


def _init_worker(impl, extract_kwargs):
    global _EXTRACT_FN, _EXTRACT_KWARGS
    if impl == "cython":
        from resiliparse.extract.html2text import extract_plain_text
    elif impl == "rust":
        from resiliparse._extract_rs import extract_plain_text
    else:
        raise ValueError(f"unknown --impl {impl!r}")
    _EXTRACT_FN = extract_plain_text
    _EXTRACT_KWARGS = extract_kwargs


def _extract(item):
    """Run the extractor on one doc. Returns a prediction record (no gold)."""
    index, html = item
    start = perf_counter()
    try:
        prediction = _EXTRACT_FN(html, **_EXTRACT_KWARGS)
        error = None
    except Exception as exc:  # one bad doc must not kill the run
        prediction, error = "", repr(exc)
    return {
        "index": index,
        "prediction": prediction,
        "runtime_ms": (perf_counter() - start) * 1000.0,
        "error": error,
    }


# --------------------------------------------------------------------------- #
# Helpers
# --------------------------------------------------------------------------- #
def load_split(dataset, split, datasets_dir):
    path = os.path.join(datasets_dir, dataset, f"{split}.jsonl.gz")
    if not os.path.exists(path):
        available = (sorted(os.listdir(datasets_dir))
                     if os.path.isdir(datasets_dir) else [])
        sys.exit(f"Split not found: {path}\nAvailable datasets: {available}")
    records = []
    with gzip.open(path, "rt", encoding="utf-8") as handle:
        for line in handle:
            line = line.strip()
            if line:
                records.append(json.loads(line))
    return records


def resiliparse_version():
    try:
        from importlib.metadata import version
        return version("resiliparse")
    except Exception:
        return "unknown"


def default_tag():
    version = resiliparse_version()
    try:
        sha = subprocess.check_output(
            ["git", "rev-parse", "--short", "HEAD"], cwd=BENCH_DIR,
            stderr=subprocess.DEVNULL,
        ).decode().strip()
        dirty = subprocess.call(
            ["git", "diff", "--quiet"], cwd=BENCH_DIR,
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
        )
        return f"v{version}-{sha}{'-dirty' if dirty else ''}"
    except Exception:
        return f"v{version}"


def cargo_build_release():
    """Rebuild the Rust extractor so we never score a stale binary."""
    print("[build] cargo build --release -p resiliparse-extract-rs ...", flush=True)
    start = perf_counter()
    env = dict(os.environ)
    env.setdefault("VCPKG_ROOT", os.path.expanduser("~/vcpkg"))
    cargo_bin = os.path.expanduser("~/.cargo/bin")
    if os.path.isdir(cargo_bin) and cargo_bin not in env.get("PATH", ""):
        env["PATH"] = cargo_bin + os.pathsep + env.get("PATH", "")
    cargo = os.path.join(cargo_bin, "cargo")
    if not os.path.exists(cargo):
        cargo = "cargo"
    proc = subprocess.run([cargo, "build", "--release", "-p", "resiliparse-extract-rs"],
                          cwd=REPO_DIR, env=env)
    if proc.returncode != 0:
        sys.exit("cargo build --release failed; refusing to run a stale binary")
    print(f"[build] done in {perf_counter() - start:.1f}s")


def write_jsonl(path, rows):
    with open(path, "w", encoding="utf-8") as handle:
        for row in rows:
            handle.write(json.dumps(row, ensure_ascii=False) + "\n")


def summarize(values):
    if not values:
        return {"mean": None, "median": None, "min": None, "max": None}
    return {
        "mean": mean(values),
        "median": median(values),
        "min": min(values),
        "max": max(values),
    }


def percentile(values, pct):
    if not values:
        return None
    ordered = sorted(values)
    k = min(len(ordered) - 1, max(0, round(pct / 100 * (len(ordered) - 1))))
    return ordered[k]


# --------------------------------------------------------------------------- #
# Stages
# --------------------------------------------------------------------------- #
def run_stage(records, impl, extract_kwargs, workers):
    htmls = [(i, r.get("html", "")) for i, r in enumerate(records)]
    start = perf_counter()
    if workers == 1:
        _init_worker(impl, extract_kwargs)
        results = [_extract(item) for item in htmls]
    else:
        with ProcessPoolExecutor(
            max_workers=workers, initializer=_init_worker, initargs=(impl, extract_kwargs)
        ) as pool:
            results = list(pool.map(_extract, htmls, chunksize=16))
    elapsed = perf_counter() - start

    results.sort(key=lambda r: r["index"])
    predictions = []
    for record, result in zip(records, results):
        predictions.append({
            "warc_record_id": record.get("warc_record_id"),
            "url": record.get("url"),
            "snapshot": record.get("snapshot"),
            "prediction": result["prediction"],
            "runtime_ms": result["runtime_ms"],
            "error": result["error"],
        })
    return predictions, elapsed


def score_stage(records, predictions, golds=None):
    start = perf_counter()
    rows = []
    if golds is None:
        golds = [r.get("final_output", "") for r in records]
    for record, pred, gold in zip(records, predictions, golds):
        scores = score_pair(pred["prediction"], gold)
        rows.append({
            "warc_record_id": pred["warc_record_id"],
            "url": pred["url"],
            "snapshot": pred["snapshot"],
            "error": pred["error"],
            **scores,
        })
    return rows, perf_counter() - start


# --------------------------------------------------------------------------- #
# Main
# --------------------------------------------------------------------------- #
def main():
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--dataset", default=DEFAULT_DATASET,
                        help="dataset under the datasets dir (e.g. lpv11, general, math)")
    parser.add_argument("--split", default="dev", help="dev | test | train")
    parser.add_argument("--datasets-dir", default=DATASETS_DIR,
                        help="datasets root (default: raw-HTML variant; "
                             "use benchmark/datasets_preprocessed for the preprocessed variant)")
    parser.add_argument("--impl", choices=["cython", "rust"], default="cython",
                        help="extractor implementation (default: cython reference)")
    parser.add_argument("--tag", default=None, help="run identifier (default: v<ver>-<sha>)")
    parser.add_argument("--workers", type=int, default=os.cpu_count(),
                        help="parallel workers (1 = serial; use 1 for official timing)")
    parser.add_argument("--limit", type=int, default=None, help="only first N docs (smoke test)")
    parser.add_argument("--skip-run", action="store_true",
                        help="reuse existing predictions; only re-score")
    parser.add_argument("--allow-test", action="store_true",
                        help="required to evaluate the held-out test split")
    parser.add_argument("--gold-from-run", default=None, metavar="TAG",
                        help="score against the cached predictions of TAG instead of the "
                             "dataset gold (parity oracle: Lev similarity 1.0 = identical)")
    parser.add_argument("--out-dir", default=os.path.join(BENCH_DIR, "runs"))
    # extraction options (mirror extract_plain_text keyword args)
    parser.add_argument("--no-main-content", dest="main_content", action="store_false",
                        help="disable main content extraction (default: enabled)")
    parser.add_argument("--preserve-formatting", choices=["true", "false", "minimal_html"],
                        default="true",
                        help="basic block formatting: true | false | minimal_html")
    for opt, default_on in (("list_bullets", True), ("alt_texts", True), ("links", False),
                            ("form_fields", False), ("noscript", False), ("comments", True),
                            ("post_meta", True), ("hidden_elements", False)):
        flag = opt.replace("_", "-")
        parser.add_argument(f"--{flag}", dest=opt, action="store_true", default=None,
                            help=f"force {opt} on (extractor default: {default_on})")
        parser.add_argument(f"--no-{flag}", dest=opt, action="store_false",
                            help=f"force {opt} off")
    args = parser.parse_args()

    if args.split == "test" and not args.allow_test:
        sys.exit("Refusing to touch the held-out test split. Iterate on dev/train. "
                 "Pass --allow-test only for a deliberate, final baseline/test run.")

    extract_kwargs = {"main_content": args.main_content,
                      "preserve_formatting": {"true": True, "false": False,
                                              "minimal_html": "minimal_html"}[args.preserve_formatting]}
    for opt in ("list_bullets", "alt_texts", "links", "form_fields", "noscript",
                "comments", "post_meta", "hidden_elements"):
        val = getattr(args, opt)
        if val is not None:
            extract_kwargs[opt] = val

    if args.impl == "rust" and not args.skip_run:
        cargo_build_release()

    tag = args.tag or default_tag()
    run_dir = os.path.join(args.out_dir, tag, args.dataset)
    os.makedirs(run_dir, exist_ok=True)
    pred_path = os.path.join(run_dir, f"{args.split}.predictions.jsonl")
    metrics_path = os.path.join(run_dir, f"{args.split}.metrics.jsonl")
    summary_path = os.path.join(run_dir, f"{args.split}.summary.json")

    records = load_split(args.dataset, args.split, args.datasets_dir)
    if args.limit:
        records = records[: args.limit]

    print(f"tag={tag}  dataset={args.dataset}  split={args.split}  docs={len(records)}  "
          f"impl={args.impl}  workers={args.workers}  opts={extract_kwargs}")

    # ---- run stage ----
    if args.skip_run and os.path.exists(pred_path):
        with open(pred_path, encoding="utf-8") as handle:
            predictions = [json.loads(l) for l in handle]
        if len(predictions) != len(records):
            sys.exit(f"Cached predictions ({len(predictions)}) != docs ({len(records)}); "
                     "drop --skip-run.")
        run_elapsed = None
        print(f"[run]   reused cached predictions ({len(predictions)} docs)")
    else:
        predictions, run_elapsed = run_stage(records, args.impl, extract_kwargs, args.workers)
        write_jsonl(pred_path, predictions)
        n_err = sum(1 for p in predictions if p["error"])
        print(f"[run]   {run_elapsed:7.2f}s  "
              f"{1000*run_elapsed/len(records):6.1f} ms/doc  "
              f"{len(records)/run_elapsed:6.1f} docs/s"
              + (f"  ({n_err} errors)" if n_err else ""))

    # ---- gold source ----
    golds = None
    if args.gold_from_run:
        oracle_path = os.path.join(args.out_dir, args.gold_from_run, args.dataset,
                                   f"{args.split}.predictions.jsonl")
        if not os.path.exists(oracle_path):
            sys.exit(f"--gold-from-run: no cached predictions at {oracle_path}")
        with open(oracle_path, encoding="utf-8") as handle:
            oracle = [json.loads(l) for l in handle]
        if len(oracle) < len(records):
            sys.exit(f"--gold-from-run: oracle has {len(oracle)} docs < {len(records)}")
        golds = [o["prediction"] for o in oracle[: len(records)]]
        print(f"[score] parity mode: gold = predictions of tag {args.gold_from_run}")

    # ---- score stage ----
    metric_rows, score_elapsed = score_stage(records, predictions, golds)
    write_jsonl(metrics_path, metric_rows)
    print(f"[score] {score_elapsed:7.2f}s  "
          f"{1000*score_elapsed/len(records):6.1f} ms/doc")

    # ---- aggregate ----
    runtimes = [p["runtime_ms"] for p in predictions if p.get("runtime_ms") is not None]
    summary = {
        "tag": tag,
        "dataset": args.dataset,
        "split": args.split,
        "n_docs": len(records),
        "config": {
            "resiliparse_version": resiliparse_version(),
            "impl": args.impl,
            "extract_kwargs": extract_kwargs,
            "paragraph_separator": PARAGRAPH_SEPARATOR,
            "workers": args.workers,
            "datasets_dir": args.datasets_dir,
            "gold_from_run": args.gold_from_run,
        },
        "timing": {
            "run_seconds": run_elapsed,
            "score_seconds": score_elapsed,
            "run_ms_per_doc": (1000 * run_elapsed / len(records)) if run_elapsed else None,
            "doc_ms_mean": mean(runtimes) if runtimes else None,
            "doc_ms_p50": percentile(runtimes, 50),
            "doc_ms_p95": percentile(runtimes, 95),
            "doc_ms_max": max(runtimes) if runtimes else None,
            "docs_per_second": (len(records) / run_elapsed) if run_elapsed else None,
        },
        "metrics": {
            "rougeL_f": summarize([r["rougeL_f"] for r in metric_rows]),
            "rougeL_p": summarize([r["rougeL_p"] for r in metric_rows]),
            "rougeL_r": summarize([r["rougeL_r"] for r in metric_rows]),
            "lev_distance": summarize([r["lev_distance"] for r in metric_rows]),
            "lev_similarity": summarize([r["lev_similarity"] for r in metric_rows]),
        },
    }
    with open(summary_path, "w", encoding="utf-8") as handle:
        json.dump(summary, handle, indent=2)

    m = summary["metrics"]
    t = summary["timing"]
    print(f"\n=== {tag} / {args.dataset} / {args.split} ({len(records)} docs) ===")
    print(f"  ROUGE-L  F1   mean {m['rougeL_f']['mean']:.4f}   median {m['rougeL_f']['median']:.4f}")
    print(f"           P/R  mean {m['rougeL_p']['mean']:.4f} / {m['rougeL_r']['mean']:.4f}")
    print(f"  Levenshtein   mean {m['lev_distance']['mean']:.1f} chars   "
          f"similarity mean {m['lev_similarity']['mean']:.4f}")
    if t["doc_ms_mean"] is not None:
        print(f"  Speed         mean {t['doc_ms_mean']:.2f} ms/doc   "
              f"p50 {t['doc_ms_p50']:.2f}   p95 {t['doc_ms_p95']:.2f}   "
              + (f"{t['docs_per_second']:.1f} docs/s" if t["docs_per_second"] else ""))
    print(f"\nwrote:\n  {pred_path}\n  {metrics_path}\n  {summary_path}")


if __name__ == "__main__":
    main()
