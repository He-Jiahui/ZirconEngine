---
title: Editor Scene Viewport Realtime Update、Preview Simulation、Time Domain、Pause/Step、Animation/Particle/Physics/Audio、Visibility Throttling、Invalidation、Performance 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor69
review_date: 2026-08-22
baseline_head: 4d5f52aa2b76a3a877aabdd47b01a98dcdd59493
baseline_epoch: 340
related_code:
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/scene/viewport/edit_mode_projection
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
tests:
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/gateway/session/frame_demand.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/toolbar_dispatch.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/dispatch.rs
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer/surface_contract.rs
  - zircon_editor/src/tests/host/retained_window/native_viewport_image.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/66-editor-scene-viewport-camera-navigation-orbit-pan-zoom-fly-projection-alignment-frame-selection-bookmark-pilot-persistence-input-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/68-editor-scene-viewport-display-mode-lighting-skybox-show-flag-debug-visualization-overlay-composition-profile-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/99d-runtime-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorViewportClient.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/PreviewScene.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PreviewScene.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Public/SLevelViewport.h
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/LevelEditorViewport.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/LevelEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Settings/LevelEditorViewportSettings.h
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.h
  - dev/godot/scene/main/viewport.cpp
  - dev/godot/scene/main/viewport.h
  - dev/Fyrox/editor/src/lib.rs
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/scene/controller.rs
  - dev/Fyrox/editor/src/audio/preview.rs
  - dev/Fyrox/editor/src/particle.rs
  - dev/Fyrox/editor/src/plugins/animation/toolbar.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/crates/bevy_time/src/virt.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/bevy/crates/bevy_time/src/common_conditions.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/stepping.rs
  - dev/bevy/examples/showcase/stepping.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/LightPlacementTool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeSubdivisionContext.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/GraphView/VFXComponentBoard.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/UIResources/uxml/VFXComponentBoard.uxml
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Debug/VFXUIDebug.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.SchedulerTracker.cs
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Realtime Update、Preview Simulation、Time Domain、Pause/Step、Animation/Particle/Physics/Audio、Visibility Throttling、Invalidation、Performance 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon已经具备实现工程级实时视口的若干真实底座。Runtime有real/virtual/fixed三类时钟、pause、relative speed、max delta、fixed overstep和每帧最大fixed step预算；`WorldDriver`会把virtual delta、real delta、paused状态和fixed plan送入Scene schedule。Dynamic Runtime能够返回Idle/After/Immediate frame demand，Editor gateway将其转换为`OnDemand / SleepUntil / Continuous`，Retained Host再通过`WaitUntil`或外部redraw桥接到原生事件循环。编辑世界的render extract、失效标记、GPU/capture产品轮询也是真实链路。这些基础应复用，而不是在Editor里另起一个定时器、第二套物理循环或无界`request_redraw`轮询。

但当前Scene Viewport没有Realtime Update或Preview Simulation产品。`ViewportCommand`、`SceneViewportSettings`、controller state、toolbar projection与ZUI均没有Realtime、Pause、Resume、Step、Time Scale、Fixed Step、Simulation、Audio Listener或Mute控制；工具栏只有进入/退出Play。更关键的是，`EditorHostEventController::pump_runtime_event_consumers`只在active Play session、`WorldDomain::Play(instance)`和play gateway同时存在时调用`tick_frame()`。Edit authoring world只消费world invalidation并按需重建extract，从未通过Runtime时钟与`WorldDriver`推进。

这意味着“编辑视口里看到静态场景”和“工程级实时预览”之间仍有完整产品层缺口。动画、粒子、物理、音频和脚本是否允许运行、以哪个时域运行、在暂停时哪些系统继续、单步跨多少fixed step、退出时如何恢复、隐藏视口是否停止、多个视口如何合并需求、后台窗口如何降频、工具临时强制实时后如何恢复，当前都没有唯一authority。直接让authoring world全量tick会把脚本、副作用、物理漂移、音频输出和不可逆mutation混入编辑真相，不能作为可接受的临时实现。

现有Host按需机制也没有接上编辑态时间变化。`submit_render_frame_if_dirty`在成功提交后清除`render_dirty`；只有业务失效显式要求RENDER时才会重建extract。`poll_viewport_image_for_native_host`在新产品到达后只请求paint-only redraw，这对展示新图是正确的，但不能替代“时间推进 -> world更新 -> 新extract -> render submission”的前置链。当前Window事件只对`Focused(false)`做交互取消，没有`Focused(true)`状态、occlusion、minimized、pane visibility或电源/远程会话政策，`Continuous`需求也没有逐视口活动性裁剪。

参考实现都把这些问题当作状态、生命周期和调度产品，而不是一个bool。Unreal区分持久Realtime与带owner名称的临时override，能请求有限实时帧，并用独立Preview World、audio device/scene与physics选项管理生命周期；Godot把viewport update mode、可见性、process/physics process、audio listener和性能显示组成per-view状态；Fyrox用`GraphUpdateSwitches`选择physics/node update，并在音频/粒子预览退出时恢复原节点；Bevy提供virtual/fixed time、focus-aware reactive loop和schedule/system级stepping；Unity Graphics展示tool-scoped `alwaysRefresh`保存/恢复、后台停算、60 Hz限流、可见cell裁剪与VFX play/pause/step/rate。Zircon要达到并超过这些引擎，必须先具备同等级的正确性与可观测合同，再讨论更高性能。

