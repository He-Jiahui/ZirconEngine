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
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_app/src/entry/entry_runner/editor.rs
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

## 现状与证据（zircon，2026-07-05 实读）

### 内核状态：单锁 14 字段

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

三个结构性问题：(a) **core 依赖 ui**——字段类型来自 `crate::ui::{control, host, workbench}` 三处（:10-13），00 §7 的 core/ui 分层不变量在此文件即被违反；(b) 全包裹于 `EditorEventRuntime { inner: Mutex<...> }` 单锁，`lock_inner()` 消费点**实测 56 处、跨 10 文件**（core/editor_event 3 + ui/host 6 + tests 1），任何子系统访问互相串行；(c) `dragging_gizmo`/`operation_stack`/`play_mode_backend` 等领域状态与事件内核混居。

### 消息总线：拓扑成熟、载荷空心、两处硬伤

`EditorMessageBus`（`bus.rs:10-154`）方法全表：`register_subscriber(topics) -> EditorSubscriberId`、`publish(topic, msg) -> EditorMessageDispatchReport`、`broadcast(topic, msg)`、`request(target, topic, msg, handler) -> Result<EditorMessageResponse>`、`deliveries_for/drain_deliveries(subscriber)`、`mark_message_dirty/mark_view_dirty`、`dirty_set/drain_dirty`。内部四张 BTreeMap（subscribers/subscriptions/inboxes/dirty）。

`EditorMessage`（`message.rs:34-66`）= `{ payload: EditorMessagePayload, dirty: Option<EditorViewDirtyMark> }`，构造器 `empty()/text()/with_dirty()`；载荷仅 `Empty | Text(String)`（:8-11）；全链 serde 可序列化（`Serialize/Deserialize` 已 derive——ABI 投递的先决条件已成立）。

两处硬伤：

1. **bus 方法全部 `&mut self`**——它没有自己的锁，靠寄生在大锁里获得线程安全。拆解大锁时 bus 必须获得自己的并发外壳，不能裸 `Arc<EditorMessageBus>`。
2. **`request()` 持锁同步回调**（`bus.rs:71-93`）：在 `&mut self` 借用期间调用 `handler.handle_editor_request(...)`——handler 内若再进 bus 即死锁/重入 panic。现状靠大锁外的调用纪律苟活；拆锁后必须结构性修复（见 §设计-并发外壳）。

### runtime 交互双轨

- 进程内直连：`retained_host/app.rs` 的 `EditorRuntimeClient/SharedEditorRuntimeClient` 直接 `use zircon_runtime::scene::{Scene, NodeId, LevelSystem}`、`zircon_runtime::core::{CoreHandle, ManagerResolver}`。
- ABI 面：`zircon_runtime_get_api_v2` → `ZrRuntimeApiV2` 17 个 session 入口：既有 create/destroy/event/frame/accessibility/surface/profile/tick/host-request 入口，加 plugin-event subscribe/unsubscribe/drain 与 operation submit/poll/harvest。V2 是唯一加载表，不保留 V1 表导出或 loader fallback。
- `RuntimeDynamicSession`（`session.rs:287-300`）：`runtime: CoreRuntime, profile, render_bridge: Option<RuntimeRenderBridge>, level: LevelSystem, selected_node: Option<u64>, camera_controller`。profile 五态 `Runtime/Editor/Dev/Minimal/Headless`。**每 session 独立 `CoreRuntime`+`LevelSystem`**（04 进程内 PIE 底座）。
- 越界痕迹：`selected_node` 是 authoring 状态住进了 runtime session，本计划裁决迁出。

### 模块接入

`EditorModule`（`ui/host/module.rs:37-101`）：五模块依赖（Foundation/Asset/Scene/Graphics/UI），`InitLevel::Editor`，注册 `EditorHostDriver`（Immediate）+ 四 Lazy manager（`EditorManager`、`EditorAssetManager`、`EditorCommandRegistry`、`EditorKeymap`）。`EditorManager` 依赖 Foundation `ConfigManager`（:57-61）。

## 目标

