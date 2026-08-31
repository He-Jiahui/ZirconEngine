---
title: Runtime Cinematic Sequencer、Shot、Track、Binding、Hierarchy、Evaluation、Take Recorder 与 Movie Render Queue 当前工作树复审
category: zircon_runtime
report_id: Runtime176
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zy-runtime-cinematic-sequencer-sequence-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-network-save-scalability-product-integration-current-source-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/236-editor-cinematic-current-working-tree-sequencer-authoring-review.md
related_code:
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/tick.rs
  - zircon_runtime/src/animation
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_plugins/animation/runtime
  - zircon_plugins/timeline_sequence
  - zircon_plugins/sound/runtime/src/timeline
  - zircon_plugins/net/features/replication/runtime/src
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/operation
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieScene.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieSceneBinding.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieSceneSection.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/Evaluation/MovieSceneSequenceHierarchy.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieSceneSequencePlayer.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/Takes/Source/TakesCore/Public/TakeMetaData.h
  - dev/UnrealEngine/Engine/Plugins/MovieScene/MovieRenderPipeline/Source/MovieRenderPipelineCore/Public/MoviePipelineQueue.h
  - dev/godot/scene/resources/animation.h
  - dev/godot/scene/animation/animation_mixer.h
  - dev/godot/servers/movie_writer/movie_writer.h
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/editor/src/plugins/animation/command/mod.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime176 · Cinematic Sequencer 与离线输出当前工程化差距

## 1. 结论

当前 Zircon 没有可部署的 Cinematic runtime。运行时有 AnimationSequenceAsset、按 World 编译的 property writer、clip event 的有限预算采样，以及 camera stack、capture、sound timeline 等可复用部件；但没有独立的 cinematic source、shot/section/hierarchy、per-instance playback authority、pre-animated restore、camera cut、take capture 或 movie render queue。它们无法组成 source -> compiler -> immutable artifact -> playback instance -> frame receipt -> domain output 的产品闭环。

当前 animation sequence schema 只有 duration_seconds、浮点 frames_per_second、Vec bindings/tracks/keys、EntityPath 和可选 target_id。core/framework/animation/asset/sequence.rs:19-73 的 identity 仍依赖数组位置。runtime animation/sequence/compiled.rs:18-90 的编译产物保存 binding_index/track_index，apply 在 123-153 重新读取 source Vec；这不是具有稳定轨道身份、层级时间变换和实例覆盖的 MovieScene 类 artifact。

runtime/src/animation/module.rs:24-67 只注册 AnimationDriver、DefaultAnimationManager 和 trait facade；tick contract 默认返回空报告。runtime/src/core/framework/animation/timeline.rs:16-187 也是通用 timeline descriptor，仍以 seconds/fps 和 Vec tracks/events 表达，不能承担 rational frame time、section overlap、subsequence hierarchy 或 render sampling policy。

当前 runtime animation 选择集为 21 个文件、3,340 行、116,404 bytes、30 个 test declaration；timeline_sequence 插件全目录 10 个文件、1,131 行、41,707 bytes、15 个 test declaration。测试证明了局部采样、排序和失败不变更，不证明 cinematic playback、camera/audio/event/record/render 输出。

因此 Runtime150 的 4 Open / 20 Partial P1 结论继续有效。本报告新增 Runtime 专属 30 项 P1（25 Open / 5 Partial / 0 Closed）、10 项 P2（10 Open）和 24 道资格门（21 Fail / 3 Partial / 0 Pass），不新增独立 P0。Editor45 的作者文档与 Workbench 假成功仍由 Editor236/既有 Editor owner 计数。

## 2. 当前源码证据

### 2.1 Sequence 编译不是 Cinematic artifact

- compiled.rs:45-96 解析 target_id 或 EntityPath，生成 property writer；缺失 target 只进入 missing_tracks，未生成可发布的 dependency diagnostic 或 source map。
- compiled.rs:37-38 保存 binding_index 与 track_index；compiled.rs:138-151 每帧回读 sequence.bindings，source Vec 发生布局变化时实例语义改变。
- compiled.rs:106-117 只比较 binding catalog/schema generation；没有 source revision、provider generation、root context、subsequence hierarchy 或 instance generation。
- target.rs:4-20 可把字符串解析为当前 World 的 EntityId 或 EntityPath；没有 possessable/spawnable、level instance、binding override、authority 或 stable cross-session identity。

### 2.2 Playback、时间与事件边界不完整

