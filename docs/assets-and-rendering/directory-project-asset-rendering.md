---
related_code:
  - zircon_resource/src/lib.rs
  - zircon_resource/src/locator.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/identity.rs
  - zircon_resource/src/lease.rs
  - zircon_resource/src/record.rs
  - zircon_resource/src/runtime.rs
  - zircon_resource/src/manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/meta.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/editor/catalog.rs
  - zircon_asset/src/editor/reference_graph.rs
  - zircon_asset/src/editor/preview.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager.rs
  - zircon_asset/src/watch.rs
  - zircon_asset/src/assets/material.rs
  - zircon_asset/src/assets/scene.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_scene/src/level_system.rs
  - zircon_scene/src/module.rs
  - zircon_graphics/src/scene/scene_renderer.rs
  - zircon_graphics/src/service.rs
  - zircon_graphics/src/types.rs
  - zircon_graphics/src/backend/render_backend.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state.rs
  - zircon_editor/src/editing/asset_workspace.rs
  - zircon_editor/src/workbench/snapshot.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/src/host/bridge/viewport.rs
implementation_files:
  - zircon_resource/src/lib.rs
  - zircon_resource/src/locator.rs
  - zircon_resource/src/handle.rs
  - zircon_resource/src/identity.rs
  - zircon_resource/src/lease.rs
  - zircon_resource/src/record.rs
  - zircon_resource/src/runtime.rs
  - zircon_resource/src/manager.rs
  - zircon_asset/src/project/manifest.rs
  - zircon_asset/src/project/meta.rs
  - zircon_asset/src/project/paths.rs
  - zircon_asset/src/project/manager.rs
  - zircon_asset/src/editor/catalog.rs
  - zircon_asset/src/editor/reference_graph.rs
  - zircon_asset/src/editor/preview.rs
  - zircon_asset/src/editor/manager.rs
  - zircon_asset/src/pipeline/manager.rs
  - zircon_asset/src/watch.rs
  - zircon_scene/src/components.rs
  - zircon_scene/src/world/bootstrap.rs
  - zircon_scene/src/world/project_io.rs
  - zircon_manager/src/lib.rs
  - zircon_scene/src/level_system.rs
  - zircon_scene/src/module.rs
  - zircon_graphics/src/scene/scene_renderer.rs
  - zircon_graphics/src/types.rs
  - zircon_editor/src/editing/command.rs
  - zircon_editor/src/editing/state.rs
  - zircon_editor/src/editing/asset_workspace.rs
  - zircon_editor/src/workbench/snapshot.rs
  - zircon_editor/src/workbench/project.rs
  - zircon_editor/src/host/resource_access.rs
  - zircon_editor/src/host/slint_host/app.rs
  - zircon_editor/src/host/slint_host/ui.rs
  - zircon_resource/src/id.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_editor/src/tests/host/resource_access.rs
plan_sources:
  - user: 2026-04-13 实现目录式 Project 资源抽象优先全链路替换计划
  - user: 2026-04-14 编辑器资源管理器 UI 真正接到 EditorAssetManager / EditorAssetServer
  - .codex/plans/全系统重构方案.md
  - .codex/plans/编辑器资源管理器双模式 UI 接线计划.md
tests:
  - zircon_resource/src/tests.rs
  - zircon_asset/src/tests/project/manifest.rs
  - zircon_asset/src/tests/project/manager.rs
  - zircon_asset/src/tests/editor/manager.rs
  - zircon_asset/src/tests/pipeline/manager.rs
  - zircon_asset/src/tests/watcher.rs
  - zircon_scene/src/lib.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_editor/src/tests/workbench/project.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/editing/import.rs
  - zircon_editor/src/tests/host/resource_access.rs
  - cargo test -p zircon_resource -p zircon_asset -p zircon_scene -p zircon_graphics -p zircon_editor
  - cargo test --workspace --locked
doc_type: module-detail
---

# Directory Project Asset Rendering

## Purpose

这一轮实现把主链替换为“目录式项目 + 资源抽象层 + UUID/meta 持久化 + runtime lease/refcount + revision 驱动 prepare/cache + editor catalog/reference/preview”模型：

