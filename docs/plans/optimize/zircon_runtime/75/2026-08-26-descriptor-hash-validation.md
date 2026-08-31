---
title: Runtime75 Descriptor Hash Validation
category: zircon_runtime
report_id: Runtime75-descriptor-hash-validation-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_queued
---

# Runtime75 Descriptor Hash Validation

## Scope

This slice removes ordered-tree membership from component descriptor validation. Prop, retained
state, option, and slot identities are checked for uniqueness only; diagnostics still report the
first invalid entry in authored schema order.

## Change

- Replace three `BTreeSet<&str>` admission structures with capacity-sized `HashSet<&str>` values.
- Keep schema, option, and slot traversal in source order so the first duplicate diagnostic is
  unchanged.
- Borrow every identity from the descriptor; validation adds no per-key string clone.

## Deterministic Performance Evidence

| Representative 65,536 unique schemas | Before | After |
|---|---:|---:|
| Membership structure | ordered tree | capacity-sized hash set |
| Membership class | O(log n) per admission | average O(1) per admission |
| Borrowed schema key allocations | 0 | 0 |
| Diagnostic order | authored order | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME75_DESCRIPTOR_HASH_VALIDATION_BENCH_V1`. Acceptance requires hash validation P95 to be at
most 60% of ordered validation P95. Exact Windows timings remain coordinator-owned.

## Acceptance

- Prop, option, and slot duplicates still return the first authored duplicate and the same typed
  error.
- A bounded source contract requires all three borrowed hash sets and rejects ordered membership.
- The release benchmark checks equivalent valid-schema results and enforces the 60% P95 threshold.

## Current-source validation handoff

- Managed ticket `3a757eb834c641e48df49f38f95d3c32` is queued against snapshot `2446` and source manifest `14cbacfa90b451ac3e95010fbd5340fff22d4e14daff6ac0b7b3b8ac9eafd728`. Queue admission is not timing evidence; terminal P95 remains pending.
- The current source contains three capacity-sized borrowed `HashSet` admissions and zero ordered membership admissions in production validation. The deterministic workload remains 65,536 unique schemas, 17 alternating pairs, and identical valid-schema results.
- The single managed batch runs first-duplicate diagnostics, the borrowed-membership source guard, and the ignored Windows release benchmark in sequence. It does not create one coordinator ticket per assertion.
- PowerShell AST parsing, Rust 1.94.1 formatting for both source files, and scoped diff checks pass locally without Cargo. Managed P95 remains pending and is the only timing accepted for final publication.

## Remaining Parent-plan Work

Runtime75 still needs one component authority, descriptor-backed v2 admission, reducer/surface
convergence, typed mutation, and component-specific accessibility. This slice only improves
descriptor identity validation.