animation manager 只提供 graph/state-machine/clip sampling；没有 sequence player registry、play/pause/seek/reverse/jump、completion mode、pre-animated state、subsequence evaluation 或 per-world scheduler owner。timeline descriptor 的 sanitized seconds/fps 会把 NaN/负值归零，却没有 tick resolution/display rate 分离、SMPTE/drop-frame、qualified frame、open/closed range和时间码。

clip_event.rs 有 max events、max bytes、resume cursor 和同时间排序测试，这是可保留的 bounded substrate；但 cursor 仍按 track_index，事件不是 cinematic section/shot event，不能保证跨层级 seek/reverse、loop、rollback、network/save replay 的 exactly-once。AnimationTickReport 的 emitted_events 是 Vec 文本记录，没有 frame receipt、causal id、target binding 或 adapter outcome。

### 2.3 Camera、Audio、Take 与 Render 没有共同 owner

camera stack/capture 只能提供当前渲染栈和有限读回；没有 authored camera-cut section、blend/easing、pre-animated camera restore、multi-camera evaluation、shot output或 render sample identity。sound timeline 由独立 audio clock 推进，不能与 cinematic qualified time、seek/reverse 或 shot range 原子同步。capture 和 render pipeline 没有 sequence/shot/frame/sample/tile/pass/checkpoint artifact；take recorder、movie pipeline queue 类型和 runtime executor均不存在。

### 2.4 Plugin 与产品装配缺口

timeline_sequence/plugin.toml 把包标为 editor_host、experimental、editor-only，依赖 animation timeline_event_track；README 声称注册 timeline editor、track descriptors、asset editor 和 operations。editor/src/plugin.rs:138-214 确实注册五个 descriptor operation、AnimationSequence augment 和三类 track descriptor，但没有 operation handler/factory。dist/src/lib.rs:28-40 明确 is_stateless=true、command/event manifest 为 None、invoke_command=None、bridge_methods 为空，不能执行或保存 editor command。

## 3. 参考引擎差异

Unreal MovieScene 将 sequence、binding、section、hierarchy、evaluation field、player、pre-animated state 和 entity system 分层；Take Recorder 保存 metadata、source、recording transaction；Movie Render Pipeline 将 queue、job、shot、frame/sample/tile/output 与 checkpoint 分开。Godot 的 AnimationMixer/AnimationPlayer 和 MovieWriter 将 playback、seek、audio/video capture 与固定帧率输出区分；Fyrox、Bevy 的 track/event/command 结构强调 stable target 与可撤销 mutation；Unity CameraCaptureBridge/AOV 说明 camera capture 必须携带 render context 与 output identity。当前 Zircon 只达到通用动画属性采样层。

## 4. P1 重构任务

