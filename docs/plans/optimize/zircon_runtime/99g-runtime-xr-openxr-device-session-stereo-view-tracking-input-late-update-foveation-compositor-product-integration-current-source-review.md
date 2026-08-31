---
title: Runtime XR/OpenXR、Device/Session、Stereo View、Tracking/Input、Late Update、Foveation、Compositor 与 Product Integration 当前源码工程化差距复核
category: zircon_runtime
report_id: Runtime106
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/builtin/runtime_modules
  - zircon_runtime/src/plugin/runtime_profile
  - zircon_runtime/src/core/framework/render/backend_types/capability.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/view_family.rs
  - zircon_runtime/src/core/framework/render/viewport_product.rs
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_new_offscreen.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/crates/zr_rhi/src/device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/device.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/frame_cadence.rs
  - zircon_editor/src
  - zircon_plugins
tests:
  - zircon_runtime/src/builtin/runtime_modules/tests
  - zircon_runtime/src/core/framework/render/backend_types/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/camera_history_key.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
  - zircon_app/src/entry/tests
  - zircon_editor/src/tests
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/66-runtime-xr-openxr-device-session-stereo-view-tracking-input-late-update-foveation-compositor-product-integration-review.md
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_Swapchain.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Internal/OpenXRHMD_Swapchain.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_Layer.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Public/IOpenXRExtensionPlugin.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRInput/Private/OpenXRInput.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/HeadMountedDisplay/Private/LateUpdateManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/HeadMountedDisplay/Public/IXRTrackingSystem.h
  - dev/godot/modules/openxr/openxr_api.cpp
  - dev/godot/modules/openxr/openxr_api.h
  - dev/godot/modules/openxr/action_map/openxr_action_map.cpp
  - dev/godot/modules/openxr/action_map/openxr_action_map.h
  - dev/godot/modules/openxr/action_map/openxr_action.cpp
  - dev/godot/modules/openxr/action_map/openxr_action.h
  - dev/godot/modules/openxr/action_map/openxr_interaction_profile.cpp
  - dev/godot/modules/openxr/action_map/openxr_interaction_profile.h
  - dev/godot/modules/openxr/extensions/openxr_fb_foveation_extension.cpp
  - dev/godot/modules/openxr/extensions/openxr_hand_tracking_extension.cpp
  - dev/godot/modules/openxr/extensions/openxr_eye_gaze_interaction.cpp
  - dev/godot/modules/openxr/extensions/openxr_visibility_mask_extension.cpp
  - dev/godot/modules/openxr/editor/openxr_action_map_editor.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/XR/XRSystemUniversal.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/XRDepthMotionPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRPassTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRLayoutTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRLayoutStackTests.cs
  - dev/bevy/crates/bevy_render/src/texture/manual_texture_view.rs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/Fyrox/fyrox-impl/src/renderer/mod.rs
doc_type: review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime106：Runtime XR/OpenXR 当前源码工程化差距复核

## 1. 结论

截至本轮冻结，Zircon 仍然**没有 XR/OpenXR 产品能力**。18,759 个 tracked 产品文本源码文件中，`OpenXR/openxr`、whole-word `XR`、`HMD/HeadMounted`、`foveat*`、`xrWaitFrame/xrBeginFrame/xrEndFrame/xrLocateViews` 与 `predicted display time` 的有效命中均为 0；Cargo manifest 没有 OpenXR loader/dependency/feature，builtin module catalog、runtime profile、App、Editor、plugins、examples 和 tests 也没有 XR provider、session、graphics binding、action map、composition layer 或产品入口。

Runtime66 的判断因此没有被后续源码推翻：其 **72 个 P1、16 个 P2、XR-M0 至 XR-M9 和 48 个验收门禁仍是唯一 canonical owner**。本轮登记 **0 个新增 P0、0 个新增 umbrella P1、0 个新增 P2**，避免把同一缺口复制成第二套责任清单；没有任何 RT66 问题可以标记为 closed。

