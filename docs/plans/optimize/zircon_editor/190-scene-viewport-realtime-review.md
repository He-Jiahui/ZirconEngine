---
title: Editor Scene Viewport Realtime Update、Preview Simulation、Time Domain、Pause/Step、Animation/Particle/Physics/Audio、Visibility Throttling、Invalidation、Performance 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor190
review_date: 2026-08-28
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/binding/viewport
  - zircon_editor/src/ui/workbench/state/editor_state_viewport.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/core/gateway/session
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/app/play_preview_redraw.rs
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/assets/ui/editor/host/scene_viewport_toolbar.zui
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/time/product_policy.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/registry/frame_activity.rs
  - zircon_runtime/src/dynamic_api/session/registry/frame_demand.rs
tests:
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/gateway/session/frame_demand.rs
  - zircon_editor/src/tests/host/binding_dispatch/viewport.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/viewport
  - zircon_editor/src/tests/host/retained_viewport_toolbar_pointer
  - zircon_editor/src/tests/host/retained_window/generic_host_boundary.rs
  - zircon_editor/src/tests/host/retained_window/native_viewport_image.rs
  - zircon_editor/src/tests/host/template_runtime/scene_viewport_toolbar_runtime_projection.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/workbench/chrome_snapshot/viewport_settings.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_demand.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Viewport Realtime Update、Preview Simulation、Time Domain、Pause/Step、Animation/Particle/Physics/Audio、Visibility Throttling、Invalidation、Performance 与 Product Integration 当前源码复核

## 1. 结论

Editor69之后，Runtime时间内核发生了实质性重构，旧报告中“`CoreHandle`持有session级共享virtual/fixed可变时钟”的事实已经过时。当前`RuntimeTimeAuthority`只拥有单调real clock和“新建Level默认策略”；每个`LevelSystem`各自持有`WorldTimeController`、`Time<Virtual>`、`Time<Fixed>`、policy generation和fixed debt。固定步以不可复制的事务令牌执行，commit/abort校验world generation，失败保留未提交debt和tick identity，插值只观察已提交状态。这是工程级基础，必须保留。

时间类型也从三个裸时钟扩展为versioned domain taxonomy。`ClockDomainId`登记MonotonicReal、WallUtc、WorldVirtual、WorldFixed、Input、Render、Audio、Network、Media和EditorPreview；stamp带domain、unit、epoch与source generation。`TimePolicy`验证max delta、relative speed和fixed timestep，`ProductTimePolicy`提供Client/Headless/Editor/Test profile、fixed-step budget与稳定digest。当前只有MonotonicReal、Virtual、Fixed真正实例化，`EditorPreview`仍只是枚举目录项，不是预览时间域。

动态帧需求和原生低功耗链同样是真实基础。Runtime会把pending asset reload或active animation转成`Immediate`，否则为`Idle`；`FrameDemandAccumulator`按Immediate优先、最早deadline、Idle中性规则确定性合并，并在tick失败时清除未发布需求。Editor Host把需求映射到OnDemand、SleepUntil或Continuous，最长deadline限制为60秒；原生事件循环在没有任何deadline时明确使用`ControlFlow::Wait`。新视口图像只触发paint-only区域失效，不会反向tick世界。

但Scene Viewport Realtime Preview产品仍不存在。`ViewportCommand`、settings、controller、toolbar projection和ZUI均没有Realtime、Pause、Resume、Step、Time Scale、Fixed Step、Simulation、Audio Listener或Mute。目标类型`PreviewWorldSession`、`PreviewTimeDomain`、`PreviewSubsystemPolicy`、`FrameDemandContributionRegistry`、`ViewportPreviewSessionRegistry`、`ViewportRealtimeProfile`、`ViewportActivityPolicy`、`ViewportAudioListenerLease`、`EffectiveViewportPreviewReceipt`、`ViewportPreviewControlIntent`和`PreviewSimulationReceipt`在当前tracked与untracked Rust语料均为零。

`EditorHostEventController::pump_runtime_event_consumers`仍只在active Play consumer、`WorldDomain::Play(instance)`和play gateway同时存在时调用`tick_frame()`。普通Edit authoring world只响应mutation/invalidation并重建extract，不进入Runtime时钟与`WorldDriver`。因此per-Level时间内核虽已具备暂停、速度、fixed budget和失败事务，Editor没有预览会话可调用它，也没有隔离world、subsystem准入、step receipt、恢复或副作用边界。

Window现在同时处理focus gain与loss，这是相对Editor69的窄进展；gain只通知owner，loss只取消交互。代码没有`WindowEvent::Occluded`调度分支，也没有pane covered/hidden、minimized/zero-size、remote/power或逐视口Hz政策。Frame demand虽然能无身份合并多个瞬时请求，却没有owner、lease、scope、expiry、撤销和provenance；异常producer或多个Scene view不能独立解释和退休需求。

