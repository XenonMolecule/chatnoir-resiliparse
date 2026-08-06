#!/usr/bin/env python3
"""Generate a self-contained HTML report to *see* the extraction data.

Opens in a browser. Shows where the gaps come from (failure taxonomy, over/under-
extraction, gold-noise vs model-limited) and a sortable/filterable doc browser with
colour-coded gold-vs-prediction diffs — so a score like F1=0.88 becomes tangible
(what's typically missing/extra at that level).

    python benchmark/eval/report.py --tag 0019-nbr --dataset general --split dev
    open benchmark/runs/0019-nbr/general/dev.report.html
"""

import argparse
import difflib
import html
import json
import os
import re
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
from analysis import load_run  # noqa: E402

_NOISE = re.compile(r"(</s>|the final answer is|the user wants|i need to (extract|find|parse)|"
                    r"we need to (output|extract)|the main content (text|is|likely)|"
                    r"the html (is|content|appears|snippet)|extract the main content)", re.I)


def diff_segments(gold, pred):
    """Word-level diff as [op, text] segments; long equal runs collapsed."""
    g, p = gold.split(), pred.split()
    segs = []
    for op, i1, i2, j1, j2 in difflib.SequenceMatcher(a=g, b=p, autojunk=False).get_opcodes():
        if op == "equal":
            words = g[i1:i2]
            if len(words) > 14:
                segs.append(["eq", " ".join(words[:6]) + f"  …({len(words)} words)…  " + " ".join(words[-6:])])
            else:
                segs.append(["eq", " ".join(words)])
        else:
            if i1 != i2:
                segs.append(["del", " ".join(g[i1:i2])])
            if j1 != j2:
                segs.append(["ins", " ".join(p[j1:j2])])
    return segs


def classify(d):
    if _NOISE.search(d.gold[:160] or ""):
        return "gold-noise"
    if not d.pred:
        return "empty"
    if d.score >= 0.95:
        return "great"
    if d.length_ratio > 1.25:
        return "over-extract"
    if d.length_ratio < 0.75:
        return "under-extract"
    return "partial"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tag", default=None)
    ap.add_argument("--dataset", default="general")
    ap.add_argument("--split", default="dev")
    ap.add_argument("--out", default=None)
    ap.add_argument("--allow-test", action="store_true",
                    help="view a held-out test run that has already been recorded")
    args = ap.parse_args()

    run = load_run(args.tag, args.split, args.dataset, allow_test=args.allow_test)
    docs = []
    for i, d in enumerate(run.docs):
        cat = classify(d)
        docs.append({
            "i": i,
            "id": (d.id or f"#{d.index}")[:24],
            "url": d.url[:80],
            "snapshot": d.snapshot,
            "f1": round(d.score, 3),
            "lev": round(d.metrics.get("lev_similarity", 0), 3),
            "ratio": round(d.length_ratio, 2),
            "gold_chars": d.gold_chars,
            "pred_chars": d.pred_chars,
            "cat": cat,
            "segs": diff_segments(d.gold, d.pred),
            "gold": d.gold,
            "pred": d.pred or "",
        })

    import numpy as np
    f1s = [d["f1"] for d in docs]
    levs = [d["lev"] for d in docs]
    cats = {}
    for d in docs:
        cats.setdefault(d["cat"], []).append(d["f1"])
    summary = {
        "tag": run.tag, "dataset": run.dataset, "split": run.split, "n": len(docs),
        "f1": round(float(np.mean(f1s)), 4), "lev": round(float(np.mean(levs)), 4),
        "median_f1": round(float(np.median(f1s)), 4),
        "median_lev": round(float(np.median(levs)), 4),
        "cats": {k: {"n": len(v), "mean_f1": round(float(np.mean(v)), 3)} for k, v in
                 sorted(cats.items(), key=lambda kv: np.mean(kv[1]))},
        "hist": [sum(1 for x in f1s if lo <= x < lo + 0.1 or (lo == 0.9 and x == 1.0))
                 for lo in [i / 10 for i in range(10)]],
        "lev_hist": [sum(1 for x in levs if lo <= x < lo + 0.1 or (lo == 0.9 and x == 1.0))
                     for lo in [i / 10 for i in range(10)]],
    }

    def embed(obj):
        # Safe inside a <script> tag: neutralise </script> & HTML chars so the tag
        # can't close early. The \uXXXX forms are valid JSON -> identical data.
        s = json.dumps(obj, ensure_ascii=False)
        s = s.replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026")
        for ch in ("\u2028", "\u2029"):
            s = s.replace(ch, "\\u%04x" % ord(ch))
        return s

    out = args.out or os.path.join(_HERE, os.pardir, "runs", run.tag, run.dataset,
                                   f"{run.split}.report.html")
    out = os.path.abspath(out)
    with open(out, "w", encoding="utf-8") as fh:
        fh.write(_HTML.replace("/*DATA*/",
                               "const SUMMARY=" + embed(summary) +
                               ";\nconst DOCS=" + embed(docs) + ";"))
    print(f"wrote {out}  ({os.path.getsize(out)/1e6:.1f} MB, {len(docs)} docs)")
    print(f"open it:  open {out}")


