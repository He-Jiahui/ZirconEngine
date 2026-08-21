---
related_code:
  - zircon_hub/src/main.rs
  - zircon_hub/src/lib.rs
  - zircon_hub/src/error.rs
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/process/editor_handshake/wait.rs
  - zircon_hub/src/build/runner.rs
  - zircon_hub/src/settings/config_path.rs
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/projects/shared_recent_projects.rs
  - zircon_hub/src/state/task_status.rs
  - zircon_hub/src/state/hub_snapshot.rs
  - zircon_hub/web/src/main.tsx
  - zircon_hub/web/src/App.tsx
  - zircon_hub/web/src/tauri/hubApi.ts
  - zircon_hub/web/src/tauri/hubStateValidator.ts
  - zircon_hub/web/src/components/shell/HubWindow.tsx
  - zircon_hub/web/src/components/shell/TopBar.tsx
  - zircon_hub/tauri.conf.json
  - zircon_hub/capabilities/default.json
  - zircon_hub/Cargo.toml
  - zircon_hub/package.json
tests:
  - zircon_hub/tests/app_error_recovery_contract.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
  - zircon_hub/tests/ui_shell_header_contract.rs
  - zircon_hub/tests/ui_shell_window_contract.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/02-web-shell-catalog-settings-team-cloud-accessibility-performance-review.md
  - docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md
  - docs/plans/optimize/zircon_hub/04-command-action-message-delivery-task-history-view-model-localization-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/optimize/zircon_tooling/37-transaction-atomicity-prepare-commit-publish-rollback-compensation-idempotency-crash-recovery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/MonitoredProcess.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/MonitoredProcess.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/godot/editor/project_manager/project_manager.h
  - dev/godot/main/main.cpp
  - dev/Fyrox/project-manager/src/main.rs
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_tasks/src/task_pool.rs
  - dev/bevy/crates/bevy_winit/src/lib.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 05 · Application Host / Bootstrap / Window / IPC / Close / Shutdown / Crash Recovery 工程化差距

## 1. 结论

Zircon Hub当前拥有可启动的Tauri 2宿主、React shell、Rust-owned state、窗口焦点刷新、后台action worker、原子config替换和Editor handshake。这说明它已经越过“静态网页套壳”阶段，相关基础应保留。

但是，当前入口还不是工程级应用宿主。`main()`只转调`tauri_app::run()`；builder在窗口出现前同步执行`HubCommandState::load()?`，随后只监听`Focused(true)`。关闭按钮直接在WebView调用`getCurrentWindow().close()`，Rust侧没有`CloseRequested`、`ExitRequested`、stop-admission、close decision、drain/cancel/detach、final checkpoint或terminal receipt。两个`thread::spawn`均丢弃`JoinHandle`，Editor `Child`也在启动后立即丢弃。配置中虽然存在`HubWindowState`，production却从不读写它。

前后端连接同样只是“能调用”。React分别发起initial snapshot和event subscription，没有原子的`subscribe -> snapshot -> sequence`握手；事件没有host/session/window/generation/sequence身份，subscription失败后也没有重连。native state load或schema validation失败会被`loadHubState()`吞掉并返回`fallbackShellState`，用户可能看到可操作的演示状态，而不是明确的BootFailed或RecoveryRequired。

本轮不新增重复P0。Hub01已经拥有detached worker、Child owner和single-instance/config writer差距；Hub04已经拥有durable operation、shutdown abandonment和effect/receipt P0。本报告把此前缺失的应用宿主调用链补齐，并新增40个P1、12个P2。目标不是复制某个引擎的窗口类，而是建立`process activation -> instance admission -> staged bootstrap -> connected window session -> close decision -> quiesce -> checkpoint -> terminal exit`的单一宿主状态机。正确关闭、崩溃恢复和连接顺序通过前，不能以启动快或窗口轻量宣称优于Unreal。

## 2. 审查范围、证据与边界

本轮冻结32个Zircon文件，共8,988行、325,518 bytes、77项selected test、0 ignored。路径按小写forward-slash排序，对每个文件取SHA-256，再以`path|hash`和LF连接形成manifest；当前工作树fingerprint为`2c9a6aa3bd69ec86e250abbb046ea7370d080e69fe3b4a02e32fbfa765b416ca`。参考侧冻结12个文件、20,983行、756,598 bytes、26项test，fingerprint为`8a5f99b1689c2f8382984375e79a69fdeff54f9c308f5faf22779470706951cc`。

