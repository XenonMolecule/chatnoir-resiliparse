# 0162-0163: recall-tail whitelists (+0.0002) and the asset-URL fallback (negative)

## Framing that motivated both
The 16 worst docs carry **0.0114 of loss — roughly double the remaining
0.0058 gap**. Five are pure recall (high P, low R): we emit clean text and
miss content. Fixing even half the tail would close the gap arithmetically.

## 0162 (shipped): container mining for recall docs
Mined containers holding gold lines absent from our output.
happysadlola.blogspot `.fauxborder-left` → **+0.145**, zero downs.
androidpolice `#dsq-comments`/`.single` added but inert.
golden 0.89423 → **0.89438**.

## 0163 (reverted): why the rest are unreachable
huskers, english-subtitles and greshamlodge have no `og:url`/canonical and
too few absolute anchors for the 0109 majority-host fallback, so NO site
rule can ever fire on them. Extending the fallback to asset URLs
(img/script/link `src`) was the obvious fix and failed: **asset hosts are
routinely CDNs** (cloudfront, static.*), so the fallback resolved the wrong
domain and misapplied rules — raptorsrepublic −0.484, bobvila −0.345,
straightdope −0.176. Reverted; parity verified.

**Consequence**: a doc with no canonical metadata and few absolute anchors
is permanently outside the site-rule mechanism. That is a structural limit
of domain-gated rules, not a missing rule — and it covers several of the
worst remaining docs (huskers 0.247, english-subtitles 0.273,
greshamlodge 0.263).
