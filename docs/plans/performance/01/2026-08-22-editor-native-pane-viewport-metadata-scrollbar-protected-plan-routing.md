---
title: Editor native-pane viewport, metadata and scrollbar protected routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-native-pane-viewport-metadata-scrollbar-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host paint_workbench_renderer/native_panes/**`
- 20/20 Rust files source-reviewed. Pre-M1 Hierarchy rediscovered viewport twice and metrics per
  visible row; asset tree count/hover still scan and parse the complete template model despite typed
  metadata, and subview scrollbars/row/text/diagnostics commands lack retained ranges. M1 changes
  anchor scans `2 -> 1` and metrics snapshots `V+2 -> 1` (focused GREEN 4/4; owned contracts GREEN
  69/69). M0/M2-M6 metadata/range/backend/profile/power acceptance remain pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to basic hierarchy/asset/viewport/diagnostics pane paint. Record anchor/count/id scans,
metadata reads, logical/visible/visited rows, scrollbar descriptor/track/thumb work, command/text/
overlay rebuild/reuse bytes, CPU/allocation/RSS/latency/context switches, WPR power/energy and
RenderDoc draw/GPU/pixel/text parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own removal of paint-time template scans/string routing and immediate pane row/scrollbar/overlay
reconstruction after typed source metadata and retained pane ranges migrate.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own typed pane/source/layout/resource generations, hierarchy/asset row indices, exact damage routing,
scrollbar descriptors and retained row/text/overlay/view ranges.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared prepared render-list/text/image resource contracts and canonical batch order consumed by
native pane ranges.

## Acceptance handoff

The handoff requires 20/20 post-change fingerprints, focused and managed Rust behavior tests, the
node/subview/hover/scroll/damage matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc
draw/GPU/pixel/text parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.
