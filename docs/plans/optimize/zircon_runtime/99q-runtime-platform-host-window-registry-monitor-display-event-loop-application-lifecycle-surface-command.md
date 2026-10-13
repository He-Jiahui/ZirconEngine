---
title: Runtime Platform Host、Window Registry、Monitor、Display、Event Loop、Application Lifecycle、Surface Command 与 Product Integration 当前源码复核
category: zircon_runtime
report_id: Runtime116
review_date: 2026-08-23
baseline_head: 4cc19615076c76c45f1fcdd587563fe5274ad8fd
baseline_epoch: 362
related_code:
  - zircon_runtime/src/core/framework/window
  - zircon_runtime/src/core/framework/platform
  - zircon_runtime/src/platform
  - zircon_app/src/entry/runtime_entry_app
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session/session.rs
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files
tests:
  - zircon_runtime/src/core/framework/window/tests.rs
  - zircon_runtime/src/platform/tests
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/dynamic_api/tests/viewport.rs
  - zircon_runtime/src/dynamic_api/session/tests/runtime_ui_surface.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
  - zircon_runtime/src/graphics/tests/surface_targets.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/abi_safety_contracts.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards
  - zircon_app/src/entry/tests/runtime_entry_surface_present_guards
  - zircon_app/src/entry/tests/runtime_entry_window_lifecycle_guards
plan_sources:
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/46-engine-module-service-contract-context-factory-descriptor-snapshot-composition-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/50-runtime-manager-resolver-named-service-handle-generation-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md
  - docs/plans/optimize/zircon_app/01-product-host-bootstrap-loop-dynamic-runtime-shutdown-review.md
  - docs/plans/optimize/zircon_tooling/03-export-preset-build-cook-pack-platform-bundle-release-review.md
reference_engines:
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_winit/src/winit_monitors.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/system.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
  - dev/godot/servers/display/display_server.h
  - dev/godot/servers/display/display_server.cpp
  - dev/godot/servers/display/display_server_enums.h
  - dev/godot/core/os/main_loop.h
  - dev/godot/core/os/main_loop.cpp
  - dev/godot/platform/windows/display_server_windows.h
  - dev/godot/platform/windows/display_server_windows.cpp
  - dev/godot/platform/android/display_server_android.cpp
  - dev/godot/platform/web/display_server_web.cpp
  - dev/godot/tests/scene/test_window.cpp
  - dev/godot/tests/display_server_mock.cpp
  - dev/godot/tests/display_server_mock.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericWindow.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericWindowDefinition.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/Windows/WindowsApplication.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/DynamicResolutionHandler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/HDROutputUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettingsHDROutput.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 99q · Runtime Platform Host 当前源码复核

## 1. 当前结论

Runtime57 的主体结论在当前源码上仍成立。Zircon 已有可保留的单窗口产品底座：`WindowDescriptor`、winit `ApplicationHandler`、Win32 raw-handle surface、runtime viewport bind/unbind、softbuffer CPU fallback、IME/cursor/gamepad host request，以及按Game/Desktop/Mobile/Headless切换的frame cadence都是真实实现。它们证明了最小产品链，而不是工程级Platform Host已经完成。

当前最大问题仍是“声明、owner、identity、lifecycle与surface lifetime”彼此断开。`PlatformModule`描述自己拥有windowing与OS integration，真实`PlatformDriver`/`PlatformManager`却只拥有PreferenceStorage；`PlatformConfig.enabled=false`不参与capability计算，任意构造的target与feature selection仍能把window、monitor、event loop、application lifecycle和metrics报告为`Supported`。App继续只有一个`Option<Arc<dyn Window>>`、固定viewport 1和固定session，`window_event`丢弃winit `WindowId`，`device_event`丢弃`DeviceId`，不存在Window Registry、Display Topology owner、qualified generation或命令回执。

两项本地P0均未关闭。第一，Platform capability仍会把未安装、未初始化、未观测且甚至被配置禁用的后端能力发布为Supported。第二，Graphics以`create_surface_unsafe`保存raw native handle，安全注释要求surface先于window销毁；但`WindowEvent::Destroyed`仍只向Runtime分发事件，不解绑surface、不清window/presenter，也没有`suspended`、`destroy_surfaces`、`memory_warning`或`exiting`实现。CloseRequested的显式路径会解绑，OS Destroyed和移动端surface lifecycle却没有汇入同一destroy transaction。

本轮账本为 **2项P0 Open、63项P1 Open、1项P1 Partial、16项P2 Open、40项Gate Fail**。唯一Partial是`PLH-P1-056`：当前`RuntimeFrameCadence`已经有runtime deadline、request replacement/coalescing、reactive/continuous/low-power/fixed模式、失焦/遮挡降频和汇总计数；但仍没有wake-source identity、deadline clock domain、lateness/backlog/starvation、per-window调度或application lifecycle generation，因此不能关闭。Runtime57之后的直接产品改动只把失焦Game cadence从约60Hz降到10Hz，并在Continuous pump时清掉已合并请求位；这是局部功耗与记账修正，不是Platform Host收敛。

