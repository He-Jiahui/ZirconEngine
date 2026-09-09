---
related_code:
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample/end_to_end.rs
  - zircon_runtime/src/asset/tests/facade/recursive_dependencies.rs
implementation_files:
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading
plan_sources:
  - user: 2026-09-09 引擎用户场景配方：安全导入并消费资源
  - docs/wiki/scene-assets/resource-runtime.md
tests:
  - zircon_runtime/src/asset/tests/project/asset_flow_sample/end_to_end.rs
  - zircon_runtime/src/asset/tests/project/manager/full_generation.rs
  - zircon_runtime/src/asset/tests/facade/handle_lifecycle.rs
  - zircon_runtime/src/asset/tests/facade/recursive_dependencies.rs
doc_type: workflow-detail
---

# 安全导入资源并在运行时消费

## 目标

将项目源文件纳入 catalog，取得类型安全的 `Handle<TAsset>`，等待依赖就绪后读取数据或租约。导入阶段由 `ProjectManager` 负责；运行时读取由 `ProjectAssetManager` facade 与 `ResourceManager` 负责，两者不能混成“直接读文件”。

## 架构与数据流

```text
项目根目录
  -> ProjectManager::open
  -> scan_and_import（元数据、导入器、registry 持久化）
  -> ResourceRecord / AssetId
  -> ProjectAssetManager::load<TAsset>(AssetUri)
  -> ensure_loaded + readiness generation
  -> Handle<TAsset> -> load_state / assets().get / acquire
```

## 前置条件

1. 项目根包含有效 manifest，且 asset roots 已在 manifest 中声明；`ProjectManager::open` 会创建 derived layout 并恢复未完成的 durable transaction。
2. 对应 importer 已注册，源文件位于项目允许的根目录内。
3. 调用方知道目标 marker（例如 `ModelMarker`、`TextureMarker`、`SceneMarker`），不以文件扩展名替代资源类型校验。

## 操作步骤

1. 通过 `ProjectManager::open(root)` 打开项目。不要自行拼 registry 路径。
2. 首次打开或批量变更后调用 `scan_and_import()`；文件 watcher 的单文件变更使用 `scan_and_import_watch_changes`，由实现决定增量还是完整 reconciliation。
3. 通过稳定 `AssetUri` 调用 `ProjectAssetManager::load::<TAsset>`。该调用会检查 locator 存在、marker 类型匹配并触发加载，返回 typed handle。
4. 立即读取 `load_state`、`dependency_load_state` 或 `recursive_dependency_load_state`。只有状态显示 ready/loaded 后才把数据交给渲染、场景或脚本层。
5. 读取数据时使用 `assets::<TAsset>().get(handle)`（共享 `Arc`）、`get_cloned`，或在需要明确生命周期时使用 `acquire` 返回 `ResourceLease`。
6. 订阅 `subscribe_asset_events::<TAsset>` 以处理 reload、unload 和 failure；事件 gap 时重新读取 readiness snapshot。

### Rust 形状（示意，manager 的获取方式取决于宿主装配）

```rust
use zircon_runtime::asset::{AssetUri, ProjectAssetManager};
use zircon_runtime::core::resource::ModelMarker;

fn request_model(
    assets: &ProjectAssetManager,
    uri: &AssetUri,
) -> Result<(), zircon_runtime::core::CoreError> {
    let handle = assets.load::<zircon_runtime::asset::ModelAsset>(uri)?;
    if assets.is_loaded_with_dependencies(handle) {
        let model = assets.assets::<zircon_runtime::asset::ModelAsset>().get(handle);
        let _ = model; // 将 Arc<ModelAsset> 交给只读消费者
    }
    let _marker_kind = <ModelMarker as zircon_runtime::core::resource::ResourceMarker>::KIND;
    Ok(())
}
```

示例仅展示 facade 合同；`ProjectAssetManager` 的实例应从运行时服务/项目 pipeline 获取，不要在消费者旁边构造第二个 resource registry。

## 预期可观测性

- locator 缺失、类型不匹配和 importer 错误以 `CoreError`/`AssetImportError` 返回。
- `AssetLoadState` 与递归依赖状态应随 readiness generation 单调变化；failure 时用 `failure_reason(handle)` 记录可读原因。
- 导入提交后可在 registry/catalog 中看到 `ResourceRecord`；watch 增量会报告受影响依赖。
- `AssetEventReceiver` 提供 typed 事件流，适合驱动 UI 进度、缓存失效和渲染资源重建。

## 恢复路径

- 导入失败：保留源文件，检查 importer 诊断后重新 `scan_and_import`；不要手工删除 registry 文件。
- durable commit 失败：再次打开项目会执行恢复逻辑；确认恢复完成后再重试导入。
- 资源 failed：读取 `failure_reason`，修复依赖或源文件，等待 watcher 重新导入，必要时显式再次 `load`。
- 事件 gap：丢弃增量假设，读取当前 readiness/resource snapshot，再重建本地缓存。

## 生产检查清单

- [ ] 所有资源消费都使用 typed handle 和 marker 校验。
- [ ] 读取数据前检查直接/递归依赖 readiness。
- [ ] 长期持有 GPU/CPU 数据时使用 `ResourceLease` 或明确的 `Arc` 所有权。
- [ ] 导入与运行时读取分属各自 authority，不绕过 registry 直接读派生文件。
- [ ] reload、failure、gap 均有日志和缓存恢复策略。
