---
title: Runtime Platform Host、Window Registry、Display、Event Loop、Application Lifecycle 与 Surface Command 当前工作树复核
category: zircon_runtime
report_id: Runtime191
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_runtime/src/core/framework/window
  - zircon_runtime/src/core/framework/platform
  - zircon_runtime/src/platform
  - zircon_app/src/entry/runtime_entry_app
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/surface_lifecycle.rs
  - zircon_app/src/reference_cpu_presenter.rs
tests:
  - zircon_runtime/src/core/framework/window/tests.rs
  - zircon_runtime/src/platform/tests
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards
plan_sources:
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99q-runtime-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/103-runtime-clock-time-policy-world-fixed-step-timer-cadence-current-source-review.md
  - docs/plans/optimize/zircon_runtime/157-runtime-core-module-lifecycle-registry-service-resolution-shutdown-current-source-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
reference_engines:
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/servers/display/display_server.h
  - dev/godot/platform/windows/display_server_windows.h
  - dev/godot/core/os/main_loop.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericWindow.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/Windows/WindowsApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderer.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateRHIRenderer/Private/SlateRHIRenderer.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/DynamicResolutionHandler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/HDROutputUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettingsHDROutput.cs
doc_type: current_working_tree_review_and_refactor_plan
review_status: complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
supersedes: Runtime116
---

# Runtime191 · Platform Host 与 Surface 生命周期当前工作树复核

## 1. 结论

当前工作树已经不再是 Runtime57/116 所描述的“只有 descriptor 和单窗口 helper”。`zircon_runtime` 里已经有一套相当完整的 typed control plane：`WindowId`/`WindowRegistryId`/`NativeWindowId` 带 generation，`WindowRegistry` 维护 native/engine 双索引和 parent graph，`WindowStateRegistry` 分开 create/requested/observed/effective state，`WindowCommand` 与 `HostCommandBroker` 提供 deadline、serial lane 和 terminal receipt，`DisplayTopologySnapshot` 提供稳定 `DisplayId`、usable rect、scale、refresh、HDR/VRR/color-space 字段，`SurfaceLeaseRegistry` 提供 prepared/published/retired lease，`PlatformHostService` 与 `ApplicationLifecycleService` 也有 health/quiesce/terminal 状态机。`EventLoopScheduler` 已经测试 source、deadline、lateness、backlog、starvation 和 background policy。

但这些实现主要构成“可测试的 Runtime 平台控制平面孤岛”，没有成为真实产品的 owner。逐个检查生产引用后，`install_platform_host`、`publish_platform_host_ready`、`submit_window_command`、`prepare_surface_lease`、`schedule_event_loop_wake`、`with_window_registry`、lifecycle publish/resume/running 等操作都只有定义处，或只有 unit/source guard；`RuntimeEntryApp` 仍自己持有一个 `Option<Arc<dyn Window>>`、一个 descriptor、一个私有 cadence 和一个私有 lifecycle machine。它丢弃 winit `WindowId`，把所有事件路由到 viewport 1，Runtime dynamic lifecycle handler也不更新 `PlatformDriver`。因此现在存在两个相互独立的 platform authority：Runtime 的完整模型和 App 的实际单窗口系统。

这使得本轮两个父 P0 从 **Open 重判为 Partial**，但不能关闭：

1. `PLH-P0-001`：capability report 已经区分 planning catalog 与 Disabled/Unavailable/Ready/Degraded/HostUnavailable/NotObserved，并消费 `PlatformConfig.enabled`、owner lifecycle 和 observed evidence；但没有真实 backend instance/producer 接入，App 也不通过该 owner 发布 readiness。
2. `PLH-P0-002`：App 的 CloseRequested、Destroyed、suspended、destroy_surfaces、resize 路径已经按 unbind-before-window-drop 和 Graphics replacement transaction 排序；但 ABI 仍传 raw Win32 handle，qualified `SurfaceLease` 没有跨 App/Runtime/Graphics 边界，平台 registry 的 lease transaction没有调用者。

本轮重判旧账本为 **64 项 P1：18 Open、45 Partial、1 Inherited；16 项 P2 Open；40 个资格门：15 Fail、25 Partial、0 Pass**。P1 的 Partial 表示 typed 模型或局部产品路径存在，但还不能证明真实 owner、跨层调用、generation 传递和多平台资格；不是“完成”。没有任何证据支持“性能和表现优于 Unreal”，本报告只确认工程化结构、正确性风险和后续验收条件。

本轮只做 review 与计划记录，没有修改 Runtime/App/Editor production、Cargo、ABI、ZUI 或测试，没有运行 Cargo、真实多窗口/多显示器、DPI/hotplug、移动端、WGPU/GPU、fault、soak 或 benchmark。Tooling 仍按用户要求排除，`PLH-P1-063`只保留为继承项，不重新计数。

## 2. 审查边界、统计与证据等级

本轮沿以下完整链路逐文件检查：

```text
OS/winit callback
  -> RuntimeEntryApp lifecycle/window event
  -> dynamic Runtime session + viewport bind
  -> PlatformDriver/Manager/WindowRegistry/DisplayTopology (当前未接通)
  -> SurfaceLeaseRegistry (当前未接通)
  -> RenderFramework ViewportSurface
  -> WGPU surface lifecycle / present / CPU reference presenter
```

