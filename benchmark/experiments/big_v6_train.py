#!/usr/bin/env python
"""v4 features (page_forms/articles/comment_cls) extraction + n120d6 (cycle 0054)."""
import gzip, json, re, sys, os, gc
from collections import Counter
import numpy as np
from resiliparse._extract_rs import block_features

SRC = "/Users/michaelryan/Documents/School/Stanford/Research/jusText/benchmark/datasets_rawhtml/lpv11/big_train.jsonl.gz"
OUT = "benchmark/experiments/big_v6_cache"
os.makedirs(OUT, exist_ok=True)
TOK = re.compile(r"\w+")
CHUNK = 5000
STOP = 100000

def doc_rows(r):
    gold_toks = Counter(t.lower() for t in TOK.findall(r["final_output"]))
    try:
        feats = block_features(r["html"])
    except BaseException:
        return []
    rows = []
    for l in feats.strip().split("\n"):
        if not l: continue
        try: b = json.loads(l)
        except Exception: continue
        btoks = Counter(t.lower() for t in TOK.findall(b["text"]))
        nb = sum(btoks.values())
        if nb < 3: continue
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
            b["prev_len"], b["next_len"],
            np.log1p(b["page_forms"]), np.log1p(b["page_articles"]),
            np.log1p(b["page_comment_cls"]),
            b["page_nav_share"], b["page_generator"], np.log1p(b["page_n_blocks"]),
            b["block_pos"], b["dist_center"],
            *b["wb"],
        ], contained/nb))
    return rows

existing = {f for f in os.listdir(OUT) if f.startswith("chunk_")}
X, y, groups = [], [], []
with gzip.open(SRC, "rt") as f:
    for i, line in enumerate(f):
        if i >= STOP: break
        cid = f"chunk_{(i//CHUNK)+1:03d}.npz"
        if cid in existing: continue
        r = json.loads(line)
        for f_, l_ in doc_rows(r):
            X.append(f_); y.append(l_); groups.append(i)
        if (i + 1) % CHUNK == 0:
            np.savez_compressed(f"{OUT}/{cid}", X=np.array(X, dtype=np.float32),
                                y=np.array(y, dtype=np.float32), groups=np.array(groups))
            print(f"chunk {cid}: {len(X)} blocks", flush=True)
            X, y, groups = [], [], []
print("extraction done", flush=True)

# lean train
rng = np.random.default_rng(4)
TARGET = 2_500_000
files = sorted(f for f in os.listdir(OUT) if f.startswith("chunk_"))
tot = 0; sizes = []
for fn in files:
    d = np.load(f"{OUT}/{fn}"); n = d["y"].shape[0]; sizes.append(n); tot += n; del d
frac = min(1.0, TARGET / (tot * 0.9))
print(f"total={tot} frac={frac:.3f}", flush=True)
Xtr, ytr, Xte, yte = [], [], [], []
gcut = 90000
for fn in files:
    d = np.load(f"{OUT}/{fn}")
    Xc, yc, g = d["X"], d["y"], d["groups"]
    tr = np.flatnonzero(g < gcut); te = np.flatnonzero(g >= gcut)
    take = rng.random(len(tr)) < frac
    Xtr.append(Xc[tr[take]]); ytr.append(yc[tr[take]])
    if len(te):
        keep = rng.random(len(te)) < 0.25
        Xte.append(Xc[te[keep]]); yte.append(yc[te[keep]])
    del d, Xc, yc, g; gc.collect()
Xtr = np.concatenate(Xtr); ytr = np.concatenate(ytr)
Xte = np.concatenate(Xte); yte = np.concatenate(yte); gc.collect()
print(f"train={len(Xtr)} test={len(Xte)} dims={Xtr.shape[1]}", flush=True)
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import roc_auc_score
m = GradientBoostingRegressor(n_estimators=120, max_depth=6, learning_rate=0.1,
                              subsample=0.7, verbose=1)
m.fit(Xtr, ytr)
pred = m.predict(Xte)
yb = (yte > 0.5).astype(int)
print(f"GBR v6 n120d6: AUC {roc_auc_score(yb, pred):.4f}", flush=True)
import joblib
joblib.dump(m, "benchmark/experiments/gbm_v6big_n120d6.joblib")
print("saved", flush=True)
