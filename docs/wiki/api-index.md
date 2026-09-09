---
related_code:
  - Cargo.toml
  - zircon_app/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime_interface/src/lib.rs
  - zircon_runtime_host/src/lib.rs
  - zircon_hub/src/lib.rs
  - zircon_plugins/plugin_sdk/src/lib.rs
implementation_files:
  - zircon_app/src
  - zircon_runtime/src
  - zircon_editor/src
  - zircon_runtime_interface/src
  - zircon_runtime_host/src
  - zircon_hub/src
  - zircon_plugins/plugin_sdk/src
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
tests:
  - zircon_app/src/entry/tests
  - zircon_runtime/src
  - zircon_editor/src
  - zircon_runtime_interface/src
  - zircon_runtime_host/src
doc_type: api-index
---

# Rust API 索引

这是按“调用者要解决的问题”组织的索引，不是未经筛选的 rustdoc dump。稳定性以所在 crate 的 re-export、feature gate 和本 Wiki 的状态标签为准；`pub(crate)`、测试 helper、内部锁和 ABI 私有字段不属于公共调用面。

完整的逐 crate 符号清单、模块 feature 和 documented/partial/internal 覆盖判定见[公开 API 参考](api-reference/index.md)与[覆盖矩阵](api-reference/coverage-matrix.md)。以下索引用于快速定位；参考页则按字段、错误、所有权、线程和 Rust 调用形状展开。

## Crate 入口

| crate | 主要入口 | 用途 |
| --- | --- | --- |
| `zircon_app` | `entry`, `plugins`, `prelude` | 产品启动、profile、窗口/输入宿主、插件组合 |
| `zircon_runtime` | `core`, `scene`, `asset`, `graphics`, `ui`, `script`, `dynamic_api` | Runtime 模块和服务 |
| `zircon_editor` | `editor`, `prelude` | retained editor、命令/事务、场景 authoring |
| `zircon_runtime_interface` | ABI DTO、serialization、project、world sync | App/Runtime/Editor 的内部锁步契约 |
| `zircon_runtime_host` | session、foreign output、decode、lifecycle | 安全拥有动态库 session 和输出缓冲 |
| `zircon_hub` | `projects`, `engines`, `process`, `tauri_app` | 桌面 Hub 和项目管理 |
| `zircon_plugin_sdk` | declaration、manifest、native、capability | 插件声明、生成和 ABI |

## Runtime 核心

| 符号 | 入口 | 说明 |
| --- | --- | --- |
| `CoreRuntime::try_new` | `zircon_runtime::core` | 创建核心容器和 task graph |
| `CoreRuntime::register_module` | `zircon_runtime::core::runtime` | 注册 `ModuleDescriptor` |
| `CoreRuntime::activate_registered_modules` | 同上 | 按依赖和 `InitLevel` 激活模块 |
| `CoreHandle` / `CoreWeak` | `zircon_runtime::core` | 强/弱访问句柄 |
| `ModuleDescriptor` / `ModuleLifecycle` | `core::runtime::descriptors` | 描述模块、服务和生命周期 |
| `ManagerResolver` | `core::manager` | 解析 typed manager handle |
| `EngineEvent` / `EventBus` | `core::framework` | topic + JSON payload 事件 |
| `EngineTaskGraph` / `JobScheduler` | `core::runtime::tasks` | 受限并发、取消和 drain |
| `RuntimeOperationService` / `RuntimeOperationHandler` | `zircon_runtime::operation` | 有界异步操作的注册、prepare/apply、取消和结果 harvest |
| `DiagnosticLogSettings` / `DiagnosticLogLevel` | `zircon_runtime::diagnostic_log` | 进程日志过滤、sink 配置、flush 和关闭（需 `diagnostic-log` feature） |

详见[核心运行时](core-runtime/index.md)。

异步操作的阶段、admission 和 handler 示例见 [Runtime Operation](core-runtime/operations.md)；进程日志过滤、sink 和 flush 见[诊断日志](core-runtime/diagnostic-log.md)。

按 `core::framework` 契约逐模块查找时，使用[Framework 模块目录](architecture/framework-module-catalog.md)。

## 场景与资源

| 符号 | 入口 | 用途 |
| --- | --- | --- |
| `World` / `Entity` | `zircon_runtime::scene` | ECS 世界和实体生命周期 |
| `Transform` / `GlobalTransform` | `zircon_runtime::scene` | 局部/全局空间变换 |
| `SceneHandle` / dynamic scene APIs | `zircon_runtime::scene` | 场景加载、实例化和层级 |
| `AssetId` / `ResourceHandle` | `zircon_runtime::asset` / `core::resource` | registry、缓存和资源生命周期 |
| `ProjectManifestSummary` | `zircon_runtime_interface::project` | 项目 manifest 解析 |