本轮不新增P0。原因不是功能已经成熟，而是当前Scene Viewport没有宣称存在Realtime/Simulation控制，也没有把静态按钮伪装成实时成功；Play入口由Editor07单独拥有。动画、粒子、物理、音频底层和各资产预览工具的既有P0/P1继续由Runtime08A/08B/08C/22/99D及Editor14/15/17/18唯一计数。本报告新增 **30项P1、8项P2**，登记 **48个全部Fail的资格门**，目标是建立Runtime-owned `PreviewWorldSession + PreviewTimeDomain + PreviewSubsystemPolicy + FrameDemandContributionRegistry`，Editor-owned `ViewportPreviewSessionRegistry + ViewportRealtimeProfile + ViewportActivityPolicy + ViewportAudioListenerLease`，以及generation-qualified `EffectiveViewportPreviewReceipt`。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、GUI/GPU、动画/粒子/物理/音频预览、save/reopen、multi-viewport、background/minimize、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。当前不能声称Scene Viewport实时预览、性能或表现达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon Editor viewport preview state and product route | **18 / 2,351 / 2,143 / 88,132 / 10** | command、settings、controller、toolbar projection、render snapshot、route与workbench state | `7a8a7b86fba8fe7022b9baa9e7d5b2e544fc016c479d613e25cce559c4536986` |
| Zircon Editor cadence, activity, invalidation and host wake | **14 / 2,694 / 2,458 / 102,541 / 25** | gateway、Play tick owner、dirty/render、frame demand、native wait与window activity | `8894a0fa5767d68015e770e11192975809dbbe9e19ffc9a251ca07d5d42bc6ad` |
| Zircon Runtime time, schedule and dynamic frame demand | **11 / 1,632 / 1,441 / 55,278 / 10** | real/virtual/fixed clocks、WorldDriver、dynamic session/profile与demand | `d95d641283bf46c48d3cbca2c71d6f3432edb393482317c805662afb8a0ff7b2` |
| Zircon focused tests | **13 / 2,717 / 2,458 / 94,774 / 66** | viewport command/state/toolbar、gateway demand、native product与runtime animation demand | `3b8ab8f3e026ac6a7aa6f1caa55874f8d94498342a8b344bfe9584d33357732e` |
| Unreal selected set | **9 / 24,381 / 20,370 / 893,941 / 0** | realtime base/override、bounded frames、Preview World、audio、PIE transition与config | `0c131dfdbd6a008a9dbb38730382496359e71967008e9ca18dbd79f35bfbbc47` |
| Godot selected set | **4 / 14,805 / 12,525 / 543,729 / 0** | viewport update mode、visibility process gate、listener/Doppler、state restore与perf | `415b4afea3cc9b3758b47261007412e26dc239b88a583a1fd6b52483c0853828` |
| Fyrox selected set | **7 / 9,399 / 8,531 / 359,751 / 9** | editor fixed loop、GraphUpdateSwitches、音频/粒子恢复与动画transport | `9d990effeb8bba3208dc6ea50570effb606e4aaab957761053ac9e2d9924c2b1` |
| Bevy selected set | **6 / 3,076 / 2,695 / 111,747 / 38** | focus-aware update mode、virtual/fixed time、paused condition与schedule stepping | `ff94c86ff5a9c75bb4f3b88ecfc21a6e42799766596d0dc6b98376f185a3a863` |
| Unity Graphics selected set | **6 / 2,430 / 2,083 / 90,201 / 0** | scoped always-refresh、background/60 Hz/culling、VFX transport/rate与scheduler visibility | `bf440c85768e0f6ec2562065b2170d0456c389d72b3134c82b58d7fb71760d5b` |

fingerprint按规范化相对路径排序，并将每个`path + newline + file SHA-256 + newline`聚合后再做SHA-256；它只证明本轮读取的working-tree语料，不是ABI、artifact、动态结果或性能receipt。主仓与Unreal镜像基线为`4d5f52aa2b76a3a877aabdd47b01a98dcdd59493`；Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal参考树不是独立Git仓，按主仓基线和文件fingerprint冻结。

### 2.2 在途修改隔离

冻结时`zircon_runtime/src/scene/module/world_driver.rs`含非本轮修改，当前版本已经把普通stage的simulation delta改为virtual delta，并单独传递real delta和paused状态。本报告按working tree计入该修正，不重复旧Runtime22中“普通stage错误使用real delta”的历史描述。`zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs`也已证明active animation tick返回Immediate、暂停后回Idle；因此不重复旧Runtime08C“animation demand没有production写入”的过时判断，父报告实施前应自行current-source recheck。

`dirty_marking.rs`与`render_submission.rs`还含非本轮diagnostics pane局部失效优化：前者增加pane到shell-content scope解析，后者消费publication-time refresh target以避免普通render frame触发全局Workbench重建。它们已计入刷新后的cadence fingerprint，减少无关presentation成本，但没有新增edit-world tick、Realtime控制、activity gate或时间驱动RENDER。相邻viewport pointer/input、retained host startup与Runtime core文件另有其他Session在途修改，不属于本轮Realtime语料，未计入fingerprint，也不用于推导新 finding。新报告与三个共享索引通过coordinator Session `optimize-editor69-viewport-realtime-preview-review-r3-20260822`取得精确lease及maintenance授权，baseline epoch为340；实施前必须重取全部语料和父报告状态。

### 2.3 范围与非范围

本报告拥有edit-world Scene Viewport的Realtime意图、Preview Simulation会话、per-view time domain、pause/resume/step/speed/reset、可选animation/particle/physics/audio预览编排、frame-demand contribution、activity/visibility throttling、时间驱动render invalidation、preview persistence、effective receipt、诊断与性能资格。

Editor07继续拥有Play/PIE/Game View及authoring/play world transition；Editor14/15/17/18拥有Animation、VFX/Particle、Sound和Physics资产工具预览；Editor25拥有通用性能工具；Editor47拥有gateway生命周期；Editor53拥有通用tool lease；Editor58拥有ViewInstance、多viewport render product与currentness；Editor66/68拥有camera和visualization profile。Runtime08A/08B/08C/22/99D拥有各子系统与通用时钟父合同。Editor69只定义这些owner必须接受的viewport-preview session、policy、demand和receipt，不重复计数其内部缺口。

## 3. 当前实现拓扑与可保留基础

### 3.1 Runtime时钟与WorldDriver是真实基础

`RuntimeTimeClocks`已经把real、virtual、fixed分开，virtual支持pause、speed与max delta，fixed accumulator可按预算drain；`WorldDriver`按stage传播virtual/real delta、paused状态并循环fixed stages。目标架构应把这些能力实例化为preview-session time domain，而不是复制时间数学。

### 3.2 Dynamic frame demand与原生WaitUntil链路真实存在

Runtime ABI的Idle/After/Immediate被gateway严格校验并映射到Editor demand；Host会替换旧deadline、限制极端delay、合并redraw，并在`about_to_wait`选择最早deadline或无限Wait。这已经是高效reactive loop底座，缺的是edit-view贡献来源和活动性resolver。

