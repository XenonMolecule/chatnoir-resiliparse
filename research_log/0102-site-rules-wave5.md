# 0102 — Site-rules wave 5 (both sides, 362 docs)

- **Date:** 2026-08-09
- **Tag:** 0102-v3 (baseline: 0101-v3)
- **Status:** landed

8 agents over all 362 remaining sub-0.90 docs (current-diff based, so
revisits target only what earlier waves left): 726 filtered rules
(545 veto + 181 whitelist, 2 conflicts dropped) → battery caught 5
craters (over-broad vetoes) AND re-proposals of previously-bisected
domains → 19-domain permanent blocklist installed
(research_log/site_rule_blocklist.json) + all offenders removed →
clean re-battery (skinet mopped in a follow-up bisect).

| target | F1 | Lev | Δ |
|---|---|---|---|
| golden | **0.8683** | **0.7855** | +0.0036 / +0.0043 |
| original dev | **0.8435** | **0.7494** | +0.0036 / +0.0041 |
| train | 0.8154 | 0.7169 | +0.0001 / +0.0000 |

Zero craters; residual both-downs ≤0.09 (8 docs, logged). General
plain 1000/1000; goldens pass. Five-wave lane total: **+0.0193 F1 /
+0.0219 Lev golden**. Goal gap: **−0.032 F1 / −0.0145 Lev**.