1. **`EditorContext` 聚合根**（00 §3 定形）：拆解 `EditorEventRuntimeState`——journal/listeners 留事件服务，operation_registry/stack 移交 03，play_mode_backend 移交 04，`dragging_gizmo` 移交 05，ui 域字段（state/transient/control_service/manager）**留在 ui 侧新建的 `WorkbenchShellState`**（core 不得再引用它们）。`EditorEventRuntime` 更名降级为 `EditorEventService`。
2. **消息并发外壳与载荷类型化**：`SharedEditorMessageBus` 提供内锁 + `request()` 重入修复；`EditorMessagePayload` 升级为四族类型化枚举，`Empty/Text` 删除（硬切换，`text()` 调用点全量迁移）。
3. **`EditorRuntimeGateway` 双实现**：`InProcessGateway`（LevelSystem 直连零成本转发）与 `SessionGateway`（11 指针表包装）；ui/ 深路径直用被守卫测试拒绝。
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
    builder.rs          # EditorContextBuilder：GUI/headless 两条构造路径共用
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
    events: Arc<EditorEventService>,          // journal + listeners + next_event_id/sequence/revision
    gateway: Arc<dyn EditorRuntimeGateway>,
    capabilities: RuntimeCapabilities,        // 构造期物化，只读快照
    // 后续计划字段（本计划留类型占位，不实现）：
    // jobs(14) / settings(17) / transactions(03) / selection(05) / contributions(06) …
}

impl EditorContext {
    pub fn bus(&self) -> &SharedEditorMessageBus { &self.bus }
    pub fn events(&self) -> &Arc<EditorEventService> { &self.events }
    pub fn gateway(&self) -> &Arc<dyn EditorRuntimeGateway> { &self.gateway }
    pub fn capabilities(&self) -> &RuntimeCapabilities { &self.capabilities }
}
```

构造顺序（`builder.rs`，两条路径共用）：

1. `SharedEditorMessageBus::default()`（无依赖）；
2. `EditorEventService::new(bus.clone())`（journal 写入即产 bus 事件）；
3. gateway：GUI 路径注入 `InProcessGateway::new(core_handle, level_system)`（由 `EditorManager` 物化时从模块内核解析）；headless/远程路径注入 `SessionGateway::new(api_table, session_handle)`；
4. `capabilities = gateway.capabilities().clone()`；
5. 组装 `EditorContext`，`Arc` 化后交 `EditorManager` 持有（GUI）或 commandlet runner 持有（16）。

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
    FocusObject { entity: u64 },
}
```

定型规则：(a) 族内变体只携带 **ID + 世代号**，不内嵌重数据——订阅者收到后经 query 面拉取（02），保证消息可无脑 clone 进多收件箱；(b) `DocumentId/HistoryContextId/SceneModeId/...` 在本切片先以 newtype(u64/String) 落地于 `topics.rs` 旁的 `ids.rs`，03/04/05 落地时迁移所有权但不改形状；(c) `PartialEq` 保留、`Eq` 移除（serde_json::Value 不 Eq）；(d) `EditorMessage::text()` 删除，既有调用点逐个改为语义正确的族变体或 `Custom{schema_id:"zircon.editor.debug-text"}`（执行时 Grep `EditorMessage::text` 清点入状态节）。

内建 topic 常量收敛于 `topics.rs`：`TOPIC_DOCUMENT/TOPIC_TRANSACTION/TOPIC_MODE/TOPIC_FOCUS`（四族各一固定 topic，`Custom` 用调用方 topic）——避免字符串散布。

### Gateway 契约（定稿）

