---
title: Editor inspector, hierarchy and console retained-item generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-inspector-hierarchy-console-retained-item-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/{inspector_fields.rs,inspector_projection.rs,hierarchy_projection.rs,console_projection.rs,inspector_pane_tests.rs}`
- 5/5 Rust files source-reviewed; inspector duplicates template/payload/view/final property rows and
rediscovers field editors, hierarchy visible paint still remaps the full tree, and bounded console
visible paint still builds every line node plus generic surface rows; M1 scan/allocation cleanup and
M0/M2-M5 dynamic/profile/power/interaction acceptance remain pending. M1 removes per-property
lowercase/escape temporaries, collapses hierarchy template scans from 3 to 1 and console control
scans from 14 to 1, and reuses aligned console level count.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to inspector, hierarchy and console MVP work. Record item generations, rows/bytes
copied/patched, field classifications/ID encodes, console splits/node/surface rows, visible visits,
allocations, CPU, latency, RSS and energy across the specified scale matrix.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of inspector intermediate DTOs/raw type rediscovery, hierarchy full remaps and all-line
console node/surface expansion after retained item generations and shared visible indices are live.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own shared typed pane item generations and exact selection/schema/log/filter/scroll receipts through
the runtime/editor UI boundary.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own one visible row/hit/accessibility index per pane. Console, hierarchy and inspector pointer routes
must consume the same row window as paint.

## `docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md`

Carry selection, property value/schema, hierarchy filter/order, console append/filter and scroll
generations independently so narrow changes cannot trigger full pane reconstruction.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own O(V) visible-row pointer routing and exact item identities for inspector, hierarchy and console.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared item/tree generations, persistent rows and visible-range scheduling across runtime and
editor UI consumers.

## Acceptance handoff

The owner handoff requires 5/5 post-change fingerprints, managed focused and behavior tests, the full
property/hierarchy/console scale matrix, current-source WPR/power artifacts on D/E/F,
interaction/screenshots, RenderDoc parity where GPU output is relevant, milestone commit and
quantified WeCom notification. Shared ledgers remain protected until then.
