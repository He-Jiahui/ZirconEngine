---
related_code:
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/core/context/editor_context.rs
  - zircon_editor/src/core/editor_event/service/editor_event_service.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/message/mod.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/exports.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_editor/src/core/gateway/mod.rs
  - zircon_editor/src/core/gateway/session.rs
  - zircon_editor/src/tests/gateway/session.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor/tests/runtime_loading.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/tests.rs
reference_sources:
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/message.rs
  - dev/godot/editor/editor_node.h
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/index.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
status: in_progress
---

# 01 编辑器内核与 runtime 交互门面

本计划落地 00 总览的 L1 内核骨架（`EditorContext` + 类型化消息）与 L2 门面（`EditorRuntimeGateway`）。它是 W1 基座计划：02/03/04/05/06 全部以本计划的类型为地基。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-editor-kernel-runtime-interaction",
  "goal": "完成编辑器内核消息、上下文与 runtime gateway 双实现，并清除 UI 对 runtime owner 的深路径旁路。",
  "milestones": [
    {"id": "M1", "title": "消息类型化与内核拆解", "depends_on": []},
    {"id": "M2", "title": "Gateway 双实现与 selected_node 迁出", "depends_on": ["M1"]}
  ]
}
```

<!-- Workflow topology mirrors the existing M1/M2 headings. Slice acceptance does not promote the parent beyond in_progress. -->

## 参照证据（dev/）

**Fyrox `Editor` 聚合根**（`dev/Fyrox/editor/src/lib.rs:615-670`）：编辑器状态集中于单一结构体——`engine: Engine`、`scenes: SceneContainer`、`message_sender/message_receiver`（MPSC）、`plugins: EditorPluginsContainer`、`docking_manager`、`property_editors: Arc<PropertyEditorDefinitionContainer>`、`running_game_process: Option<(std::process::Child, Arc<AtomicBool>)>`（:660）。一切编辑意图经 `Message` 枚举进入主循环，主循环 `match` 分派。

**Fyrox `Message` 枚举**（`dev/Fyrox/editor/src/message.rs:47-124`）23+ 变体，语义四族，是本计划消息分型的直接模板：

```rust
pub enum Message {
    DoCommand(Command), UndoCurrentSceneCommand, RedoCurrentSceneCommand,   // 事务族
    SelectionChanged { old_selection: Selection },                          // 状态变更事实族
    SaveScene { id: Uuid, path: PathBuf }, LoadScene(PathBuf),
    CloseScene(Uuid), NewScene, SetCurrentScene(Uuid),                      // 文档族
    SetInteractionMode(Uuid), SwitchToBuildMode, SwitchToEditMode,          // 模式族
    OpenAnimationEditor, OpenMaterialEditor(MaterialResource),
    FocusObject(Handle<Node>), Exit { force: bool }, ...
}
```

**godot `EditorNode`**（`dev/godot/editor/editor_node.h:120-150`）作为反例：中心类持巨型 `MenuOptions` 枚举与全局单例，扩展新功能必改中心类。zircon 取 Fyrox 的「类型化消息 + 订阅分派」，且比 Fyrox 更进一步——Fyrox 的 MPSC 是单消费者主循环，zircon 既有 bus 是多订阅者收件箱模型（更适合多面板工作台），保留 bus 拓扑。

## 历史基线与当前事实

> 下列代码与计数是 2026-07-05 实读时用于驱动 M1/M2 的历史基线，不代表当前源码仍保留这些结构。当前 owner、路径与未闭合边界以本计划后续架构设计、里程碑和状态段为准。

### 历史基线：内核状态单锁 14 字段（已删除）

`EditorEventRuntimeState`（`editor_event_runtime_state.rs:15-31`，`pub(crate)`）全文：

```rust
pub(crate) struct EditorEventRuntimeState {
    pub(crate) state: EditorState,                    // ui/workbench/state —— UI 工作台状态
    pub(crate) manager: Arc<EditorManager>,           // ui/host
    pub(crate) transient: EditorTransientUiState,     // ui/workbench/reflection
    pub(crate) journal: EditorEventJournal,
    pub(crate) event_listeners: EditorEventListenerRegistry,
    pub(crate) editor_extensions: Vec<EditorExtensionRegistration>,
    pub(crate) operation_registry: EditorOperationRegistry,
    pub(crate) operation_stack: EditorOperationStack,
    pub(crate) message_bus: EditorMessageBus,
    pub(crate) runtime_play_mode_backend: SharedEditorRuntimePlayModeBackend,
    pub(crate) control_service: EditorUiControlService,
    pub(crate) next_event_id: u64,
    pub(crate) next_sequence: u64,
    pub(crate) revision: u64,
    pub(crate) dragging_gizmo: bool,
}
```

当时存在三个结构性问题：(a) **core 依赖 ui**——字段类型来自 `crate::ui::{control, host, workbench}` 三处（:10-13）；(b) `EditorEventRuntime { inner: Mutex<...> }` 单锁让 56 个 `lock_inner()` 消费点跨域串行；(c) `dragging_gizmo`/`operation_stack`/`play_mode_backend` 等领域状态与事件内核混居。当前 `editor_event_cutover.rs` 要求 `EditorEventRuntime`、`EditorEventRuntimeState`、`lock_inner(`、`editor_event_runtime_state` 与 `core/play/bridge.rs` 均不存在，并要求 `core/` 不依赖 `crate::ui`；M1 的这部分拆解已经完成。

### 历史基线：消息总线拓扑成熟、载荷空心

`EditorMessageBus`（`bus.rs:10-154`）方法全表：`register_subscriber(topics) -> EditorSubscriberId`、`publish(topic, msg) -> EditorMessageDispatchReport`、`broadcast(topic, msg)`、`request(target, topic, msg, handler) -> Result<EditorMessageResponse>`、`deliveries_for/drain_deliveries(subscriber)`、`mark_message_dirty/mark_view_dirty`、`dirty_set/drain_dirty`。内部四张 BTreeMap（subscribers/subscriptions/inboxes/dirty）。

`EditorMessage`（`message.rs:34-66`）= `{ payload: EditorMessagePayload, dirty: Option<EditorViewDirtyMark> }`，构造器 `empty()/text()/with_dirty()`；载荷仅 `Empty | Text(String)`（:8-11）；全链 serde 可序列化（`Serialize/Deserialize` 已 derive——ABI 投递的先决条件已成立）。

两处硬伤：

1. **bus 方法全部 `&mut self`**——它没有自己的锁，靠寄生在大锁里获得线程安全。拆解大锁时 bus 必须获得自己的并发外壳，不能裸 `Arc<EditorMessageBus>`。
2. **`request()` 持锁同步回调**（`bus.rs:71-93`）：在 `&mut self` 借用期间调用 `handler.handle_editor_request(...)`——handler 内若再进 bus 即死锁/重入 panic。现状靠大锁外的调用纪律苟活；拆锁后必须结构性修复（见 §设计-并发外壳）。

### 历史基线：runtime 交互双轨

- 进程内直连：`retained_host/app.rs` 的 `EditorRuntimeClient/SharedEditorRuntimeClient` 直接 `use zircon_runtime::scene::{Scene, NodeId, LevelSystem}`、`zircon_runtime::core::{CoreHandle, ManagerResolver}`。
- ABI 面：`zircon_runtime_get_api_v2` → `ZrRuntimeApiV2` 17 个 session 入口：既有 create/destroy/event/frame/accessibility/surface/profile/tick/host-request 入口，加 plugin-event subscribe/unsubscribe/drain 与 operation submit/poll/harvest。V2 是唯一加载表，不保留 V1 表导出或 loader fallback。
- `RuntimeDynamicSession`（`session.rs:287-300`）：`runtime: CoreRuntime, profile, render_bridge: Option<RuntimeRenderBridge>, level: LevelSystem, selected_node: Option<u64>, camera_controller`。profile 五态 `Runtime/Editor/Dev/Minimal/Headless`。**每 session 独立 `CoreRuntime`+`LevelSystem`**（04 进程内 PIE 底座）。
- 越界痕迹：`selected_node` 是 authoring 状态住进了 runtime session，本计划裁决迁出。

### 历史基线：模块接入

`EditorModule`（`ui/host/module.rs:37-101`）：五模块依赖（Foundation/Asset/Scene/Graphics/UI），`InitLevel::Editor`，注册 `EditorHostDriver`（Immediate）+ 四 Lazy manager（`EditorManager`、`EditorAssetManager`、`EditorCommandRegistry`、`EditorKeymap`）。`EditorManager` 依赖 Foundation `ConfigManager`（:57-61）。

当前 `EditorEventService`、`SharedEditorMessageBus`、类型化 `EditorMessagePayload` 与 `EditorRuntimeGatewayHandle` 已分别承担事件、消息和 runtime 门面职责；原 `EditorRuntimeClient` 与 runtime session 私有 `selected_node` 已删除。仍未闭合的是 UI 中 runtime owner 深路径的全局收敛与守卫，见 M2.4。

## 目标

1. **`EditorContext` 聚合根**（00 §3 定形）：拆解 `EditorEventRuntimeState`——journal/listeners 留事件服务，operation_registry/stack 移交 03，play_mode_backend 移交 04，`dragging_gizmo` 移交 05，ui 域字段（state/transient/control_service/manager）**留在 ui 侧新建的 `WorkbenchShellState`**（core 不得再引用它们）。`EditorEventRuntime` 更名降级为 `EditorEventService`。
2. **消息并发外壳与载荷类型化**：`SharedEditorMessageBus` 提供内锁 + `request()` 重入修复；`EditorMessagePayload` 升级为四族类型化枚举，`Empty/Text` 删除（硬切换，`text()` 调用点全量迁移）。
3. **`EditorRuntimeGateway` 双实现**：`InProcessGateway`（LevelSystem 直连零成本转发）与 `SessionGateway`（11 指针表包装）；UI runtime owner 深路径最终必须由门面和全局守卫收敛，当前完成度见 M2.4。
4. **能力协商**：`RuntimeCapabilities` 由门面物化，消费 `EditorCoreProfile` 六能力（`ui_shell/asset_core/scene_interaction/runtime_render_embed/plugin_management/capability_bridge`，`zircon_runtime/src/plugin/core_profiles.rs`）。
5. **`selected_node` 迁出 session**：编辑器 SelectionModel（05）单一事实源；过渡形态为「编辑器每帧推送选中集」中性输入。

## 非目标

- Runtime 表版本与字段治理归 runtime/10；本计划只消费已经硬切的 `ZrRuntimeApiV2` 和 `EditorRuntimeGateway`。不实现事务合并（03）、PIE（04）、query/watch 协议（02，本计划只留 trait 扩展位）；不动 bus 收件箱/dirty 的算法。

## 架构设计

### 模块布局与逐文件职责

```
zircon_editor/src/core/
  context/
    mod.rs              # 声明 + 高层再导出（EditorContext, EditorContextBuilder）
    editor_context.rs   # 聚合根结构体 + 构造顺序（见下）
    builder.rs          # EditorContextBuilder：共享服务构造；detached 默认、gateway 可注入
  editor_message/
    mod.rs              # 既有
    bus.rs              # 既有算法不动
    shared.rs           # 新增：SharedEditorMessageBus 并发外壳
    message/            # 四族载荷、信封、协议、request/response 的 folder-backed owner
    topics.rs           # 新增：内建 topic 常量表（见下）
  gateway/
    mod.rs
    contract.rs         # trait + GatewayError + RuntimeCapabilities
    in_process.rs       # InProcessGateway（EditorRuntimeClient 迁入）
    session.rs          # SessionGateway（函数表包装）
  editor_event/         # 瘦身：EditorEventService（journal/listeners/序号），不再持内核状态
zircon_editor/src/ui/workbench/
  shell_state.rs        # 新增：WorkbenchShellState（state/transient/control_service/manager 四字段新家）
```

### EditorContext 与构造顺序

```rust
// core/context/editor_context.rs
pub struct EditorContext {
    bus: SharedEditorMessageBus,
    events: Arc<EditorEventService>,
    jobs: EditorJobSystem,
    notifications: EditorNotificationService,
    transactions: Arc<EditorTransactionEngine>,
    dirty_documents: DirtyRegistry,
    commands: EditorCommandRegistryHandle,
    command_eval: CommandEvalSnapshotHandle,
    tools: ToolSchedulerService,
    gateway: EditorRuntimeGatewayHandle,
}

impl EditorContext {
    pub fn bus(&self) -> &SharedEditorMessageBus { &self.bus }
    pub fn events(&self) -> &Arc<EditorEventService> { &self.events }
    pub fn gateway(&self) -> &EditorRuntimeGatewayHandle { &self.gateway }
    pub fn capabilities(&self) -> Arc<RuntimeCapabilities> { self.gateway.capabilities() }
}
```

当前构造顺序（`builder.rs`）：

1. `SharedEditorMessageBus` 与稳定 `EditorRuntimeGatewayHandle` 由 builder 持有；未注入 transport 时 handle 为 detached。
2. `build()` 依次创建 `EditorEventService`、`EditorJobSystem` 与 `EditorNotificationService`。
3. `CoreEditContext` 消费 gateway，构造 `EditorTransactionEngine`；`EditorContext::new` 将 engine 包成共享 `Arc`，并由同一 engine 创建 `DirtyRegistry`。
4. builder 创建 command registry、command-eval snapshot 与 tool scheduler，随后按源码字段顺序组装并 `Arc` 化 `EditorContext`。
5. capabilities 不在 `EditorContext` 构造期缓存；访问器从稳定 handle 取得 generation-bound `Arc<RuntimeCapabilities>`，transport 替换不会暴露已退休 gateway 的借用。

**禁止**：`EditorContext` 提供 `Default`（隐藏依赖）、字段 `pub`（绕过访问器）、任何 `Mutex<EditorContext>` 整体锁（重蹈覆辙——锁在各服务内部）。

### 并发外壳 `SharedEditorMessageBus`

```rust
// core/editor_message/shared.rs
#[derive(Clone, Default)]
pub struct SharedEditorMessageBus {
    inner: Arc<Mutex<EditorMessageBus>>,
}

impl SharedEditorMessageBus {
    // 转发面：与 EditorMessageBus 同名方法，锁内直转
    pub fn publish(&self, topic: EditorTopic, message: EditorMessage) -> EditorMessageDispatchReport;
    pub fn broadcast(...); pub fn register_subscriber(...);
    pub fn drain_deliveries(...); pub fn drain_dirty(...); pub fn mark_view_dirty(...);

    // request 重入修复：两段式——锁内登记投递并确认 target 存在，
    // 释放锁后调用 handler，handler 返回后再锁内 mark_message_dirty(response)。
    // handler 内再进 bus 因此合法（拿到的是新锁周期）。
    pub fn request(&self, target: EditorSubscriberId, topic: EditorTopic,
                   message: EditorMessage, handler: &mut impl EditorRequestHandler)
                   -> Result<EditorMessageResponse, EditorMessageBusError>;
}
```

`bus.rs` 的 `&mut self` 内核保持原样（单测直测内核）；外部消费者**只允许**持 `SharedEditorMessageBus`——迁移完成后 `EditorMessageBus` 本体降 `pub(crate)`。

### 消息载荷类型化（定稿枚举）

```rust
// core/editor_message/message/payload.rs（EditorMessage 外壳与 dirty 机制不变）
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EditorMessagePayload {
    Document(DocumentMessage),
    Transaction(TransactionMessage),
    Mode(ModeMessage),
    Focus(FocusMessage),
    Custom { schema_id: String, payload: serde_json::Value },  // 插件唯一入口（12）
}

pub enum DocumentMessage {
    Opened { doc: DocumentId }, Closed { doc: DocumentId },
    Saved { doc: DocumentId }, DirtyChanged { doc: DocumentId, dirty: bool },
    FocusRequested { doc: DocumentId },
}
pub enum TransactionMessage {            // 03 生产
    Committed { history: HistoryContextId, label: String },
    Undone { history: HistoryContextId }, Redone { history: HistoryContextId },
    HistoryTrimmed { history: HistoryContextId },
}
pub enum ModeMessage {                   // 04/05 生产
    SceneModeChanged { mode: SceneModeId },
    PlayStateChanged { from: PlayStateKind, to: PlayStateKind },
}
pub enum FocusMessage {                  // 05 生产
    SelectionChanged { domain: SelectionDomain, revision: u64 },
    FocusObject { domain: WorldDomain, entity: u64 },
}
pub enum SelectionDomain { Scene(WorldDomain), Asset }
pub enum WorldDomain { Edit, Play(PlayInstanceId) }
```

定型规则：(a) 族内变体只携带 **ID + 世代号**，不内嵌重数据——订阅者收到后经 query 面拉取（02），保证消息可无脑 clone 进多收件箱；(b) `DocumentId/HistoryContextId/SceneModeId/...` 在本切片先以 newtype(u64/String) 落地于 `topics.rs` 旁的 `ids.rs`，03/04/05 落地时迁移所有权但不改形状；(c) `PartialEq` 保留、`Eq` 移除（serde_json::Value 不 Eq）；(d) `EditorMessage::text()` 删除，既有调用点逐个改为语义正确的族变体或 `Custom{schema_id:"zircon.editor.debug-text"}`（执行时 Grep `EditorMessage::text` 清点入状态节）。

内建 topic 常量收敛于 `topics.rs`：`TOPIC_DOCUMENT/TOPIC_TRANSACTION/TOPIC_MODE/TOPIC_FOCUS`（四族各一固定 topic，`Custom` 用调用方 topic）——避免字符串散布。

### Gateway 契约（定稿）

```rust
// core/gateway/contract.rs
pub enum GatewayError {
    SessionLost,                         // gateway 构造时 session 句柄无效
    RequiresSerializedAccess,            // SessionGateway 对借用式访问的定型拒绝
    ReentrantBorrowedWorldAccess,         // 同线程借用回调重入 fail-fast
    CapabilityMissing { capability: &'static str },
    Runtime { message: String },          // runtime 状态错误透传（display 化，不跨界传类型）
    Protocol { message: String },         // ABI/JSON/owned-buffer 合同错误
}

pub struct RuntimeCapabilities {
    session_profile: SessionProfileKind,       // 五态镜像（interface 侧 DTO）
    core_capabilities: Vec<String>,            // EditorCoreProfile 六能力命中集
    plugin_summary: Vec<PluginSummaryEntry>,   // id + version + activation；完整 tuple 确定性排序
}

pub struct EditorRuntimeFrame {
    abi_version: u32,
    width: u32,
    height: u32,
    generation: u64,
    rgba: Vec<u8>,                         // host-owned；不携带 provider free 函数指针
}

pub trait EditorRuntimeGateway: Send + Sync {
    fn capabilities(&self) -> Arc<RuntimeCapabilities>;
    fn session_handle(&self) -> ZrRuntimeSessionHandle;
    // 借用式（仅 InProcess 支持）
    fn with_world(&self, f: &mut dyn FnMut(&World)) -> Result<(), GatewayError>;
    fn with_world_mut(&self, f: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError>;
    // 当前序列化式基础面；02 在此扩 query/watch，05/10 会签 overlay 输入
    fn tick_frame(&self) -> Result<bool, GatewayError>;
    fn handle_event(&self, event: ZrRuntimeEventV1) -> Result<(), GatewayError>;
    fn capture_frame(&self, viewport: ZrRuntimeViewportHandle,
                     size: ZrRuntimeViewportSizeV1) -> Result<EditorRuntimeFrame, GatewayError>;
    fn profile_control(&self, request: &ProfileControlRequest)
        -> Result<Option<ProfileControlResponse>, GatewayError>;
    fn subscribe_plugin_event(...); fn unsubscribe_plugin_event(...); fn drain_plugin_events(...);
    fn submit_operation(...); fn poll_operation(...); fn harvest_operation(...);
}
```

`SessionGateway` 收到的 `ZrOwnedByteBuffer` 必须在 gateway 调用栈内完成结构校验；拒绝或 malformed 路径在返回前恰好释放一次。成功的 frame 将已验证 buffer 和 runtime provider owner 移交给 `EditorRuntimeFrame` 的私有 pixel owner，不做 `Vec<u8>` 全帧复制；显式 `release()` 或 Drop 才恰好调用 provider `free`。禁止把带 runtime provider `free` 函数指针的 `ZrRuntimeFrameV1` 直接暴露给编辑器消费者。

行为矩阵（契约测试逐格断言）：

| 方法 | InProcessGateway | SessionGateway |
| --- | --- | --- |
| `with_world/with_world_mut` | 转发 `LevelSystem::with_world*` | `Err(RequiresSerializedAccess)`（**定型行为，非 todo**） |
| `inspect` | 直调 `WorldInspection` | 经 `handle_event`/序列化载荷往返 |
| `tick` | `CoreRuntime` 帧推进 | `tick_frame` 指针 |
| `capture_frame` | 直连 render bridge | `capture_frame` 指针 |
| `push_editor_overlay` | 直写 session 叠加输入 | `handle_event` 序列化投递 |

### 拆解时序（已完成的执行剧本）

`lock_inner()` 56 处/10 文件的迁移按**字段归属**分四批完成：

1. **消息批**：`message_bus` → `SharedEditorMessageBus`（消费点主要在 `editor_event_dispatch.rs`）；
2. **事件批**：`journal/event_listeners/next_*/revision` → `EditorEventService` 内锁（`editor_event_runtime.rs`、`editor_event_listener_control.rs`、`replay` 路径）；
3. **领域批**：`operation_registry/operation_stack` 进入 `core/editing/`，`runtime_play_mode_backend` 进入 `core/play/plugin_activation/`，`dragging_gizmo` 并入 viewport 拖拽状态；
4. **UI 批**：`state/transient/control_service/manager/editor_extensions` → `ui/workbench/shell_state.rs`（`editor_event_runtime_access/`、`editor_event_runtime_reflection.rs`、`editor_extension_registration.rs`、`editor_operation_dispatch.rs`、`editor_event_control_requests.rs` 六个 owner 改持 shell state）。

四批完成后 `EditorEventRuntimeState`/`lock_inner` 物理删除，`grep lock_inner` 归零是切片完成判据。

### 迁移映射表（执行合同）

| 现物 | 去向 | 迁移批 |
| --- | --- | --- |
| `.message_bus` | `EditorContext.bus`（SharedEditorMessageBus） | 1 |
| `.journal/.event_listeners/.next_event_id/.next_sequence/.revision` | `EditorEventService` | 2 |
| `.operation_registry/.operation_stack` | `core/editing/`（03 终态 owner 的确定路径） | 3 |
| `.runtime_play_mode_backend` | `core/play/plugin_activation/`（`PluginBridgeActivation` trait、native/noop 实现与 report） | 3 |
| `.dragging_gizmo` | `scene/viewport` 拖拽状态（05 SceneModeStack 前身） | 3 |
| `.state/.transient/.control_service/.manager/.editor_extensions` | `ui/workbench/shell_state.rs` | 4 |
| `EditorRuntimeClient`（app.rs） | `gateway/in_process.rs` | M2 |
| `RuntimeDynamicSession.selected_node` | 删除；`push_editor_overlay` 顶替输入 | M2 |

### 深度测试

新增一个消息族（夹具枚举）+ 一个 gateway 方法消费者，应只触碰 `message/` 对应族 owner、`payload.rs` 与消费者自身；`bus.rs`、`shared.rs`、`editor_context.rs` 零改动。契约测试集对双 gateway 实现共跑（`SessionGateway` 用假函数表夹具）。

## 里程碑

### M1 消息类型化与内核拆解

- 切片 1.1：`shared.rs`（并发外壳 + request 两段式）+ `message/` folder-backed 四族载荷 + `topics.rs/ids/`；`EditorMessage::text/empty` 调用点全量迁移并删除构造器；`EditorMessageProtocol` 三值 × 四族的语义矩阵写进 `docs/zircon_editor/core/editor_message.md`。
- 切片 1.2：`core/context/` 三文件落地；按四批时序拆解 `EditorEventRuntimeState`；`EditorEventRuntime` 更名 `EditorEventService`；`lock_inner` 归零；`EditorEventRuntimeState` 删除。
- 切片 1.3：`EditorModule` 的 `EditorManager` 工厂改为构造并持有 `EditorContext`（`builder.rs` GUI 路径）；`ui/workbench/shell_state.rs` 接管 UI 批字段。
- 测试阶段：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests`——`src/tests/editor_event/runtime/` 既有测试迁移后须过；新增：四族路由单测、request 重入回归（handler 内再 publish 不死锁）、journal 拆解前后等价断言（同事件序列 revision 一致）。调用点清点数记状态节；更新 `docs/zircon_editor/core/context.md`、`docs/zircon_editor/core/editor_event.md`。

### M2 Gateway 双实现与 selected_node 迁出

- 切片 2.1：`gateway/contract.rs` + `InProcessGateway`（`EditorRuntimeClient` 迁入删原位）；`src/ui/**` 的 `zircon_runtime::scene/core` 深路径直用点改走门面（执行时 Grep 清点记状态节）。
- 切片 2.2：`SessionGateway` 包装已验证的 V2 session 函数表与 handle；create/destroy 继续由 `RuntimeSession` 单一生命周期 owner 负责，gateway 持有 provider `Arc` 防止函数指针或 frame buffer 越过库生命周期，不复制 destroy 权限。当前基础面覆盖 tick/frame/event/profile/plugin-event/operation；可选 `profile_control` 缺失返回 `Ok(None)`，必需入口缺失才返回 `CapabilityMissing`；`RuntimeCapabilities` 物化为 generation-bound `Arc` 快照，stable handle 通过 `ArcSwap` 发布完整 generation，数据面不再进入共享 `RwLock`。
- 切片 2.3：runtime 侧删 `RuntimeDynamicSession.selected_node`。当前源证明该字段只保存 construction 阶段默认 cube orbit anchor，没有编辑器更新入口或高亮消费；Runtime10 删除字段与高频 pointer/scroll selection-sync helper，保留中性初始 orbit target。正式 `push_editor_overlay`/HighlightSet 输入仍由 05 接管，不得为等待 05 而保留第二份选择真相。
- 切片 2.4（部分完成）：`editor_event_cutover.rs` 已全量守卫 `core/` 不依赖 `crate::ui`，并守卫 legacy event owner/符号与 `core/play/bridge.rs` 不得恢复；`workbench_state_cutover.rs` 仅守卫 workbench construction/project transition 两个文件不出现 `LevelSystem`。`src/ui/**` 的全局 `LevelSystem/CoreHandle` 深路径守卫尚未存在，且 UI host、asset manager 与 retained viewport 当前仍有 `CoreHandle` 消费点；该部分继续 `in_progress`，不能记为守卫闭环。

  2026-08-14 owner routing 重审：不得把上述库存机械地全部改写为 `EditorRuntimeGateway`。gateway 的定型职责是 authoring/play world、session ABI、frame/viewport/overlay 数据面；它不具备也不应获得 `ProjectAssetManager`、render-framework service graph、module bootstrap 或 native host capability 注册权限。UE `UAssetEditorSubsystem::Initialize/Deinitialize` 也在 editor subsystem composition boundary 订阅 engine/asset services，而非让每个 asset toolkit自行取得 engine 全局入口。后续 hard cut 必须拆为两类并分别守卫：(a) UI 业务/工作台/scene presentation 不得持有 `LevelSystem`、`World` 或通过 `CoreHandle` 旁路 runtime scene，统一走 gateway；(b) `EditorModule`、`EditorUiHost` 的 composition root、`DefaultEditorAssetManager` 的 project asset access 与 retained viewport 的 render-framework resolver 仅可消费一个 UI-owned、typed runtime-service access，并在构造期注入具体 manager handle/weak provider，禁止任何 pane、workspace state、callback 或 picker 取得/传播裸 `CoreHandle`、`ManagerResolver`。在先建立该 typed access 和 source inventory 分类前，不增加“全 UI 禁止 CoreHandle”的错误守卫，也不把 asset/project 服务塞入 core gateway。
- 测试阶段：`.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests` + `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests`（session 字段删除牵连）+ `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime_interface -SkipBuild -LibTests`；双实现契约测试矩阵全绿；守卫红→绿记录。更新 `docs/zircon_editor/core/gateway.md`。

## 风险与开放问题

- `EditorMessageTransactionEventSink` 将 bus error/dropped 折算为 `Rejected`、将 backpressure 折算为 `Backpressured`；`EditorTransactionEngine::publish_event` 已对两者写 `tracing::warn!`，因此不是静默失败，但不会重试、补偿、持久化计数或发布结构化诊断。history mutation 仍是权威结果，依赖 lifecycle 消息的历史面板/脏态订阅者可能缺失单次通知；该可靠性边界保持开放。
- runtime session 私有 `selected_node` 已删除；05 落地 PickIdExtract 后必须继续以 Editor SelectionModel 为唯一事实源，经正式 HighlightSet/overlay 合同推送，禁止恢复 runtime session 私有选择字段。
- `Custom{schema_id}` 仍是类型系统逃生口：内建 reflection 当前生产 `zircon.editor.debug-text`，`EditorMessage::custom` 本身不校验 namespace。内建调用须留在 `zircon.editor.*`，插件贡献须由 12 的物化边界注入并限制为 `zircon.plugin.<id>.*`；全局 namespace 守卫尚未闭合。

## 产出记录与时间

请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：M1 内核拆解、legacy owner 删除与 core→UI 分层守卫已完成；M2 的基础 gateway、session forwarding、lifecycle authority 与稳定 generation 已有实现。最新 source-bound 门禁、独立复审，以及 `src/ui/**` 的 `LevelSystem/CoreHandle` 深路径收敛和全局守卫仍未闭合，因此父计划保持 `in_progress`。

- 具体记录已迁入：[M1 详细产出归档](01/2026-07-14-editor-kernel-m1-output-records.md) · [M2 当前进度](01/2026-07-17-m2-gateway-current-source.md) · [性能评审交接归档](01/2026-08-01-performance-review-handoffs.md)
- fixed 已修复：[font discovery](01/fixed-2026-07-11-editor-m1-font-discovery.md) · [plugin provider lookup](01/fixed-2026-07-11-editor-m1-plugin-provider-lookup.md) · [ZUI governance](01/fixed-2026-07-11-editor-m1-zui-governance.md) · [OIT buffer plan export](01/fixed-2026-07-12-oit-buffer-plan-export.md) · [collider shape exhaustiveness](01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md)
- fixed 已修复：[plan-output-record-archive-limit](09/fixed-2026-07-14-plan-output-record-archive-limit.md)
- open 待修复：[gateway stable call lock and clone](01/failure-2026-07-17-gateway-stable-call-lock-and-clone.md) · [editor startup single projection](01/failure-2026-07-19-editor-startup-single-projection.md) · [Editor12 document message producer](01/failure-2026-07-29-document-message-producer-missing.md) · [authoring world test concrete level manager](01/failure-2026-07-31-authoring-world-test-concrete-level-manager.md) · [highlight set gateway contract](01/failure-2026-07-31-highlight-set-gateway-contract.md)
- open 待修复：[EditorUI03 retained text](../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md) · [EditorUI05 UI Asset V2](../editor_ui/05/failure-2026-07-11-ui-asset-v2-projection-drift.md) · [EditorUI06 native painter](../editor_ui/06/failure-2026-07-11-mui-native-painter-contract-drift.md) · [EditorUI08 retained window](../editor_ui/08/failure-2026-07-11-retained-window-hard-cutover-expectations.md) · [EditorUI08 runtime diagnostics](../editor_ui/08/failure-2026-07-11-runtime-diagnostics-physics-state-format.md)
- fixed 已修复：[font-database-render-input-equivalence-visibility](01/fixed-2026-07-17-font-database-render-input-equivalence-visibility.md)
- fixed 已修复：[Runtime15 screen-space UI text font-id report mount drift](../../zircon_runtime/text/01/fixed-2026-07-17-screen-space-ui-text-font-id-report-mount-drift.md)；Runtime15 已恢复生产挂载并收敛到 shaping query/actual glyph 单一 owner，受管 `text_font` 门 47/47、独立复审 0/0/0。
- open 待回传：[Runtime10 editor selection state session boundary](../../zircon_runtime/runtime/10/failure-2026-07-17-editor-selection-state-runtime-session-boundary.md)；M2.3 当前源已删除 `RuntimeDynamicSession.selected_node` 与 pointer/scroll selection-sync helper，保留 construction-only 中性 orbit target，聚焦门 2/2、独立复审 0/0/0；`dynamic_api` 上行 94/112 后的 Runtime10 owner gate 已到 12/13，唯一 Runtime05 stale mirror 已按 single-source hard cut 删除并复审 0/0/0，精确重跑已进入 FIFO。Runtime15 与 Render01 跨 owner failure、Runtime10 重跑和 canonical failure return 全部完成前不能 fixed。
