#!/usr/bin/env python
"""Rebuild dev_golden.jsonl.gz from the fleet's verified specs (charter: research_log/GOLD_CHARTER.md)."""
import json, gzip, sys
SPECS = 'research_log/dev_golden_specs.jsonl'
SRC = 'benchmark/datasets_rawhtml/lpv11/dev.jsonl.gz'
DST = 'benchmark/datasets_rawhtml/lpv11/dev_golden.jsonl.gz'
specs = {json.loads(l)['warc_record_id']: json.loads(l) for l in open(SPECS) if l.strip()}
def apply_spec(gold, spec):
    lines = gold.split('\n'); drop = set()
    sp = spec.get('spec') or {}
    for p in (sp.get('patterns') or []):
        if p:
            drop.update(i for i, ln in enumerate(lines) if p in ln)
    for pair in (sp.get('ranges') or []):
        if pair and len(pair) == 2:
            a = next((i for i, ln in enumerate(lines) if pair[0] and pair[0] in ln), None)
            b = next((i for i in range(len(lines)-1, -1, -1) if pair[1] and pair[1] in lines[i]), None)
            if a is not None and b is not None and a <= b:
                drop.update(range(a, b+1))
    out = '\n'.join(ln for i, ln in enumerate(lines) if i not in drop)
    while '\n\n\n' in out: out = out.replace('\n\n\n','\n\n')
    return out
n = 0
with gzip.open(SRC,'rt') as fin, gzip.open(DST,'wt') as fout:
    for line in fin:
        d = json.loads(line); k = d['warc_record_id']
        if k in specs:
            ng = apply_spec(d['final_output'], specs[k])
            if 0 < len(ng) < len(d['final_output']):
                d['final_output'] = ng; d['gold_edited'] = True; n += 1
        fout.write(json.dumps(d) + '\n')
print(f'{n} docs edited')
