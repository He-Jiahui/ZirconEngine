---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-28-menu-interaction-published-row-authority.md
  - docs/plans/optimize/zircon_editor/01/2026-09-13-menu-route-scratch-reuse.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/732-menu-parent-path-reuse.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_new.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_project_route.rs
tests:
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_project_route.rs
  - tools/tests/test_editor_menu_route_scratch_reuse_performance_contract.py
---

# Editor menu popup-route scratch reuse

Popup hit projection previously allocated a reserved path vector for every
pointer route, even though the recursive path is only a temporary working
buffer. `HostMenuPointerBridge` now retains that buffer, clears it at the start
and end of projection, and reserves only when a deeper open submenu exceeds
the previous high-water capacity. Route intents continue to own cloned paths,
so event lifetime and action identity are unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 popup route projection | Reuse bridge-owned recursive path scratch while preserving child-first hit semantics and owned route outputs. | RED/GREEN source contract, lower Rust capacity regression, Rustfmt/source checks; managed Cargo/Release allocation and latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

Projection remains `O(D)` in open submenu depth. Stable depth no longer allocates
the recursive working vector per event; route-intent path cloning and any
genuine capacity growth are intentionally retained. This slice does not claim
the full menu interaction path is allocation-free and does not alter the
published-row, keyboard, or popup-surface authorities.

## Local evidence

- The focused contract was RED against the event-local `popup_item_path` call
  and GREEN after the bridge-owned scratch field, constructor initialization,
  mutable projection signatures, and clear/prepare lifecycle were added.
- The Rust regression checks pointer/capacity stability after repeated scratch
  preparation at nested and shallow depths.
- The focused Editor menu batch passes `196/196` across 48 modules. The
  one-process non-tooling Runtime/Editor performance-plus-pressure loader
  covers `353` modules and passes `1352/1352` in `11.319s`, with zero load
  failures. Rustfmt (with import reordering disabled to preserve the existing
  foreign import edit), Python compilation, and scoped diff checks pass.
- Current source hashes and the ignored Release benchmark identifier are
  recorded in the linked optimize child.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

The existing owner-attributed Windows Release admission remains deferred by the
external dirty `E:\Git\zr_vm` worktree gate. No new coordinator request or
status query is issued for this slice. Keep this record at
`implemented_pending_validation` until the next batched Runtime/Editor
validation verifies compilation, route parity, allocation behavior, and menu
input latency.
