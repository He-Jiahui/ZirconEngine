---
title: Editor01 Viewport Toolbar Cache Remap and Route-Key Ownership
category: zircon_editor
report_id: Editor01-viewport-toolbar-cache-remap-ownership-2026-09-12
date: 2026-09-12
session_id: root-astra-optimize-20260909
plan_sources:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/01/2026-08-30-viewport-toolbar-pointer-surface-reuse.md
  - docs/plans/optimize/zircon_editor/01/failure-2026-08-23-editor01-viewport-toolbar-cache-signature-move.md
related_records:
  - docs/plans/astra/features/editor/32-viewport-toolbar-stable-topology-cache-pruning.md
  - docs/plans/astra/features/editor/34-viewport-pointer-delta-contract-repair.md
  - docs/plans/astra/features/editor/664-20260911-editor-ui-optimization-batch-completion.md
implementation_files:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/surface_frame_cache.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/viewport_toolbar/surface_frame_cache.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs
  - tools/tests/test_editor_retained_viewport_toolbar_pointer_generation_performance_contract.py
  - tools/tests/test_editor_viewport_toolbar_pointer_surface_reuse.py
  - tools/tests/test_editor_menu_pointer_resize_pressure.py
  - tools/tests/test_editor_pane_surface_retention_pressure.py
  - tools/tests/test_editor_workbench_projection_commit_performance_contract.py
  - tools/tests/test_editor_template_surface_root_iteration_performance_contract.py
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor01 Viewport Toolbar Cache Remap and Route-Key Ownership

## Scope

This slice closes the ownership repair in the viewport-toolbar prelayout cache. A
same-size route change now remaps the retained `SurfaceFrameSignature` in place;
it no longer materializes a temporary `Vec<Option<String>>` before replacing the
signature's node payload. The route projection visit count and existing cache
counters remain observable, and a changed mapping still rebuilds only the
published `UiSurfaceFrame`.

The retained route key also owns one reusable `Vec<String>` per cache entry.
Updates rewrite existing string buffers in place, append only newly required
entries when the route grows, and drop the key on `None` as before. Initial projection construction
reserves the projection node upper bound before filtering disabled or unrouted
nodes. These changes preserve route ordering, hit-control mapping, frame identity
on stable hits, and the typed fallback to a full projection on a miss.

## Plan completion list

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Editor01 / viewport-toolbar cache | In-place hit-control remap, retained route-key capacity, and bounded signature construction | implemented_pending_validation | Two focused Rust regressions cover route-key reuse/clear (including long-string capacity) and in-place remap visit/change semantics. The six-module Editor cache/pointer batch passed `43/43` in `0.813s`; scoped Rustfmt parsing passed. |

## Complexity and ownership boundary

| Path | Previous shape | Current shape |
| --- | --- | --- |
| Same-size route remap | map every node into a temporary optional-string vector, compare, then replace | visit retained nodes once and update each hit ID in place |
| Stable route-key update | allocate a fresh owned string vector and string buffers | rewrite entry-owned strings in place and append only new entries |
| First projection signature | collect into an unbounded vector | reserve the projection node upper bound, then filter in one pass |
| Structural/layout miss | full projection and frame construction | unchanged; the existing conservative fallback remains authoritative |

The direct remap keeps the old closure order and visit count. It does not change
which route or control IDs are accepted, and it does not claim that the per-node
`String` values returned by the caller's mapping closure are allocation-free.

## Local verification

- The targeted Editor retained-toolbar, pointer-surface, menu-resize,
  pane-retention, workbench-projection, and template-surface-root contracts ran
  together with the Runtime layout/surface contracts as one batch: `106/106` passed in
  `1.213s` (Python unittest time); the Editor-only subset remains `43/43`.
- The refreshed combined Runtime/Editor performance-plus-pressure batch loaded
  `536` modules and passed `1993/1993` in `12.193s` after the subsequent Runtime712 summary
  extension; the Editor cache source remains included in that current-source sweep.
- After Runtime714's defensive text-decoration fallback, the same combined batch was rerun and
  passed `1993/1993` across `536` modules in `12.015s`; a later single-invocation rerun after
  the shared Runtime text source-map/caret cache change completed in `8.398s`.
- `rustfmt --edition 2021 --emit stdout` parsed the production owner without
  error, and the scoped `git diff --check` passed.
- The existing current-source Runtime/Editor performance-plus-pressure baseline
  remains `535` modules and `1991/1991` tests; this record treats it as batch
  context, while the fresh `43/43` result is the evidence for this slice.
- Tooling source was not changed. The broad non-tooling discovery still has the
  previously recorded unrelated boundary/naming/tech-stack failures.

## Validation boundary

This record remains `implemented_pending_validation`: managed Windows Cargo,
Editor Release allocation/time evidence, and viewport-toolbar product
p50/p95/p99 measurements have not been produced. The external
`E:\\Git\\zr_vm` dirty-worktree admission gate previously stopped the owner
ticket before Cargo started; no coordinator status was polled or re-submitted
for this slice. The open failure handoff remains open until that managed lane
can verify the current source.
