---
related_code:
  - zircon_editor/src/core/play
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/core/gateway
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/workbench/state/editor_state_play_mode.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions/project.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_projection.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/runtime_session_args.rs
  - zircon_app/src/bin/runtime_preview.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/EditorEngine.h
  - dev/godot/editor/run/editor_run.cpp
  - dev/godot/editor/run/editor_run_bar.cpp
  - dev/godot/editor/run/embedded_process.cpp
  - dev/godot/editor/debugger/editor_debugger_node.cpp
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/settings/build.rs
  - dev/bevy/crates/bevy_remote/src/builtin_methods.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/Graphics/Packages
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 07 · Play Session、Process Runtime、PIE、Game View、Live Edit 与 Recovery 工程化差距

## 1. 结论

Zircon Editor的Play链已经超过“按钮占位”。当前产品会把内存中的authoring world转换成versioned `DynamicScene` JSON，以`create_new + write + sync_all + rename`物化到`.zircon/play/<instance>/`，再从Editor安装目录启动同级`zircon_runtime`；Windows启动前把进程挂入kill-on-close Job Object，stdout/stderr又有行长、队列条目、总字节、单tick行数/字节/时间预算。Native plugin bridge还有进入Play前的活性快照与退出补偿，pending edit queue也实现了容量、字节、年龄、分页、coalescing和决策回执。这些不是临时空壳，应作为重构底座保留。

但产品语义仍只是“把当前scene快照交给一个独立窗口子进程”。它不是Unreal式PIE：没有独立且可检视的PIE world authority、没有embedded session backend、没有Game View运行画面、没有远程hierarchy/inspector、没有live edit、没有pause/step/eject、没有多client/server拓扑。`Play`与`Simulate`只改变一个枚举和标签，process command完全不携kind。旧计划把副session、Game View和Unity式live edit写成目标；当前实现只完成了其中的process P1底座，不能把计划文本误当产品能力。

更严重的是，Play生命周期现在由`PlaySessionController::mode`与`EditorState::play_session`两套状态分阶段切换，project切换又不参与同一事务，已经形成2个P0。第一，用户可在项目A Play时直接Close Project并打开B；旧的`EditorPlaySession`仍捕获A，A子进程退出后自动`exit_play_mode()`会把A scene写进当前已加载的B world。第二，终止和补偿会在终态确认前消费唯一child ownership：stop失败不回填child；自然退出后plugin恢复失败又让controller保持Playing，而backend因active为空持续返回Running。控制面可同时失去进程句柄、真实终态和恢复进度。

