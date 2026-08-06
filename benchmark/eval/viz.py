#!/usr/bin/env python3
"""Text-only visualization / analysis tools for a resiliparse eval run.

Built for fast pattern-hunting on the **dev** and **train** splits (test is held
out -- the loader refuses it). Every command takes optional --tag (default: latest
run) and --split (default: dev).

Commands
--------
    overview                      one-stop dashboard: distribution, tags, breakdowns
    rank   [--worst|--best|--median] [--metric M] [--n N] [--tag-filter T] [--flag F]
    show   <doc>   [--html]       full metadata + gold + prediction for one doc
    diff   <doc>   [--char] [--only added|dropped]   word/char diff vs gold
    search [--dropped RE] [--added RE] [--gold RE] [--pred RE] [--empty] [--flag F] [--tag T]
    breakdown <by>                mean/median score grouped by snapshot|has_code|tag|...
    tags                          failure-tag distribution with example docs

<doc> is a warc_record_id or any unique prefix.

Examples
--------
    python benchmark/eval/viz.py overview
    python benchmark/eval/viz.py rank --worst --n 15 --flag has_code
    python benchmark/eval/viz.py diff 025fbdd9 --only dropped
    python benchmark/eval/viz.py search --dropped 'def |class |import ' --flag has_code
    python benchmark/eval/viz.py breakdown snapshot
"""

import argparse
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
from analysis import compare, list_datasets, list_runs, load_run  # noqa: E402

# --------------------------------------------------------------------------- #
# Rendering primitives
# --------------------------------------------------------------------------- #
RULE = "-" * 88


def trunc(text, width):
    text = text.replace("\n", "⏎")  # show newlines as a glyph in one-liners
    return text if len(text) <= width else text[: width - 1] + "…"


def doc_key(doc):
    """Short addressable handle: 8-char id, or #index when the id is null."""
    return doc.id[:8] if doc.id else f"#{doc.index}"


def flags_str(doc):
    return "".join(c if f else "·" for c, f in
                   (("T", doc.has_table), ("C", doc.has_code), ("L", doc.has_list)))


def bar(fraction, width=30):
    filled = int(round(fraction * width))
    return "█" * filled + "·" * (width - filled)


def row(doc, gold_snippet=28):
    return (f"{doc.score:5.3f}  lev{doc.lev:>6}  r{doc.length_ratio:4.2f}  "
            f"k{doc.n_kept:>3}  {flags_str(doc)}  {doc.snapshot:<15}  "
            f"{doc.primary_tag:<13}  {trunc(doc.url, 38):<38}  "
            f"«{trunc(doc.gold, gold_snippet)}»")


def header(run):
    return f"{run.tag}  /  {run.dataset}/{run.split}  ({len(run)} docs)"