generated browser/Android/iOS callback继续忽略lifecycle、touch、keyboard、metrics payload并恒定返回true，唯一P0 owner仍是Tooling03的`TOOL-EXPORT-P0-005`，本文不重复计数。用户已明确暂缓tooling优化，因此本文只要求Runtime侧提供真实instance/lifecycle/window/display/surface接收合同，不规划现有脚本和工具链重写。

本轮只做review与计划记录，没有修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、真实多窗口/多显示器、DPI/hotplug、suspend/resume、surface-loss、移动端、fault injection、soak、profiler或同协议跨引擎benchmark。当前没有证据可以宣称Platform Host的稳定性、表现或性能达到，更不能宣称超过当前Unreal。

## 2. 当前源码冻结与可复现性

| 范围 | 文件 / 行 / 非空行 / bytes / `#[test]` / dirty | fingerprint / 选择规则 |
|---|---:|---|
| Window与Platform production | **34 / 2,794 / 2,524 / 96,162 / 0 / 0** | `751e8042a97c7001a04700c3d910e6f40a6c62b12e218f283793459890b857f3`；framework window/platform与Platform production，排除Preferences与tests |
| App product host完整目录 | **79 / 6,266 / 5,702 / 228,880 / 98 / 0** | `aa39192ae7c8cbdd59a4d97ee0513a019cb4531f6e1e3320bef31efc07390be8`；`runtime_entry_app`完整目录 |
| Dynamic ABI、surface与export integration | **24 / 7,913 / 7,356 / 320,284 / 25 / 1** | `90019e1fd322df13bcbca2689d8f2b9c7a31491ef828e9c2443b14da21f75b91`；Runtime Interface六文件、dynamic session/surface、Graphics surface、export host与App runtime library |
| focused direct/source-shape tests | **59 / 11,400 / 10,645 / 425,503 / 221 / 4** | `70c6eacc5047d99d920fbfa7d2602c027f819188710541e16e3cdc342bb83da3`；window/platform/dynamic/interface/App surface与lifecycle tests |
| 五引擎显式参考与focused test support | **27 / 27,316 / 23,410 / 1,053,969 / 6 / 0** | `efe87bef0b102c97606a7070f7946a2462c936e51506fd5c758046bf9bd3e04f`；Bevy/Fyrox/Godot/Unreal/Unity Graphics及Bevy inline tests、Godot Windows backend、Window/DisplayServerMock |

fingerprint算法为：仓库相对路径转`/`并排序去重；每个文件计算lowercase SHA-256；以`path|hash`按LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。行数为物理行，非空行为trim后非空；test只统计Rust的`#[test]`与`#[tokio::test]`，因此Godot C++ `TEST_CASE`不进入该数字。各组按不同审查目的允许重叠，不能把组内数字直接相加冒充去重总量。

基线HEAD为`4cc19615076c76c45f1fcdd587563fe5274ad8fd`，coordinator epoch为362。报告读取当前共享working tree；integration组的`zircon_runtime/src/dynamic_api/session/ffi.rs`，focused tests组的`runtime_ui_surface.rs`、`session_lifecycle.rs`、Platform Preferences与Runtime Interface contracts含其他Session未提交改动，本轮只读取当前结果，不接管、不回退。Preferences细节继续由Runtime45/Frameworks05拥有。`source_recheck_required`保持true。

## 3. Runtime57 后的真实变化

| 变化 | 当前证据 | 账本结论 |
|---|---|---|
| Window/Platform production未变化 | 34文件、行数、bytes与fingerprint和Runtime57完全一致 | 2项P0及window/display/capability主体finding继续Open |
| App Host只有cadence小修 | 79文件、行数、bytes与旧fingerprint一致；相对旧基线只有失焦10Hz与Continuous清request位差异 | `PLH-P1-056`按当前完整cadence底座重判Partial；不改变lifecycle/window/surface状态 |
| Dynamic session有并发改动但surface ABI未升级 | 当前working tree的session registry/world-sync等由其他owner修改；bind ABI仍只有viewport/size/raw target，FFI仍只接受`DEFAULT_VIEWPORT` | `PLH-P1-060`及两项P0不变；不把无关dynamic progress归功于本篇 |
| Host request仍无命令终态 | IME只有可选`target_viewport`；cursor没有target；请求没有统一request ID、deadline、cancel、ack或terminal result | `PLH-P1-061/062`继续Open |
| 测试数量增加但资格层未形成 | focused组比旧报告多45行、3个Rust test attribute；仍没有真实multi-window、OS Destroyed、DPI/hotplug、suspend/resume或surface lease harness | `PLH-P1-064`与G37-G40继续Fail |

## 4. 当前真实产品链

