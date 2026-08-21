---
title: Runtime XR/OpenXR、Device/Session、Stereo View、Tracking/Input、Late Update、Foveation、Compositor 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime66
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - Cargo.toml
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
  - zircon_editor/Cargo.toml
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/core/framework/platform
  - zircon_runtime/src/core/runtime/modules
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/src/graphics/backend/render_backend
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
  - zircon_app/src/entry/runtime_entry_app
  - zircon_editor/src/ui/retained_host/viewport
tests:
  - zircon_runtime/src/core/framework/render/backend_types/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop/tests/frame.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/tests/native_submission.rs
  - zircon_runtime/tests/hybrid_gi_m4_source_ledger_wgpu.rs
  - zircon_runtime/tests/m1_runtime_editor_boundary_contract.rs
  - zircon_editor/src/ui/retained_host/viewport/tests/fake_render_framework.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/56-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/58-runtime-plugin-interface-bridge-slot-generation-strong-weak-native-vm-lifecycle-diagnostics-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/OpenXR.uplugin
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/OpenXRHMD.Build.cs
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Internal/OpenXRHMD_RenderBridge.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Internal/OpenXRHMD_Swapchain.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/FBFoveationImageGenerator.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/FBFoveationImageGenerator.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_Layer.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_Layer.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_RenderBridge.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD_Swapchain.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMD.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Private/OpenXRHMDModule.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Public/IOpenXRExtensionPlugin.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRHMD/Public/OpenXRPlatformRHI.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRInput/OpenXRInput.Build.cs
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRInput/Private/OpenXRInput.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/OpenXR/Source/OpenXRInput/Private/OpenXRInput.h
  - dev/UnrealEngine/Engine/Source/Runtime/HeadMountedDisplay/Private/LateUpdateManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/HeadMountedDisplay/Public/LateUpdateManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/HeadMountedDisplay/Public/IXRTrackingSystem.h
  - dev/godot/modules/openxr/openxr_api.cpp
  - dev/godot/modules/openxr/openxr_api.h
  - dev/godot/modules/openxr/openxr_interface.cpp
  - dev/godot/modules/openxr/openxr_interface.h
  - dev/godot/modules/openxr/action_map/openxr_action_map.cpp
  - dev/godot/modules/openxr/action_map/openxr_action_map.h
  - dev/godot/modules/openxr/action_map/openxr_action.cpp
  - dev/godot/modules/openxr/action_map/openxr_action.h
  - dev/godot/modules/openxr/action_map/openxr_interaction_profile.cpp
  - dev/godot/modules/openxr/action_map/openxr_interaction_profile.h
  - dev/godot/modules/openxr/extensions/openxr_fb_foveation_extension.cpp
  - dev/godot/modules/openxr/extensions/openxr_fb_foveation_extension.h
  - dev/godot/modules/openxr/extensions/openxr_hand_tracking_extension.cpp
  - dev/godot/modules/openxr/extensions/openxr_hand_tracking_extension.h
  - dev/godot/modules/openxr/extensions/openxr_eye_gaze_interaction.cpp
  - dev/godot/modules/openxr/extensions/openxr_eye_gaze_interaction.h
  - dev/godot/modules/openxr/extensions/openxr_visibility_mask_extension.cpp
  - dev/godot/modules/openxr/extensions/openxr_visibility_mask_extension.h
  - dev/godot/modules/openxr/extensions/openxr_fb_display_refresh_rate_extension.cpp
  - dev/godot/modules/openxr/extensions/openxr_fb_display_refresh_rate_extension.h
  - dev/godot/modules/openxr/extensions/platform/openxr_vulkan_extension.cpp
  - dev/godot/modules/openxr/extensions/platform/openxr_vulkan_extension.h
  - dev/godot/modules/openxr/extensions/platform/openxr_d3d12_extension.cpp
  - dev/godot/modules/openxr/extensions/platform/openxr_d3d12_extension.h
  - dev/godot/modules/openxr/scene/openxr_composition_layer.cpp
  - dev/godot/modules/openxr/scene/openxr_composition_layer.h
  - dev/godot/modules/openxr/editor/openxr_action_map_editor.cpp
  - dev/godot/modules/openxr/editor/openxr_action_map_editor.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRBuiltinShaderConstants.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRGraphicsAutomatedTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRLayout.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRLayoutStack.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRMirrorView.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XROcclusionMesh.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRSRPSettings.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRSystem.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRView.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/XR/XRVisibleMesh.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRLayoutStackTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRLayoutTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/XR/XRPassTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/XRDepthMotionPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/XROcclusionMeshPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/XR/XRPassUniversal.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/XR/XRSystemUniversal.cs
  - dev/bevy/crates/bevy_camera/src/camera.rs
  - dev/bevy/crates/bevy_render/src/view/mod.rs
  - dev/bevy/crates/bevy_render/src/view/window/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 66 · Runtime XR/OpenXR、Device/Session、Stereo View、Tracking/Input、Late Update、Foveation、Compositor 与 Product Integration 工程化差距

## 1. 结论

Zircon当前没有XR/OpenXR产品能力。这里的“没有”不是指缺少一个可选crate或一个stereo开关，而是loader/instance/system/session、event/state、graphics binding、runtime-owned swapchain、`wait/begin/locate/render/end`时序、reference space、tracked pose、action/interaction profile、late update、composition layer、visibility mask、foveation、mirror view、asset/editor/cook以及conformance evidence整条链均未建立。生产与样例精确搜索也没有`openxr`、XR session/system/swapchain/action、headset、hand/eye tracking或foveation产品标识。

