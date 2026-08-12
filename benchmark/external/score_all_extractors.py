"""Score every available extractor on each external benchmark with ONE scorer.

Baseline predictions come from the companion research repo (stored outputs of
dripper / marin_v1.1 / upstream resiliparse / upstream trafilatura); our own
predictions are read from benchmark/runs/ext-<tag>/. Everything is then scored
here, so no number in the output mixes methodologies.

Usage: python benchmark/external/score_all_extractors.py [our_tag]
Writes benchmark/runs/ext-<tag>/cross_extractor.json
"""
import gzip
import json
import os
import statistics
import sys

sys.path.insert(0, os.path.expanduser('~/Documents/School/Stanford/Research/small-rephraser'))
from dev.evaluation.token_f1 import string_shingle_matching  # noqa: E402

S = os.path.expanduser('~/Documents/School/Stanford/Research/small-rephraser')
TAG = sys.argv[1] if len(sys.argv) > 1 else '0177'
OURS = f'benchmark/runs/ext-{TAG}'

# label -> stored prediction file, keyed by benchmark
BASELINES = {
    'zyte': {
        'dripper': 'outputs/extraction_benchmarks/zyte/dripper/predictions.jsonl',
        'marin v1.1': 'outputs/extraction_benchmarks/zyte/marin_v11/predictions.jsonl',
        'resiliparse (upstream)': 'outputs/extraction_benchmarks/zyte/baseline_resiliparse/predictions.jsonl.gz',
        'trafilatura (upstream)': 'outputs/extraction_benchmarks/zyte/baseline_trafilatura/predictions.jsonl.gz',
    },
    'trafilatura': {
        'dripper': 'outputs/extraction_benchmarks/trafilatura/dripper/predictions.jsonl',
        'marin v1.1': 'outputs/extraction_benchmarks/trafilatura/marin_v11/predictions.jsonl',
        'resiliparse (upstream)': 'outputs/extraction_benchmarks/trafilatura/baseline_resiliparse/predictions.jsonl.gz',
        'trafilatura (upstream)': 'outputs/extraction_benchmarks/trafilatura/baseline_trafilatura/predictions.jsonl.gz',
    },
    'marin': {
        'dripper': 'outputs/marin_devset/dripper_devset_predictions.jsonl',
        'resiliparse (upstream)': 'outputs/marin_devset/baselines/resiliparse.json',
        'trafilatura (upstream)': 'outputs/marin_devset/baselines/trafilatura.json',
    },
    'webmainbench': {
        'dripper': 'outputs/webmainbench/dripper_predictions.jsonl',
    },
}


def _open(path):
    return gzip.open(path, 'rt') if path.endswith('.gz') else open(path)


def load_preds(path):
    """-> {doc_id: text}; handles jsonl (metadata.record_index | id) and dict json."""
    full = os.path.join(S, path)
    if not os.path.exists(full):
        return None
    if full.endswith('.json'):
        d = json.load(open(full))
        if isinstance(d, dict):
            return {str(k): (v if isinstance(v, str) else (v or {}).get('text', '')) for k, v in d.items()}
        return {str(r.get('hid') or r.get('id')): r.get('text', '') for r in d}
    out = {}
    with _open(full) as fh:
        for line in fh:
            r = json.loads(line)
            key = (r.get('metadata') or {}).get('record_index') or r.get('id') or r.get('hid')
            key = str(key)
            # marin baselines key on the full WARC record name
            # (record_00000_121space_com_ac8df90baf); the devset keys on the
            # trailing hid. Store both so either join works.
            text = r.get('text') or r.get('prediction') or ''
            out[key] = text
            if key.startswith('record_') and '_' in key:
                out[key.rsplit('_', 1)[-1]] = text
    return out


def load_ours(which):
    p = f'{OURS}/{which}/predictions.jsonl'
    if not os.path.exists(p):
        return None
    return {str(json.loads(l)['id']): json.loads(l)['prediction'] for l in open(p)}


def prf_mean(pairs):
    """Benchmark-standard aggregate: mean P, mean R, F1 of the means."""
    ps, rs = [], []
    for gold, pred in pairs:
        tp, fp, fn = string_shingle_matching(gold, pred)
        ps.append(tp / (tp + fp) if tp + fp else 0.0)
        rs.append(tp / (tp + fn) if tp + fn else 0.0)
    P, R = statistics.mean(ps), statistics.mean(rs)
    return {'f1': round(2 * P * R / (P + R), 4) if P + R else 0.0,
            'precision': round(P, 4), 'recall': round(R, 4), 'n': len(ps)}


