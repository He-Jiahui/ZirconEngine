---
title: Editor75 keyframe lane filtered-window capacity
category: zircon_editor
report_id: Editor808-keyframe-lane-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor808 · keyframe lane filtered-window capacity

## Scope

`keyframes_in_range` filters a caller-owned key slice into borrowed timeline
rows. The previous `filter(...).collect()` path began with no capacity even
when a dense visible window was expected. The new path keeps the empty-range
allocation-free behavior, lazily reserves the known input upper bound on the
first match, and appends in the original order without copying keyframes.

## Implementation

- Keep `Vec::new()` until the first key inside the requested range.
- On the first match, reserve `keys.len()` as the bounded output upper bound;
  subsequent matches push into the retained buffer.
- Preserve borrowed `&TimelineKey` output, range predicate, source order, and
  empty-result behavior. No timeline document, selection, cache, or authority
  contract changes.
- Add a lower source regression and the ignored
  `EDITOR808_KEYFRAME_LANE_CAPACITY_BENCH_V1` marker for the managed Release
  lane.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the old
`filter(...).collect()` implementation and GREEN after the lazy bounded loop
was added. For a 4,096-key dense window, the zero-capacity model reports
geometric growth while the first-match bounded model reports zero growth. An
empty range retains zero reserved capacity.

This is allocation-shape evidence only; it is not allocator, RSS, CPU, paint
latency, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_keyframe_lane_capacity_performance_contract.py`
  (`4/4`).
- Lower Rust source regression and ignored Release marker are wired in
  `keyframe_lane.rs`.
- Exact-file Rustfmt and Python compilation pass. The merged current-worktree
  non-tooling batch loads `875` files and passes `3685/3685` tests with zero
  failures, errors, or skips in `48.506s`; managed Cargo/Release and product
  timeline percentile evidence remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/timeline/keyframe_lane.rs` | `E52D9C2C037A9354AD9DCFDD970F3D6F7E0E12DD464225BA2E82C9B91F601F65` |
| `tools/tests/test_editor_keyframe_lane_capacity_performance_contract.py` | `E9FDA0F45392CF02F445B4CB06765DBB0F6742AADCFB82EE09A17394C0410434` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Editor tree, runs the lower regression and ignored marker,
and supplies timeline allocation and product p50/p95/p99 evidence. Tooling
production remains deferred for the later Rust migration.
