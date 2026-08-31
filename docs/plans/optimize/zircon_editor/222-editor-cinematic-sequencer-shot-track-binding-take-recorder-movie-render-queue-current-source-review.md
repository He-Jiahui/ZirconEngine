---
title: Editor Cinematic Sequencer、Shot、Track、Binding、Take Recorder 与 Movie Render Queue 当前源码复审
category: zircon_editor
report_id: Editor222
review_date: 2026-08-29
baseline_head: 16e0c4f3dd15813eccd097eef56ba1488a267d86
canonical_owner: Editor45
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/83-editor-cinematic-sequencer-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/119-editor-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-current-source-review.md
  - docs/plans/optimize/zircon_editor/166-editor-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/99zy-runtime-cinematic-sequencer-sequence-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-network-save-scalability-editor-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/36-video-media-source-player-track-clock-media-texture-playback-capture-recording-authoring-review.md
  - docs/plans/optimize/zircon_editor/136-editor-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/207-editor-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/213-editor-video-media-source-player-track-clock-media-texture-playback-capture-recording-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/timeline
  - zircon_editor/src/ui/curve
  - zircon_editor/src/ui/preview_scene
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/animation
  - zircon_plugins/animation/runtime
  - zircon_plugins/timeline_sequence
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/graphics/runtime/render_framework/capture_frame
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/crates/zr_rhi/src/diagnostic_readback.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/gpu_readback_queue
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieScene.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieSceneBinding.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieSceneSection.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/Evaluation/MovieSceneSequenceHierarchy.h
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene/Public/MovieSceneSequencePlayer.h
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Public/ISequencer.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/Takes/Source/TakesCore/Public/TakeMetaData.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/Takes/Source/TakesCore/Public/TakeRecorderSource.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/Takes/Source/TakeRecorder/Public/Recorder/TakeRecorderSubsystem.h
  - dev/UnrealEngine/Engine/Plugins/MovieScene/MovieRenderPipeline/Source/MovieRenderPipelineCore/Public/MoviePipelineQueue.h
  - dev/UnrealEngine/Engine/Plugins/MovieScene/MovieRenderPipeline/Source/MovieRenderPipelineCore/Public/MoviePipelineAntiAliasingSetting.h
  - dev/UnrealEngine/Engine/Plugins/MovieScene/MovieRenderPipeline/Source/MovieRenderPipelineCore/Public/MoviePipelineHighResSetting.h
  - dev/UnrealEngine/Engine/Plugins/MovieScene/MovieRenderPipeline/Source/MovieRenderPipelineCore/Public/MoviePipelineOutputSetting.h
  - dev/godot/scene/resources/animation.h
  - dev/godot/scene/resources/animation.cpp
  - dev/godot/scene/animation/animation_mixer.h
  - dev/godot/scene/animation/animation_player.cpp
  - dev/godot/servers/movie_writer/movie_writer.h
  - dev/godot/servers/movie_writer/movie_writer.cpp
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/signal.rs
  - dev/Fyrox/editor/src/plugins/animation/track.rs
  - dev/Fyrox/editor/src/plugins/animation/command/mod.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/animation_event.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/RenderPass/AOV/AOVRequest.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/RenderPass/AOV/AOVRequestBuilder.cs
finding_status:
  p0_open: 4
  p0_partial: 1
  p0_closed: 0
  p1_open: 51
  p1_partial: 19
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
gate_status:
  fail: 27
  partial: 5
  pass: 0
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
evidence_captured_at: 2026-08-29T15:10:00+08:00
---

# Editor222 · Cinematic Sequencer / Take Recorder / Movie Render Queue 当前源码复审

## 1. 结论

当前 Zircon 仍没有工程级 Cinematic Sequencer、Take Recorder 或 Movie Render Queue。tracked 生产树和 2,459 个未跟踪生产 Rust/TOML/ZUI 文件中，`CinematicSequenceSource`、`CinematicEvaluationInstance`、`MovieRenderQueue`、`MovieRenderJob`、`TakeSession`、`TakeRecorder`、`SequenceHierarchy`、`PreAnimatedState`、`ShotAsset`、`TakeAsset`、`MovieRenderArtifact`、`CinematicArtifact` 均为 0 命中；`ResourceKind` 仍只有 `AnimationSequence`，没有 Cinematic、Shot、Take 或 Movie Render 身份。

