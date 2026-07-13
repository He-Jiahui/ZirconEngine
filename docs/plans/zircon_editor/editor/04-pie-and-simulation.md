---
related_code:
  - zircon_editor/src/core/play/bridge.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup/with_viewport/runtime_backend.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/plugin/native.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/bin/runtime_preview.rs
reference_sources:
  - dev/Fyrox/editor/src/lib.rs
  - dev/godot/editor/run/editor_run.h
  - dev/godot/editor/run/embedded_process.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/EditorEngine.h
plan_sources:
  - docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
status: planned
---

# 04 PIE 模拟运行

- 来自 Editor08 的失败交接（`open / Building/Play 权威状态投影`）：[`04/failure-2026-07-12-command-eval-play-state-projection.md`](04/failure-2026-07-12-command-eval-play-state-projection.md)

本计划落地 00 §6 的「Edit/Play 状态」权威 `PlaySessionController`，并定义 **Unity 式运行时编辑**：Playing 期间 hierarchy/inspector 与运行世界实时同步，可直接编辑运行世界（改动随 PIE 副本在退出时整体丢弃），运行时 spawn 的实体实时进 hierarchy。执行前置提醒（index「取证口径」）：UE PIE 内部流程证据为头文件级，动工前宜再读 `Editor/UnrealEd/Private/PlayLevel.cpp` 深核。

## 参照证据（dev/）

**Fyrox 三态模式**（`dev/Fyrox/editor/src/lib.rs:327-338`）——状态机直接模板：

```rust
pub enum Mode {
    Edit,
    Build { queue: VecDeque<CommandDescriptor>, process: Option<Child>, play_after_build: bool },
    Play  { process: std::process::Child, active: Arc<AtomicBool> },
}
```

`Editor.running_game_process: Option<(Child, Arc<AtomicBool>)>`（:660），主循环 `:1412` take-kill 清理。要点：**Build 是一等状态**，`play_after_build` 把「编译→运行」串成单动作（13 衔接照此）。

**godot 子进程 + 嵌入**（`dev/godot/editor/run/`）：`EditorRun` 独立小类组装子进程参数（调试端口/断点/窗口位置）；`embedded_process.h` 嵌入呈现与进程管理分层。取「run 是独立小类」。

**UE 进程内 PIE**（`EditorEngine.h`）：`CreatePIEWorldByDuplication` 复制出 `EWorldType::PIE` 世界，`FWorldContext` 同持两世界，结束销毁副本。要点：**复制而非共享**——PIE 改动天然丢弃。zircon 以「独立副 session + 序列化注入」获得同等语义，免复制器。

**Unity 运行时编辑语义（产品级参照，dev/ 无源码，按公开行为对齐）**：Play 期间 hierarchy/inspector 持续显示**运行世界**并可直接编辑——改动即时作用于运行世界、退出 Play 随副本丢弃；运行时 spawn 的对象实时入 hierarchy；Play 期间 undo/redo 对运行世界生效，退出后该段历史消失。UE 侧补充参照：`Keep Simulation Changes`（LevelEditor 命令，行为级）提供「显式把运行世界单对象状态回写编辑世界」的例外通道。本计划 P2 路径按此组合定形：**live 同步 + live 编辑 + 默认丢弃 + 显式单实体回写**。

## 现状与证据（zircon，2026-07-05 实读）

### 「play mode」已存在但语义是插件桥接切换

trait（`editor_runtime_play_mode_backend.rs:21-28`）：`enter_play_mode(project_root) / exit_play_mode -> Result<EditorRuntimePlayModeBackendReport, String>`；报告 `{ diagnostics: Vec<String>, bridge_diagnostics: Option<BridgeDiagnosticsMatrix> }`。

`NativePluginEditorRuntimePlayModeBackend`（:48-147）实际行为（本次全文实读，比 v2 更细）：

- `enter_play_mode`：双进保护（`active_snapshot` 已有即 Err，:84-86）→ **从工程根加载 native 插件**（`load_runtime_plugins_from_project_root[_with_bridge_lifecycle]`，:90-99——插件加载发生在进 play 时，不是编辑器启动时）→ `live_host.enter_runtime_play_mode()` 取活性快照 → 诊断 sort+dedup（:110-111）。
- `exit_play_mode`：take 快照，**无快照时宽容返回**（诊断提示而非 Err，:129-136）→ `exit_runtime_play_mode(&snapshot)` 恢复。

