---
title: Runtime11A Arranged Visibility Resolution Scratch Reuse
category: zircon_runtime
report_id: Runtime11A-arranged-visibility-scratch-reuse-2026-09-10
date: 2026-09-10
session_id: root-astra-optimize-20260909
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime11A Arranged Visibility Resolution Scratch Reuse

## Scope

This slice removes the per-rebuild allocation of the iterative visibility resolver's state
and ancestor-path vectors. It does not change inherited visibility, cycle/missing-parent
fail-closed behavior, sorted node publication, or the render-extract consumers.

## Implementation

- `UiArrangedVisibilityIndex` retains `resolution_states` and `resolution_path` beside the
  published node-id/bitset authority.
- Each rebuild clears and resizes the state buffer, clears the path buffer, and reuses warm
  capacity. A high-water buffer is replaced before the next rebuild when its capacity exceeds
  twice the current arranged node count, so a transient large tree cannot permanently inflate
  the Surface-owned index while small topology changes avoid reallocating on every frame.
- Equality compares only the published visibility authority; resolver scratch is an internal
  implementation detail. Cloning the index likewise copies only published data and starts with
  empty resolver scratch, avoiding a full scratch copy when a `UiSurface` is cloned.

## Regression and Local Evidence

- The Rust regression exercises a 128-node reversed chain, verifies warm rebuild pointer and
  capacity stability for both scratch buffers, verifies one-node shrink bounds both capacities,
  and checks cloned indexes omit scratch while preserving visibility; its Cargo execution remains
  in the managed testing stage.
- Existing arranged-visibility inheritance and fail-closed regressions remain unchanged.
- The post-change combined Runtime UI arranged-visibility/Taffy/layout plus adjacent Editor
  asset contract batch passed `86/86`; Python syntax and scoped Rustfmt checks also passed.
- A scoped check found no trailing whitespace in the new source and plan records; the shared
  async log retains six older malformed Hub lines and was not rewritten.

These are source and contract checks. Managed Runtime Cargo and a Windows Release workload
with allocation/time p50/p95/p99 remain pending; no product performance acceptance is inferred.

## Remaining Parent Work

Runtime11A still owns the broader UI driver, window/input lifecycle, retained layout graph,
multi-surface composition, and product/native timing acceptance. This record does not close
those parent gaps.
