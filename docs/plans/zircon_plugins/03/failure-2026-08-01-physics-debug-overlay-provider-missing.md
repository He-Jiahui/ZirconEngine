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

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：Editor05 executable scene-mode hard cut current-source review
- 修复责任计划：`docs/plans/zircon_plugins/03-physics.md`
- 交接原因：Editor05 只能删除不可执行的伪 scene mode；Physics 调试几何、capability 生命周期和 overlay provider 必须由 Plugins03 提供。

## 失败现象与复现证据

The Physics editor registered `physics.debug_overlay.mode` as descriptor-only viewport metadata.
It supplied neither a `SceneModeRegistration` factory nor a
`ViewportOverlayProviderRegistration`, so the entry could never execute or publish collision
geometry. Editor05 removed this pseudo mode during the executable scene-mode hard cut; keeping a
PassThrough factory would only disguise the missing Physics behavior.

## 最低共享层根因

The old contribution contract treated toolbar metadata as proof of an executable mode. Plugins03
did not own a provider that converts canonical Physics debug geometry into the shared viewport
extract, so the UI descriptor had no production behavior behind it.

## 架构修复验收

- Register a Physics-owned `ViewportOverlayProviderRegistration` backed by the canonical debug
  geometry generation and capability lifecycle.
- Route the existing toggle operation to `ViewportCommand::ToggleOverlayProvider`; do not model a
  debug overlay as a base scene mode.
- Prove enabled/disabled extract publication through the host's shared render/pointer interaction
  extract, including stale-frame clearing.

## 禁止临时方案

No descriptor-only mode, empty PassThrough factory, fabricated geometry, global cache, direct
viewport mutation, compatibility alias, or test-only provider.

## 修复结果与回传

Open state: Editor05 已移除 `physics.debug_overlay.mode` 伪注册，Plugins03 尚未交付真实
`ViewportOverlayProviderRegistration`、共享 extract 产品证据或 focused Cargo GREEN。本记录保持
`open`，不得把“不可点击的伪入口已删除”误记为 Physics overlay 已完成。

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-01 | Physics pseudo scene-mode hard cut | open | Editor05 executable registry migration removed `physics.debug_overlay.mode`; command/menu/view remain, but no production Physics provider exists. Plugins03 owns the canonical geometry/provider repair and product gate. |
