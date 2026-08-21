---
title: Platform Host、Window Registry、Monitor、Display、Event Loop、Application Lifecycle、Surface Command 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime57
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/window
  - zircon_runtime/src/core/framework/platform
  - zircon_runtime/src/platform
  - zircon_runtime/src/dynamic_api/surface.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/graphics/backend/render_backend/viewport_surface.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_surface
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files
  - zircon_runtime_interface/src/runtime_api/api_table.rs
  - zircon_runtime_interface/src/runtime_api/constants.rs
  - zircon_runtime_interface/src/runtime_api/events.rs
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime_interface/src/runtime_api/session.rs
  - zircon_runtime_interface/src/runtime_api/viewport.rs
  - zircon_app/src/entry/runtime_entry_app
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
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
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
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
  - dev/godot/platform/android/display_server_android.cpp
  - dev/godot/platform/web/display_server_web.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericApplication.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericWindow.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/GenericWindowDefinition.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/Windows/WindowsApplication.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Common/DynamicResolutionHandler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/HDROutputUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettingsHDROutput.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 57 · Platform Host、Window Registry、Monitor、Display、Event Loop、Application Lifecycle、Surface Command 与 Product Integration 工程化差距

## 1. 结论

当前窗口链不是完全没有实现。`WindowDescriptor`覆盖标题、模式、位置、分辨率、约束、可见性与焦点；App能够用winit创建一个窗口，将resize/focus/occlusion/close事件送入动态Runtime，在Win32上提取raw window/display handle，并通过Graphics绑定viewport surface；没有原生目标时还有softbuffer CPU presentation fallback；frame cadence可按Game、DesktopApp、Mobile、Continuous与Headless切换。这些是可保留的产品实验底座。

但它还不是一个工程级Platform Host。`PlatformModule`公开宣称负责platform、windowing与OS integration，真实`PlatformDriver`/`PlatformManager`却只拥有PreferenceStorage；能力矩阵只根据任意目标枚举与编译feature推断Supported，甚至`PlatformConfig.enabled=false`时仍可报告窗口后端、显示器、event loop与lifecycle受支持。App侧只有一个`Option<Arc<dyn Window>>`、一个descriptor和固定viewport 1，winit传入的`WindowId`与`DeviceId`被丢弃，`PrimaryWindowHandle`只检查`Some`而不参与寻址。它没有Window Registry、Monitor Registry、窗口命令协议、已请求/已观测状态分离或可恢复的Application Lifecycle状态机。

本轮确认两项本地P0。第一，Platform capability/readiness会把disabled、未安装、未观测且没有owner的窗口能力报告为Supported，产品和导出决策无法相信这份事实。第二，Graphics的raw-handle surface通过unsafe合同要求原生窗口在surface释放前保持有效，但`WindowEvent::Destroyed`只分发状态，不清除window、presenter或surface binding；App又没有`suspended`/`destroy_surfaces`路径，因此系统销毁、Android挂起或surface重建可能跨越已失效原生对象。导出模板所有lifecycle/touch/keyboard/metrics callback忽略参数并返回true，则继续由既有`TOOL-EXPORT-P0-005`唯一拥有，本篇把它列为继承发布阻断项，不重复计为第三个P0。

本轮登记 **2项P0、64项P1、16项P2和40项验收门禁**。目标不是继续向单窗口App追加事件分支，而是建立`PlatformHostService + WindowRegistry + DisplayTopologySnapshot + ApplicationLifecycleMachine + SurfaceLeaseRegistry + HostCommandBroker + EventLoopScheduler`，让每个窗口、显示器、surface和命令都具备稳定身份、generation、owner、已观测状态与terminal receipt。Runtime06继续拥有Platform/Input/Process广度，App01拥有产品host主循环，Runtime09A拥有GPU surface/present/device细节，Runtime11A拥有UI事件语义，Runtime24拥有通用identity，Runtime56拥有输入状态与设备；本篇拥有平台窗口纵向组合与这些owner之间的资格闭环。

本轮只做静态review与文档总账，没有修改production、tests、Cargo、ABI或参考源码；没有运行Cargo、真实窗口、多显示器、DPI、挂起恢复、surface-loss、移动端、soak或benchmark。不能据此宣称窗口稳定性、帧表现或性能达到、超过当前Unreal。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | fingerprint / 说明 |
|---|---:|---|
| Window与Platform contract/production | 34 / 2,794 / 96,162 / 0 | SHA-256 `751e8042a97c7001a04700c3d910e6f40a6c62b12e218f283793459890b857f3` |
| App product host完整目录 | 79 / 6,266 / 228,880 / 98 | SHA-256 `aa39192ae7c8cbdd59a4d97ee0513a019cb4531f6e1e3320bef31efc07390be8` |
| Dynamic ABI、surface与export integration | 26 / 8,375 / 336,478 / 25 | SHA-256 `9eb604923562893b0b3459191f390633d303985d95711a33aacf00f778e62d88` |
| focused direct/source-shape tests | 59 / 11,355 / 423,337 / 218 | SHA-256 `9bb0df174da2ae2cae3145249f1760966a3aa28cc2db66061f91bf756b01c852` |
| reference corpus | 23 / 18,519 / 754,201 / 6 | SHA-256 `f2b39ce28278be9a0e95d5eb78e5e8a6fddf6bd7aa550cb5e56163f7ee1766b3` |

