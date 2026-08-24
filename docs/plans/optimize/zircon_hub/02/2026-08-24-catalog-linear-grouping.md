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