```text
PlatformConfig { enabled, target, features }
  -> PlatformManager::capability_report
       -> target enum + compile feature selection
       -> Supported / FeatureDisabled / Unavailable
       -> no installed backend/window/display evidence

winit ApplicationHandler
  resumed/can_create_surfaces
    -> create one Option<Window>
    -> fixed viewport 1
    -> Win32 raw handles -> bind runtime surface
    -> otherwise implicit CPU presenter fallback

  window_event(_window_id, event)
    -> discard native WindowId
    -> route every event to viewport 1
    -> CloseRequested: unbind -> drop presenter/window -> optional exit
    -> Destroyed: dispatch event only

runtime surface ABI
  bind(session, viewport=1, size, raw Win32 handles)
  unbind(session, viewport=1)
  -> no window/output/surface generation
  -> no prepare/fence/publish/retire transaction
```

`WindowLifecyclePolicy::OnAllClosed`当前仍调用`should_exit_after_primary_close()`，没有registry就无法证明“全部窗口关闭”。`PrimaryWindowHandle(u64)`默认0、可serde且没有owner/generation，产品只检查`Option::is_some`。`WindowMonitorSelection::Index(usize)`可持久化；创建时临时枚举monitor，找不到指定monitor/video mode时回退automatic/borderless，resolution与constraints又静默clamp。requested、observed与effective state没有分离。

Graphics `ViewportSurface`处理Lost/Outdated时直接reconfigure并返回，Timeout/Occluded直接跳帧；surface negotiation自动选format/present/alpha，但没有把requested/effective/fallback reason、display identity或generation返回Platform Host。resize会在同一viewport再次bind，render framework以新surface覆盖record；没有旧surface in-flight fence或显式retire receipt。

## 5. 可保留底座

- 保留`WindowDescriptor`的字段覆盖面，但拆成create spec、requested state、observed snapshot与effective receipt。
- 保留winit `ApplicationHandler`和现有事件转换，但所有事件必须先经过native ID到qualified WindowId解析。
- 保留runtime viewport bind/unbind与Graphics surface abstraction，但ABI改为opaque、generation-qualified `SurfaceLease`。
- 保留frame cadence的deadline replacement、coalescing、low-power policy与汇总计数，升级为可审计的多source/per-window scheduler。
- 保留Platform planning matrix用于离线profile/BuildSet推演，但使用不同类型，禁止冒充当前runtime readiness。
- 保留CPU presenter作为显式`ReferenceCpuPresenter`，只能opt-in并发布Degraded、copy bytes、latency与drop。

## 6. Owner边界与继承阻断

- Runtime57/116唯一拥有Platform Host纵向组合、Window/Display registry、Application lifecycle、surface lease、window command与资格矩阵；Runtime116刷新状态，不复制编号。
- Runtime06继续拥有Platform/Input/Process首轮广度；Runtime56拥有input device/action/replay；本文只拥有window/device target identity进入platform routing的部分。
- App01拥有产品bootstrap、event loop、dynamic runtime与shutdown；本文定义App必须满足的Platform Host与surface lifetime合同。
- Runtime09A拥有RHI surface/present/device generation、format/HDR协商；本文提供window/output/surface lease和lifecycle边界。
- Runtime24、Runtime43与Runtime Interface01/05/07拥有通用identity、dynamic session与foreign ABI规则；本文落地具体WindowId/DisplayId/SurfaceLease/HostCommand schema。
- Runtime42/46/50拥有composition、module/service lifecycle与capability truth；Platform只有真实backend owner与observed evidence时才能Ready。
- Runtime45/Frameworks05拥有Preferences；Tooling03唯一拥有`TOOL-EXPORT-P0-005`，本文不重复计数。

## 7. 本地P0阻断项

| ID | Status | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PLH-P0-001 | Open | `PlatformConfig.enabled`仍未进入`capability_report()`；window/monitor/event loop/lifecycle/metrics依旧只靠target和feature selection报Supported，真实Platform owner只有Preferences。 | `PlatformActivationPlan + observed CapabilityState`；Compiled/Selected/Installed/Initialized/Observed/Ready分层，能力携provider instance、generation、evidence和typed failure；disabled不得创建owner。 |
| PLH-P0-002 | Open | Graphics以`create_surface_unsafe`持raw handle；CloseRequested会unbind，但OS `Destroyed`只dispatch，ApplicationHandler仍没有suspended/destroy_surfaces，window与surface可越过native lifetime。 | `SurfaceLease { window_id, window_generation, output_generation, surface_generation }`；Destroyed/Suspended/DestroySurfaces先stop submit、quiesce/fence、unbind/drop，再释放native window，并产出terminal receipt。 |

继承发布阻断`TOOL-EXPORT-P0-005`保持Open：generated mobile/browser callbacks仍无Runtime instance并恒真。它关闭前，移动端与browser Platform capability不得发布Ready。

## 8. P1工程化差距总账

