---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-28-menu-interaction-published-row-authority.md
  - docs/plans/optimize/zircon_editor/01/2026-09-13-menu-parent-path-reuse.md
related_records:
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
  - docs/plans/astra/features/editor/731-hierarchy-index-rebuild-reuse.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_item_tree.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_move.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_scroll.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/host_menu_pointer_bridge_handle_click.rs
tests:
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_item_tree/capacity_tests.rs
  - tools/tests/test_editor_menu_parent_path_reuse_performance_contract.py
---

# Editor menu pointer parent-path capacity reuse

The menu pointer bridge previously allocated a fresh parent `Vec<usize>` for
each menu-item move, scroll, and click route. The bridge now compares the
borrowed route prefix and copies it into the retained `open_submenu_path`
buffer, preserving its capacity. The allocating `parent_path` helper is gone;
route ordering, rebuild decisions, action identity, and empty/single-item path
behavior remain unchanged.

## Plan completion list

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Editor01 menu pointer path handling | Reuse retained parent-path storage and avoid fresh event-local parent vectors. | RED/GREEN source contract, lower Rust capacity regression, scoped Rustfmt/parse and diff checks; managed Cargo/Release allocation and latency evidence remains pending. | implemented_pending_validation |

## Complexity and allocation boundary

Parent-prefix work remains linear in route depth, but stable routes no longer
allocate a new vector or drop the prior state buffer. The helper still grows the
buffer when a genuinely deeper route exceeds retained capacity. This slice does
not change the separate full-row materialization, route-index hashing, or popup
surface rebuild costs called out by the published-row authority review.

## Local evidence

- The focused contract was RED against `parent_path(path) -> Vec<usize>` and
  GREEN after all three pointer handlers used `reuse_parent_path` with a
  borrowed-prefix comparison where a rebuild decision is needed.
- The Rust regression checks parent values for nested, single-item, and empty
  paths and verifies the destination allocation remains stable.
- Current source SHA-256 values are recorded in the linked optimize child
  `2026-09-13-menu-parent-path-reuse.md`.
- Focused parent-path contracts pass `2/2`; the refreshed one-process
  non-tooling Runtime/Editor performance-plus-pressure loader covered `352`
  modules and passed `1349/1349` tests in `10.228s`, with zero load failures.

This is source/contract evidence, not managed Cargo execution or product CPU,
allocator, RSS, or p50/p95/p99 acceptance evidence.

## Managed acceptance gate

The existing owner-attributed Windows Release admission remains deferred by the
external dirty `E:\Git\zr_vm` worktree gate. No new coordinator request or
status query is issued for this slice. Keep the record at
`implemented_pending_validation` until the next batched Runtime/Editor
validation verifies compilation, route parity, allocation behavior, and menu
input latency.
