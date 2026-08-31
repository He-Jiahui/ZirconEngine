---
related_code:
  - zircon_runtime/src/platform
  - zircon_runtime/src/input
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/core/framework/window
  - zircon_app/src/entry/runtime_entry_app
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_editor/src/core/process.rs
  - zircon_editor/src/core/play/process_backend
  - zircon_editor/src/ui/host/export_cargo_process.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-22-play-process-output-byte-budget.md
  - docs/plans/zircon_editor/editor/15/failure-2026-07-22-export-output-tail-durability-backpressure.md
  - docs/plans/performance/01/2026-08-14-runtime-input-ingress-current-review.md
  - docs/plans/performance/01/2026-08-15-input-action-evaluation-generation-and-workspace.md
reference_engines:
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/Fyrox/fyrox-impl/src/engine/mod.rs
  - dev/godot/core/input/input_event.h
  - dev/godot/core/input/input.cpp
  - dev/godot/core/os/main_loop.h
  - dev/godot/core/os/os.h
  - dev/godot/servers/display/display_server.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericApplicationMessageHandler.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericPlatformInputEvent.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/AsyncInputConsumer.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/GenericPlatform/AsyncInputConsumer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformProcess.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/MonitoredProcess.h
---

# 06 · Platform、Input 与 Process Host 工程化差距

## 1. 结论

Zircon 当前已经有若干值得保留的工程化基础。`PlatformCapabilityMatrix` 对 desktop/mobile/web/headless 和 feature gate 建立了显式状态词汇；preference backend 采用 install-once driver、bounded keyed I/O 和 250 ms cleanup budget；runtime entry 的 reactive cadence 会合并 frame request；gamepad drain 已有 256 events/2 ms 预算，rumble 有 per-gamepad effect 上限；input frame buffer会合并相邻 cursor position 和 raw motion；event recorder 默认关闭且有 record-count capacity/discard diagnostics；action evaluator 已在 map generation 上编译 binding index，并复用 workspace；Play process 在 Windows 通过 suspended spawn 后附着 Job Object，stdout/stderr 也有 line、entry、byte 与 per-poll drain 预算。旧计划里“action 每帧全扫描”“录制完全无容量限制”“Play output 只有 entry cap”的描述已经不完整，本文不重复登记为 current-source 问题。

但这些局部能力尚未形成一个统一、可验证的 platform host。`PlatformModule` 对外宣称负责 windowing/OS integration，实际 driver/manager 只拥有 preference storage；真实 window、surface、event loop、input producer 和 process tree 分散在 `zircon_app` 与 `zircon_editor`。capability report 根据 target 和编译 feature 推导 `Supported(Winit)`，没有绑定已创建的 event loop、window、monitor、input device、surface generation 或运行时探测结果。产品 host 仍只有一个 `window`、固定 `viewport=1`，进入 `ApplicationHandler` 后直接丢弃 `WindowId/DeviceId`；窗口失焦被翻译成整个 runtime 的 Background，而 `suspended`、surface destroy、memory warning、thermal/power、device loss 等应用生命周期没有入口。`InputDriver` 是空类型，所有输入继续逐事件同步穿越 ABI、session registry 和多层 mutex，coalescing 发生在这些成本之后；action map 和 replay helper 没有产品消费路径，脚本仍按每次 host call resolve manager、clone raw snapshot 并线性查键。进程侧只有 Play 获得 race-free Job lease，export/compile 路径仍共享不一致的 spawn、tree、output 和 durability 语义，其中 export Cargo 仍把完整 stdout/stderr 累积为 `Vec<u8>` 后再复制成 `String`。

本轮登记 12 项 P1 和 2 项 P2，没有新增 P0。当前证据能证明大型输入风暴、移动端 suspend/resume、多窗口和 1 GiB 子进程输出会缺少完整预算或身份合同，但没有证据表明这些缺口已经造成已发布项目的数据损坏或安全越权，因此暂按 P1。若 Play/export 终止已在产品中遗留持有项目文件的子孙进程，或 mobile surface resume 已造成不可恢复的存档/设备状态损坏，应把对应项上调为 P0 并冻结相关扩展。

## 2. 审查边界与物理覆盖

### 2.1 已读范围

