"""Title features computed EXACTLY as the Rust extractor will:
   - tokens = whitespace-delimited runs (punctuation included), lowercased
   - no stopword filtering (avoids maintaining a shared list across languages)
   - distinct token sets; title read decoded (DOM semantics)
"""
import gzip, json, re, os, html as html_mod
from collections import Counter
import numpy as np
from resiliparse._extract_rs import block_features

SRC = "benchmark/datasets_rawhtml/lpv11/train.jsonl.gz"
TOK = re.compile(r"\w+")                      # only for the LABEL (gold containment)
TITLE_RE = re.compile(r"<title[^>]*>(.*?)</title>", re.S | re.I)

def ws_tokens(s):
    return {w.lower() for w in s.split() if w}

def doc_rows(r):
    gold_toks = Counter(t.lower() for t in TOK.findall(r["final_output"]))
    m = TITLE_RE.search(r["html"])
    ttoks = ws_tokens(html_mod.unescape(re.sub(r"<[^>]+>", " ", m.group(1)))) if m else set()
    tlen = len(ttoks)
    try: feats = block_features(r["html"])
    except BaseException: return []
    rows = []
    for l in feats.strip().split("\n"):
        if not l: continue
        try: b = json.loads(l)
        except Exception: continue
        btoks_lbl = Counter(t.lower() for t in TOK.findall(b["text"]))
        nb = sum(btoks_lbl.values())
        if nb < 3: continue
        contained = sum(min(c, gold_toks.get(t, 0)) for t, c in btoks_lbl.items())
        tl = b["text_len"]
        bset = ws_tokens(b["text"])
        inter = len(bset & ttoks)
        rows.append(([
            b["tag"], b["depth"], np.log1p(tl),
            b["link_len"]/max(tl,1), b["n_links"], b["page_ld"], tl/max(b["page_text"],1),
            b["punct"], b["digit"], b["upper"], b["avgw"],
            b["nav"], b["footer"], b["header"], b["sidebar"], b["social"],
            b["article"], b["chrome"], b["byline"], b["widget"],
            b["recommended"], b["comments"],
            b["headings"], b["page_headings"], b["prev_ld"], b["next_ld"],
            b["prev_len"], b["next_len"],
            np.log1p(b["page_forms"]), np.log1p(b["page_articles"]),
            np.log1p(b["page_comment_cls"]),
            b["page_nav_share"], b["page_generator"], np.log1p(b["page_n_blocks"]),
            *b["wb"],
            inter/max(len(bset),1), inter/max(tlen,1), np.log1p(tlen),
        ], contained/nb))
    return rows

X, Y = [], []
n = 0
for line in gzip.open(SRC, "rt"):
    for x, y in doc_rows(json.loads(line)):
        X.append(x); Y.append(y)
    n += 1
    if n % 2500 == 0: print(f"{n} docs {len(X)} blocks", flush=True)
np.savez_compressed('/tmp/v7_cache/rustexact.npz', X=np.array(X, dtype=np.float32), y=np.array(Y, dtype=np.float32))
print("DONE", len(X))