本轮不新增P0。Editor69的30项P1当前为 **16 Open / 14 Partial**，8项P2为 **6 Open / 2 Partial**；48门为 **34 Fail / 12 Partial / 2 Pass**。两个Pass仅是当前源码可直接证明的Host无限Wait和viewport image paint-only边界，不代表Realtime Preview产品完成。

本轮只做review，未修改production Rust，未运行Cargo、Editor、GUI/GPU、真实animation/particle/physics/audio、save/reopen、multi-view、background/minimize、fault/scale/soak/profile/power或同硬件跨引擎benchmark。Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。当前不能声称该域功能、表现或性能达到或超过Unreal。

## 2. 审查边界与冻结语料

### 2.1 Current working tree

主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。本报告以2026-08-28读取时当前磁盘为事实源；时间、dynamic session、viewport、host/window范围包含大量其他会话的modified与untracked实现。本轮不回退、不格式化、不吸收这些代码，只按其实际行为更新review。

MVP baseline recovery仍为`in_progress`。本报告是后续RED、架构拆分和hard cutover输入，不是实现或动态验证receipt。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Editor viewport/product | **134 / 9,656 / 8,791 / 344,667 / 73** | viewport全树、binding、workbench state与ZUI；包含当前untracked拆分文件 | `ec4f4d46f639f0cb855da018527da91b80c21e523fe5b9baed9b96eb18ef904d` |
| Editor cadence/host | **116 / 14,797 / 13,584 / 553,950 / 164** | gateway session、Play tick owner、host lifecycle、image redraw与native window | `2208067e93800835a0b2bb9fa062ab47095b489d1473898d7668fdc823bf1496` |
| Runtime time/frame demand | **28 / 4,332 / 3,830 / 149,753 / 28** | domain、policy、per-Level clocks、WorldDriver、dynamic demand与accumulator | `92d2f25b81c29b2546e8ae66e944c8fcbe063b12daeb8f816c18336fd6f9471d` |
| Focused tests | **22 / 4,933 / 4,480 / 172,136 / 116** | viewport route、Host wait/image、dynamic demand、world time与fixed transaction | `365b96efea065d409adcc6d6e2f4dd1b8bcd4390c842a673887e9ddecbb112e8` |
| Unreal selected set | **9 / 24,381 / 20,370 / 893,941 / 0** | persistent/override realtime、bounded frames、Preview World、audio/physics与config | `0c131dfdbd6a008a9dbb38730382496359e71967008e9ca18dbd79f35bfbbc47` |
| Godot selected set | **4 / 14,805 / 12,525 / 543,729 / 0** | viewport update mode、visibility process、listener/Doppler、state与perf | `415b4afea3cc9b3758b47261007412e26dc239b88a583a1fd6b52483c0853828` |
| Fyrox selected set | **7 / 9,399 / 8,531 / 359,751 / 9** | fixed editor loop、GraphUpdateSwitches、音频/粒子恢复与动画transport | `9d990effeb8bba3208dc6ea50570effb606e4aaab957761053ac9e2d9924c2b1` |
| Bevy selected set | **6 / 3,076 / 2,695 / 111,747 / 38** | focus-aware loop、virtual/fixed time、run condition与schedule stepping | `ff94c86ff5a9c75bb4f3b88ecfc21a6e42799766596d0dc6b98376f185a3a863` |
| Unity Graphics selected set | **6 / 2,430 / 2,083 / 90,201 / 0** | scoped refresh restore、background/60Hz/culling、VFX transport与scheduler | `bf440c85768e0f6ec2562065b2170d0456c389d72b3134c82b58d7fb71760d5b` |

fingerprint按规范化相对路径排序，将每个`path + newline + file SHA-256 + newline`聚合后再做SHA-256，只证明本轮working-tree选择集。Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal跟随主workspace。

### 2.3 Owner边界

Editor190只刷新Editor69拥有的edit Scene Viewport Realtime意图、Preview Simulation会话、per-view time/activity、pause/step、选择性subsystem、frame-demand contribution、render invalidation、listener、persistence、effective receipt、diagnostics与资格。

Editor07继续拥有Play/PIE/Game View；Editor14/15/17/18拥有Animation/VFX/Sound/Physics资产工具；Editor25拥有通用性能工具；Editor47拥有gateway；Editor53拥有通用tool lease；Editor179/58拥有ViewInstance/render product currentness。Runtime08A/08B/08C/22/99D拥有子系统和通用时间内核。本文只定义这些owner接入Scene Viewport Preview必须满足的session合同，不重复计数其内部缺陷。

## 3. Editor69之后的事实变化