### 3.3 Animation demand已在当前源码形成最小闭环

`RuntimeDynamicSession::frame_demand`优先保活asset reload，再读取level animation continuous-frame状态；focused test验证active sequence立即续帧，暂停后回Idle。Editor69不把这条已修路径重新登记为Runtime缺陷，而是指出它只通过Play dynamic session消费，Scene Viewport edit world没有接入。

### 3.4 Edit world只有失效驱动，没有时间驱动

Retained Host tick会update scene modes、消费edit-world invalidation、重建presentation并按dirty提交render，但不会对edit world调用`tick_time`或`LevelSystem::tick`。这保证静态编辑不空转，却也证明当前没有Realtime Preview。

### 3.5 Render dirty与viewport image redraw职责清晰但尚未组合

成功render submission会清除`render_dirty`；新viewport product只触发paint-only redraw。两者各自正确，但没有`PreviewSimulationReceipt`把“世界确实前进并产生新generation”转换为view-scoped RENDER失效，因此时间变化不会自动生成新extract。

### 3.6 Scene Viewport控制面没有预览状态

`ViewportCommand`、`SceneViewportSettings`、`SceneViewportState`、`SceneViewportToolbarState`和ZUI只覆盖交互、相机、显示、grid、snap、lighting、skybox、gizmo与Play。`SceneViewportStats`只显示对象分类计数，不含FPS、frame time、virtual/fixed clock、step debt、subsystem cost或effective preview状态。

### 3.7 Window activity只处理focus loss的交互取消

当前window event match响应`Focused(false)`以取消交互，但没有记录focus regain，也没有Occluded、minimized或pane visibility状态。`window_visible`主要用于生命周期展示；它没有参与runtime frame demand或per-view预算决策。

### 3.8 当前测试证明transport，不证明edit preview

focused tests覆盖viewport settings/command/toolbar投影、gateway demand协议、native viewport image和Runtime animation demand。没有测试创建edit preview session，也没有pause-step守恒、hidden view停算、多view需求合并、audio listener、physics restore、slow-frame debt、save/reopen或性能预算证据。

## 4. 五引擎参考证据与适用边界

### 4.1 Unreal：持久Realtime、owner override与Preview World是三个不同合同

`FEditorViewportClient`区分`SetRealtime`持久状态与`AddRealtimeOverride(bool, SystemDisplayName)`临时覆盖；保存配置时明确忽略override，工具还能用`RequestRealTimeFrames`请求有限帧。Level viewport按实例恢复Realtime，远程桌面性能政策可追加命名override，PIE进入时又会调整Editor realtime。`FPreviewScene::ConstructionValues`显式选择audio、physics与world类型，创建transient EditorPreview/GamePreview world并在销毁时清理world context、audio、component和physics资源。适用于Zircon的核心不是复制类层级，而是区分用户意图、owner-scoped override和runtime-owned isolated world。

### 4.2 Godot：更新模式、可见性、listener与状态恢复均为per-view

Godot 3D editor viewport在visibility notification中同时切换process与physics process；SubViewport有Disabled、Once、WhenVisible、WhenParentVisible、Always更新模式。3D viewport state会保存listener、Doppler、frame-time/info/display/environment/gizmo等状态，性能显示只在可见且启用时采样。Zircon应借鉴“可见性决定有效调度”，而不是把所有窗口共用一个Continuous bool。

### 4.3 Fyrox：选择性更新与退出恢复优先于全量模拟

Fyrox Editor固定步进当前scene，并用`GraphUpdateSwitches`选择physics2d/3d、node subset、dead-node deletion和pause；pause会同步sound context。音频与粒子preview会clone原节点、安装override，并在离开时恢复精确原值或销毁临时source；动画toolbar提供play/pause/stop/preview/speed。其规模小于Unreal，但“只运行被准入的子系统并可恢复”比直接tick authoring world更接近正确产品。

### 4.4 Bevy：时间、低功耗loop与schedule stepping是可组合原语

Bevy `Time<Virtual>`定义pause、relative speed、max delta，`Time<Fixed>`跟随virtual clock并维护overstep；`WinitSettings`按focused/unfocused选择Continuous、Reactive或low-power Reactive。`Stepping`能对schedule和system配置always/never/break，按frame step或continue。Bevy不是完整Editor产品参考，但证明时间域、run condition、需求唤醒和单步不必耦合在一个全局开关中。

### 4.5 Unity Graphics：工具临时强制刷新必须保存/恢复并受预算约束

本地Graphics仓不是完整Unity Editor源码，只能作为可验证局部参考。`LightPlacementTool`激活时保存`SceneViewState`并设置`alwaysRefresh=true`，停用时恢复；Probe subdivision debug在后台inactive时停止、把200 Hz editor update限制到60 Hz、裁剪不可见cell并逐帧推进；VFX board提供stop/play-pause/step/restart/play rate，detach时恢复component状态；Debug scheduler对不可见内容pause。Zircon不能据此推断Unity完整Scene View内部实现，但应采用同样的owner restore和visibility budget纪律。

## 5. 差异矩阵

