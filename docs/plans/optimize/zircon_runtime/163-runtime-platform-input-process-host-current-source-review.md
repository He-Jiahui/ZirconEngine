---
title: Runtime Platform、Input 与 Process Host 当前源码复核
category: zircon_runtime
report_id: Runtime163
review_date: 2026-08-30
baseline_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
verification_head: 189f72219eaf16a6d0db880b53f3f68b4f5ee15a
canonical_owner: Runtime06
refreshes:
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
related_reports:
  - docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-dynamic-session-action-lock-domain.md
  - docs/plans/zircon_runtime/runtime/11/failure-2026-08-10-blocking-io-process-output-budget.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/2026-08-24-physical-input-before-ui-ownership-red.md
  - docs/plans/zircon_runtime/runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md
  - docs/plans/optimize/zircon_runtime/99q-runtime-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_app/08-product-host-bootstrap-loop-dynamic-runtime-shutdown-current-source-review.md
  - docs/plans/zircon_editor/editor/14/failure-2026-07-22-play-process-output-byte-budget.md
  - docs/plans/zircon_editor/editor/15/failure-2026-07-22-export-output-tail-durability-backpressure.md
related_code:
  - zircon_runtime/src/platform
  - zircon_runtime/src/input
  - zircon_runtime/src/core/framework/platform
  - zircon_runtime/src/core/framework/window
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/core/runtime/tasks/bounded_stream_io
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime_interface/src
  - zircon_runtime_host/src
  - zircon_app/src
  - zircon_editor/src/core/play
  - zircon_editor/src/core/export
  - examples/vampire/scripts
tests:
  - zircon_runtime/src/platform/tests
  - zircon_runtime/src/platform/service_types/driver/tests
  - zircon_runtime/src/input/tests
  - zircon_runtime/src/dynamic_api/session/tests/physical_input_ownership.rs
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards
reference_engines:
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/core/input/input_event.h
  - dev/godot/core/os/main_loop.h
  - dev/godot/servers/display/display_server.h
  - dev/godot/core/os/os.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericApplicationMessageHandler.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericPlatformInputEvent.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/AsyncInputConsumer.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/GenericPlatform/AsyncInputConsumer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/GenericPlatform/GenericPlatformProcess.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/MonitoredProcess.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/DynamicResolutionHandler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/HDROutputUtils.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: partial_foundations_product_host_input_and_process_closure_incomplete
source_recheck_required: true
working_tree_drift_observed_after_snapshot: true
---

# Runtime163 · Platform、Input 与 Process Host

## 1. 结论

当前工作树已经出现一批值得保留的工程化基础：Runtime 内有 typed `WindowRegistry`、`WindowStateRegistry`、display topology、`SurfaceLeaseRegistry`、`ApplicationLifecycleService`、`PlatformHostService`、host command broker、observed runtime capability report 与 event-loop scheduler；App 已实现 `suspended`、surface teardown 和 `exiting` callback；physical input 现在先提交给 `InputManager`，再进入可能停止传播的 UI route；Runtime11 的 shared bounded stream I/O 已形成，Export Cargo 也改为有界 tail；Play 的 Windows suspended spawn + Job 和 Unix process group 继续提供有价值的 tree-containment 底座。

这些改动仍没有形成完整产品闭环。`PlatformModule` 对外注册的 manager 仍只投影 `PreferenceStorage`，cleanup 也只等待 preference persistence；App 没有调用 `install_platform_host`、`publish_platform_host_ready`、window registry、display topology、surface lease、host command broker 或 Runtime scheduler。大量关键操作仍为 `pub(crate)`，而 App 通过动态 Runtime 边界运行，当前 ABI 又没有安装 host facts/command bridge 的版本化入口。因此 Runtime 内“类型存在且单元测试通过”的 owner 与真正掌握 winit/window/event loop 的 App 仍是两套状态机。

Input 仍由两行空 `InputDriver`、无 driver dependency 的 immediate managers 和 V1 逐事件 ABI 组成。`DefaultInputManager` 继续用单一 `Mutex<InputState>` 串行所有设备，frame publication 与脚本查询仍承担复制、resolve、字符串解析或扫描；Action Mapping、recording/replay 没有进入产品 gameplay 链，Vampire 仍直接查询 `W/A/S/D`。physical-before-UI 修正了一个真实顺序错误，但没有补齐 window/device/user/clock/sequence、capture generation、semantic batch、bounded edge retention 或 deterministic replay。

