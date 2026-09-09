---
related_code:
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_plugins/Cargo.toml
implementation_files:
  - zircon_runtime/src/core/framework
  - zircon_runtime/src/builtin/runtime_modules
  - zircon_plugins
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/render/index.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_plugins
doc_type: module-catalog
status: source-audited
---

# Framework 模块目录

`zircon_runtime::core::framework` 是跨模块契约层。它定义 trait、DTO、事件和能力描述，不直接拥有窗口、GPU、World 或第三方后端。每个模块的实现由 `core::runtime` 生命周期、内建模块组合或插件目录接入。本页是按源码 `framework/mod.rs` 的逐项目录，适合作为网页 wiki 的模块索引；具体行为以链接的专题页和 crate root re-export 为准。

## 阅读规则

| 字段 | 含义 |
| --- | --- |
| **契约入口** | 应用、插件和编辑器优先依赖的 Rust 路径；若标为 trait，调用方不应假设具体实现。 |
| **实现 owner** | 当前拥有实现或注册逻辑的目录；`core/framework` 本身通常只放中性类型。 |
| **启用条件** | Cargo feature、`RuntimeTargetMode`、宿主能力或插件清单限制。 |
| **状态** | `已实现` 表示有当前源码入口；`可选`/`实验` 表示不能假定默认 profile 已激活。 |

公共调用通常从 `zircon_runtime::core::framework::<module>` 导入，或使用更高层的 `zircon_runtime::{scene, graphics, ui, ...}` facade。跨动态库边界只能传 `zircon_runtime_interface` 的 ABI DTO，不能传这些模块中的 trait object、借用引用或 native handle。

## 模块总表

