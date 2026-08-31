---
title: Editor Cinematic Sequencer、Shot、Track、Binding、Take Recorder 与 Movie Render Queue 当前源码复核
category: zircon_editor
report_id: Editor119
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor45
refreshes:
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/animation_editor
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_plugins/timeline_sequence
  - zircon_plugins/editor_support/src/lib.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/36-video-media-source-player-track-clock-media-texture-playback-capture-recording-authoring-review.md
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 119 · Editor Cinematic Sequencer / Shot / Track / Take Recorder / Movie Render Queue 工程化差距

## 1. 结论

当前 Zircon 没有工程级 Cinematic Sequencer、Take Recorder 或 Movie Render Queue。Workbench 固定显示 `SEQ_Intro`、12 shots、428 keys、24/30/60 fps，Preview/Validate 返回 queued 文案；这些值来自静态 ZUI 与 callback，不来自 sequence asset、evaluation instance、recording session、render job 或 artifact。

仓内 `AnimationSequenceAsset`、Animation Editor、property track compiler、Camera stack、GPU readback 和单帧 PNG writer 是可复用窄底座，但不是电影产品：sequence 只有秒制 duration/fps、entity path/字符串 target、property track/channel，没有 stable binding、track/section/shot/subsequence、camera cut、audio/event、spawnable、hierarchy time transform、pre/post-roll、pre-animated state 或电影时钟。runtime cache 还忽略 apply 错误，无法提供上下文 binding override 和失败 receipt。

`timeline_sequence` 仅有 descriptor、实验 manifest 和 helper；声明的 ZUI 资源不存在，dist 无 factory/bridge，event marker 没有进入 `AnimationSequenceAsset` 或 sequence evaluator。`move_timeline_keyframe` 按 collection index 修改并排序后才校验，校验失败可能留下 mutation，不能接入 transaction。

Capture 只有 RGBA8/16F 单帧、width/height/generation mailbox；没有 frame/timecode/camera/shot/AOV、fixed-step、warmup、temporal/spatial sampling、motion blur、checkpoint、resume 或 atomic movie output。Camera Cut、Media 编码、Render Graph capture 和通用 Animation 分别由 Editor30/36/22/14 负责，本报告只拥有电影 orchestration。

本轮 Zircon scope 为 38 files / 6,810 lines / 6,208 non-empty / 259,253 bytes / 59 test attributes；参考 scope 为 29 / 22,553 / 19,218 / 846,294 / 12；union 为 67 / 29,363 / 25,426 / 1,105,547 / 71。Zircon fingerprint `26ab6555938d86baf2f666711b778ff7a113df1eb365c00e041e0a6e0c433a83`，refs `25da31a2fb07cc11eed818950b487df1d1ff012e0af70d73b1ab6f1a624463b0`，union `eca1bf68f01cdd822f985f5178c7f7cc69ce862372ab58701f90ef0e74d78223`。本报告登记 5 个 P0、70 个 P1、12 个 P2 与 M0-M11；不修改生产代码。

## 2. 当前实现与参考差异

### 2.1 真实执行事实

1. Sequencer ZUI 固定 sequence/shot/key/track/fps/range，binding/navigation/feedback 只操作 control-local 状态。
2. Preview/Validate 无 source revision、request/job ID、progress、cancel、diagnostic artifact 或 completion receipt。
3. `AnimationSequenceAsset` 有 revision、duration/fps、binding/property track/channel，适合通用 animation adapter，但 ResourceKind 无 Cinematic/Shot/Take/MovieJob。
4. sequence binding 使用 EntityPath/字符串 target，不能跨 World、LevelInstance、PIE copy 或 spawnable context 稳定解析。
5. runtime cache 按 asset revision/current World 编译；Loaded sample entity 未用于 apply，apply error 被丢弃。
6. player 只有 sequence/speed/time/looping/playing，没有 root evaluation instance、time hierarchy、spawn register、pre-animated restore 或 cut history。
7. timeline plugin 的 marker、operation、resource、factory、bridge 不形成持久化/编译/执行闭环。
8. key move 的 index identity 和“修改后验证”违反失败零变更；不能安全 undo/redo 或协同重放。
9. capture/readback mailbox 只保留最新单帧；PNG writer 只证明单张 output 的原子写入。
10. Unreal MovieScene/Sequencer/TakeRecorder/MoviePipeline、Godot Animation/AnimationPlayer/MovieWriter、Fyrox UUID track/command、Bevy AnimationTargetId/graph、Unity AOV callback 提供了成熟职责边界，不能以静态 ZUI 代替。