Process 侧仍没有所有受管 child 共用的 `ProcessSupervisor`。Play、Export 和其他调用点继续分叉 spawn、tree containment、cancel、reap、reader、artifact 与 terminal report 语义；Export 的有界 tail 关闭了“完整输出 Vec 线性增长”的一个局部后果，但 Play 尚未迁移到 Runtime11 shared owner，Windows export 的 pre-spawn cancellation helper仍不能建立 Job containment，业务层仍保留 platform-specific kill fallback。

所以 Zircon 目前不能声称达到 Unreal、Godot、Bevy 或 Fyrox 的 platform/input/process 工程完整度，更没有同硬件、同场景、同事件率与同子进程输出规模的证据证明性能优于 Unreal。本文刷新 Runtime06 的 14 项 canonical finding，**不新增唯一 finding**：12 项 P1 当前为 **7 Open、5 Partial、0 Closed**；2 项 P2 为 **1 Open、1 Partial、0 Closed**；10 项资格门为 **6 Fail、4 Partial、0 Pass**。Runtime99q/99r 的 5 项 P0 只记录当前源码候选状态，继续由原 owner 复核和关闭，不在本文重复计数。

## 2. 审查边界与冻结证据

### 2.1 统计口径

统计口径为当前工作树 UTF-8 physical lines、non-empty lines、bytes、精确 `#[test]` / `#[ignore]`；fingerprint 为每个选择文件的 lowercase `relative-path<TAB>SHA-256` 按路径排序，以 LF 连接且末尾不加 LF，再做外层 SHA-256。各组有意重叠，不能相加成去重总量。`dirty` 是选择集中 tracked modified 与 untracked 的去重文件数，只说明审查快照包含并发中的本地实现，不能作为实现已验收的证据。

| 选择集 | files | lines | nonempty | bytes | tests | ignored | dirty | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Platform / window production | **136** | **13,520** | **12,313** | **471,005** | **14** | **4** | **114** | `79cc1a17a85aaf1220b394fa3a2af122d992a586cbfc7cf14a5965bc749070a2` |
| Input production | **72** | **6,095** | **5,420** | **200,137** | **36** | **11** | **41** | `5b66766b72e1a524275661bd7b993e8bae2b8f840c6a7ee168a8ccc32e7249a8` |
| App product host | **91** | **7,354** | **6,695** | **267,023** | **111** | **0** | **59** | `1183ef05ec42d121b417713a65968adfeca49f3d3660994c10f5e02c8353e4ae` |
| ABI / dynamic session / script chain | **17** | **6,537** | **6,097** | **244,849** | **43** | **1** | **16** | `6b07c99d10cbeddf686a63d560e65646ccffb57497269c125ac9d4bd9b647f8d` |
| Process / stream chain | **59** | **13,624** | **12,393** | **462,985** | **125** | **5** | **49** | `185ac8ec84d2e903d52738d2fb8ce1f2e5ff7da041fd4b93154b98ea5d25cfc7` |
| Focused tests | **92** | **14,732** | **13,625** | **532,382** | **294** | **1** | **42** | `450305498ace67e5397715720203353b9ecedb9fd02b56b67f6334f7604515e4` |
| Five-engine reference corpus | **16** | **7,962** | **6,762** | **322,735** | **4** | **0** | **0** | `28449c87eadf09d27d45a9b7612e1861cea5601697f13cec47e3d32a483bb74e` |

本轮只做静态 review 与文档更新，没有修改 production/test/Cargo/ABI，也没有运行 Cargo、App、Editor、Runtime DLL、真实窗口、真实设备、跨平台、input storm、process storm、fault、scale、soak 或动态 benchmark。Tooling 按用户要求排除；本报告不查询、轮询、等待或实时跟踪协调器状态，也不把会话状态当作源码证据。

静态验收期间观察到 `zircon_runtime/src/dynamic_api/session/error.rs` 的写入时间晚于本报告创建时间，说明 ABI / dynamic-session 选择集已在冻结后漂移。本文不循环追逐持续变化的目录，也不在未重跑完整选择算法时局部篡改统计；`source_recheck_required` 与 `working_tree_drift_observed_after_snapshot` 因此均保持 `true`，实施或关闭finding前必须重取完整快照。

### 2.2 固定架构边界

本报告采用仓库 C3 边界，不新增第四个 package：