fingerprint算法与Runtime56一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`编码，LF连接且末尾不追加LF，再计算UTF-8 SHA-256。它冻结的是本轮实际读取集合，不是Platform backend、Window ID、Display topology或Surface generation的产品身份。

窗口与Platform生产目录本轮读取内容为clean；`zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs`以及所选dynamic session中的`input_events.rs`、`state.rs`已有其他会话/用户改动，本文按当前working tree读取且不覆盖。共享索引也持续变化，因此`source_recheck_required`保持true。基线HEAD为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

### 2.2 范围与去重

- 逐文件读取framework window的13个生产文件、Platform非Preferences生产实现与相关Platform/window tests；Preferences持久化细节继续由Runtime45与Frameworks05拥有。
- 读取`runtime_entry_app`完整79文件并追踪ApplicationHandler、窗口创建、事件分发、cadence、lifecycle、surface binding/present、host request和native target；详细Input状态机继续由Runtime56拥有。
- 追踪Runtime Interface viewport/event/lifecycle/host request，dynamic session surface绑定与状态，以及Graphics raw-handle surface和export-generated platform host callback。
- App01的P1-14至P1-17已拥有单窗口/固定viewport、Destroyed不清理、缺suspended/exiting/memory、resize重复bind等产品host差距；本文不重号，只把它们放进Platform contract的纵向验收。
- Runtime09A拥有Lost/Outdated、present mode、device generation与HDR等GPU细节；Runtime11A拥有UI dispatch reply；Runtime56拥有输入device/action/replay。本文不复制这些finding。
- `TOOL-EXPORT-P0-005`已经唯一拥有generated mobile/browser host callback的空实现和假成功，本篇只登记依赖和Runtime侧接收合同。
- 没有把Unity Graphics当作完整窗口系统；只用其动态分辨率与HDR debug数据验证“显示能力必须来自当前已观测输出”，不外推Unity Application/Window行为。

## 3. 当前真实产品链

### 3.1 能力声明与真实owner

```text
PlatformConfig { enabled, target, features }
  -> PlatformManager::capability_report(config)
       -> target enum + compile feature selection
       -> Supported / FeatureDisabled / Unavailable

PlatformModule descriptor: "Platform, windowing, and OS integration"
  -> PlatformDriver { preference persistence backend }
  -> PlatformManager implements PreferenceStorage
  -> no EventLoop / Window / Monitor / Display / Surface owner
```

`enabled`没有进入capability计算。测试直接认证`platform.enabled=false`与`platform.window_backend=supported:headless`可同时成立；Android窗口、monitor、event、lifecycle也能仅凭构造的`bevy_default_platform()`选择被报告Supported，而导出callback仍全部no-op。这不是缺少一项诊断字段，而是能力控制面的truth source错误。

### 3.2 单窗口、固定viewport和身份丢失

```text
winit EventLoop::run_app
  -> ApplicationHandler::resumed/can_create_surfaces
  -> create_primary_window_surface
       -> if self.window.is_some(): return true
       -> descriptor.primary_window.is_some() only
       -> create one Window
       -> bind to ZrRuntimeViewportHandle::new(1)

window_event(_window_id, event)
  -> ignore WindowId
  -> dispatch every event to viewport 1
device_event(_device_id, event)
  -> ignore DeviceId
```

`PrimaryWindowHandle(u64)`可序列化、默认0，但产品链只判断`Option`，其数值不索引任何对象。没有native WindowId到engine handle的双向映射，没有generation，也没有unknown/stale ID拒绝。`WindowEvent::Destroyed`只上报runtime状态；CloseRequested路径才清理presenter/window/surface。因此来自OS的销毁和来自用户关闭的销毁具有两套不同资源语义。

### 3.3 Descriptor、Monitor与Surface

```text
persisted WindowDescriptor
  -> requested title/mode/position/resolution/present_mode
  -> dynamic-looking focused/scale/resolution fields mixed in same object
  -> build winit WindowAttributes
       -> monitor list collected into transient Vec
       -> Current/Primary/Index selection
       -> unsupported exact mode or missing monitor silently falls back

native window
  -> Win32 raw handle -> unsafe wgpu Surface
  -> other handles -> None -> RGBA frame capture -> softbuffer CPU present