当前源码并非毫无可复用基础。普通窗口/多相机路径新增了更精确的 viewport-qualified temporal history identity，camera loop 也从整帧复制收敛为 source-state capture、`Arc::make_mut` 与 advanced payload 借用；runtime plugin/profile 已能做 generic capability、maturity 与 unavailable reason 的 fail-close。这些是正确工程方向，但仍然只解决普通 camera/window/plugin 问题，不能表达 OpenXR runtime authority、同一 predicted display time 下的 view family、runtime-owned swapchain image 或 action/space generation。

本轮还修正两项旧证据：Runtime66 的 `multiview_mask` 统计从 119 更新为当前 **122 个赋值，122 个全部为 `None`，`Some` 为 0**；当前 Bevy `manual_texture_view.rs` 只提供 manually managed `TextureView` render target landing point，源码注释不再明确提及 OpenXR，因此它只能证明一个可复用接入缝，不是 XR 实现证据。

本轮是 review-only，没有修改 production/test/Cargo/ABI，也没有运行 Cargo、真实 OpenXR runtime、头显、GPU capture、CTS、motion-to-photon、热功耗、soak 或竞争性 benchmark。因此不能宣称“已支持 XR”，也不能宣称性能或表现达到、优于 Unreal。

## 2. 审查边界、冻结与 ownership

### 2.1 唯一 owner 与去重规则

| 领域 | Canonical owner | Runtime106 的作用 | 本轮不重复登记 |
|---|---|---|---|
| XR combined contract | Runtime66 | 用当前源码重新核验 availability、device/session、view/input/compositor 事实 | RT66-P1-01..72、RT66-P2-01..16 |
| RHI/resource lifetime | Runtime09A | 说明 XR native binding/external image/lease 需要的新增合同 | 通用 RHI handle、graph lifetime、device-loss 父问题 |
| Camera/history | Runtime37 | 区分 camera stack、split viewport 与 XR view group | 通用 camera director/stack/history 父问题 |
| Module/profile truth | Runtime42/65 | 复用 capability/maturity/fallback receipt | 通用 module catalog、quality profile 父问题 |
| Input/platform | Runtime56/57 | 复用 host event/action/cadence 基础 | 普通 keyboard/pointer/gamepad/window lifecycle 父问题 |
| Authoring | Editor29/30 | XR action/origin/rig/preview adapter | 通用 action map 与 camera authoring 父问题 |

固定架构仍是 `zircon_app`、`zircon_runtime`、`zircon_editor` 三个 public root package，runtime 内部遵循 `core/{runtime,framework,manager,math,resource}` spine。XR 不应新增第四个 public root package；backend-neutral XR DTO 应位于 runtime framework，OpenXR loader/provider 与 unsafe native binding 应由 runtime/graphics owner 隔离，Editor 只消费 authoring adapter，App 只组合产品策略。不得通过 compatibility module、裸 native handle re-export 或“两台 camera 并排输出”伪装 hard cutover 已完成。

### 2.2 当前产品源码物理冻结

本轮冻结所有 tracked `.rs/.toml/.wgsl/.json/.ron/.zui/.zr` 产品文本文件，范围为 root `Cargo.toml`、`zircon_runtime*`、`zircon_app`、`zircon_editor`、`zircon_plugins`、`examples`、`templates` 与 `tests`。算法为：repo-relative path 小写排序，逐文件 lowercase SHA-256，以 `path<TAB>hash` 按 LF 连接且末尾无 LF，再计算 manifest SHA-256。

| 文件 | 行 | 非空行 | bytes | test attrs | ignored | Fingerprint |
|---:|---:|---:|---:|---:|---:|---|
| **18,759** | **3,254,992** | **3,062,126** | **113,877,529** | **20,943** | **248** | `83ea13bbc6a61a624086fd4c1921a5f2bda0a287ce56b01fb7ca7be58a41d390` |