def score_zyte(preds):
    gt = json.load(open(f'{S}/submodules/article-extraction-benchmark/ground-truth.json'))
    pairs = [(v['articleBody'], preds.get(k, '')) for k, v in gt.items() if k in preds]
    return prf_mean(pairs) if pairs else None


def score_marin(preds):
    root = f'{S}/static/warcs/marin_devset_1934_html'
    pairs = []
    for line in open(f'{root}/devset.jsonl'):
        m = json.loads(line)
        if not m.get('gold') or not m.get('gold_path'):
            continue
        gp = os.path.join(root, m['gold_path'])
        if not os.path.exists(gp) or m['hid'] not in preds:
            continue
        pairs.append((open(gp).read(), preds[m['hid']]))
    return prf_mean(pairs) if pairs else None


def score_trafilatura(preds):
    """Upstream methodology: `with` strings must appear, `without` must not."""
    man = json.load(open(f'{S}/static/extraction_benchmarks/trafilatura/manifest.json'))
    TP = FP = FN = TN = n = 0
    for k, v in man.items():
        if k not in preds:
            continue
        n += 1
        flat = ' '.join(preds[k].split())
        for w in v.get('with', []):
            if ' '.join(w.split()) in flat:
                TP += 1
            else:
                FN += 1
        for w in v.get('without', []):
            if ' '.join(w.split()) in flat:
                FP += 1
            else:
                TN += 1
    if not n:
        return None
    P = TP / (TP + FP) if TP + FP else 0
    R = TP / (TP + FN) if TP + FN else 0
    return {'f1': round(2 * P * R / (P + R), 4) if P + R else 0, 'precision': round(P, 4),
            'recall': round(R, 4), 'accuracy': round((TP + TN) / max(TP + TN + FP + FN, 1), 4), 'n': n}


def score_webmainbench(preds):
    """WMB en/dev split (md5(track_id) % 2 == 0), token-F1 against groundtruth."""
    import hashlib
    pairs = []
    for line in open(f'{S}/static/webmainbench/WebMainBench_545.jsonl'):
        m = json.loads(line)
        if (m.get('meta') or {}).get('language') != 'en':
            continue
        tid = str(m.get('track_id'))
        if int(hashlib.md5(tid.encode()).hexdigest(), 16) % 2 != 0:
            continue
        if tid not in preds:
            continue
        pairs.append((m.get('groundtruth_content') or '', preds[tid]))
    return prf_mean(pairs) if pairs else None


SCORERS = {'zyte': score_zyte, 'marin': score_marin, 'trafilatura': score_trafilatura,
           'webmainbench': score_webmainbench}

results = {}
# locally generated baseline predictions (live runs, not stored in the data repo)
LOCAL = {
    'webmainbench': {
        'resiliparse (upstream)': f'{OURS}/webmainbench/upstream_resiliparse.predictions.jsonl',
        'trafilatura (upstream)': f'{OURS}/webmainbench/upstream_trafilatura.predictions.jsonl',
    },
}

for which, scorer in SCORERS.items():
    results[which] = {}
    ours = load_ours(which)
    if ours:
        r = scorer(ours)
        if r:
            results[which]['ours (this fork)'] = r
    for label, path in LOCAL.get(which, {}).items():
        if not os.path.exists(path):
            continue
        preds = {str(json.loads(l)['id']): json.loads(l)['prediction'] for l in open(path)}
        r = scorer(preds)
        if r:
            results[which][label] = r
    for label, path in BASELINES.get(which, {}).items():
        preds = load_preds(path)
        if not preds:
            print(f'  [skip] {which}/{label}: no stored predictions at {path}', file=sys.stderr)
            continue
        r = scorer(preds)
        if r:
            results[which][label] = r

for which, table in results.items():
    print(f'\n=== {which}')
    for label, r in sorted(table.items(), key=lambda kv: -kv[1]['f1']):
        extra = f" acc={r['accuracy']}" if 'accuracy' in r else ''
        print(f"  {label:26s} F1={r['f1']:.4f} P={r['precision']:.4f} R={r['recall']:.4f}{extra} n={r['n']}")

os.makedirs(OURS, exist_ok=True)
json.dump(results, open(f'{OURS}/cross_extractor.json', 'w'), indent=1)
print(f'\nwrote {OURS}/cross_extractor.json')