- `zircon_runtime/src/platform` 52 个 Rust 文件、约 7,343 行、92 个 `#[test]`：module/driver/manager、target/features、capability matrix/report、preference persistence、cross-target/status/diagnostic tests。
- `zircon_runtime/src/input` 31 个 Rust 文件、约 4,005 行、53 个 `#[test]`，以及 `core/framework/input` 26/约 1,302、`core/framework/window` 14/约 765：event/state/snapshot/action map/evaluator/recording/replay/host requests/device DTO。
- `zircon_app/src/entry/runtime_entry_app` 79 个 Rust 文件、约 5,681 行、97 个 `#[test]`：winit handler、window/surface、cadence、pointer/keyboard/IME/gamepad producer、host request drain 与 dynamic runtime event handoff。
- 读取 `ZrRuntimeEventV1` 的 V1 event shape、dynamic session input reducer、script gameplay input host，以及 EngineEntry 的 platform config/preference backend安装路径；沿产品调用点确认 action manager、recording helper 和 `InputDriver` 的缺失消费者。
- 读取 editor process owner：`core/process.rs` 604 行、Play process backend 5/约 1,133、export process support 4/约 513，以及 compile-host/export Cargo/wizard调用点；交叉核对 Runtime11、Editor14、Editor15 和 Performance01 的 open failures/current-source记录。
- 对照 Bevy winit window entity mapping、application lifecycle 和 frame accumulators；Fyrox resume/suspend graphics context；Godot window/device-aware input、application notifications 和 OS process surface；Unreal window/device/user/timestamp input payload、async consumer和 platform process abstraction。

### 2.2 明确未覆盖

- 本篇不评价 RHI surface implementation、swapchain present、graphics device loss 的内部正确性；这里只拥有 OS/application lifecycle 到 graphics owner 的通知、generation 和 teardown合同。GPU 细节进入 `09/10`。
- Editor retained-host 多窗口、dock/floating window、快捷键仲裁和完整 UI input route 仍归 editor 专篇；本文只登记 runtime primary host 与 shared platform/input/process基础合同。
- 没有运行真实窗口、物理 gamepad、IME、Android/iOS suspend、1 GiB output storm、WPR/ETW、Tracy 或 Cargo。相关源码在本轮存在其他 Session 的活跃修改，因此所有 finding 标记 `recheck_required`，实现前必须重新取 current-source fingerprint。
- Unity Graphics 主要提供 render/surface/resource lifetime 参考，将在 graphics 专篇逐项使用；本篇不为了满足引擎列表而把 graphics-only API硬套到 OS input/process 结论。

## 3. 当前闭环与应保留能力

### 3.1 Platform vocabulary 与 preference persistence

`PlatformTarget`、`RuntimeTargetMode`、`PlatformFeatureSelection`、`CapabilityStatus` 和 backend token 已把“feature disabled”“target unavailable”“supported backend”区分开；cross-product tests 对 desktop/mobile/browser/headless、X11/Wayland、gamepad 和 diagnostics key 顺序有较密覆盖。这个纯值矩阵适合作为 build manifest 与期望能力，不应删除。

`PlatformDriver` 对 preference backend 使用 install-once gate，不能用 `Unavailable` 覆盖已安装 backend；`PreferencePersistenceAdapter` 复用 bounded keyed I/O，Platform module cleanup 有显式 deadline 和 incomplete/failed/cancelled report。后续 owner 收敛应把这套“host install + runtime service + bounded shutdown”模式推广到 window/input/process，而不是让真实 host 反向依赖静态矩阵。

### 3.2 Input reduction、action generation 与局部风暴止损

`DefaultInputManager` 已维持 pressed/just-pressed/just-released、mouse/touch/gamepad/IME/window status 的 frame semantics；focus loss 能释放 active buttons；gamepad disconnect 会清轴、模拟 transition 并释放对应按钮。`FrameEventBuffer` 对相邻 absolute cursor 取 latest、对相邻 relative motion 求和，保留中间 edge 作为 barrier。recording 默认关闭，启用时 capacity=8192 并报告 discarded records；这比永久保存全事件更合理。

`InputActionEvaluator` 当前不是旧版全表扫描。`set_action_map` 已构建 immutable generation/binding ranges，frame axis、active context 和 consumed-input 使用可复用 workspace；10/100/1k/10k source-scale fixtures已存在。后续问题是产品接线、source identity、输入批处理和动态验收，不应回退当前 generation/workspace。

App gamepad poll 已限制 256 events/2 ms，rumble 维护 per-gamepad 32 effect上限；frame cadence 能在 Reactive/LowPower 下合并 wake，并接受 runtime `Idle/Immediate/After` demand。输入 edge、state reduction、action evaluation和event-loop wake应继续分层，不能为追求吞吐把所有事件粗暴 latest-value 化。

### 3.3 Process tree 与 output capture 的已有正确方向

Play `ProcessTreeLease` 在 Windows 以 suspended child消除“spawn 后、attach Job 前”的逃逸窗口，Job Object设置 kill-on-close；Unix使用独立 process group。stop/terminal poll在释放 active mutex 后 terminate、wait、join output reader并清理 scene snapshot。Play output已有 64 KiB max line、1024 entries、4 MiB queue byte cap和 count/byte/time drain，不能再按旧 failure写成单行/queue完全无界。