_HTML = r"""<!doctype html><html><head><meta charset="utf-8"><title>resiliparse extraction report</title>
<style>
:root{--del:#ffd7d5;--delt:#82071e;--ins:#ccf0d6;--inst:#0a5d2a;--eq:#475569;--bg:#0f172a;--card:#1e293b;--mut:#94a3b8}
*{box-sizing:border-box}body{margin:0;font:14px/1.5 -apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif;background:#f1f5f9;color:#0f172a}
header{position:sticky;top:0;background:var(--bg);color:#fff;padding:14px 20px;z-index:10;box-shadow:0 2px 8px rgba(0,0,0,.2)}
header h1{margin:0 0 4px;font-size:18px}.big{font-size:26px;font-weight:700}.tgt{color:#fbbf24}.mut{color:var(--mut)}
.wrap{max-width:1180px;margin:0 auto;padding:18px 20px}
.cards{display:grid;grid-template-columns:1fr 1fr;gap:14px;margin-bottom:16px}
.card{background:#fff;border-radius:10px;padding:14px 16px;box-shadow:0 1px 3px rgba(0,0,0,.08)}
.card h3{margin:0 0 10px;font-size:13px;text-transform:uppercase;letter-spacing:.04em;color:#64748b}
.bar{height:18px;background:#e2e8f0;border-radius:4px;overflow:hidden;display:inline-block;vertical-align:middle;width:120px}
.bar>i{display:block;height:100%;background:#3b82f6}
.row2{display:flex;justify-content:space-between;align-items:center;padding:3px 0;cursor:pointer}
.row2:hover{background:#f8fafc}
.pill{display:inline-block;padding:1px 8px;border-radius:10px;font-size:11px;font-weight:600}
.c-great{background:#dcfce7;color:#166534}.c-partial{background:#fef9c3;color:#854d0e}.c-over-extract{background:#fee2e2;color:#991b1b}
.c-under-extract{background:#dbeafe;color:#1e40af}.c-empty{background:#f3e8ff;color:#6b21a8}.c-gold-noise{background:#e2e8f0;color:#334155}
.controls{display:flex;gap:10px;flex-wrap:wrap;align-items:center;margin-bottom:10px;position:sticky;top:78px;background:#f1f5f9;padding:8px 0;z-index:5}
select,input{padding:6px 8px;border:1px solid #cbd5e1;border-radius:6px;font-size:13px}
table{width:100%;border-collapse:collapse;background:#fff;border-radius:8px;overflow:hidden}
th,td{padding:7px 10px;text-align:left;border-bottom:1px solid #eef2f6;font-size:13px}
th{background:#f8fafc;cursor:pointer;user-select:none;font-size:12px;color:#475569}
tr.docrow{cursor:pointer}tr.docrow:hover{background:#f8fafc}
.num{font-variant-numeric:tabular-nums;text-align:right}
.diff{padding:14px;background:#fbfcfd;border-top:2px solid #e2e8f0;white-space:pre-wrap;word-break:break-word}
.diff .eq{color:var(--eq)}.diff .del{background:var(--del);color:var(--delt);text-decoration:line-through;border-radius:2px}
.diff .ins{background:var(--ins);color:var(--inst);border-radius:2px}
.legend{font-size:12px;color:#64748b;margin:6px 0}
.legend b.del{background:var(--del);color:var(--delt);padding:0 4px;border-radius:2px}
.legend b.ins{background:var(--ins);color:var(--inst);padding:0 4px;border-radius:2px}
.tabs{margin-bottom:8px}.tabs button{font:inherit;padding:4px 10px;margin-right:6px;border:1px solid #cbd5e1;background:#fff;border-radius:6px;cursor:pointer}
.tabs button.on{background:#3b82f6;color:#fff;border-color:#3b82f6}
.side{display:none;grid-template-columns:1fr 1fr;gap:12px}.side>div{background:#fff;padding:10px;border-radius:6px;border:1px solid #e2e8f0;white-space:pre-wrap;word-break:break-word;max-height:480px;overflow:auto}
.side h4{margin:0 0 6px;font-size:12px;color:#64748b}
</style></head><body>
<header><h1>resiliparse extraction report</h1>
<span id="hdr"></span></header>
<div class="wrap">
<div class="cards">
 <div class="card"><h3>Where the gaps come from</h3><div id="cats"></div></div>
 <div class="card"><h3>F1 distribution — click a band to see those docs</h3><div id="hist"></div></div>
 <div class="card"><h3>Lev distribution — click a band to see those docs</h3><div id="levhist"></div></div>
</div>
<div class="controls">
 <label>Sort <select id="sort">
  <option value="f1a">F1 ↑ (worst first)</option><option value="f1d">F1 ↓ (best first)</option>
  <option value="leva">Lev ↑</option><option value="ratiod">ratio ↓ (over-extract)</option><option value="ratioa">ratio ↑ (under-extract)</option>
 </select></label>
 <label>Category <select id="cat"><option value="">all</option></select></label>
 <label>F1 band <select id="band"><option value="">all</option></select></label>
 <label>Lev band <select id="levband"><option value="">all</option></select></label>
 <input id="q" placeholder="search url…" size="20">
 <span class="mut" id="count"></span>
</div>
<div class="legend">Diff vs gold: <b class="del">red strikethrough = in gold, MISSED by resiliparse</b> &nbsp; <b class="ins">green = ADDED by resiliparse, not in gold</b> &nbsp; grey = match. Click any row to expand.</div>
<table><thead><tr>
 <th data-k="f1">F1</th><th data-k="lev">Lev</th><th data-k="ratio">ratio</th><th data-k="cat">category</th><th data-k="url">url</th>
</tr></thead><tbody id="tb"></tbody></table>
</div>
<script>
/*DATA*/
const $=s=>document.querySelector(s);
$("#hdr").innerHTML=`<span class="big">${SUMMARY.tag}</span> &nbsp; ${SUMMARY.dataset}/${SUMMARY.split} (${SUMMARY.n} docs) &nbsp;&nbsp; mean F1 <span class="big">${SUMMARY.f1}</span> <span class="mut">(median ${SUMMARY.median_f1})</span> &nbsp; mean Lev <span class="big">${SUMMARY.lev}</span> <span class="mut">(median ${SUMMARY.median_lev})</span> &nbsp; <span class="tgt">target 0.90 / 0.85</span>`;
// category bars
const cmax=Math.max(...Object.values(SUMMARY.cats).map(c=>c.n));
$("#cats").innerHTML=Object.entries(SUMMARY.cats).map(([k,c])=>
 `<div class="row2" onclick="document.getElementById('cat').value='${k}';apply()"><span><span class="pill c-${k}">${k}</span> <span class="mut">mean F1 ${c.mean_f1}</span></span><span><span class="bar"><i style="width:${100*c.n/cmax}%"></i></span> <b>${c.n}</b></span></div>`).join("");
// histogram
const hmax=Math.max(...SUMMARY.hist);
$("#hist").innerHTML=SUMMARY.hist.map((n,i)=>{const lo=(i/10).toFixed(1),hi=((i+1)/10).toFixed(1);
 return `<div class="row2" onclick="document.getElementById('band').value='${i}';apply()"><span class="mut">${lo}–${hi}</span><span><span class="bar"><i style="width:${100*n/hmax}%"></i></span> <b>${n}</b></span></div>`}).join("");
// lev histogram
const lhmax=Math.max(...SUMMARY.lev_hist);
$("#levhist").innerHTML=SUMMARY.lev_hist.map((n,i)=>{const lo=(i/10).toFixed(1),hi=((i+1)/10).toFixed(1);
 return `<div class="row2" onclick="document.getElementById('levband').value='${i}';apply()"><span class="mut">${lo}–${hi}</span><span><span class="bar"><i style="width:${100*n/lhmax}%"></i></span> <b>${n}</b></span></div>`}).join("");
// fill selects
const cats=[...new Set(DOCS.map(d=>d.cat))];
$("#cat").innerHTML='<option value="">all</option>'+cats.map(c=>`<option>${c}</option>`).join("");
$("#band").innerHTML='<option value="">all</option>'+[...Array(10)].map((_,i)=>`<option value="${i}">${(i/10).toFixed(1)}–${((i+1)/10).toFixed(1)}</option>`).join("");
$("#levband").innerHTML='<option value="">all</option>'+[...Array(10)].map((_,i)=>`<option value="${i}">${(i/10).toFixed(1)}–${((i+1)/10).toFixed(1)}</option>`).join("");
const esc=s=>s.replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));
function rowHTML(d){return `<tr class="docrow" data-i="${d.i}"><td class="num">${d.f1.toFixed(3)}</td><td class="num">${d.lev.toFixed(3)}</td><td class="num">${d.ratio}</td><td><span class="pill c-${d.cat}">${d.cat}</span></td><td class="mut">${esc(d.url)}</td></tr>`}
function diffHTML(d){return d.segs.map(s=>`<span class="${s[0]}">${esc(s[1])}</span>`).join(" ")}
function apply(){
 let v=DOCS.slice();
 const cat=$("#cat").value,band=$("#band").value,levband=$("#levband").value,q=$("#q").value.toLowerCase();
 if(cat)v=v.filter(d=>d.cat==cat);
 if(band!=="")v=v.filter(d=>{const b=Math.min(9,Math.floor(d.f1*10));return b==+band});
 if(levband!=="")v=v.filter(d=>{const b=Math.min(9,Math.floor(d.lev*10));return b==+levband});
 if(q)v=v.filter(d=>d.url.toLowerCase().includes(q));
 const s=$("#sort").value;
 const cmp={f1a:(a,b)=>a.f1-b.f1,f1d:(a,b)=>b.f1-a.f1,leva:(a,b)=>a.lev-b.lev,ratiod:(a,b)=>b.ratio-a.ratio,ratioa:(a,b)=>a.ratio-b.ratio}[s];
 v.sort(cmp);
 $("#count").textContent=v.length+" docs";
 $("#tb").innerHTML=v.slice(0,400).map(rowHTML).join("")+(v.length>400?`<tr><td colspan=5 class=mut>…showing first 400 of ${v.length} (filter to narrow)</td></tr>`:"");
}
$("#tb").addEventListener("click",e=>{
 const tr=e.target.closest("tr.docrow");if(!tr)return;
 const ex=tr.nextElementSibling;
 if(ex&&ex.classList.contains("exp")){ex.remove();return}
 const d=DOCS[+tr.dataset.i];
 const row=document.createElement("tr");row.className="exp";
 row.innerHTML=`<td colspan="5"><div class="diff">
  <div class="tabs"><button class="on" onclick="setv(this,'unified')">unified diff</button><button onclick="setv(this,'side')">side by side</button>
   <span class="mut">&nbsp; ${d.gold_chars} gold chars vs ${d.pred_chars} pred chars · ratio ${d.ratio}</span></div>
  <div class="vunified">${diffHTML(d)}</div>
  <div class="side"><div><h4>GOLD (teacher)</h4>${esc(d.gold)}</div><div><h4>resiliparse prediction</h4>${esc(d.pred)}</div></div>
 </div></td>`;
 tr.after(row);
});
function setv(btn,which){const box=btn.closest(".diff");box.querySelectorAll(".tabs button").forEach(b=>b.classList.remove("on"));btn.classList.add("on");
 box.querySelector(".vunified").style.display=which=="unified"?"block":"none";box.querySelector(".side").style.display=which=="side"?"grid":"none"}
["sort","cat","band","levband"].forEach(id=>$("#"+id).addEventListener("change",apply));$("#q").addEventListener("input",apply);
apply();
</script></body></html>"""


if __name__ == "__main__":
    main()