| 选集 | 文件 | 物理行 | bytes | `#[test]` | fingerprint |
|---|---:|---:|---:|---:|---|
| Platform control plane（window/framework/platform） | 229 | 23,844 | 815,061 | 221 | `daca567954a896ee68463aa7f174664d80e534a52df065679591b43bc960ff6f` |
| App product host（runtime_entry_app/runtime_library/reference presenter） | 114 | 13,451 | 485,285 | 204 | `90a87ecce1e4211523b348367684f8579b74f80a8b4fb5ee04f1b85f04bd7f83` |
| Dynamic ABI + Graphics surface | 39 | 6,093 | 205,475 | 42 | `d78085d0d3e14d6bcce3ea9635f2126b5e0950255063d01fe1016e543dd35bba` |
| focused unique union | 382 | 43,388 | 1,505,821 | 467 | `5a4384d1c7c9e8d5374c970913c77cfd130f0867ffdfdf6e20519d40c64aa092` |
| Unreal/Bevy/Fyrox/Godot/Unity Graphics selected references | 29 | 33,239 | 1,264,240 | n/a | `a7a04d5299ec6897e8fe7bbc2544eedea4713f2f7679ff1326d55e7081d2b971` |

统计选集允许目录之间重叠，不能把行数相加冒充全仓库规模。fingerprint按仓库相对路径排序、逐文件 lowercase SHA-256 后对 `path|hash` payload 计算 SHA-256。`#[test]`数字包含目录中的测试代码，不能解释为产品资格。

证据等级：E3 是当前工作树生产代码、调用点、字段和局部测试；E2 是本地五引擎源码对照；E1 是 source guard/unit test/descriptor 意图；E0 是尚未运行的真实 OS/GPU/soak/performance qualification。`source_recheck_required` 保持 true。

## 3. 实际生产接线审计

### 3.1 Runtime 平台操作的调用点结果

下表是对 `zircon_runtime/src/core/framework/platform`、`zircon_runtime/src/core/framework/window`、`zircon_runtime/src/platform`、`zircon_app/src/entry/runtime_entry_app` 与 dynamic session 的精确生产引用检查。每行的“生产引用”排除了定义和 test module。

| 操作 | 定义 | 生产引用 | 结论 |
|---|---:|---:|---|
| `install_platform_host` / `publish_platform_host_ready` | 有 | 仅定义 | Host backend没有进入启动链 |
| `request_platform_host_quiesce` | 有 | 仅定义 | App关闭没有调用Runtime host quiesce |
| `submit_window_command` / `dispatch_next_window_command` / `complete_window_command` | 有 | 仅定义 | WindowCommand不是产品 API |
| `prepare_surface_lease` / `publish_surface_lease` | 有 | 仅定义 | Surface lease没有跨ABI传递 |
| `begin_window_close_tree_after_quiesce` / `begin_application_suspend_after_quiesce` | 有 | 仅定义 | Registry transaction未接App lifecycle |
| `schedule_event_loop_wake` / `take_due_event_loop_wakes` | 有 | 仅定义 | App仍使用私有 RuntimeFrameCadence |
| `with_window_registry` / `install_host_command_broker` | 有 | 仅定义 | Manager/driver不承载实际窗口 |
| lifecycle publish/resume/running methods | 有 | 仅定义 | App lifecycle和Runtime lifecycle双写 |

因此，不能把这些定义的存在写成“Window Registry 已完成”或“EventLoopScheduler 已接入”。它们当前应视为下一阶段的 canonical contract 候选。

### 3.2 App 仍是实际 authority

- `RuntimeEntryApp` 只有一个 `window: Option<Arc<dyn Window>>`、一个 `WindowDescriptor`、一个 `viewport_size`，并硬编码 `ZrRuntimeViewportHandle::new(1)`。
- `ApplicationHandler::window_event(..., _window_id: WindowId, ...)` 丢弃 native ID；`device_event` 也丢弃 `DeviceId`。dispatch 无论事件来自哪个窗口，都进入同一个 viewport/state。
- `resumed`、`can_create_surfaces`、`suspended`、`destroy_surfaces`、`exiting` 已有局部处理，但生命周期状态保存在 App 私有 `Cold/AwaitingSurface/SurfaceActive/Suspended/Exiting` 机器，没有写入 `PlatformDriver` 的 generation/terminal receipt。
- App 私有 cadence 仍硬编码 headless/interactive/mobile/unfocused/background 的 16ms、16.666ms、100ms、1s policy；Runtime `EventLoopScheduler` 没有收到 winit `StartCause` 或 native wake source。
- `WindowFocused` 会改变 cadence 并发送 Runtime `FOREGROUND/BACKGROUND`，把窗口 focus当成 application activation；可见性、occlusion、suspend、surface availability没有同一权威。
- `PlatformModule` 虽注册 PlatformDriver 和 lazy PlatformManager，但 Manager 以 `RegisteredManagerService<dyn PreferenceStorage>` 暴露，调用者拿不到 typed PlatformManager snapshot/operation API。App runtime bootstrap没有建立 App-side BuiltinEngineEntry/CoreHandle 到该 service 的桥。