现有渲染底座不是零：camera snapshot、camera stack、`RenderViewExtract`、名为ViewFamily的分辨率计划、temporal history、RHI capability、WGPU backend、window surface、input action和gamepad rumble都可作为未来接入点。但这些结构仍是单观察点、普通窗口present和传统2D input合同。`RenderViewExtract.cameras`表示按顺序提交的camera stack，不表示同一预测显示时刻的多视图；`RenderViewFamilyPipeline`只描述resolution/upscaler/output transfer；119处可识别`multiview_mask`赋值全部为`None`。更关键的是，WGPU backend自行选择`compatible_surface: None`的高性能adapter后请求device，无法证明它与OpenXR runtime要求的graphics device、queue、extension和swapchain image是同一权威。

本轮登记 **0项新增P0、72项P1、16项P2和48项验收门禁**。当前产品没有宣称或默认启用XR，因而不把“能力缺失”虚构为数据损坏或安全级P0；一旦任何产品profile把XR标为Available/Enabled而仍走普通window/camera路径，Runtime42的capability truth gate必须fail-close。目标架构是`XrRuntimeProvider + XrInstanceAuthority + XrSystemProfile + XrSessionSupervisor + XrFramePacer + XrSpaceGraph + XrActionRuntime + XrGraphicsBindingBridge + XrSwapchainLease + XrViewFamily + XrLateUpdateCoordinator + XrCompositionLayerGraph + XrExtensionRegistry + XrDiagnosticsReceipt`。

本轮只做静态review与计划记录，没有修改production、tests、Cargo、ABI或参考源码，也没有运行Cargo、真实OpenXR runtime、头显、GPU capture、conformance、motion-to-photon、长时热功耗或竞争性基准，因此不能宣称已具备XR，更不能宣称性能或表现超过Unreal。用户已要求暂停tooling优化，本篇不新增脚本、生成器或tooling迁移任务。

## 2. 审查边界、规模与currentness

### 2.1 Zircon物理冻结

| 冻结组 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Manifest、public render contract与builtin module组合 | 281 | 50,983 | 1,781,220 |
| Frame/view、camera submission、history与全部命中multiview consumer | 167 | 32,745 | 1,268,812 |
| RHI、WGPU、device、surface与presentation | 94 | 24,822 | 855,380 |
| App host、platform/input与Editor viewport product consumer | 143 | 10,436 | 359,322 |
| 聚焦external/inline-test-bearing文件 | 34 | 14,695 | 547,513 |
| 去重合计 | **719** | **133,681** | **4,812,247** |

Zircon冻结集fingerprint为SHA-256 `36aa70c3d7c4ef5ea42b86a71034c6804533c05f1e999d0ed55cd720547f4c10`。算法与Runtime65一致：相对路径转`/`、排序去重，以`path|lowercase per-file SHA-256`逐行编码，LF连接且末尾不追加LF，再对UTF-8 payload计算SHA-256。冻结选择覆盖四份顶层manifest、render public contract、builtin module、frame submission/history、RHI/WGPU/surface、App host/input、Editor viewport，以及所有包含`multiview_mask`的相关Rust文件和聚焦外部测试。

冻结时有101个入选working-tree路径带修改标记，主要集中在render public contract、camera stack/frame extract、post-process与environment、WGPU GPU timer、App cadence、Editor viewport tests和少量外部render测试。当前结论按共享working copy冻结；实现前必须重验这些路径、119处`multiview_mask`赋值、Cargo dependency/feature和本fingerprint，不能把本篇当作静态永真清单。

### 2.2 参考物理冻结

| 参考 | 文件 | 行 | bytes |
|---|---:|---:|---:|
| Unreal Engine OpenXR/HMD/Input/Late Update | 21 | 12,058 | 465,760 |
| Godot OpenXR core/action/extensions/editor | 28 | 13,094 | 531,416 |
| Unity Graphics Core/URP XR integration/tests | 18 | 3,140 | 131,307 |
| Bevy external view landing points | 3 | 3,096 | 122,339 |
| Fyrox renderer negative boundary | 1 | 1,470 | 57,444 |
| 合计 | **71** | **32,858** | **1,308,266** |

参考集fingerprint为SHA-256 `70653576bda8cd2131fd7009f477654ba07537d22ba1ada2dc5e2c7d2888aa92`，采用同一算法。Unreal是完整XR system、线程时序和input主参考；Godot是OpenXR core/extension/action/editor模块化主参考；Unity Graphics是single/multipass、XRPass、foveation、occlusion、mirror和late-latch渲染集成主参考。Bevy仅证明`ManualTextureView`可承接外部OpenXR texture view；Fyrox聚焦语料零命中，不能把它包装成成熟XR参考，也不能据此降低Zircon目标。

### 2.3 本轮拥有与明确不拥有