### 8.1 Authority、Capability与Backend Readiness

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-001 | Open | `PlatformModule`宣称windowing/OS integration，真实实现只装配PreferenceStorage。 | descriptor按真实已安装service发布能力，Window Host由backend service拥有。 |
| PLH-P1-002 | Open | `PlatformDriver`没有EventLoop、Window、Monitor、Display、clipboard、URL或power owner。 | Platform backend trait定义线程亲和、启动、health、quiesce与teardown。 |
| PLH-P1-003 | Open | `PlatformManager`没有Window Registry或命令/query接口。 | Manager只发布generation snapshot与operation handle，不暴露临时native对象。 |
| PLH-P1-004 | Open | `PlatformTarget`可任意构造，不证明当前binary/OS/arch/backend。 | BuildSet冻结compiled target；runtime只接受兼容observed target，planning查询使用独立类型。 |
| PLH-P1-005 | Open | compile feature selection仍被直接解释为runtime backend可用。 | 区分Compiled、Selected、Installed、Initialized、Observed、Ready。 |
| PLH-P1-006 | Open | capability无provider instance、owner session或generation。 | 每条能力绑定qualified owner/generation，owner退出自动失效。 |
| PLH-P1-007 | Open | `CapabilityStatus`仍只有Supported/FeatureDisabled/Unavailable。 | 加Starting、Ready、Degraded、Failed与typed reason；Supported只用于静态catalog。 |
| PLH-P1-008 | Open | diagnostic缺probe时间、backend版本、设备/显示证据和currentness。 | bounded evidence record；拓扑/owner变化使旧证据stale。 |
| PLH-P1-009 | Open | tests继续用人工cross-target feature matrix认证Supported，不实例化backend。 | 分离planning tests与真实backend qualification，禁止命名和断言互相替代。 |
| PLH-P1-010 | Open | Platform owner无health、restart、quiesce、terminal failure或泄漏报告。 | 接入Module lifecycle与Operation receipt；失败立即撤销Ready。 |

### 8.2 Window Identity、Registry与Multi-Window

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-011 | Open | App只保存一个`Option<Arc<dyn Window>>`。 | `WindowRegistry`管理任意数量generation-qualified slot。 |
| PLH-P1-012 | Open | `PrimaryWindowHandle`不参与寻址，只检查`Option`。 | primary是registry中的qualified role handle，命令/event真实解析。 |
| PLH-P1-013 | Open | Primary handle默认0且可serde，无namespace/owner/generation。 | live `WindowId { registry, slot, generation }`不可持久化；配置只保存placement key。 |
| PLH-P1-014 | Open | 所有窗口事件硬路由到viewport 1。 | Window-to-Viewport binding支持多窗、多viewport及无viewport工具窗。 |
| PLH-P1-015 | Open | winit `WindowId`参数仍被丢弃。 | native ID先查双向registry；unknown/stale拒绝并诊断。 |
| PLH-P1-016 | Open | 无engine handle到native ID的双向映射。 | 两索引同generation原子更新，remove同步清理；参考Bevy `WinitWindows`。 |
| PLH-P1-017 | Open | 窗口重建无generation，旧event/command/surface可误命中新对象。 | slot reuse递增generation，stale request fail-close。 |
| PLH-P1-018 | Open | unknown/stale没有typed错误，因为ID从未解析。 | event/query/command/surface统一Unknown/Stale/Closing/Destroyed。 |
| PLH-P1-019 | Open | `Destroyed`不移除对象或发布terminal generation。 | destroy事务先Closing、撤销binding/surface/native映射，再发布Destroyed receipt。 |
| PLH-P1-020 | Open | 无secondary/tool/popup/child window创建销毁协议。 | `WindowCreateOperation`声明kind、owner、parent、viewport与policy并返回handle。 |
| PLH-P1-021 | Open | 无primary选举/替换/失效；OnAllClosed实际仍是primary-close helper。 | primary role与exit policy分离，最后窗口事实来自registry snapshot。 |
| PLH-P1-022 | Open | 无transient/modal/always-on-top/owner shutdown关系。 | registry维护无cycle关系图，owner teardown按拓扑关闭。 |

### 8.3 Descriptor、Requested/Observed State与Window Command

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-023 | Open | `WindowDescriptor`混合create请求与动态外观事实。 | 分离create spec、requested state、observed snapshot、effective policy。 |
| PLH-P1-024 | Open | `focused`可序列化，实际focus事件不维护同一authority。 | focus只属于observed state，不作为下一次创建的无条件请求。 |
| PLH-P1-025 | Open | descriptor `present_mode`不进入Graphics surface选择。 | negotiation消费policy并返回requested/effective/fallback reason。 |
| PLH-P1-026 | Open | 非finite、零或非法resolution被静默改为1/default。 | typed field validator；仅显式Sanitize policy可修正并留receipt。 |
| PLH-P1-027 | Open | constraints没有backend effective-state回读。 | command完成后发布effective constraints，拒绝/clamp可见。 |
| PLH-P1-028 | Open | exact fullscreen mode找不到时静默退borderless。 | Exact失败；AllowFallback才协商并返回mode与原因。 |
| PLH-P1-029 | Open | monitor不存在时静默选其他monitor。 | strict placement返回MonitorUnavailable；fallback显式。 |
| PLH-P1-030 | Open | create期`Current`无current monitor，最后变automatic。 | Current只用于已有window；create使用Primary/StableId/Point/Automatic。 |
| PLH-P1-031 | Open | 居中使用video mode物理尺寸，不使用usable work/safe area。 | 按topology generation的logical usable rect与DPI布局。 |
| PLH-P1-032 | Open | 无运行期标题/尺寸/位置/模式/可见性/层级统一命令。 | `WindowCommand`含target generation、request ID、deadline、desired state。 |
| PLH-P1-033 | Open | 窗口操作无Accepted/Applied/Rejected/Canceled/Failed终态。 | HostCommandBroker发布exact effective state和platform error。 |
| PLH-P1-034 | Open | OS拖动/缩放/最小化/全屏后无requested/observed reconciliation。 | 更新observed generation并按policy接受、纠偏或报告冲突。 |