冻结对应 HEAD `bee4c707b714738346b49bba15c59468b8bd9b39`、baseline epoch 339。共享树当时为 degraded，协调器报告 2,278 个尚未接受的 workspace changes；本轮不接管、不回滚这些并行修改。结论绑定上述物理快照，任何相关 source drift 都要求在实现前重跑 exact search、字段审查和 fingerprint。

### 2.3 精确搜索与反例隔离

| 搜索 | 当前结果 | 判定 |
|---|---:|---|
| `OpenXR/openxr` | 0 | 无 loader、API、crate、module、profile、asset、Editor 或测试 |
| whole-word `XR` | 0 | 无正式 XR type/product ID；二进制资产中的随机字节不计 |
| `HMD/HeadMounted/head_mounted` | 0 | 无设备或 tracking system contract |
| `foveat*` | 0 | 无 VRS/foveation capability、profile、pipeline 或 receipt |
| OpenXR frame calls / predicted display time | 0 | 无 runtime-owned frame order |
| `LateUpdate` | 7 行 guard/test ledger | 仅断言 Scene 不重建 legacy alias，不是 late-latching 能力 |
| `stereo` | 121 行 / 50 文件 | 全部属于 sound channel/DSP 或 test catalog，不是 stereo view |
| `multiview_mask:` | 122 行 / 95 文件 | 122 个 `None`、0 个 `Some`、0 个其他赋值 |

这组反例很重要：audio stereo、render graph “external texture”、普通 multi-camera、split viewport 和测试中禁止 `LateUpdate` 名称都不能作为 XR 完成度。产品 capability 必须由 typed provider/session/device/view receipt 证明，而不是关键词相似性。

### 2.4 参考引擎物理冻结

聚焦参考共 32 个文件、21,672 行、18,378 非空行、866,379 bytes、14 个 test attributes，fingerprint 为 `044d1e8ddcb8d662308e348fa124dc0ba4e4d40e905237e2d24ce703e46cca4d`，采用与产品冻结相同算法。

| 参考 | 本轮主证据 | 使用边界 |
|---|---|---|
| Unreal | 完整 OpenXR session/frame/swapchain、late update、input 与 extension hooks | 主参考 authority、线程时序、graphics binding；不复制 UObject/RHI 形态 |
| Godot | OpenXR core/action/extension/editor 的可拆分 owner 和较晚 `xrLocateViews` | 主参考 provider/extension/action/editor 分层；不复制 RID/Variant 模型 |
| Unity Graphics | `XRPass/XRSystem` 的 pass/view/layout、late latch、depth motion 与 focused tests | 主参考 renderer integration 和 single/multipass；不是 OpenXR loader 主参考 |
| Bevy | `ManualTextureView` + camera render target landing point | 仅作 external/manual view 接入下限，不宣称 XR lifecycle |
| Fyrox | 聚焦 renderer 和 Rust 树 XR/OpenXR/HMD/foveation/multiview 零命中 | negative boundary，不能据此降低目标 |

## 3. 当前源码逐层事实

### 3.1 Product、module 与 capability truth

`BuiltinRuntimeModuleId` 当前只有 Foundation、Log、Tasks、Time、FrameCount、DiagnosticsCore、Platform、Input、Asset、Scene、Graphics 和 Script；不存在 XR module ID。root `Cargo.toml` 固定 `wgpu = "29.0.1"`，全部 Cargo manifests 对 `openxr` 精确搜索为 0。

runtime profile/plugin catalog 已有值得保留的通用底座：package descriptor 具有 `PluginMaturity`，profile 有 `minimum_maturity` 与 required capabilities，availability projection 会区分 Externalized、Stub、BelowMinimum 并发布 reason。这可承接未来 RT66-P1-03/P1-06 的 truth gate，但当前 catalog 没有任何 XR provider/capability，因此不能把“通用机制存在”写成“XR 部分可用”。