| 子域 | 文件 / 行 / bytes | 测试 | 本轮判定 |
|---|---:|---:|---|
| Native host与build metadata | 9 / 332 / 9,663 | 0 | E3：入口、builder、managed state、window event、Tauri config/capability与依赖面 |
| Worker、process与task status | 10 / 4,974 / 186,430 | 44 | E3：thread/Child ownership、background action、handshake、task projection与host exit交点 |
| Persistence与shared registry | 3 / 1,347 / 46,754 | 12 | E3：startup load、fixed temporary、window/runtime config及跨进程writer边界 |
| React bootstrap、window与IPC | 6 / 637 / 21,536 | 0 | E3：initial load、event subscription、native controls、error fallback与unmount cleanup |
| Focused lifecycle contract tests | 4 / 1,698 / 61,135 | 21 | E2：全部为source/doc字符串合同；0个Tauri runtime harness、0个真实window/exit test |
| Unreal参考 | 3 / 8,030 / 266,851 | reference only | E3核对受管进程取消、进程树终止、thread join、OnExit queue cancellation与全局shutdown |
| Godot参考 | 3 / 7,654 / 300,018 | reference only | E3核对Project Manager close notification、recovery mode和有序`Main::cleanup` |
| Fyrox参考 | 3 / 1,941 / 69,208 | reference only | E3核对CloseRequested确认、owned Child/`try_wait`与LoopExiting settings save |
| Bevy参考 | 3 / 3,358 / 120,521 | 26 reference tests | E3核对typed `AppExit`、pre-exit system set和TaskPool shutdown/join |

检索确认Hub production/test范围内不存在`RunEvent`、`ExitRequested`、`CloseRequested`、`prevent_close`、`single_instance`、`beforeunload`、`visibilitychange`、`shutdown`、`JoinHandle`、`try_wait`或background cancellation实现。`getCurrentWindow`只在`TopBar.tsx`出现3次，`appWindow.close()`只有1处；现有测试反而明确要求这条直接关闭路径存在。

本报告拥有Hub application process、native window、WebView connection和shutdown bridge。Hub01继续拥有具体build/package/install/editor process backend、single-instance store与Child supervisor；Hub02继续拥有页面、通用shell视觉/accessibility、fallback演示数据和CSP；Hub03继续拥有Auth/Cloud/Marketplace；Hub04继续拥有command envelope、durable scheduler、TaskRegistry、effect ledger与message/read model。交叉事实只在canonical owner计数一次。

本轮是review-only。没有运行Cargo、Tauri真窗口、第二实例、OS关机/注销、kill/crash、后台copy/build、DPI/多显示器、网络盘或power-loss测试。静态缺失可证明当前没有相应合同，不能证明未来实现的时延、稳定性或恢复性能。

## 3. 当前值得保留的基础

1. Rust入口返回`Result`，Tauri builder setup错误不会被无条件转成成功退出码。
2. `HubCommandState`集中持有session，未来可演进为application-scoped owner，而不必让React拥有业务真相。
3. window focus refresh使用`AtomicBool`合并重复请求，避免每个focus event都创建并发扫描。
4. background task已把prepare、external run和complete分开，宿主可以在这些边界插入cancellation与shutdown fence。
5. worker panic已有局部`catch_unwind`，单个operation失败不会必然毒化整个进程。
6. config采用同目录staging和Windows `ReplaceFileW(...WRITE_THROUGH)`，可作为checkpoint store的底座。
7. shared recents已有跨进程writer lease和三方合并，说明项目已接受跨进程一致性需求。
8. React initial response通过sequence/generation ref避免部分旧promise覆盖新state，这是连接协议的局部基础。
9. subscription cleanup在React unmount时调用`unlisten`，至少没有永久保留前端listener的设计意图。
10. native window capability当前只开放default/minimize/toggle-maximize/close，没有顺手开放filesystem或shell广权限。

这些优点仍是局部机制。AtomicBool不是lifecycle state，原子替换不是shutdown transaction，React ref也不是跨IPC的server generation。

## 4. 当前实现事实

### 4.1 Bootstrap是一次同步构造，不是可观测状态机

`main.rs`只有4行有效入口，`tauri_app::run()`只有一条builder链。`.manage(HubCommandState::load()?)`在Tauri event loop和主窗口可恢复UI建立前执行。load会读取Hub TOML、刷新recent project manifest、获取shared registry writer lease、修复registry、扫描source-scoped catalogs并再次persist。任一步失败都会让`run()`返回，release Windows binary又使用`windows_subsystem = "windows"`，没有console、native error window、boot log receipt或Recovery UI保证用户看得到原因。

初始化没有phase、deadline、cancel、retry、degraded policy或reverse rollback。`HubRuntimeSession::load_from_paths`还在纯“load”流程末尾主动persist，使读配置、repair、catalog discovery和写配置混成一个不可分阶段。应用无法回答当前处于AcquiringInstance、LoadingConfig、Repairing、Scanning、CreatingWindow、Connecting还是Ready。

### 4.2 原生窗口关闭绕过后端所有权