详见[场景与资产](scene-assets/index.md)。

## 图形与 UI

| 符号/模块 | 入口 | 备注 |
| --- | --- | --- |
| `RenderGraph`、render framework | `zircon_runtime::render_graph`, `graphics` | pass、资源依赖和提交 |
| `RhiDevice`、WGPU backend | `zircon_runtime::rhi`, `rhi_wgpu` | adapter、surface、buffer/texture |
| `SceneRenderer`、mesh/material/texture | `zircon_runtime::graphics` | 场景可见性和材质数据 |
| `UiTree`、layout、text | `zircon_runtime::ui`, `text` | retained UI、布局、字体和输入 |

详见[图形](graphics/index.md)和[UI](ui/index.md)。

## 脚本、反射、动画与导航

| 符号 | 入口 | 用途 |
| --- | --- | --- |
| `VmBackend` / `VmPluginManager` | `zircon_runtime::script` | VM backend、插件发现和热重载 |
| `ZrReflect` / `ReflectSchemaCatalog` | `zircon_runtime::script::reflection` | 类型/字段 schema 和安全读写 |
| `RUNTIME_API_V8` / `zircon_runtime_get_api_v8` | `zircon_runtime::dynamic_api` | 版本化 C ABI 函数表 |
| `DefaultAnimationManager` | `zircon_runtime::animation` | clip、graph、state machine、sequence |
| `BuiltinNavigationManager` | `zircon_runtime::navigation` | baked navmesh 查询和 agent tick |

详见[脚本、反射与动画](script-animation/index.md)。

## Editor 与 App/Host

| 符号 | 入口 | 用途 |
| --- | --- | --- |
| `EditorHost` / `EditorSession` | `zircon_editor` | 编辑器工作台和 runtime session |
| `EditorCommand` / transaction | `zircon_editor` | 可撤销 authoring 操作 |
| `RuntimeSession` / `RuntimeEntryApp` | `zircon_app::entry` | 宿主启动、帧和 shutdown |
| `RuntimeApiV8` DTO | `zircon_runtime_interface` | request/output ABI 数据 |
| `RuntimeSessionHost` | `zircon_runtime_host` | 动态库句柄、owned buffer、decode |

详见[编辑器](editor/index.md)与[App/Runtime API](app-runtime-api/index.md)。

## 插件与 Hub

| 符号 | 入口 | 用途 |
| --- | --- | --- |
| `declare_plugin!` / `PluginDeclaration` | `zircon_plugin_sdk` | 单一来源元数据 |
| `RuntimePlugin` / `EditorPlugin` | plugin SDK/catalog | linked plugin 注册 |
| `CreateProjectRequest` / `package_project` | `zircon_hub::projects` | 项目创建和打包 |
| `SourceEngineInstall` / `BuildCommand` | `zircon_hub::engines`, `build` | 引擎登记和构建 |
| `HubRuntimeSession` / `HubViewModel` | `zircon_hub::tauri_app` | Tauri 状态和动作 |

详见[插件](plugins/index.md)和[Hub/工具链](hub-tooling/index.md)。

## 深入参考入口

| 领域 | 入口 | 重点 |
| --- | --- | --- |
| Runtime core | [CoreRuntime 参考](core-runtime/reference/index.md) | 构造、句柄、模块、任务、操作、诊断 |
| App/Host | [App 接口参考](app-runtime-api/reference/app-entry-runner-api.md) | Entry、Session、Window、Input、Shutdown、Export |
| 场景/资产 | [World 查询参考](scene-assets/reference/world-entity-query.md) | ECS、快照、ProjectManager、Registry、Importer |
| 图形 | [RenderFramework 参考](graphics/reference/render-framework-api.md) | RenderGraph、RHI、Shader、资源与 GPU 诊断 |
| UI | [UI v2 参考](ui/reference/v2-assets-and-retained-tree.md) | 保留树、布局、文本、输入、无障碍 |
| 编辑器 | [EditorHost 参考](editor/reference/host-session-project.md) | 会话、文档、事务、Viewport、Workbench |
| 插件 | [Plugin API 参考](plugins/reference/index.md) | Manifest、Native ABI、Capability、SDK |
| 工具链 | [Hub API 参考](hub-tooling/reference/cargo-zircon-cli.md) | CLI、构建、Catalog、Receipt、自动化 |

## 调用前检查

1. 查目标 crate 的 feature 和 target profile。
2. 优先使用 root/prelude/re-export 的类型，避免依赖私有路径。
3. 为 handle、generation、错误 enum 和 shutdown 顺序保留 owner。
4. 跨动态库边界只传 ABI DTO、owned buffer 和显式长度；不要传 Rust trait object、`String` 所有权或跨库 allocator 指针。
5. 用本 Wiki 页面中的状态标签确认功能是否只是 skeleton、fallback 或实验能力。
