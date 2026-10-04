---
title: Editor75 timeline track-list projection capacity
category: zircon_editor
report_id: Editor809-track-list-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor809 · timeline track-list projection capacity

## Scope

`project_track_list` materializes one retained row for each caller-owned track.
The previous iterator collection started with no capacity even though the
input slice gives an exact output upper bound. The new loop reserves that bound
once and preserves the existing row order and field projection.

## Implementation

- Reserve `tracks.len()` before the first row is appended.
- Keep the explicit source-order loop and clone only the existing owned row
  fields (`TrackId` and display name).
- Preserve lane-kind classification, key/section counts, empty-input behavior,
  and the borrowed input contract. No timeline authority or model semantics
  change.
- Add a lower source regression and the ignored
  `EDITOR809_TRACK_LIST_CAPACITY_BENCH_V1` marker for the managed Release lane.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the old
`collect()` implementation and GREEN after the bounded loop was added. For a
4,096-track dense projection, the zero-capacity model reports geometric growth
while the reserved-bound model reports zero growth events. Empty input keeps a
zero-sized allocation request.

This is allocation-shape evidence only; it is not allocator, RSS, CPU, paint
latency, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_track_list_capacity_performance_contract.py`
  (`4/4`).
- Lower Rust source regression and ignored Release marker are wired in
  `track_list.rs`.
- Exact-file Rustfmt and Python AST checks pass. The merged current-worktree
  non-tooling batch loads `876` files and passes `3689/3689` tests with zero
  failures, errors, or skips in `57.003s`; managed Cargo/Release and product
  timeline percentile evidence remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline/track_list.rs` | `733604EF81F880133DA37C893D15373983E049922B982C51FDB0834D343FF4E4` |
| `tools/tests/test_editor_track_list_capacity_performance_contract.py` | `7F03786C38F3C586A374AA0F0AAD1913870AC498C6F30FB5A419E4030ED2ED1B` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Editor tree, runs the lower regression and ignored marker,
and supplies timeline allocation and product p50/p95/p99 evidence. Tooling
production remains deferred for the later Rust migration.