- `zircon_resource` 定义跨 crate 的 locator、typed handle、record、state、event 和 manager 契约
- `zircon_asset::AssetManager` 负责 runtime resident 资源生命周期
- `zircon_asset::DefaultEditorAssetManager` 负责 editor catalog/meta/reference/preview 生命周期，并通过 `zircon_manager::EditorAssetManager` façade 对外暴露
- `zircon_scene::World` 运行时只持 typed handle，不再持路径语义
- `zircon_scene::LevelSystem` 托管运行中的 world、metadata 和子系统生命周期
- `zircon_graphics` 按 `ResourceId + revision` 准备 GPU 资源
- `zircon_editor` 通过 `AssetManager + ResourceManager + EditorAssetManager` 消费这些层

目标不是先堆更多 importer 分支，而是先把“project -> resource -> scene -> render -> editor”变成统一的框架主链。

## Resource Foundation

`zircon_resource` 是新的基础层，提供四类核心对象：

- `ResourceLocator`
  - 统一支持 `res://`、`lib://`、`builtin://`、`mem://`
  - 负责规范化、越界拒绝和 `#label` 子资源语法
- `AssetUuid` / `AssetReference`
  - 项目资产稳定身份改成 `UUID 主、locator 辅`
  - 旧 locator-only TOML 会在读取时按 locator 稳定派生 UUID
- `ResourceHandle<TMarker>` / `UntypedResourceHandle`
  - `ModelMarker`、`MaterialMarker`、`TextureMarker`、`ShaderMarker`、`SceneMarker` 把运行时引用类型化
- `ResourceLease<T>`
  - 运行时获取资源时返回 typed lease
  - lease drop 后递减 refcount，最后一个 lease 释放时 resident payload 转 `Unloaded`
- `ResourceRecord`
  - 权威索引项，记录 `id`、`primary_locator`、`artifact_locator`、`revision`、`state`、`dependency_ids`、`diagnostics`
- `ResourceManager`
  - 管 ready payload、runtime refcount、resident unload/reload 和 reload failure
  - 重载失败时保留 last-good payload，只把 record 状态与诊断改成错误态

`res://`、`lib://`、`builtin://` 的 `ResourceId` 都由规范化 locator 稳定派生。项目源资源的主 id 现在改为 `AssetUuid + #label` 稳定派生，`mem://` 则只在当前进程内稳定，不能写回 project/scene/material 文件。

## Project And Import Layer

目录式项目根继续固定为：

- `Project/zircon-project.toml`
- `Project/assets/`
- `Project/library/`

`ProjectManager` 负责：

- manifest / path layout
- 扫描 `assets/`
- 为缺失资源补写 `*.meta.toml`
- 调 importer 解析 PNG/JPEG、WGSL、TOML material、TOML scene、OBJ、glTF/GLB
- 把导入物写到 `library/`
- 生成 `ResourceRecord` 元数据和 `AssetUuid` 驱动的 `ResourceId`

sidecar meta 文件当前固定为 `foo.ext.meta.toml`，至少记录：

- `asset_uuid`
- `primary_locator`
- `kind`
- `source_mtime_unix_ms`
- `source_hash`
- `preview_state`

`AssetManager` 现在是 runtime 资产管理器，而不是 project/editor 混合 façade。它内部组合：

- `ProjectManager`
- `ResourceManager`
- watcher / broadcaster
- runtime lease/refcount 恢复逻辑

`EditorAssetManager` 是新的 sibling manager，负责：

- 基于 project scan 建 catalog
- 载入 `*.meta.toml`
- 解析 material/scene 直接引用
- 维护“谁引用我 / 我引用谁”的直接引用图
- 管理 `library/editor-previews/` 的缓存路径和 dirty/visible refresh 策略

`AssetManager` 继续负责项目打开、重导入、watch 生命周期。  
`ResourceManager` 负责 locator 解析、resource status/revision 查询和资源事件订阅。  
`EditorAssetManager` 负责 catalog、引用图和 preview 刷新。