| 能力 | Zircon当前事实 | 工程级目标 | 参考证据 | 判定 |
|---|---|---|---|---|
| Realtime状态 | 无命令、状态或toolbar控制 | per-view持久意图与effective state | Unreal/Godot | Missing |
| 临时强制实时 | 无owner、lease或restore | owner-scoped override stack/lease | Unreal/Unity | Missing |
| Preview World | edit world静态extract；Play另管 | isolated/snapshot-backed runtime session | Unreal/Fyrox | Missing |
| Time domain | Runtime只有session全局clocks | per-preview real/virtual/fixed clock | Bevy/Unreal | Missing |
| Pause/Step/Speed | Scene Viewport无控制 | typed transition与step receipt | Bevy/Fyrox/Unity | Missing |
| 子系统选择 | 无preview policy | dependency-closed allow/deny policy | Fyrox/Unreal | Missing |
| Animation | Runtime demand存在，edit view不消费 | preview contribution + render invalidation | Bevy/Fyrox | Partial |
| Particle/VFX | 资产工具父报告拥有，edit view无编排 | scoped preview session、reset/reseed | Fyrox/Unity | Missing |
| Physics | fixed loop存在，edit view无sandbox | selective fixed simulation + restore | Unreal/Fyrox | Missing |
| Audio | Scene Viewport无listener/mute | view camera listener lease + output policy | Unreal/Godot/Fyrox | Missing |
| Frame demand | 只由Play gateway进入Host | multi-owner contribution resolver | Bevy/Unreal | Partial |
| Visibility throttle | 无focus regain/occlusion/pane gate | activity snapshot + foreground/background budget | Godot/Bevy/Unity | Missing |
| Render invalidation | mutation-driven；时间不产生新extract | preview receipt驱动view-scoped render generation | Unreal/Godot | Missing |
| Persistence | serde类型能力，不是存储闭环 | versioned per-user/workspace/view profile | Unreal/Godot | Missing |
| Diagnostics | 只有对象计数 | clock、tick、debt、cost、drop、effective policy | Godot/Unity | Missing |
| Qualification | 无edit preview测试/benchmark | correctness/fault/soak/power/perf evidence | 五引擎共同 | Missing |

## 6. 新增发现

### 6.1 P1：架构、正确性与产品闭环

#### ED69-P1-01：没有Viewport Preview Session authority与qualified identity

Scene viewport controller只有一份settings/camera/selection/drag状态，没有`PreviewSessionId`、ViewInstance、DocumentSession、World epoch、generation或owner。任何未来异步tick、step或subsystem回执都无法证明仍属于当前view和world。

#### ED69-P1-02：Edit authoring world从不进入Runtime tick链

Host只在Play domain调用gateway `tick_frame()`；edit world只消费invalidation并extract。Animation、particle、physics、audio或脚本即使在Runtime可运行，也不会在普通Scene Viewport随时间前进。

#### ED69-P1-03：Realtime用户意图与临时owner override均不存在

没有持久Realtime toggle，也没有工具、camera transition、capture或plugin可取得的临时override lease。以后若多个owner直接写同一个bool，工具退出、异常和嵌套覆盖必然产生状态泄漏。

#### ED69-P1-04：Pause、Resume与Step没有typed状态机

`ViewportCommand`无法表达pause/resume/step，Runtime CoreHandle的pause API也没有view/session owner或transition receipt。UI无法区分requested、accepted、effective、pending和denied状态。

#### ED69-P1-05：Time Scale、Fixed Step、Reset、Seek与Reseed没有preview合同

Runtime提供relative speed和fixed timestep数学，但Scene Viewport不能按会话设置、重置或观察；particle/animation seek、physics fixed step与deterministic reseed也没有统一时间轴。

#### ED69-P1-06：CoreHandle时钟是session级共享可变状态，不能直接作为per-view时域

`pause_virtual_time`、`set_virtual_time_relative_speed_f64`和`set_fixed_timestep`修改Core全局bundle，没有world/view/owner/generation限定。多个预览或Play并存时直接调用会互相污染。

#### ED69-P1-07：没有isolated preview world、checkpoint或rollback边界

当前若直接tick authoring world，physics位置、脚本状态、particle emitter、audio voice和dead-node清理都可能改变编辑真相。系统没有clone/snapshot、participant checkpoint、Apply/Discard或退出恢复receipt。

#### ED69-P1-08：没有selective Preview Subsystem Policy与依赖闭包

Animation、particle、physics、audio、scripts、navigation、AI等没有Allow/Deny/DiagnosticOnly政策，也没有声明依赖、固定阶段、暂停行为和资源预算。全量scene schedule不是工程级默认值。

#### ED69-P1-09：脚本、网络、保存和外部副作用没有preview admission

Preview中哪些system可写文件、发网络、spawn process、提交operation或修改project资源没有sandbox/capability政策。没有fail-close admission就不应开放“Simulate Entire Scene”。

#### ED69-P1-10：Animation continuous demand没有Scene Viewport consumer

当前Runtime test证明animation能请求Immediate，但只有Play session把demand送入Host。Scene Viewport既没有animation contribution owner，也没有在动画结束/暂停/hidden时可靠撤销需求的会话链。

#### ED69-P1-11：Particle/VFX预览没有Scene Viewport生命周期编排

没有play/pause/stop/restart/seek/reseed、emitter selection、CPU/GPU capability、warm-up、loop或退出恢复合同。Editor15和Runtime99D拥有子系统内部实现，Editor69拥有其进入通用Scene Viewport的session边界。

#### ED69-P1-12：Physics预览没有fixed-step sandbox与状态恢复

Runtime fixed loop存在，但Scene Viewport不能选择physics2d/3d、collision-only、gravity、substep、max catch-up或kinematic authoring policy，也不能在结束时恢复body transform/velocity/sleep/contact状态。

#### ED69-P1-13：Audio listener、mute与preview output route缺席

Scene camera没有audio listener lease，view focus不能决定listener owner，toolbar没有mute/solo，background/hidden view没有输出政策，也没有voice cleanup、Doppler/attenuation和device failure receipt。

#### ED69-P1-14：产品UI把Enter Play留作唯一时间入口，不能替代Realtime Preview

Play会切换到独立runtime session和Editor07生命周期；Realtime Scene Viewport应保持authoring上下文并只准入选定预览能力。两者语义、风险和恢复完全不同，不能用Play按钮掩盖缺失。

#### ED69-P1-15：Command、settings、chrome与projection没有requested/effective预览状态

从binding到toolbar projection没有preview字段，无法展示paused、stepping、throttled、background、degraded或unavailable，也无法让automation走同一typed command链。

#### ED69-P1-16：没有capability admission、effective receipt和denial provenance

请求physics/audio/GPU particle后，系统不能返回实际启用的子系统、被裁剪原因、fallback、time generation、world generation或provider版本。UI只能猜测，而不是投影Runtime事实。

#### ED69-P1-17：Host frame demand入口被Play session独占

`pump_runtime_event_consumers`在没有active Play session时直接返回OnDemand，edit view没有独立producer。现有高效WaitUntil链可保留，但缺少Editor preview demand的合并入口。