本报告记录2个P0、32个P1、8个P2。没有运行Cargo、真实Editor、子进程、DLL、窗口嵌入、跨平台终止故障、远程调试、1 GiB scene或多实例性能测试；性能与竞态结论来自同步调用、所有权移动、状态发布和产品装配代码，不宣称已经完成与Unreal/Godot/Fyrox的同机性能比较。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Play core/process clean set | 33 / 4,419 | E3：controller、backend、snapshot、output、plugin activation、domain link、edit protection与process tree；fingerprint `a416ba78...da8428b` |
| Editor产品装配与状态 clean set | 18 / 4,736 | E3：command、menu enter/exit、project clear/replace、host tick/close、Game pane与startup wiring；fingerprint `cfab49de...e4f4bda3` |
| Runtime启动/报告 clean set | 5 / 2,704 | E3：Editor/Runtime entry、CLI、preview binary与Cargo target；fingerprint `abe4f8e0...f3bd47e4` |
| focused clean tests | 8 / 1,705 / 48 test attributes | E2：policy、queue、snapshot、process command、state、event stack与toolbar测试源码已读；fingerprint `3f56aa66...aa4aaff0` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`串联后再计算SHA-256。它只标识本轮clean阅读集合，不是协议或schema ID，也不能替代运行验证。

### 2.2 在途文件隔离

`zircon_editor/src/core/play/tests.rs`和`zircon_editor/src/ui/host/play_pending_decision/tests/*`成文时有其他Session或用户修改，本轮没有用这些测试证明deactivation failure、pending decision或terminal recovery正确。`zircon_app/src/entry/runtime_library/runtime_session.rs`及其相邻foreign-output/owned-buffer文件也在修改中；本报告只依据clean的Editor composition证明“启动时传入的Editor session gateway被永久挂到PlayDomainLink”，不把在途ABI实现当稳定算法证据。

工作树中`zircon_editor/src/ui/workbench/state/console_history.rs`、`editor_state_render.rs`等其他修改未回退，也未纳入指纹。实施前必须重读所有重叠owner，尤其是Runtime session frame/output ownership与pending-edit tests。

### 2.3 本轮追踪的产品链

1. `EnterPlayMode` -> 同步snapshot当前world -> `EditorState::enter_play_mode`捕获authoring scene -> `PlaySessionController::request_play`。
2. Native plugin load/activation -> edit protection begin -> snapshot落盘 -> sibling `zircon_runtime` spawn -> controller立即进入Playing。
3. Runtime解析`--play-scene/--play-report-pipe` -> 加载DLL和session -> stdout发布starting/start-failed/ready/terminal文本记录 -> 创建独立窗口event loop。
4. Editor每tick `poll_backend` -> 有界drain stdout/stderr -> 子进程退出后清snapshot、恢复plugin bridge、恢复EditorState authoring checkpoint。
5. `ExitPlayMode` -> 先停runtime consumers -> backend hard terminate/reap -> plugin bridge restore -> pending edit prompt -> `EditorState::exit_play_mode`覆盖当前world。
6. `CloseProject` -> manager close -> apply Welcome -> `clear_project`；另一路Open Project -> `replace_world`。本轮核对这两条路径与Play session是否同一事务。
7. Game document -> pane projection -> retained presenter；本轮核对它是否真的消费runtime frame以及pointer/keyboard/resize/focus是否进入Play world。
8. `route_edit/running_document/WorldDomain::Play` -> hierarchy/inspector/transaction产品caller；本轮核对已有抽象是否已经接入真实编辑工作流。

## 3. 已有工程基础，重构时必须保留

### 3.1 Process tree、输出预算与资源所有权

- Windows child先以suspended状态创建，再绑定带`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`的Job Object后恢复初始线程，避免子孙进程在attach前逃逸；Unix也会配置process group。这一设计比直接`Child::kill`可靠。
- `PlayOutputPump`同时读取stdout/stderr，限制1,024条、4 MiB queued bytes、64 KiB单行，并把每tick drain限制为64条、256 KiB和2 ms；drop/truncate计数进入diagnostic。
- 正常stop按“terminate persistent tree -> wait/reap -> join readers -> cleanup snapshot”执行。重构需要修复失败态所有权，不能退回无界channel、阻塞逐行读取或只杀root PID。

### 3.2 Scene snapshot与插件补偿

- `PlaySceneSource`区分Persisted与Snapshot；产品会把未保存的当前内存world序列化，所以Play不是只能运行磁盘旧场景。
- snapshot使用项目受控相对路径和原子文件发布；`MaterializedPlayScene`拥有临时根并在正常终态及Drop清理。
- `NativePluginBridgeActivation`在退出失败时会把active snapshot放回，允许再次恢复；这项可重试所有权思想应扩展到child、world checkpoint、transport和presentation lease。

### 3.3 Typed gateway、domain与pending queue底座

- Runtime gateway已有tick、event、frame capture、viewport bind/present、query/watch等契约，`PlayDomainLink`也有generation-safe handle与`PlayInstanceId`。
- Selection模型已区分Edit/Play domain，进入Play会复制选择并把active domain切到Play。
- Pending edit queue是有界且可分页、可coalesce、可决策恢复的基础。缺口是产品命令没有调用`route_edit`，不是queue算法本身必须重写。

### 3.4 Runtime startup report与产品轮询

- Runtime已经区分`starting/ready/start-failed/terminal`，并在多个启动失败stage发报告；Editor tick也持续轮询backend而不是只在按钮回调检查一次。
- 当前协议虽然只是stdout文本，但stage概念、logical outlet和有界transport可作为升级为typed handshake的迁移输入。
- Product确实安装`ProcessPlayBackend`，不是Noop backend；旧计划关于“尚无子进程spawn/监控”的现状描述已经过期。

## 4. P0：跨项目世界污染与终态所有权失真

### E-PLAY-P0-01 · 项目A的Play checkpoint可在A关闭后覆盖项目B的authoring world

确定复现链：

1. 项目A执行Enter Play；`EditorState::enter_play_mode`把A的整份`Scene`、selection、gizmo与session mode存入`play_session`。
2. `file.project.close`只有`WhenClause::ProjectOpen`，没有`PlayMode(Edit)`；`CloseProject`也不调用`request_stop`。
3. `close_project_from_workbench`关闭manager project并应用Welcome session；`clear_project()`清world/path/history/selection，但不清`play_session`，controller与A子进程仍Playing。
4. 用户在A子进程退出前打开项目B；`replace_world(B)`同样不清`play_session`，此时当前world是B、checkpoint仍是A。
5. A子进程自然退出后，host tick先把controller切到Edit，再调用`EditorState::exit_play_mode()`；该函数只检查“当前有已加载world”，随后把`session.scene`直接赋给当前world。
6. 结果是B的内存authoring world被A scene覆盖，并恢复A的selection与session mode。若随后保存，错误内容可持久化到B。

若没有打开B，自动退出会因“No project open”失败：controller已经Edit，`EditorState::play_session`却继续存在，命令投影与场景保护仍分裂。修复必须让Close/Open/Replace World与Play stop共享同一个project-scoped session authority；任何异步completion都必须携`project_id + project_revision + document_id + play_session_id`并做compare-and-reject，禁止把旧session checkpoint应用到新project。

### E-PLAY-P0-02 · 终止前消费唯一child，失败后可失去进程所有权并永久报告伪Playing

存在两个同源终态缺陷：

1. `ProcessPlayBackend::stop`先对`active.take()`，再调用`child.stop()?`。若persistent tree termination或reap失败，child不会放回active；controller因`BackendStop`仍保持Playing，但backend已没有PID/Child/reader/snapshot owner可供重试。Windows正常Job handle close通常会kill-on-close，因此不能声称每次都遗留进程；但Unix lease无Drop kill，Windows终止与CloseHandle双失败也会丢失所有权，跨平台契约没有恢复保证。
2. 自然退出时backend先`active.take()`并完成child，然后controller才恢复plugin bridge。若`deactivate()`失败，controller不切Edit；下一tick调用backend.poll时active为空，却返回`Running { diagnostics: [] }`。真实进程已退出，控制面可无限保持Playing，直到用户显式Exit再次尝试plugin恢复。

这不是增加一条error log能修复的问题。`PlaySessionRecord`必须在runtime终止后保留`TerminalOutcome`、所有cleanup lease和失败stage；stop在确认terminate/reap之前不得释放child owner，失败后必须能Retry/Force/Detach/Inspect。`poll`在Running session却没有active runtime时必须产生typed invariant failure，绝不能合成Running。Plugin cleanup失败应进入`RuntimeExitedAwaitingCleanup`或`CleanupFailed`，而不是篡改runtime事实。

## 5. P1：Play、PIE与产品工作流缺口

### 5.1 Authority、状态机与恢复

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLAY-P1-01 | `PlaySessionController::mode`与`EditorState::play_session/session_mode`分别切换；enter先State后Controller，exit先Controller后State，任一步失败都会产生两个truth。 | 单一`PlaySessionAuthority`持session record和authoring checkpoint；State/command/toolbar只消费同一immutable projection，状态变更一次提交。 |
| E-PLAY-P1-02 | mode只有Edit/Building/Playing，无法表达Preflight、Snapshotting、Launching、WaitingReady、Paused、Stopping、RuntimeExited、CleanupFailed与Detached。 | 明确phase与合法迁移表；runtime liveness、cleanup进度、user intent和presentation attachment分成正交字段。 |
| E-PLAY-P1-03 | transition report没有session/project/document identity、expected revision、operation ID、PID、snapshot lease、transport generation或terminal timestamp。 | 每个request/completion携稳定ID与expected revision；stale callback被拒绝并记诊断，UI可以关联一次完整启动/停止。 |
| E-PLAY-P1-04 | backend启动失败后若plugin deactivate也失败，controller仍返回Edit；bridge可能实际保持runtime play activation，却没有repair state。 | rollback失败进入durable `StartupRollbackFailed`，保留activation lease、诊断与Retry/Force Reset，不得投影为干净Edit。 |
| E-PLAY-P1-05 | backend poll/cleanup错误每tick只覆盖status line，没有有界重试、退避、incident history、notification或修复命令。 | lifecycle coordinator记录first/last error、retry count、deadline和next action；重复错误去重，终态必须可检查和可恢复。 |
| E-PLAY-P1-06 | 主窗口close只处理dirty documents；`RetainedEditorHost::Drop`只停autosave/hierarchy watch，未执行有序Play stop。backend Drop又静默忽略stop错误。 | Window/Project/App shutdown统一走session close protocol：dirty decision、graceful runtime exit、force escalation、plugin restore、snapshot cleanup、最终closeout report。 |
| E-PLAY-P1-07 | Editor启动时把自身persistent runtime session gateway永久attach到PlayDomainLink；spawn的process backend从不替换/连接它，也没有产品detach。 | gateway必须由具体Play instance/backend在handshake后提供，instance停止必detach；Editor authoring gateway不得冒充child Play world。 |
| E-PLAY-P1-08 | command enablement只看controller mode，scene mutation保护只看EditorState checkpoint；ModeMessage也只有from/to枚举。 | command、save/project close、inspector、hierarchy和status订阅同一session snapshot；事件携cause、phase、IDs、capability与recovery action。 |

### 5.2 启动、进程协议、构建与snapshot

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLAY-P1-09 | Enter Play回调同步执行whole-world pretty JSON、plugin discovery/load、目录/文件sync和process spawn，全部占用UI事件链。 | 启动作为Editor job分阶段后台执行；主线程只做有预算的world checkpoint，支持progress、cancel、timeout和stale revision abort。 |
| E-PLAY-P1-10 | `spawn()`成功即返回Playing；Runtime DLL、project、scene、window或首帧失败只能稍后显示成Crash。 | backend先进入WaitingReady；只有通过版本/项目/scene握手且首个可呈现frame ready后才发布Running，startup failure保持原stage。 |
| E-PLAY-P1-11 | `--play-report-pipe`实际只是stdout记录中的logical name；Editor完全不解析`zircon_play_report`，所有内容变成`process.stdout:`字符串。 | 使用framed typed local transport或严格JSON-lines parser，校验schema/session/outlet/sequence，区分control、log、telemetry与user stdout。 |
| E-PLAY-P1-12 | Runtime在`RuntimeSession::create`后、`event_loop.run_app`前发Ready，尚未证明window/swapchain/scene首帧已经present。 | 定义`SessionCreated/WindowReady/SceneReady/FirstFramePresented`里程碑；产品Running gate至少绑定目标presentation的首帧。 |
| E-PLAY-P1-13 | Stop只有process-tree强杀，没有cooperative shutdown request、游戏保存/网络退出、ack deadline和逐级升级。 | `RequestExit -> Ack -> Drain -> Reap`为正常路径，deadline后才`TerminateTree`；保留每级结果并保证reader/transport/snapshot回收。 |
| E-PLAY-P1-14 | `PlayStartRequest::after_build`与`on_build_finished`无产品caller；独立script build orchestrator的`play_after_build`没有连接controller，Building在产品不可达。 | Build coordinator与Play authority以typed ticket交接，支持cancel/rebuild/retry和artifact/build-set identity，编译成功才进入launch。 |
| E-PLAY-P1-15 | snapshot文件只sync自身，不sync父目录；异常退出残留目录没有startup scavenger、age/owner manifest、quota或恢复记录，cleanup失败只进一次diagnostic。 | content-addressed或session-manifest管理的snapshot store，含directory durability、owner PID/session、TTL/quota、startup reconciliation与可见cleanup incident。 |
| E-PLAY-P1-16 | 启动没有统一preflight确认scene/resource generation、pending import、script/plugin build、runtime executable/build-set/ABI和目标profile兼容。 | `PlayPreflightReport`收敛所有dependency barrier，fatal项阻止launch，warning需明确policy；结果与启动artifact revision一起冻结。 |

### 5.3 PIE world、Game View、inspection与live edit

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLAY-P1-17 | 产品只安装`ProcessPlayBackend`，没有旧计划所述`EmbeddedSessionPlayBackend`；已有SessionGateway/capture API未组成第二runtime world。 | 首选PIE路径创建独立Runtime profile副session/世界；process路径保留用于真实standalone验证，两者实现同一session contract。 |
| E-PLAY-P1-18 | `EditorPlaySession`只保存authoring scene用于退出回滚；Editor UI期间仍持原authoring world，运行子进程world完全不可见。 | authoring world与每个Play world是显式不同identity/context；退出销毁Play world，不靠把旧整scene覆盖回“当前world”恢复隔离。 |
| E-PLAY-P1-19 | Game document虽位于中心区并显示viewport尺寸，pane projection明确给`is_viewport=false`；产品没有runtime frame capture/bind/present caller。 | Game View拥有稳定surface/viewport ID、frame generation、aspect/resolution policy与placeholder/error states，真实消费当前attached Play instance输出。 |
| E-PLAY-P1-20 | Game View没有向runtime路由pointer、keyboard、IME、focus、pointer lock、resize、DPI、visibility、audio focus或camera possession。 | 建立Game presentation/input bridge；焦点与编辑器shortcut有明确优先级，resize/floating/多窗切换不重建session或错投事件。 |
| E-PLAY-P1-21 | process path不可attach，hierarchy/inspector只看Edit world；没有remote object tree、property query/watch、logs/debugger/profiler与runtime selection关联。 | 统一typed debug transport，先交付read-only hierarchy/inspector/watch，再接debugger/profiler；每条数据携instance/world/entity generation。 |
| E-PLAY-P1-22 | `PlaySessionController::route_edit`没有production caller；真实`EditorState::apply_intent/execute_scene_commands`在Play时硬拒绝所有scene mutation。 | 所有编辑入口先经domain-aware operation router；Play domain写入runtime副本，Edit domain按document lock/queue policy处理，不能旁路。 |
| E-PLAY-P1-23 | 产品构造`PlayStartRequest`从不`with_running_document`，所以running document lock为空；pending queue/decision UI虽复杂，却收不到真实操作。 | 启动冻结scene document ID/revision；command registry、asset/toolkit操作都传typed target，queue只接可重放且revision-safe的Edit-domain intent。 |
| E-PLAY-P1-24 | 没有per-instance volatile undo/redo、runtime spawn/despawn投影、property diff或Keep Simulation Changes；退出只能整体丢弃runtime变化。 | Play历史按instance隔离并随session销毁；提供显式、可预览、可冲突检测的selected-object/property apply-back事务，默认仍零污染。 |

### 5.4 Play/Simulate语义、设置、规模与诊断

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-PLAY-P1-25 | `PlayKind`存入controller mode，但process command不读取kind；Play与Simulate行为完全相同。 | Play创建player/input possession，Simulate保持editor/spectator control并可在PIE/SIE间切换；backend handshake显式接收kind。 |
| E-PLAY-P1-26 | 无Pause/Resume、single-frame Step、Eject/Possess、time scale、camera sync、audio mute或game focus控制。 | session capability驱动toolbar/commands；每项有typed request、ack、可用原因与多实例目标，不用本地布尔值假切换。 |
| E-PLAY-P1-27 | 只有“当前内存world snapshot”一种产品启动源；Persisted source无caller，也没有Run Main/Current/Custom/From Here/Selected viewport。 | 统一`PlayLaunchTarget`表达main/current/custom/from-here/standalone/test map，冻结document/source revision与spawn transform。 |
| E-PLAY-P1-28 | 没有Play settings：window mode/size/monitor/DPI、graphics preset、args/env、debug flags、audio、fixed timestep、movie/capture与last-used profile。 | 项目默认 + 用户override + 临时launch override的typed settings schema，先validate再冻结到session record并可复现实验。 |
| E-PLAY-P1-29 | controller与backend都只容纳一个child；没有client/server/dedicated、多本地实例、端口/账号分配、instance选择或late join。 | session group管理N个typed instances和拓扑；每个有独立PID/world/log/view，stop group与stop instance语义明确。 |
| E-PLAY-P1-30 | 只有`try_wait`，没有startup/runtime heartbeat、hang detection、responsive state、automatic crash restart、debugger-break例外或用户延长deadline。 | watchdog区分Starting slow、Paused at breakpoint、Hung和Exited；policy化提示Wait/Attach Debugger/Restart/Terminate并保留证据。 |
| E-PLAY-P1-31 | stderr和预算告警靠字符串前缀定severity；没有phase timeline、crash dump/minidump、last frame、launch manifest、child resource stats或可导出incident bundle。 | `PlayDiagnosticEvent`具category/severity/code/instance/phase/time；Crash workspace关联logs、report、dump、snapshot manifest、GPU/runtime evidence和恢复动作。 |
| E-PLAY-P1-32 | 没有Play专项benchmark或budget gate，无法回答大scene快照、插件加载、进程ready、Game View frame、remote tree或多实例的延迟/内存目标。 | 建立规模矩阵与p50/p95/p99：10k/100k实体、1 GiB资源集、日志洪峰、慢DLL、4客户端；CI记录趋势并按产品预算阻断回退。 |

## 6. P2：测试、协议演进与维护债

| ID | 当前差距 | 必须补齐 |
|---|---|---|
| E-PLAY-P2-01 | 没有“项目A Play -> Close -> Open B -> A退出”的回归测试；现有state tests只覆盖同一world的进入/退出。 | deterministic host test断言旧completion不能触碰新project，并覆盖无project、同project新revision和close失败。 |
| E-PLAY-P2-02 | 没有terminate/reap/plugin deactivate/snapshot cleanup各阶段fault injection，尤其没有证明失败后child owner仍可重试。 | 可脚本化fake process/tree/plugin leases逐阶段失败；断言状态、owner、retry和最终closeout，不只匹配error string。 |
| E-PLAY-P2-03 | Runtime report producer有source guard，Editor却没有parser/sequence/schema contract，也没有乱码、截断、伪造user stdout与乱序测试。 | producer/consumer共享versioned DTO与golden corpus，加入property/fuzz测试和未知字段/版本降级策略。 |
| E-PLAY-P2-04 | 无真实binary E2E验证spawn -> ready -> first frame -> input -> graceful stop -> zero residual；Game View也无截图/像素验收。 | Windows/Linux产品E2E含真实runtime、窗口或offscreen surface、frame evidence、输入回显、进程树与临时目录清零。 |
| E-PLAY-P2-05 | 进程树测试主要聚焦Windows source/fixture，没有Linux/macOS group、grandchild逃逸、handle close failure与reader阻塞矩阵。 | OS CI故障注入覆盖root/grandchild、继承pipe、权限拒绝、kill race、PID reuse和Editor崩溃后的系统回收。 |
| E-PLAY-P2-06 | snapshot实例名依赖PID+wall-clock nanos+sequence；没有碰撞、时钟回拨、symlink/reparse、quota、巨型JSON与磁盘满的系统性测试。 | 使用不可猜测稳定session ID和secure path policy；加入filesystem adversarial/property测试与bounded serialization。 |
| E-PLAY-P2-07 | Play输出、phase、world sync和Game View没有统一trace correlation，无法做启动critical path或卡顿归因。 | session/instance/operation trace ID贯穿Editor job、runtime process、render frame和cleanup；导出可比较timeline。 |
| E-PLAY-P2-08 | `docs/plans/zircon_editor/editor/04-pie-and-simulation.md`的现状段落仍称无状态机/无process backend，目标与已完成项混在一起。 | 本报告成为current-state owner；实施时更新旧计划为milestone/history链接，并以source fingerprint/currentness检查阻止陈旧事实继续指导代码。 |

## 7. 与参考引擎的差异及适用边界

| 参考 | 本轮直接证据 | Zircon应吸收 | 不应机械照搬 |
|---|---|---|---|
| Unreal | `RequestPlaySession`把请求留到next tick并复制/校验Play settings；`CreatePIEWorldByDuplication`创建带PIE package/instance ID的独立world；`EndPlayMap/TeardownPlaySession`遍历PIE world contexts；PIE/SIE可切换并支持viewport、多client/server与in/out-of-process。 | project/session/world identity、独立world context、阶段化启动、SIE语义、完整teardown和多实例拓扑。 | 不必复制UObject/package全套；Zircon可用副Runtime session + versioned DTO，但隔离与身份不变量必须同级。 |
| Godot | `EditorRun`维护多个PID、remote-debug URI、breakpoints、window placement与instance args；Run Bar统一stop；`EmbeddedProcess`有embed retry/timeout、focus/resize/visibility和析构kill；debugger维护remote scene tree。 | 外部进程仍需remote inspection、multi-PID authority、embedded presentation生命周期和丰富launch settings。 | Godot的直接PID kill和简单status不是终态事务上限；Zircon已有process-tree/output基础，应保留更强所有权。 |
| Fyrox | `Mode::Build/Play`持真实Child，build profile有command queue与run command，主loop poll child，退出时kill active child。 | Building是一等状态、profile驱动build/run和loop-owned cleanup。 | Fyrox该版本的reader/kill/status较简，不能作为Zircon输出预算、跨平台树终止或PIE world隔离的最终标准。 |
| Bevy | Remote Protocol提供world query/get/insert/mutate/remove、watch、registry schema和schedule graph；App支持SubApp与typed AppExit。 | remote world control面必须typed、可发现、可watch、可扩展；副app/world可作为embedded backend设计参照。 | Bevy源码不是完整Editor/PIE产品，不能据此推导Game View、dirty gate或Keep Simulation Changes UX。 |
| Unity Graphics checkout | 仓内主要是SRP/HDRP/URP/ShaderGraph/VFX与测试工程，不包含Unity Editor Play Mode authority源码。 | 后续用它验证Game View画质、frame settings、capture和render parity。 | 本轮不能把Unity公开产品行为当本地源码证据，也不能用Graphics package替代Editor lifecycle设计。 |

目标不是复制某一个引擎。Zircon应组合Unreal的world/session rigor、Godot的外进程调试/嵌入、Bevy的typed remote surface和现有Zircon的Runtime DLL/session gateway、process tree与有界输出基础。

## 8. 目标架构与唯一权威

### 8.1 `PlaySessionAuthority`

```text
Idle
  -> Preflighting
  -> Snapshotting
  -> Launching
  -> WaitingReady
  -> Running <-> Paused
  -> Stopping
  -> RuntimeExitedAwaitingCleanup
  -> Closed

任一阶段 -> StartupFailed / CleanupFailed
```

authority持完整`PlaySessionRecord`：`session_id`、project/document identity与revision、launch kind/target/settings、backend kind、instance group、child/session owner、plugin activation lease、snapshot lease、runtime transport、world/presentation attachment、phase timeline、terminal outcome与repair actions。Controller、EditorState、command registry和UI不得再各自保存可分裂的Playing真相。

### 8.2 Backend与lease协议

- `ProcessBackend`返回`ProcessInstanceLease`，提供`request_exit/poll_ack/force_terminate/reap/drain`；任何错误都不消费唯一owner。
- `EmbeddedSessionBackend`通过Runtime owner创建真正的secondary session/world，返回可attach gateway与viewport surface lease；销毁前先detach consumers和presentation。
- Plugin、snapshot、world checkpoint、transport、Game View surface都实现显式close，返回可聚合的`CloseoutReport`。Drop只做最后兜底，不承担用户可见成功语义。
- backend不再返回`attachable: bool`；它发布typed capabilities和实际attachment ticket。`active=None`只在Closed合法。

### 8.3 Typed startup/control transport

Editor与Runtime共享versioned envelope：`Hello/Accepted/Rejected/Phase/Ready/Heartbeat/Log/WorldEvent/FrameReady/ExitRequested/ExitAck/Terminal`。Envelope携session/instance ID、monotonic sequence、protocol/build-set/schema、project identity和capabilities。Process stdout只承载用户日志或作为受限fallback，不能再同时充当不可解析的控制面。

`Ready`定义为目标scene完成、必要plugin/system启动且指定Game presentation已经提交首个有效frame；如果选择headless/dedicated profile，则用对应的simulation-ready barrier，不伪造frame条件。

### 8.4 World、Game View与live edit

- Edit world始终归authoring document；每个Play instance拥有独立world ID。Project替换必须先关闭或明确detach对应session，旧completion不能通过ID检查。
- Game View绑定当前attached instance的surface/frame stream；input/focus/audio/camera与Editor shortcuts由同一presentation controller仲裁。
- hierarchy/inspector先交付read-only typed watch/query；live edit随后使用同一transport发送versioned mutation DTO，写入`HistoryContextId::PlaySession(id)`。
- Apply Back只提交用户显式选择的property/entity diff，带Edit world expected revision和冲突预览；禁止整world覆盖。

## 9. 硬切重构范围

1. `core/play`收敛为session authority、phase machine、instance/backend leases、typed protocol与closeout report；删除`poll(active=None) => Running`和无身份transition。
2. `EditorState::play_session`不再拥有可无条件覆盖当前world的整scene；authoring checkpoint必须在authority内绑定project/document revision，或由独立world隔离后彻底删除整world回写。
3. Project Open/Close/Replace Scene、window close与app shutdown必须经过同一session coordinator；command when-clause只读authority projection。
4. Process report从字符串logical pipe硬切为共享DTO/transport；保留有界output pump作为日志面，不留双协议长期兼容层。
5. 产品加入secondary Runtime session backend；startup时的Editor gateway不得永久注册成Play gateway，attach/detach由instance lifetime拥有。
6. Game pane改成真实presentation owner，接frame、resize和input；Scene viewport与Game View不共享一张无身份全局image。
7. 所有scene/property/toolkit操作接domain router；删除Play期间分散的`is_playing()`硬编码旁路，pending policy只保留一份。
8. Script build/export artifact authority与Play preflight用ticket连接；移除不可达的`after_build/on_build_finished`孤立路径或把它们纳入唯一编排。
9. `PlayKind::Simulate`在backend/runtime有真实协议语义后再保留；在此之前UI不得把同一行为包装成已实现Simulate。

## 10. 测试先行的依赖序里程碑

### M0 · P0封口与失败态所有权

- 先写A Play -> Close -> B Open -> A Exit测试，并让Close Project在Play session有明确Stop/Cancel/Keep Running policy。
- child、plugin、snapshot各阶段fault injection；stop失败后PID/lease仍可Retry，terminal runtime事实不被cleanup错误覆盖。
- 为所有异步completion加入project/document/session expected identity。

### M1 · 单一session authority与close protocol

- 落地完整phase machine、session record、typed transitions和immutable UI projection。
- 合并Controller与EditorState双truth，统一project/window/app shutdown。
- 加入durable incident/repair state与closeout report。

### M2 · Process handshake、build/preflight与snapshot store

- 共享typed protocol、startup phase/heartbeat/graceful exit和first-ready gate。
- 连接script build coordinator，冻结build-set/artifact/scene/resource revisions。
- snapshot manifest、quota、scavenger、durability和后台序列化预算。

### M3 · Embedded PIE与真实Game View

- 创建secondary Runtime session/world并通过同构payload注入。
- Game View bind/present/capture、resize/aspect、focus/input/audio/camera闭环。
- Editor/Play gateway按instance attach/detach，退出证明Edit world零污染。

### M4 · Runtime inspection与debug transport

- hierarchy/inspector/query/watch、runtime selection、spawn/despawn和diagnostic timeline。
- process与embedded backend复用同一world/debug DTO，先只读、带能力协商和流量预算。
- debugger/profiler/log/crash bundle按instance关联。

### M5 · SIE、live edit与Apply Back

- Play/Simulate真实分流，支持PIE/SIE toggle、pause/step/eject/possess。
- per-instance volatile history、domain-aware commands和可重放pending edit。
- selected property/entity diff apply-back，冲突检测与authoring transaction原子提交。

### M6 · 多实例、产品设置与性能门

- client/server/dedicated topology、instance picker、端口/账号/窗口分配和group lifecycle。
- typed Play settings与main/current/custom/from-here launch targets。
- 大scene、日志洪峰、慢启动、Game View frame、remote tree和4-client性能/稳定性CI。

## 11. 产品级验收门

1. A项目Play期间Close/Open B，A的任何late event都不能修改B world、selection、history、status owner或磁盘。
2. terminate、reap、plugin restore、snapshot cleanup任一失败后，session保留全部必要owner并提供确定Retry/Force/Inspect。
3. runtime已退出但plugin cleanup失败时，UI显示`Runtime exited / cleanup failed`，绝不显示Playing或Running。
4. Enter Play在Runtime Hello、版本/项目/scene校验与首个目标ready barrier前不进入Running。
5. malformed、截断、乱序、重复、旧session和user伪造的report不能驱动lifecycle。
6. 正常Stop先cooperative exit，超时才terminate tree；root/grandchild、reader thread、transport和snapshot全部收敛。
7. Editor window close、Project Close、engine crash recovery三条路径都有可检查closeout和零孤儿进程证明。
8. Embedded PIE退出前后Edit world content hash、document revision和undo history不变。
9. Game View显示真实首帧并持续更新；resize/DPI/floating/focus不会空白、拉伸错位或把输入交给Scene viewport。
10. Play与Simulate在player possession、editor selection/gizmo、camera和input语义上可观察地不同，且可切换。
11. runtime spawn/despawn在预算内进入remote hierarchy；选中对象Inspector值按generation更新，不读取旧instance。
12. Play-domain edit/undo/redo只改变副world；Exit默认丢弃，Apply Back只提交预览确认的diff。
13. Running document lock与pending queue有真实产品caller；stale revision intent不能在退出后误作用新文档。
14. build-required Play等待正确artifact；build失败/cancel/rebuild不会启动旧binary或旧scene。
15. main/current/custom/from-here与process/embedded profile都可从session manifest完全复现。
16. 多instance logs、windows、world IDs、input target和stop actions不串线；单instance失败不篡改其他实例状态。
17. 10k/100k实体snapshot与remote tree有明确p95预算；1 GiB资源项目不在UI线程做无界序列化/加载。
18. stdout/stderr洪峰保持内存和tick预算，control transport不因日志drop而丢lifecycle事件。
19. hang、debug breakpoint、GPU device loss、runtime crash和Editor transport断开有不同typed状态与恢复动作。
20. Windows/Linux/macOS实机或CI矩阵验证process tree、路径、window/presentation和cleanup；所有benchmark保留可比较artifact。

## 12. 依赖、owner与后续复核

- `zircon_app`负责Runtime process handshake、ready/terminal定义、editor/runtime build-set协商和graceful exit host；现有stdout reporter需要与Editor共同硬切。
- `zircon_runtime_interface`负责secondary session、frame/presentation、remote world/debug DTO与foreign output ownership；其在途ABI文件稳定后必须重读。
- `zircon_runtime` scene/resource/plugin/system owner负责Play preflight barrier、独立world reset、runtime mutation与terminal drain。
- Editor document/transaction owner负责project/document revision、volatile Play history、pending replay和Apply Back复合事务。
- Retained UI owner负责真实Game View surface/input/focus、phase/status/repair投影和多instance选择，不自行保存lifecycle bool。
- Build/tooling owner负责artifact identity、Play benchmark、fault injection binary与跨平台process-tree CI。
- 旧`04-pie-and-simulation.md`可保留历史设计与会签，但当前事实、P0和重构优先级由本报告拥有。实施完成后再按实际代码更新两者，不能提前把M3-M6标成产品能力。