- `zircon_app` 拥有进程、native event loop、winit window object、主线程亲和 callback 与 host command 实际执行。
- `zircon_runtime` 拥有 platform/input 的 typed contract、registry、lifecycle、observed capability、input reducer/action/replay 与 process supervision 内部实现。
- `zircon_editor` 只拥有 authoring 与 workflow projection；Play/Export 提交受管 operation，不自行复制 platform spawn/kill/reader policy。
- App 与 Runtime 之间必须通过 versioned host ABI 安装 host instance、发布 observed facts、提交 event batch、拉取 commands/receipts。跨动态库直接共享 `Arc<PlatformDriver>` 或把 Runtime `pub(crate)` 类型公开给 App 都不是可接受产品桥。

## 3. 当前产品链与断点

### 3.1 Platform / window / lifecycle

```text
winit ApplicationHandler in zircon_app
  -> App-owned Option<Arc<dyn Window>> + local ApplicationLifecycleMachine
  -> fixed viewport handle 1 + ZrRuntimeEventV1
  -> Runtime dynamic session

Runtime PlatformDriver
  -> PlatformHostService / WindowRegistry / SurfaceLease / DisplayTopology
  -> HostCommandBroker / EventLoopScheduler / RuntimeCapabilityReport
  -X- no App product installer, publisher, command pump or event-batch ABI
```

`zircon_runtime/src/platform/module.rs` 仍只把 `PlatformManager` 包装为 `RegisteredManagerService<dyn PreferenceStorage>`，module cleanup 只处理 preference persistence。`zircon_runtime/src/platform/service_types/driver.rs` 已拥有更完整的 service graph，但产品调用搜索只命中定义和测试。App 仍保存单一 `window: Option<Arc<dyn Window>>`，以 `ZrRuntimeViewportHandle::new(1)` 发送事件，并在 hook 中丢弃 `_window_id` / `_device_id`。这使 typed registry 无法成为事实 authority，也使 stale window/surface generation 无法在 ABI admission 时被拒绝。

App 新增的 `suspended`、`WindowEvent::Destroyed` surface teardown 与 `exiting` 是真实改进；但它们驱动的是 App 私有 lifecycle machine，而不是 Runtime `ApplicationLifecycleService`。focus 仍被折叠为 foreground/background，memory warning、power/thermal、device loss、surface generation、destroy fence/receipt 及多窗口 close transaction 仍未进入产品链。

### 3.2 Input

```text
winit event
  -> one ZrRuntimeEventV1 call per sample
  -> Runtime physical state commit
  -> UI route / possible propagation stop
  -> DefaultInputManager Mutex<InputState>
  -> optional action evaluator (no product consumer)
  -> gameplay raw key query
```

`zircon_runtime/src/input/runtime/input_driver.rs` 仍只有空 `InputDriver`。`zircon_runtime/src/input/module/descriptor.rs` 虽依赖 Platform module，但 InputDriver、InputManager、InputActionManager 自身没有 backend/driver service dependency 或启动协商；默认 config 也没有证明真实 physical backend ready。`DefaultInputManager` 每个事件锁一次完整 state，`begin_frame` 清理 frame buffers，snapshot/recording/action 路径仍依赖 owned collection。event buffer只对相邻 cursor/motion做晚期合并，edge 总量、bytes、age 和 per-source fairness 没有 hard budget。

`zircon_runtime/src/dynamic_api/session/events.rs` 已把 mouse/touch/motion 的 physical commit 放在 UI route 之前，`physical_input_ownership.rs` 也证明 UI capture 外释放仍更新物理状态。这一修正只把 INP-P0-002 判为 Partial：V1 event 仍不携带合格 window/device/user/timestamp/sequence，且不存在 `InputOwnershipArbiter`、capture generation 或 batch commit receipt。

Action Mapping 与 replay 仍主要存在于 trait、实现和测试。产品示例 `examples/vampire/scripts/vampire_game/main.zr` 继续直接查询字符串 raw keys，脚本 `button_pressed` 虽避免了整份 snapshot clone，仍在调用时 resolve manager、解析 raw key并扫描状态。序列化 binding 仍可把瞬态 `GamepadId` 当作持久配置，map/trace 没有完整 schema/build/content/config/clock/source identity 与 completeness contract。

### 3.3 Process / stream