compile host会把完整 stdout/stderr流式写文件、计算 digest，并只在结果里保留64 KiB tail；wizard也有16 KiB line和512 line tail。目标是把这些已验证的局部机制收敛为共享 process/stream ticket，而不是重写为单个 `Command::output()` helper。

## 4. 差距清单

### P1-1：`PlatformModule` 的公开职责与真实 owner 不一致，平台主链绕开模块生命周期

**证据**

- `zircon_runtime/src/platform/module.rs:58,99` 把模块描述为 `Platform, windowing, and OS integration`，但唯一 manager注册是 `RegisteredManagerService<dyn PreferenceStorage>`；`PlatformDriver` 在 `service_types/driver.rs:15-17` 只有 preference adapter和install mutex。
- 实际 event loop、window/surface、monitor选择、cursor/IME请求和physical input producer都位于 `zircon_app/src/entry/runtime_entry_app`；process tree又位于 `zircon_editor/src/core/process.rs`。这些 owner不会随 PlatformModule activate/deactivate 一起创建、暂停、恢复或销毁。
- `InputDriver` 在 `zircon_runtime/src/input/runtime/input_driver.rs:2` 是零字段空类型，module注册它并不代表存在平台 input driver。

**风险与目标合同**

模块 registry、capability report和产品真实资源可能给出互相矛盾的生命周期：PlatformModule可以ready但没有event loop/window/input backend；产品host可以拥有window和reader thread而Runtime认为platform已deactivate。目标应先定义 `PlatformHost` 进程级权威，再通过 typed service把 `WindowRegistry`、`ApplicationLifecycleSource`、`InputDeviceRegistry/InputIngress`、`ProcessSupervisor` 和 `PreferenceStorage` 安装到runtime。每个service有generation、thread affinity、ready/unavailable reason与bounded shutdown；`PlatformModule`只在这些依赖满足后ready。若保持app/editor为资源owner，模块描述和service surface必须诚实反映host-installed bridge，不能继续由空driver代替。

### P1-2：capability report 是编译期/目标期推导，不是运行时已协商能力

`PlatformConfig::default` 在 `platform/config.rs:59-62` 默认 `enabled=false` 且从 `cfg!(feature=...)`构建feature selection；`capability_report`只执行纯值matrix。`matrix/window.rs:37` 只要 desktop target + platform-window + platform-winit就返回 `Supported(Winit)`，随后 monitor/window event/lifecycle也级联返回Supported。它没有检查event loop是否已建立、primary window是否创建成功、当前display server/protocol、monitor权限、IME/drag-drop/cursor API实测、surface format/device、gamepad初始化或host backend generation。

目标分离三层：`CompiledCapabilityManifest` 描述二进制可能做什么；`RequestedPlatformProfile` 描述项目/entry想启用什么；`ObservedPlatformCapabilities` 来自已安装host/backend和真实设备探测，携带generation、limits、degraded reason和last failure。产品feature决策只消费 observed report；编译矩阵只用于startup preflight。window/device hotplug、surface recreate、permission变化会发布capability generation变化，不能把一次启动静态值永久当真。

### P1-3：window、viewport、physical device、platform user 与 event time 身份在host/ABI边界丢失

`RuntimeEntryApp` 在 `mod.rs:43,61-63` 只有一个 `window`、一个viewport和一份last pointer；`construct.rs:38`把viewport固定为handle 1。winit callback在 `application_handler/hooks.rs:36,55` 明确把参数命名为 `_window_id` 和 `_device_id` 后丢弃。`ZrRuntimeEventV1` 虽携带viewport和pointer_id，但没有host window generation、physical device、seat/platform user、source timestamp或producer sequence；进入 `InputEvent` 后除gamepad/touch外，keyboard/mouse/cursor/IME/window status都没有source identity。`InputFrameSnapshot`进一步聚合成一个cursor和一组全局buttons。

这阻断 runtime多窗口、编辑器多viewport正确路由、多键盘/鼠标、多seat/local multiplayer、笔/触摸设备校准、窗口销毁后的stale event拒绝和producer latency测量。目标建立稳定但不持久化OS raw handle的 `WindowHandle(index,generation)`、`ViewportHandle(index,generation)`、`InputDeviceId`、`PlatformUserId/SeatId`、`PointerId` 和 monotonic `InputSampleTime/Sequence`。host维护Winit/Win32/Wayland/etc到typed handle的映射；ABI batch携带source header；每个event明确window/device/user，可在设备映射变化时发布remap/hotplug事件。未知或stale handle fail closed并计数，不能落入primary window。