`RenderCapabilityKind::ALL` 当前 19 项覆盖 virtual geometry、hybrid GI、ray tracing、binding arrays、AA、storage/indirect/readback、async、neural、sparse、subgroup 和 pipeline statistics；没有 multiview、external runtime image、variable rate shading/foveation、late-latch 或 XR graphics binding。未来不能只添加一个 `XrSupported: bool`，必须把 runtime availability、RHI device qualification 和 effective feature receipt 分开。

### 3.2 Instance、system、session 与 event authority 完全缺席

当前没有 `XrRuntimeProvider`、loader、instance、API layer/extension negotiation、system/form-factor、view configuration enumeration、environment blend mode、session state machine、event polling、begin/end/loss/restart 或 runtime generation。window focus/occlusion 只是 host/window 事实，不能替代 OpenXR VISIBLE/FOCUSED/STOPPING/LOSS 状态。

因此 Runtime66 的 P1-01..20 全部保持 open。generic module/profile foundation 只减少未来接线成本，没有关闭任何 XR lifecycle gate。

### 3.3 RHI、adapter、graphics binding 与 swapchain

`RenderBackend::new_offscreen` 仍创建 WGPU instance，然后以 `HighPerformance + compatible_surface: None + force_fallback_adapter: false` 自选 adapter，再调用 `request_device`。这条顺序与 OpenXR graphics requirements authority 相反：真实 XR 必须先由 runtime/system 限定 graphics API、physical device/adapter、版本、扩展和 queue，再创建或验证 engine RHI device。

`zr_rhi::RenderDevice` 提供 engine-owned `create_texture/texture_desc/destroy_texture`，没有 native image import、runtime-owned image lease、layout/queue-family ownership、acquire/wait/release 或 retirement generation。`RenderViewportProduct` 刻意不暴露 native texture handle，这对 public runtime/editor 边界是正确的；缺失的是 graphics owner 内部的 typed unsafe bridge，而不是把裸 handle 塞回 public camera DTO。

render graph 的 `import_texture_view/import_borrowed_texture` 接收已经存在的 WGPU `TextureView/Texture` 并按 logical name 绑定。源码中的 “external texture” 指 graph 外部资源，不证明 OpenXR native swapchain image 已被正确创建/导入，也不证明 runtime acquire/wait/release、queue fence 和 image lifetime。Runtime66 P1-21..30 因而全部 open。

### 3.4 Camera stack 不是 XR view family

`ViewportCameraSnapshot` 仍是一份 transform、projection mode、FOV/aspect/near/far、optional projection override、HDR/MSAA、dynamic resolution 和 temporal jitter。它没有 view configuration、view index/count、per-view FOV angles、predicted time、pose validity、space generation、array slice 或 runtime image rect。

`CameraRenderDescriptor` 的 Base/Overlay、stack、target、viewport、clear、culling/volume mask 解决普通 camera composition；它不能表达同一 predicted display time 下由 runtime 返回的多 view。`RenderViewExtract` 虽含 `cameras: Vec<CameraRenderDescriptor>`，但 `select_camera_descriptor` 会把它缩为单一 descriptor；当前 submit loop 仍按 camera sequence 串行提交。两台 camera、Base/Overlay 和 XR projection views 是三种不同 contract，不能复用一个枚举值混合。

`RenderViewFamilyPipeline` 仍只持 resolution plan、spatial/temporal upscaler、output transfer 和 graph phases。名字里的 ViewFamily 不等于已有 `XrViewFamily`；当前没有 shared predicted time、view mask、per-view constants、shared/per-view culling、single/multipass selection、visibility mask 或 quad-view layout。

### 3.5 普通多相机路径的真实进展与严格边界