编辑器产品表面仍表达不存在的事实。233 行 Sequencer ZUI 固定写入 `SEQ_Intro`、`Camera_A`、Camera Cut、Audio Theme、Event Cues、12 shots、428 keys、24 fps 和固定范围；feedback callback 把 open/preview/validate 直接写成 `Native extension workspace opened for SEQ_Intro`、`Preview queued SEQ_Intro 24 fps`、`Validation queued 12 shots 1 gap`。这些 route、binding、navigation 和 allowlist 没有连接 document/provider、source revision、evaluation instance、job、receipt 或 artifact。

`timeline_sequence` 只有 descriptor、manifest 和局部 key move helper。声明的 `plugins://timeline_sequence/editor/authoring.zui` 不存在；五个 operation 没有 factory/executor；dist 的 command/event manifest 是空字节串，`invoke_command` 为 `None`，bridge method 为 0。key move 已先做完整 preflight、失败零变更、有限值检查、二分插入和 equal-time 稳定排序，这是应保留的局部 kernel，但仍使用三层 collection index，不能作为产品 identity、transaction 或协同重放基础。event marker 仍是插件私有内存结构，未进入 asset codec、compiler 或 runtime evaluator，且 marker 校验仍没有显式 finite duration/time gate。

通用 Animation document/compiler、Camera stack、RHI bounded readback 和单帧 PNG staging 是真实底座，但不能被包装成电影能力。Animation 的 compiled track 仍按 binding/track index 回读 source，runtime cache 以 asset ID 共享且忽略 sample entity，compile/apply 错误没有 frame receipt；Editor preview 只有 fake `PreviewSceneBackend`。Capture 只有 RGBA8/16F、尺寸、generation 和 capture report；没有 frame/timecode/camera/shot/sample/tile/pass、AOV/color、ordered writer、encoder/muxer、checkpoint 或 whole-run artifact。

本轮维持 Editor45 的 canonical 账本：**P0 4 Open / 1 Partial；P1 51 Open / 19 Partial；P2 12 Open；32 个资格门 27 Fail / 5 Partial / 0 Pass**。没有同一 source、镜头、采样质量、输出完整度和故障条件下的可复现结果，不能声称 Zircon 功能、性能或表现优于 Unreal。

## 2. 范围、统计与方法

本轮读取 commit `16e0c4f3dd15813eccd097eef56ba1488a267d86` 的当前物理工作树；不回退用户 dirty 文件。选择集按 normalized path 计算行数、非空行、bytes、test/ignored 属性和 SHA-256 manifest fingerprint。Editor 组包含 Sequencer surface、Animation document、Animation Editor、Timeline、Curve、Preview；runtime 组包含通用 Animation/compiler/evaluator；plugin 组包含完整 `timeline_sequence` package；capture 组包含 camera/capture/readback/PNG substrate。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored / dirty | fingerprint |
|---|---:|---|
| Editor Sequencer/document/timeline/curve/preview | **50 / 7,829 / 7,368 / 304,381 / 25 / 0 / 48** | `aa4e1dc4a0360ad46b811587b4f64efcaff7ad5b4e5e528f5865e0306996eeeb` |
| Runtime Animation/compiler/evaluator | **235 / 30,241 / 27,659 / 1,046,667 / 224 / 12 / 107** | `f02b058bd0449750a5d8ff6b37b37705269ec1130b4949acabae71c1d9360a93` |
| Timeline plugin product boundary | **10 / 1,142 / 1,042 / 42,168 / 15 / 1 / 3** | `1a7b3fba4f2a86407bc7fb1fc3b4a4ad0124e01d0a1ad4076fa4b3928f486970` |
| Camera/capture/RHI/PNG substrate | **33 / 6,020 / 5,443 / 208,614 / 69 / 2 / 20** | `ee5f44a70189765a4cdb2f0b93346a496f9e64baa5ad270cde57342b0bf3f50a` |
| Zircon union | **328 / 45,232 / 41,512 / 1,601,830 / 333 / 15 / 178** | `280ff91cc0cb204ec16b2392d9b1aec31db093e09cb3dc0bee177a42337b8ff6` |
| Unreal/Godot/Fyrox/Bevy/Unity reference | **29 / 22,524 / 19,218 / 846,294 / 10 / 0 / 0** | `2aa441a3ae88ed4c7278ee320a6e28187037275846e83309efefdb702f31c01f` |