- Runtime66拥有XR combined contract：provider/instance/system/session、frame order、graphics binding、swapchain lease、view family、space/action、late update、composition layer、extension/capability receipt与runtime产品接入。
- Runtime09A继续拥有通用adapter/device/queue、GPU object lifetime、submission/completion、device loss和generation；Runtime66规定OpenXR graphics requirements如何约束并绑定该同一device generation。
- Runtime09B拥有通用visibility/GPU scene；Runtime66拥有per-view frustum、view mask、single/multipass组织与XR visibility/occlusion mesh消费。
- Runtime09H1拥有通用history、velocity、TAA/upscaler；Runtime66拥有view configuration/index、predicted display time、pose generation进入history key，以及late-latch后的history一致性。
- Runtime37拥有普通camera/rig/director/cut；Runtime66把玩家camera意图解析为head/reference-space和逐视图FOV/pose，不能把两个camera stack entry冒充双眼。
- Runtime56拥有通用physical/input action/frame state；Runtime66拥有OpenXR action set、subaction path、interaction profile、pose action和XR haptics adapter。
- Runtime57拥有普通window/display/event loop/application lifecycle；Runtime66拥有runtime-driven frame pacing、session focus/visibility和mirror window桥。
- Runtime24拥有通用stable identity/generation；Runtime42拥有module/catalog/profile/capability truth；Runtime07/58拥有通用plugin/bridge lifecycle。Runtime66消费这些合同，不复制父owner。
- Runtime65拥有通用quality/device profile、dynamic resolution与frame budget；Runtime66拥有XR view configuration、refresh/foveation/performance setting如何进入该统一policy。
- Editor29/30与后续XR authoring拥有控件、asset编辑和preview；Runtime66定义action map、XR origin/view/layer等runtime schema。用户已暂停tooling优化，loader packaging脚本和生成器不在本篇实施范围。

## 3. 当前实现的真实能力与断裂

### 3.1 Camera、ViewFamily与history不是XR view family

`ViewportCameraSnapshot`只有一个transform、projection mode/FOV、aspect、MSAA、固定dynamic-resolution scale和一份jitter。`CameraRenderDescriptor`再增加Base/Overlay、stack、target、viewport、clear和layer mask。`RenderViewExtract`的`Vec<CameraRenderDescriptor>`由scene camera排序产生，选中descriptor时又把`cameras`缩成单元素；camera loop逐项替换共享extract中的camera、串行提交并恢复。它适合普通camera stack，却没有view configuration、view index、per-eye FOV/pose、array slice、predicted time、pose validity或同帧共享culling pass。

`RenderViewFamilyPipeline`这个名字目前只覆盖display/primary/secondary extent、upscaler和output transfer。history key只含entity、order、render type、target、viewport和layer，没有XR view configuration/index、session generation、space generation、predicted display time或pose generation。若直接复用，双眼/quad view可能互串history，recenter/session restart也无法可靠失效。

### 3.2 RHI与present没有OpenXR graphics authority

`RenderBackend::new_offscreen`创建WGPU instance，以`compatible_surface: None`自行请求高性能adapter，再用通用feature/limit调用`request_device`。`RenderBackendCaps`与19项`RenderCapabilityKind`没有XR graphics binding、external swapchain image import、multiview/VRS/foveation或runtime-required adapter identity。`viewport_surface.rs`只创建普通WGPU surface并blit；`zr_rhi`的native surface目前只公开Win32窗口target。

这意味着未来不能仅在上层“拿到OpenXR image后画进去”。OpenXR runtime会规定graphics API、minimum API version、physical device/adapter和graphics binding，交换链image还有acquire/wait/release顺序及native layout/queue ownership。`XrGraphicsBindingBridge`必须位于RHI边界：允许qualified WGPU interop，也允许D3D12/Vulkan等native backend路径；不能要求OpenXR接受Zircon先随机选出的WGPU device，更不能把opaque native handle散落进camera代码。

### 3.3 Host cadence与input仍是普通窗口模型

App frame loop由window focus/occlusion和固定16.67 ms等cadence决定，先`session.tick_frame()`再请求window redraw；device event只把raw pointer motion视为reactive frame需求。OpenXR要求session event pump与`xrWaitFrame`给出predicted display time/`shouldRender`，然后有序begin、locate、render、end。普通window redraw只能作为mirror consumer，不能成为XR compositor时钟owner。

Input已有keyboard/pointer/touch/gamepad、字符串action map、frame snapshot和两马达rumble，这些是可保留的通用底座；但没有action set/subaction path/interaction profile、Boolean/Float/Vector2/Pose action、reference/action space、tracked/valid flag或OpenXR haptic action。把controller pose压成gamepad axis会丢失空间、时刻、设备代际和追踪置信语义。

### 3.4 产品、编辑器、构建与测试均无闭环

Cargo只有WGPU/Winit/Gilrs等依赖和平台feature，没有OpenXR loader/dependency/feature。builtin module、project profile、App startup、Editor viewport、样例配置和asset中也没有XR system/session/action/layer schema。没有runtime availability receipt、OpenXR extension policy、headless/mock provider、action map editor、XR origin/camera rig、mirror/spectator设置、platform loader packaging、权限/隐私策略或runtime selection UI。

生产与插件范围精确搜索得到122处`multiview_mask`命中，其中119处实际赋值全部为`None`；本轮重点graphics/RHI冻结子集内112处赋值同样全部为`None`。现有测试覆盖普通camera stack、surface和native submission，却没有OpenXR fake runtime、session transition、swapchain lease、multi-view pixel/history、late update、action profile或Khronos conformance evidence。

## 4. 五套参考实现的语义差异