| Editor69旧事实 | 当前源码事实 | 本轮判定 |
|---|---|---|
| Core持有共享real/virtual/fixed bundle | Core只持有monotonic real和新Level默认policy；virtual/fixed由每个Level独立持有 | 旧结论撤销，per-preview ownership仍Partial |
| 时间只有real/virtual/fixed分类 | versioned domain registry含10类domain与unit/epoch/source generation stamp | Runtime taxonomy完成，EditorPreview实例仍缺 |
| fixed loop只有accumulator/drain | begin/commit/abort事务、stable simulation tick、world generation fence、committed interpolation | deterministic foundation显著增强 |
| Pause是Core全局API | Level拥有pause/unpause与policy transaction；paused时只运行显式monotonic-real system | world scope正确，preview transport仍缺 |
| Focus只处理loss | gain和loss均路由；gain仅owner acknowledgement | activity evidence由Missing升Partial |
| frame demand只有单一结果 | session accumulator可确定性合并多个调用 | merge primitive存在，owner registry仍缺 |
| image poll只paint-only | 仍保持paint-only，且viewport image resource key带generation | 单向边界可判Pass，simulation bridge仍缺 |
| edit world不tick | 仍只有Play gateway进入tick | 核心产品缺口不变 |

## 4. 当前实现拓扑

### 4.1 Runtime时间authority已经按world隔离

`RuntimeTimeAuthority`的policy只影响之后创建的Level。`WorldTimeController`保存独立virtual/fixed clock、policy generation和active fixed transaction；Level公开pause/unpause和policy apply。两个Level不再因为一个CoreHandle变速而互相污染，这是正确架构方向。

边界仍不够：一个Level没有Preview Session identity、source world epoch、owner generation或checkpoint；`ClockDomainId::EditorPreview`无法创建marker/time实例。若多个view共享同一个authoring Level，仍不能各自Pause、Step或使用不同速度。

### 4.2 Fixed-step failure语义是真实工程底座

`WorldDriver`在每个fixed stage前取得`ActiveFixedStep`，成功后commit，错误与Drop路径abort。`SimulationTickId`由world generation、fixed epoch和tick index组成。测试证明第二步失败只提交第一步，15ms debt保留，零delta重试提交相同下一步；world replacement会拒绝旧事务；普通系统只读取已提交插值。

这些能力可直接成为Preview Step执行内核，但目前没有用户step token、transition sequence、variable/fixed执行receipt、reset/seek/reseed或跨animation/particle/physics的统一时间轴。

### 4.3 Paused schedule已有明确时间政策

默认virtual-time system在pause时不运行，显式`SceneSystemTickPolicy::monotonic_real()`仍可执行；event/message maintenance保持边界。该设计优于全局停表，但尚未扩展为Preview subsystem allow/deny、pause behavior、side-effect class和dependency closure。

### 4.4 Dynamic demand与Host wait边界正确但无owner

Dynamic session优先为pending asset reload请求Immediate，否则读取animation continuous状态。Accumulator会选Immediate或最早After，并在consume后归零。ABI校验version/kind/delay，超长delay夹到60秒；失败tick不发布旧需求。

该结构不能回答“谁在要求连续帧”。producer没有stable id、scope、lease、expiry、priority、target Hz或budget，撤销只能整体consume。Editor又只有Play gateway producer，所以它不能承载多Scene view、工具和插件的独立需求生命周期。

### 4.5 原生事件循环具备真正的idle sleep

`about_to_wait_impl`合并runtime、maintenance、input timer、lifecycle、resize、present retry和presenter upgrade deadline。存在deadline时使用WaitUntil，无deadline时使用无限Wait；focused source test还守卫`about_to_wait`不得无条件request redraw。这个门禁已在源码层满足。

### 4.6 Edit world仍没有simulation producer

Host tick会更新scene mode、同步Play input/camera、调用`pump_runtime_event_consumers`并应用frame demand。后者没有active Play consumer时返回OnDemand；只有attached `WorldDomain::Play`才调用gateway `tick_frame()`。Edit world没有等价preview gateway、session registry或subsystem adapter。

### 4.7 Render与paint职责分开，但缺generation桥

simulation成功后应产生qualified world generation，再只标记对应ViewInstance的RENDER。当前render submission只在`render_dirty`时提取；新image到达只记录`PAINT_ONLY | VIEWPORT_IMAGE`并请求区域重绘。该paint-only职责是正确的，缺少的是`PreviewSimulationReceipt -> extract generation -> image generation`链。

### 4.8 Activity与产品控制面仍不完整

Window focus gain/loss已路由，但没有occlusion事件、zero-size/minimized判断与frame policy；Workbench pane visible/covered也不进入Runtime demand resolver。Scene Viewport toolbar没有Realtime或transport控件，settings没有profile，HUD没有clock/debt/effective Hz/provider cost，UI无法显示requested、effective、throttled、denied或degraded状态。

## 5. 五引擎参考结论

### 5.1 Unreal

`FEditorViewportClient`把持久`SetRealtime`与命名`AddRealtimeOverride`分开，并允许`RequestRealTimeFrames`只请求有限帧。Level viewport按实例恢复Realtime，RDP与PIE用具名override改变有效状态。`FPreviewScene::ConstructionValues`选择EditorPreview/GamePreview、audio与physics；析构停止音频、释放physics/world context。Zircon需要学习的是“用户意图、临时owner override、isolated world、有限帧请求和终止清理”五个合同，而不是一个Realtime bool。

