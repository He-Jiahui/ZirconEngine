---
title: Runtime11A Focus Navigation Output Capacity
category: zircon_runtime
report_id: Runtime11A-focus-navigation-capacity-2026-09-16
date: 2026-09-16
session_id: root-astra-optimize-20260916
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime11A · Focus navigation output capacity

## Finding

`UiRuntimeTreeFocusExt::focusable_nodes_in_navigation_order` collected every
focus candidate into a zero-capacity `Vec`. The recursive collector can emit no
more IDs than the tree's node map contains, so `self.nodes.len()` is a safe
bounded upper-capacity hint. It may reserve more than the reachable or
focusable subset, but it does not change traversal authority or output order.

## Implementation

- Reserve `self.nodes.len()` before walking roots and recursively collecting
  focus candidates.
- Keep missing-node errors, root order, depth-first child order, visibility,
  enabled-state, and focus-candidate semantics unchanged.
- Add a lower regression that constructs enabled focusable roots and verifies
  exact navigation order, count, and the capacity upper bound.

## Performance contract

The lower module contains the ignored managed Release marker
`RUNTIME791_FOCUS_NAVIGATION_CAPACITY_BENCH_V1`. It alternates legacy and
reserved implementations over 17 sample pairs and 2,048 lookups per sample on
a 4,096-root fixture, emitting raw samples and p95 values for the owner lane.
The benchmark only asserts non-zero measurements and the deterministic growth
ordering; it does not turn noisy local wall-clock timing into product
acceptance.

The deterministic doubling model starts at a four-entry vector: 4,096 output
IDs require 11 modeled legacy growth events, while the node-count reservation
requires 0. This is allocation-shape evidence, not allocator, RSS, CPU, or
product p50/p95/p99 evidence.

## Local TDD receipt

The source contract was RED first (one reservation assertion failed while the
old `Vec::new()` implementation remained), then GREEN at `3/3`. The lower
order/capacity regression and ignored Release marker are wired, and both focus
Rust files are Rustfmt-clean. The refreshed single-process Runtime/Editor
performance-contract batch loads 553 files and passes `1978/1978` tests in
`5.513s`, with zero failures, errors, or skips. This is local source/model
evidence only. No standalone Cargo command was started.

## Acceptance boundary

Keep this slice `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Runtime tree,
runs the lower test and ignored marker, and supplies allocator plus navigation
product p50/p95/p99 evidence. The external `E:\Git\zr_vm` dirty-worktree
admission boundary remains recorded in the asynchronous validation log. No
coordinator polling or retry is part of this slice; tooling production work is
deferred for the later Rust migration.