| 参考 | 已验证结构 | Zircon应吸收 | 不照搬 |
|---|---|---|---|
| Unreal | `OpenXRHMD.cpp`有wait/begin/end、locate views、predicted time、`shouldRender`和READY/STOPPING/LOSS_PENDING；swapchain在RHI thread acquire/wait/release；LateUpdate在render thread改scene proxy；Input建立action set/action/binding/sync/pose/haptics | 分线程frame state、一次wait对应一次begin、graphics/runtime同device、swapchain lease、late update、action/interaction profile及明确session loss | 不照搬CVar、全局模块或宏式错误处理，也不以类数量作为性能证据 |
| Godot | OpenXR core把frame/session/swapchain闭环；extension wrapper分别管理foveation、hand、eye gaze、visibility mask、display refresh和Vulkan/D3D12；action map与Editor是显式资源 | 小核心+扩展注册表、extension availability receipt、typed action asset、权限/capability、composition layer和跨APIgraphics bridge | 不照搬singleton/RenderingServer形状或每个平台宏布局 |
| Unity Graphics | `XRSystem`从native render pass构造layout；`XRPass/XRView`携view count、矩阵、viewport、slice、occlusion、foveation并选择single/multipass；URP显式mark/unmark late-latch矩阵 | 把XR视为一等render pass/view family、支持single/multipass fallback、quad view、occlusion/mirror、per-view constants和late-latch边界 | Unity源码只覆盖Graphics层，不足以替代OpenXR instance/session/action owner |
| Bevy | camera target可引用外部创建的`ManualTextureView`，注释明确举OpenXR；view/window保持外部target落点 | Rust侧使用typed external texture view/owner generation，而不是让业务层传裸native pointer | 没有first-party完整OpenXR system，不能把一个landing point外推成产品能力 |
| Fyrox | 本轮418个聚焦Rust/TOML文件对OpenXR/XR session/headset/foveation为零命中，renderer仍是普通窗口/scene路径 | 只用于证明Rust引擎也必须主动设计边界；“同为Rust”不会自动解决XR lifecycle | 不把缺失能力当作目标上限，也不虚构未观察到的XR架构 |

共同规律是XR runtime拥有显示时钟、view configuration和swapchain；engine拥有world/render/input/product composition，但必须以generation、lease和ordered receipt连接两边。高性能来自减少重复culling/submission、正确late update、合格multiview/foveation和稳定资源生命周期，而不是跳过session状态、忽略`shouldRender`或把两只眼串行当作两个普通camera。

## 5. P0审计

本轮新增P0为0。当前manifest、module catalog、App和Editor没有把XR标为可用，也没有默认进入伪XR路径；因此“尚未交付一项大型能力”按P1工程缺口登记。以下情况一旦出现，应升级并路由到既有P0 owner：产品对用户宣称XR Available但没有真实provider/artifact/runtime generation；OpenXR交换链image被越序释放或跨device使用；late pose/history错误导致已启用产品稳定输出错误；权限受限的eye/hand数据被无授权持久化。当前源码没有足够证据宣称这些失败已发生。

## 6. P1工程化差距

