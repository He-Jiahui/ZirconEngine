---
title: Editor05 Empty Selection Fast Path
category: zircon_editor
report_id: Editor805-empty-selection-fast-path-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor05 Empty Selection Fast Path

## Scope

`EditorState::delete_selected` and `EditorState::apply_inspector_changes` both
materialized an owned selection `Vec` before discovering that the active selection
was empty. The change adds an empty-selection guard at each command boundary while
preserving the existing status line, `NoSelection` error, command payload, and
`prepare_non_gizmo_scene_action` ordering.

This is a narrow performance slice. It does not claim to solve the parent Inspector
plan's changed-property or multi-target authoring correctness work.

## Implementation

- `delete_selected` returns the existing `Nothing selected` result before entering
  the iterator/collection path.
- `apply_inspector_changes` keeps its preparatory validation first, then returns the
  existing `InspectorEditError::NoSelection` before collecting target IDs.
- Non-empty selections still produce the same owned `selected` vector and feed the
  existing delete/inspector transaction paths.

## Deterministic performance target

For 100,000 empty invocations of both commands, the old shape entered two owned-ID
collection paths per invocation. The new shape performs the borrowed emptiness checks
and enters zero collection paths, removing 200,000 unnecessary iterator/collection
attempts. The model is structural evidence, not product latency or allocator data.

## Local validation

- TDD source contract was RED before the guards and GREEN at `4/4` after the patch.
- The merged non-tooling Runtime/Editor contract batch loaded `871` files and passed
  `3669/3669` tests with zero failures, errors, or skips in `95.820s`; exact-file
  Rustfmt also passed. Managed Cargo, Release allocation, and Editor input
  p50/p95/p99 remain pending.
- Tooling production remains deferred as requested.

## Acceptance boundary

Keep this record at `managed_validation_pending` until the owner-attributed Windows
batch compiles the current Runtime/Editor source and supplies lower Rust, allocation,
and product percentile evidence. The parent Inspector M0-M5 plan remains open.