### 8.4 Monitor、Display与Output State

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-035 | Open | `WindowMonitorSelection::Index(usize)`可持久化，热插拔后漂移。 | stable display key + EDID/connector/profile hint及迁移策略。 |
| PLH-P1-036 | Open | 创建时临时收集`Vec<MonitorHandle>`，无inventory owner。 | Platform Host维护immutable `DisplayTopologySnapshot`。 |
| PLH-P1-037 | Open | monitor/output无稳定ID或generation。 | DisplayId区分physical output、logical screen、render output。 |
| PLH-P1-038 | Open | 无monitor add/remove/mode change/hotplug事件。 | backend重建topology diff并发布added/changed/removed与失效原因。 |
| PLH-P1-039 | Open | 缺usable rect、DPI、scale、refresh、orientation、safe area、color/HDR/VRR统一事实。 | backend observed字段带capability bits；未知值显式Unknown。 |
| PLH-P1-040 | Open | 无window所属monitor和跨monitor迁移事件。 | observed window snapshot带DisplayId与topology generation。 |
| PLH-P1-041 | Open | ScaleFactorChanged只发事件/调surface，不更新权威window/display snapshot。 | DPI事务同时更新logical/physical geometry、UI scale与surface extent。 |
| PLH-P1-042 | Open | display change无metrics rebuild/broadcast owner。 | 原子snapshot replacement并发布订阅通知。 |
| PLH-P1-043 | Open | surface未绑定render output/display identity和协商generation。 | lease记录output、format、present、color space与display generation。 |
| PLH-P1-044 | Open | dynamic resolution/HDR状态不区分available/requested/active/effective。 | 发布per-output observed状态，不从compile flag推断active。 |

### 8.5 Event Loop与Application Lifecycle

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-045 | Open | ApplicationHandler仍无`suspended`。 | `ApplicationLifecycleMachine`发布WillSuspend/Suspended并有序quiesce。 |
| PLH-P1-046 | Open | 无`destroy_surfaces`。 | window仍存在时可撤销全部surface lease并等待in-flight。 |
| PLH-P1-047 | Open | 无`memory_warning`。 | memory pressure进入Runtime预算收缩与bounded purge operation。 |
| PLH-P1-048 | Open | 无`exiting`。 | event loop仍活跃时关闭admission、清registry、收terminal receipts。 |
| PLH-P1-049 | Open | 无`new_events`/StartCause处理。 | timer/poll/resume/wait-cancel/proxy wake原因进入scheduler evidence。 |
| PLH-P1-050 | Open | WindowFocused仍直接映射Application Background/Foreground。 | window focus、application activation、visibility、suspend独立。 |
| PLH-P1-051 | Open | Occluded只改cadence并发事件，无权威visibility snapshot。 | 每窗维护visible/minimized/occluded，再由policy决定render/tick。 |
| PLH-P1-052 | Open | Mobile仍只是cadence枚举，无surface/app lifecycle。 | Android/iOS走真实resume/suspend/surface create/destroy状态机。 |
| PLH-P1-053 | Open | resumed以`window.is_some()`判幂等，不能识别native/surface已失效。 | 依据window/surface generation与backend state transition。 |
| PLH-P1-054 | Open | CloseRequested teardown，Destroyed只发event，两路不收敛。 | close/destroy/backend-loss统一进入幂等destroy transaction。 |
| PLH-P1-055 | Open | resumed与can_create_surfaces无single-flight generation。 | lifecycle operation使用CAS state与唯一terminal receipt。 |
| PLH-P1-056 | Partial | 已有runtime deadline、request coalescing、四类mode、10Hz失焦/1Hz遮挡降频和计数；但没有source identity、clock domain、lateness、backlog、starvation、per-window与lifecycle generation。 | `EventLoopScheduler`融合frame demand、host command、timer、lifecycle与background policy，输出可审计deadline/lateness。 |

### 8.6 Surface Binding、Host Command、Export与Tests

