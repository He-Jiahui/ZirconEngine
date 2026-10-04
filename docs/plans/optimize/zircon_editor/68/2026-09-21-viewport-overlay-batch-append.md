---
title: Editor Viewport Overlay Batch Append
category: zircon_editor
report_id: Editor898-viewport-overlay-batch-append-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor898 Viewport Overlay Batch Append

## Finding and optimization

Enabled viewport-overlay providers returned already-materialized gizmo vectors,
but the registry flattened their outputs through an iterator with no global
length hint and collected into a growable result. The registry now visits the
same ordered providers, reserves each nonempty returned batch's actual length
before moving it into the result, and skips allocation for empty/faulted
batches. This bounds output growth to provider-batch boundaries rather than
individual gizmos. It does not eliminate the provider-owned vectors, claim a
single global output allocation, or change plugin callback, capability,
fault-quarantine, visibility, or gizmo order semantics.

## TDD and deterministic evidence

The source/lower contract was RED with one failing assertion and one missing
lower module, then GREEN `3/3`. A lower regression exercises real registry
registration, toggling, empty capacity, ordered provider output, and exact
flattening parity for sparse/dense batches. An ignored
`EDITOR898_VIEWPORT_OVERLAY_BATCH_APPEND_BENCH_V1` runs 101 alternating
Release p50/p95/p99 sample pairs over eight provider-like batches and requires
optimized p95 <= 110% of the retired flattening path; it has not run.
The deterministic static target is one reservation per real batch before its
append, with zero result capacity for no gizmos; actual allocation count and
frame percentiles require measured evidence.

The focused Editor898/Editor881 viewport and recent Runtime/Editor contract
batch passes `56/56`; exact Rustfmt and scoped diff checks pass. This source
change happened after v25 started and therefore is not covered by v25. No
standalone Cargo run or duplicate coordinator submission was performed while
shared source-copy admission remained unstable.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs` | `F3B35ED5CA8FB9B14C7EF0F0E3B7E3F601CCA84D21E231D902DE5E4EBB35726D` |
| `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers/batch_append_tests.rs` | `DBBBE41536504924132B757267D04DD5D01EEBD892981576968E61B2538E68F9` |
| `tools/tests/test_editor898_viewport_overlay_batch_append_performance_contract.py` | `8F42EB08DFDB5FE34A31974B203FA583F5DEDE41FFE609808020E9FA2278FE60` |

## Acceptance boundary

This slice has a static batch-reservation target, not accepted performance.
Current-source managed Editor compilation, Rust lower tests, ignored Release
percentiles, allocator count/bytes, and viewport product p50/p95/p99 are
pending in the next grouped validation wave.