| ID | 差距 | 当前证据/风险 | 重构要求 |
|---|---|---|---|
| RT66-P1-01 | 无XR product contract | manifest/module/App均无XR profile | 建立`XrProductPolicy`，区分Required/Preferred/Disabled与fallback |
| RT66-P1-02 | 无provider boundary | 没有OpenXR/mock/vendor provider trait | 建立可替换`XrRuntimeProvider`，核心不直接散用OpenXR函数 |
| RT66-P1-03 | 无builtin module装配 | module catalog没有XR capability | 通过Runtime42登记provider、feature、target、load receipt |
| RT66-P1-04 | 无loader/dependency | Cargo无OpenXR loader或feature | 明确dynamic/static loader、版本、平台和发行边界 |
| RT66-P1-05 | 无project/cook schema | 项目与资产无XR配置 | 定义versioned view/action/layer/extension product schema |
| RT66-P1-06 | 无availability receipt | 无法解释runtime/device/extension为何不可用 | 发布requested/resolved/reason/provider/runtime generation |
| RT66-P1-07 | 无XR stable identity/generation | session/space/action/swapchain均无handle合同 | 采用Runtime24 scoped handle、owner epoch和stale拒绝 |
| RT66-P1-08 | 无world/session ownership | 不清楚多World、PIE、mirror谁拥有session | 每product session唯一supervisor，world只持lease |
| RT66-P1-09 | 无extension registry | feature只能未来硬编码 | 建立requires/conflicts/order/pNext/capability registry |
| RT66-P1-10 | 无product entry/fallback | App/Editor没有启动、退出和普通窗口回退 | 统一bootstrap、fail-close、last-good与显式fallback |
| RT66-P1-11 | 无instance lifecycle | 无create/destroy、API version、application info | `XrInstanceAuthority`拥有loader、instance和debug messenger |
| RT66-P1-12 | 无extension/API layer协商 | 不能区分required与optional | 先枚举再解析，保留缺失原因和启用列表hash |
| RT66-P1-13 | 无system/form-factor选择 | 没有HMD system或blend mode事实 | 枚举system、form factor、environment blend和properties |
| RT66-P1-14 | 无view configuration枚举 | 固定单camera | 支持primary stereo、mono、quad及runtime推荐extent/sample |
| RT66-P1-15 | 无device capability snapshot | RHI caps不含XR facts | 建立绑定instance/system/runtime generation的typed profile |
| RT66-P1-16 | 无session state machine | 没有READY/RUNNING/FOCUSED/STOPPING/LOSS | supervisor消费event并执行合法状态转换 |
| RT66-P1-17 | 无begin/end/loss recovery | session重启和runtime loss无owner | ordered start/stop、instance-loss、restart与last-good策略 |
| RT66-P1-18 | 无focus/visibility/presence语义 | 只看window focus/occlusion | XR session state驱动input、simulation、audio和render demand |
| RT66-P1-19 | 无多实例/多设备策略 | process-global假设将污染PIE和测试 | 定义单runtime多session限制、mock isolation与冲突receipt |
| RT66-P1-20 | 无runtime failure taxonomy | loader/runtime/device/permission混成不可用 | typed terminal/retryable/degraded error与operator guidance |
| RT66-P1-21 | adapter由WGPU自行选择 | `compatible_surface: None`高性能adapter | 先取得OpenXR graphics requirements/physical device再创RHI |
| RT66-P1-22 | 无graphics requirements校验 | 未校验API版本、extension、queue family | bridge验证minimum/maximum API与required native extensions |
| RT66-P1-23 | 无graphics binding | session create无D3D12/Vulkan等binding | RHI提供typed native binding，禁止camera持裸handle |
| RT66-P1-24 | 无XR swapchain协商 | 只有普通surface/offscreen texture | 枚举format、usage、sample、array size并记录选择reason |
| RT66-P1-25 | 无acquire/wait/release lease | 无runtime-owned image状态机 | `XrSwapchainLease`强制一次acquire/wait/write/release终态 |
| RT66-P1-26 | 无depth/motion swapchain链 | 不能提交depth或space-warp motion | 每layer声明color/depth/motion能力与同代extent/view mapping |
| RT66-P1-27 | 无external image import | WGPU texture由engine创建 | RHI封装native image/view、layout、ownership与retirement |
| RT66-P1-28 | 无queue/thread/fence同步 | 普通queue submit不足以证明runtime可见 | 定义RHI/render thread边界、barrier、completion和release顺序 |
| RT66-P1-29 | 无device/session/swapchain恢复图 | resize与device loss只面向window | generation化重建并失效所有旧lease/view/history |
| RT66-P1-30 | 无mirror/spectator桥 | XR compositor与window presenter未区分 | mirror只消费已解析layer/view，不拥有XR frame cadence |
| RT66-P1-31 | 无`wait/begin/end` frame authority | App只tick后request redraw | `XrFramePacer`严格配对wait/begin/end并处理discard |
| RT66-P1-32 | cadence由普通窗口决定 | 固定16.67 ms/focus/occlusion | predicted display time驱动XR帧，window cadence仅服务mirror |
| RT66-P1-33 | 无predicted-time采样 | transform/input没有显示时刻 | simulation、locate views/spaces和receipt绑定同一XrTime |
| RT66-P1-34 | 无`shouldRender`分支 | 每次tick都请求redraw | false时仍合法end frame但禁止制造陈旧layer |
| RT66-P1-35 | camera snapshot是单view | 一份transform/FOV/projection | `XrView`携pose/FOV/matrix/viewport/slice/validity |
| RT66-P1-36 | descriptor无view identity | camera stack字段不能表达双眼 | 加view configuration、view index、view count和group ID |
| RT66-P1-37 | Base/Overlay不能表达XR pass | render type只有两值 | XR pass独立于camera overlay，layer组合走composition graph |
| RT66-P1-38 | camera loop串行改共享extract | 两眼会重复构建并易串状态 | 一次frame冻结view family，复用scene extract并显式per-view数据 |
| RT66-P1-39 | ViewFamily名实不符 | 只有resolution/upscaler/output | 扩展为`XrViewFamily`，普通view family保留清晰边界 |
| RT66-P1-40 | history key缺XR维度 | 双眼/recenter/session restart可能串history | 纳入session/view config/index/space/pose/history generation |
| RT66-P1-41 | multiview全部关闭 | 119处赋值均为`None` | pipeline/PSO以view mask、array slice和capability形成variant |
| RT66-P1-42 | 无per-view/shared culling合同 | camera逐个提交 | 支持shared culling pass与per-view frustum/LOD/occlusion差异 |
| RT66-P1-43 | 无single/multipass fallback | 不能按capability降级 | typed选择single-pass multiview/instancing或multipass并回执 |
| RT66-P1-44 | 无visibility/occlusion mask | 浪费隐藏区域且无更新事件 | 消费runtime per-view mask，generation化更新GPU mesh |
| RT66-P1-45 | 无quad/foveated view layout | 假设一个rect和一个extent | 支持多render pass、inner/outer view与recommended image rect |
| RT66-P1-46 | HDR/motion/UI无per-view合同 | 普通terminal chain只有单输出 | 明确每pass color/depth/velocity/post/UI和array slice语义 |
| RT66-P1-47 | 无reference space graph | 没有VIEW/LOCAL/STAGE/LOCAL_FLOOR | `XrSpaceGraph`管理base space、offset、validity和lifecycle |
| RT66-P1-48 | 无pose validity/tracked flag | transform默认被视为有效 | 区分position/orientation valid/tracked及last-good policy |
| RT66-P1-49 | 无recenter/origin change | history/physics/audio不会同步 | 原子发布space generation并通知camera/input/history consumers |
| RT66-P1-50 | 无late update | game snapshot后pose不再修正 | render提交前relocate，受控更新view与eligible scene proxies |
| RT66-P1-51 | 无head/controller/hand space ownership | 无设备空间及销毁顺序 | action/device space随session和binding generation创建/退休 |
| RT66-P1-52 | 无OpenXR action lifecycle | 通用字符串action map不能attach/sync | action set/action/path/binding建议/attach/sync有序闭环 |
| RT66-P1-53 | 无interaction profile | controller型号差异无法解析 | profile metadata、localized source、binding provenance与change event |
| RT66-P1-54 | input event不携pose/action identity | 只有keyboard/pointer/touch/gamepad | 添加typed XR sample，保留time/subaction/source/generation |
| RT66-P1-55 | gamepad rumble冒充不了XR haptics | 两马达+毫秒不含action/path/frequency | XR haptic adapter支持duration/frequency/amplitude/stop与focus gate |
| RT66-P1-56 | 无hand tracking | 无joint set、radius、velocity、confidence | extension provider发布bounded joint snapshot和permission state |
| RT66-P1-57 | 无eye gaze | 无gaze pose、permission或fallback | 显式capability/permission、validity、privacy和foveation consumer |
| RT66-P1-58 | 无tracker/render model/spatial extension边界 | 扩展会侵入核心 | 通过extension registry发布tracker/model/anchor optional service |
| RT66-P1-59 | 无composition layer graph | 只能把scene blit到window | `XrCompositionLayerGraph`生成projection/quad/cylinder/equirect层 |
| RT66-P1-60 | 无native UI layer策略 | UI只能进入scene texture | 支持head/world-locked layer、eye visibility、filter和fallback |
| RT66-P1-61 | 无layer order/alpha/color contract | compositor提交无法验证 | 稳定sort、blend flags、premultiply、color space和lifetime |
| RT66-P1-62 | 无foveation/VRS闭环 | capability/profile/pipeline均无此维度 | 解析fixed/eye-tracked模式、swapchain chain、shader/VRS fallback |
| RT66-P1-63 | 无refresh/performance setting | Runtime65无法消费XR display状态 | refresh、CPU/GPU level、thermal结果进入统一quality/budget receipt |
| RT66-P1-64 | 无frame synthesis/space warp合同 | motion/depth/pose时序缺失 | 作为optional provider，先定义输入资格和artifact/generation |
| RT66-P1-65 | 无XR origin/rig scene schema | 普通camera无法表达tracking origin | runtime定义origin/head/controller anchor，Editor30负责authoring |
| RT66-P1-66 | 无action map asset/editor | 通用map不含profile/subaction/pose | versioned XR action asset、validation、import/export与Editor29接入 |
| RT66-P1-67 | 无layer/mask/render-model authoring | content无法配置compositor | 提供typed component/asset并显示effective runtime capability |
| RT66-P1-68 | 无platform packaging/runtime selection | loader/runtime manifest未进入产品 | 按target/package声明loader、runtime requirements与fallback |
| RT66-P1-69 | 无privacy/permission/accessibility边界 | eye/hand/spatial数据可能被误用 | permission gate、retention policy、seated/standing与comfort设置 |
| RT66-P1-70 | 无XR diagnostics/capture receipt | 无法定位runtime/device/frame/layer错误 | bounded event、frame receipt、timing、pose age、lease和extension快照 |
| RT66-P1-71 | 无conformance/fake-runtime/soak测试 | 普通camera/surface测试无法验XR | fake provider、state/fault matrix、OpenXR conformance和设备实验室 |
| RT66-P1-72 | 无竞争性XR资格 | 无motion-to-photon或同画质证据 | 固定设备/场景/refresh比较CPU/GPU、latency、dropped frame与视觉oracle |