它做的是 native 插件活性快照的进出切换：不运行游戏、不隔离世界、不产视口。装配点 `create_startup_runtime_backend`（`with_viewport/runtime_backend.rs:10-33`）：构造 `NativePluginLiveHost::default()` + `EditorEventRuntime::new` 后 `set_runtime_play_mode_backend(...)`（该 setter 经 `lock_inner()`，随 01 拆解消亡）。

### 进程内 PIE 底座异常充分

- `RuntimeDynamicSession`（`session.rs:287-300`）每 session 独立 `CoreRuntime` + `LevelSystem` + `render_bridge`——第二 session 即第二世界。
- `ZrRuntimeApiV1` 11 指针含全套：`create_session/destroy_session/tick_frame/handle_event/capture_frame/bind_viewport_surface/present_viewport/drain_host_requests`。
- `LevelSystem::snapshot()/replace()/replace_world_and_reset_runtime_state()`（`level_system.rs:111-119`，02 已核）——注入载体现成，且 `replace_world_and_reset_runtime_state` 正是 PIE 注入需要的「换世界+清运行态」语义。
- profile 五态，PIE 副会话用 `Runtime` 档。

### 子进程底座

`runtime_preview` 可执行 + `RuntimeSessionStartupArgs { profile, project_root, help_requested, remaining_args }`（`runtime_session_args.rs:37-52`）。

### 缺口

无 Edit/Play 状态机；无游戏子进程 spawn/监控；无未保存场景快照传递；无 PIE 视口；PIE 期间编辑保护缺失；trait 名与真实语义错位；**无 play 域数据同步与运行时编辑路由**——hierarchy/inspector 无法跟随运行世界，更无法编辑它（Unity 对齐的两个半程都缺，但 02 的 `SubscriptionTable` 挂在 session 级、每 session 独立 `LevelSystem`，读半程的 runtime 底座已现成）。

## 目标

1. **状态机与后端分层**：`PlaySessionController` 持 `PlayMode { Edit, Building{..}, Playing{..} }`；`PlayBackend` 三实现——`ProcessPlayBackend`（P1 子进程）、`EmbeddedSessionPlayBackend`（P2 副 session）、现 backend 改名 `PluginBridgeActivation` 降为 Playing 进出流程的一个步骤。
2. **P1 子进程 Play**：快照落盘 → `runtime_preview` 参数组装 → spawn/监控/stop；stdout/stderr 回流 17 日志；崩溃 → `PlayCrashed`。
3. **P2 进程内 PIE**：副 session（Runtime 档）+ 序列化注入（**即便进程内也走 DTO**，与子进程路径同构）；PIE 视口文档 tab；结束销毁，编辑世界零污染。
4. **PIE 域实时同步（Unity 对齐读半程）**：Playing（P2）期间 hierarchy/inspector/视口选中切换到 play 域数据源——02 协议对副会话 attach（watch/query/invalidation 全套复用，契约测试双会话复跑），运行时 spawn/despawn 实时进 hierarchy（合帧 + 节流）；inspector 对选中实体值级节奏拉取；SelectionModel 双域隔离。
5. **运行时编辑（Live Edit，Unity 对齐写半程）**：Playing 期间场景编辑事务路由到 play 域 volatile 历史（03 `HistoryContextId::PlaySession(PlayInstanceId)`，会签件），直接作用运行世界，undo/redo 在 Play 期间可用；退出 Play 随副本整体丢弃；显式「保留运行时更改」单实体回写命令为唯一例外（UE Keep Simulation Changes 对齐）。edit 域按 `PlayEditPolicy` 分级保护（被运行文档锁定、其余 pending）。
6. **Play/Simulate 两档**；**Building 前置**（消费 13 编排器，`play_after_build` 照 Fyrox）。

## 非目标

- 不做外窗嵌入（P1 独立窗口）；不做联网 PIE；**P1 子进程本期不可 attach**（缺的是管道/socket 传输层而非模型——`WorldSyncProtocol` 传输无关，godot Remote 场景树式的子进程 attach 列为预留扩展）；play 域改动**不自动回写** edit 域（仅显式单实体「保留运行时更改」命令）；不做 play 期间的资产级 live 编辑（材质/脚本热改走既有热重载通道，不入本计划）；多 PIE 实例只留接口。