### 5.2 Godot

SubViewport提供Disabled、Once、WhenVisible、WhenParentVisible、Always更新模式；3D editor viewport按`is_visible_in_tree()`同时启停process和physics process，并把audio listener、Doppler、frame time和信息显示保存为per-view状态。可见性必须参与有效调度，不能让全窗口共享一个Continuous值。

### 5.3 Fyrox

Fyrox editor用`GraphUpdateSwitches`选择physics、node更新、删除与pause；pause会同步sound context。Audio/particle preview保存原节点状态并在退出时恢复，animation toolbar提供transport和speed。规模虽小，但“选择性准入、精确恢复”比直接tick authoring world更接近正确产品。

### 5.4 Bevy

`Time<Virtual>`定义pause、relative speed与max delta，`Time<Fixed>`跟随virtual并维护overstep；Winit按focused/unfocused选择Continuous或Reactive low-power模式；Stepping可按schedule/system continue或step。它不是完整Editor参考，却证明时域、run condition、低功耗唤醒与单步应是可组合原语。

### 5.5 Unity Graphics

本地Graphics不是完整Unity Editor源码，只能使用局部可验证证据。LightPlacementTool保存并恢复SceneView `alwaysRefresh`；Probe subdivision在inactive时停止、把200Hz update限制为60Hz、裁剪不可见cell并逐帧预算；VFX board有play/pause/step/rate，detach恢复状态；debug scheduler按可见性pause。Zircon不能据此推断Unity完整SceneView内部实现，但必须达到同等级的owner restore与visibility budget纪律。

## 6. 差异矩阵

| 能力 | Zircon当前事实 | 工程级目标 | 判定 |
|---|---|---|---|
| Preview identity/world | 无edit preview session；Play独立存在 | qualified isolated/snapshot-backed Preview World | Missing |
| Time domain | per-Level virtual/fixed与domain taxonomy | per-preview instance、transport、receipt | Partial |
| Pause/Step/Speed | Level pause/policy/fixed事务 | per-view intent、exact step、reset/seek/reseed | Partial |
| Subsystem admission | schedule tick policy局部存在 | dependency-closed preview provider policy | Missing |
| Animation | Runtime continuous demand可用 | edit preview contribution/currentness | Partial |
| Particle/VFX | 通用Scene Viewport无生命周期 | CPU/GPU reset/reseed/restore/warm-up | Missing |
| Physics | fixed transaction/debt健全 | preview sandbox、collision policy、restore | Partial |
| Audio | 无Scene view listener/mute/route | listener lease、output/device lifecycle | Missing |
| Frame demand | deterministic anonymous accumulator | owner lease/scope/expiry/Hz/budget registry | Partial |
| Native idle | no deadline时ControlFlow::Wait | 保持并纳入多view resolver | Present |
| Activity | focus gain/loss存在 | pane/window/occlusion/minimize/power policy | Partial |
| Render currentness | image paint-only，resource key有generation | simulation到view/extract/image qualified chain | Partial |
| Persistence/UI | 无preview字段与transport | versioned profile与effective projection | Missing |
| Qualification | Runtime time tests强，edit preview tests为零 | correctness/fault/scale/power/benchmark | Partial |

## 7. Canonical finding状态

### 7.1 P1

#### ED69-P1-01 [Open]：没有Viewport Preview Session authority与qualified identity

Controller仍没有PreviewSessionId、ViewInstance/Document/SourceWorld epoch或owner generation。异步tick、step和subsystem回执无法证明属于当前view/world。

#### ED69-P1-02 [Open]：Edit authoring world从不进入Runtime tick链

只有Play domain调用`tick_frame()`；普通Scene Viewport继续只消费world invalidation与extract。

#### ED69-P1-03 [Open]：Realtime用户意图与临时owner override均不存在

没有持久toggle、命名owner、lease、嵌套覆盖、异常退休或恢复。

#### ED69-P1-04 [Partial]：Runtime已有world pause，Preview transport状态机缺失

Level pause/unpause与policy transaction是真实基础；UI仍不能表达Pause/Resume/Step，也没有transition sequence、pending/denied/effective receipt。

#### ED69-P1-05 [Partial]：Time Scale与Fixed Step有内核，Reset/Seek/Reseed无preview合同

TimePolicy验证speed、max delta、fixed timestep；Preview会话、timeline reset、seek、particle reseed和可观察回执仍为空。

#### ED69-P1-06 [Partial]：旧Core全局时钟缺陷已修，per-preview ownership未完成

virtual/fixed已下沉到每个Level，旧报告的Core共享污染结论撤销。一个Level上的多个preview仍无法拥有独立clock/session；EditorPreview domain只登记未实例化。

