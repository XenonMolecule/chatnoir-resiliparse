# 0128: gold DOM-order census — family is a singleton, no edits

Followed up brokenbats (0127, +0.24 from reordering gold to rendered
order): scripted census of all 1000 golds comparing block order to
first-occurrence position in tag-stripped HTML. 69 docs showed >=1
inversion, but under the safety gates (every block must match UNIQUELY in
the html; reorder must improve BOTH metrics vs the current prediction)
**zero survived** — the inversions are match artifacts (forum quotes
duplicate earlier-post text, so first-occurrence positions lie) rather
than true annotation-order noise. brokenbats was the lone real instance
and is already repaired. Golden basis unchanged (v13 0.8888/0.8085;
rebuild verified byte-stable, 0 down docs).

Census script pattern preserved here for reuse if the train-gold audit is
ever authorized (order noise may be more common there).