```rust
// core/gateway/contract.rs
pub enum GatewayError {
    SessionLost,                         // 函数表调用失败/句柄失效
    RequiresSerializedAccess,            // SessionGateway 对借用式访问的定型拒绝
    CapabilityMissing { capability: &'static str },
    Runtime(String),                     // runtime 侧错误透传（display 化，不跨界传类型）
}

pub struct RuntimeCapabilities {
    pub session_profile: SessionProfileKind,       // 五态镜像（interface 侧 DTO）
    pub core_capabilities: Vec<String>,            // EditorCoreProfile 六能力命中集
    pub plugin_summary: Vec<PluginSummaryEntry>,   // id + version + activation
}

pub trait EditorRuntimeGateway: Send + Sync {
    fn capabilities(&self) -> &RuntimeCapabilities;
    // 借用式（仅 InProcess 支持）
    fn with_world(&self, f: &mut dyn FnMut(&World)) -> Result<(), GatewayError>;
    fn with_world_mut(&self, f: &mut dyn FnMut(&mut World)) -> Result<(), GatewayError>;
    // 序列化式（双实现都支持；02 在此扩 query/watch）
    fn inspect(&self, query: WorldInspectionQuery) -> Result<WorldInspection, GatewayError>;
    fn tick(&self, dt: FrameTick) -> Result<(), GatewayError>;
    fn capture_frame(&self, viewport: ViewportRef) -> Result<FramePayload, GatewayError>;
    fn push_editor_overlay(&self, overlay: EditorOverlayInput) -> Result<(), GatewayError>; // 选中集推送过渡通道
    // 02 追加 query/watch；04 追加 spawn_secondary_session
}
```

行为矩阵（契约测试逐格断言）：

| 方法 | InProcessGateway | SessionGateway |
| --- | --- | --- |
| `with_world/with_world_mut` | 转发 `LevelSystem::with_world*` | `Err(RequiresSerializedAccess)`（**定型行为，非 todo**） |
| `inspect` | 直调 `WorldInspection` | 经 `handle_event`/序列化载荷往返 |
| `tick` | `CoreRuntime` 帧推进 | `tick_frame` 指针 |
| `capture_frame` | 直连 render bridge | `capture_frame` 指针 |
| `push_editor_overlay` | 直写 session 叠加输入 | `handle_event` 序列化投递 |

### 拆解时序（执行剧本）

`lock_inner()` 56 处/10 文件的迁移按**字段归属**分四批，每批独立可 `cargo check`：

1. **消息批**：`message_bus` → `SharedEditorMessageBus`（消费点主要在 `editor_event_dispatch.rs`）；
2. **事件批**：`journal/event_listeners/next_*/revision` → `EditorEventService` 内锁（`editor_event_runtime.rs`、`editor_event_listener_control.rs`、`replay` 路径）；
3. **领域批**：`operation_registry/operation_stack`（→03 前先原样搬 `core/editing/` 临时 owner）、`runtime_play_mode_backend`（→04 前搬 `core/play/bridge.rs` 临时 owner）、`dragging_gizmo`（→05 前并入 viewport 现有拖拽状态）——临时 owner 均为**最终 owner 的确定路径**，不是过渡目录；
4. **UI 批**：`state/transient/control_service/manager/editor_extensions` → `ui/workbench/shell_state.rs`（`editor_event_runtime_access.rs`、`editor_event_runtime_reflection.rs`、`editor_extension_registration.rs`、`editor_operation_dispatch.rs`、`editor_event_control_requests.rs` 六文件消费点改持 shell state）。

四批完成后 `EditorEventRuntimeState`/`lock_inner` 物理删除，`grep lock_inner` 归零是切片完成判据。

### 迁移映射表（执行合同）

| 现物 | 去向 | 迁移批 |
| --- | --- | --- |
| `.message_bus` | `EditorContext.bus`（SharedEditorMessageBus） | 1 |
| `.journal/.event_listeners/.next_event_id/.next_sequence/.revision` | `EditorEventService` | 2 |
| `.operation_registry/.operation_stack` | `core/editing/`（03 终态 owner 的确定路径） | 3 |
| `.runtime_play_mode_backend` | `core/play/bridge.rs`（04 收编为 PluginBridgeActivation） | 3 |
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
- 测试阶段：`cargo test -p zircon_editor --lib --locked`——`src/tests/editor_event/runtime/` 既有测试迁移后须过；新增：四族路由单测、request 重入回归（handler 内再 publish 不死锁）、journal 拆解前后等价断言（同事件序列 revision 一致）。调用点清点数记状态节；更新 `docs/zircon_editor/core/context.md`、`docs/zircon_editor/core/editor_event.md`。