tracked 与未跟踪产品树的核心电影类型精确检索均为零。参考集 29 个文件全部存在；Unreal 用于 MovieScene/Sequencer/Take/MRQ 主合同，Godot 用于 typed tracks/player/writer，Fyrox 用于 UUID/command，Bevy 用于 target/event/graph，Unity Graphics 仅用于 camera capture/AOV callback 合同。Tooling 不在本轮范围内。

## 3. 当前源码断路

### 3.1 Sequencer surface 不是 document projection

1. [workbench_extension_sequencer_workspace.zui](/E:/Git/ZirconEngine/zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui) 只有固定节点和文本：`SEQ_Intro`、三个候选 sequence、Camera Cut/Audio/Event 行、12 shots/428 keys、24/30/60 fps 和 `0100-1460` work range；没有 provider/controller/data source 绑定。
2. `extension_module_feedback.rs` 的 open/preview/validate 分支返回固定文本，不能产生 operation request、request/job ID、progress、cancel、diagnostic 或 artifact locator。
3. `extensions.rs`、template binding 和 navigation spec 只维护 route 字符串及 whitelist。route 改变 control-local selection/value，未触发真实 source、compiler 或 runtime evaluation。
4. `tools/editor-workbench-preview/design.js` 只能是设计 fixture；不能提升 production 证据等级。

### 3.2 Timeline plugin 的声明、资源和执行链断裂

1. `editor/src/plugin.rs:139-215` 注册 Open/Create/Delete/Move/Validate 五个 operation、三个 track descriptor 和 `ResourceKind::AnimationSequence` toolkit，却没有 operation factory/executor。
2. `template_document` 指向不存在的 `plugins://timeline_sequence/editor/authoring.zui`；manifest 只写 capability、dependency 和 packaging，没有资源 hash、schema loader 或 admission 检查。
3. `dist/src/lib.rs:17-43` 明确把 command/event manifest 设为空、`invoke_command: None`、`bridge_methods: []`；这证明动态分发只有 entry descriptor，没有可执行远端行为。
4. `TimelineEventMarker`（`editor/src/lib.rs:24-28`）不是 `AnimationSequenceAsset` 字段。依赖的 `runtime.feature.animation.timeline_event_track` 实际覆盖通用 clip event，而非 sequence marker 的保存、cook、interval evaluation。
5. `validate_event_marker_payload`（`lib.rs:162-178`）仅比较 `< 0`/`> duration`；NaN 可绕过比较，duration 本身也未先验证 finite/positive。
6. `move_timeline_keyframe`（`lib.rs:80-145`）现在实现失败零变更和 equal-time 稳定，这是 Partial 进展；但 request 仍是 `binding_index/track_index/key_index`，无 stable key ID、source revision、CAS、dirty/selection/history 或 undo receipt。

### 3.3 通用 Animation 不能充当 Cinematic runtime