Runtime66 后 camera loop 有实质进展：多 submission 时会捕获 source state，通过 `Arc::make_mut` 复用 extract，暂存 virtual geometry/hybrid GI payload，并只恢复会被 per-camera submit 修改的 viewport、target、visibility 和 post-process derived state。对应 tests 验证 source extract streaming 与 derived-state restore。这应保留，避免未来 XR 退回整帧深拷贝。

但它仍然是 serial camera loop：每个 descriptor 被选为唯一 camera 后提交，未冻结一个 runtime view group，也没有 shared culling、per-view array slice、multiview PSO 或同代 swapchain lease。它只把 RT66-P1-38 的“粗糙整帧复制”前置成本降低，**没有关闭 P1-38**。

history 也有真实进展。`RenderTemporalHistoryKey` 现在包含 display extent、history viewport position/size、allocation extent 和 upscaler；`ViewportCameraHistoryKey` 包含 entity/order/BaseOverlay/target/viewport/culling layers/volume layers，且有 split viewport、Base/Overlay 和宽 layer set 隔离测试。这能防止普通 viewport/camera 的明显 history 串用。

XR 仍要求 session generation、view configuration、view index、reference-space generation、pose/history generation 与 runtime image generation。现有 key 因此只是 RT66-P1-40 的可复用普通视口基础，**没有关闭 P1-40**。122 个 `multiview_mask: None` 则证明 P1-41 仍完全 open。

### 3.6 Frame pacing 与 App ownership

`RuntimeEntryApp::pump_frame_loop` 的 authority 是 Winit：cadence 决定是否 `session.tick_frame()`，随后处理 host request，再 `window.request_redraw()`。`RuntimeFrameCadence` 依据 continuous/reactive、16.667 ms、headless 16 ms、unfocused 100 ms、background 1 s、focus 和 occlusion 选择 deadline。

OpenXR frame authority 必须来自 `xrWaitFrame` 返回的 predicted display time 与 `shouldRender`，并严格配对 begin/end；mirror window 只能消费已完成的 XR frame，不能反向决定 session cadence。当前 App 路径没有这些 state，Runtime66 P1-31..34 全部 open。

### 3.7 Tracking、space、action 与 haptics

`InputEvent` 当前覆盖 cursor、mouse、keyboard、window status、drag/drop、IME、touch、gamepad 和 two-motor rumble。`InputAction` 只有 string ID、optional context/display name，`InputActionMap` 是 contexts/actions/bindings 集合。

这些是普通 input 的合理基础，但不包含 OpenXR action set/action/path、subaction path、interaction profile、attach/sync lifecycle、changed/active state、XrTime、pose space、valid/tracked flags、hand joints、eye gaze permission 或 action-based haptic frequency/duration/stop。gamepad rumble 不能冒充 XR haptics。Runtime66 P1-47..58 全部 open。

Scene 中唯一 `LateUpdate` 命中来自结构 guard：它禁止重建 legacy alias/shim。这条 hard-cutover discipline 应保留，但 XR late update 需要新的 typed `XrLateUpdateCoordinator`，在 render submission 前重新 locate eligible view/proxy，并同步 history/velocity generation；不能为了复用名字把旧 Scene stage 恢复回来。

### 3.8 Composition、Editor、App 与 qualification

当前没有 projection/depth/motion composition layer、quad/cylinder/equirect layer、native UI layer、visibility mask、color/alpha/order contract、foveation/VRS、refresh/performance setting 或 frame synthesis provider。Editor 没有 XR origin/rig、action/profile、layer、preview、runtime capability inspector；App/examples/templates 没有 XR product policy、packaging/runtime selection、mirror、permission 或 fallback。

20,943 个 test attributes 证明仓库测试规模很大，但 exact XR 名称为 0；现有 camera/history/RHI/App/input tests 不能替代 fake provider、session-state/fault matrix、swapchain-ordering、view-layout pixel oracle、recenter/pose validity、profile change、CTS、device lab、latency、thermal 和 soak。Runtime66 P1-59..72 全部 open。

