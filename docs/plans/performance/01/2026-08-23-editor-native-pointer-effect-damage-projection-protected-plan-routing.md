---
title: Editor native pointer effect damage projection protected routing
date: 2026-08-23
status: routing_requested
owner_record: 2026-08-23-editor-native-pointer-effect-damage-projection-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Replace the matching portions of the obsolete 2026-07-17 pointer-damage coverage with one concise
`pending.md` module entry:

`zircon_editor retained-host native_pointer/{chrome_damage,close_prompt_damage,pane_button_damage,viewport_toolbar_damage}.rs + matching folders`
- 31/31 current Rust files source-reviewed. Typed chrome routes, visibility checks and local toolbar/
  dock damage foundations are retained. M0 borrowed model rows are applied and statically GREEN.
  Pending M1-M3 and M0 dynamic acceptance: typed action invalidation effects and retained owner/
  overlap lookup; multi-region propagation; scale/WPR/power/RenderDoc acceptance.

Do not add these files to `review.md` before M0-M3 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M3 to MVP pointer-to-present latency. Record reasons/owners, row visits/cloned bytes,
regions/useful/union/submitted area, floating overlaps, promotions, CPU p50/p95/p99, WPR context
switches/power and exact source/workload fingerprints.

## `docs/plans/performance/01/2026-08-23-editor-redraw-coalescing-damage-queue-architecture-review.md`

Consume typed effects and exact region sets at the redraw boundary; preserve regions/reasons through
external state, event-loop coalescing, retry and presenter.

## `docs/plans/performance/01/2026-08-23-editor-frame-paint-geometry-damage-region-set-architecture-review.md`

Own the shared bounded `DamageRegionSet`, overlap/promotion policy and useful-versus-union area
telemetry. Leaf pointer helpers must not invent another region-set type.

## `docs/plans/zircon_editor/editor_layout/18-input-response-and-hit-testing.md`

Own typed action invalidation effects, stable route owner ids and exact pointer-result damage semantics.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Own generation owner/frame lookup, floating z-order overlap projection and scene-range invalidation.

## `docs/plans/zircon_editor/editor/12-plugin-management.md`

Own plugin action invalidation-effect registration/validation and unload cleanup; no plugin control-id
prefix may decide damage scope at input time.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own reusable reason-coded invalidation and retained owner/range propagation shared with editor UI.

## Acceptance handoff

The handoff requires 31/31 post-change fingerprints, focused and managed Rust tests, window/tab/node/
action/region/placement/repeat/backend/scale matrices, same-executable WPR artifacts on D/E/F,
RenderDoc scissor/GPU/pixel parity, milestone commit and quantified WeCom notification. Protected
ledgers remain unchanged until then.