#### ED69-P1-18：没有多owner Frame Demand Contribution registry

Animation、particle、physics、audio、camera transition、tool和plugin不能以owner/lease/priority/deadline/budget贡献需求。单一latest demand会让一个consumer覆盖另一个，异常退出也无法自动撤销。

#### ED69-P1-19：时间推进不会产生view-scoped render invalidation

成功render后`render_dirty=false`，只有显式RENDER失效再次提交。缺少world/update generation到ViewInstance的映射，time-only animation或particle变化不会可靠生成新extract。

#### ED69-P1-20：Viewport image paint-only更新与simulation drive之间缺少generation桥

image/product polling只负责展示已产生的图，这是正确边界；但没有preview receipt证明哪次simulation生成了哪次extract/image，也没有dropped/stale generation处理。不能靠poll callback反向驱动世界。

#### ED69-P1-21：Visibility、focus、occlusion、minimized与pane activity没有调度政策

Window只处理focus loss交互取消，没有完整activity snapshot；Scene pane是否可见、被tab覆盖、窗口是否occluded/minimized均不参与需求解析。隐藏视口可能继续继承Play连续帧，未来edit realtime也无处降频。

#### ED69-P1-22：没有foreground/background帧率、catch-up与debt预算

系统没有per-view target Hz、background Hz、max delta loss、fixed-step debt、drop/skip政策或恢复后的catch-up上限。慢帧、系统休眠和远程桌面下无法保证稳定性与电源效率。

#### ED69-P1-23：Multi-viewport没有独立policy与聚合规则

Editor58拥有ViewInstance产品identity，但当前Scene controller仍是一份共享状态。两个Scene view无法一个Realtime、一个Paused，也不能按最高需求、可见性、世界共享关系和audio owner解析有效帧率。

#### ED69-P1-24：Preview profile没有真实持久化、schema与迁移

`Serialize/Deserialize`只证明DTO可编码；Realtime/subsystem/time/activity字段甚至不存在，更没有per-user/workspace/view scope、version、migration、unknown provider preservation或crash restore。

#### ED69-P1-25：Viewport diagnostics不含时间、系统成本与调度结果

HUD只有selected/node/visible/camera/mesh/light计数。没有real/virtual/fixed delta、step count/debt、effective Hz、frame demand owner、throttle reason、animation/particle/physics/audio cost、dropped frame或restore failure。

#### ED69-P1-26：World replace、asset reload、device loss与preview failure没有统一生命周期

打开/关闭/重载Scene、Play transition、plugin reload、audio device loss、renderer failure或subsystem panic时，preview session应先撤销demand和listener，再quiesce/restore资源。当前没有这条terminal choreography。

#### ED69-P1-27：Plugin不能安全贡献realtime需求或preview subsystem

没有stable provider descriptor、capability closure、owner generation、budget、callback fault domain和lease retirement。允许插件直接调用continuous redraw或CoreHandle time API会破坏Host调度与世界隔离。

#### ED69-P1-28：没有deterministic reset/reference state与重复运行守恒

Preview的随机种子、clock epoch、initial snapshot、asset generation和external input没有冻结；Stop/Restart后无法证明回到同一状态，也不能比较CPU/GPU particle或不同后端结果。

#### ED69-P1-29：聚焦测试没有覆盖edit preview正确性

现有66个test declaration没有创建Viewport Preview Session、推进edit world、暂停/单步、切换activity、恢复原状态或合并multi-owner demand。源码词法/route测试不能证明模拟正确。

#### ED69-P1-30：没有性能、功耗、故障与长时间资格证据

无visible/hidden Hz、CPU/GPU frame cost、10/100 viewport、长时间particle/physics/audio、sleep/resume、device loss、plugin failure、memory growth、电池/远程会话或同语义Unreal基线。当前无法支持“性能优于Unreal”的结论。

### 6.2 P2：质量、可维护性与资格表达

#### ED69-P2-01：Preview控制没有stable descriptor与可扩展transport vocabulary

未来若继续在`ViewportCommand`闭集枚举中逐个堆Play/Pause/Step，将难以表达provider-specific controls、availability和automation schema。应先定义stable descriptor与typed intent。

#### ED69-P2-02：Preview默认值、preset与scope policy尚无统一owner

Realtime默认开关、foreground/background Hz、fixed budget、audio mute和subsystem preset没有明确project/user/view归属；不应散落为Host或plugin裸常量。

#### ED69-P2-03：Serializable viewport state容易被误读为已持久化产品

当前settings派生serde但没有storage roundtrip。文档、测试和UI应区分“可编码DTO”与“已通过schema/migration/atomic write恢复”的持久化事实。

#### ED69-P2-04：临时override缺少可检查的provenance/history设计

即使后续实现override stack，也需要owner display name、reason、start time、priority和retirement状态，供UI与diagnostics解释“为何Realtime被强制开/关”。

#### ED69-P2-05：Transport控件尚无keyboard、accessibility与disabled-reason规范

Play/Pause/Step/Restart/Speed需要一致icon、focus order、checked/mixed/pending状态、accessible name和不可用原因；不能只放一排无回执icon。

#### ED69-P2-06：Diagnostics缺少单位、freshness和采样成本schema

未来FPS、step debt、system cost和drop count必须带source、unit、window、timestamp/generation与采样预算，不能再扩充为无来源字符串HUD。

#### ED69-P2-07：没有跨后端、跨平台和跨引擎可复现基线recipe

应冻结scene、camera、time trace、seed、subsystem policy、quality、hardware/driver与warm-up；只比较截图或平均FPS无法证明同语义表现和性能。

#### ED69-P2-08：本地Unity Graphics参考不包含完整SceneView owner实现

报告只采用可验证的tool/VFX/debug scheduler证据，不把缺失源码推断为Unity行为。后续benchmark与产品设计仍需以Zircon自身合同和可执行证据为准。

## 7. 目标架构与职责边界

### 7.1 Runtime：唯一Preview World、Time与Subsystem执行authority

Runtime建立`PreviewWorldSession`，identity至少包含`PreviewWorldSessionId + SourceWorldId + SourceWorldEpoch + PreviewWorldGeneration + OwnerGeneration`。session通过isolated clone或声明式snapshot/checkpoint创建；任何可写participant必须实现capture/restore或明确被拒绝。普通authoring world不得因一个Realtime bool直接进入完整game schedule。