Tauri只通过`.on_window_event`处理`Focused(true)`，没有匹配CloseRequested、Destroyed、Moved、Resized、ScaleFactorChanged或主题/电源生命周期。`TopBar`的close handler直接调用`appWindow.close()`；capability又直接允许close。后端没有机会冻结admission、检查running/queued operation、展示close decision、持久化窗口状态、停止listener、排空worker或记录exit reason。

`runWindowAction`把返回的Promise直接丢给`void`，minimize/maximize/close rejection没有error surface或receipt。关闭按钮也没有pending/reentrancy guard；用户重复点击、OS标题栏关闭、Alt+F4、任务栏退出、系统关机和未来tray quit不会进入同一个协议。

### 4.3 Worker和Child没有接入application lifetime

focus refresh与background worker都使用detached `thread::spawn`，`HubCommandState`不保存JoinHandle、CancellationToken、worker count或quiescence barrier。线程clone `Arc<Mutex<HubRuntimeSession>>`和`AppHandle`，但没有host generation；窗口关闭后仍可能持锁、persist或emit。emit结果一律忽略，无法区分正常window gone和IPC故障。

Editor启动返回`Child`后立即赋给`_child`并drop；empty editor同样只保留PID。这里的具体Child owner由Hub01修复，本报告关心的是application close必须消费统一supervisor快照并决定Cancel、Drain、Detach-with-owner-transfer或Block。当前宿主既不知道有哪些线程/进程，也没有stop-admission时刻，因此无法给出可靠close receipt。

### 4.4 Frontend连接没有无缝snapshot/event切换

`App.tsx`用两个独立effect分别调用`loadHubState()`和`subscribeHubStateChanged()`。本地generation可以阻止部分旧initial response覆盖已经apply的event，但无法覆盖“snapshot已取出而listener尚未安装”或“listener已安装但event没有server sequence”的窗口。事件payload没有host instance、window session、state generation或sequence，WebView也无法检测duplicate、gap、stale host和out-of-order delivery。

subscription失败只写一条warning，生命周期内不重试、不backoff、不重新snapshot，也没有Connected/Degraded/Reconnecting/Disconnected状态。invalid event payload只console.warn并丢弃；连续schema mismatch不会隔离连接、展示升级冲突或请求full resync。React unmount会unlisten，但native close不等待unlisten acknowledgement。

`loadHubState()`把invoke失败与validation失败合并捕获并返回`fallbackShellState`。fallback包含可浏览页面和演示内容，调用者不知道是浏览器预览、后端启动失败、协议skew还是数据损坏。这一truth问题由Hub02计数，本报告要求host connection不得把它标成Ready。

### 4.5 Window state和session recovery只有数据结构

`HubConfig`包含position、size和maximized字段，但全production只有声明与default，没有读取、更新或应用。窗口永远使用Tauri config固定1568x1003并center；移动、缩放、最大化、DPI变化和多显示器拓扑均不保存。结构存在会让调用方误以为restore已完成，实际只是dead persisted schema。

没有process instance ID、window session ID、activation sequence、clean-shutdown marker、last exit reason或boot attempt。崩溃后task status、worker active和queue全部重置，宿主无法区分正常关闭、强杀、panic、OS终止和更新重启。Godot式project recovery mode不能照搬，但“上次启动未完成”这一事实必须存在，并能选择safe/recovery boot而非重复进入同一失败路径。

### 4.6 测试固定了外观和源码形状，没有资格证明生命周期

四个focused integration文件共有21个tests、7次`read_to_string`、59处`.contains()`和33次`assert_contains` helper使用，0次Tauri runtime harness。它们检查窗口尺寸、capability字符串、React组件拆分、本地化copy和`appWindow.close()`源码片段。

`frameless_window_controls_call_tauri_current_window_actions`与`topbar_window_controls_are_bound_to_tauri_current_window_api`会在未来把直接close改为graceful request时误红，却不能证明真实窗口能关闭、拒绝关闭、等待任务或在失败时恢复。没有第二实例、CloseRequested/ExitRequested、worker-close race、listener gap、crash marker、window restore、monitor removal、DPI change、system shutdown或exit-code测试。

## 5. 参考源码约束

### 5.1 Unreal

`FMonitoredProcess`持有process handle、monitor thread、pipes和terminal delegates。析构时若仍运行会`Cancel(true)`，随后`WaitForCompletion()`；取消路径终止process tree、读取最后pipe bytes、关闭pipe并发布cancel terminal。`FSerializedUATProcess`还把queue cancellation绑定到`FCoreDelegates::OnShutdownAfterError`和`OnExit`。Zircon不必复制类层次，但所有后台process/thread必须能被application exit发现、停止并等待。

`LaunchEngineLoop`的全局退出与cleanup范围远大于Hub，本报告只引用它证明“request exit”与“立即销毁窗口/进程”不是同一阶段，不把完整engine teardown强塞进launcher。