```text
Editor Play ----> private ProcessTreeLease + private reader thread
Editor Export --> Command + cancellation helper + Runtime bounded tail
Other callers --> their own Command / kill / wait policy
                   -X- no common ProcessSupervisor terminal contract
```

Play 的 Windows suspended process + Job lease 和 Unix process group 应保留；它们证明可以在 child 执行前建立 containment。问题是该逻辑仍属于 Play，而不是 App/Runtime 的公共 process host。Export 已使用 256 KiB `BoundedOutputTail`，关闭了完整 stdout/stderr `Vec` 的直接线性增长；但它没有让所有 reader 共享 Runtime11 ticket，也没有统一 artifact durability、EOF/join、cancel deadline、descendant census 与 terminal report。

`configure_process_tree_cancellation` 在 Windows export 的 spawn 前阶段不能像 Play 一样建立 Job，后续仍依赖 `taskkill` fallback。只要业务 owner 仍能直接 `Command::spawn`、选择不同 reader 与 kill 顺序，就无法证明 editor drop、timeout、crash 或 1 GiB unterminated output 后没有 orphan、泄漏、死锁或项目目录句柄残留。

## 4. Runtime06 canonical finding 重判

状态规则：`Closed` 需要产品调用链和对应资格证据；`Partial` 表示当前源码确有可保留实现且至少一个旧后果已消除，但 owner/consumer/ABI/scale 仍未闭环；“新建类型 + 单元测试 + 产品零调用”最多只能支持 Partial。

| Canonical finding | 当前状态 | 当前源码判据 | 硬切重构要求 |
|---|---|---|---|
| P1-1 PlatformModule 公开职责与真实 owner 不一致 | **Partial** | PlatformDriver 已聚合 host/window/display/lifecycle/surface/command/scheduler，但 module manager 与 cleanup 仍只覆盖 preference，App 产品零接入 | 让 module lifecycle 管理完整 platform service graph；通过 versioned host ABI 安装/ready/quiesce/stop，删除双 lifecycle authority |
| P1-2 capability 是编译/目标推导而非运行时协商 | **Partial** | compiled、requested、observed/runtime report 已分型，disabled 也可表达；产品没有 provider/evidence publisher/consumer | App 发布 host/device evidence 与 generation；Runtime 只在 observation 完成后声明 Ready，并给 graphics/input/editor 使用 |
| P1-3 window/viewport/device/user/time identity 丢失 | **Open** | App 单窗口、固定 viewport 1，hook 丢弃 window/device；V1 event 无 user/time/sequence/generation | 新建 versioned event batch header 与 typed IDs；window registry validation 成为 admission gate；V1 退出产品热路径 |
| P1-4 focus 与 app background 混淆，suspend/surface 状态机缺失 | **Partial** | App 已有 suspended/destroyed/exiting teardown，但仍是本地 machine，focus 仍折叠，缺 memory/power/thermal/device/surface generation | 建立正交 lifecycle axes、ordered transition transaction、surface lease fence/receipt，并覆盖 100/1k cycle |
| P1-5 空 InputDriver、无 backend 生命周期 | **Open** | `InputDriver` 仍为空；manager 无 driver dependency；physical/synthetic/replay/gamepad source 无统一 register/start/stop/failure | InputDriver 拥有 backend registry、device/seat/user inventory、source clock、capacity-one wake、shutdown census |
| P1-6 高频输入逐样本 ABI/多锁，coalescing 太晚且 edge 无总预算 | **Open** | V1 每样本同步调用；manager每事件锁；frame buffer晚期相邻合并；无 entries/bytes/age/page/fairness 总预算 | producer semantic batch；absolute latest、relative sum、edge barrier；一次 batch admission/commit 与 bounded diagnostics |
| P1-7 单 Mutex、snapshot复制、脚本重复 resolve/scan | **Partial** | 脚本路径减少整 snapshot clone，但 `Mutex<InputState>`、owned frame snapshot 和逐查询 raw-key成本仍在 | single-writer reducer + immutable generation；一次 action evaluation；脚本只读 cached action state |
| P1-8 Action Mapping 无产品 gameplay 主链 | **Open** | evaluator 只在实现/测试命中；Vampire仍 `key_pressed("W")` 等 raw keys | 固定 UI consume -> player contexts -> action evaluate -> gameplay/script 顺序；删除默认玩法 raw-key依赖 |
| P1-9 recording/replay 不是 deterministic system | **Open** | recorder/replay helper能复制事件，但无 versioned trace、completeness、digest、clock/config/device identity 或 world result | streaming trace artifact + deterministic profile；逐 frame action/world digest；incomplete trace fail closed |
| P1-10 输入 observation 与 OS command 混在通道 | **Open** | cursor/IME/rumble request仍与 input event/state/recording强耦合，缺 target generation/ack/deadline | 独立 `PlatformCommand/Result` schema、broker、coalesce和structured failure；recording只录确定性 observation |
| P1-11 process supervision 语义按调用点分叉 | **Open** | Play containment较强但局部；Export/其他 caller仍各自 spawn/kill/wait，Windows export helper不建 Job | App host 执行、Runtime common ProcessSupervisor；统一 descriptor/ticket/tree/cancel/reap/terminal report |
| P1-12 subprocess stream I/O owner 分裂/完整输出增长 | **Partial** | Runtime shared bounded lane存在；Export Cargo使用256 KiB tail；Play仍私有thread reader且未迁移，artifact/durability未统一 | 所有受管 child 硬切到 shared stream ticket；删除私有 reader；统一tail/artifact/digest/EOF/join receipt |
| P2-1 frame cadence 是硬编码策略 | **Partial** | App已有 Continuous/Reactive/LowPower/Fixed 与10 Hz/1 Hz降频；Runtime scheduler类型未接产品，仍硬编码16.67 ms等 | 接 refresh/present feedback、per-window demand、project target、power/thermal/background task；统一scheduler telemetry |
| P2-2 缺真实跨平台、设备、storm和进程规模证据 | **Open** | 有较多unit/source-shape测试；无真实device/multi-window/100k burst/100 child/1 GiB/功耗或对照benchmark | 执行第9节矩阵并绑定source/binary/target/device manifest和trace；静态guard不得提升为accepted |

