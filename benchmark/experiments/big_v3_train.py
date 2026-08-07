#!/usr/bin/env python
"""v3-feature extraction over big_train + GBR n60d6 training (cycle 0047)."""
import gzip, json, re, sys, os
from collections import Counter
import numpy as np
from resiliparse._extract_rs import block_features

SRC = "/Users/michaelryan/Documents/School/Stanford/Research/jusText/benchmark/datasets_rawhtml/lpv11/big_train.jsonl.gz"
OUT = "benchmark/experiments/big_v3_cache"
os.makedirs(OUT, exist_ok=True)
TOK = re.compile(r"\w+")
CHUNK = 5000
STOP = int(sys.argv[1]) if len(sys.argv) > 1 else 100000

def doc_rows(r):
    gold_toks = Counter(t.lower() for t in TOK.findall(r["final_output"]))
    try:
        feats = block_features(r["html"])
    except BaseException:
        return []
    rows = []
    for l in feats.strip().split("\n"):
        if not l:
            continue
        try:
            b = json.loads(l)
        except Exception:
            continue
        btoks = Counter(t.lower() for t in TOK.findall(b["text"]))
        nb = sum(btoks.values())
        if nb < 3:
            continue
        contained = sum(min(c, gold_toks.get(t, 0)) for t, c in btoks.items())
        tl = b["text_len"]
        rows.append(([
            b["tag"], b["depth"], np.log1p(tl),
            b["link_len"]/max(tl,1), b["n_links"], b["page_ld"],
            tl/max(b["page_text"],1),
            b["punct"], b["digit"], b["upper"], b["avgw"],
            b["nav"], b["footer"], b["header"], b["sidebar"], b["social"],
            b["article"], b["chrome"], b["byline"], b["widget"],
            b["recommended"], b["comments"],
            b["headings"], b["page_headings"], b["prev_ld"], b["next_ld"],
            b["prev_len"], b["next_len"], *b["wb"],
        ], contained/nb))
    return rows

existing = {f for f in os.listdir(OUT) if f.startswith("chunk_")}
X, y, groups = [], [], []
docs = 0
with gzip.open(SRC, "rt") as f:
    for i, line in enumerate(f):
        if i >= STOP:
            break
        cid = f"chunk_{(i//CHUNK)+1:03d}.npz"
        if cid in existing:
            continue
        r = json.loads(line)
        for f_, l_ in doc_rows(r):
            X.append(f_); y.append(l_); groups.append(i)
        docs += 1
        if (i + 1) % CHUNK == 0:
            np.savez_compressed(f"{OUT}/{cid}", X=np.array(X, dtype=np.float32),
                                y=np.array(y, dtype=np.float32), groups=np.array(groups))
            print(f"chunk {cid}: {len(X)} blocks (doc {i+1})", flush=True)
            X, y, groups = [], [], []
if X:
    np.savez_compressed(f"{OUT}/chunk_tail.npz", X=np.array(X, dtype=np.float32),
                        y=np.array(y, dtype=np.float32), groups=np.array(groups))
print("extraction done", flush=True)

# ---- train ----
Xs, ys, gs = [], [], []
for fn in sorted(os.listdir(OUT)):
    if not fn.startswith("chunk_"):
        continue
    d = np.load(f"{OUT}/{fn}")
    Xs.append(d["X"]); ys.append(d["y"]); gs.append(d["groups"])
X = np.concatenate(Xs); y = np.concatenate(ys); groups = np.concatenate(gs)
print(f"total blocks={len(X)} docs~{len(np.unique(groups))}", flush=True)

rng = np.random.default_rng(0)
cut = int(groups.max() * 0.9)
tr_mask = groups < cut
te_mask = ~tr_mask
tr_idx = np.flatnonzero(tr_mask)
if len(tr_idx) > 3_000_000:
    tr_idx = rng.choice(tr_idx, 3_000_000, replace=False)
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import roc_auc_score
m = GradientBoostingRegressor(n_estimators=60, max_depth=6, learning_rate=0.1,
                              subsample=0.7, verbose=1)
m.fit(X[tr_idx], y[tr_idx])
pred = m.predict(X[te_mask])
yb = (y[te_mask] > 0.5).astype(int)
print(f"GBR n60d6 v3 @{len(tr_idx)}: AUC {roc_auc_score(yb, pred):.4f}", flush=True)
for t in (0.1, 0.15, 0.2, 0.25, 0.3):
    veto = pred < t
    if veto.sum():
        print(f"  veto@{t}: {veto.mean():.2%} cov, {(yb[veto]==1).mean():.2%} fv", flush=True)
for t in (0.8, 0.85, 0.9):
    keep = pred > t
    if keep.sum():
        print(f"  keep@{t}: {keep.mean():.2%} cov, {(yb[keep]==0).mean():.2%} fk", flush=True)
import joblib
joblib.dump(m, "benchmark/experiments/gbm_v3big_n60d6.joblib")
print("saved gbm_v3big_n60d6.joblib", flush=True)
