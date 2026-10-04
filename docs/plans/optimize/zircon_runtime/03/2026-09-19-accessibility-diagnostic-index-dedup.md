---
title: Runtime03 accessibility diagnostic node-index deduplication
category: zircon_runtime
report_id: Runtime819-accessibility-diagnostic-index-dedup-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime819 · accessibility diagnostic node-index deduplication

## Scope

`validate_snapshot_bounded` previously built a `BTreeSet` to reject duplicate
node IDs and then inserted the same unique IDs into a second `BTreeMap` for
relation and focus lookup. The two ordered containers duplicated key storage
and tree bookkeeping on every accessibility snapshot validation.

The first pass now uses one `BTreeMap` entry lookup: `Vacant` entries retain
the first node index and `Occupied` entries emit the same duplicate diagnostic
and callback count as before. Relation, description, cycle, focus fallback,
diagnostic order, and first-index semantics are unchanged.

## TDD and deterministic model

- The Python source/model contract was RED against the old parallel set and
  GREEN at `3/3` after the entry-based index and lower duplicate regression
  were added.
- `RUNTIME819_A11Y_DIAGNOSTIC_INDEX_BENCH_V1` is wired for the managed Windows
  Release lane.
- For a 4,096-node validation model, auxiliary ordered index allocations move
  from `2` (`BTreeSet` + `BTreeMap`) to `1` authoritative map (`2 -> 1`).
  This is allocation-shape evidence only; it is not allocator, CPU, RSS, or
  product accessibility percentile evidence.

## Local validation

- `tools/tests/test_runtime_accessibility_diagnostic_index_capacity_performance_contract.py`:
  `3/3`.
- `python -m py_compile` for the contract: pass.
- Lower duplicate/order regression and ignored Release marker are wired in
  `zircon_runtime/src/ui/accessibility/diagnostics/index_dedup_tests.rs`.
- Scoped Rustfmt, managed Windows Cargo/Release execution, and product
  accessibility p50/p95/p99 evidence remain pending under the shared external
  admission gate. No standalone Cargo process was started and this session
  does not poll the coordinator; tooling production remains deferred for the
  later Rust migration.

## Source fingerprints

- Source SHA-256: `B87B9833B4AD109CFFFC67070ABCA61FB5A78BA5C5D018D5876CDB9E51251A54`.
- Contract SHA-256: `2F00A2AC114DBE25D68CB1334C5286B5CEB84A8A918DD6CEEE5EB6E8EC6B7B98`.
