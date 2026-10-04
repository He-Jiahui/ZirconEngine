---
title: Editor01 Viewport Toolbar Control Projection Capacity
category: zircon_editor
report_id: Editor799-viewport-toolbar-control-projection-capacity-2026-09-18
date: 2026-09-18
session_id: root-runtime-editor-async-optimization-20260918
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor799 · Viewport Toolbar Control Projection Capacity

## Scope

`ViewportToolbarPointerBridge::sync_surface_frame` already classifies an
unchanged hit-grid identity before projecting controls. When the source frame
changes but the retained toolbar topology remains stable, the bridge must still
materialize one `ViewportToolbarPointerControl` for each retained control to
compare action keys and frames. That output has an exact existing-control
bound, but previously began with a zero-capacity vector.

## Implementation

- Read the retained control count once as `existing_capacity`.
- Reserve that bound for the required `controls` projection before the hit-grid
  scan.
- Keep the sparse `changes` vector lazy: it is empty for a semantically
  unchanged projection and can be much smaller than the complete control set.
- Preserve the existing hit-grid authority, action-key/topology fallback,
  geometry delta construction, applied-frame cursor, and `NoChange` behavior.

## Deterministic performance model

For a stable 64-control projection, a simple zero-capacity doubling model grows
the mandatory controls vector seven times; reserving the known retained bound
has zero modelled growth events. This bounds only the mandatory control
projection. It deliberately does not preallocate the sparse geometry-change
collector, so no-change and low-`K` geometry updates do not pay a 64-entry
allocation. This is allocation-shape evidence, not measured CPU, RSS, or
input-to-present timing.

## Local evidence

- The source contract was RED before the retained-bound reservation existed,
  then GREEN at `6/6` after the implementation.
- Existing lower Rust regressions remain wired for stable frame identity,
  geometry-only publication, and topology fallback; managed Cargo has not run
  them in this shared checkout.
- The combined non-tooling Runtime/Editor performance-contract batch passes
  `2205/2205` across 598 modules in `6.538s`; exact Rustfmt for the eight owned
  Runtime/Editor files, Python AST, scoped diff, whitespace, and Wiki
  validation (`272/272`, one pre-existing metadata warning) also pass.

## Acceptance boundary

Keep this record `managed_validation_pending` until one owner-attributed Windows
batch validates Editor799 with the current Runtime200/205/802 work and supplies
Cargo, Release allocation, and viewport-toolbar product p50/p95/p99 evidence.
Tooling production remains deferred and this session does not poll the
coordinator.