### 2.2 关键边界

1. Editor14 拥有通用 Animation sequence/graph/curve；Editor45 拥有 cinematic source、shot/subsequence、take、queue/job orchestration。
2. Editor30 拥有 camera endpoint/director/authoritative cut；Editor45 只保存 binding 与 cut request，不复制 camera solver。
3. Editor22/Runtime capture 拥有 Render Graph/readback/debug；Editor36 拥有 timestamped media/encoder/muxer；Editor45 只编排 sample/shot/output。
4. 普通 Animation player、CameraCutOrInvalid heuristic、PBR Viewer PNG 和 `AnimationClip.event_tracks` 不得被包装为 cinematic playback/take/render。

## 3. 差距清单

### 3.1 P0：实施前必须阻断

1. **P0-01** 固定 `SEQ_Intro`、12 shots、428 keys 与 queued/success feedback 必须从 production route 移除或明确标 fixture。
2. **P0-02** Timeline plugin 无资源/factory/compiler/evaluator 时不得显示可执行 operation 或 event marker track。
3. **P0-03** `move_timeline_keyframe` 在重构为 stable key ID + preflight + one transaction 前不得被产品调用。
4. **P0-04** event marker 未进入 versioned cinematic source 与 runtime interval evaluator 前不得声明可用。
5. **P0-05** 无独立 cinematic source、binding/hierarchy compiler、movie clock、take session、queue/job 和 artifact 时不得开放 Cinematic/Take/MRQ。

### 3.2 P1：70 项重构主线