```

descriptor不会被实际focus、monitor、mode或外部OS变化持续回写为权威observed snapshot；`present_mode`也没有进入Graphics surface选择，Graphics独立使用AutoVsync/Fifo/first fallback。Monitor只在创建时临时枚举，持久Index在热插拔、远程桌面、拓扑变化后没有稳定含义。resize会更新viewport后再次bind，没有显式旧surface unbind、replace generation或terminal receipt。

### 3.4 Application Lifecycle与导出host

ApplicationHandler实现`resumed`、`can_create_surfaces`、`proxy_wake_up`、`window_event`、`about_to_wait`和`device_event`，但没有`suspended`、`destroy_surfaces`、`memory_warning`、`exiting`或`new_events`。窗口focus false/true被直接映射为application Background/Foreground；Mobile只是一种cadence参数，不是移动平台的surface/application状态机。

browser、Android和iOS导出模板调用`zircon_export_handle_lifecycle/touch/keyboard/viewport_metrics`，但这些generated Rust callback忽略instance和payload并恒定返回true。它们甚至没有`ExportRuntimeInstance` owner、start/tick/stop或错误回执。该发布阻断由`TOOL-EXPORT-P0-005`拥有；Runtime57要求Platform Host为它提供可接入的真实lifecycle/window/display/surface合同。

## 4. 可保留基础

- `WindowDescriptor`已经把常用初始窗口参数集中到typed结构，而不是散落CLI magic number。
- resolution与constraints有最基本的finite/positive归一化，避免直接把NaN或零尺寸传给部分后端。
- `WindowLifecyclePolicy`显式区分close request和是否退出，后续可扩展成多窗口关闭策略。
- winit ApplicationHandler和frame cadence已形成独立文件边界，未来可以替换owner而不必重写整个entry。
- App能够把resize、scale、focus、occlusion和destroyed转换到Runtime event，Win32 raw handle也确实进入Graphics surface。
- Graphics对raw window/display handle做了目标类型验证，surface bind ABI含viewport与size，不是完全无类型裸指针。
- CPU fallback让无原生surface目标时仍可观察画面，适合作为显式diagnostic/reference backend。
- EventLoop control flow区分poll、wait和deadline，为后续scheduler提供了可迁移的局部策略。

这些基础只证明单窗口desktop实验链可运行，不证明Platform capability可信、多窗口身份安全、显示拓扑可恢复、移动生命周期闭合或surface lifetime满足unsafe合同。

## 5. P0 阻断项

| ID | 当前证据 | 工程后果 | 硬切目标 / owner |
|---|---|---|---|
| PLH-P0-001 | `PlatformConfig::default().enabled=false`，但`capability_report()`不读取enabled；Platform driver/manager只有Preferences owner，窗口/monitor/event/lifecycle仍可仅凭目标枚举与编译feature报Supported；测试主动认证disabled与supported并存 | capability、profile、导出和上层产品无法区分“代码被编译”“后端被选中”“owner已安装”“真实窗口已创建”“能力已观测”；shipping可对不存在的窗口链发布Ready | 建立`PlatformActivationPlan`与observed capability state：Selected -> Starting -> Ready/Degraded/Failed；每条能力携带provider instance、backend、generation、证据和失败原因；disabled必须Disabled且不能创建owner。Runtime57 + Runtime06/42/46/50/Tooling16 |
| PLH-P0-002 | Graphics通过`create_surface_unsafe`持有raw window/display handle，安全注释要求surface在原生窗口销毁前解绑/释放；App的`WindowEvent::Destroyed`只dispatch状态，不清window、presenter或surface，且ApplicationHandler没有`suspended`/`destroy_surfaces` | OS销毁、移动端挂起、surface重建或异常关闭后，Runtime仍可能redraw/present/rebind已失效原生对象；这是unsafe lifetime与资源代际合同被产品host违反，不只是缺少恢复动画 | 引入`SurfaceLease { window_id, window_generation, surface_generation }`；所有Destroyed/Suspended/DestroySurfaces先停止提交、等待in-flight、unbind/drop surface，再释放原生窗口；stale lease fail-close并有terminal receipt。Runtime57 + App01/Runtime09A/Runtime24 |

### 5.1 继承发布阻断，不重复计数

`TOOL-EXPORT-P0-005`：generated browser/Android/iOS host start与lifecycle/input/metrics callback没有真实Runtime instance，忽略参数并恒定成功。唯一canonical owner仍是Tooling03。本篇的Platform Host、ApplicationLifecycleMachine和HostCommandBroker必须先提供真实接收端；在该P0关闭前，移动端与browser Platform capability不得发布Ready。

## 6. P1 工程化差距

### 6.1 Authority、Capability 与 Backend Readiness

| ID | 差距 | 目标 / owner |
|---|---|---|
| PLH-P1-001 | `PlatformModule`宣称windowing/OS integration，真实实现只装配PreferenceStorage | 模块descriptor按已安装service发布精确能力，Window Host由真实driver/service拥有 |
| PLH-P1-002 | `PlatformDriver`没有EventLoop、Window、Monitor、Display、clipboard、URL或power owner | 定义Platform backend trait与线程亲和、启动、health、quiesce、teardown合同 |
| PLH-P1-003 | `PlatformManager`没有Window Registry或命令/query接口 | Manager只发布generation snapshot与operation handle，不暴露临时native对象 |
| PLH-P1-004 | 配置中的target是可任意构造枚举，不证明当前binary/OS/arch/backend匹配 | BuildSet冻结compiled target，runtime只允许兼容observed target；cross-target查询明确为planning capability |
| PLH-P1-005 | compile feature selection被直接解释为runtime backend可用 | 区分Compiled、Selected、Installed、Initialized、Observed、Ready六种事实 |
| PLH-P1-006 | capability没有provider instance、owner session或generation | 每条capability绑定qualified owner与generation，owner退出后自动失效 |
| PLH-P1-007 | `CapabilityStatus`只有Supported/FeatureDisabled/Unavailable | 增加Starting、Ready、Degraded、Failed与typed reason，Supported只用于静态catalog |
| PLH-P1-008 | capability diagnostic缺probe时间、backend版本、设备/显示器证据与currentness | 发布bounded evidence record，拓扑或owner变化使旧证据stale |
| PLH-P1-009 | Platform tests用人工`bevy_default_platform()`认证Android/desktop能力，不实例化backend | 分离pure planning tests与real backend qualification，命名和断言禁止互相替代 |
| PLH-P1-010 | Platform owner没有health、restart、quiesce、terminal failure或资源泄漏报告 | 接入Engine Module lifecycle与Operation receipt；失败不可继续保持Ready |

### 6.2 Window Identity、Registry 与 Multi-Window

| ID | 差距 | 目标 / owner |
|---|---|---|
| PLH-P1-011 | App只保存一个`Option<Arc<dyn Window>>` | `WindowRegistry`按stable handle管理任意数量window slot |
| PLH-P1-012 | `PrimaryWindowHandle`数值完全不参与寻址，只检查`Option::is_some` | primary是registry中的qualified handle，所有命令和event都真实解析 |
| PLH-P1-013 | Primary handle默认0且可直接serde持久化，没有namespace/owner/generation | 使用`WindowId { registry, slot, generation }`；持久化只保存placement key，不保存live handle |
| PLH-P1-014 | 所有窗口事件硬路由到`ZrRuntimeViewportHandle::new(1)` | Window-to-Viewport binding table支持一对一、一对多与无viewport工具窗 |
| PLH-P1-015 | winit `WindowId`参数被丢弃 | native ID先查双向registry，再生成engine event；unknown ID拒绝并诊断 |
| PLH-P1-016 | 没有engine handle -> native ID与native ID -> engine handle双向映射 | 两个索引同generation原子更新，remove时一起清理；参考Bevy `WinitWindows` |
| PLH-P1-017 | 窗口重建后没有generation，旧event/command/surface可误命中新对象 | registry slot reuse必须递增generation，stale request fail-close；Runtime24 |
| PLH-P1-018 | unknown/stale WindowId没有typed错误路径，因为ID从未被解析 | event、query、command、surface bind统一返回Unknown/Stale/Closing/Destroyed |
| PLH-P1-019 | `Destroyed`不从任何registry移除对象，也不发布terminal generation | destroy事务先Closing、撤销binding、释放surface、移除native映射，再发布Destroyed receipt |
| PLH-P1-020 | 没有secondary/tool/popup/child window创建与销毁协议 | WindowCreateOperation声明kind、owner、parent、viewport、policy并返回handle |
| PLH-P1-021 | primary window没有选举、替换、失效或“最后窗口关闭”策略 | primary role与application exit policy分离，替换产生generation event |
| PLH-P1-022 | 没有transient parent、modal、always-on-top或owner shutdown关系 | registry维护窗口关系图并检测cycle；owner teardown按拓扑关闭 |

### 6.3 Descriptor、Requested/Observed State 与 Window Command

| ID | 差距 | 目标 / owner |
|---|---|---|
| PLH-P1-023 | `WindowDescriptor`混合创建请求与动态外观字段 | 分离immutable create spec、requested state、observed snapshot和effective policy |
| PLH-P1-024 | `focused`可序列化为配置，但实际focus事件不维护同一权威对象 | focus只属于observed state，不作为下一次创建的无条件请求 |
| PLH-P1-025 | descriptor `present_mode`不进入Graphics surface选择 | present policy由surface negotiation消费并返回requested/effective/fallback reason；Runtime09A |
| PLH-P1-026 | 非finite、零或非法resolution被静默改为1/default | typed validator返回field diagnostic；只有显式Sanitize policy可修正并留receipt |
| PLH-P1-027 | min/max constraints与后端实际接受值没有effective-state回读 | command完成后发布effective constraints，拒绝或clamp必须可见 |
| PLH-P1-028 | exact fullscreen video mode找不到时静默退为borderless | Exact请求失败；AllowFallback才可协商，并返回选中的mode与原因 |
| PLH-P1-029 | monitor选择不存在时静默使用其他monitor | strict placement返回MonitorUnavailable；fallback policy必须显式 |
| PLH-P1-030 | 创建期`Current`没有当前monitor，最终退为automatic placement | Current只对已有window有效；create必须使用Primary/StableId/Point/Automatic |
| PLH-P1-031 | 居中使用完整video mode物理尺寸而非usable work area/safe area | 通过DisplayTopologySnapshot的logical usable rect与DPI generation布局 |
| PLH-P1-032 | 没有运行期标题、尺寸、位置、模式、可见性、装饰、层级等统一命令协议 | `WindowCommand`含target generation、request ID、deadline与desired state |
| PLH-P1-033 | 窗口操作没有Accepted/Applied/Rejected/Canceled/Failed terminal receipt | HostCommandBroker发布exact effective state和platform error，不用bool假成功 |
| PLH-P1-034 | 外部OS拖动、缩放、最小化、全屏切换后没有requested/observed reconciliation | event更新observed generation，并按policy接受、纠偏或提示冲突 |

### 6.4 Monitor、Display 与 Output State

| ID | 差距 | 目标 / owner |
|---|---|---|
| PLH-P1-035 | `WindowMonitorSelection::Index(usize)`可序列化，索引随热插拔与远程会话改变 | 持久placement使用stable display key + EDID/connector/profile hint并允许迁移 |
| PLH-P1-036 | 创建窗口时临时收集`Vec<MonitorHandle>`，之后没有inventory owner | Platform Host维护immutable `DisplayTopologySnapshot` |
| PLH-P1-037 | monitor/output没有engine稳定ID或generation | DisplayId区分physical output、logical screen和render output，拓扑变化递增generation |
| PLH-P1-038 | 没有monitor add/remove/mode change/hotplug事件 | backend重建拓扑diff并发布added/changed/removed与失效reason |
| PLH-P1-039 | 缺position、logical/physical size、usable rect、DPI、scale、refresh、orientation、safe area、color/HDR/VRR统一事实 | 以backend可观测字段发布capability bits；未知值显式Unknown，不填默认假值 |
| PLH-P1-040 | 没有window当前所属monitor及跨monitor迁移事件 | 每个observed window snapshot带DisplayId与topology generation |
| PLH-P1-041 | ScaleFactorChanged只调整surface尺寸，不更新权威window/display snapshot | DPI transaction同时更新logical/physical geometry、UI scale与surface extent |
| PLH-P1-042 | display change没有类似Unreal display metrics rebuild/broadcast | WM_DISPLAYCHANGE/对应平台事件触发原子snapshot replacement和订阅通知 |
| PLH-P1-043 | viewport surface没有绑定render output/display identity与协商generation | surface lease记录output、format、present、color space和display generation；Runtime09A |
| PLH-P1-044 | dynamic resolution/HDR调试状态没有区分available、requested、active与effective | 参考Unity Graphics发布observed per-output状态，不从compile flag推断active |

### 6.5 Event Loop 与 Application Lifecycle

| ID | 差距 | 目标 / owner |
|---|---|---|
| PLH-P1-045 | ApplicationHandler没有`suspended` | ApplicationLifecycleMachine发布WillSuspend/Suspended并执行有序quiesce |
| PLH-P1-046 | 没有`destroy_surfaces` | surface owner可在window仍存在时先撤销所有surface lease并等待in-flight |
| PLH-P1-047 | 没有`memory_warning` | memory pressure进入Runtime事件、预算收缩和bounded purge operation |
| PLH-P1-048 | 没有`exiting` | event loop仍活跃时停止接收命令、清registry并收集terminal receipts |
| PLH-P1-049 | 没有`new_events`/StartCause处理 | timer、poll、resume、wait-cancel和proxy wake原因进入scheduler evidence |
| PLH-P1-050 | WindowFocused false/true直接等价Application Background/Foreground | 窗口focus、application activation、visibility和suspend是独立状态维度 |
| PLH-P1-051 | Occluded只改变frame cadence，没有权威visibility/covered snapshot | 每窗维护visible/minimized/occluded状态，再由policy决定渲染与tick |
| PLH-P1-052 | Mobile只是cadence枚举，不处理平台surface与app lifecycle | Android/iOS走真实resume/suspend/surface create/destroy状态机 |
| PLH-P1-053 | resumed以`window.is_some()`作为幂等条件，无法识别原生对象或surface已失效 | transition依据window/surface generation与backend state，不依据Option存在 |
| PLH-P1-054 | CloseRequested执行teardown，系统Destroyed却只发event，两条路径不收敛 | 所有close/destroy/backend-loss进入同一idempotent destroy transaction |
| PLH-P1-055 | `resumed`与`can_create_surfaces`都可触发创建，却没有single-flight transition generation | lifecycle operation使用CAS state与single terminal receipt |
| PLH-P1-056 | EventLoop cadence只决定Poll/Wait/WaitUntil，没有wake source、deadline domain、backlog或starvation事实 | EventLoopScheduler合并frame demand、host command、timer、background policy并记录lateness |

### 6.6 Surface Binding、Host Command、Export 与 Tests

| ID | 差距 | 目标 / owner |
|---|---|---|
| PLH-P1-057 | native surface target只实现Win32，其他平台handle一律None | 每个shipping平台有qualified backend；缺失时Unavailable，不自动冒充正常路径 |
| PLH-P1-058 | CPU fallback每帧捕获完整RGBA再softbuffer present，却没有Degraded capability和成本预算 | 把它命名为ReferenceCpuPresenter，显式opt-in并发布copy bytes/latency/drop |
| PLH-P1-059 | resize在同一viewport上再次bind，没有显式unbind/replace transaction | prepare新surface/extent -> fence旧提交 -> atomic publish -> retire旧generation |
| PLH-P1-060 | `ZrRuntimeBindViewportSurfaceRequestV1`只有viewport、size和target，没有window/surface/output generation | ABI加入qualified surface lease或opaque generation handle；stale target拒绝 |
| PLH-P1-061 | host request ABI只有IME、rumble、cursor，没有WindowCommand、clipboard、open URL或display operation | 扩展versioned HostCommand page并按capability路由；Interface01/05/07 |
| PLH-P1-062 | cursor等host request缺target window/viewport generation，整体也缺deadline、cancel和ack | 所有OS副作用必须绑定target generation并有terminal result |
| PLH-P1-063 | export callback的Runtime接收端没有instance/lifecycle/window/display/surface operation合同 | 为Tooling03的`TOOL-EXPORT-P0-005`提供真实opaque instance和fail-close API，不复制其P0 |
| PLH-P1-064 | App测试主要`include_str!`检查fragment/顺序；没有真实EventLoop创建、多窗路由、系统Destroyed、DPI/hotplug、suspend/resume或surface lease集成测试 | 建立backend contract tests、headless deterministic harness与至少Windows/Android真实产品资格矩阵 |

## 7. P2 完整产品能力

| ID | 能力 | 前置条件 |
|---|---|---|
| PLH-P2-001 | 多窗口Editor工具窗、游戏子窗、popup和modal产品体验 | Window Registry与关系图完成 |
| PLH-P2-002 | 跨显示器窗口placement profile与拓扑迁移 | stable DisplayId、usable rect和placement migration完成 |
| PLH-P2-003 | HDR、wide gamut、VRR、refresh与color-space用户选择 | observed output capability与surface negotiation完成 |
| PLH-P2-004 | exclusive fullscreen mode chooser和安全回滚倒计时 | exact mode operation与terminal receipt完成 |
| PLH-P2-005 | per-monitor DPI、safe area、orientation与折叠屏响应式布局 | DisplayTopologySnapshot与DPI transaction完成 |
| PLH-P2-006 | 虚拟桌面、workspace、远程桌面和display reconnect恢复 | topology generation与placement policy完成 |
| PLH-P2-007 | 无边框、透明、click-through、always-on-top与窗口形状能力 | backend capability和安全权限模型完成 |
| PLH-P2-008 | 窗口级accessibility title/role/state与系统辅助技术桥 | stable window identity与observed state完成 |
| PLH-P2-009 | clipboard、open URL、drag/drop与文件选择器统一operation | HostCommandBroker、principal与receipt完成 |
| PLH-P2-010 | kiosk、display wall、presentation与多输出同步模式 | multi-output ownership和frame pacing完成 |
| PLH-P2-011 | headless、virtual display、remote stream和automation window backend | backend qualification与surface abstraction完成 |
| PLH-P2-012 | power、thermal、battery、background execution与platform policy | ApplicationLifecycleMachine和resource budgets完成 |
| PLH-P2-013 | high-refresh、variable-refresh和multi-window frame pacing | EventLoopScheduler与per-output timing完成 |
| PLH-P2-014 | window/display/surface/lifecycle时间线与故障诊断UI | bounded evidence journal和qualified IDs完成 |
| PLH-P2-015 | hotplug、DPI、destroy、suspend、OOM和backend restart fault-injection/soak | correctness与idempotent teardown gates完成 |
| PLH-P2-016 | 与Unreal/Godot/Bevy/Fyrox同协议的窗口延迟、resize、present与恢复benchmark | 功能、硬件、OS、输出模式和统计协议先冻结 |

P2不能替代P0/P1。多窗口工具体验、HDR面板或高刷benchmark再完整，也不能掩盖假Supported、失效raw handle或没有generation的surface绑定。

## 8. 参考引擎对照

| 参考 | 本轮源码事实 | 对Zircon的约束 | 不照搬的部分 |
|---|---|---|---|
| Bevy | `WinitWindows`维护WindowId -> window、entity -> WindowId、WindowId -> entity三向事实，remove同步清理；window event先解析ID，unknown会告警拒绝；每个事件携带window entity。`AppLifecycle`独立定义Idle/Running/WillSuspend/Suspended/WillResume；Android suspend移除RawHandleWrapper以销毁surface，resume恢复 | 必须有双向registry、unknown/stale拒绝、每窗事件identity；window focus不能代替application lifecycle；suspend必须撤销surface generation | 不把ECS Entity直接当跨ABI/持久window ID；Zircon仍需自己的qualified handle |
| Fyrox | Executor在Resumed初始化graphics context并通知插件，在Suspended显式销毁graphics context并通知插件 | 即便单窗口产品，也必须把graphics/surface lifecycle与app lifecycle闭合 | 不以其单窗口Executor作为多窗口架构上限 |
| Godot | DisplayServer拥有WindowID、MAIN/INVALID、window list、subwindow create/delete，所有window callback/command带目标ID；screen API提供count、primary、focus、position、size、usable rect、DPI、scale与refresh。Windows backend维护WindowID -> WindowData map、focused IDs与transient关系；MainLoop另有memory warning、resume、pause、focus通知 | Window registry、display topology、命令target与application notification必须分层；unsupported operation要显式报错 | 不复制Singleton式DisplayServer全局authority；按Runtime/Host实例隔离 |
| Unreal | GenericApplication拥有Make/InitializeWindow、MessageHandler、PumpMessages和display metrics changed；WindowsApplication维护window数组并按HWND解析消息，WM_DESTROY移除window，WM_DISPLAYCHANGE重建并广播metrics，WM_DPICHANGED按窗处理；window activation与application activation分开 | native消息必须先解析窗口身份；销毁、显示变化和DPI是registry/topology事务；app/window activation不可混同 | 不复制Slate/Windows类层级与宏；提取owner、generation、event和metrics合同 |
| Unity Graphics | DynamicResolutionHandler按camera owner维护实例和最终viewport；HDR Output debug公开当前display的available/active/gamut/format/luminance/request状态 | 输出能力必须是per-output/per-owner observed state，区分available、requested、active和effective | Graphics仓库不提供完整window/event loop实现，不据此推断Unity平台架构 |

参考结论不是“接口数量要等同Unreal”。真正的工程线是：平台事实必须来自已安装且已观测的owner；窗口与显示器必须有稳定身份和generation；lifecycle、surface lifetime与命令完成必须可证明；测试必须穿过真实产品链。满足这些正确性门后，才有资格比较延迟、CPU、内存、frame pacing与恢复表现。

## 9. Target Architecture、Owner 与硬切边界

### 9.1 目标数据流

```text
OS backend / export host
  -> PlatformHostService(instance, thread affinity, health)
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
  -> Runtime / UI / gameplay consumers