Bevy至少维护 `WindowId <-> Entity` 双向表并拒绝unknown window；Godot `InputEvent`有device且`InputEventFromWindow`有window id；Unreal generic input payload为相关事件携带window shared pointer，并为controller/touch携带 `FInputDeviceId + FPlatformUserId + timestamp`。Zircon不能只复制其中一种ID，还要加入跨ABI generation和stale语义。

### P1-4：window focus 被当成 application background，真实 suspend/resume 与surface失效状态机缺失

`window_lifecycle/focus.rs:10-23` 把任意primary window Focused(false)直接发送runtime Background，Focused(true)发送Foreground。`ApplicationHandler`只实现`resumed`和`can_create_surfaces`；current source没有`suspended`、`destroy_surfaces`、memory warning、low-memory、thermal/power、session lock、display disconnect或surface/device lost callback。`create_primary_window_surface`看到`self.window.is_some()`就直接成功，无法表达OS要求销毁后重建native surface。

窗口焦点、窗口可见/occluded、进程foreground、OS paused/suspended、surface unavailable、graphics device lost是不同状态轴。多窗口应用失去一个窗口焦点不应暂停整个runtime；移动端suspend必须停止present/device-dependent工作、flush关键状态、暂停或继续允许的任务，再按新surface generation恢复。目标引入显式application lifecycle state machine及顺序事件：WillSuspend、Suspended、WillResume、Foreground、Background、MemoryPressure、Thermal/PowerMode、SurfaceLost/Available、DeviceLost/Recreated。每个子系统声明pause/flush/recreate policy和deadline；resume生成新window/surface generation，旧host request/event不得命中新对象。

Bevy区分 `WillSuspend/Suspended/WillResume` 并在Android suspend移除raw handle触发surface destruction；Fyrox在`Event::Suspended`销毁graphics context并保存window attributes，Resumed重新初始化；Godot把ApplicationPaused/Resumed、FocusIn/Out和OSMemoryWarning作为不同notification。Zircon当前focus二态不能承载这些合同。

### P1-5：空 `InputDriver` 与host直接ABI注入之间没有设备/backend生命周期

Input module注册Immediate `InputDriver`，但该类型没有backend、device registry、producer、poll/wake、calibration、mapping或shutdown。App converter直接把winit/gilrs事件构造成 `ZrRuntimeEventV1` 并同步调用runtime session；gamepad是compile feature分支，keyboard/mouse依赖winit callback，没有统一driver ticket。结果是synthetic/replay、physical input、remote input和test input都通过同一个无source `InputEvent` surface进入state，却没有可审计的trust/source policy。

目标 `InputDriver` 只做真实工作：注册backend实例与设备，发布hotplug/capability/calibration/battery/mapping generation；为producer提供bounded batch writer和capacity-one wake；定义input thread/main thread affinity、pause/resume、disconnect release、clock domain转换和shutdown drain。synthetic/replay/remote source使用不同 `InputSourceKind` 与权限，不能伪装成physical device。driver不拥有UI/gameplay消费策略，但必须在进入ABI前保留source identity和order barrier。

### P1-6：高频输入仍逐样本同步穿越ABI和多层锁，coalescing发生得太晚且frame edge retention无总预算

Performance01 current-source调用图确认，cursor/raw motion/wheel/gamepad axis每个样本都单独调用V1 `handle_event`；普通未被UI消费的event至少经过global session registry、session lifecycle begin/finish、session state、core service registry和input state等多次mutex acquisition。`FrameEventBuffer`在runtime manager lock之后才合并相邻CursorMoved/MouseMotion，无法收回ABI、session lookup、UI route或lock成本；gamepad axis仍逐项跨ABI。`events: Vec<InputEvent>`对edge类没有entry/declared-byte/age上限，`InputEventQueueStatus`也只有retained/coalesced。

目标新增versioned paged/batch ingress。App在一个pump周期用可复用scratch按 `(window,device,pointer/control)` 聚合：absolute position/axis取latest，relative motion和同单位wheel饱和求和；button/touch/IME/lifecycle/hotplug/file-drop是有序barrier，先flush前一coalescible segment再保留edge。batch设置entries、bytes、age、page和decode time预算，一次进入session并在一次manager transaction应用。edge容量耗尽不能静默drop，应进入明确backpressure/fatal input-loss policy；所有drop/coalesce/queue peak/age/ABI call/lock wait可观测。gilrs producer通过event-loop proxy唤醒reactive host，不能靠持续Poll隐藏wake缺口。

### P1-7：单一 `Mutex<InputState>` 与owned snapshot复制形成全设备串行点，脚本按查询重复支付resolve/clone/scan