### 5.2 Godot

Project Manager显式接收`NOTIFICATION_WM_CLOSE_REQUEST`，在退出前更新窗口表现；项目启动前还检查last startup recovery状态并可传`--recovery-mode`。`Main::cleanup`按顺序shutdown extension、清thread load task、移除成功lock、flush queue、停止language worker、同步render、finalize server/OS/display，最后处理restart-on-exit。

Godot Project Manager本身也不是durable Hub scheduler。应借鉴的是close notification、clean/unclean marker、recovery admission和有序cleanup，不是它的singleton或具体UI。

### 5.3 Fyrox

Fyrox Project Manager在winit `CloseRequested`中调用`request_close`。若当前处于build mode并持有Child，会弹确认而非直接退出；运行中的Child保存在`Mode::CommandExecution`并以`try_wait`轮询。`LoopExiting`再保存settings。它仍缺少Zircon所需的durable journal，但已经明确区分window close intent、child activity和terminal loop exit。

### 5.4 Bevy

Bevy把退出建模为typed `AppExit`，提供`OnAppExitSystems`作为退出前系统集；`TaskPool::drop`关闭shutdown channel并drain/join全部worker threads。Zircon Hub不需要ECS schedule，但需要同等可观察的pre-exit phase、typed outcome和owned worker join。

### 5.5 Unity Graphics适用边界

本地`dev/Graphics`是SRP/Graphics源码，不含Unity Hub、Editor application host、native launcher或Player process lifecycle权威实现。本报告不从闭源缺口推断Unity行为，也不把render package utility当作window/IPC参考。

## 6. 目标架构与Owner

```text
ProcessActivation
  -> HubInstanceCoordinator
  -> HubBootstrapPipeline
  -> WindowSessionRegistry
  -> HubConnectionCoordinator
  -> HubLifecycleCoordinator
       -> OperationShutdownBridge
       -> ProcessSupervisor
       -> HostCheckpointStore
       -> CrashRecoveryService
  -> ExitReceipt
```

| Owner | 唯一职责 | 禁止承担 |
|---|---|---|
| `HubInstanceCoordinator` | process/session identity、single/multi-instance policy、second-launch activation与writer admission | 不保存业务project catalog |
| `HubBootstrapPipeline` | phase、deadline、cancel、retry、degraded/recovery和reverse rollback | 不在constructor里隐式完成全部I/O |
| `WindowSessionRegistry` | native window identity、geometry/DPI/monitor、focus/visibility、close source与session generation | 不直接执行build/package |
| `HubConnectionCoordinator` | subscribe-snapshot handshake、sequence/gap、schema/capability negotiation、reconnect/resync | 不用fallback data伪装backend Ready |
| `HubLifecycleCoordinator` | process/window状态机、stop admission、close decision、quiesce/checkpoint/terminal exit | 不拥有domain operation细节 |
| `OperationShutdownBridge` | 聚合Hub04 TaskRegistry和Hub01 ProcessSupervisor的shutdown snapshot/decision/receipt | 不复制task或Child registry |
| `HostCheckpointStore` | window/session/clean-exit checkpoint、generation/CAS、bounded atomic write | 不与action history共用自由文本协议 |
| `CrashRecoveryService` | unclean marker、boot failure streak、safe/recovery mode、orphan reconcile入口 | 不把未知副作用自动标成成功 |

宿主状态至少包含：

```text
Cold
  -> AcquiringInstance
  -> Loading
  -> CreatingWindow
  -> Connecting
  -> Ready
  -> CloseRequested
  -> Quiescing
  -> Checkpointing
  -> Closing
  -> Closed

Loading/Connecting -> Degraded | RecoveryRequired | BootFailed
Quiescing/Checkpointing -> CloseBlocked | ForcedExit
```

所有关闭来源必须进入同一个`CloseIntent { source, requested_at, force, deadline }`。只有Lifecycle Coordinator发布`ClosePermit`后native window才可真正close；forced OS termination无法保证完成，但必须留下unclean marker，并在下次启动进入reconcile。

## 7. 已继承的P0阻断项，不重复计数

### ZHUB-HOST-B0-01 · 关闭路径无法drain/cancel/transfer已接收操作

直接`appWindow.close()`和缺失CloseRequested handler证明Hub04 `ZHUB-CTL-P0-03`没有application adapter；detached worker缺少JoinHandle又继承Hub01 `ZHUB-P1-10`。canonical P0仍由Hub04计数，本报告只新增`OperationShutdownBridge`实施与测试门。

### ZHUB-HOST-B0-02 · 退出中断外部effect后没有可恢复terminal receipt

package/install/build在close或crash中断时，宿主没有checkpoint/reconcile阶段；这是Hub04 `ZHUB-CTL-P0-04`和Hub01 backend transaction差距的交叉结果。canonical effect P0不在本报告重复累加。