1. **P1-01** 建立 versioned `CinematicSequenceSource`、source ID、revision、catalog fingerprint。
2. **P1-02** 为 sequence/binding/track/section/shot/folder/marker/channel/key 分配 stable ID。
3. **P1-03** 分离 tick resolution、display rate 与 timecode，使用有理数 frame/subframe。
4. **P1-04** 建立 schema migration、unknown section 与 plugin capability policy。
5. **P1-05** 定义 source/world/instance/player qualified identity。
6. **P1-06** 建立 typed property address、binding target与schema fingerprint。
7. **P1-07** 定义 root/local/global qualified time、range、pre/post-roll。
8. **P1-08** 定义 owner/generation/request/job/receipt 传播。
9. **P1-09** display path、EntityPath、collection index 不得成为 authority key。
10. **P1-10** 建立 deterministic ordering、canonical serialization 与 content digest。
11. **P1-11** 实现 possessable/spawnable binding source 与 qualified resolver。
12. **P1-12** 实现 nested sequence/subsequence hierarchy、time transform、bias、trim。
13. **P1-13** 实现 spawn register、lifetime、orphan、missing binding diagnostics。
14. **P1-14** 建立 binding override、instance context、PIE/world duplication policy。
15. **P1-15** 建立 track/section/shot/folder registry 与 typed factory。
16. **P1-16** 定义 section range、row、overlap、priority、completion、blend policy。
17. **P1-17** 实现 transform/animation/property/camera/audio/event typed adapters。
18. **P1-18** 让 plugin track provider 拥有 codec/compiler/evaluator/editor contract。
19. **P1-19** 建立 compiler dependency graph、validation、cache key 与 artifact。
20. **P1-20** 编译 typed evaluation field，运行时不得解析字符串 property path。
21. **P1-21** 实现 evaluation phase schedule、pre/post evaluation hooks 与 deterministic order。
22. **P1-22** 建立 `CinematicEvaluationInstance` root context 与 scoped state。
23. **P1-23** 实现 pre-animated state capture/restore、abort、error、sequence switch。
24. **P1-24** 定义 Play/Jump/Scrub/Reverse/Loop event traversal 语义。
25. **P1-25** 将 Camera Cut 接入 Editor30 endpoint/history epoch contract。
26. **P1-26** 将 Animation curves/keys 复用 Editor14 typed editor，不复制曲线引擎。
27. **P1-27** 将 media/audio sample 复用 Editor36 timestamp/clock/encoder contract。
28. **P1-28** 建立 preview parity：Editor/PIE/runtime 同 artifact/time 得到同结果。
29. **P1-29** 实现 sequence document session、dirty/save/autosave/recovery。
30. **P1-30** 实现 key/section/shot add/move/trim/slip/split one-transaction commands。
31. **P1-31** 将 key identity 从 collection index 迁移到 stable ID。
32. **P1-32** 任何 validation/compile/save failure 保证 source/dirty/history/selection 不变。
33. **P1-33** 建立 Timeline virtualized rows、ruler、zoom、curve、selection projection。
34. **P1-34** 实现 multi-select、drag、snap、ripple、overlap 和 keyboard commands。
35. **P1-35** 建立 source revision/external-change conflict 与 rebase policy。
36. **P1-36** 让 all UI feedback 来自 provider/job/receipt，不再固定回写。
37. **P1-37** 实现 TakeSource registry、typed source capability 与 arm/prepare lifecycle。
38. **P1-38** 建立 TakeSession clock、frame counter、timecode、metadata、slate、take number。
39. **P1-39** 为 take source 提供 bounded buffer、backpressure、drop/error policy。
40. **P1-40** 实现 start/tick/stop/finalize/cancel/recover 幂等状态机。
41. **P1-41** 录制结果先写 staging，完整校验后 atomic publish TakeAsset。
42. **P1-42** source failure/disk full/device loss/finalize crash 不得发布半 Take。
43. **P1-43** 将录制 section 以 stable binding/key/channel 写回 sequence transaction。
44. **P1-44** 建立 `MovieRenderQueue`、Job、Shot、Preset、Config、Output artifact 类型。
45. **P1-45** Queue submit 冻结 source/map/content/plugin/engine/config fingerprints。
46. **P1-46** Queue 展开 deterministic shot/frame/pass plan 和 checkpoint。
47. **P1-47** 建立 fixed-step movie clock、warmup、pre-roll、post-roll、cut history reset。
48. **P1-48** 实现 temporal/spatial sample、shutter、AA、tile、high-resolution policy。
49. **P1-49** 建立 camera/audio/event/AOV pass 选择与 metadata schema。
50. **P1-50** 复用 Runtime capture/readback，补 format/stride/color/frame/timecode metadata。
51. **P1-51** 建立 bounded readback/CPU conversion/writer pipeline 与 backpressure。
52. **P1-52** 接入 Editor36 encoder/muxer，不在 Cinematic 域复制 codec。
53. **P1-53** output naming、resolution、color、alpha、depth、normal、motion、ID policy typed 化。
54. **P1-54** worker 支持 cancel/retry/resume，并按 shot/frame/pass checkpoint 恢复。
55. **P1-55** artifact 仅在全部 frame/pass/audio 完整校验后 atomic publish。
56. **P1-56** headless 与 Editor worker 共享 compiler、clock、binding、sample schedule。
57. **P1-57** 接入 Editor09 job admission、quota、progress、cancel、shutdown drain。
58. **P1-58** 接入 Editor22 capture/Render Graph 与 Editor30 camera、Editor36 media receipt。
59. **P1-59** 建立 stable diagnostic code、source/shot/frame/pass/item 定位。
60. **P1-60** 建立 compile/evaluate/preview/take/render telemetry 与 budget。
61. **P1-61** 增加 source/schema/ID/time precision/migration golden tests。
62. **P1-62** 增加 binding/spawn/hierarchy/evaluation/pre-animated restore tests。
63. **P1-63** 增加 key/section/shot transaction/failure-zero-mutation tests。
64. **P1-64** 增加 event traversal、camera cut、audio sync、preview parity tests。
65. **P1-65** 增加 Take state、buffer overflow、device/disk/finalize crash tests。
66. **P1-66** 增加 queue expansion、fixed-step、sampling、AOV/color golden tests。
67. **P1-67** 增加 worker cancel/retry/resume、artifact completeness/atomic tests。
68. **P1-68** 增加 plugin unknown provider、codec mismatch、unload、schema migration tests。
69. **P1-69** 增加 1k track/100k key、long take、large queue、multi-shot performance tests。
70. **P1-70** 删除 static feedback、index key、ambiguous Animation API 与第二套 movie writer authority。

### 3.3 P2：主线完成后扩展

