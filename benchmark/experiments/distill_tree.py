#!/usr/bin/env python
"""Distill the block classifier to exportable forms; compare veto-tier quality."""
import gzip, json, re, sys
from collections import Counter
import numpy as np
from resiliparse._extract_rs import block_features

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
cut = int(docs*0.8); tr = groups < cut; te = ~tr
yb = (y > 0.5).astype(int)
from sklearn.tree import DecisionTreeRegressor
from sklearn.metrics import roc_auc_score
for depth in (6, 8, 10):
    t_ = DecisionTreeRegressor(max_depth=depth, min_samples_leaf=200)
    t_.fit(X[tr], y[tr])
    pred = t_.predict(X[te])
    auc = roc_auc_score(yb[te], pred)
    line = f"tree d{depth}: AUC {auc:.4f}"
    for thr in (0.05, 0.1, 0.15):
        v = pred < thr
        if v.sum(): line += f" | veto@{thr}: {v.mean():.2%} cov, {yb[te][v].mean():.2%} fv"
    print(line, flush=True)
# export the best (d8) as Rust code
best = DecisionTreeRegressor(max_depth=8, min_samples_leaf=200)
best.fit(X, y)
t = best.tree_
names = ["tag","depth","log_text_len","link_density","n_links","page_ld","frac_page",
         "punct","digit","upper","avgw","nav","footer","header","sidebar","social",
         "article","chrome","byline","widget","recommended","comments"]
lines = []
def emit(node, indent):
    pad = "    " * indent
    if t.children_left[node] == -1:
        lines.append(f"{pad}return {t.value[node][0][0]:.4f};")
        return
    f, thr = names[t.feature[node]], t.threshold[node]
    lines.append(f"{pad}if f.{f} <= {thr:.6f} {{")
    emit(t.children_left[node], indent+1)
    lines.append(f"{pad}}} else {{")
    emit(t.children_right[node], indent+1)
    lines.append(f"{pad}}}")
emit(0, 1)
open("benchmark/experiments/tree_export.rs.txt","w").write("\n".join(lines))
print(f"exported {len(lines)} lines of Rust tree", flush=True)