### ZHUB-HOST-B0-03 · 多Hub writer与fixed temporary没有instance/CAS保护

当前没有single-instance依赖或实现，Hub config又使用固定`.tmp`。该事实已经由Hub01 `ZHUB-P1-04`拥有，本报告只要求second-launch activation和instance lifecycle接入同一owner。

## 8. P1工程化差距

### 8.1 Bootstrap、process identity与启动终态

- ZHUB-HOST-P1-01：入口没有`HubApplicationHost`或等价owner，初始化、run和退出只是一条builder表达式，无法注入阶段、policy和测试替身。
- ZHUB-HOST-P1-02：没有typed BootId、process instance ID、boot attempt和启动phase；日志、IPC、window与operation无法关联同一次宿主启动。
- ZHUB-HOST-P1-03：pre-window load错误只向`main() -> Result`传播，Windows GUI release没有保证可见的native failure surface、support code或recovery action。
- ZHUB-HOST-P1-04：bootstrap没有per-phase deadline、cancel、retry和degraded policy，阻塞writer lease或慢manifest可让用户只看到“没有窗口”。
- ZHUB-HOST-P1-05：load、repair、catalog refresh和persist混在constructor，没有prepare/commit、reverse cleanup和partial-init terminal receipt。
- ZHUB-HOST-P1-06：没有command-line/deep-link/file-association/second-launch activation envelope，project intent不能路由到现有窗口并带稳定request identity。
- ZHUB-HOST-P1-07：没有boot failure streak、safe-mode admission或provider quarantine；同一catalog/plugin/config启动故障会在每次重启重复触发。

### 8.2 Native window与application exit状态机

- ZHUB-HOST-P1-08：Rust只处理`Focused(true)`，没有CloseRequested、Destroyed、Moved、Resized、ScaleFactorChanged及window/session terminal事件。
- ZHUB-HOST-P1-09：WebView直接持有`allow-close`并调用native close，close intent绕过后端lifecycle authority和业务权限/decision层。
- ZHUB-HOST-P1-10：没有`prevent_close`或ClosePermit，running/queued task、dirty settings draft和checkpoint失败均不能阻止销毁。
- ZHUB-HOST-P1-11：没有处理Tauri `RunEvent::ExitRequested/Exit`或等价app-wide event，最后一个窗口关闭、显式quit和OS退出没有统一路径。
- ZHUB-HOST-P1-12：没有close source、reason、force flag、deadline、exit code和terminal status，无法区分user close、update restart、fatal error与OS termination。
- ZHUB-HOST-P1-13：close没有pending/reentrancy/dedup状态；重复点击和多个退出源可并发发起不同副作用。
- ZHUB-HOST-P1-14：minimize/maximize/close Promise被`void`丢弃，capability denial、window gone和platform error不会进入UI或diagnostic receipt。
- ZHUB-HOST-P1-15：没有明确“关闭窗口但保持后台运行”“退出Hub但Editor继续”“终止owned build tree”策略，window lifetime与process lifetime被默认等同。

### 8.3 Worker、process与shutdown bridge

- ZHUB-HOST-P1-16：focus refresh thread没有JoinHandle、cancel或panic containment；panic可让`focus_refresh_pending`永远保持true并永久禁用后续refresh。
- ZHUB-HOST-P1-17：background worker thread没有JoinHandle、worker registry、liveness或terminal callback，宿主无法证明quiescent。
- ZHUB-HOST-P1-18：线程持有`AppHandle`和session Arc却不携host/window generation，旧window关闭或重建后仍可能emit/persist。
- ZHUB-HOST-P1-19：emit结果全部忽略，没有区分normal disconnect、queue full、serialization failure和stale window；terminal state可能只存在内存。
- ZHUB-HOST-P1-20：没有stop-admission fence，CloseRequested之后直到实际进程结束前仍可从WebView提交新的background action。
- ZHUB-HOST-P1-21：没有shutdown snapshot和deadline escalation；宿主不能列出active/queued/attached/detached工作并让用户选择Wait、Cancel、Leave Running或Abort Close。

### 8.4 WebView连接、snapshot/event顺序与重连

- ZHUB-HOST-P1-22：initial load与event listen独立启动，没有server-side subscribe-snapshot token，安装listener的竞态窗口可能漏状态变化。
- ZHUB-HOST-P1-23：event payload没有host instance、window session、schema version、state generation或monotonic sequence，无法识别gap、duplicate、stale和out-of-order。
- ZHUB-HOST-P1-24：subscription失败后没有bounded retry/backoff、重新snapshot或connection health state，应用生命周期内永久退化为command-response刷新。
- ZHUB-HOST-P1-25：invalid event payload只被丢弃，没有schema-skew terminal、quarantine、compatibility提示和full resync。
- ZHUB-HOST-P1-26：前端没有Booting/Connecting/Ready/Degraded/Reconnecting/RecoveryRequired/Closing状态，fallback content可与真实后端状态混淆。
- ZHUB-HOST-P1-27：native window controls绕过typed Hub IPC，没有request/response identity、capability reason、close decision或state generation。
- ZHUB-HOST-P1-28：unlisten只依赖React unmount，native shutdown不等待listener detach，也没有server-side subscription owner/lease和disconnect acknowledgement。

