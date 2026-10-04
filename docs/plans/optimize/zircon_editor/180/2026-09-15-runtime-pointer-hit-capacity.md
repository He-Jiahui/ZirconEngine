---
title: Editor180 Runtime Pointer Hit Capacity
category: zircon_editor
report_id: Editor180-runtime-pointer-hit-capacity-2026-09-15
date: 2026-09-15
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor180 · Runtime pointer hit capacity

## Scope

The Editor viewport runtime-picking adapter receives two bounded candidate sources:
the retained stacked UI hits and renderer-visible candidates. It previously collected the
first source into a zero-capacity `Vec` and then extended it with the second source, allowing
geometric growth on every multi-source pointer query. The adapter now reserves the saturating
sum of both source lengths once, while preserving candidate filtering, source order, empty-hit
behavior, and the existing Runtime hit-record contract.

## Implementation

- Derive one overflow-safe upper bound from `stacked.len()` and
  `renderer_candidates.len()`.
- Build the hit-record vector with that bound and append stacked hits before renderer hits.
- Keep the empty result as `Vec::new()` after filtering, so no `PointerHits` wrapper is published
  when every candidate misses or the sources are empty.
- Keep the broader ordered-hit and visibility authority work in the existing Editor180/Runtime47
  plans; this is only the allocation slice.

## Regression and performance contract

The lower module
`zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter/hit_capacity_tests.rs`
covers empty, combined, and overflow-safe capacity bounds. Its ignored release benchmark emits
`EDITOR765_RUNTIME_POINTER_HIT_CAPACITY_BENCH_V1` and compares the legacy geometric-growth model
with the pre-sized model over paired samples. The Python source contract is
`tools/tests/test_editor_runtime_picking_hit_capacity_performance_contract.py`.

Current source fingerprints: `runtime_picking_adapter.rs`
`9C80BBD04C83C7A13A1D7C78361C17259DBB258B4802D82C884C3FD746227715`, lower regression
`DCD5AFDF5CC14CF8ADAD4AEE6FB7442C88C1BAE3A78743D7A28861AAA73C02EA`, and Python contract
`0D367B34F196A0352E859034A6D9922DC7C8C16C4115BF825A97E16F90EA66A2`.

The deterministic 32,768 stacked + 4,096 renderer model grows the legacy
vector 17 times to capacity 65,536, while the optimized path reserves exact
capacity 36,864. This is allocation-shape evidence only; managed timing and
product percentile gates remain separate.

## Local receipt

The focused source contract passes `3/3`; the lower Rust regression and ignored benchmark are
wired and Rustfmt-clean. A single-process current-source Runtime/Editor performance-contract
batch covers `548` modules and passes `1963/1963` with zero failures, errors, or skips; the
focused pointer/navigation/style/capacity subset passes `99/99`. These are local source/model
receipts, and the exact batch is also recorded in the Astra completion ledger and asynchronous
validation log.

## Validation boundary

Managed Windows Cargo/Release execution, allocator counts, and product pointer p50/p95/p99 remain
coordinator-owned and pending. This record does not claim those gates or complete the broader
Editor180/Runtime47 picking contracts. Tooling production work remains deferred for the later Rust
migration.
