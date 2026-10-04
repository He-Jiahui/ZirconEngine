---
title: Editor drag-session and world-surface generation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-drag-world-surface-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{drag_overlay,world_space}/**`
- 13/13 Rust files source-reviewed. All nodes currently parse world-only fields before activation;
  drag pointer updates rebuild static payload strings; downstream world submissions rescan/clone host
  candidates. M1 gates disabled world fields (10 -> 1 lookups, focused contract GREEN 2/2; owned
  contracts GREEN 42/42). M0/M2-M5 typed-generation/input/profile/power/render acceptance remain
  pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to MVP direct manipulation and viewport UI. Record world field lookups/build/copies,
drag static/dynamic work, pointer coalescing/edges, CPU/allocation/RSS/latency/power and GPU parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of flat drag/world host DTO fields and full-scene world candidate discovery after typed
session/surface generations have migrated all consumers.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own `DragSessionGeneration`, latest-wins move coalescing, ordered edge receipts and pointer capture.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Carry shared world-surface generations through host scene and viewport without full-node rescans or
owned submission copies.

## `docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`

Own typed drag/world component capability declaration and deletion of unrelated flat DTO defaults.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own retained world-surface content/revision contracts shared by layout, hit, input and render extract.

## `docs/plans/zircon_runtime/render/09-camera-render-ordering.md`

Own camera target, order, depth, billboard and scene/UI ordering consumption from one surface record.

## `docs/plans/zircon_runtime/render/14-2d-stack.md`

Own final world-space UI sprite/draw integration without a second CPU presentation authority.

## Acceptance handoff

The handoff requires 13/13 post-change fingerprints, managed component/submission/input tests, the
full node/world/payload/pointer matrix, current-source WPR/power artifacts on D/E/F, capture and
accessibility parity, RenderDoc world/drag draw parity, milestone commit and quantified WeCom
notification. Protected ledgers remain unchanged until then.