`DefaultInputManager`在 `default_input_manager.rs:16,47` 用一个Mutex包住所有button、cursor、touch、gamepad、IME、host request、recorder和frame queue。每个event在同一临界区reducer、clone到recorder（启用时）并push frame event；`frame_snapshot`在锁内clone pressed state和多组Vec/String/map projection。action manager又用另一个Mutex串行evaluator。产品script `gameplay_host/input.rs:16-31` 每次 `key_pressed` 都重新resolve service、取得owned `InputSnapshot`，再对pressed vector执行contains/iter；它没有消费action state或frame-local borrowed view。

目标不是把每个字段换成一把锁，而是建立single-writer frame reducer和immutable published `Arc<InputFrameState>`/dense generation。producer只写bounded batch；frame barrier一次reduce/publish；UI/action/script/gameplay以borrowed/read-only generation读取。热键/axis使用dense device/control id和索引，字符串只在binding/config边界解析。script frame context缓存input generation和action state，一个frame内重复查询不再resolve manager/clone snapshot。snapshot/diagnostics按需物化，并记录clone bytes与subscriber count。

### P1-8：Action Mapping 已有实现但没有进入产品玩法主链，raw key helper绕过context/consume/rebind合同

对 `INPUT_ACTION_MANAGER_NAME`、`InputActionManager` 和 `evaluate_actions` 的production search只找到framework trait、resolver和input module自身，没有 `zircon_app`、`zircon_editor`、script gameplay或scene system消费者。InputConfig可以安装action map，DefaultInputActionManager也注册成功，但runtime帧没有形成“UI route消费结果 -> active context stack -> evaluate once -> gameplay/script read action state”的产品链。唯一脚本输入host只暴露raw `key_pressed`。

这意味着context priority、UI consumed input、rebind、chord、axis deadzone和action edge主要存在于测试/API，而不是可交付玩法能力。目标在frame schedule中固定阶段：platform batch reduce；UI route/capture产生consumed physical inputs；player/input-user context stack更新；每个local player一次action evaluate；发布immutable `PlayerActionState`；script/scene systems读取action id。raw physical query仅用于低层工具并显式标记device/window，不能成为默认玩法API。action map变更生成generation并在frame boundary原子切换，旧script handle返回stale而非悄然读新map。

### P1-9：Input recording/replay 只是事件复制helper，不是versioned deterministic replay系统

`InputEventRecorder`有record-count capacity，但每条在 `recorder.rs:54-58` 调 `SystemTime::now()`并clone整个event；capacity不限制String/path/IME payload总bytes或age。`InputRecording`的outer `frames: Vec` 可无限push，没有format/schema version、platform/build/map/device catalog、clock domain、viewport metrics、lifecycle、random seed或frame timebase。Replay在 `recording.rs:198-199` 忽略record timestamp/sequence pacing，按frame vector顺序clone event重新submit；`from_events`甚至统一写timestamp 0。production search未找到外部recording consumer。

目标先定义用途：debug input trace、automated deterministic replay、network input log不能混成一个格式。deterministic replay header固定engine/build ABI、input schema/action-map generation、window/device/user catalog、frame/fixed-step clock、project/content hash和required subsystem seeds；frame pages有entry/byte/age总预算、checksum和completeness。monotonic source timestamp与frame assignment分离，replay按明确clock驱动并验证每帧action/world digest。未知event/version、discarded required edge或host capability不匹配必须拒绝deterministic claim。长录制流式写artifact，不在内存无限累积frames。

### P1-10：观察到的输入和发往OS的命令混在同一 `InputEvent`/recorder/state通道

`InputEvent`在 `input_event.rs:37-38,58` 同时包含 `ImeHostRequest`、`CursorHostRequest` 和 `GamepadRumbleRequest`；`InputFrameSnapshot`也把这些命令和cursor/buttons/touches并列。`DefaultInputManager::submit_event`无论event方向都在 `default_input_manager.rs:254-255` 先record再压入frame events，随后host通过另一ABI drain。结果是record/replay会把cursor grab、IME surrounding text和rumble当成“输入”重放；命令没有request id、target window/device generation、deadline、result/denial/unsupported/error ack或coalescing policy。

目标拆为 `InputSample/Event` 与 `PlatformCommand` 两条typed stream。Platform command带request id、target typed handle/generation、origin system、deadline和idempotency/coalesce key；host返回Applied/Denied/Unsupported/Stale/Failed及observed state。IME enable/geometry/surrounding text、cursor visibility/grab/position和rumble分别定义安全payload及隐私策略。deterministic trace可以选择记录request与result，但不能把command重新注入input reducer。window/device销毁会批量terminalize未完成request。

### P1-11：process supervision 虽有共享helper，但spawn/attach/termination语义仍按调用点分叉