### 8.5 Window session、checkpoint与crash recovery

- ZHUB-HOST-P1-29：`HubWindowState`是dead production schema；position、size和maximized从不采集、persist或restore。
- ZHUB-HOST-P1-30：window config没有monitor identity、DPI/scale、work area、display topology generation和off-screen clamp，直接恢复旧坐标会有不可见窗口风险。
- ZHUB-HOST-P1-31：geometry没有debounce、generation/CAS和final checkpoint；resize storm若直接接现有全TOML persist会放大I/O和writer竞争。
- ZHUB-HOST-P1-32：没有clean-shutdown marker、last exit reason、last completed phase和checkpoint generation，启动无法可靠判断unclean termination。
- ZHUB-HOST-P1-33：没有host-level RecoveryRequired read model和safe boot入口；config/catalog/window/operation恢复只能以普通error或演示fallback呈现。
- ZHUB-HOST-P1-34：没有restart handoff token和activation acknowledgement，更新、语言/渲染设置重启或self-repair无法证明新实例接管成功。

### 8.6 Test architecture与qualification

- ZHUB-HOST-P1-35：focused 21 tests全部读取源码/文档，没有Tauri mock runtime、真实AppHandle、WebviewWindow或event-loop行为测试。
- ZHUB-HOST-P1-36：现有合同正向锁定`appWindow.close()`，会阻碍close request/permit重构，却不能证明任何shutdown语义。
- ZHUB-HOST-P1-37：没有CloseRequested、ExitRequested、last-window、Alt+F4、taskbar quit、system shutdown、duplicate close和forced exit矩阵。
- ZHUB-HOST-P1-38：没有worker/refresh/emit/persist与close的deterministic race、loom/model test、fault injection或bounded timeout。
- ZHUB-HOST-P1-39：没有subscribe-snapshot gap、event duplicate/reorder/drop、schema skew、reconnect和WebView reload测试。
- ZHUB-HOST-P1-40：没有clean/unclean marker、boot-loop、safe mode、window off-screen/DPI/monitor removal、second-instance activation和exit-code端到端资格门。

## 9. P2完善项

- ZHUB-HOST-P2-01：启动阶段没有可导出的phase duration、critical path、retry与degraded原因指标。
- ZHUB-HOST-P2-02：窗口title/icon/taskbar progress没有绑定真实host/task lifecycle，无法展示Closing、Recovery或attention状态。
- ZHUB-HOST-P2-03：没有“重新打开上次窗口/页面”与“始终显示项目首页”的显式用户策略和隐私边界。
- ZHUB-HOST-P2-04：多窗口/辅助窗口尚无owner模型；未来新增登录、日志或下载窗口容易复制全局状态和close逻辑。
- ZHUB-HOST-P2-05：没有tray/background mode的产品决策、资源预算和可发现退出入口；实现前应保持disabled而非隐式常驻。
- ZHUB-HOST-P2-06：close dialog没有预计等待阶段、active operation摘要、不可取消原因和support detail设计。
- ZHUB-HOST-P2-07：window control没有基于真实maximized/fullscreen/platform state切换icon、tooltip和accessibility announcement。
- ZHUB-HOST-P2-08：没有session restore、cold boot、warm activation、close drain和reconnect微基准。
- ZHUB-HOST-P2-09：没有boot/close trace timeline和跨Rust-WebView correlation visualization，现场诊断只能靠散落console/eprintln。
- ZHUB-HOST-P2-10：没有用户可导出的startup/shutdown recovery bundle，且bundle尚无path/PII/secret redaction policy。
- ZHUB-HOST-P2-11：没有Windows shutdown/logoff、macOS terminate/reopen、Linux session end等平台差异文档和资格矩阵。
- ZHUB-HOST-P2-12：没有生成的host state/event/error/exit receipt协议目录和operator troubleshooting文档。

## 10. 分层重构计划

### M0 · Truth freeze与close止血

1. 冻结32-file fingerprint、现有Tauri event/capability/config和21个source-shape tests，标记哪些合同必须被替换。
2. 把window close capability从直接销毁改为`request_close`命令；Rust CloseRequested先`prevent_close`，直到明确ClosePermit。
3. 增加最小BootState和ConnectionState，native load/validation失败不得返回Ready fallback。
4. 引入host/process/window session ID和clean-shutdown marker，不改变domain operation实现。
5. 明确Hub01/04 canonical P0 owner，本报告不复制TaskRegistry、EffectLedger或Child supervisor。

