---
title: Editor scene-layer damage and paint-state protected routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-scene-layer-damage-and-paint-state-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host paint_workbench_renderer/scene_layers{.rs,/**}`
- 10/10 Rust files source-reviewed. Scene composition is still immediate fixed fan-out; floating
  panes prepare three paint snapshots before routing, componentized chrome repeats focus/range work,
  page overflow lacks an owner damage gate, and no layer command ranges are retained. M1 adds exact
  damage/lazy-state gates: off-damage floating state `3 -> 0`, both-clip chrome focus `2 -> 1`,
  off-popup visible-row/text work `V -> 0` (focused GREEN 3/3; owned contracts GREEN 74/74).
  M0/M2-M6 counters/context/ranges/index/profile/power acceptance remain pending.

Do not add these files to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to MVP workbench scene composition. Record layer visits/rejects, state preparations,
template queries/visited rows, logical/visited floating windows, command-range rebuild/reuse bytes,
CPU/allocation/RSS/latency/context switches, WPR power/energy and RenderDoc draw/GPU/pixel/text parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of immediate scene fan-out, paint-time componentized graph fallback and duplicate layer
builders after the typed retained scene plan becomes authoritative.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own typed scene/overlay generations, exact layer/shadow/modal bounds, one lazy paint context and stable
back-to-front retained command ranges.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared invalidation reasons, prepared render-list ranges and compact text/image/interaction patch
contracts consumed by the editor scene plan.

## Acceptance handoff

The handoff requires 10/10 post-change fingerprints, focused and managed Rust behavior tests, the
node/window/overlay/damage/state matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc
draw/GPU/pixel/text parity, milestone commit and quantified WeCom notification. Protected ledgers
remain unchanged until then.