| 模块 | 主要职责 | 契约入口（示例） | 实现 owner / 约束 | 状态 |
| --- | --- | --- | --- | --- |
| `ai` | 行为树、黑板、感知和 agent tick 的中性描述 | `AiManager`, `AiAgentId`, `AiBehaviorTreeDescriptor`, `AiAgentTickRequest` | `zircon_plugins/ai/runtime`；`ai-contracts` | 可选/实验 |
| `animation` | clip、pose、状态机、IK 和动画事件 | `AnimationManager`, `AnimationClipAsset`, `AnimationGraphAsset`, `AnimationStateMachineAsset` | `zircon_runtime/src/animation`、`zircon_plugins/animation` | 可选 |
| `asset` | 资源状态、定位和 registry 访问合同 | `ResourceManager`, `ResourceCacheIdentity`, `ResourceState` | `zircon_runtime/src/asset`、`core::resource` | 已实现 |
| `audio` | 后端无关的声道布局和扬声器描述 | `AudioChannelLayout`, `AudioSpeakerChannel` | `zircon_runtime/src/core/framework/audio`；播放服务由 sound 插件提供 | 可选/实验 |
| `bridge` | 模块间 bridge、接口槽位和 owner 生命周期 | `PluginInterface`, `InterfaceSlot`, `StrongBridge`, `BridgeInvocationTable` | `zircon_runtime/src/core/framework/bridge` | 已实现/内部接线 |
| `camera_controller` | 相机输入、轨道/自由/平移控制状态和输出 | `FreeCameraInput`, `FreeCameraSettings`, `OrbitCameraInput`, `PanCameraInput`, `CameraControllerOutput` | `zircon_runtime/src/core/framework/camera_controller` | 可选 |
| `channel` | 跨线程 channel 别名、最新值读取和超时等待 | `ChannelSender`, `ChannelReceiver`, `recv_latest`, `wait_for` | `zircon_runtime/src/core/framework/channel.rs` | 已实现 |
| `events` | topic 事件、投递策略、订阅与诊断 | `EngineEvent`, `EngineEventDeliveryPolicy` | `CoreRuntime` EventBus | 已实现 |
| `foundation` | 配置、身份、生命周期通用 DTO | `ConfigManager`, `ConfigManagerError`, `FOUNDATION_MODULE_NAME` | `core::runtime::config_store` | 已实现 |
| `gizmos` | 场景作者态辅助线、轴、网格和 overlay 描述 | `GizmoBuffer`, `GizmoCommand`, `GizmoConfig`, `GizmoOverlayExtractRequest` | `zircon_runtime/src/core/framework/gizmos`、`zircon_editor::scene::viewport` | 可选 |
| `input` | 规范化键鼠/手柄/触摸动作与 action map | `InputManager`, `InputActionManager` | `zircon_runtime/src/input`、winit adapter | 已实现/按平台 |
| `navigation` | navmesh、路径、agent 和动态 obstacle 合同 | `NavigationManager`, `NavMeshAsset`, `NavPathQuery`, `NavSampleQuery` | `zircon_plugins/navigation`；Recast/Jolt 等 backend | 可选/实验 |
| `net` | 连接、传输、RPC、复制和网络诊断 | `NetManager`, `NetEndpoint`, `RpcPayloadSchema`, `SyncReplicationBudget` | `zircon_plugins/net` feature crates | 可选/实验 |
| `physics` | 固定步模拟、碰撞查询、约束和触发器 | `PhysicsManager`, `PhysicsQueryInterface`, `PhysicsRayCastQuery`, `PhysicsWorldStepPlan` | `zircon_plugins/physics` | 可选/实验 |
| `picking` | 指针命中、射线、pipeline 和后端能力 | `PickingBackend`, `PickingPipelineInput`, `PickingPointerEvent`, `PointerRay` | graphics/editor picking route | 已实现/异步 |
| `platform` | target mode、宿主生命周期、窗口能力和偏好 | `RuntimeTargetMode`, `PlatformHostDescriptor`, `PlatformHostSnapshot`, `ApplicationLifecycleSnapshot` | `zircon_runtime/src/core/framework/platform`、app host | 已实现/按平台 |
| `project` | 项目 manifest、导出 profile、插件选择和打包策略 | `ProjectManifest`, `ExportProfile`, `ProjectPluginSelection` | `zircon_runtime/src/asset/project`、`zircon_app` | 已实现/受限 |
| `random` | 可重现随机流、seed、checkpoint 与 replay | `RandomSeedReceipt`, `RandomServiceCheckpoint`, `RandomState`, `RandomStreamCheckpoint` | `core::runtime::random` | 已实现 |
| `render` | 帧抽取、视口、材质、灯光、环境和后处理 DTO | `RenderFramework`, `RenderFrameExtract` | `zircon_runtime/src/graphics`、RHI/WGPU | 已实现/高级能力可选 |
| `scene` | Level、World、实体层级和场景生命周期 | `LevelManager`, `WorldHandle`, `SceneResource`, `SceneArtifactTicket` | `zircon_runtime/src/scene` | 已实现 |
| `script` | VM host、脚本调用点、权限和 world access | `ScriptHostCallFrame`, `ScriptHostFunctionDescriptor`, `ScriptBehaviorBridge`, `ZirconScriptType` | `zircon_runtime/src/script`、`zr_vm_language` | 可选/实验 |
| `sound` | 事件驱动音频、listener/emitter、混音和空间声学 | `SoundManager`, `SoundSourceDescriptor`, `SoundPlaybackSettings`, `SoundMixerGraph` | `zircon_plugins/sound` | 可选/实验 |
| `tasks` | 后端无关的并行切片执行合同 | `ParallelSliceExecutor` | `zircon_runtime/src/core/framework/tasks`；完整 task graph 在 `core::runtime::tasks` | 已实现 |
| `text` | 字体、文本方向、shaping、测量和渲染模式 | `TextLayoutService`, `TextFontRequest` | `zircon_runtime/src/text`、UI | 已实现/按字体 backend |
| `time` | 实时、虚拟和固定步时间合同 | `Time`, `Fixed`, `Virtual`, `TimePolicy`, `FixedStepPlan` | `core::runtime::frame_clock` | 已实现 |
| `ui` | UI 模块身份和跨层接线标识 | `UI_MODULE_NAME` | `zircon_runtime/src/core/framework/ui`；完整树/表面在 `zircon_runtime/src/ui` | 已实现/按 feature |
| `window` | 窗口描述、surface lease、尺寸和生命周期 | `WindowDescriptor`, `WindowId`, `SurfaceLease`, `WindowStateSnapshot` | app/editor host 与 WGPU | 已实现/按平台 |

`ai`、`net`、`physics`、`sound` 等行的 trait 只有在对应 `*-contracts` feature 打开时才会编译。服务器 profile 可以保留同一套 DTO，但通常不激活图形、窗口、文本或脚本实现。

## 按数据流分组