1. `AnimationSequenceAsset`（`asset/sequence.rs:19-133`）只有 name、秒制 duration、浮点 fps、`EntityPath`、可选 `String target_id` 和 property tracks；没有 sequence/binding/track/section/shot/channel/key stable ID、subsequence、camera cut、audio/event/spawnable 或 timecode。
2. `ResourceKind`（`zircon_runtime_interface/src/resource/marker.rs:8-30`）没有 CinematicSequence、Shot、Take、MovieRenderQueue、Preset 或 Artifact。
3. `CompiledAnimationSequence`（`animation/sequence/compiled.rs:18-142`）把 compiled track 保存为 `binding_index/track_index`，apply 时再次回读 source Vec；这不是自包含 dense evaluation field。
4. `apply_loaded_sequences`（`zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs:31-80`）的 cache key 只有 asset ID/revision/world currentness，忽略 `LoadedSequenceSample.entity`；同一个 asset 的两个实例无法拥有不同 binding override 或 pre-animated state。
5. 同一函数在 compile 失败时移除 cache 后 `continue`，apply 结果用 `let _ =` 丢弃；没有 previous/current qualified time、Play/Jump/Scrub/Reverse/Loop request、phase schedule、event receipt、atomic frame commit 或 terminal disposition。
6. `AnimationAuthoringDocument` 的 revision/CAS、transaction、undo/redo、compiler diagnostic、last-good product 是可保留底座；但 mutation 仍由 `AnimationTrackPath + frame` 定位，Timeline/Curve projection 用 `track_id@time_bits`（`timeline_foundation.rs:76`、`curve_foundation.rs:107`）生成 key identity，重定时后 identity 漂移。
7. `add_key` 仍复制最后一个值，空轨道写入 `Scalar(0.0)`（`mutation/sequence.rs:34,212`），没有从 resolved typed property 读取当前值；这是通用 Animation 编辑器的伪造值风险，不能用于电影 authoring。
8. `PreviewSceneBackend` 生产实现为 0，只有 `ui/preview_scene/tests.rs` 的 fake backend；没有将编译 artifact 安装到隔离 preview World，也没有 Editor/PIE/runtime parity。

### 3.4 Camera、Capture、Take 与 MRQ 仍是分离底座

1. Camera stack 的 `CameraSequenceReport` 只按 render order/target/entity 排序和校验 overlay；没有 authored director、typed cut request/receipt 或 history epoch。`CameraCutOrInvalid` 是 temporal invalidation heuristic，不是电影 cut authority。
2. `CapturedFrame` 与 `CapturedHdrFrame`（`render/capture.rs:107-143`）只有 width/height、RGBA8 或 RGBA16F、generation、capture report 和调试 profile；没有 stride、color/transfer、PTS/timecode、camera/shot/sample/tile/pass/AOV identity。
3. `GpuReadbackQueue` 有 3-slot ring、per-frame/pending bytes budget、cancel/abort、map failure、device loss/shutdown callback；`DiagnosticReadbackTracker` 还提供 submission-qualified receipt。这是 bounded transport，不是 ordered movie packet、writer backpressure 或 exactly-once artifact。
4. `frame_capture.rs` 的 PNG writer 有 staging、flush、file sync、Windows `ReplaceFileW` 和失败清理；它只证明单帧 durable publish，不能证明多 frame/pass/audio 的 manifest、checkpoint、encoder/muxer 或 whole-run atomic publication。
5. 没有 TakeSource registry、TakeSession 状态机、slate/take/timecode metadata、bounded source buffer、journal、TakeAsset 或 sequence write-back transaction。
6. 没有 MovieRenderQueue/Job/Shot/Preset/Config/Run identity、submit freeze、deterministic shot/frame/sample/tile/pass expansion、fixed-step movie clock、warmup、temporal/spatial sample、shutter、tile/AOV/color policy、worker retry/resume 或 headless parity。

## 4. 与本地参考引擎的最低合同差异

