---
related_code:
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/load_state.rs
implementation_files:
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/asset/facade
plan_sources:
  - user: 2026-09-09 资源运行时说明
tests:
  - zircon_runtime/src/asset/tests/facade
  - zircon_runtime/src/asset/tests/pipeline/manager/resource_records.rs
doc_type: module-detail
---

# 资源句柄与运行时注册表

## 概念模型

`zr_resource` 是 canonical foundation；`core::resource` 仅做运行时投影。`ResourceId` 由稳定标签生成，`ResourceLocator`/`AssetReference` 表示 URI 和依赖；`ResourceHandle<T>` 携带 marker 类型（如 `ModelMarker`、`TextureMarker`、`MaterialMarker`、`SceneMarker`）以在编译期区分资源类别。

`ResourceRegistry`/`ResourceManager` 保存 `ResourceRecord`、`ResourceData`、运行时状态和事件流。`ResourceSnapshot`、`ResourceProjectionSnapshot` 用于无锁读取；`ResourceLease` 表示加载期间的所有权租约。资产 facade 的 `Assets<T>` 提供 typed handle 到数据的访问。

## 状态与就绪

`ResourceState`/`RuntimeResourceState` 描述 unloaded、loading、ready、failed 等阶段；`ResourceReadinessGeneration` 为一批依赖计算提供单调 generation。资产 facade 进一步以 `AssetLoadState`、`DependencyLoadState`、`AssetReadinessReport` 表示递归依赖是否可用，并通过 `AssetEventReceiver` 接收加载/卸载/失败事件。

```rust
use zircon_runtime::core::resource::{ModelMarker, ResourceHandle, ResourceId};
let model: ResourceHandle<ModelMarker> = ResourceHandle::new(
    ResourceId::from_stable_label("asset://characters/hero.glb#model"));
let id = model.id();
// 将句柄写入 MeshRenderer；实际数据由 AssetManager/ResourceRegistry 异步解析
```

## 生命周期与错误

导入器产生资源记录后，ProjectAssetManager 发布 generation；ResourceManager 同步记录并发出 `ResourceEvent`。句柄本身不保证数据存在，调用方必须检查 readiness 或处理 `ResourceRegistryError`。资源 ID 稳定但 schema/type marker 不匹配会拒绝注册；事件接收器可能报告 gap，需要重新读取 snapshot。

## 实现状态

句柄、marker、注册表、快照、事件、依赖就绪和 mutation batch 均已实现并由 facade/pipeline 测试覆盖。GPU 上传和特定资源解码属于 graphics/asset importer 页面范围。

## 源码与测试

[core/resource/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/core/resource/mod.rs)、[facade](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/facade/mod.rs)、[handle.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/facade/handle.rs)、[load_state.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/asset/facade/load_state.rs)。
