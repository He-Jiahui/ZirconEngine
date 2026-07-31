---
handoff_kind: failure
status: open
created_at: 2026-08-01
summary_slug: physics-debug-overlay-provider-missing
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_plugins/03-physics.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_plugins/03
related_code:
  - zircon_plugins/physics/editor/src/plugin.rs
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
tests:
  - physics editor provider registration and capability lifecycle tests
  - host toggle to shared viewport interaction extract product test
---

# Plugins03: Physics debug overlay lacks an executable provider

## Failure

The Physics editor registered `physics.debug_overlay.mode` as descriptor-only viewport metadata.
It supplied neither a `SceneModeRegistration` factory nor a
`ViewportOverlayProviderRegistration`, so the entry could never execute or publish collision
geometry. Editor05 removed this pseudo mode during the executable scene-mode hard cut; keeping a
PassThrough factory would only disguise the missing Physics behavior.

## Required Repair

- Register a Physics-owned `ViewportOverlayProviderRegistration` backed by the canonical debug
  geometry generation and capability lifecycle.
- Route the existing toggle operation to `ViewportCommand::ToggleOverlayProvider`; do not model a
  debug overlay as a base scene mode.
- Prove enabled/disabled extract publication through the host's shared render/pointer interaction
  extract, including stale-frame clearing.

## Forbidden Workarounds

No descriptor-only mode, empty PassThrough factory, fabricated geometry, global cache, direct
viewport mutation, compatibility alias, or test-only provider.

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-01 | Physics pseudo scene-mode hard cut | open | Editor05 executable registry migration removed `physics.debug_overlay.mode`; command/menu/view remain, but no production Physics provider exists. Plugins03 owns the canonical geometry/provider repair and product gate. |