# --------------------------------------------------------------------------- #
# Commands
# --------------------------------------------------------------------------- #
def cmd_overview(run, args):
    vals = run.values("rougeL_f")
    n = len(vals)
    print(RULE)
    print(header(run))
    print(RULE)
    print("ROUGE-L F1   mean {:.4f}   median {:.4f}      "
          "Lev-sim   mean {:.4f}".format(
              sum(vals) / n, sorted(vals)[n // 2],
              sum(run.values("lev_similarity")) / n))
    print("ROUGE-L P/R  {:.4f} / {:.4f}                  "
          "Lev-dist  mean {:.0f} chars".format(
              sum(run.values("rougeL_p")) / n, sum(run.values("rougeL_r")) / n,
              sum(run.values("lev_distance")) / n))

    print("\nROUGE-L F1 distribution")
    edges = [i / 10 for i in range(11)]
    for lo, hi in zip(edges, edges[1:]):
        c = sum(1 for v in vals if (lo <= v < hi) or (hi == 1.0 and v == 1.0))
        print(f"  [{lo:.1f},{hi:.1f}) {bar(c / n)} {c:>4}")

    print("\npercentiles  " + "  ".join(
        f"p{p}={v:.3f}" for p, v in run.percentiles().items()))

    print("\nfailure tags (primary)")
    for tag, c in run.tag_counts().items():
        print(f"  {tag:<13} {bar(c / n, 24)} {c:>4} ({100*c/n:4.1f}%)")

    for by in ("has_code", "has_table", "has_list"):
        bd = run.breakdown(by)
        parts = "   ".join(f"{k}: {v['mean']:.3f} (n={v['n']})" for k, v in bd.items())
        print(f"\nby {by:<10} {parts}")

    snap = run.breakdown("snapshot")
    items = list(snap.items())
    print("\nworst snapshots:  " + "  ".join(
        f"{k} {v['mean']:.3f}(n{v['n']})" for k, v in items[:4]))
    print("best  snapshots:  " + "  ".join(
        f"{k} {v['mean']:.3f}(n{v['n']})" for k, v in items[-4:]))
    print(RULE)
    print("drill in:  viz.py rank --worst   |   viz.py diff <id>   |   viz.py tags")


def cmd_rank(run, args):
    metric = args.metric
    if args.median:
        docs, label = run.around_median(args.n, metric), "around median"
    elif args.best:
        docs, label = run.best(args.n, metric), "best"
    else:
        docs, label = run.worst(args.n, metric), "worst"
    if args.flag:
        docs = [d for d in docs if getattr(d, args.flag)]
    if args.tag_filter:
        docs = [d for d in docs if args.tag_filter.upper() in d.tags]
    print(f"{header(run)}   --   {label} {len(docs)} by {metric}")
    print(f"{'id':<10}  {'score':>5}  {'lev':>9}  ratio  kept  TCL  "
          f"{'snapshot':<15}  {'tag':<13}  url / gold")
    print(RULE)
    for d in docs:
        print(f"{doc_key(d):<10}  {row(d)}")


def cmd_show(run, args):
    d = run.get(args.doc)
    print(RULE)
    print(f"id={d.id}\nurl={d.url}\nsnapshot={d.snapshot}  split={d.split}  "
          f"flags=[T{d.has_table} C{d.has_code} L{d.has_list}]")
    print(f"tags={sorted(d.tags)}  primary={d.primary_tag}")
    print(f"ROUGE-L F1={d.score:.4f} (P{d.metrics.get('rougeL_p',0):.3f}/"
          f"R{d.metrics.get('rougeL_r',0):.3f})  lev={d.lev}  "
          f"chars pred/gold={d.pred_chars}/{d.gold_chars} (ratio {d.length_ratio:.2f})")
    if d.error:
        print(f"ERROR: {d.error}")
    if args.html:
        print(RULE + "\nHTML (first 2000 chars)\n" + RULE)
        print(d.html[:2000])
    print(RULE + "\nGOLD\n" + RULE)
    print(d.gold)
    print(RULE + "\nPREDICTION (resiliparse)\n" + RULE)
    print(d.pred or "(empty)")
    print(RULE)


def cmd_diff(run, args):
    d = run.get(args.doc)
    print(f"{header(run)}   diff {doc_key(d)}  F1={d.score:.3f}  lev={d.lev}  "
          f"[- dropped by resiliparse -]  {{+ added by resiliparse +}}")
    print(RULE)
    chunks = []
    for op, g, p in d.diff_ops("char" if args.char else "word"):
        if op == "equal":
            words = g.split()
            if len(words) > 12:
                g = f"{' '.join(words[:5])} …({len(words)} eq)… {' '.join(words[-5:])}"
            chunks.append(g)
        elif op == "delete" and args.only != "added":
            chunks.append(f"[-{g}-]")
        elif op == "insert" and args.only != "dropped":
            chunks.append(f"{{+{p}+}}")
        elif op == "replace":
            if args.only != "added":
                chunks.append(f"[-{g}-]")
            if args.only != "dropped":
                chunks.append(f"{{+{p}+}}")
    print(" ".join(c for c in chunks if c))
    print(RULE)


def cmd_search(run, args):
    docs = run.find(
        gold=args.gold, pred=args.pred, diff_added=args.added, diff_dropped=args.dropped,
        tag=args.tag, pred_empty=True if args.empty else None,
        has_code=True if args.flag == "has_code" else None,
        has_table=True if args.flag == "has_table" else None,
        has_list=True if args.flag == "has_list" else None,
        snapshot=args.snapshot)
    docs = sorted(docs, key=lambda d: d.score)
    print(f"{header(run)}   --   {len(docs)} matches "
          f"(mean F1 {sum(d.score for d in docs)/len(docs):.3f})" if docs else
          f"{header(run)}   --   0 matches")
    print(RULE)
    for d in docs[: args.n]:
        print(f"{doc_key(d):<10}  {row(d)}")
    if len(docs) > args.n:
        print(f"… and {len(docs) - args.n} more (raise --n)")


def cmd_breakdown(run, args):
    bd = run.breakdown(args.by, args.metric)
    print(f"{header(run)}   --   {args.metric} by {args.by}")
    print(RULE)
    for k, v in bd.items():
        print(f"  {str(k):<18} {bar(v['mean'], 24)} mean {v['mean']:.3f}  "
              f"median {v['median']:.3f}  (n={v['n']})")


def cmd_runs(args):
    runs = list_runs()
    datasets = list_datasets()
    print(f"datasets available: {datasets or '(none)'}")
    if not runs:
        print("No cached runs. Run run_eval.py first.")
        return
    print("\ncached runs (tag -> dataset -> splits):")
    for tag, by_ds in runs.items():
        print(f"  {tag}")
        for ds, splits in by_ds.items():
            print(f"      {ds:<14} {', '.join(splits)}")
    print("\ncompare two:  viz.py --dataset general compare <tagA> <tagB> [--split dev]")


def cmd_compare(args):
    run_a = load_run(args.a, args.split, args.dataset)
    run_b = load_run(args.b, args.split, args.dataset)
    rows = compare(run_a, run_b, args.metric)
    n = len(rows)
    if not n:
        print("No overlapping docs between the two runs.")
        return
    mean_a = sum(r["a"] for r in rows) / n
    mean_b = sum(r["b"] for r in rows) / n
    improved = [r for r in rows if r["delta"] > 1e-9]
    regressed = [r for r in rows if r["delta"] < -1e-9]
    print(RULE)
    print(f"COMPARE  {args.metric}   A={args.a or '(latest)'}   B={args.b}   "
          f"{args.dataset}/{args.split}  ({n} docs)")
    print(RULE)
    print(f"mean  A {mean_a:.4f}  ->  B {mean_b:.4f}   "
          f"Δ {mean_b - mean_a:+.4f}")
    print(f"docs  improved {len(improved)}   regressed {len(regressed)}   "
          f"unchanged {n - len(improved) - len(regressed)}")
    print(f"\nTOP {args.n} REGRESSIONS (B worse than A)")
    print(f"{'id':<10} {'A':>6} {'B':>6} {'Δ':>7}  url")
    for r in rows[: args.n]:
        if r["delta"] < 0:
            print(f"{r['id'][:8]:<10} {r['a']:6.3f} {r['b']:6.3f} {r['delta']:+7.3f}  "
                  f"{trunc(r['url'], 50)}")
    print(f"\nTOP {args.n} IMPROVEMENTS (B better than A)")
    print(f"{'id':<10} {'A':>6} {'B':>6} {'Δ':>7}  url")
    for r in reversed(rows[-args.n:]):
        if r["delta"] > 0:
            print(f"{r['id'][:8]:<10} {r['a']:6.3f} {r['b']:6.3f} {r['delta']:+7.3f}  "
                  f"{trunc(r['url'], 50)}")
    print(RULE)
    print("inspect a mover:  viz.py diff <id> --tag " + (args.b or "<tagB>"))


def cmd_tags(run, args):
    print(f"{header(run)}   --   failure tags")
    print(RULE)
    for tag, c in run.tag_counts().items():
        print(f"\n{tag}  ({c} docs)")
        for d in run.find(tag=tag)[:3]:
            print(f"    {doc_key(d)}  F1={d.score:.3f}  {trunc(d.url,50)}")


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #
def main():
    p = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--tag", default=None)
    p.add_argument("--dataset", default="lpv11",
                   help="benchmark dataset (lpv11, general, math, code, ...)")
    p.add_argument("--split", default="dev")
    sub = p.add_subparsers(dest="cmd", required=True)

    sp = sub.add_parser("overview"); sp.set_defaults(func=cmd_overview)

    sp = sub.add_parser("rank"); sp.set_defaults(func=cmd_rank)
    sp.add_argument("--worst", action="store_true")
    sp.add_argument("--best", action="store_true")
    sp.add_argument("--median", action="store_true")
    sp.add_argument("--metric", default="rougeL_f")
    sp.add_argument("--n", type=int, default=20)
    sp.add_argument("--flag", choices=["has_table", "has_code", "has_list"])
    sp.add_argument("--tag-filter", default=None)

    sp = sub.add_parser("show"); sp.set_defaults(func=cmd_show)
    sp.add_argument("doc"); sp.add_argument("--html", action="store_true")

    sp = sub.add_parser("diff"); sp.set_defaults(func=cmd_diff)
    sp.add_argument("doc")
    sp.add_argument("--char", action="store_true")
    sp.add_argument("--only", choices=["added", "dropped"], default=None)

    sp = sub.add_parser("search"); sp.set_defaults(func=cmd_search)
    sp.add_argument("--gold"); sp.add_argument("--pred")
    sp.add_argument("--added"); sp.add_argument("--dropped")
    sp.add_argument("--tag"); sp.add_argument("--snapshot")
    sp.add_argument("--empty", action="store_true")
    sp.add_argument("--flag", choices=["has_table", "has_code", "has_list"])
    sp.add_argument("--n", type=int, default=25)

    sp = sub.add_parser("breakdown"); sp.set_defaults(func=cmd_breakdown)
    sp.add_argument("by", help="snapshot | has_code | has_table | has_list | primary_tag")
    sp.add_argument("--metric", default="rougeL_f")

    sp = sub.add_parser("tags"); sp.set_defaults(func=cmd_tags)

    sp = sub.add_parser("runs"); sp.set_defaults(func=cmd_runs, _no_run=True)

    sp = sub.add_parser("compare"); sp.set_defaults(func=cmd_compare, _no_run=True)
    sp.add_argument("a", nargs="?", default=None, help="baseline tag (default: latest)")
    sp.add_argument("b", help="candidate tag to compare against the baseline")
    sp.add_argument("--metric", default="rougeL_f")
    sp.add_argument("--n", type=int, default=15)

    args = p.parse_args()
    if getattr(args, "_no_run", False):
        args.func(args)            # runs / compare manage their own loading
    else:
        args.func(load_run(args.tag, args.split, args.dataset), args)


if __name__ == "__main__":
    main()
