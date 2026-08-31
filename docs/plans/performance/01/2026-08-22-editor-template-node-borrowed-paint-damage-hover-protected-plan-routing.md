---
title: Editor template-node borrowed paint, damage and hover protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-template-node-borrowed-paint-damage-hover-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor retained-host template_node_pipeline/** + template_nodes/**`
- 22/22 Rust files source-reviewed. Pre-M1 stable untransformed visible nodes were cloned and culled
  twice; damage rows still rebuild command payloads, hover clones whole option/menu row collections,
  and transforms consume the full node model. M1 changes stable node clones `V -> 0`, clip clones
  `V -> 0` and equivalent culls `2V -> V` (focused GREEN 3/3; owned contracts GREEN 59/59).
  M0/M2-M6 retained-range, addressed-hover, transform-instance, dispatch/profile/power acceptance
  remain pending.

Do not add these folders to `review.md` before M0-M6 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M6 to the shared MVP editor retained-node paint path. Record visited/culled/cloned/rebuilt/
reused nodes, commands and bytes, hover-row patches, dispatch counts, CPU/allocation/RSS/latency/
context switches, WPR power/energy and RenderDoc draw/GPU/pixel parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own removal of full-frame node/clip ownership conversion and duplicate command-list reconstruction
after retained node ranges and prepared render-list consumers migrate.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own immutable node generations, addressed interaction overlays, per-node retained command ranges,
compact transform/clip instances and granular dirty-range updates.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own shared prepared render-list generation, source/style/geometry/clip/resource generation contracts
and canonical cross-consumer batch order.

## Acceptance handoff

The handoff requires 22/22 post-change fingerprints, focused and managed Rust behavior tests, the
scale/damage/hover/transform matrix, same-executable WPR/power artifacts on D/E/F, RenderDoc draw/GPU
and pixel parity, milestone commit and quantified WeCom notification. Protected ledgers remain
unchanged until then.