| ID | Status | 当前差距 | 必须重构为 |
|---|---|---|---|
| PLH-P1-057 | Open | native surface target仍只实现Win32，其他平台返回None。 | 每个shipping平台有qualified backend；缺失时Unavailable。 |
| PLH-P1-058 | Open | CPU fallback全帧RGBA capture到softbuffer，无Degraded能力和预算。 | 显式opt-in `ReferenceCpuPresenter`，报告bytes/latency/drop。 |
| PLH-P1-059 | Open | resize在同一viewport再次bind，没有replace transaction。 | prepare新surface -> fence旧提交 -> atomic publish -> retire旧generation。 |
| PLH-P1-060 | Open | bind ABI仍只有viewport、size、raw target。 | 加qualified surface lease或opaque generation handle，stale拒绝。 |
| PLH-P1-061 | Open | host request只有IME、rumble、cursor，无WindowCommand/clipboard/URL/display operation。 | versioned HostCommand page按capability路由。 |
| PLH-P1-062 | Open | cursor无target，IME仅可选viewport；整体无deadline/cancel/ack。 | 所有OS副作用绑定target generation并有唯一terminal result。 |
| PLH-P1-063 | Open | export callback无instance/lifecycle/window/display/surface接收合同。 | 为Tooling03 P0提供真实opaque instance和fail-close API。 |
| PLH-P1-064 | Open | 测试仍以pure matrix、unit与`include_str!`守卫为主，无真实EventLoop、多窗、Destroyed、DPI/hotplug、suspend/resume、surface lease集成。 | deterministic backend harness及Windows+至少一个移动平台产品资格矩阵。 |

## 9. P2完整产品能力

| ID | Status | 能力 | 前置条件 |
|---|---|---|---|
| PLH-P2-001 | Open | 多窗口Editor工具窗、游戏子窗、popup、modal体验 | Window Registry与关系图 |
| PLH-P2-002 | Open | 跨显示器placement profile与拓扑迁移 | stable DisplayId与usable rect |
| PLH-P2-003 | Open | HDR、wide gamut、VRR、refresh、color-space选择 | observed output与surface negotiation |
| PLH-P2-004 | Open | exclusive fullscreen chooser与安全回滚倒计时 | exact mode operation与receipt |
| PLH-P2-005 | Open | per-monitor DPI、safe area、orientation、折叠屏布局 | topology与DPI transaction |
| PLH-P2-006 | Open | 虚拟桌面、远程桌面、display reconnect恢复 | topology generation与placement migration |
| PLH-P2-007 | Open | 无边框、透明、click-through、always-on-top、窗口形状 | backend capability与权限模型 |
| PLH-P2-008 | Open | 窗口级accessibility title/role/state | stable window identity与observed state |
| PLH-P2-009 | Open | clipboard、URL、drag/drop、文件选择器统一operation | HostCommandBroker、principal、receipt |
| PLH-P2-010 | Open | kiosk、display wall、presentation、多输出同步 | multi-output ownership与frame pacing |
| PLH-P2-011 | Open | headless、virtual display、remote stream、automation backend | backend qualification与surface abstraction |
| PLH-P2-012 | Open | power、thermal、battery、background execution policy | lifecycle与resource budgets |
| PLH-P2-013 | Open | high-refresh、VRR与multi-window frame pacing | scheduler与per-output timing |
| PLH-P2-014 | Open | window/display/surface/lifecycle时间线与诊断UI | bounded evidence journal与qualified IDs |
| PLH-P2-015 | Open | hotplug/DPI/destroy/suspend/OOM/backend restart fault/soak | 幂等teardown与correctness gates |
| PLH-P2-016 | Open | 同协议窗口延迟、resize、present、恢复benchmark | 先冻结功能/硬件/OS/输出/统计协议 |

P2不能替代P0/P1。多窗口体验、HDR面板或高刷benchmark再完整，也不能掩盖假Supported、失效raw handle、固定viewport 1或无generation surface绑定。

## 10. 五引擎参考对照

| 参考 | 当前源码事实 | 对Zircon的约束 | 不照搬的部分 |
|---|---|---|---|
| Bevy | `WinitWindows`维护WindowId->window、entity->WindowId、WindowId->entity三向事实，remove同步清理；event先解析WindowId，unknown明确warn并拒绝。`AppLifecycle`独立为Idle/Running/WillSuspend/Suspended/WillResume；Android suspend移除`RawHandleWrapper`触发surface销毁，resume重建。state内tests验证scale/resize更新与target entity事件。 | 必须有双向registry、unknown/stale拒绝、per-window event identity；focus不能代替application lifecycle；suspend先撤销surface。 | 不把ECS Entity直接当跨ABI/持久WindowId；Zircon使用qualified handle。 |
| Fyrox | Executor在`Event::Resumed`初始化graphics context并通知plugin，在`Event::Suspended`先销毁graphics context再通知plugin；close/resize/redraw进入明确分支。 | 即使单窗口产品，也必须闭合graphics/surface与application lifecycle。 | 不把单窗口Executor当多窗口架构上限。 |
| Godot | `DisplayServer`所有window callback/command都带WindowID，提供window list、subwindow create/delete、transient关系与完整screen metrics；Windows backend删除子窗时先销毁rendering context/GL window，再DestroyWindow并从map erase。HDR区分supported/requested/enabled。Window test借DisplayServerMock验证真实窗口输入语义。 | Window Registry、display topology、targeted command、render context teardown和application notification必须分层。 | 不复制全局Singleton authority，按Runtime/Host实例隔离。 |
| Unreal | GenericApplication定义Make/InitializeWindow、PumpMessages、display metrics change；WindowsApplication维护window数组，按HWND解析消息，`WM_DESTROY`立即`Windows.Remove`，`WM_DISPLAYCHANGE`重建并广播metrics，`WM_DPICHANGED`按窗defer。 | native消息先解析窗口；destroy、display change、DPI是registry/topology事务；window/application activation分离。 | 不复制Slate类层级与宏，提取owner/generation/event/metrics合同。 |
| Unity Graphics | DynamicResolutionHandler按camera identity维护实例与最终viewport；HDR debug对每display分别公开active、available、gamut、format、luminance和mode-change-requested。 | output事实必须per-owner/per-output observed，区分available/requested/active/effective。 | Graphics仓库不含完整Unity window/event loop，不能据此外推其Platform架构。 |