## 4. 参考引擎交叉证据

### 4.1 Unreal：frame 配对和 runtime authority 不是可选细节

Unreal OpenXR 路径明确拥有 `xrBeginSession/xrEndSession`、`xrWaitFrame`、`xrLocateViews`、`xrBeginFrame` 与 `xrEndFrame`。其源码还专门使用 wait count，防止一次 `xrWaitFrame` 被额外 render pump 消费成两次 `xrBeginFrame`；这说明 frame pairing 是可导致 runtime 永久异常的 correctness invariant，不是普通 redraw optimization。

`OpenXRHMD_Swapchain.cpp` 明确执行 acquire、wait、release；render bridge、swapchain、layer 与 extension plugin 分开承担 graphics binding、image lifecycle、composition layers 和 extension hooks。`LateUpdateManager`/tracking system 则说明 render-time pose update 必须由受控 owner 完成。Zircon 应吸收 authority/invariant，不复制 Unreal 的 UObject、module macro 或具体 RHI class shape。

### 4.2 Godot：core、extension、action 与 editor 必须能独立演进

Godot `openxr_api.cpp` 同样完整执行 swapchain acquire/wait/release、session begin/end、wait/locate/begin/end frame。其注释明确说明更接近 submission 的 `xrLocateViews` 会得到更准确预测，并在 Vulkan command buffer build 后、queue submit 前再次 locate；这为 Zircon late update 的位置提供了直接证据。

Godot 将 action map/interaction profile、foveation、hand tracking、eye gaze、visibility mask、display refresh、performance、frame synthesis、render model 等拆成 extension owner，并提供 action map editor。Zircon 不应先把 vendor extension if/else 写进 camera loop；必须先建 extension registry、provider capability 和 authoring adapter。

### 4.3 Unity Graphics：XR pass/layout 是 renderer 一等结构

Unity `XRPass` 持 view count、per-view matrices/viewport/slice、render target、culling pass 与 foveation状态；`XRSystem` 从 native display subsystem render passes 构建 layout，区分 single/multipass。URP 另有 begin/end late latching 与 depth/motion pass，previous/current per-view matrix 是 motion/space-warp 资格的一部分。

本地 focused tests 验证 empty pass first/last、multipass 数量、single-pass layout 数量、layout stack LIFO/reuse/异常终态。测试规模不大，但至少证明 pass/layout/stack 是可独立测试的 contract；Zircon 当前完全没有相应 XR test seam。

### 4.4 Bevy 与 Fyrox：只采用证据能够支持的结论

当前 Bevy `ManualTextureView` 只保存 `TextureView + size + view_format`，通过 typed handle 供 camera render target 查找。它可以启发 Zircon 将 backend-owned view 映射到 neutral render target identity，但没有 OpenXR provider/session/frame/swapchain ordering；当前源码也没有 OpenXR 注释，不能继续沿用“Bevy 明确为 OpenXR 提供 landing point”的旧措辞。

Fyrox 聚焦 Rust renderer/source 对 XR/OpenXR/HMD/foveation/multiview 为零命中，只能作为 negative boundary。Zircon 的工程目标来自 Unreal/Godot/Unity 的成熟结构和用户设定，不能以 Fyrox/Bevy 当前缺少完整 XR 为降低标准的理由。

## 5. Runtime66 问题状态刷新

