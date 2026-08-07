#!/usr/bin/env python
"""Intra-page structural template subtraction — final prototype.

Score each DOM container by structural repetition of its element children
(depth-2 tag+class subtree signatures, digits normalized). Drop containers
where >=3 children share a signature, the repeated fraction is >=0.5, and the
container's link density is >=0.6 — unless the container holds >50% of body
text (main-content guard). Compare simple text dump with/without subtraction
against gold via ROUGE-L.

Usage: python final_prototype.py [n_docs=200] [seed=42]
"""
import collections
import gzip
import json
import random
import re
import sys

sys.path.insert(0, '/Users/michaelryan/Documents/School/Stanford/Research/'
                   'chatnoir-resiliparse/benchmark/eval')
from metrics import score_pair
from resiliparse.parse.html import HTMLTree, NodeType

DATA = ('/Users/michaelryan/Documents/School/Stanford/Research/'
        'chatnoir-resiliparse/benchmark/datasets_rawhtml/lpv11/dev.jsonl.gz')

SKIP_TAGS = {'script', 'style', 'noscript', 'template', 'svg', 'iframe',
             'head', 'link', 'meta', 'object', 'embed', 'param', 'source',
             'track', 'map', 'area', 'datalist', 'option'}
BLOCK_TAGS = {'p', 'div', 'section', 'article', 'header', 'footer', 'aside',
              'nav', 'ul', 'ol', 'li', 'table', 'tr', 'td', 'th', 'h1', 'h2',
              'h3', 'h4', 'h5', 'h6', 'blockquote', 'pre', 'figure',
              'figcaption', 'dl', 'dt', 'dd', 'main', 'form', 'fieldset',
              'hr', 'br', 'address', 'summary', 'details', 'center'}

_DIGITS = re.compile(r'\d+')
_WS = re.compile(r'\s+')

# Winning config (tuned on 200 dev docs, confirmed on 200 held-out dev docs).
CFG = dict(
    K=2,                 # signature depth: tag+class of node, children, g-children
    min_children=3,      # container must have >=3 element children
    min_repeated=3,      # >=3 children whose signature occurs >=2 times
    min_frac=0.5,        # repeated children >= 50% of element children
    link_density=0.6,    # anchor-text chars / total chars of container
    max_container_frac=0.5,  # never drop a container holding >50% of body text
)


class SNode:
    __slots__ = ('tag', 'cls', 'attr_str', 'children', 'sigs',
                 'text_len', 'link_len', 'drop')

    def __init__(self, tag, cls, attr_str):
        self.tag, self.cls, self.attr_str = tag, cls, attr_str
        self.children = []          # SNode | ('t', text)
        self.sigs = None
        self.text_len = self.link_len = 0
        self.drop = False


def build_shadow(node, in_link=False, maxk=2):
    tag = node.tag
    if tag in SKIP_TAGS:
        return None
    cls_raw = node.getattr('class') or ''
    cls = _DIGITS.sub('N', ' '.join(sorted(cls_raw.lower().split())))
    ident = node.getattr('id') or ''
    sn = SNode(tag, cls, (cls_raw + ('#' + ident if ident else '')).strip())
    link = in_link or tag == 'a'
    child_sigs = []
    for c in node.child_nodes:
        t = c.type
        if t == NodeType.TEXT:
            txt = c.value
            if txt and not txt.isspace():
                txt = _WS.sub(' ', txt)
                sn.children.append(('t', txt))
                n = len(txt.strip())
                sn.text_len += n
                if link:
                    sn.link_len += n
        elif t == NodeType.ELEMENT:
            csn = build_shadow(c, link, maxk)
            if csn is not None:
                sn.children.append(csn)
                child_sigs.append(csn.sigs)
                sn.text_len += csn.text_len
                sn.link_len += csn.link_len
    base = (tag, cls)
    sigs = [hash(base)]
    for k in range(1, maxk + 1):
        sigs.append(hash((base, tuple(s[k - 1] for s in child_sigs))))
    sn.sigs = tuple(sigs)
    return sn