| 合同 | Zircon 当前 | 参考源码证据 | 必须重构 |
|---|---|---|---|
| source/identity | AnimationSequence、path/String、Vec index | Unreal binding GUID；Fyrox Track UUID；Bevy `AnimationTargetId` | 独立 versioned cinematic source，所有元素 stable ID，display path 仅显示 |
| time | `f32` seconds/fps | Unreal tick/display `FFrameRate/FFrameTime`；Godot double time | rational tick/subframe、display rate、timecode、checked conversion |
| sections/hierarchy | 无 cinematic section/subsequence | Unreal section range、row、overlap、pre/post-roll、completion、hierarchy transforms | section/shot/subsequence 一等对象和 interval evaluation field |
| binding/lifecycle | EntityPath + optional string | Unreal possessable/spawnable、binding override、spawn register | qualified resolver、spawn register、orphan diagnostic、per-instance context |
| playback | `time += delta` 风格 sample/apply | Unreal Play/Jump/Scrub；Bevy event traversal；Godot backup/restore | evaluation request、phase、event policy、pre-animated restore、receipt |
| authoring | generic CAS；plugin descriptor | Fyrox UUID command/undo；Unreal Sequencer transaction | stable-ID domain command、one transaction、coalescing、external conflict |
| tracks/events | property curve；marker 私有内存 | Godot value/method/bezier/audio/animation；Bevy event | typed provider codec/compiler/evaluator/editor/migration contract |
| take | 无 | Unreal TakeMetaData/TakeRecorderSource/Subsystem | source registry、clock、metadata、bounded buffer、journal、atomic Take |
| render queue | 无 | Unreal Queue/Job/Shot、AA/high-res/output settings | frozen queue、deterministic expansion、fixed-step、checkpoint |
| output | 单帧 RGBA/PNG | Godot MovieWriter begin/frame/end+audio；Unity camera capture/AOV | typed frame packet、ordered backpressure、AOV/color、encoder/muxer、whole-run artifact |

## 5. P0：实施前必须封口

1. **P0-01 · Open**：删除 production 固定 `SEQ_Intro`、12 shots、428 keys、候选 fps/range 和 queued/success feedback；真实 provider 不存在时只显示 unavailable。
2. **P0-02 · Open**：资源、factory、codec/compiler/evaluator/editor bridge 任一缺失时，Timeline plugin operation 和 track type 必须 fail-close，不得显示可创建菜单。
3. **P0-03 · Partial**：保留当前 preflight/失败零变更/equal-time helper；在 stable key ID、revision CAS、Editor transaction、dirty/selection/history 和 undo receipt 完成前禁止产品调用。
4. **P0-04 · Open**：event marker 进入 versioned cinematic source、compiler interval field 和 runtime traversal 前不得 admission，并先补 finite time/duration 验证。
5. **P0-05 · Open**：独立 source、evaluation instance、take session、queue/job、artifact 均不存在时，Cinematic/Take/MRQ 入口保持 Unavailable；不得包装普通 AnimationSequence 或单帧 capture。

## 6. P1：工程化重构清单

### Source、identity、time、schema

1. **P1-01 · Open** 独立 `CinematicSequenceSource`，持有 schema/source ID/revision/catalog fingerprint。
2. **P1-02 · Open** sequence/binding/track/section/shot/folder/marker/channel/key 全部使用持久 stable ID。
3. **P1-03 · Open** 分离 tick resolution、display rate、subframe 和 timecode。
4. **P1-04 · Partial** 复用 Animation kind/V1 fallback；补 cinematic migration、unknown section opaque preservation 和 provider capability policy。
5. **P1-05 · Open** 定义 source/world/instance/player qualified identity。
6. **P1-06 · Partial** 将 ComponentPropertyPath/EntityPath 收敛为 typed binding target、field ID 和 schema fingerprint。
7. **P1-07 · Open** 定义 root/local/global qualified time、range、pre/post-roll。
8. **P1-08 · Open** 统一 owner/generation/request/job/receipt 传播和 terminal disposition。
9. **P1-09 · Open** display path、EntityPath、collection index 不得作为 authority key。
10. **P1-10 · Partial** 保留 Animation compiler 排序和 helper 确定性；补 canonical cinematic order/content digest/跨平台浮点规范。
11. **P1-11 · Open** possessable/spawnable binding source 与 qualified resolver。
12. **P1-12 · Open** nested sequence/subsequence hierarchy、time transform、bias、trim。
13. **P1-13 · Open** spawn register、lifetime、orphan、missing binding diagnostics。
14. **P1-14 · Open** binding override、instance context、PIE/world duplication policy。
15. **P1-15 · Open** track/section/shot/folder registry 与 typed factory。
16. **P1-16 · Open** section range、row、overlap、priority、completion、blend policy。
17. **P1-17 · Open** transform/animation/property/camera/audio/event typed adapters。
18. **P1-18 · Open** plugin track provider 必须同时提供 codec/compiler/evaluator/editor/migration。
19. **P1-19 · Partial** 复用通用 compiler/cache；补 cinematic dependency graph、artifact key、provider/root-context key 和 LKG/CAS install。
20. **P1-20 · Partial** 保留 compiled property writer；改为自包含 dense channel storage 和 interval field，不回读 source Vec index。
21. **P1-21 · Open** evaluation phase schedule、pre/post hooks 与 deterministic order。
22. **P1-22 · Open** `CinematicEvaluationInstance` root context、scoped state 和 frame receipt。
23. **P1-23 · Open** pre-animated state capture/restore、abort/error/sequence switch。
24. **P1-24 · Open** Play/Jump/Scrub/Reverse/Loop event traversal 语义。