`zircon_editor/src/core/process.rs`已实现platform process-group/Job能力，但race-free `ProcessTreeLease::attach_and_start`目前只被Play child使用。export Cargo和wizard只调用 `configure_process_tree_cancellation`；该函数在Windows不设置Job，主要依赖后续`taskkill /T /F`和direct-child fallback。compile-host `SystemZirconBuildCommandRunner`又自行spawn、pipe、wait/join，没有统一cancel/deadline/tree lease。于是Play、compile host、export Cargo、export wizard和“打开输出目录”等 `Command`调用点对descendant containment、stdin、environment allowlist、priority、resource limits、cancel escalation、reap和Drop有不同语义。

目标提供host-owned `ProcessSupervisor`：descriptor包括resolved executable identity、argument/env policy、working directory capability、tree containment、stdio mode、CPU/memory/time/output预算、cancel escalation和security provenance；`spawn`只有在platform containment建立后才允许child执行。返回 `ProcessTicket` 暴露poll/cancel/deadline/exit/output artifact与structured diagnostics，Drop必须bounded且不能静默detach。Windows Job、Unix process group/cgroup/rlimit、mobile/web unsupported在backend层实现；业务模块不再直接决定taskkill/kill细节。Play现有suspended Job路径应成为Windows backend基础，而不是被删除。

Unreal把进程句柄、pipe、terminate、return code和monitored/interactive process建立在 `FPlatformProcess` 上；Godot将create/kill置于OS abstraction。Zircon还需要更强的budget/security ticket，但共享platform owner的方向一致。

### P1-12：subprocess stream I/O没有共享资源owner，至少一个export路径仍按完整输出线性增长内存

Runtime11 open failure已经准确界定根因：Play虽有local bounded decoder/queue/drain，但reader仍是每次Play私有thread，未计入共享blocking-I/O admission/shutdown。Editor15 wizard虽流式落盘并保留bounded tail，durability仍有串行sync barrier和tail复制问题。更直接的是 `ui/host/export_cargo_process.rs:62-63,196-202` 持有完整stdout/stderr `Vec<u8>`，每个poll append，完成后在100-101再复制成两个String；1 GiB日志使RSS、copy和report serialization随输出线性增长。已有5000-line test反而锁定完整文本可见，不是常数内存合同。

目标由Runtime11或更合适的shared host I/O owner提供 `BoundedProcessStreamTicket`：固定chunk、max logical line、queue entries/bytes/age、per-poll count/bytes/time、shared reader admission、cancel/EOF/join和drop/truncate counters。完整输出流式写artifact并计算digest；UI/report只持bounded tail、artifact handle、byte count和status。terminal durability通过有deadline的persistence ticket合并flush/commit，不能在reader/caller链路串行等待多次fsync，也不能把I/O挪进无界private thread。Play、export Cargo、wizard和compile host全部消费同一合同。

### P2-1：frame cadence是硬编码策略，尚未接入present反馈、功耗/thermal、项目目标帧率与后台任务需求

`frame_cadence.rs:10` 把unfocused Game interval设为interactive 16.67 ms，也就是标记LowPower后仍可60 Hz；mobile foreground同样固定60 Hz，background固定1 s。策略没有present mode/refresh rate、runtime simulation clock、项目target FPS、battery saver、thermal throttling、audio/network/background entitlement或render workload反馈。Reactive/Immediate/After与coalesced wake是正确基础，但“Game/DesktopApp/Mobile”token不足以决定工程级cadence。

目标由FramePolicy合并simulation deadline、present feedback、input latency、animation/audio/network deadline、window visibility、power/thermal与OS background permission，输出可解释的next wake/render/sim decision。render和simulation可降频分离；unfocused multiplayer可以继续simulation但不必present 60 Hz。30/60/120/144/VRR、focused/occluded/background、电池/thermal和headless分别记录wake、tick、present、deadline miss、CPU/GPU/power与input latency，不能仅以ControlFlow枚举验收。

### P2-2：测试数量较多但缺少真实跨平台、设备、storm和进程规模证据，source-shape guard仍被当作一部分完成证明

platform/input/runtime-entry当前分别有92/53/97个tests，Play/process约16个；capability cross-product、input edge、action mapping、process tree和output budget已有行为覆盖，应保留。但input目录仍有23处`include_str!`，runtime entry和process也有source-shape guards；它们能守文件/调用锚，不能证明真实Winit WindowId映射、physical device hotplug、IME/native cursor结果、mobile surface recreation、1k Hz producer、1 GiB output、descendant escape和drop shutdown。

目标建立分层acceptance：pure reducer/action/process descriptor unit；fake backend的multi-window/device/lifecycle/property tests；Windows/Linux/macOS真实host integration；Android/iOS suspend/surface/device tests；browser/headless matrix；managed storm/resource tests；product Play/export/close/rename probe。每次产物绑定source manifest、binary hash、target/features/device/OS和profiling trace。静态guard只能标记structure passed，不能把Cargo pending或真实设备pending提升为feature accepted。