## 架构设计

### 模块布局

```
zircon_editor/src/core/play/
  mod.rs
  controller.rs        # PlaySessionController + PlayMode + 迁移表
  backend.rs           # PlayBackend trait + PlayStartRequest/ActiveBackend/PlayError
  process_backend.rs   # P1
  session_backend.rs   # P2
  plugin_activation.rs # 现 backend 迁入改名（enter/exit 语义与双进保护原样保留）
  snapshot.rs          # 快照落盘/清理（.zircon/play/<uuid>/）
  live_link.rs         # play 域链路：副会话 gateway 暴露 + 02 watch attach/detach + 泵登记（M3/M4）
  edit_policy.rs       # PlayEditPolicy 分级判定（play 域放行 / 被运行文档锁定 / 其余 pending）
  pending_edits.rs     # Playing 期间 edit 域编辑意图暂存（分级表第三档）
```

### 关键类型与控制器 API

```rust
pub enum PlayMode {
    Edit,
    Building { play_after_build: bool },     // 构建队列归 13 编排器，此处只持状态
    Playing  { instances: Vec<PlayInstance>, attached: Option<PlayInstanceId> },  // 本期 instances 长度恒 1
}
pub enum PlayKind { Play, Simulate }
pub struct PlayInstance {
    pub id: PlayInstanceId,          // 会话内单调发号
    pub kind: PlayKind,
    pub backend: ActiveBackend,
    pub attach: AttachState,         // Detached | Attached(PlayDomainLink)——同步是实例属性，不是启动模式
}
pub enum ActiveBackend {
    Process { child: PlayChildHandle, alive: Arc<AtomicBool> },   // Fyrox Play 变体直译；attachable()=false（本期）
    Session { session: SecondarySessionHandle, viewport_doc: DocumentId },  // attachable()=true（进程内 gateway 现成）
}
// PlayDomainLink（live_link.rs，M3 起）：play 域 gateway 句柄 + 已注册 watch token 集 + 泵登记凭据

impl PlaySessionController {
    pub fn mode(&self) -> PlayModeKind;                       // 轻量枚举供 WhenClause（08）
    pub fn request_play(&self, kind: PlayKind) -> Result<(), PlayError>;   // 需要构建时先入 Building
    pub fn request_stop(&self) -> Result<(), PlayError>;
    pub fn on_build_finished(&self, ok: bool);                // 13 编排器回报入口
    pub fn poll_backend(&self);                               // 每帧：进程存活/退出码检查
    pub fn attach(&self, id: PlayInstanceId) -> Result<(), PlayError>;  // 旧 detach → 新 attach（M3）；不可 attach 后端返回 Err
    pub fn detach(&self);                                     // 仅断链路，实例继续运行
    pub fn play_gateway(&self) -> Option<Arc<dyn EditorRuntimeGateway>>;  // 当前 attached 实例的 gateway；无 attach 恒 None
}
```

状态迁移表（单测逐格断言，非法格返回 `PlayError::InvalidTransition`）：

| 当前 \ 事件 | request_play | build_ok | build_fail | request_stop | backend_died |
| --- | --- | --- | --- | --- | --- |
| Edit | →Building（需构建）或→Playing（免构建） | — | — | no-op | — |
| Building | 置 `play_after_build=true` | →Playing（若 play_after_build）否则→Edit | →Edit + `BuildFailed` | →Edit（撤销排队） | — |
| Playing | Err（已在玩） | — | — | →Edit（有序停止） | →Edit + `PlayCrashed{exit_code}` |

事件（`PlayStarted/PlayStopped/PlayCrashed/BuildFailed`）经 01 bus `ModeMessage::PlayStateChanged` 族广播；工具栏/编辑保护/日志面板全为订阅者。

### 进入 Playing 的编排序（P1/P2 共用骨架，差异只在第 4 步）