#### ED69-P1-07 [Open]：没有isolated preview world、checkpoint或rollback边界

直接tick authoring world仍会污染transform、script、particle、voice与资源状态；没有clone/snapshot、participant restore或Apply/Discard receipt。

#### ED69-P1-08 [Open]：没有selective Preview Subsystem Policy与依赖闭包

现有system tick policy只区分clock，不定义Animation/Particle/Physics/Audio/Script的准入、依赖、暂停行为与预算。

#### ED69-P1-09 [Open]：脚本、网络、保存和外部副作用没有preview admission

文件、网络、process、operation和project resource mutation没有capability sandbox或默认fail-close。

#### ED69-P1-10 [Partial]：Animation demand已闭合到Play，Scene Viewport consumer缺失

active sequence会请求Immediate、暂停后回Idle且失败不遗留需求；edit view没有session/owner贡献和结束撤销链。

#### ED69-P1-11 [Open]：Particle/VFX预览没有Scene Viewport生命周期编排

缺play/pause/stop/restart/seek/reseed、CPU/GPU capability、warm-up、loop、budget与退出恢复。

#### ED69-P1-12 [Partial]：Fixed-step事务已健全，Physics preview sandbox与恢复缺失

fixed budget、debt、commit/abort、world generation和插值守恒可复用；仍无physics2D/3D选择、collision policy、authoring body checkpoint和状态恢复。

#### ED69-P1-13 [Open]：Audio listener、mute与preview output route缺席

Scene camera没有listener lease，focus不决定owner，hidden/background没有输出政策，device/voice终止没有preview receipt。

#### ED69-P1-14 [Open]：Enter Play仍是唯一时间产品入口

Play切换独立runtime session；它不能替代保持authoring上下文、只准入选定能力的Realtime Preview。

#### ED69-P1-15 [Open]：Command、settings、chrome与projection没有requested/effective状态

产品链没有preview字段，无法显示Paused、Stepping、Throttled、Unavailable、Degraded或override owner。

#### ED69-P1-16 [Partial]：Time policy/domain receipt有基础，Preview capability receipt缺失

policy commit、domain stamp和generation可表达部分事实；请求的subsystem、fallback、denial、provider/world/view generation仍无统一回执。

#### ED69-P1-17 [Partial]：Host demand链可复用，但入口仍由Play独占

OnDemand/SleepUntil/Continuous到WaitUntil已完整；没有edit preview producer或独立session合并入口。

#### ED69-P1-18 [Partial]：有匿名demand accumulator，没有多owner registry

Immediate/After/Idle合并规则确定且有测试；没有owner identity、lease、scope、expiry、priority、Hz、budget与单owner撤销。

#### ED69-P1-19 [Open]：时间推进不会产生view-scoped render invalidation

没有simulation generation到ViewInstance的映射，time-only world变化不会自动标记正确render target。

#### ED69-P1-20 [Partial]：Image paint-only边界正确，simulation generation桥缺失

poll只展示已产生图并使用generation资源键；没有simulation receipt到extract/image currentness的可追踪关系。

#### ED69-P1-21 [Partial]：Focus路由已补，完整activity policy缺失

gain/loss已处理，但没有native occlusion、minimized/zero-size、pane covered/hidden、remote/power或逐view activity snapshot。

#### ED69-P1-22 [Partial]：Max delta、fixed budget/debt存在，foreground/background预算缺失

Runtime能限制delta和每帧fixed step；没有target/background Hz、drop/skip、bounded catch-up和focus regain政策。

#### ED69-P1-23 [Open]：Multi-viewport没有独立policy与聚合规则

两个view不能独立Realtime/Pause，也不能按visibility、共享world和audio owner解析需求。

#### ED69-P1-24 [Open]：Preview profile没有真实持久化、schema与迁移

字段本身不存在，更没有user/workspace/view scope、version、migration、unknown provider preservation或crash restore。

#### ED69-P1-25 [Partial]：Runtime时间/physics diagnostics增强，viewport-qualified调度观测缺失

通用store已有单位化time/fps与physics状态，fixed clock也暴露debt；Scene HUD仍没有clock generation、effective Hz、owner、throttle、subsystem cost和freshness。

#### ED69-P1-26 [Open]：World replace、reload、device与preview failure没有统一生命周期

Play和render各有局部错误路径，但不存在preview先撤销demand/listener、再quiesce/restore/destroy的terminal choreography。

#### ED69-P1-27 [Open]：Plugin不能安全贡献realtime需求或preview subsystem

没有stable descriptor、capability closure、budget、fault domain、owner generation和lease retirement。

#### ED69-P1-28 [Partial]：Clock/tick identity稳定，deterministic reset/reference state缺失

domain epoch、source generation、policy digest与SimulationTickId是可保留基础；initial snapshot、seed、asset generation、input trace和world/render hash仍未冻结。

#### ED69-P1-29 [Partial]：Runtime time测试工程化，edit preview正确性测试仍为零