## 7. P2治理与可维护性项

| ID | 差距 | 建议 |
|---|---|---|
| RT66-P2-01 | XR缩写与普通view术语易混 | 固定Instance/System/Session/Space/View/Layer/Lease词汇表 |
| RT66-P2-02 | OpenXR结果码可能被字符串化 | closed enum保留raw code、function、runtime和generation |
| RT66-P2-03 | native handle容易泄露到public API | 使用opaque scoped wrapper并限制unsafe owner模块 |
| RT66-P2-04 | 时间单位可能混用 | `XrTime`、host instant、simulation time通过显式转换receipt连接 |
| RT66-P2-05 | pose/matrix坐标约定易漂移 | 集中定义handedness、unit、clip depth和view transform测试 |
| RT66-P2-06 | extension feature命名可能绑定vendor | capability ID与具体extension name分离并记录provider |
| RT66-P2-07 | per-view数组易产生隐式上限 | runtime枚举后bounded allocation，禁止写死双眼长度2 |
| RT66-P2-08 | frame tracing易形成高基数 | session/view/frame采用bounded sampling与structured correlation |
| RT66-P2-09 | mirror设置可能污染XR quality | mirror resolution/present policy独立但共享明确资源预算 |
| RT66-P2-10 | debug layer可能改变timing | evidence注明validation/API layer和capture状态 |
| RT66-P2-11 | action本地化与stable path易混 | stable path、localized label和Editor display name三者分离 |
| RT66-P2-12 | mock runtime可能反客为主 | mock只实现contract/fault injection，不成为production fallback |
| RT66-P2-13 | extension顺序和`pNext`难审计 | canonical chain snapshot、duplicate/cycle检查和human diff |
| RT66-P2-14 | XR默认值容易散落 | 由product/device policy解析，构造器只提供Unavailable-safe值 |
| RT66-P2-15 | 参考引擎版本会漂移 | 保留路径/fingerprint/applicability，变化自动标记recheck |
| RT66-P2-16 | 未来生成器可能重建双authority | tooling迁Rust后只消费runtime schema，不另建XR source truth |