## Scene Runtime

`zircon_scene` 的关键切换是 `LevelManager -> LevelSystem -> World` 分层下的 handle-runtime：

- `MeshRenderer.model: ResourceHandle<ModelMarker>`
- `MeshRenderer.material: ResourceHandle<MaterialMarker>`
- `RenderMeshSnapshot` 也直接携带这两个 typed handle

`SceneAsset` 和 `MaterialAsset` 文件现在统一存 `AssetReference { uuid, locator }`。加载规则是：

- `res://` 先按 UUID 命中 project catalog，再按 locator 回退修复旧引用
- `builtin://` 直接由 locator 派生 stable id
- 找不到的 `res://` 会回退到 `builtin://missing-model` 或 `builtin://missing-material`

保存规则也做了硬切换：

- `res://` 原样写回
- `builtin://` 原样写回
- 没有持久 locator 的运行时资源直接报错

这次已经删除 `builtin://cube <-> res://models/cube.obj` 这类隐式改写。

运行中 scene 不再暴露旧的 session façade。实际持有 world 与生命周期的是 `LevelSystem`，而 `SceneAssetSerializer` 负责 `SceneAsset <-> World` 边界。

## Graphics Prepare And Cache

`zircon_graphics::ResourceStreamer` 不再把 importer DTO 当作业务缓存键。当前实现按 `ResourceId + revision` 维护：

- prepared model
- prepared material
- prepared texture
- prepared shader

prepare 流程是：

1. scene render extract 提供 typed handle
2. streamer 用 handle.id() 查询 `ResourceRecord.revision`
3. cache miss 或 revision 变化时，从 `AssetManager` 取 ready payload 重建 GPU 资源
4. `SceneRendererCore` 按 `shader ResourceId + shader revision + double_sided + alpha_mode` 缓存 pipeline

shader 缺失时回退到 `builtin://shader/pbr.wgsl`。  
当前材质工作流仍然是 glTF metallic-roughness 的核心最小集，重点是先把资源抽象和 revision invalidation 跑通。

## Editor Asset Layer

`EditorAssetManager` 当前已经具备第一批可用能力，对应 concrete type 为 `DefaultEditorAssetManager`：

- `AssetCatalogRecord`
  - 持有 `asset_uuid`、`asset_id`、`locator`、`meta_path`、`preview_state`、`preview_artifact_path`
- `ReferenceGraph`
  - 直接边当前覆盖 material -> shader/texture、scene -> model/material
  - 解析时执行“UUID 优先、locator 回退”的迁移修复
- `PreviewCache`
  - 统一放在 `Project/library/editor-previews/`
- `PreviewScheduler`
  - 文件变更后先标 `Dirty`
  - 只有 `visible = true` 时才刷新 preview artifact
  - 不可见资源保留旧缓存

## Editor Flow

`zircon_editor` 现在通过共享的 `AssetWorkspaceState` 维护 editor 资产会话，再统一投影为 `AssetWorkspaceSnapshot`。运行时与编辑器资产层的职责已经稳定分开：

- `AssetManager`
  - 目录项目生命周期、导入、watch、runtime resident load/unload
- `ResourceManager`
  - ready/error/reloading 状态、revision、typed handle 解析
- `EditorAssetManager`
  - folder tree、catalog details、reference graph、preview policy

UI 侧不再消费 `asset_entries: Vec<String>` 这类降级模型，也不再保留旧 `iced` fallback host：

- Slint 正式宿主
  - `host/slint_host/app.rs` 从 `EditorAssetManager` 拉取 catalog/details/change
  - `host/slint_host/ui.rs` 把 `AssetWorkspaceSnapshot` 投影成 Unity-first `Assets` 和 Unreal-first `AssetBrowser`
  - 只对当前可见 tile、选中资源预览、展开 preview tab 请求刷新
- 共享 editor 状态层
  - `AssetWorkspaceState` / `AssetWorkspaceSnapshot` 作为宿主无关数据模型保留
  - 旧 `iced` host、presenter、viewport bridge 已删除，不再保留并行宿主实现

