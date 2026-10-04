---
title: Editor48 UI delta reflection-patch projection capacity
category: zircon_editor
report_id: Editor810-ui-delta-reflection-patch-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor810 · UI delta reflection-patch projection capacity

## Scope

`EditorUiDeltaBatch::reflection_patches` flattens the node entries while
skipping barrier entries. The previous iterator chain exposed only a weak
size hint when a segment contained many patches, so a dense frame could grow
the returned vector geometrically. This slice reserves the exact existing
`node_delta_count()` bound before cloning the patch payloads.

## Implementation

- Reserve `node_delta_count()` once before the projection loop.
- Keep barrier entries out of the returned patch list and clone the same
  `UiReflectionNodePatch` values in their existing entry order.
- Preserve queue coalescing, barrier ordering, view provenance, and the public
  owned `Vec<UiReflectionNodePatch>` contract.
- Add a lower Rust regression and the ignored
  `EDITOR810_UI_DELTA_REFLECTION_PATCH_CAPACITY_BENCH_V1` marker for the
  managed Release lane.

## TDD and deterministic model

The Python source/model contract was intentionally RED against the old
`filter_map/flatten/collect` shape and GREEN after the bounded projection loop
was added. For a 4,096-patch dense segment, the zero/weak iterator-bound model
reports `11` geometric growth events while the exact node-delta bound reports
zero. Empty batches still request zero capacity.

This is allocation-shape evidence only; it is not allocator, RSS, CPU, or
product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_ui_delta_reflection_patch_capacity_performance_contract.py`
  (`4/4`).
- The combined recent Runtime/Editor slice (Runtime804, Runtime807, Runtime808,
  Editor805-810) passes `36/36` in one process with zero failures, errors, or
  skips.
- The strict non-tooling performance/pressure batch (tooling, export, and
  coordinator test paths excluded) loads `679` files and passes `2631/2631`
  tests in `44.473s` (unittest runner `42.451s`), with zero failures, errors,
  or skips.
- Lower Rust source regression and ignored Release marker are wired in
  `editor_ui_delta.rs`.
- Exact-file Rustfmt and Python AST checks pass. The strict merged
  current-worktree non-tooling batch receipt is recorded above; managed
  Cargo/Release and product UI-delta percentile evidence remain pending.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/editor_message/editor_ui_delta.rs` | `EA73A41490224083157300A196F81EF6D485BCAFB5B426C07195506DF3CE5439` |
| `tools/tests/test_editor_ui_delta_reflection_patch_capacity_performance_contract.py` | `B1CD416D5C744025156CDD14743F1996554B6956809EF5D1D286AD28DED31F25` |

## Acceptance boundary

Keep this record `implementation_complete` /
`managed_validation_pending` until the owner-attributed Windows Release batch
compiles the current Editor tree, runs the lower regression and ignored marker,
and supplies UI-delta allocation and product p50/p95/p99 evidence. Tooling
production remains deferred for the later Rust migration.