汇总：P1 为 **7 Open / 5 Partial / 0 Closed**；P2 为 **1 Open / 1 Partial / 0 Closed**。

## 5. 专项 P0 去重状态

本节不增加或关闭专项 P0，只防止后续实施把旧措辞当作当前事实。

| Specialist owner | 当前源码候选状态 | 说明 |
|---|---|---|
| Runtime99q PLH-P0-001：平台能力 truth/host readiness 不可信 | **Partial candidate** | observed runtime report与host evidence合同已出现，但 App 无产品 provider/publisher/consumer；不能关闭 |
| Runtime99q PLH-P0-002：surface/native handle teardown 无受管资格 | **Partial candidate** | App已在 suspended/destroyed/exiting执行surface teardown，但未走 generation-qualified SurfaceLease/fence/receipt产品链 |
| Runtime99r INP-P0-001：输入 source identity/qualified ingress 缺失 | **Open** | V1仍丢 window/device/user/time/sequence，空 InputDriver 未建立source registry |
| Runtime99r INP-P0-002：UI capture 可遮蔽物理状态提交 | **Partial candidate** | physical-before-UI顺序和回归测试已存在；缺 ownership arbiter/capture generation/batch receipt |
| Runtime99r INP-P0-003：高频输入无有界事务与确定性顺序 | **Open** | per-sample ABI、晚期coalesce、无edge总预算和deterministic batch order均未改变 |

Runtime11 的 open failure 已交付 shared `bounded_stream_io` 源码/行为合同，但 Editor14 Play 尚未迁移，产品 perf/power 与受管 Cargo 验证也未完成；它仍是 open owner，不因 Export 局部 tail 自动关闭。

## 6. 参考引擎差距

### 6.1 Bevy 与 Fyrox

Bevy `WinitWindows` 维护 native `WindowId` 与 ECS Entity 的双向表，并在window移除时清理映射；`bevy_winit::state` 把 `WillSuspend`、`Suspended`、`WillResume` 和 `Resumed` 分开；mouse motion/scroll同时保留raw events与frame accumulator。Zircon当前 Runtime registry 的数据结构方向合理，但没有 App bridge，所以还没有达到 Bevy 的实际 host-to-world identity闭环。目标应继续向 generation、stale rejection、多用户和ABI批处理推进，而不是复制Bevy仍会忽略部分native device identity的限制。

Fyrox executor在 `Resumed` 初始化 graphics context，在 `Suspended` 销毁并保留window attributes。Zircon App新增的surface teardown与这一最低原则对齐，但 Runtime SurfaceLease没有进入产品路径，仍不能证明graphics resources、input devices和reader/task按同一transition完成quiesce。