参考结论不是“接口数量要等同Unreal”。工程线是：平台事实来自已安装且已观测的owner；window/display/surface有稳定identity与generation；lifecycle、unsafe lifetime与command terminal可证明；测试穿过真实产品链。正确性门通过后，才有资格比较事件延迟、CPU/RSS、frame pacing与恢复表现。

## 11. Target Architecture与硬切边界

```text
OS backend / export host
  -> PlatformHostService(instance, affinity, health)
       -> ApplicationLifecycleMachine
       -> WindowRegistry <-> DisplayTopologySnapshot
       -> HostCommandBroker
       -> EventLoopScheduler
       -> SurfaceLeaseRegistry
            -> Runtime viewport binding
            -> Graphics surface generation

OS event
  -> resolve native WindowId / DisplayId
  -> qualified event { owner, slot, generation, sequence, observed state }
  -> Runtime / UI / gameplay

command
  -> validate capability + target generation + deadline
  -> execute on platform thread
  -> publish observed/effective snapshot
  -> one terminal receipt
```

硬切删除：disabled却Supported的runtime capability、伪PrimaryWindowHandle、固定viewport 1全事件路由、丢弃WindowId、持久化monitor index、focus冒充application lifecycle、Destroyed不teardown、隐式exact-mode/monitor fallback、无generation surface bind、无ack OS副作用、shipping自动CPU fallback、generated callback恒真。

## 12. 重构里程碑

### M116.0 · Truth Freeze与Unsafe Repro

- RED测试冻结disabled/compiled/installed/observed/ready差异；
- 复现OS Destroyed后present、suspend时surface存活、resize重复bind；
- 冻结descriptor、event、surface ABI、host request与export callback schema。

### M116.1 · Platform Host与Capability Truth

- 引入真实backend instance、thread affinity、health、quiesce、terminal result；
- capability只由owner和observed evidence发布；planning matrix使用不同类型；
- disabled/unsupported严格fail-close。

### M116.2 · Window Registry与Command Protocol

- 建立stable WindowId、native双向映射、generation、关系图与primary role；
- create/update/close/destroy统一operation、deadline、terminal receipt；
- 多窗口event按真实target路由viewport/UI。

### M116.3 · Display Topology与State Reconciliation

- 建立DisplayId、immutable topology generation、hotplug diff和完整metrics；
- 分离create/requested/observed/effective；
- exact/fallback、DPI transaction与placement migration显式化。

### M116.4 · Application Lifecycle与Event Loop Scheduler

- 独立application activation、window focus、visibility、suspend、surface availability；
- 实现WillSuspend/Suspended/WillResume/Running/Exiting；
- scheduler融合frame demand、command、timer、wake、background并记录lateness/starvation。

### M116.5 · Surface Lease与Graphics Cutover

- window/output/surface generation贯穿ABI与Graphics；
- prepare/fence/publish/retire替换resize重复bind；
- Destroyed/Suspended/DestroySurfaces先撤销lease再释放native window。

### M116.6 · Export Host与Product Cutover

- browser/Android/iOS使用真实opaque instance和start/tick/event/metrics/suspend/resume/stop；
- unsupported/stale/failed返回typed terminal，不忽略payload；
- App、Editor、export、headless共享Platform Host contract。

### M116.7 · Qualification与Competitive Evidence

- 多窗、多显示、DPI、hotplug、系统destroy、suspend/resume、OOM、backend restart、soak；
- correctness/fault gates通过后再做latency/frame-pacing/CPU/RSS benchmark；
- 同硬件、OS、显示设置、present与统计协议对照Unreal/Godot/Bevy/Fyrox。

## 13. 验收矩阵