### 3.3 Surface 与 ABI 现状

- App 的 `surface_present/lifecycle.rs` 在关闭、暂停、destroy 时先 unbind 再释放 native window；resize 会更新 viewport 后重新 bind。Graphics `ViewportSurface` 先创建 replacement、`finish_submission()`、原子替换 surface/extent、释放 history；WGPU RHI destroy/reconfigure 会 settle abandoned submission tickets。这些是可靠的局部顺序。
- `ZrRuntimeBindViewportSurfaceRequestV1` 仍只有 ABI version、viewport、size 和 `ZrRuntimeNativeSurfaceTargetV1`；target 只支持 NONE/WIN32，包含 raw `u64 window_handle/display_handle`。没有 WindowId/registry generation、DisplayId/topology generation、SurfaceLease generation、present mode、format、color space、fallback reason 或 terminal receipt。
- `dynamic_api/surface.rs` 只接受 Win32，并把零尺寸 clamp 到 1；Graphics backend 固定 `PresentMode::Fifo` 和 `TextureFormat::Bgra8UnormSrgb`，只接受 BGRA/RGBA sRGB。没有 HDR/wide gamut/VRR/refresh negotiation，`WindowDescriptor.present_mode`也不消费。
- CPU presenter 已是显式 `--reference-cpu-presenter` degraded path，并记录 frame/copy bytes/drop/latency；但能力没有进入 Platform Host readiness，也没有与多窗口/显示拓扑建立 target contract。

## 4. 可保留底座与不可误报项

### 4.1 可以保留

- generation-qualified identity、双向 registry、parent graph、primary role、child-first close tree 和 stale/closing typed errors；
- create/requested/observed/effective state 与 `WindowCommandAccepted`/`WindowCommandReceipt` 的字段划分；
- immutable `DisplayTopologySnapshot`、display geometry/usable rect/scale/refresh/orientation/HDR/VRR/color-space 字段；
- `SurfaceLeaseRegistry` 的 prepare/publish/retire 以及 Graphics replacement transaction；
- Platform host/lifecycle health/quiesce 状态机和 capability report 的 planning/runtime 分层；
- EventLoopScheduler 的 source/deadline/lateness/backlog/starvation 统计；
- 显式 degraded CPU presenter 和 WGPU submission settlement。

### 4.2 当前不能写成完成

- registry/lease/scheduler 的 unit tests 不能证明 App 生产路径已经调用它们；
- PlatformManager 的 public methods 不能替代 typed manager service exposure；
- winit `ApplicationHandler` 的回调覆盖不能替代 application lifecycle owner；
- App 单窗口安全 teardown 不能替代跨平台、跨generation ABI lease；
- fixed FIFO/sRGB Win32 surface 不能替代 output negotiation；
- 仅有 source guard 和 `include_str!` 不能替代真实 native multi-window、hotplug、DPI、surface-loss、suspend/resume qualification。

## 5. 本地 P0 阻断项

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PLH-P0-001 | Partial | `PlatformConfig.enabled` 已参与 runtime capability projection，report 已有 Disabled、FeatureDisabled、Unavailable、Ready、Degraded、HostUnavailable、NotObserved；但真实 `PlatformHostBackend` instance、observed producer 和 App bridge 没接入，不能证明 Ready来自当前OS owner。 | `PlatformActivationPlan` 与 observed `CapabilityState` 分离；Compiled/Selected/Installed/Initialized/Observed/Ready 每层带 provider instance、generation、evidence、typed failure；disabled 不得创建 owner，App 只能通过该 owner获取平台能力。 |
| PLH-P0-002 | Partial | App CloseRequested/Destroyed/suspended/destroy_surfaces/resize 已有有序 unbind 和 Graphics replacement；但 ABI 仍传 raw Win32 handle，Platform lease/registry transaction没有调用者，跨层无法证明 window/output/surface generation 一致。 | `SurfaceLease { window_id, window_generation, display_id, topology_generation, surface_generation }` 进入 App-Runtime-Graphics；prepare -> settle/fence -> publish -> retire 具有唯一 terminal receipt，native destroy/suspend/backend loss 共用同一 transaction。 |

继承发布阻断 `TOOL-EXPORT-P0-005` 仍由 Tooling03 拥有，本文不重复计数；在该阻断关闭前，generated mobile/browser callback不能发布 Platform Ready。

## 6. P1 工程化差距总账

状态含义：`Open` 表示当前链路仍缺少产品级 owner/字段/语义；`Partial` 表示局部 typed 底座或单窗口路径存在，但接线、generation、跨平台或资格证据不足；`Inherited` 表示由其他报告拥有，本轮不重复计数。