### Editor document、Sequencer、preview

25. **P1-25 · Open** Camera Cut 接入 Editor30 authoritative endpoint/director/history epoch。
26. **P1-26 · Partial** 复用 Editor14 Animation/Curve foundation；Sequencer 尚未消费 stable typed curve/key/artifact。
27. **P1-27 · Open** 接入 Editor36 media/audio timestamp、clock、encoder/muxer contract。
28. **P1-28 · Open** Editor/PIE/runtime 使用同 artifact、time、binding 取得 preview parity。
29. **P1-29 · Partial** 复用 Animation document revision/CAS/history；补 cinematic dirty/autosave/recovery/external-change。
30. **P1-30 · Partial** 复用通用 transactional key/track；补 stable-ID move/trim/slip/split key/section/shot command。
31. **P1-31 · Open** key identity 从 index 或 `path@time_bits` 迁移到 stable ID。
32. **P1-32 · Partial** 保留 helper/CAS rollback；补 source/dirty/history/selection/save/compile 全链 failure-zero-mutation。
33. **P1-33 · Open** provider-backed virtualized outliner/timeline/ruler/zoom/curve/selection projection。
34. **P1-34 · Open** multi-select、drag、snap、ripple、overlap、keyboard commands。
35. **P1-35 · Open** source revision conflict、external-change rebase policy。
36. **P1-36 · Open** UI feedback 只来自 provider/job/receipt，删除固定回写。

### Take Recorder

37. **P1-37 · Open** TakeSource registry、typed capability、arm/prepare lifecycle。
38. **P1-38 · Open** TakeSession clock/frame/timecode/metadata/slate/take number。
39. **P1-39 · Open** 每个 source 的 bounded buffer、backpressure、drop/error receipt。
40. **P1-40 · Open** start/tick/stop/finalize/cancel/recover 幂等状态机。
41. **P1-41 · Open** journal/staging 校验后 atomic publish TakeAsset。
42. **P1-42 · Open** source failure、disk full、device loss、cancel、finalize crash 不发布半 Take。
43. **P1-43 · Open** recorded section 以 stable binding/key/channel 写回 sequence transaction。
44. **P1-44 · Open** Take metadata、browser、命名、collision/CAS 和检索。
45. **P1-45 · Open** recording UI 显示 clock lock、sample/drop/buffer/I/O/finalize progress。

### Movie Render Queue、capture、artifact

