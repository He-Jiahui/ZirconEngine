---
title: Editor05 Inspector Command Buffer Capacity
category: zircon_editor
report_id: Editor806-inspector-command-capacity-2026-09-18
date: 2026-09-18
session_id: root-runtime-editor-async-optimization-20260918
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor806 · Inspector command-buffer capacity

## Scope

`EditorState::apply_inspector_changes` can emit up to four built-in
Name/Parent/Translation/Scale commands for each selected node before any
dynamic component fields are appended. The command vector previously started
empty, so a changed multi-selection paid geometric growth even though the
built-in lower bound was known. This slice reserves that lower bound lazily
after the first effective command, so a no-op Apply does not reserve the
command buffer; dynamic field overflow keeps the existing vector semantics.

## Implementation

- On the first effective command, reserve `selected.len() * 4` with saturating
  arithmetic; the existing `commands.is_empty()` gate keeps no-op command-buffer
  reservation lazy.
- Keep the empty-selection guard before target-ID materialization and before
  any possible reservation.
- Preserve per-target reflected reads, command order, transaction mode,
  no-command return, and dynamic-field append behavior.
- Do not claim the parent Inspector05 M0/M1 property-authority or
  changed-path work is complete; this is allocation-shape evidence only.

## Deterministic model

For 1,024 selected nodes, the four built-in updates require 4,096 command
slots. Starting from `Vec::new()` produces geometric growth events once a
change is admitted; the lazy reservation produces zero growth events for that
fixed built-in portion. A no-op Apply takes the empty command-buffer branch and
performs zero reservations; the existing reflected-update preparation is out of
scope. Dynamic component updates may still grow the vector when present.
This is allocation-shape evidence, not allocator, RSS, CPU, or product
Inspector p50/p95/p99 evidence.

## Local evidence

- TDD source/model contract: `4/4` after an intentional RED baseline.
- Lower source regression `inspector_command_buffer_reserves_builtin_update_slots`
  is wired in the existing EditorState test module.
- Exact-file Rustfmt passes for the touched Rust source.
- The merged non-tooling Runtime/Editor batch covers `872` files and passes
  `3673/3673` tests with zero failures, errors, or skips in `52.370s` after
  this slice; managed Cargo/Release and product percentile evidence remain
  pending.

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed Windows Release batch compiles the current Editor tree,
runs the Inspector lower source regression, and supplies allocation plus
product Inspector latency evidence. Tooling production remains deferred for
the later Rust migration.
