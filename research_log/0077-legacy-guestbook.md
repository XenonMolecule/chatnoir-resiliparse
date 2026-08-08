# 0077 — Legacy.com guestbook handler

- **Date:** 2026-08-08
- **Tag:** 0077-legacy (baseline: 0076-lj)
- **Status:** landed (golden-primary)

## What changed
`extract_legacy_gb`: `div.GuestBookEntry` (server-rendered — the JsonP
templates are for pagination only) → `# name`, `**years**`, `##
Condolences`, per-entry `**DATE**  \nmessage  \n— signee`; Disclaimer
kept per charter C4. Probe 0.21 → **0.914/0.827** on golden.

## Results
golden **0.8469/0.7587** (+0.0007/+0.0007, 1 up 0 down). Original dev
−0.0001 Lev: the ORIGINAL gold of this doc lacks the disclaimer and
name-header the golden keeps — golden-primary divergence #2 (original
gold is the noisy one here). train untouched; general plain 1000/1000;
goldens pass.
