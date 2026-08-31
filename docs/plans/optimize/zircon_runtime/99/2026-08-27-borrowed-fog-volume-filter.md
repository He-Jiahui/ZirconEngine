---
title: Runtime99 Borrowed Fog Volume Filter
category: zircon_runtime
report_id: Runtime99-borrowed-fog-volume-filter-2026-08-27
date: 2026-08-27
session_id: root-runtime99-borrowed-fog-volume-filter-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime99 Borrowed Fog Volume Filter

## Scope

`AdvancedLightingExtract::fog_volumes_for_layers` previously cloned every matching
`FogVolumeData`, including its owned render-layer mask, into a new vector. The helper now returns
an order-preserving iterator over borrowed records and performs only the existing layer
intersection predicate.

The focused Rust test verifies that the matching item is the same record stored in the extract,
not an equivalent clone, and that the iterator ends after the one visible item. Filtering order,
empty results, and render-layer semantics are unchanged.

The current froxel media-inject production path already fuses layer filtering with GPU DTO
conversion and does not call this helper. This slice removes the remaining allocation-prone
public API from the Runtime99 source boundary; the benchmark is a call-level comparison and is not
claimed as an additional current-frame product speedup.

## Performance Evidence

The isolated optimized Rust model filters 65,536 matching records. Each record owns a render-layer
mask, so the legacy path measures the full deep-clone cost. It runs 21 alternating sample pairs
with two rounds per sample and was compiled with `rustc +1.94.1 -O` on Windows.

| Metric | Clone and collect | Borrowed iterator | Change |
|---|---:|---:|---:|
| Allocator calls | 65,551 | 0 | -100.000% |
| Cumulative requested bytes | 9,961,184 | 0 | -100.000% |
| Cloned fog-volume records | 65,536 | 0 | -100.000% |
| P50 for two rounds | 17,705,000 ns | 773,000 ns | -95.634% |
| P95 for two rounds | 20,439,300 ns | 1,011,000 ns | -95.054% |

Model source:
`.codex/state/session-coordinator/runtime99-borrowed-fog-volume-filter-model.rs`.

## Contracts And Validation

- `tools/tests/test_runtime99_borrowed_fog_volume_filter_performance_contract.py` locks the
  borrowed iterator signature, source-order layer predicate, absence of clone/materialization, and
  Rust reference-identity coverage.
- TDD RED produced three expected source-contract failures against the old `Vec` implementation;
  the implemented contract passes 3/3.
- Python bytecode compilation, scoped `rustfmt +1.94.1 --edition 2021 --check`, and scoped
  `git diff --check` pass.
- The post-implementation release model passes zero-allocation and P50/P95 reduction gates.
- Cargo compilation and the focused Rust behavior test remain pending in the next managed
  asynchronous coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime99 still owns prepared compact light tables, persistent fog-volume GPU records, dirty-range
uploads, stable identities, multi-view budgets, history validity, authoring, diagnostics, and
product qualification. This slice only removes the residual clone-and-collect layer-filter helper.