### M1 · Application host与staged bootstrap

1. 新建`HubApplicationHost`和`HubBootstrapPipeline`，将instance admission、config read、repair、catalog discovery、window creation、IPC ready拆为typed phase。
2. 每阶段声明deadline、cancel、retry、degraded/recovery和rollback；输出BootReceipt与stable error/support code。
3. release GUI启动失败展示受限native recovery surface并写bounded/redacted boot log，不能依赖stderr。
4. 加入second-launch activation envelope和existing-window acknowledgement；single-instance/store细节复用Hub01 owner。
5. load路径不隐式persist；repair preview、user/automatic policy和commit receipt分离。

### M2 · Window session与connection protocol

1. `WindowSessionRegistry`拥有window ID、generation、focus/visibility、monitor/DPI/geometry和close source。
2. native event全部进入typed reducer；window controls通过host command返回receipt，platform error可见。
3. 建立`SubscribeHubStateV1`：先注册server cursor，再返回同代snapshot和next sequence；client检测gap后full resync。
4. connection支持bounded reconnect/backoff、schema/capability negotiation、stale-host rejection和disconnect acknowledgement。
5. HostCheckpointStore以unique staging、generation/CAS和debounce保存window/session state；多进程writer规则消费Hub01。

### M3 · Graceful close、worker supervision与terminal exit

1. `CloseIntent`统一窗口按钮、OS close、last-window、explicit quit、restart、fatal和forced termination来源。
2. CloseRequested后stop admission，向Hub04 TaskRegistry与Hub01 ProcessSupervisor请求同代ShutdownSnapshot。
3. 用户/策略选择Wait、Cancel、Detach-with-owner-transfer或Abort Close；每个选择有deadline和不可逆effect提示。
4. owned threads保存JoinHandle与CancellationToken；focus worker用RAII恢复pending flag，panic进入terminal diagnostic。
5. quiescent后按顺序unlisten/stop emit、checkpoint、release instance/writer lease、发布ExitReceipt，最后允许native close。

### M4 · Crash recovery与restart handoff

1. boot开始写unclean marker，只有terminal checkpoint与ExitReceipt完成后原子标clean。
2. 下次启动读取boot phase、operation/process snapshot和store generation，进入Resume、Reconcile、SafeMode或RecoveryRequired。
3. 建立boot failure streak和provider/config quarantine，safe mode禁止自动加载可疑扩展与恢复危险window state。
4. restart使用handoff token、new-instance ready ack和timeout rollback；旧实例在接管成功前保持有限服务。
5. recovery UI只展示可证明事实，不把orphan operation、unknown Child或损坏config自动标成功。

### M5 · Behavioral qualification与性能门

1. 建立Tauri mock runtime/component harness和真实窗口OS lane，source-shape tests降级为structure lane。
2. 用fake clock/process/store/event transport验证close state machine、listener cursor、worker join和restart handoff。
3. 执行kill/panic/disk full/writer wait/emit failure/event reorder/monitor removal/DPI/system shutdown fault matrix。
4. 真窗口验证running build/package/install/open editor时的Wait/Cancel/Detach/Abort Close和重启reconcile。
5. 性能比较固定硬件、项目、cache、operation和correctness parity，报告boot phase、time-to-interactive、close drain、RSS/I/O及p50/p95/p99。

## 11. 验收矩阵