### 生命周期与调度

`foundation`、`platform`、`project`、`time`、`tasks` 和 `events` 共同构成模块运行骨架：入口解析 profile，CoreRuntime 注册依赖，FrameClock 产生时间快照，TaskGraph 执行有界工作，EventBus 传递结构化事件。领域模块不应自行创建第二套时钟、全局线程池或无界事件队列。

典型调用顺序：

```rust
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::core::framework::events::EngineEventDeliveryPolicy;

let runtime = CoreRuntime::try_new()?;
runtime.activate_registered_modules()?;
let events = runtime.subscribe_events("scene.changed", EngineEventDeliveryPolicy::Latest);
runtime.publish_event("scene.changed", serde_json::json!({"generation": 1}));
let _event = events.try_recv()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

### 世界、资源与内容

`scene` 持有 World 事实，`asset`/`project` 持有持久化输入和资源身份，`math`（位于 `core::math`，不属于 framework 子模块）提供变换和有限性验证。`render`、`ui`、`script` 只消费抽取或受限访问，不应把派生缓存写回资产或 editor 文档。

### 表现与平台

`render`、`picking`、`text`、`ui`、`window` 和 `input` 形成 client/editor 表现链。它们都允许异步或能力缺失：例如拾取和 readback 返回 pending，Surface 可能是 zero-size/non-renderable，文本 backend 可能拒绝不支持的脚本。调用方必须处理 typed error/status，而不是用 `unwrap` 把能力假设固化。

### 可选运行时族

`ai`、`animation`、`audio`、`navigation`、`net`、`physics`、`script` 和 `sound` 通过插件目录加入。插件 manifest 的 target、capability、maturity 和依赖决定它们是否进入模块组合；仅存在 trait 或 crate 不能证明默认产品已经激活。具体插件入口见[运行时插件族](../plugins/runtime-families.md)和[插件清单](../plugins/inventory.md)。

## Rust 接口准则

### 依赖契约而不是具体实现

```rust
use zircon_runtime::core::manager::ManagerResolver;

let resolver = ManagerResolver::new(runtime.handle());
let render_handle = resolver.render_framework_handle()?;
let render = resolver.resolve(render_handle)?;
let stats = render.query_stats()?;
```

`ManagerResolver` 返回带 runtime identity 的 typed handle；跨异步边界保存 handle，使用前重新解析。若需要领域实现的额外方法，先在对应插件/产品 feature 中确认 concrete type，不要把它提升为 framework 通用契约。

### 能力和错误必须显式传播

`RenderFrameworkError`、`CoreError`、`InputError`、`UiError`、各资产 validation error 以及插件注册错误都携带阶段或 owner 信息。公共函数推荐返回 `Result<T, E>`，轮询接口的 `None`/pending 不应当转换成成功的空数据。能力检查应在创建资源或提交操作之前完成，并把 fallback 记录到 diagnostics。

### ABI 边界

Framework trait 只适合同一 Rust 进程内的模块调用。动态 runtime、Hub 或原生插件边界使用 `zircon_runtime_interface` 的 `#[repr(C)]` DTO、长度显式的 `ZrByteSlice`、generation 句柄和版本化函数表；资源、World、UI 树和 GPU 对象必须在边界内重新解析。

## 专题页映射

| 模块族 | 深入说明 |
| --- | --- |
| CoreRuntime、events、tasks、time | [核心运行时](../core-runtime/index.md) |
| scene、asset、project、resource | [场景与资产](../scene-assets/index.md) |
| render、picking、window | [图形与渲染](../graphics/index.md) |
| ui、text、input | [UI、文本与输入](../ui/index.md) |
| script、animation、navigation | [脚本、反射与动画](../script-animation/index.md) |
| editor/gizmos/authoring | [编辑器](../editor/index.md) |
| plugin、net、physics、sound、AI | [插件系统](../plugins/index.md) |
| app/platform/ABI | [应用入口与运行时 API](../app-runtime-api/index.md) |

## 版本与状态维护

本目录页按当前 `framework/mod.rs` 维护。新增或迁移模块时，需要同时更新：模块总表、对应专题页、`docs/wiki/navigation.yaml`、API 索引和功能状态页。若模块只有计划或 descriptor，没有生产 owner，应标为“可扩展基础/规划中”，不得写成“默认可用”。
