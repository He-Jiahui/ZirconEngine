---
related_code:
  - zircon_runtime/src/asset/importer/mod.rs
  - zircon_runtime/src/asset/project/manager/mod.rs
  - zircon_runtime/src/asset/registry/mod.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/core/resource/mod.rs
implementation_files:
  - zircon_runtime/src/asset/project
  - zircon_runtime/src/asset/pipeline/manager
  - zircon_runtime/src/asset/facade
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 机制 Wiki
  - docs/wiki/scene-assets/project-assets.md
  - docs/wiki/scene-assets/resource-runtime.md
tests:
  - zircon_runtime/src/asset/tests/project
  - zircon_runtime/src/asset/tests/pipeline
  - zircon_runtime/src/asset/tests/facade
doc_type: mechanism-guide
---

# 资产导入、就绪与驻留机制

资产系统把“源文件已扫描”“artifact 已生成”“资源可被运行时使用”拆成不同阶段。`ProjectManager` 负责项目级扫描和导入，`AssetImporterRegistry` 选择 importer，`ResourceRegistry`/`ResourceManager` 保存运行时记录；typed facade 只投影状态，不绕过这些所有权边界。

```mermaid
stateDiagram-v2
  [*] --> NotLoaded
  NotLoaded --> Loading: record discovered / request load
  Loading --> Loaded: ResourceState::Ready + payload
  Loading --> Failed: importer/dependency error
  Loaded --> Reloading: watcher generation changed
  Reloading --> Loaded: new artifact committed
  Reloading --> Failed: rebuild or dependency error
  Failed --> Loading: corrected source + new generation
```

## 导入到资源记录

项目打开后，`ProjectManager` 规范化 `ProjectPaths`，扫描 asset roots 并读取或创建 `.zmeta`。`AssetImporterRegistry` 按扩展名与优先级选择 `AssetImporter`；结果形成 `ImportedAssetEntry` 和 artifact，并通过 generation 关联依赖。监听器把文件系统噪声折叠为 `AssetWatchBatch`，避免 `.meta` 侧车回声重复导入。

```rust
use zircon_runtime::asset::project::ProjectManager;

let mut project = ProjectManager::open("./DemoProject")?;
let receipt = project.scan_and_import()?;
```

这段代码展示源码中存在的 `ProjectManager::open`/`scan_and_import` 形状；具体 importer 必须在当前 profile 注册。

## 就绪是三维判定

`AssetLoadStates` 同时保存根资产、直接依赖和递归依赖状态。`is_loaded()` 只表示根 payload 已加载；`is_loaded_with_direct_dependencies()` 还要求直接依赖 loaded；`is_loaded_with_dependencies()` 才代表递归依赖闭包完整。`AssetLoadState::Loaded` 的前提是 `ResourceState::Ready` 且存在 payload，只有 Ready 记录而 payload 缺失仍投影为 `NotLoaded`。

```text
root Loaded + direct Loaded + recursive Reloading
  => is_loaded() == true
  => is_loaded_with_direct_dependencies() == true
  => is_loaded_with_dependencies() == false
```

因此渲染器或场景实例化器需要按工作要求选择门槛：材质预览可接受 direct-ready，最终提交通常要求 recursive-ready。

## 驻留、租约与事件

`ResourceHandle<T>` 只携带稳定 `ResourceId` 和 marker 类型；它不保证数据驻留。加载期间由 `ResourceLease` 表达所有权，`ResourceSnapshot`/`ResourceProjectionSnapshot` 提供无锁读取。`AssetEventReceiver` 可观察加载、卸载和失败，但事件 gap 发生时必须重新读取 snapshot，而不是继续假定本地状态。

资源 ID 稳定性和数据驻留是两件事：重新导入可以保留同一 URI/ID 但替换 payload；schema/type marker 不匹配则拒绝注册。GPU 上传与显存驻留属于 graphics 层，不能由 CPU `Loaded` 状态推断。

## 错误、回滚与重试

- 路径越界、重复 UUID 或 importer 冲突：停止本批 mutation，修正 manifest/注册表后重启 generation。
- 依赖缺失：保留根记录与 `Loading`/`NotLoaded` 投影；依赖补齐后按新 generation 重试。
- artifact 生成失败：记录 `ResourceState::Error`，不要手工改 `.zmeta` 伪造 Ready。
- 热重载竞争：仅当 generation 与目标资源仍匹配时提交；过期结果丢弃并重新导入。
- 事件 gap：调用 snapshot 查询，重新建立本地 readiness，而不是重复消费旧事件。

### 排查清单

- `.zmeta` 的 `AssetUuid` 是否稳定且未被复制冲突？
- importer 是否按完整 suffix/extension 注册并具有预期优先级？
- 根 payload 与依赖状态是否分别检查？
- 是否把 `ResourceHandle` 当作强驻留引用，遗漏 lease 或 readiness？
- 失败后是否等待新 generation，而不是在旧 generation 上重试写入？

## 参考实现与测试

- 实现：`zircon_runtime/src/asset/project`、`asset/facade/load_state.rs`、`core/resource/mod.rs`。
- 相关概念页：[项目资产与导入管线](../scene-assets/project-assets.md)、[资源句柄与运行时注册表](../scene-assets/resource-runtime.md)。
- 测试覆盖 importer 选择、artifact cache、依赖 readiness、reload 和恢复失败。
