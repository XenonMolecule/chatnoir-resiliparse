# 0132: 0.7-0.9 band taxonomy — the honest final decomposition

## Census (209 docs, loss mass 0.0371, checkpointed fleet, all done)
dominant_loss: site_chrome 81 · missed_content 60 · mixed_small 21 ·
engine 20 · formatting_shape 17 · gold_noise 7 · js_only 2 · order 1.
fix_route: **unfixable 62** · site_whitelist 58 · site_veto 56 ·
lexicon_line 24 · gold_edit_needs_owner 7 · gold_edit_approved 2.

## Action attempt and result
Regex-extracted the literal selectors from the fleet's fix descriptions
(29 vetoes / 24 whitelists → only 26 new rows after dedupe against the
~2100-row tables) — battery net-zero plus one −0.003 regression
(devilslakejournal pagination whitelist; removed). Verdict: **the band's
cleanly-selectable chrome was already mined** — waves 8-13 targeted
F1<0.93, which contains this band; what the census labels site_veto/
site_whitelist is mostly the residue where agents could NOT verify a
clean container (mixed containers, per-page ids), which is why waves
plateaued. The est_f1_gain sum (+0.0162) is triage optimism over
already-burned ground (learned-lesson: est_gains need per-doc verify).

## Final reachable-gap decomposition (golden v13, gap −0.0112)
- unfixable by construction (formatting_shape/js/mixed): ~62 docs ≈ −0.006
- owner-gated (7 needs_owner + 0108's three) ≈ −0.002
- residual diffuse (site-rule residue, engine tails) ≈ −0.003-0.004
The 0.90 F1 target on golden is NOT reachable by any measured autonomous
lane; it requires the owner-gated edits plus either computed-CSS
formatting or JS rendering (walls #16/#5).

0132-v2: golden 0.8888/0.8085 (0 downs), plain 1000/1000, tests 7/7.