1. 03 `saved_top` 判定脏文档 → `snapshot.rs` 将脏场景 `LevelSystem::snapshot()` 序列化落 `.zircon/play/<uuid>/`（11 契约格式；干净文档直接引用源文件路径）；
2. `PluginBridgeActivation.enter(project_root)`（现 backend 语义：加载工程 native 插件 + 活性快照切换；诊断入 17 日志）；
3. `PlayEditPolicy` 上闸（分级表见「PIE 域与运行时编辑」节；M1 期无 play 域，等价于全量拒绝 + `pending_edits`）；
4. backend 启动：
   - **P1**：组参数 `runtime_preview --project <root> --runtime-session-profile runtime --play-scene <路径> --play-report-pipe <名>`（后两 flag 与 16 M1 联合定稿）→ spawn；监控走 14 job 门面 Process 类别，等待退出码；
   - **P2**：`SessionGateway.create_session(Runtime)` → `load_world_payload(session, payload)`（runtime 侧 dynamic_api 增项，见下）→ 视口文档创建（`bind_viewport_surface` 离屏，每帧 `tick_frame + capture_frame/present_viewport`）→ **auto-attach**（`live_link.rs`：对副会话注册 02 watch + 泵登记 play 域；此后可随时 detach/re-attach，实例不受影响）→ hierarchy/选中切 play 域（M3 起）；
5. 广播 `PlayStarted`。

退出（request_stop / backend_died 共用）：backend 停止（P1 置 alive=false + 平台 kill；P2 先 **live 链路 detach**——watch 注销、play 域历史整栈 finalize 清空、hierarchy/选中切回 edit 域（play 选中按 `EntityId` 锚映射，运行时 spawn 实体无锚即丢弃）——再 `destroy_session` + 关视口文档）→ `PluginBridgeActivation.exit()` → 快照目录清理 → `PlayEditPolicy` 落闸 → `pending_edits` 非空则弹「应用/丢弃」决策（17 通知 Decision 类）→ 广播 `PlayStopped/PlayCrashed`。

**顺序不变量**：2 先于 4（插件活性在游戏世界起来前就位）；exit 时逆序。零污染断言：步骤 1 前后与退出后编辑 `LevelSystem::snapshot()` hash 三点一致——**M4 起扩展第四点**：含 live edit 的整轮 Play 后 edit 域 hash 仍与 Play 前一致（live edit 只作用副本）。

### PIE 域与运行时编辑（Unity 对齐设计，M3/M4 落地）

#### 启动形态与 attach 模型（定形：一个 Play 概念，同步是 attach 属性）

**不存在「同步版 Play」与「独立版 Play」两种启动模式。** Play 只有一个概念（`PlayInstance`），差异由两个正交属性表达：

- **backend（怎么跑）**：`Session`（进程内副 session，P2）或 `Process`（子进程，P1）。
- **attach（编辑器是否挂上去）**：`Attached` 时该实例的世界喂 hierarchy/inspector 并接收 live edit；`Detached` 时实例照常运行（tick/视口呈现不受影响），只断数据与编辑链路。attach/detach 可在运行中随时切换，无需重启实例。

可 attach 性由 backend 的传输能力决定：`Session` 后端进程内 gateway 现成，恒可 attach；`Process` 后端本期不可 attach——**这是传输缺口而非模型限制**：`WorldSyncProtocol` DTO 全 serde、传输无关（02 与 bevy BRP 同构），对子进程经管道/本地 socket 跑同一协议即可解锁（godot「Remote 场景树」正是「子进程 + 调试协议 attach」的成品参照），列为预留扩展不入本期。

默认 UX 映射（Unity 直觉不变）：

- 工具栏 **Play/Simulate** = 创建 `Session` 实例并 **auto-attach**——用户感知即 Unity 的 Play。
- **「以独立进程运行」**（菜单/命令）= 创建 `Process` 实例，Detached 盲跑（本期等价 godot F5 体验）。
- **多实例**：`Vec<PlayInstance>` 允许并存（如一个 attached 的 Session 实例 + 一个 Process 盲跑验证真实进程行为）；**attach 同一时刻至多一个**（hierarchy/inspector 单焦点），`attach(id)` 切换焦点 = 旧实例 detach + 新实例 attach；play 域历史按实例隔离（`PlaySession(PlayInstanceId)`），**detach 不清历史、实例停止才清**。本期实装单实例，多实例只留接口与上述不变量。

#### 远程 attach 预留（发布产物的运行时调试，非本期）