46. **P1-46 · Open** Queue/Job/Shot/Preset/Config/OutputArtifact source types。
47. **P1-47 · Open** submit 冻结 source/map/content/plugin/engine/config fingerprints。
48. **P1-48 · Open** deterministic shot/frame/sample/tile/pass expansion 和 checkpoint。
49. **P1-49 · Open** fixed-step movie clock、warmup、pre/post-roll、cut history reset。
50. **P1-50 · Open** temporal/spatial sample、shutter、AA、tile、高分辨率 policy。
51. **P1-51 · Open** camera/audio/event/AOV pass 与 metadata schema。
52. **P1-52 · Partial** Capture 已有 RGBA8/16F、尺寸、generation、typed report；补 format/stride/color/PTS/timecode/shot/frame/sample/tile/pass。
53. **P1-53 · Partial** RHI 已有 bounded slots/bytes/cancel/abort/shutdown；补 ordered movie packet、writer backpressure、exactly-once receipt。
54. **P1-54 · Open** Editor36 encoder/muxer provider 接入，不复制 codec。
55. **P1-55 · Partial** 单帧 PNG staging/flush/sync/replace 可保留；补全 frame/pass/audio 后 whole-run atomic artifact。
56. **P1-56 · Open** headless 与 Editor worker 共享 compiler/clock/binding/sample schedule。
57. **P1-57 · Open** 接入 Editor09 admission/quota/priority/progress/cancel/shutdown drain。
58. **P1-58 · Partial** Animation/Camera/Capture 是真实底座；补 Editor22/30/36 typed cross-owner orchestration 与统一 receipt。
59. **P1-59 · Partial** compiler/readback 有稳定诊断/终态基础；补 source/shot/frame/sample/tile/pass/item 定位和 error artifact。
60. **P1-60 · Partial** frame profile/capture stats 可复用；apply stats 被丢弃，仍无 compile/evaluate/take/render budget telemetry。
61. **P1-61 · Open** source/schema/ID/time/migration golden tests。
62. **P1-62 · Open** binding/spawn/hierarchy/evaluation/pre-animated restore tests。
63. **P1-63 · Partial** helper/CAS tests 已存在；补 stable key/section/shot transaction、undo/redo、selection/dirty matrix。
64. **P1-64 · Open** event traversal、camera cut、audio sync、preview parity tests。
65. **P1-65 · Open** Take state、buffer overflow、device/disk/finalize crash tests。
66. **P1-66 · Open** queue expansion、fixed-step、sampling、AOV/color golden tests。
67. **P1-67 · Open** worker cancel/retry/resume、artifact completeness/atomic tests。
68. **P1-68 · Open** plugin unknown provider、codec mismatch、unload、schema migration tests。
69. **P1-69 · Partial** 仅有 ignored 16,384-key helper benchmark；补 1k track/100k key、long take、large queue、多 shot、allocation 和 cross-platform benchmark。
70. **P1-70 · Partial** mutation-before-validation 已封口；仍需硬切 static feedback、index identity、Scalar(0) Add Key、缺失资源和第二 writer authority。

## 7. P2：主线完成后的扩展

1. **P2-01 · Open** Live Link、Virtual Production、硬件 timecode。
2. **P2-02 · Open** procedural shot、batch variant、OTIO/EDL/AAF 交换。
3. **P2-03 · Open** distributed render farm、remote worker、cloud queue。
4. **P2-04 · Open** collaborative Sequencer lock、annotation、review。
5. **P2-05 · Open** ML 镜头/key reduction/质量辅助。
6. **P2-06 · Open** viewport stream、remote take、multi-camera capture。
7. **P2-07 · Open** HDR mastering、OCIO、deep output、高级 AOV。
8. **P2-08 · Open** shot dependency graph、partial rerender、content-addressed frame cache。
9. **P2-09 · Open** audio post、ADR、subtitle、DAW interchange。
10. **P2-10 · Open** deterministic replay、evaluation debugger、scrub archive。
11. **P2-11 · Open** headless CI、long-run soak、fault campaign。
12. **P2-12 · Open** 在相同数据、采样、输出和故障完整性下实测超过参考引擎。

## 8. 目标架构、owner 与里程碑

```text
CinematicSource -> schema/migration/dependency compiler -> CinematicArtifact
  -> EvaluationInstance(root context + binding overrides + spawn + pre-animated)
  -> Camera/Animation/Audio/Event adapters -> FrameReceipt
TakeSession -> timestamped bounded samples -> staged TakeAsset -> sequence transaction
RenderQueue -> frozen Job/Shot/Frame/Sample/Tile/Pass plan
  -> capture/readback -> Editor36 encoder/muxer -> manifest/checkpoint -> atomic artifact
```