`PreviewTimeDomain`组合现有`RuntimeTimeClocks`，增加session-scoped transition sequence、step token、reset/seek/reseed与fixed debt policy。Pause只停止声明跟随virtual time的stage；real-time maintenance、asset completion或diagnostic是否继续由policy显式决定。Step必须返回实际运行的fixed/variable stage、consumed delta、remaining debt和world generation。

`PreviewSubsystemPolicy`使用stable subsystem/provider id，声明Allowed/Denied/DiagnosticOnly、依赖闭包、stage、pause behavior、side-effect class、resource budget和restore participant。Animation、particle、physics、audio只是第一方provider，不允许Editor按类型直接调用内部manager。

`FrameDemandContributionRegistry`接受带owner lease、session/view/world scope、deadline/continuous、target/min Hz、priority、budget class和expiry的贡献，解析为`EffectiveFrameDemand`。Runtime输出immutable `PreviewSimulationReceipt`和`EffectiveViewportPreviewReceipt`，包含requested/effective policy、capability/fallback/denial、clock、step、world/extract generation与diagnostics引用。

### 7.2 Editor：per-view意图、活动性与产品投影

Editor建立`ViewportPreviewSessionRegistry`，以`ViewInstanceId + DocumentSessionId + WorldEpoch`定位session，并持有Runtime stable handle而非World authority。`ViewportRealtimeProfile`保存用户意图、subsystem preset、time scale、foreground/background budget与audio policy；`ViewportPreviewControlIntent`统一UI、keymap、automation和tool override入口。

`ViewportActivityPolicy`只拥有Editor可知事实：pane visible/covered、window focused/occluded/minimized、active tab、remote/power preference。它生成qualified activity snapshot交给resolver，不直接tick world。`ViewportAudioListenerLease`以view camera和focus policy竞争唯一/分组listener，Runtime负责实际device/voice生命周期。

Toolbar投影`EffectiveViewportPreviewReceipt`，明确显示Realtime、Paused、Stepping、Throttled、Unavailable、Degraded和强制override owner。Settings service负责versioned per-user/workspace/view持久化；project不能私自持久化机器audio device或窗口activity。

### 7.3 Host/App：只负责合并有效需求与原生唤醒

Host接收已解析的viewport contribution和activity policy，合并maintenance/render/presenter retry deadline，继续使用现有`WaitUntil`与coalesced redraw。它不解释animation/physics语义，也不维护第二套clock。有效preview receipt若产生新world generation，只对对应ViewInstance标记RENDER；viewport image到达仍保持paint-only职责。

Window event必须完整记录focus gain/loss、occlusion、zero-size/minimized和visibility transition；pane activity由Workbench projection提供。Hidden/occluded策略默认降到OnDemand或bounded background Hz，除非明确的capture/audio/remote owner持有预算化override。

### 7.4 不可违反的owner规则

- Runtime拥有world mutation、clock、schedule、subsystem admission、restore与effective simulation receipt。
- Editor拥有view/user intent、tool override、activity snapshot、UI projection与preference persistence。
- App/Host拥有原生event loop、wake deadline、redraw coalescing和surface lifecycle。
- Play与Preview使用共同底层原语但不同session/profile，不共享裸CoreHandle可变时钟。
- Asset toolkit preview可消费同一Runtime preview service，但其document/selection/transport仍由Editor14/15/17/18拥有。
- Plugin只能通过descriptor、lease、budget和fault boundary贡献需求或subsystem，禁止直接无界redraw。
- 任意Apply回authoring world必须走Editor63 transaction和qualified object generation；默认Stop/Close只Discard并恢复。

## 8. 分阶段重构计划

### ED69-M0：真实性、owner与RED基线

Goal：冻结Preview与Play、资产预览、父Runtime报告的边界，建立capability matrix和当前失败测试。

Implementation slices：定义术语、identity/owner表和unsupported surface；添加RED tests证明edit world不tick、无控制、无activity gate和无receipt；修正会误导为已支持的状态文案，但不造临时simulation。

Testing stage：运行focused source/contract tests与文档owner guard，逐项确认失败原因来自缺合同而非父报告已修路径；实施前重算语料fingerprint。

Exit evidence：边界表、RED suite与无重复P0/P1审计通过，才能进入M1。

### ED69-M1：Stable identity、DTO、registry与receipt

Goal：建立Runtime-neutral Preview session、time、subsystem、demand和receipt合同。

Implementation slices：新增stable IDs、generation rules、intent/effective DTO、provider descriptor、lease/expiry和serialization version；Editor只保存handle与intent。

Testing stage：unit/property/negative tests覆盖stale view/world/provider generation、重复lease retirement、unknown field preservation、invalid speed/timestep/budget和receipt immutability。

Exit evidence：所有identity与schema边界可独立验证，无裸Entity/View整数跨session复用。

### ED69-M2：Isolated Preview World与terminal lifecycle

Goal：Runtime创建、启动、停止、discard并销毁可恢复Preview World。

Implementation slices：实现source snapshot/preflight、participant capture/restore、world context、resource lease、quiesce顺序和terminal receipt；默认拒绝不可checkpoint的写系统。

Testing stage：clone/snapshot roundtrip、plugin component preservation、partial failure rollback、close/reload/crash/device failure和leak tests；从Runtime底层到Editor handle逐层验证。

Exit evidence：重复start/stop不改变authoring source，失败无残留world、voice、task或demand lease。

### ED69-M3：Per-session Time Domain、Pause/Step/Speed

Goal：在Preview session内复用real/virtual/fixed clock，完成transport状态机。

Implementation slices：实现pause/resume/step/reset/seek/reseed、max delta、fixed debt和transition sequence；明确real-time maintenance与virtual-time stage。

Testing stage：deterministic time property tests、0/slow/fast speed、sleep resume、step exactly-once、fixed overstep边界、pause期间allowed maintenance和stale token rejection。

Exit evidence：相同seed/trace产生相同clock与world hash；任意step receipt可解释实际执行量。

### ED69-M4：Selective subsystem provider与恢复

