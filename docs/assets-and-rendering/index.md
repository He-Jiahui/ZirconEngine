---
related_code:
  - zircon_resource/src/lib.rs
  - zircon_resource/src/locator.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/record.rs
  - zircon_resource/src/manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/pipeline/manager.rs
  - zircon_asset/src/watch.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_scene/src/module.rs
  - zircon_graphics/src/scene/scene_renderer.rs
  - zircon_graphics/src/service.rs
  - zircon_graphics/src/types.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/host/app.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state.rs
implementation_files:
  - zircon_resource/src/lib.rs
  - zircon_resource/src/locator.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/record.rs
  - zircon_resource/src/manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/pipeline/manager.rs
  - zircon_asset/src/watch.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_scene/src/module.rs
  - zircon_graphics/src/scene/scene_renderer.rs
  - zircon_graphics/src/types.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/host/app.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state.rs
plan_sources:
  - user: 2026-04-13 实现目录式 Project 资源抽象优先全链路替换计划
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_resource/src/tests.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_scene/src/lib.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_editor/src/lib.rs
  - cargo test -p zircon_resource -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor
  - cargo test --workspace --locked
doc_type: category-index
---

# Assets And Rendering

## Purpose

本目录记录目录式项目、资源抽象层、导入/监听、场景实例化、prepare/cache 渲染链路以及 editor 视口联动这条主链的实现约束。

## Documents

- [Directory Project Asset Rendering](./directory-project-asset-rendering.md): `zircon_resource` locator/handle/state 契约，`Project/assets` 与 `Project/library` 的职责，`res://`/`lib://`/`builtin://`/`mem://` 统一来源，`AssetManager`/`ResourceManager`/`EditorAssetManager`、`SceneAssetSerializer`、`LevelManager -> LevelSystem -> World` 与 graphics revision cache 的自动刷新路径。

## Current Scope

当前文档覆盖的交付边界是：

- `zircon_resource` 作为跨 crate 资源基础层，统一 locator、typed handle、state、record、event、manager 契约
- `zircon-project.toml` + `assets/` + `library/` 的目录式项目根
- `res://` / `lib://` / `builtin://` / `mem://` 的统一资源来源模型
- PNG/JPEG、WGSL、TOML material、TOML scene、OBJ、glTF/GLB 的导入与 library artifact 持久化
- `SceneAssetSerializer` 驱动的 `SceneAsset <-> World` 转换，以及 `LevelSystem` 对运行中 world 的托管
- `MeshRenderer`/`RenderExtract` 基于 `ResourceHandle<ModelMarker/MaterialMarker>` 的渲染输入
- `zircon_graphics` 基于 `ResourceId + revision` 的 prepare/cache 与 WGSL shader / pipeline 选择
- editor 打开目录项目、导入模型、保存默认 level、通过 `ResourceManager` 读取资源树并在 watcher 变化后重建 viewport render service

尚未覆盖的高阶内容仍包括完整 metallic-roughness 扩展材质模型、FBX/ASTC/PVRTexTool 真正导入链，以及 editor 对 `ResourceRecord.diagnostics` 的完整可视化面板。