| owner | 拥有 | 不拥有 |
|---|---|---|
| Editor45 / Cinematic | source、shot/section/binding、hierarchy compiler、Sequencer projection、Take/MRQ orchestration | 通用曲线、camera solver、Render Graph、codec |
| Editor14 / Animation | Animation asset/curve/graph/document/compiler基础 | shot/subsequence、take、movie queue |
| Editor30 / Camera | camera endpoint/director/blend/shake、authored cut/history epoch | sequence hierarchy、job scheduling |
| Editor36 / Media | media clock、timestamped sample、encoder/muxer、durable media artifact | shot expansion、evaluation authority |
| Editor22 + RHI / Capture | Render Graph、AOV/capture、bounded GPU readback | cinematic source、queue policy |
| Runtime Asset/Scene | versioned codec、cook/dependency、stable object identity、runtime install | Editor transaction、queue UX |

依赖顺序：M0 诚实性封口 -> M1 source/ID/time/schema -> M2 binding/spawn/hierarchy -> M3 typed track/section/shot adapters -> M4 compiler/evaluation/restore/event/cut -> M5 provider-backed Sequencer/document/preview -> M6 Camera/Animation/Media/Capture parity -> M7 Take Recorder -> M8 Queue/Job/fixed-step/sampling -> M9 encoder/AOV/checkpoint/atomic artifact -> M10 fault/cook/scale/cross-platform -> M11 legacy hard cut、docs/manifest/CI/benchmark。

退出条件：M4 前不得声称 runtime playback，M7 前不得录制真实 Take，M8 前不得显示可提交 render job，M9 前不得显示 artifact 成功。任何 plugin capability 不完整时只读 opaque；任何 source/compile/save/finalize 失败保留 last-good 或 staging，不能发布半状态。

## 9. 32 个资格门

| Gate | 状态 | 当前证据 |
|---|---|---|
| G01-G04 | Fail | 固定 UI；无 stable element ID；仍用 f32 seconds/fps。 |
| G05 | Partial | Animation kind/V1 fallback 存在；无 cinematic migration/unknown sections。 |
| G06 | Fail | 无 cinematic source/content digest。 |
| G07 | Partial | EntityPath/target_id 与 compiled writer 存在；无 qualified context。 |
| G08-G10 | Fail | 无 spawn lifecycle、hierarchy、可执行 section。 |
| G11 | Partial | 通用 property compiler 存在；无 cinematic typed provider registry。 |
| G12-G16 | Fail | 无 artifact/orphan、event traversal、restore、authored cut、audio sync。 |
| G17 | Partial | helper/CAS 局部失败零变更；无 cinematic stable-ID transaction。 |
| G18-G20 | Fail | 无 preview parity；无 Take lifecycle/metadata。 |
| G21-G27 | Fail | 无 Take buffer/recovery/publication、queue freeze/expansion/fixed-step/sampling/AOV。 |
| G28 | Partial | RHI bounded readback；无 ordered movie packet/writer backpressure。 |
| G29-G32 | Fail | 无 encoder/muxer、checkpoint/resume/whole artifact、规模 benchmark、E2E/cook/headless parity。 |

## 10. 验证与限制

已完成：逐文件读取 Editor45/119/166 历史报告；扫描当前 tracked 与未跟踪生产树的 12 个核心电影契约；逐文件读取当前 Sequencer ZUI、route/binding/navigation/feedback、完整 timeline plugin、Animation asset/compiler/cache/document/timeline/curve/preview、camera/capture/readback/PNG；逐段阅读 Unreal、Godot、Fyrox、Bevy、Unity Graphics 29 个参考文件；重算当前选择集统计和 fingerprint。

未完成且不应伪称通过：Cargo/Editor 编译、Sequencer UI 动态运行、Take 设备录制、GPU movie readback、encoder/muxer、cook/headless worker、fault/recovery/long-run、跨平台 determinism、跨引擎性能 benchmark。共享工作树有大量非本轮 dirty 文件；实施前必须重新冻结 HEAD、选择集 fingerprint 和 provider owner。

本报告只修改 review 文档和索引，不修改 Runtime、Editor、Interface、Plugin、App 或 tests。Editor45 继续是 canonical owner；Editor222 只刷新 currentness，不重复增加总账。按用户要求未查询、轮询、等待或实时跟踪协调器，Tooling 继续排除。