| Runtime66 owner | 当前状态 | 本轮证据 | 可复用基础但不构成关闭 |
|---|---|---|---|
| P1-01..10 Product/provider/module | **Open** | exact zero、无 Cargo/module/profile/product entry | generic plugin maturity/capability/availability reason |
| P1-11..20 Instance/system/session | **Open** | 无 loader/instance/system/session/event/state | platform/runtime module lifecycle 可供组合 |
| P1-21..30 RHI/swapchain/mirror | **Open** | engine 自选 adapter、无 native binding/lease | backend-neutral viewport product、WGPU graph import |
| P1-31..46 Frame/view/render | **Open** | Winit cadence、单 camera descriptor、122 mask 全 None | viewport history identity、source-state streaming camera loop |
| P1-47..58 Space/input/extensions | **Open** | 无 space/action/profile/pose/hand/eye/haptic | generic action map、gamepad/input host |
| P1-59..64 Compositor/foveation/performance | **Open** | 无 layer graph、VRS、refresh、space warp | 通用 render graph、quality/profile framework |
| P1-65..72 Authoring/product/evidence | **Open** | Editor/App/package/tests exact zero | generic Editor29/30、App profile、test infrastructure |
| P2-01..16 Discipline/maintainability | **Open** | 尚无 XR 类型可验证术语、native/time/coordinate/extension纪律 | Runtime24 generation/hard-cutover 规则可复用 |

本轮不新建 issue ID。后续实现必须更新 Runtime66 对应行的 evidence/status；不得只在 Runtime106 写“完成”而让 canonical owner 保持旧状态。

## 6. 目标架构与当前 refactor 落点

Runtime66 的目标链保持有效：

```text
Project/Profile policy
        |
        v
XrRuntimeProvider --> XrExtensionRegistry --> XrInstanceAuthority --> XrSystemProfile
                                                                  |
                                                        graphics requirements
                                                                  v
RHI device owner <--> XrGraphicsBindingBridge --> XrSessionSupervisor
                                                  | event/state/generation
                 +--------------------------------+--------------------+
                 v                                v                    v
           XrFramePacer                     XrSpaceGraph         XrActionRuntime
      wait/begin/end/time              locate/recenter       sets/profile/haptic
                 +--------------------------------+--------------------+
                                                  v
                                            XrViewFamily
                                 views/culling/history/multiview
                                                  v
                                      XrLateUpdateCoordinator
                                                  v
                         render + XrSwapchainLease + layer graph
                                                  v
                                xrEndFrame + mirror + diagnostics
```

结合当前源码，重构落点必须满足以下边界：

1. `core::framework` 只放 backend-neutral provider/session/view/space/action/layer DTO、stable identity、generation 与 typed outcomes；不得依赖 WGPU/OpenXR raw handles。
2. loader/provider/session supervisor 属于 runtime service/module owner，必须通过 Runtime42 profile/capability truth 接入，不在 App event loop 散调 OpenXR。
3. graphics/RHI owner 独占 unsafe native binding、external image import、queue/fence/layout 和 swapchain lease；public `RenderViewportProduct` 继续保持 opaque。
4. `XrViewFamily` 与普通 `CameraRenderDescriptor` 分离；可共享 scene extract、visibility、history allocator 和 render phases，但不能共享错误 identity。
5. current camera loop 的 source-state streaming 可以成为 shared extract 输入，XR submission 必须一次冻结 view group并显式提供 per-view constants/slices，而非循环调用 `select_camera_descriptor`。
6. current history key 继续服务普通 camera；XR 在其上组合 session/view/space/pose generation，禁止把 view index 偷塞进 camera entity/order。
7. App frame loop 在 XR mode 下消费 `XrFramePacer` demand；Winit redraw 只负责 mirror/Editor surface，window occlusion不得暂停 runtime-required XR frame finalization。
8. InputActionMap 可以成为 authoring入口，但 OpenXR action/action set/subaction/profile/space lifecycle必须有独立 compiled artifact和runtime receipt。
9. Editor 只编辑 XR origin/action/layer/product policy并展示 effective capability；不能拥有 live session或 native swapchain。
10. 无 provider/device/runtime/permission 时必须 Unavailable/fail-close；普通 window fallback必须是显式 product policy，不能静默冒充 XR success。

## 7. 依赖顺序与门禁状态

继续沿用 Runtime66 `XR-M0` 至 `XR-M9`，不创建平行 milestone。当前状态如下：

