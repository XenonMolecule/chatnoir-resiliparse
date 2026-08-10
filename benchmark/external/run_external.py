"""Run OUR resiliparse (markdown, main_content) over the small-rephraser
external benchmarks and score with their token_f1 (4-gram shingle) metric.
Usage: run_external.py [zyte|trafilatura|marin|webmainbench] [tag]"""
import json, os, sys, glob, re
sys.path.insert(0, os.path.expanduser('~/Documents/School/Stanford/Research/small-rephraser'))
sys.path.insert(0, 'resiliparse-py')
from dev.evaluation.token_f1 import string_shingle_matching, levenshtein_sim
from resiliparse._extract_rs import extract_plain_text
S = os.path.expanduser('~/Documents/School/Stanford/Research/small-rephraser')

def ours(html):
    try:
        return extract_plain_text(html, preserve_formatting='markdown', main_content=True)
    except BaseException:
        return ''

def agg(rows):
    import statistics
    P = statistics.mean(r[0] for r in rows); R = statistics.mean(r[1] for r in rows)
    f1 = 2*P*R/(P+R) if P+R else 0.0
    return {'f1': round(f1,4), 'precision': round(P,4), 'recall': round(R,4), 'n': len(rows)}

def prf(gold, pred):
    tp, fp, fn = string_shingle_matching(gold, pred, 4)
    p = tp/(tp+fp) if tp+fp else (1.0 if not gold else 0.0)
    r = tp/(tp+fn) if tp+fn else 1.0
    return p, r

which, tag = sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else 'ours'
outdir = f'benchmark/runs/ext-{tag}/{which}'
os.makedirs(outdir, exist_ok=True)
rows, preds = [], []
if which == 'zyte':
    gt = json.load(open(f'{S}/submodules/article-extraction-benchmark/ground-truth.json'))
    for k, v in gt.items():
        html = open(f'{S}/static/extraction_benchmarks/zyte/html/{k}.html', encoding='utf-8', errors='replace').read()
        pred = ours(html)
        rows.append(prf(v['articleBody'], pred))
        preds.append({'id': k, 'prediction': pred})
elif which == 'trafilatura':
    # upstream methodology: `with` strings must appear, `without` must not
    man = json.load(open(f'{S}/static/extraction_benchmarks/trafilatura/manifest.json'))
    TP = FP = FN = TN = 0
    n = 0
    for k, v in man.items():
        f = f"{S}/static/extraction_benchmarks/trafilatura/html/{k}.html"
        if not os.path.exists(f): continue
        pred = ours(open(f, encoding='utf-8', errors='replace').read())
        n += 1
        flat = ' '.join(pred.split())
        for w in v.get('with', []):
            if ' '.join(w.split()) in flat: TP += 1
            else: FN += 1
        for w in v.get('without', []):
            if ' '.join(w.split()) in flat: FP += 1
            else: TN += 1
        preds.append({'id': k, 'prediction': pred})
    P_ = TP/(TP+FP) if TP+FP else 0; R_ = TP/(TP+FN) if TP+FN else 0
    res = {'f1': round(2*P_*R_/(P_+R_), 4) if P_+R_ else 0, 'precision': round(P_,4),
           'recall': round(R_,4), 'accuracy': round((TP+TN)/max(TP+TN+FP+FN,1),4), 'n': n}
    json.dump(res, open(f'{outdir}/summary.json','w'), indent=1)
    with open(f'{outdir}/predictions.jsonl','w') as fh:
        for pr in preds: fh.write(json.dumps(pr, ensure_ascii=False) + '\n')
    print(which, json.dumps(res)); sys.exit(0)
elif which == 'marin':
    root = f'{S}/static/warcs/marin_devset_1934_html'
    lev = []
    for line in open(f'{root}/devset.jsonl'):
        m = json.loads(line)
        if not m.get('gold') or not m.get('gold_path'): continue
        gp = os.path.join(root, m['gold_path'])
        if not os.path.exists(gp): continue
        gold = open(gp, encoding='utf-8', errors='replace').read()
        html = open(os.path.join(root, m['html_file']), encoding='utf-8', errors='replace').read()
        pred = ours(html)
        rows.append(prf(gold, pred))
        lev.append(levenshtein_sim(gold, pred))
        preds.append({'id': m['hid'], 'prediction': pred})
    extra = {'lev_sim_mean': round(sum(lev)/max(len(lev),1), 4)}
elif which == 'webmainbench':
    # deterministic md5(track_id) split; language filter en; dev only
    import hashlib
    lev = []
    for line in open(f'{S}/static/webmainbench/WebMainBench_545.jsonl'):
        m = json.loads(line)
        if (m.get('meta') or {}).get('language') != 'en': continue
        tid = str(m.get('track_id'))
        if int(hashlib.md5(tid.encode()).hexdigest(), 16) % 2 != 0: continue  # dev
        html = m.get('html') or ''
        gold = m.get('groundtruth_content') or ''
        if not html or not gold: continue
        pred = ours(html)
        rows.append(prf(gold, pred))
        lev.append(levenshtein_sim(gold, pred))
        preds.append({'id': tid, 'prediction': pred})
    extra = {'lev_sim_mean': round(sum(lev)/max(len(lev),1), 4)}
res = agg(rows)
try: res.update(extra)
except NameError: pass
json.dump(res, open(f'{outdir}/summary.json','w'), indent=1)
with open(f'{outdir}/predictions.jsonl','w') as f:
    for p in preds: f.write(json.dumps(p, ensure_ascii=False) + '\n')
print(which, json.dumps(res))
