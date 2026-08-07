#!/usr/bin/env python
"""Confident-learning retrain (0061): drop rows where the v5 model and the
noisy containment label disagree by >0.6 (suspected gold-noise labels)."""
import os, gc, numpy as np, joblib
OUT = "benchmark/experiments/big_v5_cache"
rng = np.random.default_rng(5)
m0 = joblib.load("benchmark/experiments/gbm_v5big_n120d6.joblib")
Xtr, ytr, Xte, yte = [], [], [], []
files = sorted(f for f in os.listdir(OUT) if f.startswith("chunk_"))
gcut = 90000
kept = dropped = 0
for fn in files:
    d = np.load(f"{OUT}/{fn}")
    X, y, g = d["X"], d["y"], d["groups"]
    tr = np.flatnonzero(g < gcut); te = np.flatnonzero(g >= gcut)
    if len(tr):
        sub = tr[rng.random(len(tr)) < 0.35]
        if len(sub):
            pred = m0.predict(X[sub])
            ok = np.abs(pred - y[sub]) <= 0.6
            kept += int(ok.sum()); dropped += int((~ok).sum())
            Xtr.append(X[sub[ok]]); ytr.append(y[sub[ok]])
    if len(te):
        keep = rng.random(len(te)) < 0.25
        Xte.append(X[te[keep]]); yte.append(y[te[keep]])
    del d, X, y, g; gc.collect()
Xtr = np.concatenate(Xtr); ytr = np.concatenate(ytr)
Xte = np.concatenate(Xte); yte = np.concatenate(yte); gc.collect()
print(f"kept {kept} dropped {dropped} ({dropped/(kept+dropped):.1%} suspected-noise rows)", flush=True)
from sklearn.ensemble import GradientBoostingRegressor
from sklearn.metrics import roc_auc_score
m = GradientBoostingRegressor(n_estimators=120, max_depth=6, learning_rate=0.1,
                              subsample=0.7, verbose=1)
m.fit(Xtr, ytr)
pred = m.predict(Xte)
yb = (yte > 0.5).astype(int)
print(f"GBR v5-confident n120d6: AUC {roc_auc_score(yb, pred):.4f} (vs v5 0.8592 on noisy labels)", flush=True)
joblib.dump(m, "benchmark/experiments/gbm_v5conf_n120d6.joblib")
print("saved", flush=True)
