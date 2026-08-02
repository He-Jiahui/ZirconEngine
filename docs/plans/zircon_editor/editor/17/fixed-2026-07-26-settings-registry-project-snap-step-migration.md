---
handoff_kind: fixed
status: fixed
created_at: 2026-07-23
summary_slug: settings-registry-project-snap-step-migration
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_workflow_node: M1.1
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/viewport/route_mapping.rs
  - zircon_editor/src/core/settings/
tests:
  - project snap-step SettingsRegistry resolve and persistence
  - viewport route observes resolved project values
  - User fallback and Session override do not mutate project settings
resolved_at: 2026-07-26
---


# Editor05: Project 吸附步进尚未迁入 SettingsRegistry

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：Editor17 M1.1 Project SettingsRegistry 落地后的 snap-step migration handoff
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：viewport snap-step 的事实源与场景交互接线属于 Editor05；Editor17 只提供 Project/User/Session SettingsRegistry 基础设施。

## 失败现象与复现证据

`SceneViewportSettings` 仍在 `scene/viewport/settings.rs:41-54` 直接持有 `translate_step`、`rotate_step_deg`、`scale_step`；默认值在 :67-82，retained viewport route 在 `route_mapping.rs:56-75` 直接循环这些字段。Editor05 计划已指定这些字段保持事实源，待 Editor17 落地后迁入 Project 域。

Editor17 的 Project 路径现为 `<root>/.zircon/settings.toml`，但 Editor05 尚未把吸附步进定义、加载和变更应用接到该层，因此工程间仍不能获得计划要求的持久化边界。

## 最低共享层根因

`SceneViewportSettings` 仍直接持有并向 route 暴露三个 snap-step 字段，尚未接入 SettingsRegistry 的 Project 层，形成平行持久化事实源。

## 架构修复验收

- Editor05 注册三个有限 Float Settings 项，Project 为持久层，User 为允许回退层，Session 仅作易失覆盖。
- 打开工程时从 `SettingsStore` Project 层应用解析值；编辑步进后仅更新该层，不把值写入场景源文件或 UI 私有文件。
- route/model 读取解析后的权威值，删除平行持久化状态；不改变现有循环与显示语义。
- 覆盖 Project 覆盖、User fallback、Session 覆盖、`<root>/.zircon/settings.toml` round-trip 和工程源文件 digest 不变。

## 禁止临时方案

- 不得将吸附值序列化进 scene 文档、layout 文档或 retained UI 状态。
- 不得保留旧 Project 文件的回退解析或双写。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-23 | Editor17 M1.1 -> Editor05 snap-step migration handoff | open | 现场定位三个 step 字段仍直接位于 `SceneViewportSettings`，且 retained route 直接读取；Project SettingsStore 已就绪，等待 Editor05 将场景交互接线硬切。 |

## 修复结果与回传

- 根因：Viewport snap steps were serialized as mutable SceneViewportSettings fields, leaving SettingsRegistry and its project settings store outside the authoring path.
- 架构修复：Registered bounded project-scoped float keys; moved the authoritative registry and project SettingsStore lifecycle into SceneViewportController; project transitions reload layers, commands atomically persist the Project layer, and chrome/drag state consume resolved projections.
- 验证：.\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter viewport_snap_steps_resolve_at_project_scope_and_round_trip_without_touching_project_sources -Ephemeral (exit 0); rustfmt --check; git diff --check.
- 回传：Project snap-step settings now resolve User, Project, and Session precedence through SettingsRegistry; Project persistence uses .zircon/settings.toml without modifying scene sources; SceneViewportSettings no longer owns snap-step persistence.
