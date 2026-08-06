"""Scoring metrics for resiliparse benchmark evaluation.

Both metrics are backed by rapidfuzz (C++), so they scale to thousands of docs:

  * levenshtein  -- character-level edit distance on the *raw* strings. Newlines,
                    repeated spaces and case all count, by design.
  * rouge_l      -- LCS-based ROUGE-L (precision / recall / F1) over `\\w+` unicode
                    tokens. Computed from rapidfuzz's Indel distance:
                        indel = len(a) + len(b) - 2 * LCS   =>   LCS = (len(a)+len(b)-indel)/2
                    which is exactly the longest-common-subsequence ROUGE-L uses.
"""

import re

from rapidfuzz.distance import Indel, Levenshtein

_WORD = re.compile(r"\w+", re.UNICODE)


def tokenize(text):
    """ROUGE-style tokenization: lowercase, split on unicode word boundaries."""
    return _WORD.findall(text.lower())


def levenshtein(pred, gold):
    """Character-level edit distance (raw strings; newlines/spaces/case matter).

    Returns (distance, similarity) where similarity = 1 - distance/max_len in [0, 1].
    """
    distance = Levenshtein.distance(pred, gold)
    denom = max(len(pred), len(gold)) or 1
    return distance, 1.0 - distance / denom


def rouge_l(pred, gold):
    """ROUGE-L over word tokens. Returns (precision, recall, f1).

    F1 uses beta=1 (the modern `rouge_score` default).
    """
    p_tokens = tokenize(pred)
    g_tokens = tokenize(gold)
    if not p_tokens or not g_tokens:
        # If both are empty the prediction perfectly matched an empty gold.
        return (1.0, 1.0, 1.0) if not p_tokens and not g_tokens else (0.0, 0.0, 0.0)

    indel = Indel.distance(p_tokens, g_tokens)
    lcs = (len(p_tokens) + len(g_tokens) - indel) / 2.0

    precision = lcs / len(p_tokens)
    recall = lcs / len(g_tokens)
    f1 = 2 * precision * recall / (precision + recall) if (precision + recall) else 0.0
    return precision, recall, f1


def score_pair(pred, gold):
    """Compute every metric for one (prediction, gold) pair as a flat dict."""
    lev_distance, lev_similarity = levenshtein(pred, gold)
    rl_p, rl_r, rl_f = rouge_l(pred, gold)
    return {
        "rougeL_p": rl_p,
        "rougeL_r": rl_r,
        "rougeL_f": rl_f,
        "lev_distance": lev_distance,
        "lev_similarity": lev_similarity,
        "pred_chars": len(pred),
        "gold_chars": len(gold),
    }
