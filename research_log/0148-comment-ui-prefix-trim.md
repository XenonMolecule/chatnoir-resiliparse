# 0148: comment-UI prefix trim — golden 0.8915/0.8117, 1 up / 0 down

Deferred item from 0147: "Reply · Report · Jane on Feb 17, 2014" mixes UI
affordances with the attribution gold keeps. Prefix-trim (drop
"Reply · Report · ", keep the rest) instead of dropping the line.

Battery: golden up 1 / down 0 (Lev 0.81168 → 0.81171), train unchanged,
dev unchanged. Tiny but strictly non-negative — the correct shape for this
family, and the pattern generalizes to other "affordance-prefixed
attribution" lines if they show up in future censuses.

Guardrails: plain 1000/1000 identical, extract_golden 7/7.