attach 模型天然外推到「远程 attach 已发布游戏」（Unity development build 连接 / godot remote debug 同类能力，覆盖 Android / Windows standalone / Web 发布）：`PlayInstance` 未来增 `Remote { transport }` 后端——实例生命周期不归编辑器（**连接**而非 spawn，stop 语义降级为「断开/请求退出」），attach/detach、双域数据源、按实例历史等语义与本期完全一致。前提与形态：

- **传输与连接方向**：统一取**反向连接**（游戏连编辑器，godot 模型）——Windows standalone TCP/管道皆可、Android 经 adb forward 或局域网、Web 只能 WebSocket 且浏览器不可监听；反向连接是三平台的最大公约数。编辑器侧起 debug 监听服务（归属 16/17 届时定）。
- **协议前提 = 本期守住的前向兼容闸门**：
  1. `WorldSyncProtocol` DTO 保持全 serde、不含进程内指针（02 已约束，勿破）；
  2. 写半程需要**序列化突变 DTO**（BRP `world.insert_components` 同类）——本期 play gateway 的命令下发若走进程内捷径，必须收敛在 gateway 实现内部，`EditCommand` 面不得感知传输形态；
  3. attach 握手需「协议版本 + 工程 id + 场景 `format_version`」协商——远程世界可能来自旧版发布，锚失配时显式降级只读而非静默错配；
  4. 节流参数（invalidation 批频 / inspector 拉取频率）设置化（M3 已列），远程链路直接复用调低。
- **发布侧闸门（15 会签）**：debug attach 服务作为 runtime feature/插件**仅在 development 类导出档启用**，默认鉴权 token + 本机/adb 通道；正式发布档编译期剔除——不在生产环境留世界控制端口。
- **能力分级**：远程 attach 首期目标 = 只读检视（hierarchy/inspector live）；live edit 与「保留运行时更改」依突变 DTO 与握手成熟度分期解锁。

#### 双域模型

```rust
pub enum WorldDomain { Edit, Play }   // core/play/live_link.rs；02 泵、watch_map、SelectionModel、03 routing 共用词汇
```

- **数据源双通道**：`EditorContext.gateway` 恒为 edit 域；play 域 gateway = **当前 attached 实例**的 gateway（`PlaySessionController::play_gateway()`，无 attach 恒 `None`；本期即 P2 的 `SessionGateway` 实例）。02 契约面（query/watch/unwatch/drain_invalidations）对两域**同一 trait 同一语义**——这是 02「契约测试双实现复跑」的直接收益。
- **零 runtime 增项的读半程**：02 `SubscriptionTable` 挂在 `RuntimeDynamicSession` 旁（session 级字段），副会话创建即自带一张订阅表、销毁即回收——attach 只是编辑器侧对 play gateway 发 `watch`，runtime 不需要为 live 同步新增任何东西（`load_world_payload` 之外无第二个会签件）。

#### 数据面（读半程：运行世界 → hierarchy/inspector）

- 02 编辑器泵扩展为**双源 drain**：每帧依次 `drain_invalidations` 两个 gateway，`InvalidationBatch` 入 bus 前标注 `domain`；`watch_map` 键升级为 `(WatchToken, WorldDomain) → ViewInstanceId`（token 由各自 session 发号，跨域可能撞号）。
- **hierarchy 切域（Unity 同面板语义）**：Playing（P2）期间 hierarchy 面板数据源整体切到 play 域（顶栏运行标记 + play 模式着色），退出切回 edit 域——不做双根并排。运行时 spawn/despawn 经 `WorldFact` 实时入面板，行级 diff 复用 02 的 `entity` 锚 + `subtree_hash`。
- **spawn 风暴节流**：`InvalidationBatch` 本身按帧合并（02 tick 末 flush）；编辑器侧再加「hierarchy 重建最小间隔」（设置化，默认约 100ms，17 设置项）——弹幕类场景每秒数百 spawn 时面板降频重建而非逐 fact 重建。
- **inspector 值级节奏拉取**：02 明确值级 change-tick 不在其本期（结构级世代先行），play 域 inspector 不等它——对 play 域选中实体按节奏（默认每 UI 帧，可降频设置）调 `inspect_fields(entity)`，`generation_hint` 短路结构未变情形；值抖动（gameplay 每帧改值）属预期展示行为。
- **PIE 视口拾取**：视口点击经 play gateway 的拾取查询产 play 域选中（05 拾取通道域参数化）；`SelectionModel` 双域各持一份选中集（05 会签），bus `Focus` 族事件携带 domain。