command
  -> validate target generation + capability + deadline
  -> execute on platform thread
  -> publish observed snapshot
  -> terminal receipt
```

### 9.2 Owner边界

| Owner | 本篇负责 | 继续由父报告负责 |
|---|---|---|
| Runtime57 | Platform Host纵向组合、Window/Display registry、Application lifecycle、surface lease、window command与资格门 | 不拥有所有Input、GPU内部或Preferences实现 |
| Runtime06 | Platform/Input/Process首轮广度和platform owner方向 | 不再复制本篇已编号的window/display/lifecycle纵向finding |
| App01 | 产品bootstrap、event loop、动态Runtime、shutdown和当前单窗口实现 | Runtime57定义其必须满足的平台合同与surface安全门 |
| Runtime09A | RHI surface、present status、device generation、format/HDR协商 | Runtime57提供window/output/surface lease与lifecycle边界 |
| Runtime11A / Runtime56 | UI事件语义、input state/device/action/replay | WindowId、focus/app activation与platform target由Runtime57提供 |
| Runtime24 / 43 / Interface01/05/07 | 通用stable identity、dynamic session和foreign ABI安全 | 本篇定义window/display/surface/command具体schema |
| Runtime42/46/50 / Tooling16 | module composition、service/manager lifecycle和capability truth | Platform owner只有真实backend与observed evidence时可Ready |
| Tooling03 | export build/cook/pack和`TOOL-EXPORT-P0-005` | Runtime57提供导出host可调用的真实运行时接收合同 |

硬切删除清单：disabled却Supported的Platform capability；只用于`is_some`的伪PrimaryWindowHandle；固定viewport 1的全事件路由；丢弃WindowId；可持久化monitor index；把focus当application lifecycle；Destroyed不teardown；隐式exact-mode/monitor fallback；无generation的surface bind；没有ack的窗口副作用；shipping平台自动降级为无标识CPU presenter；generated callback恒真成功。

## 10. 重构里程碑

### M0 · Truth Freeze 与 Unsafe Repro

- 把Platform disabled/compiled/installed/observed/ready状态拆开并建立source/product reachability guard；
- 建立OS Destroyed后present、suspend时surface存活和resize重复bind的最小失败repro；
- 冻结现有descriptor、viewport surface ABI、lifecycle event和export callback schema。

### M1 · Platform Host 与 Capability Truth

- 引入真实backend instance、thread affinity、health、quiesce和terminal result；
- capability由owner实例与observed evidence发布，disabled/unsupported fail-close；
- planning target matrix与runtime readiness使用不同类型和API。

### M2 · Window Registry 与 Command Protocol

- 建立stable WindowId、native双向映射、generation、关系图与primary role；
- create/update/close/destroy全部走有deadline和terminal receipt的operation；
- 多窗口event按真实target路由到viewport/UI，而非固定viewport 1。

### M3 · Display Topology 与 Requested/Observed State

- 建立DisplayId、immutable topology generation、hotplug diff和完整metrics；
- 分离create spec、requested、observed与effective state；
- exact/fallback policy、DPI transaction与placement migration全部显式化。

### M4 · Application Lifecycle 与 Event Loop Scheduler

- 独立application activation、window focus、visibility、suspend和surface availability；
- 实现WillSuspend/Suspended/WillResume/Running/Exiting有序状态机；
- scheduler融合frame demand、host command、timer、wake和background policy并输出lateness。

### M5 · Surface Lease 与 Graphics Cutover

- window/output/surface generation贯穿ABI与Graphics；
- prepare/fence/publish/retire替换resize重复bind；
- Destroyed/Suspended/DestroySurfaces严格先撤销surface lease再释放native window。

### M6 · Export Host 与 Product Cutover

- browser/Android/iOS使用真实opaque runtime instance和start/tick/event/metrics/suspend/resume/stop；
- 不支持的平台/命令返回typed error，禁止callback恒真；
- App、Editor、export和headless通过同一Platform Host contract，backend差异留在provider。

### M7 · Qualification、Fault 与 Competitive Evidence

- 完成多窗、多显示器、DPI、hotplug、系统销毁、挂起恢复、OOM、backend restart与长时soak；
- 在correctness和fault gates通过后再做事件延迟、resize、surface replace、frame pacing、CPU/RSS benchmark；
- 同硬件、OS、窗口/显示设置和统计协议下比较Unreal/Godot/Bevy/Fyrox，禁止用源码规模或单次微基准声称领先。

## 11. 验收矩阵

| Gate | 验收内容 |
|---|---|
| PLH-G01 | `PlatformConfig.enabled=false`时所有runtime-owned platform capability为Disabled，且没有backend/window owner被创建 |
| PLH-G02 | capability明确区分Compiled、Selected、Installed、Initialized、Observed与Ready，且携带provider/generation/evidence |
| PLH-G03 | 任意构造的cross-target planning matrix不能被Runtime或shipping profile当作当前backend readiness |
| PLH-G04 | backend启动失败、owner退出或health降级会撤销Ready并给出typed terminal reason |
| PLH-G05 | 每个native WindowId可双向解析为唯一engine WindowId，unknown/stale event被拒绝并诊断 |
| PLH-G06 | window slot复用递增generation，旧command/event/surface lease不能命中新窗口 |
| PLH-G07 | primary role可替换/失效；最后窗口关闭、工具窗关闭和application exit policy彼此独立 |
| PLH-G08 | 至少两个窗口绑定不同viewport，resize/focus/close/destroy不会串路由到viewport 1 |
| PLH-G09 | transient/modal/parent窗口关系无cycle，owner teardown按关系图有序完成 |
| PLH-G10 | persisted配置不包含live WindowId或monitor index，placement key可在拓扑变化后迁移/降级并报告 |
| PLH-G11 | create spec、requested、observed、effective window state为独立generation，可解释任何OS clamp/fallback |
| PLH-G12 | invalid/nonfinite resolution与constraints返回field diagnostic，不静默改成1/default |
| PLH-G13 | exact fullscreen或monitor请求失败时不隐式borderless/automatic；fallback必须由policy允许且有receipt |
| PLH-G14 | runtime窗口命令均含target generation、request ID、deadline，并只产生一个terminal result |
| PLH-G15 | 外部拖动、缩放、最小化、全屏与DPI变化更新observed snapshot并触发明确reconciliation |
| PLH-G16 | DisplayTopologySnapshot包含stable DisplayId、generation、geometry、usable rect、DPI、scale和refresh |
| PLH-G17 | 可选HDR/VRR/color/orientation/safe-area字段以capability/Unknown表达，不用假默认值冒充观测 |
| PLH-G18 | monitor add/remove/mode change产生原子topology diff，旧DisplayId/generation请求被拒绝或迁移 |
| PLH-G19 | window跨monitor与DPI变化同时更新logical/physical geometry、UI scale和surface extent |
| PLH-G20 | output available/requested/active/effective状态可查询，present/color policy能回溯到display generation |
| PLH-G21 | window focus、application activation、visibility、occlusion、suspend和surface availability互不冒充 |
| PLH-G22 | suspend顺序为stop submit -> quiesce -> fence -> unbind/drop surface -> publish Suspended |
| PLH-G23 | resume只为有效window generation重建surface，并在成功后发布Running/Ready |
| PLH-G24 | CloseRequested、OS Destroyed、backend loss与owner shutdown进入同一幂等destroy事务 |
| PLH-G25 | `resumed`与`can_create_surfaces`并发/重复触发只创建一个window/surface generation和一个terminal receipt |
| PLH-G26 | memory warning进入budgeted purge路径；失败或超预算进入Degraded而不是无声忽略 |
| PLH-G27 | exiting在event loop仍可处理平台操作时清registry、取消命令并收集terminal receipts |
| PLH-G28 | EventLoopScheduler记录wake source、deadline、lateness、backlog和starvation，background policy可审计 |
| PLH-G29 | raw native handle只在对应window generation存活期使用，unsafe surface contract有模型/故障测试 |
| PLH-G30 | resize/replace通过prepare-fence-publish-retire完成，旧surface generation无提交者后才销毁 |
| PLH-G31 | surface ABI包含qualified window/output/surface generation；stale/unknown/closing target fail-close |
| PLH-G32 | Win32、Android、iOS、browser等shipping backend各自通过真实资格；缺失backend报告Unavailable |
| PLH-G33 | ReferenceCpuPresenter必须显式opt-in并报告Degraded、copy bytes、latency与drop，不冒充native path |
| PLH-G34 | WindowCommand、clipboard、open URL、cursor、IME等host副作用统一target generation、deadline和ack |
| PLH-G35 | export host拥有真实opaque Runtime instance，start/tick/event/metrics/suspend/resume/stop均改变产品状态 |
| PLH-G36 | generated callback不再忽略payload或恒定返回true；unsupported、stale与failed有typed terminal result |
| PLH-G37 | deterministic headless contract tests覆盖registry、generation、topology、lifecycle、command和surface lease |
| PLH-G38 | Windows与至少一个移动平台真实集成测试覆盖系统Destroyed、DPI/hotplug、suspend/resume和surface recreate |
| PLH-G39 | correctness、fault injection、multi-window/display、soak全部通过后，才运行latency/frame-pacing/RSS benchmark |
| PLH-G40 | 同硬件、同OS、同窗口/显示/刷新/present设置与统计协议完成Unreal对照前，不允许“性能或表现优于Unreal”结论 |

## 12. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Window与Platform contract审查 | review_complete | 2026-08-20 | 34文件、2,794行、96,162 bytes |
| App product host完整目录审查 | review_complete | 2026-08-20 | 79文件、6,266行、228,880 bytes、98 test attributes |
| Dynamic ABI/surface/export审查 | review_complete | 2026-08-20 | 26文件、8,375行、336,478 bytes、25 test attributes |
| focused tests审查 | review_complete | 2026-08-20 | 59文件、11,355行、423,337 bytes、218 test attributes；无真实多窗/挂起/热插拔资格矩阵 |
| Bevy/Fyrox/Godot/Unreal/Unity Graphics对照 | review_complete | 2026-08-20 | 23文件、18,519行、754,201 bytes |
| P0/P1/P2与验收门禁 | review_complete | 2026-08-20 | 2 P0 / 64 P1 / 16 P2 / 40 gates；另有1项继承Tooling03 P0 |
| Production重构 | pending | - | 本篇不修改production、tests、Cargo或ABI |
| 动态/性能/竞争性验证 | pending | - | 未运行Cargo、窗口、多显示器、移动生命周期、surface fault、soak或benchmark |

Runtime57的review完成不等于Platform/Window系统完成。实施前必须重读current source、App01、Runtime09A和Tooling03的`TOOL-EXPORT-P0-005`；任何ApplicationHandler、WindowDescriptor、Platform capability、viewport surface ABI、raw-handle backend或export callback变化都应使本报告进入recheck。下一批review应转向尚未深审的独立Runtime垂直面，不继续扩写本篇或回到tooling优化。