Goal：按依赖闭包接入Animation、Particle、Physics和Audio，拒绝未准入副作用。

Implementation slices：建立provider registry、policy resolver、stage/pause behavior、budget和restore participant；逐个接第一方provider，不做Editor旁路。

Testing stage：每个provider的play/pause/step/stop/reset、组合依赖、disabled capability、partial startup rollback、state restore和cross-provider order tests。

Exit evidence：四类provider均有独立与组合证据，scripts/network/file/process默认fail-close。

### ED69-M5：Frame Demand resolver、activity与低功耗Host

Goal：多owner需求合并后通过现有WaitUntil驱动，hidden/background按政策降频。

Implementation slices：接入contribution registry、deadline/Hz/budget、view aggregation；补全focus gain、occlusion、minimized、pane visibility和remote/power activity；删除任何临时busy loop。

Testing stage：owner add/remove/expiry、deadline ordering、multi-view merge、hidden/covered/minimized、focus transition、remote/background override和no-demand indefinite wait tests。

Exit evidence：无可见需求时事件循环Wait；有效需求不丢帧且不被无关owner覆盖；异常owner自动退休。

### ED69-M6：Render invalidation、Audio listener与generation currentness

Goal：simulation generation只重绘正确view，audio listener与camera/focus一致。

Implementation slices：将simulation receipt映射为view-scoped RENDER invalidation；保持image poll paint-only；建立listener lease、mute/output policy和stale product rejection。

Testing stage：time-only animation/particle render、multi-view currentness、dropped/stale extract、listener handoff、mute/background/device loss与render submission failure recovery。

Exit evidence：每次可见世界变化都可追溯到receipt/extract/image generation；旧view或旧world产品不可展示/发声。

### ED69-M7：Editor控制面、override与持久化

Goal：产品提供可检查Realtime、Pause/Step、Speed、Subsystem和Audio控制。

Implementation slices：typed command/binding、toolbar/menu/status、owner override stack、requested/effective projection、disabled reason、keymap/accessibility和versioned preference migration。

Testing stage：route/automation parity、nested override restore、save/reopen、unknown provider preservation、locale/keyboard/screen-reader、view duplicate/close/reopen与Play transition tests。

Exit evidence：UI不猜测状态；所有控件展示Runtime receipt；临时工具退出后精确恢复用户意图。

### ED69-M8：Diagnostics、budget、fault与scale

Goal：对clock、tick、subsystem、demand、throttle和restore形成bounded observability。

Implementation slices：viewport-qualified metric schema、sampling budget、timeline markers、override provenance、fault injection与memory/resource census；不复制Editor25 profiler authority。

Testing stage：10/100 view demand、100K scene、long-running animation/particle/physics/audio、plugin panic、asset reload、sleep/resume、device loss、memory growth和diagnostic disabled-cost tests。

Exit evidence：超预算会degrade/fail-close并给出receipt，不静默空转或隐藏错误。

### ED69-M9：单一产品硬切与跨引擎资格

Goal：删除旧旁路，完成同语义正确性、表现、性能和功耗资格。

Implementation slices：所有Scene View、tool override与asset preview迁移共享service；删除直接CoreHandle time mutation和无界redraw consumer；冻结benchmark recipe与baseline。

Testing stage：Windows优先的完整Editor/GPU/real audio/physics/particle矩阵、fault/soak/power、multi-window/multi-view、save/reopen和同场景Unreal/Godot/Fyrox/Bevy/Unity Graphics可比证据；Cargo验证按里程碑统一批次执行。

Exit evidence：48项门禁全部Pass，性能结论包含同硬件/同画质/同simulation/统计置信与回归阈值，才能声称达到或超过参考引擎。

## 9. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED69-G01 | Preview与Play具有不同stable session identity和profile | Fail |
| ED69-G02 | View/Document/World/Provider generation完整限定所有请求与回执 | Fail |
| ED69-G03 | Runtime是Preview World mutation与tick唯一authority | Fail |
| ED69-G04 | Preview创建失败原子回滚且不改变authoring world | Fail |
| ED69-G05 | Stop/Close/Reload可quiesce并释放全部lease/resource | Fail |
| ED69-G06 | 用户Realtime意图可持久化且与临时override分离 | Fail |
| ED69-G07 | owner-scoped override支持嵌套、异常退休和精确恢复 | Fail |
| ED69-G08 | Pause/Resume transition有sequence和effective receipt | Fail |
| ED69-G09 | Step exactly-once且报告variable/fixed执行量 | Fail |
| ED69-G10 | Time Scale、max delta和fixed timestep有validated scope | Fail |
| ED69-G11 | Reset/Seek/Reseed得到deterministic state receipt | Fail |
| ED69-G12 | 多Preview session不共享裸可变Core clock | Fail |
| ED69-G13 | Subsystem policy具有stable id、依赖闭包与pause behavior | Fail |
| ED69-G14 | 未准入script/network/file/process副作用fail-close | Fail |
| ED69-G15 | Animation preview play/pause/stop/step/currentness通过 | Fail |
| ED69-G16 | Particle CPU/GPU preview reset/reseed/restore通过 | Fail |
| ED69-G17 | Physics fixed-step、debt、restore与collision policy通过 | Fail |
| ED69-G18 | Audio listener/mute/output/device lifecycle通过 | Fail |
| ED69-G19 | Provider partial startup failure原子回滚 | Fail |
| ED69-G20 | Frame demand contribution带owner lease、scope和expiry | Fail |
| ED69-G21 | 多owner deadline/continuous/Hz按确定规则合并 | Fail |
| ED69-G22 | 无需求时native loop进入无限Wait而非轮询 | Fail |
| ED69-G23 | visible/focused Scene view按目标Hz更新 | Fail |
| ED69-G24 | covered/hidden pane按policy停止或降频 | Fail |
| ED69-G25 | occluded/minimized window按policy停止或降频 | Fail |
| ED69-G26 | focus regain不产生无界catch-up或delta jump | Fail |
| ED69-G27 | background/remote/power override受预算与owner限制 | Fail |
| ED69-G28 | 多view可独立Realtime/Pause并正确聚合共享world需求 | Fail |
| ED69-G29 | simulation generation只标记对应ViewInstance render dirty | Fail |
| ED69-G30 | time-only world变化产生新extract与current image | Fail |
| ED69-G31 | viewport image polling保持paint-only且不反向tick world | Fail |
| ED69-G32 | stale world/view/extract/image generation被拒绝 | Fail |
| ED69-G33 | render/device failure不冻结未标注旧图或泄漏需求 | Fail |
| ED69-G34 | World replace/asset reload/plugin reload有terminal choreography | Fail |
| ED69-G35 | requested/effective/capability/fallback/denial在UI可检查 | Fail |
| ED69-G36 | Toolbar、menu、keymap与automation使用同一typed intent | Fail |
| ED69-G37 | transport controls具备accessible name/focus/disabled reason | Fail |
| ED69-G38 | per-user/workspace/view profile可versioned save/reopen/migrate | Fail |
| ED69-G39 | unknown provider preference可保留且不静默启用 | Fail |
| ED69-G40 | clock/step/debt/Hz/throttle/provider cost有qualified diagnostics | Fail |
| ED69-G41 | diagnostics关闭时热路径成本有上界 | Fail |
| ED69-G42 | plugin contribution有capability、budget、fault和retirement | Fail |
| ED69-G43 | 10/100 viewport需求与visibility scale测试通过 | Fail |
| ED69-G44 | 100K scene与长时间四子系统soak无无界增长 | Fail |
| ED69-G45 | sleep/resume、device loss、audio loss、panic fault矩阵通过 | Fail |
| ED69-G46 | 固定recipe的重复运行world/render hash在容差内稳定 | Fail |
| ED69-G47 | Windows真实Editor/GPU/audio/physics产品矩阵通过 | Fail |
| ED69-G48 | 同硬件同画质同simulation跨引擎表现/性能证据达到目标 | Fail |