目录项目打开流程：

1. `AssetManager::open_project` 打开并扫描 project
2. editor 读取 manifest 默认 scene locator
3. `LevelManager` 通过 `SceneAssetSerializer` 和 locator 实例化 `LevelSystem`，其中持有 ECS `World`
4. editor host 通过 `AssetManager` 读项目生命周期，通过 `EditorAssetManager` 读 folder tree/detail/reference/preview，通过 `ResourceManager` 读 ready 状态和 revision
5. viewport render service 只在 GPU bridge 边界拿到 façade 或 helper，避免把资源实现细节继续扩散到宿主编辑逻辑

资源刷新流程：

1. watcher 折叠 `assets/` 文件事件
2. `AssetManager` 重新导入并更新 `ResourceRecord.revision`
3. `EditorAssetManager` 同步 catalog/details/reference/preview 状态，`ResourceManager` 保留 runtime revision/state
4. `zircon_editor::host::resource_access` 只负责把 `ResourceStatusRecord` 解析成 ready typed handle，不再承担 UI 列表组装
5. editor 收到 change record 后重建 `AssetWorkspaceSnapshot`，并在需要时重置 viewport render service
6. 新 render service 按最新 revision 重建 prepared GPU 资源

## Constraints

- `ResourceLocator` 拒绝绝对路径、`..` 逃逸和空路径
- `library/` 只存可重建导入物，不是权威源文件
- watcher 只观察 `assets/`，不观察 `library/`
- 外部 `.gltf` 仍要求用户先处理外部依赖目录；单文件 `.glb` 是当前推荐路径
- `FBX`、`ASTC`、`PVRTexTool` 目前只保留扩展位，不承诺真实导入链

## Test Coverage

当前主链已经有直接证据覆盖：

- `zircon_resource/src/tests.rs`
  - locator 规范化
  - stable/non-stable id 规则
  - `AssetUuid` / `AssetReference` roundtrip
  - runtime lease/refcount/unload
  - `ResourceId` display/parse roundtrip
  - typed handle 转换
  - registry rename/remove
  - manager last-good reload 语义
- `zircon_asset/src/tests/project/manager.rs`
  - 扫描 `assets/` 并生成 `library/`
  - 自动补写 `*.meta.toml`
- `zircon_asset/src/tests/pipeline/manager.rs`
  - 目录项目打开、资产状态和 watcher 重导入
  - `ResourceManager` status/revision/artifact locator 查询
  - reimport revision bump 和资源更新事件
- `zircon_asset/src/tests/editor/manager.rs`
  - editor catalog 构建
  - 直接引用图
  - preview dirty / visible refresh / meta 回写
- `zircon_scene/src/lib.rs`
  - `LevelManager` 创建/加载/保存 `LevelSystem`
  - `SceneAssetSerializer` 的 locator/handle roundtrip
  - level load/save 与 gizmo overlay
- `zircon_graphics/src/tests/project_render.rs`
  - headless project render
  - shader 驱动颜色回归
  - gizmo overlay 非空帧断言
- `zircon_editor/src/tests/workbench/project.rs`
  - editor project/workspace roundtrip
- `zircon_editor/src/tests/editing/state.rs`
  - `EditorState` 将 catalog/detail/resource 同步为共享 `AssetWorkspaceSnapshot`
  - 资产引用跳转同时重定向 `Assets` 与 `AssetBrowser` 两个 surface
- `zircon_editor/src/tests/editing/import.rs`
  - editor import command 在 typed handle 模型下仍可 undo
- `zircon_editor/src/tests/host/resource_access.rs`
  - host 侧按 `ResourceManager` 解析 ready typed handle
  - 非 ready 资源诊断透传

## Follow-up

- `zircon_graphics` 内部 shader locator 解析还在直接走 `AssetManager` 内部查询，后续可以继续收敛到更纯粹的 resource-only helper
- `mem://` 资源创建入口还未暴露给 editor/runtime
- 更完整的 PBR 扩展材质和 importer/transcoder 扩展仍需后续落地