### 6.1 Authority、Capability 与 Backend Readiness

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-001 | Partial | PlatformModule已声明windowing/OS integration，PlatformDriver也已有字段，但真实 backend 没有从模块启动。 | descriptor按真实service/provider发布能力，Platform Host成为唯一window/display/event-loop owner。 |
| PLH-P1-002 | Partial | `PlatformHostBackend` trait已有instance/generation/health/lifecycle，但没有App/native实现或线程亲和桥。 | 每个shipping backend实现start、thread affinity、health、quiesce、restart、terminal。 |
| PLH-P1-003 | Partial | PlatformManager已有host/lifecycle/display/capability snapshot，但注册为`PreferenceStorage`类型擦除，调用者拿不到typed manager。 | 发布专用`PlatformManager` service handle；snapshot只读，operation通过driver receipt提交。 |
| PLH-P1-004 | Partial | runtime capability report已有planning/runtime区分，但planning target仍可构造，未绑定实际BuildSet。 | BuildSet冻结compiled target；runtime仅接受兼容observed target，planning使用不同类型。 |
| PLH-P1-005 | Partial | report已区分Disabled/Unavailable/Ready等状态，compile feature仍没有真实 backend evidence。 | Compiled、Selected、Installed、Initialized、Observed、Ready分层并禁止跨层推断。 |
| PLH-P1-006 | Partial | host snapshot含owner/generation类型，App没有发布或消费。 | 能力与provider instance/session generation绑定，owner退出自动撤销Ready。 |
| PLH-P1-007 | Partial | status枚举已扩展Starting/Ready/Degraded/Failed等，但实际 product status不经过它。 | 所有Runtime/App查询使用统一 typed status；静态 catalog不得冒充runtime readiness。 |
| PLH-P1-008 | Partial | evidence record和currentness字段已有，缺少native probe/version/device/display producer。 | bounded evidence含probe time、backend version、device/output identity、generation和stale reason。 |
| PLH-P1-009 | Open | platform tests仍可通过人工矩阵和source guard，未实例化真实 backend。 | planning tests 与 backend qualification 分离，至少有 deterministic host 和 Windows product harness。 |
| PLH-P1-010 | Partial | host lifecycle有quiescing/quiesced/failed/stopped模型，但没有App关闭、restart、leak report调用。 | Module lifecycle接入Host operation receipt，health失败立即撤销Ready并取消未完成命令。 |

### 6.2 Window Identity、Registry 与 Multi-Window

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-011 | Partial | `WindowRegistry`能管理任意slot，但App仍是单一`Option<Window>`。 | App创建、查询、销毁全部通过Registry，支持多窗并返回qualified handle。 |
| PLH-P1-012 | Partial | primary role和replacement模型存在，App仍用`Option`/helper判断primary。 | primary是registry中的qualified role handle，exit policy读取registry事实。 |
| PLH-P1-013 | Partial | typed WindowId不可持久化已存在，`PrimaryWindowHandle(u64)`仍可serde且无owner/generation。 | live handle只在运行时存在；配置只保存 placement/profile key。 |
| PLH-P1-014 | Open | App所有事件固定viewport 1，没有无viewport工具窗或多viewport绑定。 | Window-to-Viewport binding支持多窗、工具窗、popup、modal和独立 render target。 |
| PLH-P1-015 | Open | winit `WindowId`参数仍被丢弃。 | native ID先经双向registry解析，unknown/stale拒绝并生成诊断。 |
| PLH-P1-016 | Partial | Registry双索引和remove同步清理已实现，App没有接入；Bevy `WinitWindows`提供了同类生产语义。 | engine/native两索引在一个generation事务内原子更新，并由产品事件路径调用。 |
| PLH-P1-017 | Partial | slot generation与wrap retirement已有，实际window重建不携带它。 | 每次native重建递增generation，旧event/command/surface fail-close。 |
| PLH-P1-018 | Partial | Unknown/Stale/Closing typed error已有，因事件未解析仍不会在产品中出现。 | event/query/command/surface统一走identity resolver并观测拒绝原因。 |
| PLH-P1-019 | Partial | registry close/destroy transaction已有，App Destroyed只做Runtime event dispatch。 | Destroyed先进入Closing、撤销binding/lease/native map，再发布Destroyed receipt。 |
| PLH-P1-020 | Open | 没有secondary/tool/popup/child window生产创建协议。 | `WindowCreateOperation`声明 kind、owner、parent、viewport、display placement和policy。 |
| PLH-P1-021 | Partial | primary/last-close模型存在，App的`OnAllClosed`仍是primary-close helper。 | primary role、last-window事实和exit policy分离，基于registry snapshot。 |
| PLH-P1-022 | Partial | parent graph和cycle检查存在，没有native owner/child shutdown接线。 | transient/modal/always-on-top/owner关系进入无环图，owner teardown按拓扑执行。 |