### 6.2 Godot

Godot `InputEvent` 保存device，`InputEventFromWindow` 保存window id；`DisplayServer` 提供多窗口/display OS authority；`MainLoop` 将memory warning、application resume/pause、focus in/out拆成不同notification；`OS`抽象承接process create/kill。Zircon不应复制其global singleton或裸整数ID，但必须达到同等的状态轴分离和业务层不直接执行platform-specific process policy。

### 6.3 Unreal

Unreal `GenericApplicationMessageHandler` 为window事件携带具体window，并为controller/touch携带platform user与input device；`GenericPlatformInputEvent`保存enqueue timestamp。`AsyncInputConsumer` 明确MPSC producer、single consumer和scratch复用，只对可安全覆盖的absolute analog按source保留latest，其余edge保持顺序。Zircon当前逐样本V1 ABI、单Mutex和无source identity的晚期coalesce与这条工程边界仍有根本差距。

`GenericPlatformProcess` 与 `MonitoredProcess` 把process handle、pipe、cancel、return code和monitoring放在platform层。Zircon Play的pre-spawn Job containment是可保留优势，但只有所有受管child共用同一 supervisor、stream owner和terminal report后，才能形成可比较的系统能力。

### 6.4 Unity Graphics

本轮本地 Unity Graphics 镜像只提供 dynamic resolution 与 HDR output policy/state的参考，适合约束window/display变化如何影响render output，不提供通用OS input或process authority。它不能被用来证明 Zircon platform/input/process 已对齐 Unity Engine；相关差距应以 Unreal/Godot/Bevy/Fyrox 的可见源码和 Zircon 自身产品资格为准。

## 7. 目标架构

```text
zircon_app (native owner)
  event loop / windows / displays / process execution
       |
       | versioned host ABI: install + facts + batches + commands + receipts
       v
zircon_runtime (semantic authority)
  PlatformDriver
    WindowRegistry + Lifecycle + SurfaceLease + ObservedCapabilities
    HostCommandBroker + EventLoopScheduler
  InputDriver
    SourceRegistry -> BoundedBatchReducer -> FrameGeneration -> Actions/Replay
  ProcessSupervisor
    Descriptor -> Containment -> StreamTickets -> Cancel/Reap -> TerminalReceipt
       ^
       |
zircon_editor (workflow projection)
  Play / Export / Build operations; no private spawn/kill/reader policy
```

### 7.1 必须硬切的公共合同

1. `PlatformHostInstanceId + generation`：一次App host实例只能安装一次，ready/degraded/quiesce/stopped都带evidence和有序generation。
2. `WindowId/ViewportId/SurfaceId/DisplayId`：都包含owner/generation；primary window是policy role，不是固定存储槽或数字1。
3. `PlatformEventBatchV2`：header包含host/window/device/user/clock/first-sequence/count/bytes；payload按语义coalesce并保留edge barrier。
4. `InputFrameGeneration`：reducer单写，consumer只读immutable snapshot；UI consume与action state使用同一input generation。
5. `PlatformCommandV1`：cursor/IME/rumble/window命令独立于observed input，带target generation、deadline、idempotency/coalesce policy和receipt。
6. `ManagedProcessTicket`：descriptor、containment、stdout/stderr tickets、artifact、cancel/reap与terminal report绑定为一个owner lifecycle。

旧V1入口只允许在迁移清单中短期存在，不能新增caller、alias或第二authority。硬切完成后删除旧字段、旧helper、私有reader和直接业务 `Command::spawn` 路径，不维持双写兼容层。

## 8. 重构序列

### M0：冻结 manifest 与产品调用图

- 生成 window/device/user/clock/process descriptor 的 Schema/ABI/Owner/Operation manifest。
- 列出所有 V1 input producer/consumer、App local lifecycle状态、Runtime platform service定义及直接 `Command` caller。
- 把 compiled/requested/observed capability diagnostic分开；没有runtime evidence时必须是Unavailable/Degraded，不能用target推导成Ready。

### M1：接通 App host 与 Runtime platform authority

- App启动时通过versioned ABI安装唯一platform host，并发布真实backend/display/window/lifecycle evidence。
- 单 `Option<Window>` 硬切为window record map；固定viewport 1退出产品路径。
- close/suspend/surface loss使用Runtime transaction与lease receipt；module cleanup覆盖host quiesce、command drain、surface release和registry census。

