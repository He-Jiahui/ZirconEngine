---
title: Editor visual projection and transition activation protected plan routing
date: 2026-08-22
status: routing_requested
owner_record: 2026-08-22-editor-visual-projection-transition-activation-architecture-review.md
---

# Protected plan updates

## Performance ledgers

Keep one concise `pending.md` module entry:

`zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/{button_style.rs,surface_defaults/**,surface_metrics/**,visual_state/**,visual_style/**,transition_metadata/**}`
- 34/34 Rust files source-reviewed. Generic nodes perform 29 empty visual-state lookups and resolve a
  wide element/button style; inert transitions add 16 lookups plus discarded/retained strings.
  M1 reduces ordinary transition lookups 18 -> 2 and timing-string allocations 2 -> 0 (focused
  contract GREEN 2/2; owned contracts GREEN 47/47). M0/M2-M5 descriptor-generation/active-session/
  invalidation/profile/power/render acceptance remain pending.

Do not add these files to `review.md` before M0-M5 pass.

## `docs/plans/performance/01-mvp-performance-audit-and-optimization.md`

Attach M0-M5 to MVP retained editor presentation. Record style/transition lookups, descriptor builds,
dirty reasons, active sessions/wakeups, CPU/allocation/RSS/latency/context-switch/power and GPU parity.

## `docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md`

Own deletion of flat visual/transition DTO fields and duplicate layouts/retained resolvers after all
consumers migrate to typed generations.

## `docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md`

Own UI-thread interaction receipts, active transition-session scheduling, ordered start/finish edges
and zero idle animation wakeups.

## `docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`

Own compile-time role/default/variant/element descriptors and capability-specific optional records.

## `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`

Consume shared descriptor generations and apply interaction/transition patches without stable TOML
reprojection or whole-node rebuild.

## `docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`

Own typed paint/layout/accessibility invalidation reasons and the shared transition-spec/session
contract used by layouts and retained host.

## `docs/plans/zircon_runtime/render/14-2d-stack.md`

Consume active transition opacity/transform from render extraction without creating a second timing
authority; own RenderDoc pixel/draw parity only.

## Acceptance handoff

The handoff requires 34/34 post-change fingerprints, managed behavior tests, the full node/style/
transition/display matrix, current-source WPR/power artifacts on D/E/F, visual/accessibility parity,
RenderDoc transition draw parity, milestone commit and quantified WeCom notification. Protected
ledgers remain unchanged until then.
