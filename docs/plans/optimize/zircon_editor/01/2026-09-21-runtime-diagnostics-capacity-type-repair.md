---
title: Editor Runtime Diagnostics Capacity Type Repair
category: zircon_editor
report_id: Editor878-runtime-diagnostics-capacity-type-repair-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_preserved
---

# Editor878 Runtime Diagnostics Capacity Type Repair

## Failure receipt

The one bounded v7 coordinator reconciliation showed the current Runtime package
build passing, then the Editor package failing with four E0689 diagnostics in
`detail_item_capacity`: the untyped integer initializer could not select a
concrete receiver type for `saturating_add`, including chains that later add
`usize::from(...)`. App validation was therefore not reached.

## Support-first repair

The lower shared owner is the Editor818 capacity helper, not its pane caller or
the app package. Its accumulator now starts as `1usize`, matching the helper's
`usize` return type and every `Vec::with_capacity` consumer. Base/render/error/
profiling counts, item order, exact 1/11 bounds, output schema, and the ignored
performance marker remain unchanged.

## TDD and local evidence

- A new explicit-type assertion in the existing Editor818 contract was observed
  RED at `1/4`, reproducing the v7 source shape, then GREEN at `4/4` after the
  lower repair.
- Exact-file Rustfmt and scoped `git diff --check` pass.
- The existing lower default/dense semantic test and
  `EDITOR818_RUNTIME_DIAGNOSTICS_DETAIL_CAPACITY_BENCH_V1` marker remain wired.
- The repair was submitted with Editor875–877 in the asynchronous v8
  current-source multi-task batch; it was not submitted alone.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs` | `D2F4A3D94699E4F52C15507BA271B88823536D0335EFF352575F3FC7B2007380` |
| `tools/tests/test_editor_runtime_diagnostics_detail_capacity_performance_contract.py` | `74BA8E460394A8015C91B8A35896E5D76D336B2980643AED8CC69CC0AA941D03` |

## Acceptance boundary

v7 proves the old snapshot fails Editor compilation; the local source contract
proves only that the ambiguous shape is removed. Keep this record pending until
the next combined current-source Windows lane compiles Runtime, Editor, and App.
Release, allocator, ignored-marker, and Runtime Diagnostics product p50/p95/p99
evidence also remain pending.