### 6.3 Descriptor、Requested/Observed State 与 Window Command

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-023 | Partial | WindowStateRegistry已拆状态，App仍把`WindowDescriptor`当create和动态事实的混合输入。 | create spec、requested state、observed snapshot、effective receipt四种类型硬切。 |
| PLH-P1-024 | Partial | observed focus字段存在，descriptor的focused仍可序列化且与事件authority分离。 | focus只存在observed state；创建请求使用initial activation policy。 |
| PLH-P1-025 | Open | descriptor `present_mode`没有进入Graphics选择，surface固定FIFO。 | requested present mode进入lease negotiation并返回 effective/fallback reason。 |
| PLH-P1-026 | Open | ABI和Graphics会把零尺寸clamp为1，非法值没有产品诊断。 | typed validator；只有显式Sanitize policy可修正并产生 receipt。 |
| PLH-P1-027 | Partial | effective constraints结构存在，没有backend readback和command终态。 | Applied/Rejected receipt带effective constraints和platform reason。 |
| PLH-P1-028 | Open | exact fullscreen缺少video mode时自动退borderless。 | Exact失败；AllowFallback才协商，并记录requested/effective/reason。 |
| PLH-P1-029 | Open | 指定monitor找不到时静默选其他monitor。 | strict placement返回MonitorUnavailable；fallback必须显式。 |
| PLH-P1-030 | Partial | `Current`在typed model中被限制，旧App descriptor仍允许create期Current。 | create只接受 Primary/StableId/Point/Automatic；Current只查询已有window。 |
| PLH-P1-031 | Open | 居中使用`current_video_mode().size()`，没有usable work/safe area。 | 按topology generation的logical usable/safe rect和DPI布局。 |
| PLH-P1-032 | Partial | WindowCommand含target/deadline/desired state，但App没有提交标题/尺寸/位置/模式等运行期命令。 | 所有OS window mutation统一经command broker。 |
| PLH-P1-033 | Partial | broker有Accepted/Applied/Rejected/Canceled/Failed模型，native executor和App receipt消费缺失。 | executor在platform thread执行并发布exact effective state、error和terminal。 |
| PLH-P1-034 | Partial | reconciliation state存在，没有OS move/resize/minimize/fullscreen/DPI producer。 | observed generation驱动接受、纠偏或冲突报告，禁止静默覆盖requested。 |

### 6.4 Display、Monitor 与 Output State

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-035 | Open | App持久化`WindowMonitorSelection::Index(usize)`，热插拔会漂移。 | stable DisplayId加EDID/connector/profile hint和迁移策略。 |
| PLH-P1-036 | Partial | immutable DisplayTopologySnapshot和index已有，没有OS inventory producer。 | Platform Host维护并原子发布真实topology snapshot。 |
| PLH-P1-037 | Partial | DisplayId/topology generation类型已有，但surface/window没有跨边界携带。 | 区分physical output、logical screen、render output，并贯穿lease。 |
| PLH-P1-038 | Open | 没有monitor add/remove/mode-change/hotplug事件生产者。 | backend生成added/changed/removed diff及失效原因。 |
| PLH-P1-039 | Partial | snapshot字段覆盖usable rect、DPI、scale、refresh、orientation、HDR/VRR/color space，App只使用局部winit数据。 | observed capability bits和Unknown状态由真实output probe填充。 |
| PLH-P1-040 | Partial | WindowObservedState有DisplayId/topology generation，App没有更新所属display。 | 窗口跨显示器迁移、DPI、mode change都更新observed generation并通知订阅者。 |
| PLH-P1-041 | Open | ScaleFactorChanged只派发事件并调surface，不更新权威snapshot/UI scale。 | DPI事务同时更新logical/physical geometry、UI scale和surface extent。 |
| PLH-P1-042 | Partial | snapshot replacement/订阅模型存在，没有hotplug broadcast owner。 | 原子topology replacement，所有受影响window/lease收到有序通知。 |
| PLH-P1-043 | Partial | SurfaceLease已有output/topology identity字段，没有format/present/color negotiation和调用者。 | lease记录output、format、present mode、alpha/color space、display generation。 |
| PLH-P1-044 | Open | 没有dynamic resolution/HDR available/requested/active/effective状态。 | per-output observed output state和mode-change receipt，不能从compile flag推断active。 |

### 6.5 Event Loop 与 Application Lifecycle

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-045 | Partial | App已实现`resumed`和私有生命周期机，Runtime ApplicationLifecycleService不接收它。 | lifecycle operation发布WillResume/Running/WillSuspend/Suspended并带generation。 |
| PLH-P1-046 | Partial | `destroy_surfaces`已能在window存在时解绑，未撤销Platform lease或等待统一receipt。 | surface destroy operation覆盖所有window/viewport lease并等待in-flight。 |
| PLH-P1-047 | Open | 没有memory warning/pressure生产事件。 | memory pressure进入Runtime预算收缩和bounded purge operation。 |
| PLH-P1-048 | Partial | App已有`exiting`，Host/Registry admission、命令取消和terminal收集未接通。 | exit关闭admission、清registry、取消命令并等待唯一terminal receipts。 |
| PLH-P1-049 | Partial | Scheduler有wake source类型，App没有new_events/StartCause/ProxyWake接入。 | timer/poll/resume/wait-cancel/proxy wake统一进入scheduler evidence。 |
| PLH-P1-050 | Open | focus仍直接映射FOREGROUND/BACKGROUND。 | window focus、application activation、visibility、occlusion、suspend独立建模。 |
| PLH-P1-051 | Partial | App occlusion只改cadence；WindowObservedState已有visibility字段。 | 每窗发布visible/minimized/occluded，render/tick policy只读取snapshot。 |
| PLH-P1-052 | Open | mobile只有cadence枚举，没有Android/iOS真实surface/app lifecycle backend。 | mobile走resume/suspend/surface create/destroy同一状态机。 |
| PLH-P1-053 | Partial | App私有机有幂等判断，不能识别native/surface generation已失效。 | transition依据window/surface generation和backend state，不依据`window.is_some()`。 |
| PLH-P1-054 | Partial | App close/destroy局部有序，Runtime close tree/backend loss没有共同transaction。 | close/destroy/backend-loss统一幂等destroy transaction。 |
| PLH-P1-055 | Partial | `resumed`/`can_create_surfaces`已有局部single-flight，未产生共享CAS generation/receipt。 | lifecycle operation使用跨层CAS和唯一terminal。 |
| PLH-P1-056 | Partial | App cadence有deadline/coalescing/四种mode；Runtime scheduler另有source/lateness/backlog测试，二者双 authority。 | 单一多source/per-window scheduler，输出可审计deadline、lateness、backlog、starvation与policy decision。 |