1. **P2-01** remote camera/virtual production、live link、timecode hardware。
2. **P2-02** procedural shot generation、batch variant render 与 editorial EDL/OTIO。
3. **P2-03** distributed render farm、asset reservation 与 cloud queue。
4. **P2-04** collaborative Sequencer locks、review、annotation、approval。
5. **P2-05** curve/key ML assist、pose search、motion matching cinematic adapter。
6. **P2-06** realtime viewport stream、remote take monitor 与 multi-camera capture。
7. **P2-07** HDR mastering、OCIO、deep/AOV output 与 color-managed review。
8. **P2-08** shot dependency graph、partial re-render、content-addressed frame cache。
9. **P2-09** audio post mix、ADR、subtitle、caption 与 external DAW interchange。
10. **P2-10** deterministic session replay、temporal debugging 与 frame scrub archive。
11. **P2-11** headless commandlet/CI movie qualification 与 long-run soak。
12. **P2-12** 以相同数据、采样质量、artifact 完整度和 durability 建立超过参考引擎的 benchmark。

## 4. 目标架构与里程碑

```text
CinematicSource -> schema/dependency compiler -> CinematicArtifact
CinematicArtifact + World -> EvaluationInstance -> Camera/Animation/Audio/Event adapters
TakeSession -> staged samples -> TakeAsset transaction
RenderQueue -> Job/Shot/Frame/Pass plan -> capture/readback -> Media encoder -> atomic artifact
```

| Milestone | 退出条件 |
|---|---|
| M0 | 固定 Sequencer feedback、Timeline plugin 假入口和危险 key helper 封口。 |
| M1 | source/ID/time/schema/migration/canonical artifact 冻结。 |
| M2 | binding/spawn/subsequence hierarchy、orphan、qualified resolver 完成。 |
| M3 | track/section/shot/event/camera/audio adapter registry 完成。 |
| M4 | compiler/evaluation/pre-animated/event/cut deterministic parity 完成。 |
| M5 | provider-backed Sequencer document、transaction、save/recovery、virtualized UI 完成。 |
| M6 | Camera/Animation/Media/Render Graph 交叉 contract 与 preview parity 完成。 |
| M7 | Take Recorder source/clock/buffer/metadata/staging/finalize/recovery 完成。 |
| M8 | Queue/Job/Shot/Preset、fixed-step、sampling、output policy/headless worker 完成。 |
| M9 | readback/AOV/color、Editor36 encoder/muxer、checkpoint/resume/atomic artifact 完成。 |
| M10 | cook、fault、migration、long take/large queue、cross-platform determinism/performance 完成。 |
| M11 | 删除 legacy/static authority，32 门资格、docs/manifest/CI/benchmark 闭合。 |

## 5. 验收门

1. **G01-G06** sequence/shot/track/key 来源于真实 provider；plugin capability 缺失时 unavailable；stable ID/time/schema/migration 通过。
2. **G07-G12** binding/spawn/subsequence、range/overlap、typed track/compiler、deterministic artifact、orphan diagnostics 通过。
3. **G13-G18** Play/Jump/Scrub/Loop/event、pre-animated restore、camera cut、audio sync、transaction/preview parity 通过。
4. **G19-G24** Take state、timecode、buffer/backpressure、failure recovery、staging/finalize/atomic publish 通过。
5. **G25-G30** queue freeze/expansion、fixed-step/warmup/sampling/AOV、bounded readback、encoder/muxer、checkpoint/resume 通过。
6. **G31-G32** 1k/100k/long-run benchmarks、fault/security/plugin compatibility、Editor/headless/cook/docs/manifest/telemetry 全部一致。

## 6. 本轮验证与限制

本轮只做静态源码、测试 inventory、参考源码和物理范围 fingerprint 复核；没有修改 Runtime、Editor、Interface、Plugin、App 或 tests，也没有运行 Sequencer、Take、Movie Render、GPU/readback、codec 或跨平台动态验证。frontmatter 路径需在实施前重新展开；P0/P1/P2=5/70/12、M0-M11、32 门和三处索引唯一链接是收尾门。Editor14/22/30/36 的 owner 边界必须保持，不能通过复制 animation/camera/capture/media 实现来“补齐”电影能力；整体 review 仍保持进行中。
