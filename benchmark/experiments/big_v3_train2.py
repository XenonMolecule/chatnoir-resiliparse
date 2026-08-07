#!/usr/bin/env python
"""Round 2 lean (0053): n120d6 on ~3M subsampled rows, low peak memory."""
import os, gc, numpy as np
OUT = "benchmark/experiments/big_v3_cache"
rng = np.random.default_rng(1)
TARGET = 3_000_000
files = sorted(f for f in os.listdir(OUT) if f.startswith("chunk_"))
# estimate total rows first (cheap header reads)
tot = 0
sizes = []
for fn in files:
    d = np.load(f"{OUT}/{fn}")
    n = d["y"].shape[0]; sizes.append(n); tot += n
    del d
frac = min(1.0, TARGET / (tot * 0.9))
print(f"total={tot} sample_frac={frac:.3f}", flush=True)
Xtr, ytr, Xte, yte = [], [], [], []
gcut = None
for fn, n in zip(files, sizes):
    d = np.load(f"{OUT}/{fn}")
    X, y, g = d["X"], d["y"], d["groups"]
    if gcut is None:
        gmax = 100000  # docs indexed by line number; test = last 10%
        gcut = int(gmax * 0.9)
    tr = g < gcut
    idx = np.flatnonzero(tr)
    take = rng.random(len(idx)) < frac
    Xtr.append(X[idx[take]]); ytr.append(y[idx[take]])
    te = np.flatnonzero(~tr)
    if len(te):
        keep = rng.random(len(te)) < 0.5
        Xte.append(X[te[keep]]); yte.append(y[te[keep]])
    del d, X, y, g; gc.collect()
Xtr = np.concatenate(Xtr); ytr = np.concatenate(ytr)
Xte = np.concatenate(Xte); yte = np.concatenate(yte)
gc.collect()
print(f"train={len(Xtr)} test={len(Xte)}", flush=True)
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import roc_auc_score
m = GradientBoostingRegressor(n_estimators=120, max_depth=6, learning_rate=0.1,
                              subsample=0.7, verbose=1)
m.fit(Xtr, ytr)
pred = m.predict(Xte)
yb = (yte > 0.5).astype(int)
print(f"GBR n120d6 v3: AUC {roc_auc_score(yb, pred):.4f}", flush=True)
import joblib
joblib.dump(m, "benchmark/experiments/gbm_v3big_n120d6.joblib")
print("saved", flush=True)