## 10. 测试与验证矩阵

### 10.1 Runtime unit / property / fuzz

覆盖session/view/world/provider identity、schema version、lease expiry、time transition、step token、fixed debt、subsystem dependency、side-effect admission、restore participant、demand merge和receipt immutability。属性测试必须证明pause不推进virtual/fixed、step exactly-once、max delta/budget有界、stale generation永远拒绝。

### 10.2 Preview World与subsystem integration

以同一source snapshot分别运行Animation、Particle CPU/GPU、Physics、Audio和组合政策，覆盖start/pause/step/stop/reset、partial failure、asset reload、plugin unload、world close和exact restore。不得以手工构造空World或单一happy path替代普通project/scene入口。

### 10.3 Editor/Host product integration

从toolbar、menu、keymap、automation和tool override发送同一intent，验证effective receipt、per-view projection、nested restore、save/reopen、Play transition、multi-view、covered tab、focus/occlusion/minimized、WaitUntil和view-scoped invalidation。

### 10.4 Render/audio currentness与fault

验证simulation/world/extract/image generation一一可追踪，旧产品被拒绝；renderer/device/audio failure、voice cleanup、listener handoff、present retry、plugin panic和session teardown不泄漏资源或保留continuous demand。

### 10.5 Performance、power、soak与跨引擎比较

固定scene、camera、time trace、seed、subsystem policy、quality、resolution、hardware/driver、warm-up和采样窗；分别记录visible/hidden/background的CPU/GPU、wake count、fixed debt、memory、audio/physics/particle cost与功耗。比较Unreal等引擎时必须同语义、同画质、同simulation，报告分位数、置信区间和回归阈值。

## 11. Owner路由与非重复计数

| 范围 | Canonical owner | Editor69处理 |
|---|---|---|
| Play/PIE/Game View/authoring-play transition | Editor07 | 复用底层session原语，不重复Play问题 |
| Animation asset toolkit与Runtime animation内部 | Editor14 / Runtime08C | 只拥有Scene Viewport preview adapter/demand |
| VFX/Particle asset toolkit与Runtime particle内部 | Editor15 / Runtime99D | 只拥有通用view session接入 |
| Sound asset toolkit与Runtime audio内部 | Editor17 / Runtime08B | 只拥有view listener/output policy |
| Physics asset toolkit与Runtime physics内部 | Editor18 / Runtime08A | 只拥有preview sandbox/fixed policy/restore |
| 通用real/virtual/fixed clock与determinism | Runtime22 | 复用clock，新增per-preview ownership |
| Runtime diagnostics工具产品 | Editor25 | 只定义viewport-qualified指标consumer |
| Gateway lifecycle/backpressure/reconnect | Editor47 | 只新增preview stable handles/receipt |
| Tool owner/capture/lease | Editor53 | 复用tool identity，新增realtime override lease |
| ViewInstance/render product/currentness | Editor58 | 消费identity，新增simulation generation关系 |
| Camera/navigation与visualization profile | Editor66 / Editor68 | 消费camera/listener与显示状态，不重复finding |
| Editor69新增30项P1、8项P2 | 本报告 | 唯一计数edit Scene Viewport realtime preview产品缺口 |

## 12. 状态与产出记录

每个里程碑测试通过后记录一次；实现切片不单独写入产出记录。

| 里程碑 | 范围 | 状态 | 完成日期 | 验证批次 / 残余风险 |
|---|---|---|---|---|

## 13. 最终判定

当前Zircon Scene Viewport是按失效重绘的静态authoring view，Play session是唯一连续Runtime tick入口；它不是Realtime Preview或Preview Simulation产品。底层时钟、WorldDriver、frame demand与WaitUntil证明项目不需要从零重写，但这些能力尚未形成per-view session、隔离world、selective subsystem、activity budget、generation receipt和产品控制面。

正确整改顺序必须从Runtime identity/Preview World/Time/Subsystem policy开始，再接Frame Demand与Host activity，最后开放Editor toolbar和persistence。禁止先加Realtime bool、在Editor里直接调用CoreHandle clock、无界request redraw、全量tick authoring world，或用Enter Play冒充Scene Viewport实时预览。

本报告完成current-source review，不代表实现完成。30项P1、8项P2和48个资格门保持Open/Fail，直到代码、产品、动态验证、故障/规模/功耗证据和同语义跨引擎基线逐项关闭。
