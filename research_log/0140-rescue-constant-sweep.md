# 0140: rescue/model constant sweep — CONFIRMATORY, no change

Swept the tunables never touched this session (inline, no agents), all on
dev_golden against the 0.8902/0.8106 baseline:

| constant | value | F1 | Lev |
|---|---|---|---|
| MODEL_VETO_BIG_THRESHOLD | **0.10 (ship)** | 0.8902 | 0.8106 |
| | 0.05 | 0.8902 | 0.8106 |
| | 0.20 | 0.8902 | 0.8106 |
| RESCUE_NEAR_EMPTY_ABS | **200 (ship)** | 0.8902 | 0.8106 |
| | 150 | 0.8902 | 0.8106 |
| | 300 | 0.8880 | 0.8087 |
| RESCUE_KEEP_FACTOR | **20 (ship)** | 0.8902 | 0.8106 |
| | 15 | 0.8897 | 0.8103 |
| | 25 | 0.8884 | 0.8088 |

The big-block veto threshold is INERT on this corpus (no >1500B block
scores in 0.05-0.20), and both rescue gates sit on plateaus whose only
gradient is downward. Together with 0126 (veto/keep thresholds), every
numeric knob in the extractor is now measured optimal-or-neutral at its
shipped value. Source restored byte-identical; build + tests verified.