def mark_drops(sn, body_text_len, dropped_log):
    el = [c for c in sn.children if isinstance(c, SNode)]
    for c in el:
        mark_drops(c, body_text_len, dropped_log)
    n = len(el)
    if n < CFG['min_children'] or sn.text_len == 0:
        return
    groups = collections.Counter(c.sigs[CFG['K']] for c in el)
    repeated = sum(v for v in groups.values() if v >= 2)
    if repeated < CFG['min_repeated'] or repeated / n < CFG['min_frac']:
        return
    if sn.link_len / sn.text_len < CFG['link_density']:
        return
    if sn.text_len > CFG['max_container_frac'] * body_text_len:
        return  # main-content guard: refuse to nuke most of the page
    sn.drop = True
    dropped_log.append((sn.tag, sn.attr_str, repeated, n,
                        round(sn.link_len / sn.text_len, 2), sn.text_len))


def extract(sn, with_drops, out):
    if with_drops and sn.drop:
        return
    block = sn.tag in BLOCK_TAGS
    if block:
        out.append('\n')
    for c in sn.children:
        if isinstance(c, SNode):
            extract(c, with_drops, out)
        else:
            out.append(c[1])
    if block:
        out.append('\n')


def get_text(sn, with_drops):
    out = []
    extract(sn, with_drops, out)
    txt = ''.join(out)
    txt = re.sub(r'[ \t]*\n[ \t]*', '\n', txt)
    txt = re.sub(r'\n{2,}', '\n\n', txt)
    txt = re.sub(r'[ \t]{2,}', ' ', txt)
    return txt.strip()


def main():
    n_docs = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    seed = int(sys.argv[2]) if len(sys.argv) > 2 else 42
    random.seed(seed)
    idx = set(random.sample(range(1000), n_docs))
    rows = []
    with gzip.open(DATA) as f:
        for i, line in enumerate(f):
            if i not in idx:
                continue
            d = json.loads(line)
            tree = HTMLTree.parse(d['html'])
            if tree.body is None:
                continue
            sn = build_shadow(tree.body, maxk=CFG['K'])
            if sn is None:
                continue
            base_txt = get_text(sn, False)
            dropped = []
            mark_drops(sn, sn.text_len, dropped)
            treat_txt = get_text(sn, True)
            b = score_pair(base_txt, d['final_output'])
            t = score_pair(treat_txt, d['final_output'])
            rows.append((i, d['url'], b, t, dropped))

    n = len(rows)
    mb = sum(r[2]['rougeL_f'] for r in rows) / n
    mt = sum(r[3]['rougeL_f'] for r in rows) / n
    mbp = sum(r[2]['rougeL_p'] for r in rows) / n
    mtp = sum(r[3]['rougeL_p'] for r in rows) / n
    mbr = sum(r[2]['rougeL_r'] for r in rows) / n
    mtr = sum(r[3]['rougeL_r'] for r in rows) / n
    deltas = [r[3]['rougeL_f'] - r[2]['rougeL_f'] for r in rows]
    wins = sum(1 for d in deltas if d > 0.005)
    losses = sum(1 for d in deltas if d < -0.005)
    print(f'n={n}  F1 {mb:.4f}->{mt:.4f} ({mt - mb:+.4f})  '
          f'P {mbp:.4f}->{mtp:.4f} ({mtp - mbp:+.4f})  '
          f'R {mbr:.4f}->{mtr:.4f} ({mtr - mbr:+.4f})')
    print(f'wins={wins} losses={losses} ties={n - wins - losses}  '
          f'losses<-0.1: {sum(1 for d in deltas if d < -0.1)}  '
          f'worst={min(deltas):+.3f}')


if __name__ == '__main__':
    main()
