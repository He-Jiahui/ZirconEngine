---
title: Editor14 animation timeline projection capacity
category: zircon_editor
report_id: Editor813-animation-timeline-projection-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor813 · animation timeline projection capacity

## Scope

`project_sequence_timeline` rebuilt every visible animation timeline by
collecting the flattened track iterator and each track's key iterator into
growth-based vectors. The projection is a refresh hot path, so a dense
sequence paid repeated reallocations for both the outer track list and every
key list.

## Implementation

- Count the bounded source track list before projection and reserve the exact
  outer `TimelineTrackView` capacity.
- Materialize tracks with direct binding/track loops, preserving source order,
  path-derived IDs, value-kind classification, timeline range/playback fields,
  and empty section vectors.
- Reserve each key list from `track.channel.keys.len()` before appending the
  same formatted key IDs, times, and labels.
- Add a lower Rust empty-projection regression and the ignored
  `EDITOR813_ANIMATION_TIMELINE_PROJECTION_CAPACITY_BENCH_V1` Release marker.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the previous
`flat_map(...).collect()` projection and GREEN after the bounded direct-append
shape was added. A deterministic 4,096-track sequence with 64 keys per track
models `20,491` geometric growth events for the old outer/key collectors and
`0` for the exact reservations. This is allocation-shape evidence only; it is
not allocator, RSS, CPU, or product timeline p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_animation_timeline_projection_capacity_performance_contract.py`
  (`4/4`).
- The combined Runtime/Editor focused batch (Runtime804/807/808/809 and
  Editor805-814) passes `61/61` in one process with zero failures, errors, or
  skips.
- The strict non-tooling performance/pressure batch (tooling, export, and
  coordinator paths excluded) loads `683` files and passes `2634/2634` tests
  in `31.255s`, with zero failures, errors, or skips.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.
- The lower Rust semantic regression and ignored Release marker are wired in
  `zircon_editor/src/ui/animation_editor/session/timeline_foundation.rs`.
- The existing animation-session projection tests remain the semantic oracle;
  no runtime/compiler or tooling contract is changed by this slice.
- Managed Cargo/Release and animation timeline allocation/product percentile
  evidence remain pending; tooling production remains deferred for the later
  Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/animation_editor/session/timeline_foundation.rs` | `3642393A5DE3F9E461B1FC4D4FB28BDC65A1F104539D8C78868469CBD5C10F5C` |
| `tools/tests/test_editor_animation_timeline_projection_capacity_performance_contract.py` | `124580BE8E387055C2AA37E775CBB54D6D1CAE57E09E7557ED01AEF199A3ACC7` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending`
until the owner-attributed Windows Release batch compiles the current Editor
tree, runs the lower regression and ignored marker, and supplies animation
timeline allocation and product p50/p95/p99 evidence. No coordinator status is
polled by this session.