### M2：InputDriver 与 bounded batch reducer

- 让InputDriver真正管理winit/gamepad/synthetic/replay source生命周期、device/seat/user inventory与clock/sequence。
- 建立producer-side semantic batch、entry/byte/age/page/fairness预算、capacity-one wake和完整drop/coalesce diagnostics。
- `DefaultInputManager`硬切为single-writer reducer + immutable publication；去掉热路径逐样本ABI和全state锁竞争。

### M3：Action、UI ownership 与 deterministic replay

- 固定 physical commit -> UI route/consume -> per-player context -> action evaluation -> gameplay/script 的单一阶段序。
- Gameplay和默认项目只消费action state；raw input只留明确低层权限接口。
- replay artifact记录schema/build/content/config/source/clock/completeness/digest；逐frame action/world digest必须可复验。

### M4：ProcessSupervisor 与共享 stream owner

- 将Play containment提炼到App/Runtime共用backend，覆盖Play、Export、Build与其他受管child。
- 所有stdout/stderr迁移到Runtime11 bounded stream tickets；统一tail、artifact、digest、reader EOF/join和durability receipt。
- cancel严格执行 graceful request -> deadline -> tree terminate -> reap -> reader join -> artifact commit；Drop失败不得伪装成成功。

### M5：Cadence 与平台状态闭环

- Runtime scheduler接入App事件循环，使用per-window refresh/present feedback、项目target、task demand与power/thermal/background policy。
- graphics/input/audio/task/process consumer声明各lifecycle phase的quiesce/resume策略，禁止focus替代application lifecycle。

### M6：资格与性能对照

- 执行第9节矩阵，产物绑定source hash、binary hash、target/features、OS/device、配置和trace。
- 只有同硬件、同场景、同窗口/设备/事件/输出规模的Zircon与参考实现trace，才允许形成“优于Unreal”的性能结论。

## 9. 资格门重判

| Gate | 当前状态 | 缺失证据 |
|---|---|---|
| G1 Desktop 1/4/16 windows、display/DPI/fullscreen/surface loss、stale rejection | **Fail** | App仍单窗口/固定viewport，Runtime registry没有产品publisher或consumer |
| G2 Android/iOS 100/1k suspend-resume、surface generation、memory/background/IME | **Partial** | App callback与teardown存在；无Runtime transaction、generation/fence和真实设备循环 |
| G3 Browser/headless observed capability 与 synthetic identity | **Partial** | compiled/observed类型存在；无产品provider/evidence与真实target矩阵 |
| G4 125-1000 Hz、1k-100k burst、多window/device/UI surface输入 | **Fail** | 仍是per-sample ABI、single Mutex、无总预算；未运行storm |
| G5 输入queue/lock/allocation/latency/RSS/power完整观测 | **Fail** | 缺产品metric与trace，单元测试不能给出p50/p95/p99或功耗 |
| G6 absolute/relative/edge/disconnect/focus/UI consume/action/multi-user语义 | **Partial** | physical-before-UI已修；source identity、multi-user、action产品链和qualified ordering缺失 |
| G7 Deterministic replay逐frame action/world digest | **Fail** | helper无完整trace合同与产品replay profile |
| G8 1/10/100 children、3层descendant、exit/cancel/timeout/crash/drop | **Fail** | supervision按caller分叉；无统一census/reap/terminal receipt与scale run |
| G9 1 GiB line/total输出的常数内存、tail、artifact/digest/EOF | **Partial** | Runtime shared lane和Export bounded tail存在；Play未迁移，未运行1 GiB或durability验证 |
| G10 Windows Job/Unix group/unsupported backend统一且业务无platform kill | **Fail** | Play局部满足，Export/其他caller仍分叉且保留taskkill/kill策略 |

汇总：**6 Fail / 4 Partial / 0 Pass**。

## 10. 当前状态

`review_complete；implementation_partial；product_closure_pending；recheck_required`

本文只完成 Runtime06 当前源码重判、参考引擎对照与依赖顺序，不修改任何生产实现。冻结fingerprint来自成文快照；相关源目录含大量未提交改动，进入M0或判定任何finding关闭前必须重新读取current source、重取fingerprint，并执行对应动态资格门。没有Cargo、真实平台、设备、process规模或同硬件benchmark证据时，任何“功能完整”或“性能优于Unreal”的声明均不成立。
