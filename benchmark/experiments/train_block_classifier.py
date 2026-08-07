#!/usr/bin/env python
"""Learned block classifier v0: features from Rust, labels from gold containment."""
import gzip, json, re, sys, pickle
from collections import Counter
import numpy as np
from resiliparse._extract_rs import block_features

TOK = re.compile(r"\w+")
N_DOCS = int(sys.argv[1]) if len(sys.argv) > 1 else 3000

X, y, groups = [], [], []
docs = 0
with gzip.open("benchmark/datasets_rawhtml/lpv11/train.jsonl.gz", "rt") as f:
    for line in f:
        if docs >= N_DOCS: break
        r = json.loads(line)
        gold_toks = Counter(t.lower() for t in TOK.findall(r["final_output"]))
        try:
            feats = block_features(r["html"])
        except BaseException:
            continue
        rows = []
        for l in feats.strip().split("\n"):
            if not l: continue
            try: b = json.loads(l)
            except Exception: continue
            btoks = Counter(t.lower() for t in TOK.findall(b["text"]))
            nb = sum(btoks.values())
            if nb < 3: continue
            contained = sum(min(c, gold_toks.get(t, 0)) for t, c in btoks.items())
            label = contained / nb
            tl = b["text_len"]
            rows.append(([
                b["tag"], b["depth"], np.log1p(tl),
                b["link_len"]/max(tl,1), b["n_links"], b["page_ld"],
                tl/max(b["page_text"],1),
                b["punct"], b["digit"], b["upper"], b["avgw"],
                b["nav"], b["footer"], b["header"], b["sidebar"], b["social"],
                b["article"], b["chrome"], b["byline"], b["widget"],
                b["recommended"], b["comments"],
            ], label))
        if not rows: continue
        for f_, l_ in rows:
            X.append(f_); y.append(l_); groups.append(docs)
        docs += 1

X = np.array(X); y = np.array(y); groups = np.array(groups)
print(f"docs={docs} blocks={len(X)}  label mean={y.mean():.3f}", flush=True)

from sklearn.ensemble import HistGradientBoostingRegressor
from sklearn.metrics import roc_auc_score
cut = int(docs * 0.8)
tr = groups < cut; te = ~tr
m = HistGradientBoostingRegressor(max_iter=200, max_depth=6, learning_rate=0.08)
m.fit(X[tr], y[tr])
pred = m.predict(X[te])
yb = (y[te] > 0.5).astype(int)
print(f"test AUC (containment>0.5): {roc_auc_score(yb, pred):.4f}", flush=True)
# veto-policy value: blocks with pred < t that are actually kept-worthy (label>0.5) = errors
for t in (0.05, 0.1, 0.15, 0.2, 0.3):
    veto = pred < t
    if veto.sum() == 0: continue
    err = (yb[veto] == 1).mean()
    cov = veto.mean()
    print(f"  veto@{t}: coverage {cov:.2%} of blocks, false-veto rate {err:.2%}", flush=True)
# keep-policy: high-pred blocks that are junk
for t in (0.8, 0.9):
    keep = pred > t
    if keep.sum() == 0: continue
    err = (yb[keep] == 0).mean()
    print(f"  keep@{t}: coverage {keep.mean():.2%}, false-keep rate {err:.2%}", flush=True)
import joblib
joblib.dump(m, "benchmark/experiments/block_classifier_v0.joblib")
print("saved model", flush=True)
