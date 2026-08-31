---
title: Editor workbench menu visible-row and state-generation protected routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-workbench-menu-visible-row-state-generation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host paint_workbench_renderer/menus/**`
- 12/12 Rust files source-reviewed. Pre-M1 root/submenu popup rows prepared the complete item model
  before clipping and menu geometry rematerialized cloned/Arc-owned state per bar row; stable text is
  still remeasured/rebuilt. M1 changes rows `O(N) -> O(V)`, bar fallback state materializations
  `M+1 -> 1`, root popup `2 -> 1`, and visible-range formula owners `2 -> 1` (focused GREEN 4/4;
  owned contracts GREEN 65/65). M0/M2-M6 retained text/popup/range/profile/power acceptance remain
  pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to the basic editor menu path. Record logical/visible/visited rows, state materialization
and cloned bytes, text measure/layout/shaping, popup/range rebuild/reuse, CPU/allocation/RSS/latency/
context switches, WPR power/energy and RenderDoc draw/GPU/pixel/text parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own removal of full-model popup row preparation, per-row paint-state ownership and duplicate
immediate menu commands after retained menu/presentation generations migrate.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own menu source/state/layout/text generations, exact visible ranges, open popup stack, addressed
hover patches and retained row/menu-bar command ranges.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared prepared render-list/text artifacts and canonical batch order consumed by menu ranges.

## Acceptance handoff

The handoff requires 12/12 post-change fingerprints plus supporting shared-owner fingerprints,
focused and managed Rust behavior tests, the item/depth/viewport/scroll matrix, same-executable
WPR/power artifacts on D/E/F, RenderDoc draw/GPU/pixel/text parity, milestone commit and quantified
WeCom notification. Protected ledgers remain unchanged until then.