| Gate | 验收内容 |
|---|---|
| HHOST-G01 | 每次启动有BootId、process instance、phase、deadline和terminal BootReceipt |
| HHOST-G02 | GUI release启动失败有可见stable error、support code和受限recovery，不依赖stderr |
| HHOST-G03 | bootstrap phase可cancel/retry/rollback，阻塞I/O不会无限无窗口等待 |
| HHOST-G04 | load、repair、catalog discovery与persist分阶段，读路径不隐式提交未知变更 |
| HHOST-G05 | second launch携typed activation并收到existing-window ack；writer policy与Hub01一致 |
| HHOST-G06 | 所有窗口/应用退出源进入同一CloseIntent和Lifecycle Coordinator |
| HHOST-G07 | native window在ClosePermit前不会销毁，duplicate/reentrant close只产生一个decision |
| HHOST-G08 | CloseRequested后新高风险command被typed rejection，不能继续入队 |
| HHOST-G09 | shutdown snapshot列出全部active/queued thread/process/effect及可用决策 |
| HHOST-G10 | Wait/Cancel/Detach/Abort Close都有deadline、terminal receipt和用户可见结果 |
| HHOST-G11 | owned worker全部可cancel并join；超时进入明确escalation而非silent drop |
| HHOST-G12 | focus worker panic或close不会永久卡住pending flag或向stale window发布 |
| HHOST-G13 | Editor/build/process attach/detach policy由Hub01 supervisor证明，Hub close不遗失owner |
| HHOST-G14 | effect在close/crash中断后由Hub04 journal恢复为terminal或ReconcileRequired |
| HHOST-G15 | window action failure有typed receipt，close/min/max Promise rejection不再被丢弃 |
| HHOST-G16 | subscribe与snapshot同代，client按sequence检测gap/duplicate/stale并自动resync |
| HHOST-G17 | listener失败执行bounded retry/backoff；schema skew进入明确compatibility状态 |
| HHOST-G18 | Booting/Connecting/Ready/Degraded/Reconnecting/RecoveryRequired/Closing不可混淆 |
| HHOST-G19 | backend load/validation失败不返回可操作Ready demo state |
| HHOST-G20 | window geometry按monitor/DPI/work area恢复并clamp，不产生不可见窗口 |
| HHOST-G21 | geometry checkpoint有debounce、generation/CAS和bounded I/O，不逐像素写全TOML |
| HHOST-G22 | clean marker只在terminal checkpoint后提交；kill/panic/OS终止下次可识别 |
| HHOST-G23 | boot loop触发safe/recovery admission，可隔离坏provider/config/window state |
| HHOST-G24 | restart handoff在新实例Ready ack后完成；timeout不会同时留下两个active owner |
| HHOST-G25 | source-shape tests不再锁定direct close，真实Tauri harness覆盖event reducer |
| HHOST-G26 | Close/Exit/last-window/system shutdown/forced exit跨平台矩阵有machine-readable结果 |
| HHOST-G27 | worker-close、emit-close、persist-close和listener-gap有deterministic race/fault tests |
| HHOST-G28 | second-instance、crash marker、safe mode、window restore和restart有端到端测试 |
| HHOST-G29 | 10k event/reconnect与长任务close soak无漏event、悬挂thread、deadlock或unbounded RSS |
| HHOST-G30 | boot/close/reconnect基准绑定source/build/workload/hardware/cache和correctness parity |
| HHOST-G31 | Unity Graphics参考适用边界被保留，不用缺失源码证明宿主能力已完成 |
| HHOST-G32 | `git diff --check`、frontmatter path、finding ID、severity、index/coverage/link和fingerprint验证通过 |

## 12. 与现有报告的依赖和非目标

| 依赖 | 本报告消费/提供 | 不重复拥有 |
|---|---|---|
| Hub01 | 消费single-instance/store CAS、ProcessSupervisor与domain shutdown capability；提供application close/activation adapter | 不重复build/package/install/editor Child backend finding |
| Hub02 | 消费React shell、fallback truth、CSP、drag/accessibility；提供Boot/Connection/Closing read state | 不重复页面、视觉、catalog/settings性能finding |
| Hub03 | 未来消费account/session close与restart policy | 不实现Auth/RBAC/Marketplace/Cloud provider |
| Hub04 | 消费TaskRegistry、OperationJournal、EffectLedger和typed command；提供stop-admission与shutdown bridge | 不重复durable queue、target、history/message和effect P0 |
| App01 | 对齐host lifecycle术语、exit receipt和ordered cleanup | Hub仍是独立产品，不复用Runtime DLL host作为窗口owner |
| Editor09 | 对齐job close decision、cancel/deadline和terminal receipt | 不把Editor job scheduler嵌入Hub |
| Tooling23/24/37 | 消费failure、thread lifetime和transaction全局门 | Hub05 product findings仍由本报告唯一计数 |

本报告不要求Hub永远单实例，也不要求关闭Hub时强杀所有Editor。正确设计可以允许Editor独立存活、后台下载转交service或多个只读窗口；关键是策略必须显式、有owner、有身份、有handoff和terminal receipt，不能由`Child`/`JoinHandle`被drop或window被销毁来偶然决定。

## 13. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| 32个Hub host selected files逐文件静态审查 | review_complete | 2026-08-19 | 8,988行、325,518 bytes、fingerprint `2c9a6aa3bd69ec86e250abbb046ea7370d080e69fe3b4a02e32fbfa765b416ca` |
| 12个reference files对照 | review_complete | 2026-08-19 | 20,983行、756,598 bytes、fingerprint `8a5f99b1689c2f8382984375e79a69fdeff54f9c308f5faf22779470706951cc` |
| Severity与owner去重 | review_complete | 2026-08-19 | 0个新增P0、40个P1、12个P2；3个继承阻断明确路由Hub01/04，不重复计数 |
| Production与tests修改 | pending | - | 本轮review-only，没有修改或运行产品代码 |
| 动态资格 | pending | - | 未运行Cargo、Tauri真窗口、第二实例、close/crash、DPI/多显示器、OS shutdown或性能测试 |
