---
handoff_kind: failure
status: source_complete_dynamic_validation_pending
created_at: 2026-08-08
summary_slug: camera-table-render-extract-stale-map
origin_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
fixing_plan: docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/runtime/09
fixing_child_dir: docs/plans/zircon_runtime/render/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/render.rs
tests:
  - tools/build-editor.ps1
---

# Render07: Camera render extraction still reads the removed fixed map

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行记录：`docs/plans/zircon_runtime/runtime/09/2026-08-07-runtime-ui-incremental-refresh.md`
- 来源执行切片：M7 product editor bundle build
- 修复责任计划：`docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md`
- 交接原因：失败位于 Render07 的 scene render table-component migration，低于 Editor/UI 构建层。

## 失败现象与复现证据

`tools/build-editor.ps1` 编译 `zircon_runtime` 时在 `zircon_runtime/src/scene/world/render.rs:535` 和 `:550` 报错：`World` 已无 `cameras` 字段，但 camera descriptor 构建仍读取旧 map。

## 最低共享层根因

Mesh、sprite、particle 和 post-process 读取已迁移到 typed component storage；camera descriptor 的单实体读取与全量枚举遗漏了同一 hard cutover。

## 架构修复验收

- 单 camera 读取使用 `World::get::<CameraComponent>`。
- camera 枚举直接遍历 registered typed table，并通过 entity registry 恢复稳定 ID。
- 不扫描全部实体，不恢复 `World.cameras`，保持 render order/target/entity 排序语义。
- 原始 Editor bundle production build 通过。

## 禁止临时方案

- 不恢复旧 camera map、镜像字段或同步 shim。
- 不退化为每帧全实体扫描。
- 不添加测试专用分支或静默 fallback。

## 修复结果与回传

2026-08-10 current-source result:

- camera override/active selection 通过 `contains_component::<CameraComponent>` 与 `World::get::<CameraComponent>` 读取 typed storage；首个 fallback camera 直接遍历 camera table 并按稳定 entity id 取最小值，不恢复旧 map。
- `scene_camera_descriptors_with_override` 预分配 registered camera component row count，直接遍历 `for_each_table_component::<CameraComponent>`，通过 entity registry 恢复 stable id，并保持 `(render_order, target_key, entity)` 排序。
- 当前 owner 内无 `self.cameras` 读取；`rustfmt --edition 2021 --check zircon_runtime/src/scene/world/render.rs` 与 scoped source contract 通过。
- 本轮未执行原始 `tools/build-editor.ps1` 或受管 Cargo，不能从静态门推导 Editor bundle 或真实 camera render 已通过。

Open state: `camera_table_source_repair_complete_pending_managed_editor_bundle_and_render_extract_gate`; no pass is claimed.