| Milestone | 当前状态 | 下一步必须先完成 |
|---|---|---|
| XR-M0 Truth freeze | **Review evidence refreshed，implementation 未接受** | 固化 Disabled-safe product policy、capability ID、zero-runtime fake provider |
| XR-M1 Provider/instance/system | **Not started** | loader/version/extension/system/fault matrix |
| XR-M2 Graphics binding | **Not started** | runtime requirements 约束真实 adapter/device 的 typed bridge |
| XR-M3 Session/frame/swapchain | **Not started** | event/state、wait/begin/end、lease/fence/loss/restart |
| XR-M4 View family/render | **Not started** | per-view DTO、single/multipass、shared culling、history/mask |
| XR-M5 Space/late update | **Not started** | spaces、validity/recenter、submission-time locate、history/velocity |
| XR-M6 Action/input | **Not started** | action/profile/subaction/pose/haptic、hand/eye permission |
| XR-M7 Composition/quality | **Not started** | layers、depth/motion、foveation、refresh/performance receipt |
| XR-M8 Product/editor/cook | **Not started** | App/Editor/schema/package/runtime selection/mirror/fallback |
| XR-M9 Qualification | **Not started** | fake/fault、CTS、device lab、capture、latency、thermal/soak/benchmark |

Runtime66 的 48 个 gate 当前 **0 个通过**。普通 history tests、camera loop tests、generic capability tests 只能作为未来 gate 的 prerequisite；它们没有 provider/session/device/view evidence，不能把任何 XR-G gate标记为 pass。

禁止的捷径保持不变：M1 前不得画两个 camera 叫 XR；M2 前不得让 engine 自选 device 后再要求 runtime 接受；M3 前不得把 Winit redraw 包装成 frame pacing；M4 前不得只把一个 `multiview_mask` 改成 `Some` 就宣称 single-pass；M9 前不得宣称达到或超过 Unreal。

## 8. 性能与表现超越基线

未来“优于当前 Unreal”至少需要同设备、同 runtime、同 refresh、同 view configuration、同分辨率/画质/场景/interaction profile 下比较：

- CPU：wait-to-submit、game/render/RHI thread、action sync、late locate、layer build 的 p50/p95/p99；
- GPU：per-view/shared culling、single/multipass、visibility mask、foveation、depth/motion、post/UI、mirror 的 pass 时间与带宽；
- 时序：motion-to-photon、pose age、missed/dropped/reprojected frame、wait/begin/end pairing violations；
- 质量：双眼/quad view 像素 oracle、边缘/中心清晰度、temporal stability、depth/motion/space-warp correctness、color/alpha/layer ordering；
- 稳定性：runtime/session/device loss、recenter/profile change、permission、sleep/resume、长时 thermal/soak 和 stale generation；
- 产品：package/install/runtime selection、Editor preview、mirror、accessibility/comfort、diagnostic receipt 可解释性。

必须报告均值、尾延迟、方差、场景、硬件、runtime/driver、validation/capture 状态和 fallback reason。没有这些 evidence 时，“更快”“更稳”“表现更好”一律视为未证明。

## 9. 本轮验证边界与下一次复核

本轮验证覆盖：HEAD/epoch、共享树 degraded 状态、18,759 文件物理计数/指纹、tracked 精确零命中、Cargo dependency、122 个 multiview assignment 分类、逐字段阅读 module/profile/camera/view/history/RHI/graph/App/input、五参考引擎交叉阅读、frontmatter 路径、索引链接、Markdown 与 `git diff --check`。

本轮没有运行 Cargo、真实 WGPU、Editor/App、OpenXR runtime、头显、GPU capture 或性能测试，因为没有改生产代码且当前产品没有可执行 XR 路径。后续进入实现前必须重新冻结 source fingerprint、coordinator leases、exact-zero、multiview assignment、adapter creation、history fields 和 canonical Runtime66 status；任一漂移都要求重新审查受影响结论。