### 6.6 Surface Binding、Host Command、Export 与资格测试

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-057 | Open | native surface target仍只实现Win32，其他平台返回Unavailable/degraded；没有shipping backend资格。 | 每个shipping平台实现qualified backend，缺失时fail-close。 |
| PLH-P1-058 | Partial | CPU presenter已显式opt-in并有frames/copy/drop/latency metrics，但不是Platform capability和多输出合同。 | presenter作为明确Degraded provider，受预算、target generation和receipt约束。 |
| PLH-P1-059 | Partial | Graphics replacement transaction和WGPU settlement真实存在，未与SurfaceLease prepare/publish/retire相连。 | 跨层执行prepare -> stop/settle -> atomic publish -> retire，旧generation拒绝present。 |
| PLH-P1-060 | Open | bind ABI仍是viewport/size/raw Win32 target，没有qualified lease。 | ABI改为opaque generation-qualified lease，携window/display/surface identity和终态。 |
| PLH-P1-061 | Partial | host request有IME、rumble、cursor、clipboard、UiAction/UiHost，缺WindowCommand、URL、file dialog等统一操作。 | versioned HostCommand page按target/capability路由并返回receipt。 |
| PLH-P1-062 | Partial | IME可选viewport target，cursor无target；大多数副作用没有deadline/cancel/ack。 | 所有OS副作用绑定target generation、principal、deadline、cancel和terminal result。 |
| PLH-P1-063 | Inherited | generated export callback仍缺opaque instance/lifecycle/window/display/surface合同。该项由Tooling03的`TOOL-EXPORT-P0-005`拥有。 | Runtime提供真实instance和fail-close callback API；本文不重复计数。 |
| PLH-P1-064 | Partial | 467个focused test marker和source guard覆盖类型/局部事务，但没有真实EventLoop、多窗、Destroyed、DPI/hotplug、suspend/resume、lease或跨平台产品矩阵。 | deterministic backend harness，加Windows和至少一个移动平台的OS callback、fault、recreate、soak资格。 |

## 7. P2 完整产品能力

| ID | Status | 能力 | 前置条件 |
|---|---|---|---|
| PLH-P2-001 | Open | Editor工具窗、游戏子窗、popup、modal和transient owner关系 | Window Registry/parent graph |
| PLH-P2-002 | Open | 跨显示器placement profile、拓扑迁移和恢复 | stable DisplayId/usable rect |
| PLH-P2-003 | Open | HDR、wide gamut、VRR、refresh、color-space选择 | observed output/surface negotiation |
| PLH-P2-004 | Open | exclusive fullscreen chooser、安全回滚和失败倒计时 | exact mode command/receipt |
| PLH-P2-005 | Open | per-monitor DPI、safe area、orientation、折叠屏布局 | topology/DPI transaction |
| PLH-P2-006 | Open | 虚拟桌面、远程桌面、display reconnect恢复 | topology generation/migration |
| PLH-P2-007 | Open | 无边框、透明、click-through、always-on-top、窗口形状 | backend capability/permission |
| PLH-P2-008 | Open | accessibility title/role/state和辅助技术通知 | stable identity/observed state |
| PLH-P2-009 | Open | clipboard、URL、drag/drop、file picker统一operation | HostCommandBroker/principal/receipt |
| PLH-P2-010 | Open | kiosk、display wall、presentation、多输出同步 | multi-output ownership/frame pacing |
| PLH-P2-011 | Open | headless、virtual display、remote stream、automation backend | backend qualification/surface abstraction |
| PLH-P2-012 | Open | power、thermal、battery、background execution policy | lifecycle/resource budget |
| PLH-P2-013 | Open | high-refresh、VRR与multi-window frame pacing | per-output scheduler |
| PLH-P2-014 | Open | window/display/surface/lifecycle timeline和诊断UI | bounded evidence journal |
| PLH-P2-015 | Open | hotplug/DPI/destroy/suspend/OOM/backend restart fault/soak | unified teardown/correctness gates |
| PLH-P2-016 | Open | 同协议窗口延迟、resize、present、恢复benchmark | 先冻结功能、硬件、OS、输出、统计协议 |

P2不能替代P0/P1。HDR或高刷演示没有意义，除非先能证明 owner、window/display/surface generation、lifecycle terminal 和真实平台能力。

## 8. 五引擎参考对照