测试覆盖world独立、pause、fixed budget、commit/abort、debt和demand；没有创建Viewport Preview Session、Pause/Step产品链、activity切换、restore或multi-owner需求。

#### ED69-P1-30 [Open]：没有性能、功耗、故障与长时间资格证据

无visible/hidden Hz、wake count、10/100 view、100K scene、四subsystem soak、sleep/device/audio/plugin fault或同语义跨引擎基线。

### 7.2 P2

#### ED69-P2-01 [Open]：Preview控制没有stable descriptor与可扩展transport vocabulary

继续向闭集ViewportCommand堆枚举不能表达provider controls、availability和automation schema。

#### ED69-P2-02 [Partial]：Product time preset已集中，Preview默认值与scope owner缺失

versioned Client/Headless/Editor/Test policy集中定义fixed budget与clock policy；Realtime、background Hz、audio和subsystem preset仍无user/project/view owner。

#### ED69-P2-03 [Open]：Serializable viewport state容易被误读为已持久化产品

可编码DTO不等于schema/migration/atomic save/reopen，preview字段甚至不存在。

#### ED69-P2-04 [Open]：临时override缺少provenance/history设计

需要owner display name、reason、priority、start/expiry、effective result与retirement history。

#### ED69-P2-05 [Open]：Transport控件无keyboard、accessibility与disabled-reason规范

Play/Pause/Step/Restart/Speed必须有统一focus、checked/pending、accessible name和不可用原因。

#### ED69-P2-06 [Partial]：通用diagnostic已有unit/path，viewport freshness与成本schema缺失

现有time/fps和physics数据可复用；仍需source、view/session generation、window、timestamp、sampling budget和disabled-cost。

#### ED69-P2-07 [Open]：没有跨后端、平台和引擎的可复现基线recipe

应冻结scene、camera、time trace、seed、policy、quality、hardware/driver、warm-up与统计阈值。

#### ED69-P2-08 [Open]：Unity Graphics镜像不包含完整SceneView owner实现

只采用本地可验证tool/VFX/debug scheduler证据，不推断缺失的Unity Editor内部行为。

### 7.3 状态统计

| 等级 | Open / Fail | Partial | Closed / Pass |
|---|---:|---:|---:|
| P1 | 16 | 14 | 0 |
| P2 | 6 | 2 | 0 |
| Qualification gates | 34 | 12 | 2 |

## 8. 目标架构与重构边界

### 8.1 Runtime唯一执行authority

建立`PreviewWorldSession`，identity至少包含PreviewSession、SourceWorld、SourceEpoch、PreviewGeneration与OwnerGeneration。创建必须在isolated clone或声明式snapshot上进行；所有可写participant必须支持capture/restore，否则默认拒绝。普通authoring world不得因UI bool直接进入完整game schedule。

`PreviewTimeDomain`应复用当前per-Level `WorldTimeController`，而不是再造clock。它增加session ownership、transition sequence、exact step token、reset/seek/reseed与receipt。`ClockDomainId::EditorPreview`必须从目录项变成可实例化且qualified的domain。

`PreviewSubsystemPolicy`以stable provider id声明Allowed/Denied/DiagnosticOnly、依赖、clock/stage、pause behavior、side-effect class、resource budget和restore participant。Animation、Particle、Physics和Audio作为第一方provider接入，Editor不得调用内部manager旁路。

`FrameDemandContributionRegistry`以owner lease接收scope、deadline/continuous、target/min Hz、priority、budget class和expiry，并输出immutable effective demand。当前匿名accumulator可作为resolver内部最小值运算，但不能继续承担registry。

### 8.2 Editor只拥有意图与活动事实

`ViewportPreviewSessionRegistry`以ViewInstance、DocumentSession与WorldEpoch定位Runtime handle。`ViewportRealtimeProfile`保存用户意图、subsystem preset、time scale、foreground/background budget和audio policy；UI、keymap、automation与tool override统一发送`ViewportPreviewControlIntent`。

`ViewportActivityPolicy`只采集pane visible/covered、window focused/occluded/minimized、active tab、remote/power preference并生成qualified snapshot，不直接tick。`ViewportAudioListenerLease`竞争view camera listener；实际device/voice由Runtime管理。

Toolbar与HUD只投影`EffectiveViewportPreviewReceipt`，明确requested/effective、Paused/Stepping/Throttled/Unavailable/Degraded、override owner、clock/world/provider generation和denial reason，不根据本地bool猜状态。

### 8.3 Host保持低功耗与单向失效

Host继续合并runtime/maintenance/input/lifecycle/present deadline并使用Wait/WaitUntil。simulation receipt产生新world generation时只标记对应ViewInstance RENDER；viewport image到达仍只paint。禁止image poll反向tick、无界request redraw或在Host维护第二套time/subsystem政策。

## 9. 分阶段重构计划

### ED69-M0：真实性、owner与RED基线

