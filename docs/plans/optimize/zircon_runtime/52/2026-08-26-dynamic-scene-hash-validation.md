---
title: Runtime52 Dynamic Scene Hash Validation
category: zircon_runtime
report_id: Runtime52-dynamic-scene-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_queued
validation_ticket: efb97b91c4db45569d092ef412235536
---

# Runtime52 Dynamic Scene Hash Validation

## Scope

This slice removes ordered-tree membership from dynamic-scene source-entity and component-type
uniqueness validation. Both checks remain private, walk serialized inputs in their original order,
and report the first repeated identity without publishing set iteration order.

## Change

- Replace source entity `BTreeSet<u64>` admission with `HashSet<u64>`.
- Replace borrowed component type `BTreeSet<&str>` admission with `HashSet<&str>`.
- Preserve schema checks, descriptor validation order, borrowed string identity, and first errors.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique source and component IDs | Before | After |
|---|---:|---:|
| Membership class | O(log n) | average O(1) |
| Component type key allocations | 0 | 0 |
| Validation order | serialized input | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME52_DYNAMIC_SCENE_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95 to be
at most 60% of ordered validation P95. Exact Windows timings remain coordinator-owned.

## Acceptance

- A real `DynamicScene` preserves the first duplicate source error.
- A source contract requires two hash sets and borrowed component type admission.
- The release benchmark checks equivalent unique counts and enforces the 60% P95 threshold.

## Remaining Parent-plan Work

Runtime52 still needs durable multi-process archive transactions, restart-stable revisions,
bounded migration, replace-style restore, rollback, product consumers, and scale qualification.
This slice only improves current scene invariant validation.

## Current-source recovery batch

This task shares one managed ticket with Runtime54 reader-count hashing. The exclusive
`runtime_hash_recovery_batch_` filter runs four ordinary regressions and two ignored release P95
gates in two Cargo invocations; queue admission is not timing evidence.
The ticket is sealed against snapshot `2450` and source manifest
`c185bdebb641bd095aae18cfa4f624615f281cb8fa2f1e79d1c79d324dfa462c`.