| 参考 | 本地证据 | 对 Zircon 的工程约束 | 不照搬 |
|---|---|---|---|
| Bevy | `WinitWindows`维护`WindowId -> wrapper`、engine entity -> native ID、native ID -> entity；remove同步清理。`ApplicationHandler`的resumed/suspended/about_to_wait/window_event都经过状态 owner，unknown native ID会被拒绝。 | 所有事件必须先解析native ID；多窗、suspend、surface重建由同一状态 owner驱动。 | 不把ECS Entity当跨ABI持久ID，Zircon使用registry/slot/generation。 |
| Fyrox | `Executor`在`Event::Resumed`初始化graphics context，在`Suspended`销毁context；close/resize/redraw在executor分支中形成明确生命周期。 | 即使先支持单窗，graphics/surface与application lifecycle也必须同一条可证明链。 | 不把executor的单循环直接当作多window架构。 |
| Godot | `DisplayServer`是window、screen metrics、clipboard、subwindow、native handle和window callbacks的抽象 owner；Windows backend在删除子窗时先撤销render context再DestroyWindow；HDR区分supported/requested/enabled。 | Platform Host必须是实例化owner，WindowId进入每个命令/callback，output能力分available/requested/effective。 | 不复制全局Singleton；按Runtime/Host实例隔离。 |
| Unreal | `GenericApplication`提供PumpMessages/ProcessDeferredEvents/MakeWindow/InitializeWindow；`WindowsApplication`维护窗口数组并按HWND处理destroy/display/DPI；`SlateRHIRenderer`维护SWindow到viewport map、HDR pixel format和resize/present。 | native消息、window registry、display metrics、render viewport和destroy必须是相互可追踪的事务。 | 不复制Slate宏和类层级，只吸收owner、identity、effective state和receipt。 |
| Unity Graphics | `DynamicResolutionHandler`按camera/pipeline追踪requested/resolved scale；HDR debug暴露gamut、format、paper white、luminance和mode change requested。Graphics本身不拥有OS loop。 | output negotiation与render observability分开，并明确available/requested/active/effective；Platform owner不能从Graphics compile flag推断。 | 不把Unity Graphics仓库误当完整Platform实现。 |

## 9. 目标架构与硬切边界

```text
OS backend / winit / mobile export host
  -> PlatformHostService(instance, thread affinity, health, generation)
       -> ApplicationLifecycleService
       -> WindowRegistry <-> DisplayTopologySnapshot
       -> HostCommandBroker
       -> EventLoopScheduler
       -> SurfaceLeaseRegistry
            -> Runtime viewport binding
            -> Graphics surface generation

native event
  -> resolve NativeWindowId / DisplayId
  -> qualified event { owner, slot, generation, sequence, observed state }
  -> Runtime / Editor / gameplay

window or surface command
  -> capability + target generation + deadline validation
  -> platform-thread executor
  -> observed/effective snapshot
  -> exactly one terminal receipt
```

必须硬切删除或禁止：

- compile feature/target 直接发布 runtime Ready；
- App的伪`PrimaryWindowHandle`、固定viewport 1和丢弃winit WindowId；
- PlatformManager被PreferenceStorage类型擦除后继续假装是window manager；
- descriptor里的持久monitor index、focused事实和隐式Current；
- exact fullscreen/monitor自动静默fallback；
- focus冒充application foreground/background；
- Destroyed、suspended、backend loss各自拥有不同teardown；
- raw Win32 surface bind越过window/display/surface generation；
- shipping路径自动CPU fallback；
- 无target/deadline/ack/cancel的cursor/IME/clipboard/URL等OS副作用。

## 10. 重构里程碑

### M191.0 · Truth Freeze 与接线红测

- 为所有Runtime platform operation增加“production caller或明确未接线”测试；
- 固定App、PlatformDriver、Manager、dynamic ABI、Graphics surface的owner图；
- 红测丢弃native ID、fixed viewport、disabled->Ready、raw lease和Destroyed/suspend交叉路径。

### M191.1 · Platform Host Product Cutover

- 实现每个目标平台的`PlatformHostBackend` instance、thread affinity、health和terminal；
- 用typed PlatformManager service替换PreferenceStorage wrapper；
- App启动、退出、resumed/suspended全部通过Host/Lifecycle service，不再维护第二套 authority。

### M191.2 · Window Registry 与 Event Routing

- 把winit native ID接入双向 registry；
- 将多窗口 create/close/parent/primary/viewport mapping接入真实产品路径；
- 所有unknown/stale/closing事件形成有界诊断和terminal receipt。

### M191.3 · Display Topology 与 Window Command

- 接入真实monitor inventory、hotplug、DPI、usable/safe rect、mode和output capability；
- 取消持久化index和隐式fallback；
- 将resize/move/fullscreen/title/visibility/focus等统一为deadline command并回读effective state。

### M191.4 · Surface Lease 与 ABI Hard Cutover

- 设计vNext opaque generation-qualified lease，覆盖Win32并为Android/iOS/web提供明确Unavailable/qualified实现；
- 将prepare/publish/retire与Graphics/WGPU settlement连接；
- 将format/present/alpha/color/HDR/VRR negotiation和CPU degraded receipt写入surface结果。

### M191.5 · Scheduler、Qualification 与 Observability

