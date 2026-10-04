---
title: Editor activity-log projection capacity
category: zircon_editor
report_id: Editor840-activity-log-projection-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor840 · activity-log projection capacity

## Scope

The retained activity-log projection already owns exact record slices for both
incremental append and full rebuild. Both temporary line vectors previously
used iterator collection from zero capacity on the console refresh path.

## Implementation

- Reserve the exact entered-record count before incremental line projection and
  append in source order.
- Reserve `records.len()` before full projection and append the same line
  mapping directly.
- Preserve tail identity reuse, retained chunks, bounded logical-line
  clipping, source IDs, severity/action projection, filter behavior, and empty
  input semantics.
- Add a folder-backed lower order/capacity regression and the ignored
  `EDITOR840_ACTIVITY_LOG_PROJECTION_CAPACITY_BENCH_V1` Release marker.

No logging authority, snapshot generation, retention policy, or tooling
production code changes are included.

## Deterministic work model

For a dense 4,096-record projection, each zero-capacity collector models 11
geometric growth events; exact reservations change each to `11→0`. Empty
projections remain zero-capacity. This is allocation-shape evidence only, not
allocator, CPU, RSS, UI latency, or product p50/p95/p99 evidence.

## TDD and local evidence

- The Python source/model contract was intentionally RED against the two
  original iterator collectors and GREEN after direct bounded append loops and
  lower-test wiring were added (`3/3`).
- The lower Rust source/order regression and ignored Release marker are wired
  in `activity_log_console_projection/capacity_tests.rs`.
- Exact-file Rustfmt and Python compilation pass. The focused Runtime/Editor
  batch containing this slice and the adjacent recent slices passes `46/46`;
  the one-process broad non-tooling loader covers `933` modules and passes
  `3907/3907` tests in `62.574s`, with zero failures, errors, load errors, or
  skips.
- Managed Cargo/Windows Release, allocator, and activity-log product
  percentile evidence remain pending.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/activity_log_console_projection.rs` | `8BDFCA57DD990E39605DCA424E45F41D7209002A6B0D3D32D487B456D53CA591` |
| `zircon_editor/src/ui/workbench/activity_log_console_projection/capacity_tests.rs` | `D12CA094A85E715C4E9085ED1F882C16D37DA30846F65336E220E3505ED4928B` |
| `tools/tests/test_editor_activity_log_projection_capacity_performance_contract.py` | `26FC0F3655A4A6192752E3F9DA2DA5E9CB2DA960E990926C4265A641A624920A` |

## Managed acceptance gate

Keep this record at `managed_validation_pending` until the owner-attributed
batched Windows Release lane proves current-source compilation, snapshot
parity/allocation behavior, and the declared activity-log product p50/p95/p99
gates.