#### 写半程（Live Edit：编辑器 → 运行世界）

- **03 routing 前插一档（会签件）**：`HistoryContextId` 增 `PlaySession(PlayInstanceId)` 变体（一开始就按实例参数化，多实例接口免改契约）；判定序第 0 步——目标实体属 play 域 → `PlaySession(attached 实例 id)`。命令 `apply` 经 play gateway 下发（帧边界应用，00 §4 线程模型不变），命令实现与 edit 域**同一族**（05 场景命令域无感，`EditContext` 注入哪个 gateway 就作用哪个世界）。
- **volatile 历史生命周期（按实例）**：`PlaySession(id)` 历史随该实例首笔 play 域事务而建、**随该实例停止**整栈 `finalize` 清空；detach/attach 切换不清空（切回可继续 undo）；**无 `saved_top` 概念**——play 域永不产生文档脏态，不触发标题星号/关闭拦截/autosave（03 `is_dirty` 对 `PlaySession` 恒 false）。Play 期间 Ctrl+Z 焦点在 PIE 视口/play 域面板时路由到当前 attached 实例的 `PlaySession` 历史（Unity 亦如此）。
- **跨域守卫**：play 域实体禁入 edit 域历史、edit 域实体禁入 `PlaySession`——03 风险节「`ContainsPieObjects` 等价（04 会签）」在此落定：routing 按 domain 硬分流 + 显式守卫单测，跨域混合事务返回 `EditCommandError::InvariantViolation`。
- **`PlayEditPolicy` 分级表**（`edit_policy.rs`，Playing 期间生效）：

| 编辑目标 | Playing 期间行为 |
| --- | --- |
| play 域场景实体（hierarchy/inspector/PIE 视口 gizmo） | **放行** → `PlaySession` volatile 历史 |
| edit 域·被运行场景对应文档 | **锁定**：03 `begin` 返回 `Err(PlayModeActive)`（防止磁盘态与快照态分叉） |
| edit 域·其它文档 / 资产 / 工程级操作 | M1 保守入 `pending_edits`；M4 后依证据逐类放宽（Unity 允许 Play 期间改资产，zircon 待热重载互扰证据） |

- **显式回写「保留运行时更改」**（UE Keep Simulation Changes 对齐，唯一回写通道）：hierarchy 右键/命令面板命令，取 play 域选中实体 → play gateway `inspect_fields` 抓取组件状态（11 契约序列化） → 构造 **edit 域普通事务**（可撤销）按 `EntityId` 锚写回同一实体。运行时新 spawn 实体无 edit 域锚，命令拒绝并提示；被运行文档的锁定对该命令豁免（它就是为此存在的）。

#### 端到端时序（live edit 一拍，验收剧本同款）

```
PIE 视口拖 gizmo → 05 工具（play 域）→ TransactionScope(PlaySession) push set_transform
→ play gateway 排队 → 副 session 帧边界 with_world_mut 应用
→ 副 session SubscriptionTable 打点 → tick 末 flush InvalidationBatch(domain=Play)
→ 泵双源 drain → mark_view_dirty(hierarchy/inspector·play 域)
→ 面板 query(play gateway, generation_hint) → 行级 diff 刷新
```

**不变量**：play 域事务零文档脏态；live edit 不触碰 edit 域世界（零污染第四点断言）；watch DTO / play 载荷不含 authoring 词（既有守卫矩阵扩展声明）。

### runtime 侧扩项（owner 会签件）

`load_world_payload(session_handle, buffer) -> status`：dynamic_api 追加 v1.1 表项（不改既有 11 指针语义），session 内实现 = 反序列化 + `replace_world_and_reset_runtime_state`（现成方法，`level_system.rs:119`）。被否兜底：`create_session` 参数带快照文件路径注入，牺牲内存直传。

