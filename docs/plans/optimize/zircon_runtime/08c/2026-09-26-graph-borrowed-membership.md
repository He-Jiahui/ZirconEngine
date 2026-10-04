---
title: Runtime08C graph traversal borrowed membership keys
category: zircon_runtime
report_id: Runtime08C-graph-borrowed-membership-2026-09-26
date: 2026-09-26
session_id: root-runtime08c-graph-borrowed-membership-20260926
parent_plan: docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_allocation_reduction
---

# Runtime08C graph traversal borrowed membership keys

## Scope

The animation graph evaluator already reserves cycle-detection membership from
the frozen graph node count. Each recursive lookup still converted its node ID
to an owned `String` before inserting it into `HashSet<String>`, including
cycle-hit lookups that were immediately rejected. This slice keeps the graph's
node IDs borrowed for the duration of traversal, so membership does not create
per-node key allocations.

## Implementation

`evaluate_graph` now keeps the output source as a graph-borrowed `&str` while
the recursive traversal runs, cloning it only for the returned evaluation
record. `collect_graph_clips` uses a lifetime-bound `HashSet<&str>` and inserts
`node_id` directly. Traversal order, cycle suppression, blend weights, mask
inheritance, and the public owned output-node field are unchanged.

## Evidence

- TDD RED: the production-section source contract failed while traversal still
  used `HashSet<String>` and `node_id.to_string()`.
- GREEN: the contract now requires `HashSet<&'a str>`, direct borrowed insert,
  and rejects the owned-key insertion form.
- A direct Output→Blend→Clip behavior regression checks that the returned
  output node remains owned and the projected clip count/weight stay unchanged.
- The existing Runtime624 preallocation contract remains alongside this slice,
  preserving the graph-node-count capacity bound.
- The ignored Release marker
  `RUNTIME945_GRAPH_BORROWED_MEMBERSHIP_BENCH_V1` compares the owned-key and
  borrowed-key paths over 17 alternating samples, 32,768 node IDs, and 32
  traversal passes. It reports the deterministic per-sample key-allocation
  counts (`1,048,576` owned-key constructions versus `0` borrowed-key
  constructions) alongside p50/p95; the managed lane owns the p95 threshold.
- The current-source local Windows Release run passed that marker in the same
  grouped graph batch: owned-key p95 `430,150µs`, borrowed-key p95
  `165,672µs` (38.5% of the baseline, below the 85% threshold), with the
  printed allocation counts `1,048,576` versus `0`. The companion Runtime624
  and Runtime08C markers also passed in that batch (`345,684→149,009µs` and
  `13,547→698µs`, respectively).
- The approved Windows `core-min,animation` target completed the grouped Rust
  `performance_contract_tests` module against the current source: `12` tests
  discovered, `9` passed, `0` failed, and `3` Release tests ignored by their
  existing gate conditions. This includes the direct Output→Blend→Clip
  behavior regression.
  This is local debug-profile evidence; the managed Release allocation/p50/p95
  receipt remains the acceptance gate.

## Acceptance boundary

Managed Cargo must compile the current Runtime source and run the graph behavior
contracts. The grouped Release lane should compare owned-key and borrowed-key
traversal over the existing 32,768-node, 17-pair benchmark and report allocation
count plus p50/p95. This record does not claim managed compilation or Release
performance completion until that receipt is attached.