冻结Preview/Play/资产预览边界，新增RED证明edit world不tick、目标类型与产品controls缺失、activity/receipt缺失；同时把旧Core全局时钟描述从父报告实现输入中删除。

### ED69-M1：Stable identity、DTO、registry与receipt

定义session/view/world/provider identity、schema version、intent/effective DTO、provider descriptor、lease/expiry与stale rejection；unit/property tests覆盖generation复用、重复retire、invalid speed/timestep/budget。

### ED69-M2：Isolated Preview World与terminal lifecycle

实现source snapshot/preflight、participant capture/restore、resource lease、quiesce和terminal receipt；partial creation、close/reload/crash/device failure必须原子回滚且不改变authoring source。

### ED69-M3：Per-preview Time Domain与transport

将现有per-Level controller装入Preview Session，增加Pause/Resume/Step/Reset/Seek/Reseed sequence和receipt；保留fixed commit/abort/debt守恒，验证step exactly-once与sleep resume有界。

### ED69-M4：Selective subsystem provider

按依赖闭包逐个接Animation、Particle、Physics、Audio，定义pause/stage/budget/restore；scripts/network/file/process默认fail-close。组合失败必须回滚已启动provider。

### ED69-M5：Owner demand、activity与低功耗

把匿名accumulator收进owner registry resolver，补focus/occlusion/minimize/pane/remote/power activity，支持expiry与单owner撤销。无可见需求继续无限Wait，hidden/background按budget降频。

### ED69-M6：Render currentness与Audio listener

建立simulation/world/extract/image generation链与view-scoped invalidation；旧generation拒绝。实现listener handoff、mute/output policy、device failure和voice cleanup，保持image poll paint-only。

### ED69-M7：产品控制面、override与持久化

接通typed toolbar/menu/keymap/automation、requested/effective projection、nested owner override、disabled reason、accessibility与versioned user/workspace/view persistence。

### ED69-M8：Diagnostics、fault与scale

增加viewport-qualified clock/step/debt/Hz/owner/throttle/provider cost、sampling budget和fault injection；完成10/100 view、100K scene、四subsystem soak、sleep/device/plugin panic与memory census。

### ED69-M9：单一产品硬切与跨引擎资格

所有Scene View和asset preview迁移共享service，删除任何直接clock mutation、无界redraw和旁路provider。仅在同scene/camera/time/policy/quality/hardware统计基线下讨论达到或超过参考引擎。

## 10. 资格门

| Gate | 要求与当前证据 | 当前 |
|---|---|---|
| ED69-G01 | Preview与Play具有不同stable session identity/profile | Fail |
| ED69-G02 | View/Document/World/Provider generation限定请求与回执 | Fail |
| ED69-G03 | Runtime已有WorldDriver mutation/tick authority，Preview World尚不存在 | Partial |
| ED69-G04 | Preview创建失败原子回滚且不改变authoring world | Fail |
| ED69-G05 | Stop/Close/Reload quiesce并释放全部lease/resource | Fail |
| ED69-G06 | Realtime持久意图与临时override分离 | Fail |
| ED69-G07 | owner override支持嵌套、异常退休与精确恢复 | Fail |
| ED69-G08 | Level pause存在；Preview transition sequence/effective receipt缺失 | Partial |
| ED69-G09 | fixed commit/abort/tick identity健全；用户Step receipt缺失 | Partial |
| ED69-G10 | max delta/speed/fixed timestep已验证；preview scope缺失 | Partial |
| ED69-G11 | Reset/Seek/Reseed得到deterministic state receipt | Fail |
| ED69-G12 | virtual/fixed已per-Level；多Preview session identity仍缺 | Partial |
| ED69-G13 | Subsystem policy有stable id、依赖闭包与pause behavior | Fail |
| ED69-G14 | 未准入script/network/file/process副作用fail-close | Fail |
| ED69-G15 | Runtime animation demand/play-pause可测；edit preview/currentness缺失 | Partial |
| ED69-G16 | Particle CPU/GPU reset/reseed/restore通过 | Fail |
| ED69-G17 | fixed/debt事务通过；physics preview restore/collision policy缺失 | Partial |
| ED69-G18 | Audio listener/mute/output/device lifecycle通过 | Fail |
| ED69-G19 | Provider partial startup failure原子回滚 | Fail |
| ED69-G20 | Demand contribution带owner lease/scope/expiry | Fail |
| ED69-G21 | 匿名Immediate/deadline合并确定；multi-owner身份/撤销缺失 | Partial |
| ED69-G22 | 无任何deadline时native loop明确进入ControlFlow::Wait | Pass |
| ED69-G23 | visible/focused Scene view按目标Hz更新 | Fail |
| ED69-G24 | covered/hidden pane按policy停止或降频 | Fail |
| ED69-G25 | occluded/minimized window按policy停止或降频 | Fail |
| ED69-G26 | max delta有界；focus regain活动政策/产品测试缺失 | Partial |
| ED69-G27 | background/remote/power override受budget/owner限制 | Fail |
| ED69-G28 | 多view独立Realtime/Pause并正确聚合共享world需求 | Fail |
| ED69-G29 | simulation generation只标记对应ViewInstance render dirty | Fail |
| ED69-G30 | time-only world变化产生新extract/current image | Fail |
| ED69-G31 | image polling只记录paint-only invalidation且不tick world | Pass |
| ED69-G32 | stale world/view/extract/image generation全部拒绝 | Fail |
| ED69-G33 | render/device failure不冻结未标注旧图或泄漏需求 | Fail |
| ED69-G34 | asset reload/Play cleanup有局部基础；Preview terminal choreography缺失 | Partial |
| ED69-G35 | requested/effective/capability/fallback/denial在UI可检查 | Fail |
| ED69-G36 | Toolbar/menu/keymap/automation使用同一typed intent | Fail |
| ED69-G37 | transport controls有accessible name/focus/disabled reason | Fail |
| ED69-G38 | per-user/workspace/view profile可versioned save/reopen/migrate | Fail |
| ED69-G39 | unknown provider preference保留且不静默启用 | Fail |
| ED69-G40 | time/unit/debt基础存在；viewport/provider qualified diagnostics缺失 | Partial |
| ED69-G41 | diagnostics关闭时热路径成本有上界证据 | Fail |
| ED69-G42 | plugin contribution有capability/budget/fault/retirement | Fail |
| ED69-G43 | 10/100 viewport demand与visibility scale通过 | Fail |
| ED69-G44 | 100K scene与四subsystem soak无无界增长 | Fail |
| ED69-G45 | sleep/device/audio/panic fault矩阵通过 | Fail |
| ED69-G46 | stable tick/domain identity存在；重复world/render hash未证明 | Partial |
| ED69-G47 | Windows真实Editor/GPU/audio/physics产品矩阵通过 | Fail |
| ED69-G48 | 同硬件同画质同simulation跨引擎证据达到目标 | Fail |