**注入的 `EntityId` 稳定性附加会签**：序列化注入往返必须保持 edit 域实体 id 在 play 域稳定（注入往返测试断言 id 集合一致）——双域选中映射、跨域守卫、「保留运行时更改」回写三者共用该锚。若序列化路径不能保 id，则 runtime 侧需在注入报告中回传 `旧 id → 新 id` 映射表（DTO 走 11 契约），编辑器侧 `live_link.rs` 持表换算；两案择一在 M2 切片 2.1 定稿并记状态节。

### 现物迁移映射

| 现物 | 去向 |
| --- | --- |
| `EditorRuntimePlayModeBackend` trait + Noop/NativePlugin 实现 + 既有单测 | `plugin_activation.rs`（改名 `PluginBridgeActivation`，enter/exit/双进保护/宽容退出语义与测试原样迁移）；原 trait 名删除 |
| `EditorEventRuntimeState.runtime_play_mode_backend` + `set_runtime_play_mode_backend` setter | 随 01 M1 拆解移交 `PlaySessionController`（排程与 01 M1 对齐） |
| `create_startup_runtime_backend`（runtime_backend.rs:10-33） | 改装配 `PlaySessionController`（`NativePluginLiveHost` 构造保留，注入 activation） |

## 里程碑

### M1 状态机与 P1 子进程 Play

- 切片 1.1：`core/play/` 骨架 + 状态机 + 迁移表单测 + 现 backend 迁移改名（上表三行硬切换）。
- 切片 1.2：`ProcessPlayBackend`：快照落盘/参数组装（16 M1 联合）/spawn/监控/stop/崩溃事件；日志回流（17 未落地前暂投 bus）。
- 切片 1.3：编辑保护 + `pending_edits` + 退出决策提示。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`（迁移表逐格/参数组装/快照清理/pending 队列，spawn 用假可执行夹具；`plugin_activation` 既有 roundtrip 测试迁移后须过）；手验启停三轮无孤儿进程、快照目录无残留。更新 `docs/zircon_editor/core/play.md`。

### M2 P2 进程内 PIE

- 切片 2.1：runtime 侧 `load_world_payload`（会签落地；`cargo test -p zircon_runtime --lib --locked` 注入往返测试）。
- 切片 2.2：`EmbeddedSessionPlayBackend` + PIE 视口文档（bind/tick/capture/present 链）；销毁与零污染三点 hash 断言。
- 切片 2.3：Simulate 档（输入不转发 + `RuntimeCameraController` 编辑相机注入）。
- 测试阶段：`cargo test -p zircon_runtime --lib --locked` + `cargo test -p zircon_editor --lib --locked`；验收：启停 10 轮 session 计数归零；编辑世界 hash 不变；authoring token 守卫不回归（PIE 载荷走 11 契约不含 authoring 词）。

### M3 PIE 域实时同步（Unity 对齐读半程）

前置：02 M2/M3（泵与 `SessionGateway` 协议实现）就绪。

- 切片 3.1：`live_link.rs`（play gateway 暴露 + attach/detach 编排）；02 泵双源 drain + `InvalidationBatch` 域标注 + `watch_map` 键域化；02 契约测试对副会话复跑。
- 切片 3.2：hierarchy Playing 切 play 域（运行标记/着色/退出切回）+ 行级 diff 跟随 spawn/despawn + 重建最小间隔节流（设置项挂 17）。
- 切片 3.3：inspector play 域值级节奏拉取（默认每 UI 帧、可降频，`generation_hint` 短路）；SelectionModel 双域 + PIE 视口拾取（05 拾取通道域参数化）+ 停止时选中锚映射。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`；验收：P2 运行持续 spawn 夹具场景（每帧 spawn 若干实体），hierarchy 行数实时跟随且节流生效（重建频率 ≤ 设置间隔，计数器断言）；停止后 hierarchy 回 edit 域且行集与 Play 前一致；watch attach/detach 与 session 销毁生命周期矩阵（02 矩阵对 play 域复跑）；泵每帧对每 gateway 至多一次 drain 断言。

### M4 运行时编辑（Live Edit 写半程）与多实例接口

前置：03 M2（场景命令族纯化迁移）就绪。