## 8. 目标架构与数据流

```text
Project/Profile/Platform policy
              |
              v
      XrRuntimeProvider -----> XrExtensionRegistry
              |                        |
              v                        v
      XrInstanceAuthority --> XrSystemProfile
              |                        |
              +---- graphics requirements
                                       v
                              XrGraphicsBindingBridge <--> Runtime09A RHI device
                                       |
                                       v
                              XrSessionSupervisor
                         event/state/loss/focus generation
                                       |
          +----------------------------+---------------------------+
          v                            v                           v
    XrFramePacer                  XrSpaceGraph                XrActionRuntime
 wait/begin/end/time          locate/recenter/validity     sets/profiles/haptics
          |                            |                           |
          +----------------------------+---------------------------+
                                       v
                                  XrViewFamily
                         views/culling/history/multiview
                                       |
                         XrLateUpdateCoordinator
                                       |
                           render + XrSwapchainLease
                                       |
                         XrCompositionLayerGraph
                                       |
                       xrEndFrame + mirror + diagnostics
```

关键不变量：

1. OpenXR runtime选出的system/graphics requirements、RHI实际device/queue和swapchain image必须绑定同一代receipt；任何一项不一致都fail-close。
2. 每次成功`wait`最多对应一次`begin`和一次terminal `end/discard`；swapchain image必须按acquire、wait、write-complete、release顺序终止，旧generation不可复用。
3. `XrViewFamily`在一个predicted display time下冻结view configuration和per-view pose/FOV；camera stack、split-screen和XR view是不同概念。
4. late update只能修改明确eligible的view/proxy数据，并同时更新history/velocity所需generation；不能在提交中随意改World真值。
5. action sample必须保留action/subaction/source profile、XrTime、active/changed、pose validity和session generation；不能退化为无来源gamepad数值。
6. extension capability、permission、requested policy和effective feature是四种不同事实；foveation、hand/eye、spatial或frame synthesis都必须有独立receipt和fallback。
7. mirror window、Editor preview和capture是XR frame的consumer，不得夺取session cadence、swapchain ownership或伪造runtime availability。

## 9. 依赖顺序与实施里程碑

| 里程碑 | 目标 | 依赖 | 完成证据 |
|---|---|---|---|
| XR-M0 · Truth freeze | 固化零能力、owner route、冻结集和产品禁用状态 | 本篇、Runtime42 | fingerprint、zero search、capability fail-close测试 |
| XR-M1 · Provider/instance/system | loader、instance、extension、system profile与mock provider | Runtime24/42/58 | loader/version/extension/system/fault matrix |
| XR-M2 · Graphics binding | OpenXR requirements约束RHI device并建立typed native bridge | XR-M1、Runtime09A | D3D12/Vulkan至少一条真实binding与mismatch拒绝 |
| XR-M3 · Session/frame/swapchain | session state、event、wait/begin/end与swapchain lease | XR-M1-M2、Runtime57 | ordered trace、timeout/loss/restart/fence fault injection |
| XR-M4 · View family/render | per-view数据、single/multipass、history、visibility/mirror | XR-M3、Runtime09B/09H1/37 | stereo/quad pixel、history isolation、multiview fallback |
| XR-M5 · Space/late update | reference space、validity、recenter与render-time locate | XR-M3-M4、Runtime23/37 | pose age、recenter、late-latch和velocity/history测试 |
| XR-M6 · Action/input | action set/profile/space/haptic、hand/eye optional service | XR-M1/XR-M3/XR-M5、Runtime56 | profile change、sync、pose/haptic/permission矩阵 |
| XR-M7 · Composition/quality | projection/native UI layer、foveation、refresh/performance | XR-M3-M6、Runtime65 | layer order/color/depth、foveation/fallback与budget receipt |
| XR-M8 · Product/editor/cook | App/Editor/schema/package/runtime selection闭环 | XR-M1-M7、Editor29/30 | create/open/run/mirror/restart/package/install evidence |
| XR-M9 · Qualification | conformance、设备实验室、soak、latency与竞争基准 | XR-M0-M8、O11/O14 | CTS、GPU capture、motion-to-photon、dropped frame和视觉oracle |

M1前不得先把两个camera并排画到window并命名为XR；M2前不得让OpenXR接受engine自选device；M4前不得以`multiview_mask != None`单点变化宣称single-pass完成；M9前不得宣称达到或超过Unreal。

## 10. 验收门禁