| ID | 当前问题 | 必须完成 |
|---|---|---|
| RT-CINE-01 | 无 Cinematic domain owner | 新建 per-World CinematicPlaybackService、service handle、generation、shutdown/drain 合同。 |
| RT-CINE-02 | 无独立 source schema | 定义 CinematicSequenceSource、shot/section/track/binding/folder/marker schema、version/migration/provenance。 |
| RT-CINE-03 | Vec index identity | 为所有对象生成 stable id，Vec 只作为布局；删除/重排不改变引用。 |
| RT-CINE-04 | 无 dependency closure | 编译 manifest 覆盖 subsequence、animation/audio/camera/map/plugin/output preset。 |
| RT-CINE-05 | 无 deterministic compiler | 生成 immutable CinematicProgramArtifact、diagnostic/source map、LKG 和 atomic install。 |
| RT-CINE-06 | 无 rational frame time | 引入 FrameRate/FrameTime/SubFrame、tick/display rate、overflow 和 canonical compare。 |
| RT-CINE-07 | 无 range/section语义 | 统一 playback/work/view/render/section range、overlap、trim、pre/post-roll、easing。 |
| RT-CINE-08 | 无 hierarchy | 编译 root/local time transform、offset/scale/loop/warp、parent/child 与 inverse failure。 |
| RT-CINE-09 | 无 evaluation field | 生成 interval evaluation field、phase schedule、deterministic ordering 和 bounded work budget。 |
| RT-CINE-10 | 无 playback instance | 建立 root context、binding override、spawn register、pre-animated store、completion/stop/error restore。 |
| RT-CINE-11 | 无 stable target resolver | possessable/spawnable/level-instance target 具 world/authority/generation admission。 |
| RT-CINE-12 | 无 camera adapter | camera cut/blend/easing/multiview/pre-animated restore 走 typed frame request/receipt。 |
| RT-CINE-13 | 无 animation adapter | sequence/graph/pose 与 cinematic frame atomic 对齐，禁止直接写 Transform 绕过 receipt。 |
| RT-CINE-14 | 无 audio adapter | 统一 qualified time、seek/reverse/loop、audio clock drift 和 sound event receipt。 |
| RT-CINE-15 | 无 event adapter | marker payload、causal id、crossing/reverse/loop、exactly-once 与 bounded queue。 |
| RT-CINE-16 | 无 gameplay/VFX adapter | Gameplay Cue、particle/VFX、UI 与 cinematic section 具有 typed provider 和 cancellation。 |
| RT-CINE-17 | 无 runtime scheduler integration | service 纳入 world phase、fixed-step、priority/fairness、cancel/deadline/backpressure。 |
| RT-CINE-18 | 无 camera/audio/render output identity | 每帧携 sequence/shot/frame/sample/pass/tile/color-space/PTS。 |
| RT-CINE-19 | 无 Take Recorder | 定义 source discovery、metadata、timestamped buffer、journal/staging、crash recovery、take artifact。 |
| RT-CINE-20 | 无 Movie Render Queue | 定义 queue/job/shot/frame/sample/tile/output preset、checkpoint、resume、cancel 和 output receipt。 |
| RT-CINE-21 | 无 network/save/replay participant | playback state、qualified time、binding/shot state、event cursor、take/render checkpoint 可复制、存档、重放。 |
| RT-CINE-22 | plugin dist stateless | 为 runtime/editor plugin 提供 command/bridge/event manifest、handler、provider closure 与 ABI conformance。 |
| RT-CINE-23 | 缺失错误被降级为统计 | compile/apply/capture/render 失败必须返回结构化 receipt，禁止 continue 或丢弃结果。 |
| RT-CINE-24 | 无 resource budgets | graph depth、section count、events、targets、samples、tiles、bytes、GPU readback 和 queue length 有上限。 |
| RT-CINE-25 | 无 scale/fault evidence | 100K tracks、1K simultaneous instances、long take、device loss、network reorder、crash/reopen、soak、P99 benchmark。 |
| RT-CINE-26 | camera cut 由启发式推断 | authored cut 必须来自 section artifact，不得用运动阈值替代语义事件。 |
| RT-CINE-27 | capture mailbox 无 provenance | readback/encoding 绑定 frame generation、PTS、format、stride、color space 与 dropped receipt。 |
| RT-CINE-28 | animation 与 cinematic authority 分裂 | 明确 animation sequence 是被消费的 source/clip，不允许两个 manager 同时推进同一实例。 |
| RT-CINE-29 | editor-only timeline 误作 runtime | runtime target admission 缺失时拒绝发布 cinematic artifact，而不是包装 AnimationSequence。 |
| RT-CINE-30 | 跨平台数值策略缺失 | 规定 float/固定点、negative zero、timecode、hash 和 provider version negotiation。 |

## 5. P2 与资格门

P2 共 10 项：SMPTE/drop-frame UI、shot thumbnail cache、camera path diagnostics、take metadata review、render farm scheduling、AOV/cryptomatte outputs、replay diff、telemetry redaction、artifact eviction、localization/accessibility。均必须消费真实 artifact/receipt。

24 道门中，G1 domain owner、G2 source identity、G3 compiler artifact、G4 frame time、G5 hierarchy、G6 playback instance、G7 camera、G8 audio、G9 event、G10 take、G11 movie queue、G12 network/save/replay、G13 plugin execution、G14 failure receipt、G15 output provenance、G16 deterministic parity、G17 scheduler budget、G18 scale、G19 fault recovery、G20 target admission、G21 pre-animated restore、G22 product consumer、G23 benchmark、G24 release packaging 当前均 Fail；仅通用 animation sampling 与 shared time substrate可记为 Partial。不存在 Pass。

## 6. 实施顺序

先建立 source schema、stable IDs、qualified frame、compiler/artifact、World playback owner 和 failure receipt；再实现 hierarchy、binding、section evaluation、camera/animation/audio/event adapters；随后实现 take/render queue、network/save/replay 和 plugin dist；最后连接 Editor236 的 authoring document、Preview/PIE 与 debug mirror。任何 Workbench Preview 或 Validate 文案都必须等待真实 frame/diagnostic receipt。

本轮仅修改 review/index/coverage 文档，没有修改 runtime、editor、tests、Cargo、ABI 或 ZUI，也没有运行 Cargo、PIE、录制、离线渲染、GPU、fault、scale、soak 或 benchmark；按用户要求未查询、轮询、等待或实时跟踪协调器。