- App cadence与Runtime EventLoopScheduler合并，接入native wake reason和per-window budget；
- 建立deterministic host、Windows、多显示器、DPI/hotplug、Destroyed、surface-loss、suspend/resume、OOM/backend restart fault harness；
- correctness/soak通过后，再按同协议冻结延迟、frame pacing、CPU/RSS和恢复benchmark。

## 11. 验收矩阵

| Gate | Status | 验收内容 |
|---|---|---|
| PLH-G01 | Partial | disabled capability不能Ready；当前projection已改进但无真实owner接线 |
| PLH-G02 | Partial | Compiled/Selected/Installed/Initialized/Observed/Ready与owner/generation/evidence分层 |
| PLH-G03 | Partial | planning matrix不能作为shipping readiness |
| PLH-G04 | Partial | backend failure/exit/degraded撤销Ready |
| PLH-G05 | Partial | native WindowId双向resolve、unknown/stale reject；registry有模型，App未接 |
| PLH-G06 | Partial | slot reuse generation；局部测试有，产品重建未接 |
| PLH-G07 | Partial | primary/last close/tool close/app exit独立；App仍单窗 |
| PLH-G08 | Fail | 两窗口必须路由到两个viewport |
| PLH-G09 | Partial | parent graph无cycle和拓扑teardown模型存在，native owner未接 |
| PLH-G10 | Fail | 持久配置不能保存live WindowId/monitor index |
| PLH-G11 | Partial | create/requested/observed/effective独立 |
| PLH-G12 | Partial | invalid/nonfinite resolution产生诊断而非静默clamp |
| PLH-G13 | Fail | exact fullscreen/monitor失败不能隐式fallback |
| PLH-G14 | Partial | command target generation/request/deadline/terminal模型存在但无executor |
| PLH-G15 | Fail | OS move/resize/minimize/fullscreen/DPI必须回写observed/reconcile |
| PLH-G16 | Partial | topology有DisplayId/generation/geometry/usable/DPI字段，无producer |
| PLH-G17 | Partial | HDR/VRR/color/orientation/safe-area capability字段有，未完成output probe |
| PLH-G18 | Fail | hotplug diff、失效、迁移未实现 |
| PLH-G19 | Fail | monitor/DPI更新没有权威geometry/UI scale/surface事务 |
| PLH-G20 | Fail | output available/requested/active/effective trace未贯通 |
| PLH-G21 | Partial | focus/activation/visibility/occlusion/suspend字段模型有，App仍混用 |
| PLH-G22 | Partial | suspend stop-submit/quiesce/fence/unbind/drop顺序局部存在，未统一lease |
| PLH-G23 | Partial | resume重建surface局部存在，无window/surface generation receipt |
| PLH-G24 | Partial | CloseRequested/Destroyed/backend-loss统一destroy模型存在，App未调用 |
| PLH-G25 | Partial | resumed/can_create_surfaces局部single-flight，无共享CAS |
| PLH-G26 | Fail | memory warning进入预算purge未实现 |
| PLH-G27 | Partial | exiting已有，registry清理/command cancel/receipt收敛缺失 |
| PLH-G28 | Partial | scheduler source/deadline/lateness/backlog/starvation有单测，无native StartCause |
| PLH-G29 | Partial | raw handle局部有序销毁，但没有qualified lease和cross-platform fault test |
| PLH-G30 | Partial | replacement/fence/publish/retire在Graphics有，Platform lease未接 |
| PLH-G31 | Fail | ABI没有qualified window/output/surface generation |
| PLH-G32 | Fail | Win32以外没有真实平台qualification |
| PLH-G33 | Partial | CPU presenter显式degraded且有metrics，缺host capability/预算合同 |
| PLH-G34 | Partial | clipboard/IME等局部存在，统一target/deadline/ack缺失 |
| PLH-G35 | Fail | export host没有真实opaque instance/callback lifecycle |
| PLH-G36 | Fail | generated callback payload和typed terminal缺失 |
| PLH-G37 | Partial | deterministic/unit/source guards较多，尚无真实host harness |
| PLH-G38 | Fail | Windows+mobile Destroyed/DPI/hotplug/suspend/resume/recreate未资格化 |
| PLH-G39 | Fail | correctness/fault/soak先于性能的证据链未建立 |
| PLH-G40 | Fail | 同协议同硬件比较前置条件未冻结，不能宣称超过Unreal |

## 12. 状态与交付记录

- 本报告是 Runtime191 当前工作树 review；继承并重判 Runtime57/116 的 Platform Host ledger，不删除旧报告。
- Runtime90继续拥有RHI device/surface/present/device-loss canonical contract；Runtime103拥有clock/time policy；Runtime157拥有Core lifecycle；App01拥有bootstrap/event-loop integration；Editor179拥有Editor viewport host消费边界。本文只负责Runtime Platform Host、Window/Display identity、application lifecycle、surface lease和App接线要求。
- 仅写入本报告、Runtime index、根 optimize index 和 coverage 台账；没有生产代码、ABI、Cargo、测试或参考源码变更。
- review-only 验证：执行文档路径/编号/链接/格式检查与`git diff --check`；未运行 Cargo、GPU、OS、PIE、mobile、fault、soak、benchmark；未查询、轮询、等待或实时跟踪协调器。