| Gate | 验收内容 |
|---|---|
| XR-G01 | XR product policy有stable ID/schema/target/provider requirement与Disabled-safe默认 |
| XR-G02 | 无runtime/loader/provider时capability fail-close并产生typed receipt |
| XR-G03 | instance创建验证API version、required/optional extension和API layer |
| XR-G04 | system选择记录runtime name/version/form factor/blend/view configurations |
| XR-G05 | extension registry检测duplicate、conflict、dependency、unsupported和chain错误 |
| XR-G06 | instance/session/space/action/swapchain handle均有owner generation和stale拒绝 |
| XR-G07 | session event状态机覆盖READY/RUNNING/VISIBLE/FOCUSED/STOPPING/LOSS |
| XR-G08 | begin/end session、runtime loss、exit和restart保持last-good或明确Unavailable |
| XR-G09 | OpenXR graphics requirements先于RHI device创建并约束真实adapter/API version |
| XR-G10 | graphics binding只来自RHI owner，裸native handle不进入camera/scene public API |
| XR-G11 | format/sample/array/usage协商与runtime推荐extent形成可重建receipt |
| XR-G12 | swapchain每个image严格acquire/wait/write-complete/release，fault injection无越序 |
| XR-G13 | color/depth/motion swapchain的extent/view/generation一致且可独立fallback |
| XR-G14 | device loss/session restart使旧image/view/history/space lease全部失效 |
| XR-G15 | 每次wait至多对应一次begin和一次terminal end/discard |
| XR-G16 | predicted display time贯穿locate views/spaces、input sample、render和frame receipt |
| XR-G17 | `shouldRender=false`不提交陈旧layer且仍遵守合法frame终态 |
| XR-G18 | XR cadence不由普通window redraw/focus/occlusion替代，mirror不反向阻塞session |
| XR-G19 | view family支持runtime返回的view count，不写死2 |
| XR-G20 | 每个view携FOV/pose/matrix/viewport/slice/validity/configuration/index |
| XR-G21 | camera stack、split-screen与XR view group具有不同typed identity |
| XR-G22 | shared culling与per-view frustum结果有correctness oracle和成本对比 |
| XR-G23 | single-pass multiview/instancing与multipass按capability选择并记录fallback |
| XR-G24 | pipeline/PSO/shader/view constants覆盖实际view mask和array slice |
| XR-G25 | history key覆盖session/view config/index/space/pose generation，双眼不串history |
| XR-G26 | visibility/occlusion mask按view/generation更新且不会读取越界mesh |
| XR-G27 | stereo、mono和quad layout的render pass/view mapping通过像素及结构测试 |
| XR-G28 | HDR/color/depth/velocity/post/UI在single/multipass下语义一致 |
| XR-G29 | VIEW/LOCAL/STAGE/LOCAL_FLOOR支持情况、offset和fallback可解释 |
| XR-G30 | pose position/orientation valid/tracked分别传播，invalid不伪造identity pose |
| XR-G31 | recenter/origin change原子发布space generation并失效相关history |
| XR-G32 | late update在受控render边界执行，pose age和eligible proxy集合可审计 |
| XR-G33 | action set/action/path/binding suggestion/attach/sync生命周期严格有序 |
| XR-G34 | Boolean/Float/Vector2/Pose action保留subaction/profile/time/active/change语义 |
| XR-G35 | interaction profile change会重解析binding并发布provenance，不丢输入 |
| XR-G36 | XR haptics支持apply/stop/focus/session终止清理并与gamepad rumble隔离 |
| XR-G37 | hand/eye/tracker服务按extension、device support和permission三重资格发布 |
| XR-G38 | eye/hand/spatial数据有retention/privacy策略，默认不持久化敏感样本 |
| XR-G39 | composition graph稳定生成projection/quad/cylinder/equirect层及排序 |
| XR-G40 | layer alpha、premultiply、color space、eye visibility和space binding可验证 |
| XR-G41 | foveation固定/eye-tracked模式有swapchain/pipeline/capability/fallback闭环 |
| XR-G42 | refresh/performance/thermal输入进入Runtime65同一quality/budget authority |
| XR-G43 | mirror/spectator、capture和Editor preview不改变XR frame ownership |
| XR-G44 | App/Editor/project asset/package/runtime selection共享同一runtime schema |
| XR-G45 | fake provider覆盖state、timeout、loss、stale handle、invalid pose和permission fault |
| XR-G46 | 至少一个真实runtime/backend通过OpenXR conformance、长时soak和GPU capture |
| XR-G47 | 同设备/refresh/场景报告CPU/GPU、latency、pose age、dropped frame、VRAM和视觉oracle |
| XR-G48 | `git diff --check`、frontmatter path/link、0/72/16 finding计数、48 gates和五份账本一致 |

## 11. 状态与产出记录

| 项目 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Zircon 719文件纵向冻结 | review_complete | 2026-08-20 | 133,681行、4,812,247 bytes；SHA-256 `36aa70c3d7c4ef5ea42b86a71034c6804533c05f1e999d0ed55cd720547f4c10` |
| 五参考71文件语义对照 | review_complete | 2026-08-20 | 32,858行、1,308,266 bytes；SHA-256 `70653576bda8cd2131fd7009f477654ba07537d22ba1ada2dc5e2c7d2888aa92` |
| Severity与owner路由 | review_complete | 2026-08-20 | 0 P0 / 72 P1 / 16 P2；48 gates；共享父owner不重复计数 |
| Production、tests与Cargo变更 | pending | - | 本篇只review；MVP gate下未运行Cargo或产品验证 |
