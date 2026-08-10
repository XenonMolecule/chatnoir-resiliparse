"""Score OUR resiliparse against the small-rephraser extraction unit tests
(108 curated keep/drop spans), using their normalize()."""
import json, os, sys, glob
S = os.path.expanduser('~/Documents/School/Stanford/Research/small-rephraser')
sys.path.insert(0, f'{S}/scripts'); sys.path.insert(0, S)
sys.path.insert(0, 'resiliparse-py')
from content_fidelity import normalize
from resiliparse._extract_rs import extract_plain_text

lines = open(f'{S}/static/unit_tests/extraction_tests.jsonl').read().strip().split('\n')[1:]
tests = [json.loads(l) for l in lines]
root = f'{S}/static/warcs/marin_devset_1934_html'
hid2file = {}
for line in open(f'{root}/devset.jsonl'):
    m = json.loads(line)
    hid2file[m['hid']] = os.path.join(root, m['html_file'])
cache = {}
def out_for(doc):
    if doc in cache: return cache[doc]
    text = None
    if doc in hid2file:
        html = open(hid2file[doc], encoding='utf-8', errors='replace').read()
        try: text = extract_plain_text(html, preserve_formatting='markdown', main_content=True)
        except BaseException: text = ''
    cache[doc] = text
    return text

results, skipped = [], 0
by_type = {}
for t in tests:
    o = out_for(t['doc'])
    if o is None: skipped += 1; continue
    no = normalize(o)
    if t.get('match') == 'all_parts':
        parts = t['span'] if isinstance(t['span'], list) else [t['span']]
        present = all(normalize(p) in no for p in parts)
    else:
        present = normalize(t['span']) in no
    ok = present if t['polarity'] == 'keep' else not present
    results.append(ok)
    d = by_type.setdefault(t['type'], [0,0]); d[0] += ok; d[1] += 1
    if not ok:
        print(f"FAIL [{t['type']}/{t['polarity']}] {t['id']}: {str(t['span'])[:70]!r}")
print(f"\nPASS {sum(results)}/{len(results)} ({sum(results)/len(results):.1%}); skipped (non-devset docs): {skipped}")
for ty, (p, n) in sorted(by_type.items()): print(f"  {ty:12s} {p}/{n}")