### M2 Gateway 双实现与 selected_node 迁出

- 切片 2.1：`gateway/contract.rs` + `InProcessGateway`（`EditorRuntimeClient` 迁入删原位）；`src/ui/**` 的 `zircon_runtime::scene/core` 深路径直用点改走门面（执行时 Grep 清点记状态节）。
- 切片 2.2：`SessionGateway` 包装函数表（create/destroy/tick_frame/capture_frame/handle_event/drain_host_requests 六指针先行；viewport 三件套与 profile_control 留 04 接线）；`RuntimeCapabilities` 物化。
- 切片 2.3：runtime 侧删 `RuntimeDynamicSession.selected_node`，`push_editor_overlay` 中性通道顶替（dynamic_api 消费点同步迁移，与 runtime owner 会签）。
- 切片 2.4：守卫测试——`core/` 下 `use crate::ui` 为零（00 §7 不变量，拆解后首次可启用）；`src/ui/**` 禁 `LevelSystem/CoreHandle` 深路径（白名单=gateway 实现文件）。
- 测试阶段：`cargo test -p zircon_editor --lib --locked` + `cargo test -p zircon_runtime --lib --locked`（session 字段删除牵连）+ `cargo test -p zircon_runtime_interface --locked`；双实现契约测试矩阵全绿；守卫红→绿记录。更新 `docs/zircon_editor/core/gateway.md`。

## 风险与开放问题

- 56 处 `lock_inner` 一次迁完是最大回归面；四批时序已把它切成可 check 的步进，若单批内出现借用交叉（同函数同时摸两批字段），按「先拆函数再迁批」处理，不允许两批合迁。
- `request()` 两段式在释放锁窗口内 target 可能被注销——二次上锁时重验 target，失效返回 `UnknownSubscriber`（新增该竞态单测）。
- `selected_node` 迁出改变渲染选中高亮的输入来源，M2 期间 `push_editor_overlay` 每帧推送顶替；05 落地 PickIdExtract 后该通道升级为正式 HighlightSet 推送（05 计划接管）。
- `Custom{schema_id}` 是插件旁路类型系统的口子——12 的贡献物化器是唯一合法生产者，schema_id 命名空间 `zircon.plugin.<id>.*` 预留，守卫随 12 落地。

## 产出记录与时间

请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：M1.1-M1.3 的实现与聚焦合同已完成；最后一轮完整门禁仍为 2763 passed / 133 failed / 34 ignored，失败已按 Editor UI 03/05/06/08 功能计划接管，因此 Editor01 M1 保持 `in_progress`，M2 尚未开始。

- M1 详细产出归档：[2026-07-14-editor-kernel-m1-output-records](01/2026-07-14-editor-kernel-m1-output-records.md)
- fixed 已修复：[font discovery](01/fixed-2026-07-11-editor-m1-font-discovery.md) · [plugin provider lookup](01/fixed-2026-07-11-editor-m1-plugin-provider-lookup.md) · [ZUI governance](01/fixed-2026-07-11-editor-m1-zui-governance.md) · [OIT buffer plan export](01/fixed-2026-07-12-oit-buffer-plan-export.md) · [collider shape exhaustiveness](01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md)
- fixed 已修复：[plan-output-record-archive-limit](09/fixed-2026-07-14-plan-output-record-archive-limit.md)
- open 待修复：[EditorUI03 retained text](../editor_ui/03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md) · [EditorUI05 UI Asset V2](../editor_ui/05/failure-2026-07-11-ui-asset-v2-projection-drift.md) · [EditorUI06 native painter](../editor_ui/06/failure-2026-07-11-mui-native-painter-contract-drift.md) · [EditorUI08 retained window](../editor_ui/08/failure-2026-07-11-retained-window-hard-cutover-expectations.md) · [EditorUI08 runtime diagnostics](../editor_ui/08/failure-2026-07-11-runtime-diagnostics-physics-state-format.md)
