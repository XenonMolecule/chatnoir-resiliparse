#!/usr/bin/env python
"""Chunked feature+label extraction over big_train for model v2."""
import gzip, json, re, sys, os
from collections import Counter
import numpy as np
from resiliparse._extract_rs import block_features

SRC = "/Users/michaelryan/Documents/School/Stanford/Research/jusText/benchmark/datasets_rawhtml/lpv11/big_train.jsonl.gz"
OUT = "benchmark/experiments/big_cache"
os.makedirs(OUT, exist_ok=True)
TOK = re.compile(r"\w+")
CHUNK = 5000
start = int(sys.argv[1]) if len(sys.argv) > 1 else 0
stop = int(sys.argv[2]) if len(sys.argv) > 2 else 100000

X, y, groups = [], [], []
chunk_id = start // CHUNK
docs = 0
with gzip.open(SRC, "rt") as f:
    for i, line in enumerate(f):
        if i < start: continue
        if i >= stop: break
        r = json.loads(line)
        gold_toks = Counter(t.lower() for t in TOK.findall(r["final_output"]))
        try: feats = block_features(r["html"])
        except BaseException: continue
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
            y.append(contained/nb)
            groups.append(i)
        docs += 1
        if (i + 1) % CHUNK == 0:
            np.savez_compressed(f"{OUT}/chunk_{(i+1)//CHUNK:03d}.npz",
                                X=np.array(X, dtype=np.float32), y=np.array(y), groups=np.array(groups))
            print(f"chunk to doc {i+1}: {len(X)} blocks", flush=True)
            X, y, groups = [], [], []
if X:
    np.savez_compressed(f"{OUT}/chunk_final.npz", X=np.array(X, dtype=np.float32), y=np.array(y), groups=np.array(groups))
print(f"done: {docs} docs", flush=True)