- 切片 4.1：03 `HistoryContextId::PlaySession` 会签落地——routing 第 0 步域分流 + volatile 历史生命周期（Playing 建、停止整栈 finalize 清空、`is_dirty` 恒 false）+ 跨域守卫（03 风险节「`ContainsPieObjects` 等价」在此闭环）。
- 切片 4.2：`edit_policy.rs` 分级表实装（play 域放行 / 被运行文档锁定 / 其余 pending——M1 全量拒绝逻辑收窄为第二、三档）；PIE 视口 gizmo 与 play 域 inspector 编辑走 `PlaySession` 事务（05 工具与 `EditContext` 域参数化）；Play 期间 Ctrl+Z 焦点路由。
- 切片 4.3：「保留运行时更改」显式命令（单实体，运行时 spawn 实体拒绝提示，产 edit 域可撤销事务）；多实例接口化——`Vec<PlayInstance>` + `attach(id)/detach`（单焦点不变量、按实例历史隔离、detach 不清/停止清入单测），UI 不实装。
- 测试阶段：`cargo test -p zircon_editor --lib --locked`；验收矩阵——play 域编辑→undo→redo 往返；停止后 play 历史零残留 + 零污染**第四点**（含 live edit 整轮后 edit 域 hash 不变）；跨域混合事务 `InvariantViolation` 单测；分级表三档行为逐档断言；回写命令产生 edit 域可撤销事务且值与运行世界抓取一致；gameplay 覆写下 undo 目标已消失 → `TargetMissing` 不中断（通知降级）断言。证据记状态节。

## 风险与开放问题

- 同进程双 `CoreRuntime` 的图形设备共享未证实：不可共享则 P2 视口降级 `capture_frame` 离屏拷贝呈现（可接受兜底）；实测证据记状态节。
- `PluginBridgeActivation.enter` 内含插件加载 IO（现语义），进 Play 延迟可感——M2 后评估把插件加载前移到工程打开时、enter 只做活性切换（需 runtime plugin owner 会签，记债）。
- Play 期间资产热重载与快照世界互扰：M1 一律暂停编辑侧导入任务（14 互斥类别），M3 依证据放宽。
- `pending_edits` 的「退出后应用」可能撞上世界结构已变（Play 前保存的场景 vs 退出后编辑世界相同，理论安全；但 pending 中的命令引用的实体若被同队列前项删除会失败）——应用时逐条 try，失败项列入决策对话报告。
- **gameplay 每帧覆写 live edit 值**：脚本/动画系统持续写同一字段时，编辑器的 set 下一帧即被覆盖，「看似编辑无效」——Unity 同语义，**不做写保护**，文档化为预期行为；inspector 值抖动经降频设置缓解。
- **play 域 undo 与 gameplay 并发变更交错**：revert 目标实体可能已被 gameplay despawn——revert 返回 `TargetMissing` 时不中断（降级为 17 通知），该历史条目标记失效；不追求 play 域撤销的强一致（volatile 历史，成本收益不成比）。
- **hierarchy 面板规模上限**：02 以 5k 节点为设计目标，弹幕类 gameplay 可能数万实体——节流 + `subtree_hash` diff 之外，若实测仍超帧预算，降级方案为 play 域 hierarchy 仅展开路径按需查询（`Subtree` watch 只挂已展开节点）；实测证据记状态节。
- **`EntityId` 注入稳定性被否**：若序列化往返不能保 id 且映射表方案实施成本高，则双域选中映射与「保留运行时更改」按名称+路径弱锚降级（准确性打折，明示于命令提示）；裁决记 M2 状态节。
- **live edit 与 02 值级通道的时序耦合**：02 组件值级 change-tick 明确不在其本期，play 域 inspector 靠节奏拉取兜底——若 03 提交事件通道（02 风险节预告）先行落地，M3 切片 3.3 改订阅制并删除轮询路径，执行时依 02/03 实际进度取舍。

## 产出记录与时间

| 日期 | 里程碑/切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-12 | Editor08 M1.2 失败移交：`CommandEvalCtx` Building/Play 权威投影 | 待修复（open） | Editor08 已落地类型化 `PlayMode` when 谓词，但当前 Chrome 投影只能从 `EditorSessionMode::{Welcome,Project,Playing}` 生成 `Edit/Playing`，无法表达 `Building`，也尚未以本计划 `PlaySessionController` 为权威源；修复要求与静态复现证据见 [failure 交接](04/failure-2026-07-12-command-eval-play-state-projection.md)。本行仅登记待修复，不声明本计划完成。 |
