---
title: Hub 02 Catalog Linear Grouping Performance
category: zircon_hub
report_id: Hub02-catalog-linear-grouping-2026-08-24
date: 2026-08-24
session_id: optimize-hub02-catalog-linear-grouping-r1-20260824
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Hub 02 Catalog Linear Grouping Performance

## Scope

This batch replaces the Catalog page's per-item group-array copy with a shared linear grouping
helper. It preserves first-key order, per-group item order, and exactly one key callback per item.
It does not claim the parent plan's marketplace, package lifecycle, permission-policy, accessibility,
or full shell acceptance work is complete.

## Change

The legacy reducer rebuilt the current group with a spread for every item. A 10,000-item single
group therefore copied 50,005,000 references. The shared helper mutates the retained group array and
performs 10,000 appends instead. It also caches the immediately previous key and group. Consecutive
single-group input now performs one Map lookup instead of 10,000, while non-consecutive repeated
keys still resolve through the Map and append to the original group.

The deterministic 10,000-entry counts are:

- reference copy/write operations: `50,005,000 -> 10,000` (`99.980%` lower);
- mutable appends: `0 -> 10,000`;
- Map lookups for consecutive single-group input: `10,000 -> 1` (`99.99%` removed);
- consecutive cache hits: `0 -> 9,999`.

## Performance

Coordinator ticket `9ec8e3976de849b09668471b32c1bb31` used 21 alternating Windows Node
sample pairs and nearest-rank percentiles after the consecutive-group cache was added:

- 10k legacy P50 `209,713,600 ns`, optimized P50 `428,100 ns`;
- 10k legacy P95 `291,978,900 ns`, optimized P95 `627,500 ns` (`99.785%` lower);
- 100k P95 `6,292,100 ns` versus 10x10k P95 `4,083,300 ns`, ratio `1.541`, below the
  required `5.000` ceiling.

The accepted ticket also measured `50,005,000 -> 10,000` reference copy/write operations
(`99.980%` lower), `10,000 -> 1` Map lookups, and 9,999 consecutive cache hits.

## Validation

- Coordinator r2 restored pinned dependencies, passed the source contract and TypeScript
  typecheck, and passed 2/3 Node tests. Its 100k scale gate failed before the consecutive-group
  cache (`45,056,800 ns` versus 10x10k `4,983,200 ns`).
- Coordinator ticket `9ec8e3976de849b09668471b32c1bb31` passed the source contract,
  TypeScript typecheck, all three Node tests, both performance rows, and the 100k completeness gate.
- Scoped whitespace validation passes.
- Final combined Hub02 seal validation, commit, and WeCom publication are pending.

## 2026-08-25 Catalog Search Index Follow-Up

The Catalog page now builds one memoized search index when its row projection changes and reuses
that index across query edits. The six searchable fields are joined with a NUL boundary, so a query
cannot match across adjacent fields. A query that itself contains NUL takes the previous per-field
path, preserving the legacy result set for arbitrary input. Asset, plugin, and learn tab predicates
and stable row order are unchanged.

The focused behavior suite covers empty and trimmed queries, case folding, Unicode, field-boundary
isolation, embedded NUL, all three Catalog modes, and all tab keys. The source contract also requires
`CatalogPage` to construct and consume the shared search index instead of normalizing every row for
every query.

Local Windows Node evidence uses 10,000 rows, 32 queries, 21 alternating sample pairs, nearest-rank
percentiles, and an index built outside the timed query burst:

- legacy P50 `189,061,100 ns`, indexed P50 `28,012,100 ns` (`85.184%` lower);
- legacy P95 `278,685,100 ns`, indexed P95 `93,200,000 ns` (`66.557%` lower), below the required
  indexed-P95-at-most-60%-of-legacy gate;
- field/row normalization calls `1,901,868 -> 10,000` (`99.474%` lower).

The existing 100k grouping scale benchmark was also stabilized without relaxing its 5x gate. Each
sample now batches ten equivalent 10x10k and 100k projections so its tens-of-milliseconds measurement
window is not dominated by a scheduler or GC pause. The prior 3-5 ms method reproduced one false
failure in five isolated processes (`68,686,200 ns > 5 x 5,549,200 ns`); the batched method passed
five of five processes with P95 ratios `1.940`, `1.431`, `1.226`, `1.079`, and `1.682`. The final
combined run passed all six tests; TypeScript typecheck and scoped whitespace validation also pass.
Coordinator validation, integration commit, and WeCom publication for this follow-up remain pending.
