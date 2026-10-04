---
title: Runtime200 Empty Dispatch Output Fast Path
category: zircon_runtime
report_id: Runtime804-empty-dispatch-output-fast-path-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime200 Empty Dispatch Output Fast Path

## Scope

`RuntimeUiSurfaceSet::record_dispatch_outputs` is called for every routed UI event,
but most events publish neither a host request nor a component action. Before this
slice it still looked up the surface, cloned the `UiTreeId` (including its owned
`String`), and entered both output queues. The new guard returns before those steps
when the two queues this method owns are empty.

The guard deliberately does not inspect widget events, binding reports, applied
effects, or rejected effects: those fields are not consumed by this projection and
must not change the routing contract. Non-empty host/component output continues
through the existing queue and secure-text revocation path unchanged.

## Deterministic performance target

For 100,000 empty dispatch results, the old shape performs at least 100,000
`UiTreeId` string clones and two empty queue traversals per event. The new shape
performs one pair of slice emptiness checks and performs zero tree-id clones and
zero queue traversals. This is a lower-bound structural model, not product timing;
managed Release allocation and input-to-present p50/p95/p99 remain required.

## Local validation

- TDD source contract was RED before the guard and GREEN at `4/4` after it.
- The merged non-tooling contract batch later loaded `870` files and passed
  `3665/3665` tests with zero failures, errors, or skips in `40.083s`.
- Rustfmt and diff checks remain part of the next batched static receipt.

## Acceptance boundary

This is a Runtime production hot-path change with no ABI or wire-shape change.
Keep the record at `managed_validation_pending` until the owner-attributed Windows
Release batch compiles the complete Runtime/Editor source and supplies allocation
plus product percentile evidence. Tooling production remains deferred, and this
session does not poll the coordinator.
