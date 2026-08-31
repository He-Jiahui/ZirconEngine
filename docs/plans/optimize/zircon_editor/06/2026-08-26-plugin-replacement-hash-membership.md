---
title: Editor06 Plugin Replacement Hash Membership
category: zircon_editor
report_id: Editor06-plugin-replacement-hash-membership-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor06 Plugin Replacement Hash Membership

## Scope

This slice removes ordered membership sets from editor-plugin catalog replacement cleanup and
hot-reload dispatch. It does not change the lifecycle state machine, publication generation,
failure recovery, native host authority, or product status reporting.

## Change

- Collect replaced live package IDs into a `HashSet<String>`.
- Collect the retirement subset into a second `HashSet<String>`.
- Keep cleanup, activation, and hot-reload event order driven by the sorted manager entry slice.
- Preserve replacement fault handling and atomic catalog publication behavior.

## Deterministic Performance Evidence

| 32,768 replaced packages | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-set admissions across two sets | 65,536 | 0 | 100% removed |
| Hash-set admissions across two sets | 0 | 65,536 | average O(1) admission |
| Membership probes across cleanup/dispatch | 98,304 | 98,304 | O(log packages) to average O(1) |
| Lifecycle iteration order | manager entry order | same | unchanged |

The ignored release gate alternates 17 two-tree and two-hash membership workflows. It emits
`EDITOR06_PLUGIN_REPLACEMENT_HASH_MEMBERSHIP_BENCH_V1`; acceptance requires hash-membership P95 to
be at most 60% of ordered-membership P95. Exact Windows timings remain pending the coordinator
run.

## Acceptance

- `optimization_batch_20260826p_editor06_hash_membership_preserves_replacement_state_order`
  replaces two active packages through the real manager and checks sorted active output.
- `optimization_batch_20260826p_editor06_replacement_uses_hash_membership_sets` requires all three
  hash-set type boundaries and rejects an ordered production set.
- `optimization_batch_20260826p_editor06_plugin_replacement_hash_membership_performance_evidence`
  emits two-set workload counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor06 still needs unified manual/automatic unload and hot-reload receipts, manager-generation
status publication, persistence rollback, last-good recovery reporting, native artifact lifecycle
coordination, and full startup/reload scale qualification.
