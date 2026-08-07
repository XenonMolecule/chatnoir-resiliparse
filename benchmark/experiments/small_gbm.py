#!/usr/bin/env python
import gzip, json, re, os
from collections import Counter
import numpy as np
from resiliparse._extract_rs import block_features

CACHE = "benchmark/experiments/block_features_cache.npz"
if os.path.exists(CACHE):
    d = np.load(CACHE)
    X, y, groups = d["X"], d["y"], d["groups"]
    docs = int(groups.max()) + 1
else:
    TOK = re.compile(r"\w+")
    X, y, groups = [], [], []
    docs = 0
    with gzip.open("benchmark/datasets_rawhtml/lpv11/train.jsonl.gz", "rt") as f:
        for line in f:
            if docs >= 9999: break
            r = json.loads(line)
            gold_toks = Counter(t.lower() for t in TOK.findall(r["final_output"]))
            try: feats = block_features(r["html"])
            except BaseException: continue
            any_row = False
            for l in feats.strip().split("\n"):
                if not l: continue
                try: b = json.loads(l)
                except Exception: continue
                btoks = Counter(t.lower() for t in TOK.findall(b["text"]))
                nb = sum(btoks.values())
                if nb < 3: continue
                contained = sum(min(c, gold_toks.get(t, 0)) for t, c in btoks.items())
                tl = b["text_len"]
                X.append([b["tag"], b["depth"], np.log1p(tl), b["link_len"]/max(tl,1),
                          b["n_links"], b["page_ld"], tl/max(b["page_text"],1),
                          b["punct"], b["digit"], b["upper"], b["avgw"],
                          b["nav"], b["footer"], b["header"], b["sidebar"], b["social"],
                          b["article"], b["chrome"], b["byline"], b["widget"],
                          b["recommended"], b["comments"]])
                y.append(contained/nb); any_row = True
            if any_row:
                groups.extend([docs]*(len(X)-len(groups))); docs += 1
    X = np.array(X, dtype=np.float32); y = np.array(y); groups = np.array(groups)
    np.savez_compressed(CACHE, X=X, y=y, groups=groups)
print(f"docs={docs} blocks={len(X)}", flush=True)
cut = int(docs*0.8); tr = groups < cut; te = ~tr
yb = (y > 0.5).astype(int)
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import roc_auc_score
import joblib
for n, d in ((30,5),(60,5),(60,6)):
    m = GradientBoostingRegressor(n_estimators=n, max_depth=d, learning_rate=0.15, subsample=0.7)
    m.fit(X[tr], y[tr])
    pred = m.predict(X[te])
    line = f"gbm n{n} d{d}: AUC {roc_auc_score(yb[te], pred):.4f}"
    for thr in (0.05, 0.1, 0.15):
        v = pred < thr
        if v.sum(): line += f" | veto@{thr}: {v.mean():.2%} cov, {yb[te][v].mean():.2%} fv"
    for thr in (0.9,):
        k = pred > thr
        if k.sum(): line += f" | keep@{thr}: {k.mean():.2%} cov, {(yb[te][k]==0).mean():.2%} fk"
    print(line, flush=True)
    joblib.dump(m, f"benchmark/experiments/gbm_n{n}d{d}.joblib")