| Gate | Status | 验收内容 |
|---|---|---|
| PLH-G01 | Fail | disabled时全部runtime-owned platform capability为Disabled且不创建backend/window owner |
| PLH-G02 | Fail | capability区分Compiled/Selected/Installed/Initialized/Observed/Ready并携owner/generation/evidence |
| PLH-G03 | Fail | cross-target planning matrix不能被shipping当当前readiness |
| PLH-G04 | Fail | backend失败/退出/降级撤销Ready并给typed reason |
| PLH-G05 | Fail | native WindowId双向解析，unknown/stale拒绝并诊断 |
| PLH-G06 | Fail | slot reuse递增generation，旧event/command/surface不能命中新窗 |
| PLH-G07 | Fail | primary role、最后窗口关闭、工具窗关闭、app exit彼此独立 |
| PLH-G08 | Fail | 两窗口绑定不同viewport且事件不串到viewport 1 |
| PLH-G09 | Fail | transient/modal/parent关系无cycle并按拓扑teardown |
| PLH-G10 | Fail | 持久配置无live WindowId/monitor index，placement可迁移 |
| PLH-G11 | Fail | create/requested/observed/effective为独立generation |
| PLH-G12 | Fail | invalid/nonfinite resolution/constraints返回field diagnostic |
| PLH-G13 | Fail | exact fullscreen/monitor失败不隐式fallback |
| PLH-G14 | Fail | window命令含target generation/request ID/deadline且唯一终态 |
| PLH-G15 | Fail | 外部移动/缩放/最小化/全屏/DPI更新observed并reconcile |
| PLH-G16 | Fail | topology含DisplayId/generation/geometry/usable rect/DPI/scale/refresh |
| PLH-G17 | Fail | HDR/VRR/color/orientation/safe area以capability/Unknown表达 |
| PLH-G18 | Fail | hotplug产生原子diff，旧display generation拒绝或迁移 |
| PLH-G19 | Fail | 跨monitor/DPI同时更新geometry/UI scale/surface extent |
| PLH-G20 | Fail | output available/requested/active/effective可追溯display generation |
| PLH-G21 | Fail | focus/activation/visibility/occlusion/suspend/surface availability不互相冒充 |
| PLH-G22 | Fail | suspend为stop submit -> quiesce -> fence -> unbind/drop -> Suspended |
| PLH-G23 | Fail | resume只为有效window generation重建surface并成功后Ready |
| PLH-G24 | Fail | CloseRequested/Destroyed/backend loss/shutdown进入同一destroy事务 |
| PLH-G25 | Fail | resumed/can_create_surfaces重复触发只创建一个generation/receipt |
| PLH-G26 | Fail | memory warning进入budgeted purge，失败转Degraded |
| PLH-G27 | Fail | exiting清registry、取消命令并收terminal receipts |
| PLH-G28 | Fail | scheduler记录wake source/deadline/lateness/backlog/starvation |
| PLH-G29 | Fail | raw handle只在window generation存活期使用并有模型/故障测试 |
| PLH-G30 | Fail | surface replace为prepare/fence/publish/retire |
| PLH-G31 | Fail | ABI含qualified window/output/surface generation，stale fail-close |
| PLH-G32 | Fail | Win32/Android/iOS/browser shipping backend分别通过真实资格 |
| PLH-G33 | Fail | ReferenceCpuPresenter显式opt-in并报告Degraded/bytes/latency/drop |
| PLH-G34 | Fail | window/clipboard/URL/cursor/IME统一target/deadline/ack |
| PLH-G35 | Fail | export host有真实opaque instance且所有callback改变产品状态 |
| PLH-G36 | Fail | generated callback不忽略payload或恒真，失败有typed terminal |
| PLH-G37 | Fail | deterministic harness覆盖registry/generation/topology/lifecycle/command/surface |
| PLH-G38 | Fail | Windows与至少一移动平台覆盖Destroyed/DPI/hotplug/suspend/resume/recreate |
| PLH-G39 | Fail | correctness/fault/multi-window-display/soak通过后才跑性能benchmark |
| PLH-G40 | Fail | 同硬件/OS/显示/present/统计协议前禁止“优于Unreal”结论 |

## 14. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Runtime57账本逐项current-source复核 | review_complete | 2026-08-23 | 2 P0 Open / 63 P1 Open / 1 P1 Partial / 16 P2 Open |
| Window/Platform与App Host | review_complete | 2026-08-23 | 34 + 79文件；两组fingerprint与Runtime57相同 |
| ABI/surface/export与focused tests | review_complete | 2026-08-23 | 24 + 59文件；bind仍固定viewport/raw target |
| Bevy/Fyrox/Godot/Unreal/Unity Graphics | review_complete | 2026-08-23 | 27文件、27,316行；实现加Bevy/Godot focused tests/support |
| 40项资格门 | review_complete | 2026-08-23 | 40 Fail / 0 Pass |
| Production重构 | pending | - | 本篇未修改production、tests、Cargo或ABI |
| 动态/性能/竞争验证 | pending | - | 未运行Cargo、窗口/移动生命周期、fault/soak/profiler/benchmark |

Runtime116的review完成不等于Platform/Window系统完成。首个实现切片必须从M116.0的capability truth与unsafe surface lifetime RED repro开始；在两项P0关闭前，不应继续向单窗口App堆叠更多事件分支，也不得以cadence优化、source-shape测试或CPU fallback宣称Platform Host已经工程化。