## 5. 参考引擎对齐结论

### 5.1 Bevy 与 Fyrox

Bevy `WinitWindows`维护OS WindowId与ECS Entity双向映射，unknown window event会被拒绝；lifecycle区分WillSuspend/Suspended/WillResume，Android suspend会移除raw handle；mouse motion/scroll在保留raw message的同时发布per-frame accumulator。可借鉴的是身份表、lifecycle phase和“raw edge + semantic accumulator”分层，不是照搬Bevy当前仍忽略raw `DeviceId` 的所有限制。

Fyrox executor在Resumed初始化graphics context，在Suspended销毁并保存window attributes，说明surface/context lifetime必须跟OS lifecycle走。Fyrox仍偏单主窗口且executor有固定循环假设，不能作为Zircon多窗口/高性能input最终上限；它只证明当前Zircon完全缺失suspend hook不可接受。

### 5.2 Godot

Godot `InputEvent`保存device，`InputEventFromWindow`保存window id；DisplayServer拥有window列表/焦点等OS surface；MainLoop把OSMemoryWarning、ApplicationResumed/Paused、FocusIn/Out分成不同notification；OS abstraction拥有create/kill process。Zircon应借鉴“不同状态轴和platform owner”的边界，但不能复制Godot全局singleton和缺少generation的裸整数ID。

### 5.3 Unreal

Unreal GenericApplicationMessageHandler为window事件携带具体window，为controller/touch携带PlatformUserId/InputDeviceId；GenericPlatformInputEvent额外保存enqueue timestamp。AsyncInputConsumer用MPSC fan-out、single-consumer thread invariant和复用scratch，只对absolute analog按 `(axis,device)`保留latest，其他event顺序发送。它与Zircon目标最相关的不是类名，而是source identity、thread contract和按语义coalesce。

Unreal `FPlatformProcess`及Monitored/InteractiveProcess把process handle、pipe、termination和return code放到platform层。Zircon现有Windows suspended Job lease在“防子进程逃逸”上可以比普通spawn后kill更严格，应保留并扩展到所有受管process；目标不是逐API仿制，而是在统一ticket上增加明确资源预算、artifact和security provenance。

## 6. 目标架构与所有权

### 6.1 Platform host

1. `CompiledCapabilityManifest`：只读build/target事实，不声称runtime ready。
2. `PlatformHostBackend`：进程级、主线程亲和，拥有event loop、window registry、monitor inventory、application lifecycle、process backend和preference backend安装。
3. `ObservedPlatformCapabilities`：由backend实例和真实设备生成，generation化发布。
4. `WindowRegistry`：typed generational window/viewport/surface映射，所有event/command验证target generation。
5. `ApplicationLifecycleService`：独立focus/visibility/foreground/suspend/memory/power/surface/device状态轴和ordered transition。

### 6.2 Input data path

1. platform backend注册device/user/seat和clock domain。
2. producer写bounded semantic batch；coalescible sample按类型聚合，edge形成barrier。
3. runtime一次验证batch header/bytes/sequence并在single-writer reducer提交。
4. frame boundary发布immutable input generation。
5. Runtime09 UI route生成consumed set；per-player action evaluator一次运行并发布action state。
6. script/gameplay只读action state；raw source query显式低层权限。
7. PlatformCommand独立发往host并接收structured result。

### 6.3 Process host

1. business owner提交ProcessDescriptor，不直接调用platform-specific kill/spawn。
2. ProcessSupervisor先建立tree containment/resource limits，再启动child。
3. shared stream tickets流式保存artifact、发布bounded tail和diagnostics。
4. cancel按graceful request、deadline、tree terminate、reap、reader EOF/join、artifact commit顺序执行。
5. terminal report携带exit/signal/timeout/cancel/tree/output/durability结果；Drop不能把失败吞掉成成功。

## 7. 重构顺序

### M0：合同冻结与current-source复核

- 冻结window/device/user/time/lifecycle/process ticket的typed identity与generation语义，列出所有V1 ABI producer/consumer和直接`Command`调用点。
- 对重叠Session重新取fingerprint；确认Performance01 2026-08-15 action workspace改动和input ingress活跃改动已经落定，禁止覆盖。
- 将capability字段分为compiled/requested/observed，先改文档和diagnostic语义，不以alias维持虚假Supported。

### M1：Window registry 与application lifecycle

- RuntimeEntryApp从单 `window` 改为window record map，primary window只是policy，不是存储结构。
- ABI新增versioned event batch/header，保留V1兼容入口但不再作为新产品热路径；window/viewport/device generation全链验证。
- 实现suspended/surface destroy/memory pressure/foreground/focus/occlusion分离，并让graphics/input/audio/task声明transition策略。

