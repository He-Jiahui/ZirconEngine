---
title: Editor runtime diagnostics detail projection capacity
category: zircon_editor
report_id: Editor818-runtime-diagnostics-detail-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor818 · runtime diagnostics detail projection capacity

## Scope

`detail_items` always emits one base line and conditionally emits a bounded
render-stat block, three subsystem error lines, and two profiling lines. The
new `detail_item_capacity` helper counts those same predicates before string
formatting, reserving the exact output bound while preserving item order,
wording, and empty/default behavior.

The change is limited to the retained Runtime Diagnostics pane projection. It
does not alter diagnostic source authority, status formatting, profile data,
or the UI payload schema.

## TDD and deterministic model

- The Python source contract was intentionally RED against the original
  `Vec::new()` collector and missing lower marker, then GREEN at `3/3` after
  the exact capacity helper and semantic regression were added.
- The lower regression checks the default one-item path, a dense eleven-item
  path, output order, and the exact helper bound. The ignored marker
  `EDITOR818_RUNTIME_DIAGNOSTICS_DETAIL_CAPACITY_BENCH_V1` is wired for the
  managed Windows Release lane.
- For the maximum eleven detail lines, the zero-capacity model performs three
  geometric growth events and the exact-count model performs zero (`3 -> 0`).
  This is allocation-shape evidence only, not allocator, CPU, RSS, or product
  percentile evidence.

## Local validation

- `tools/tests/test_editor_runtime_diagnostics_detail_capacity_performance_contract.py`:
  `3/3`.
- `python -m py_compile` for the contract: pass.
- Scoped `rustfmt --edition 2021 --check` for
  `zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs`:
  pass.
- Source SHA-256:
  `D2F4A3D94699E4F52C15507BA271B88823536D0335EFF352575F3FC7B2007380`
  (shared current-worktree hash after the Editor878 explicit-`usize` compile repair).
- Contract SHA-256:
  `74BA8E460394A8015C91B8A35896E5D76D336B2980643AED8CC69CC0AA941D03`
  (the contract is now `4/4` and guards the compile-relevant accumulator type).

## Managed validation boundary

Managed Windows Cargo/Release execution, ignored benchmark timing, allocator
evidence, and Runtime Diagnostics pane p50/p95/p99 evidence remain pending
under the shared external `E:\Git\zr_vm` admission gate. This session does not
poll or monitor the coordinator; tooling production remains deferred for the
later Rust migration.