Pass只表示当前静态源码直接满足该窄门；本轮未执行Cargo或真实窗口，因此动态回归仍应纳入实施验证。

## 11. 测试与验证矩阵

### 11.1 Runtime unit/property/fuzz

覆盖identity、schema、lease expiry、time transition、step token、fixed debt、subsystem dependency、side-effect admission、restore participant、owner demand merge和receipt immutability。属性测试必须证明pause不推进virtual/fixed、step exactly-once、abort不提交、stale generation永远拒绝。

### 11.2 Preview World与subsystem integration

从普通project/scene source创建isolated preview，分别运行Animation、Particle CPU/GPU、Physics、Audio及组合policy；验证start/pause/step/stop/reset、partial failure、asset/plugin/world reload和exact restore。禁止用空World happy path代替产品入口。

### 11.3 Editor/Host产品集成

Toolbar、menu、keymap、automation和tool override发送同一intent，验证effective receipt、nested restore、save/reopen、Play transition、multi-view、covered tab、focus/occlusion/minimize、Wait/WaitUntil与view-scoped invalidation。

### 11.4 Currentness、fault、scale与power

验证simulation/world/extract/image generation可追踪，旧产品拒绝；renderer/audio/device/plugin failure不泄漏资源或continuous demand。记录visible/hidden/background CPU/GPU、wake count、fixed debt、memory和subsystem cost，并运行10/100 view、100K scene与长时间soak。

### 11.5 跨引擎比较

冻结scene、camera、time trace、seed、subsystem policy、quality、resolution、hardware/driver、warm-up和采样窗。比较必须报告分位数、置信区间、功耗和回归阈值，不能只比较截图或平均FPS。

## 12. 最终判定

当前Zircon已经拥有比Editor69记录更成熟的Runtime时间内核：per-Level virtual/fixed clock、versioned domain/policy、fixed事务、world generation fence、deterministic tick identity、anonymous demand merge和真正的native idle wait都应保留。这些进展证明项目无需在Editor临时造timer、物理循环或busy redraw。

Scene Viewport本身仍是按失效重绘的静态authoring view，Play仍是唯一连续Runtime入口。缺失的是完整产品层：qualified Preview Session、isolated world、per-preview time、selective subsystem、side-effect admission、owner demand、activity budget、simulation-to-render generation、audio listener、effective UI、persistence与资格证据。

整改顺序必须从Runtime Preview identity/world/time/provider开始，再接owner demand、activity和currentness，最后开放Editor controls。禁止先加Realtime bool、直接tick authoring world、从image poll反向驱动simulation、允许插件无界request redraw，或用Enter Play冒充Scene Viewport Realtime Preview。

本报告完成current-source refresh，不代表实现完成。16项Open P1、14项Partial P1、6项Open P2、2项Partial P2以及34 Fail/12 Partial/2 Pass资格门必须由代码、产品、动态验证、fault/scale/power与同语义跨引擎证据逐项关闭。