### M2：Input driver 与batch reducer

- 用真实InputDriver替换空type，接winit/gilrs/synthetic/replay backend registry和capacity-one wake。
- producer-side semantic coalescing、entry/byte/age/page budget、single-manager transaction和完整diagnostics。
- `DefaultInputManager`迁移为single-writer reducer + immutable frame publication；保留现有button edge和action generation语义。

### M3：Action、replay 与host command产品闭环

- 固定UI-consume -> player contexts -> evaluate -> script/gameplay阶段；删除默认玩法raw-key依赖。
- 将recording升级为versioned streaming trace/deterministic replay profile，加入completeness/digest/config/device/clock元数据。
- 从InputEvent拆出PlatformCommand/Result，并为IME/cursor/rumble实现target generation、ack、deadline和coalesce policy。

### M4：Process supervisor 与shared stream I/O

- 将Play suspended Job lease提炼成platform backend，覆盖compile/export/wizard等受管child。
- Runtime11或共享host层交付bounded stream ticket；Play私有reader和export完整Vec逐步硬切。
- 统一cancel/reap/artifact/durability report，加入orphan/rename/file-handle probe。

### M5：产品规模与跨平台验收

- 运行下述acceptance matrix并绑定source/binary/device manifest；独立review后才允许关闭Runtime11/12、Editor14/15 failures。
- 只有Zircon与参考引擎在同硬件、同场景、同输入率/窗口数/输出量下的trace可用于“性能优于”结论；静态代码形状不能证明胜过Unreal。

## 8. 验收矩阵

### 8.1 Window/lifecycle

- Windows/Linux/macOS：1/4/16 windows，create/close/recreate、focus/occlusion/DPI/monitor/fullscreen、surface loss；stale event/command命中数必须为0。
- Android/iOS：resume/suspend 100/1k cycles，surface generation、memory warning、background/foreground、rotation/IME；旧surface资源和reader/task泄漏为0。
- browser/headless：能力report只声明真实backend；headless不创建window/input physical backend，synthetic source仍有明确identity。

### 8.2 Input

- 125/500/1,000 Hz cursor/raw motion/gamepad axis，1k/10k/100k burst，1/4 windows，1/8 devices，0/1/4 UI surfaces，30/60/120 Hz consumers。
- 记录producer events、batch/pages、ABI calls、每层lock wait、queue entries/bytes/age、coalesce/drop、snapshot/action allocations、input-to-frame p50/p95/p99、CPU/RSS/power。
- absolute latest、relative sum、edge barrier、device disconnect、focus loss、UI consume、action context/rebind和multi-user映射必须语义等价；60 s停止消费后RSS/queue不继续增长。
- deterministic replay在相同content/config/seeds下逐frame action/world digest一致；不完整trace不得宣称deterministic。

### 8.3 Process

- 1/10/100 children和3层descendants，normal exit/cancel/timeout/crash/editor drop；所有descendant被contain/reap，项目目录rename和pipe EOF在deadline内成功。
- stdout/stderr：1/1k/1M lines、64 B/64 KiB/1 MiB/1 GiB unterminated line、1 MiB/1 GiB total；RSS、queued bytes、tail和reader count有常数上限，artifact byte count/digest/exit/cancel完整。
- Windows Job、Unix process group、unsupported targets分别验证；业务层不出现platform-specific taskkill/kill。

## 9. 与既有计划的关系

- Runtime12继续拥有input/action map与App producer batch；本篇要求重开“产品接线、source identity、window/device generation、deterministic replay和command split”，不否定2026-07-17 event/action局部closeout。
- Runtime10拥有versioned dynamic ABI batch和session lock收敛；Runtime09拥有UI route/capture/consumed inputs。三者必须共同验收，不能由Runtime12绕过UI或由Runtime10复制input policy。
- Runtime11 open failure继续拥有shared blocking stream I/O；Editor14只消费Play diagnostics，Editor15只消费export artifact/tail。现有local bounds是止损，不是shared owner完成。
- PlatformModule preference backend已完成的install/shutdown模式应作为新host service模板；Frameworks05的module identity closeout不等于window/input/process能力已完成。
- RHI/device loss、Editor retained multi-window和plugin remote input分别转交09/10、editor和plugin专篇；本篇拥有它们共同依赖的platform identity/lifecycle contract。

## 10. 当前状态

`review_complete；implementation_pending；recheck_required`

本篇只完成静态current-source审查和重构路由，没有修改生产代码，也没有声明Cargo、真实设备、跨平台、input storm、process storm或性能对比通过。06关联的input/app/process文件存在其他Session修改；进入M0前必须重新读取diff、运行协调器授权并复核所有行号与既有failure状态。
