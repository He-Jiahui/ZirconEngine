---
title: Editor Cinematic Sequencer、Shot、Track、Section、Binding、Hierarchy、Evaluation、Camera Cut、Audio、Event、Take Recorder、Movie Render Queue 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor83
review_date: 2026-08-23
baseline_head: 21242973f5255d6e7066842aa99ffd13df53301d
baseline_epoch: 361
canonical_owner: Editor45
refreshes:
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/animation_editor/session/sequence.rs
  - zircon_plugins/timeline_sequence
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/animation/sequence/compiled.rs
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_plugins/sound/runtime/src/timeline
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture_mailbox.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
tests:
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
  - zircon_editor/src/tests/editor_event/runtime/animation_assets.rs
  - zircon_editor/src/tests/editor_event/support.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/36-video-media-source-player-track-clock-media-texture-playback-capture-recording-authoring-review.md
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
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
doc_type: current_source_refresh
review_status: complete
implementation_status: not_started
source_recheck_required: true
canonical_finding_delta:
  p0: 0
  p1: 0
  p2: 0
---

# Editor Cinematic Sequencer、Shot、Track、Section、Binding、Hierarchy、Evaluation、Camera Cut、Audio、Event、Take Recorder、Movie Render Queue 与 Product Integration 当前源码复核

## 1. 结论

Editor45的主结论仍成立：Zircon当前没有工程级Cinematic Sequencer、Take Recorder或Movie Render Queue产品链。当前工作树确实修复了一个重要的局部故障：`move_timeline_keyframe`不再先修改、排序再校验，而是在任何mutation前验证完整sequence，随后用`partition_point`和一次slice rotation维持equal-time稳定顺序；新增测试也覆盖无关坏track、NaN/Inf和相等时间。**旧P0-03所指的“返回失败但对象已被改坏”机制在helper范围内已经静态关闭。**

这个进展不能被扩大成“Sequencer已有事务编辑”。请求仍以`binding_index / track_index / key_index`寻址，没有stable element ID、document revision、selection、dirty、history、undo/redo或operation receipt；插件也没有执行该helper的operation factory。完整P0-03因此只是Partial，不是Closed。性能测试还是`#[ignore = "release performance gate"]`，而每次move仍先全量验证sequence并移动一段Vec，不能证明大规模Timeline编辑预算。

其余电影产品事实没有改变。230行Workbench继续固定显示`SEQ_Intro`、12 shots、428 keys、Camera Cut、Audio Theme、Event Cues、24 fps和`0100-1460`，Preview/Validate直接返回固定queued文本。`timeline_sequence`仍声明一个不存在的`plugins://timeline_sequence/editor/authoring.zui`，五个operation只有descriptor，没有factory；native dist仍是`invoke_command: None`、空command/event manifest、零bridge method。Workbench使用`workbench.extension.sequencer.*`，插件使用`timeline_sequence.*`，两套authority没有桥接。

Runtime `AnimationSequenceAsset`和compiled property writer是真实且应保留的通用底座，但不是电影系统。sequence仍只有秒制duration、浮点fps、path/可选字符串target和property channel；compiled artifact仍保存source Vec的binding/track index。`PendingSequenceSample.entity`进入`LoadedSequenceSample`后从未参与binding，cache按asset ID共享；compile失败静默continue，apply stats又被`let _`丢弃。同一asset在多个player实例上无法拥有不同root context、binding override、spawn register、pre-animated state或可审计receipt。

仓库精确搜索没有发现隐藏的Take Recorder、Movie Render Queue、shot/subsequence、possessable/spawnable、pre-animated state或电影evaluation field实现。Sound拥有独立且真实的`SoundTimelineSequence` automation clock；Render拥有有界viewport readback和单帧PNG原子staging；两者都没有接入Cinematic source/job。capture mailbox甚至在异步readback失败时直接丢弃错误，frame packet也没有format/stride/color/timecode/shot/sample/tile/pass元数据。

因此本轮不新增canonical finding，继续由Editor45唯一拥有原有 **5项P0、72项P1、12项P2**。当前闭合状态为：P0 **4 Open / 1 Partial / 0 Closed**，P1 **61 Open / 11 Partial / 0 Closed**，P2 **12 Open**；Editor45的32个原始产品资格门仍全部Fail。本报告补充48个current-source复验门，也全部Fail。未运行Cargo、真实Editor、GUI/GPU、cook、preview、record、render、fault、soak、profile或同语义跨引擎benchmark，不能声称产品可用，更不能声称性能或表现超过Unreal。

## 2. Owner、currentness与冻结语料

### 2.1 唯一owner与去重边界

本报告是Editor45的current-source refresh，不建立第二套Cinematic owner，不把旧5/72/12重新计入索引。

- Editor45继续拥有Cinematic source、binding/hierarchy、typed track/section、evaluation instance、Sequencer产品、Take orchestration和Movie Render Queue orchestration。
- Editor14、Editor75、Editor77继续拥有通用Animation Sequence、Timeline/Curve、Clip/channel/event/root-motion/sync与prepared animation底座。
- Editor30继续拥有camera endpoint/director/lens/blend、authoritative authored cut与history epoch。
- Editor36继续拥有媒体时钟、timestamped sample、encoder、muxer和durable media artifact。
- Editor22继续拥有Render Graph、capture/debug与下游render authoring；Movie Queue不得复制这些能力。
- Editor63继续拥有document-qualified transaction、history、savepoint、dirty、generation和async operation合同。
- Plugins08/Plugins13继续拥有first-party package/catalog/resource/factory/capability admission闭环；本轮只检查这些缺口如何阻断Editor45产品。
- Runtime08C继续拥有通用Animation运行时；`AnimationSequenceAsset`不得被无界扩展成Level Sequence。

### 2.2 Currentness

- 审查HEAD：`21242973f5255d6e7066842aa99ffd13df53301d`；baseline epoch：`361`。
- 协调session：`optimize-editor83-cinematic-sequencer-review-r2-20260823`；model tier `5.6-sol`，thinking depth `High`。
- 从`da0819cd1134826c26ac2afbaefd3d1c9cfc1804`到本HEAD的提交没有触碰本报告Sequencer/Animation/Capture selected set。
- 冻结时只有`timeline_sequence/README.md`、`editor/src/lib.rs`、`editor/src/tests.rs`三份selected file含其他Session未提交改动；本报告读取当前工作树，不回退、不修改这些源码。
- 当前MVP仍未完成。本轮是C2 review-only文档交付，不提前实现高级Cinematic产品。

### 2.3 可复算selected set

统计口径：路径转小写正斜杠并排序；逐文件SHA-256后，以`path + NUL + lowercase hash + LF`拼接再计算集合SHA-256。tests统计Rust `#[test]`、常见C++ test macro与C# test attribute；ignored单独计数。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Zircon product surface | 9 / 4,668 / 4,505 / 212,283 / 11 / 0 | `9e1b40de9f93e64d1d022fae37d578a499eaacf5b74f3460fe0793cdc67f457b` |
| Zircon timeline plugin | 11 / 1,456 / 1,338 / 54,800 / 16 / 1 | `a266444aafaa1733f496980a63dccdc8ed381f853d7a27ca2fe22af627bd7f18` |
| Zircon animation/editor/runtime | 11 / 2,270 / 2,065 / 75,736 / 3 / 0 | `fd49091586f370a4758686ceabbdc74ab65bec3e0e8567c5ed5284a839ab6b46` |
| Zircon audio/capture substrate | 9 / 1,983 / 1,782 / 65,643 / 25 / 0 | `9bfea0a93b9f20da95f5c0c1a3df6e2b70bcf249c47cf38545b8049d69981656` |
| **Zircon total** | **40 / 10,377 / 9,690 / 408,462 / 55 / 1** | `05aa5c5cefd5517a3952e53702fd4d5999fe7da41f7c94d0a73909ba5848d1b1` |
| Unreal | 13 / 7,277 / 5,906 / 276,477 / 0 / 0 | `a09e021382869e3a7b15c632967ca39ebf683074c0dce4838548f107b4272b87` |
| Godot | 6 / 9,160 / 7,796 / 333,715 / 0 / 0 | `35ba18d1390b864f95bcb382799bf8a3e3607b38307b17e3db0d4ba93b073658` |
| Fyrox | 4 / 2,767 / 2,510 / 107,811 / 1 / 0 | `fd4ccc5ebc1fe0adad4fc6db92b78cd15e36371796d6386ef9dd34eb5c4ede32` |
| Bevy | 3 / 2,882 / 2,607 / 110,234 / 9 / 0 | `667000cb7617c03ab3ac136d8f826ba5f1c7e808d112fe4c87f2637d6d39530c` |
| Graphics | 3 / 438 / 399 / 18,057 / 0 / 0 | `354d965ae2498c3e6c1f9cbd50f29f9a6d4389db8fb6be39acf542324637f846` |
| **Five-engine total** | **29 / 22,524 / 19,218 / 846,294 / 10 / 0** | `1bb3af164fd738f8f514235845a5dd81c4957d0438f14d06c9af5590a0b52e32` |
| **All selected** | **69 / 32,901 / 28,908 / 1,254,756 / 65 / 1** | `3da63504e04ede7f6a526324b19c235787f52e628ed383ed4f42565515b3cbc4` |

Zircon 40文件exact manifest如下；参考29文件即frontmatter的`reference_engines`完整列表：

```text
zircon_app/src/entry/runtime_entry_app/frame_capture.rs
zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
zircon_editor/src/core/editor_authoring_extension.rs
zircon_editor/src/tests/editor_event/runtime/animation_assets.rs
zircon_editor/src/tests/editor_event/support.rs
zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
zircon_editor/src/ui/animation_editor/session.rs
zircon_editor/src/ui/animation_editor/session/sequence.rs
zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs
zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs
zircon_plugins/editor_support/src/lib.rs
zircon_plugins/sound/runtime/src/timeline/advance.rs
zircon_plugins/sound/runtime/src/timeline/validation.rs
zircon_plugins/timeline_sequence/README.md
zircon_plugins/timeline_sequence/dist/Cargo.toml
zircon_plugins/timeline_sequence/dist/src/lib.rs
zircon_plugins/timeline_sequence/editor/Cargo.toml
zircon_plugins/timeline_sequence/editor/src/capability.rs
zircon_plugins/timeline_sequence/editor/src/extension_ids.rs
zircon_plugins/timeline_sequence/editor/src/lib.rs
zircon_plugins/timeline_sequence/editor/src/plugin.rs
zircon_plugins/timeline_sequence/editor/src/tests.rs
zircon_plugins/timeline_sequence/plugin.toml
zircon_runtime/src/animation/sequence/compiled.rs
zircon_runtime/src/animation/sequence/target.rs
zircon_runtime/src/core/framework/animation/asset/sequence.rs
zircon_runtime/src/core/framework/animation/timeline.rs
zircon_runtime/src/core/framework/render/capture.rs
zircon_runtime/src/core/framework/sound/automation.rs
zircon_runtime/src/dynamic_api/frame.rs
zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture.rs
zircon_runtime/src/graphics/runtime/render_framework/viewport_record/capture_mailbox.rs
zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity/velocity_camera_params.rs
zircon_runtime/src/scene/components/scene/animation.rs
```

这69个文件是对Editor45原131文件大清单的current-source delta复核集，不替代旧清单。它专门覆盖发生变化的plugin helper/tests、所有可见Sequencer route/feedback、AnimationSequence source/compiled/apply链、Sound timeline、capture mailbox/PNG publication及五套参考合同。

## 3. 当前Zircon产品链事实

### 3.1 Workbench仍是静态产品伪装

`workbench_extension_sequencer_workspace.zui`固定三份sequence选项、四条track row和摘要。19条route经template binding归一化为tab/row/field/button action，但它们只改变control selection、popup或文本。`extension_module_feedback.rs`为Open、Preview、Validate、Hero Transform和Event Cues直接生成固定结果，既没有source revision，也没有request/job ID、progress、cancel、diagnostic artifact或completion receipt。

Workbench的`workbench.extension.sequencer.*` namespace与timeline插件的`timeline_sequence.*` operation namespace完全分离。仓内没有controller把前者解析到后者，也没有provider把ZUI行投影为`AnimationSequenceAsset`。因此即使插件未来获得factory，当前Workbench仍不会自动成为真实Sequencer。

### 3.2 Plugin只有metadata和局部helper

`timeline_sequence`是editor-only、experimental package，augment `ResourceKind::AnimationSequence`并注册transform/component_property/event_marker三类descriptor。声明的`authoring.zui`不存在，插件目录中ZUI文件数为0。`EditorAuthoringContributionBatch`只包含commands、menus、asset type、track和timeline editor descriptor；没有operation factory字段。native registration还显式声明`extensions: []`，dist没有command invocation和bridge。

当前helper的有效进展必须精确描述：

1. `validate_timeline_sequence`现在拒绝非有限或非正duration/fps，并拒绝非有限key time。
2. `move_timeline_keyframe`先只读校验index、target time和完整sequence，再执行一次局部rotation；失败不再修改sequence对象。
3. equal-time移动采用方向相关的`<`或`<=`边界，保留原有稳定顺序。
4. helper仍以三个Vec index寻址，没有stable key ID或domain command。
5. 每次move仍全量扫描sequence；slice rotation最坏仍为O(n)。唯一性能门被ignore，不能作为release证据。
6. `validate_event_marker_payload`没有检查marker time或duration是否finite/positive；NaN比较为false，可穿过范围检查。

### 3.3 Event capability仍然错配

插件依赖`runtime.feature.animation.timeline_event_track`，但`TimelineEventMarker`只出现在插件lib/tests。它没有进入`AnimationSequenceAsset`、binary codec、dependency/cook、compiled artifact或runtime evaluator。capability字符串由Animation package提供，只能证明package admission metadata存在，不能证明Sequence event track存在。`event_marker`继续可创建会违反fail-close原则。

### 3.4 Runtime底座按asset共享，未形成evaluation instance

`AnimationSequencePlayerComponent`只有sequence、speed、seconds、looping、playing。compiled sequence把每个writer与`binding_index / track_index`绑定，应用时重新从传入source Vec按index取track。它可以用World binding catalog generation检查writer currentness，这是可保留的局部优化；但artifact不是自包含、没有source digest/dependency/provider/root context。

`PendingSequenceSample.entity`被复制到`LoadedSequenceSample.entity`，`apply_loaded_sequences`却从不读取该entity。cache key只有asset ID；同一asset的多个player共享同一份按当前World解析的writer集合。compile失败删除cache后continue，apply结果被丢弃。其后果不是“少一个诊断”这么简单，而是无法定义per-instance binding override、重复level instance、preview/runtime隔离、ownership和restore语义。

### 3.5 Sound和Capture是真实底座，但不是电影adapter

Sound的`SoundTimelineSequence`具有独立sequence ID、duration、looping、automation tracks、schedule/advance和finite validation；这是应复用的音频自动化基础。但它使用Sound manager自身clock/sequence列表，没有Cinematic section ID、qualified time、shot hierarchy、seek/reverse政策或frame receipt。

Render capture有RGBA8/RGBA16F、尺寸、generation、capture report和`GpuReadbackQueue::FRAME_SLOTS`上限；PBR viewer写PNG时使用邻接staging、flush/sync和rename，单帧publication比Editor45审查时的简单描述更可靠。但viewport mailbox把`Err`在`let Ok(rgba) = result else { return; }`处静默丢弃，保留的是最新ready frame而不是有序job frame。它没有movie frame packet、backpressure receipt、AOV assembly、checkpoint或resume。

### 3.6 精确缺失面

排除test、docs、材质render queue和命令面板的“subsequence fuzzy match”后，生产代码只剩静态Camera Cut row和temporal velocity的`CameraCutOrInvalid`运动启发式。以下核心类型/产品均不存在：

- `CinematicSequenceSource / LevelSequence / ShotSection / SubSequenceSection`；
- possessable、spawnable、qualified binding、binding override和spawn register；
- tick resolution/display rate分离、qualified frame time和hierarchy transform；
- evaluation field、pre-animated state、completion mode和evaluation receipt；
- Take source registry、recording state machine、timecode lock、staging/finalize/recovery；
- Movie Queue/Job/Shot/Preset、fixed-step worker、sample/tile/pass、manifest/checkpoint/resume；
- production Sequencer document/controller/factory、真实Outliner/Timeline/Curve selection和transaction。

## 4. Editor45 finding闭合状态

### 4.1 P0状态

| Finding | 状态 | 当前证据与剩余阻断 |
|---|---|---|
| P0-01 静态Sequencer假成功 | Open | 固定`SEQ_Intro / 12 shots / 428 keys`及queued反馈仍在production Workbench route |
| P0-02 无资源/工厂/桥接不得admit | Open | `authoring.zui`不存在，operation无factory，dist零command/bridge，菜单仍注册 |
| P0-03 key move失败零变更 | Partial | 直接mutation-before-validation机制已静态关闭；stable ID、document transaction、dirty/selection/history/revision和undo receipt仍缺 |
| P0-04 Event Marker能力错配 | Open | marker仅为插件本地struct/helper，未持久化、compile或runtime dispatch，NaN还可穿过validator |
| P0-05 禁止包装通用Sequence/单帧capture | Open | 独立cinematic source/job仍不存在，静态产品仍以通用底座制造电影完成感 |

### 4.2 P1状态

没有任何P1达到Closed。以下11项为Partial：

| Finding | 已有可保留部分 | 未闭合核心 |
|---|---|---|
| P1-05 范围/有限值验证 | duration/fps/key/move finite检查 | marker NaN/invalid duration、六类range、统一structured validator |
| P1-06 revision/artifact | asset revision、World writer generation/currentness | dependency/provider/root context fingerprint、LKG、CAS install和receipt |
| P1-12 canonical确定性 | diagnostic/path排序、equal-time稳定顺序、binary codec | 独立cinematic canonical source、stable IDs、跨平台digest |
| P1-19 binding诊断 | helper字符串与apply missing count | source-located typed diagnostic；当前apply report被丢弃 |
| P1-21 property schema compile | property writer预编译并校验binding generation | stable field ID、自包含artifact、schema migration和instance context |
| P1-30 transform/property | 通用property channel可采样和写入 | typed transform空间/compose、section overlap/blend、readonly/type migration receipt |
| P1-31 animation/audio adapter | 通用Animation evaluator与Sound automation均真实存在 | cinematic section adapter、qualified clock、offset/fade/root-motion和统一receipt |
| P1-40 cache/预算 | revision/currentness cache；binary position helper | root-context cache key、worker staging/CAS/cancel、公开预算；perf test仍ignored |
| P1-48 domain command/undo | helper preflight后一次mutation，失败对象不变 | stable ID、revisioned command、selection/history/dirty、coalescing和undo/redo |
| P1-69 bounded readback | viewport pending ring受`FRAME_SLOTS`约束 | ordered movie packet、error receipt、format/timecode/shot/sample/pass与backpressure |
| P1-71 manifest/atomic artifact | 单帧PNG有staging、flush/sync和atomic rename | run manifest、shot/frame/pass checkpoint、checksum、crash resume和整体atomic publish |

其余61项保持Open：

- Source/time：P1-01、02、03、04、07、08、09、10、11。
- Binding/hierarchy：P1-13、14、15、16、17、18、20、22、23、24。
- Track/evaluation：P1-25、26、27、28、29、32、33、34、35、36、37、38、39。
- Sequencer Editor：P1-41、42、43、44、45、46、47、49、50、51、52。
- Take Recorder：P1-53至P1-62全部Open。
- Movie Render：P1-63、64、65、66、67、68、70、72。

### 4.3 P2与原资格门

P2-01至P2-12全部Open。Editor45原G01-G32全部Fail：helper的局部atomic move不足以通过G03，因为G03还要求source bytes、revision、dirty、selection和history对任意失败编辑均不变；有界viewport mailbox也不足以通过G29，因为它会丢错误且缺movie packet metadata。

## 5. 五套参考引擎的工程合同差

| 参考 | 本轮逐实现确认的合同 | Zircon当前差距 | 采用边界 |
|---|---|---|---|
| Unreal MovieScene/Sequencer | tick/display rate、GUID binding、first-class section range/row/overlap/pre-post roll/completion、hierarchy parent/children/transform/bias、Play/Jump/Scrub、root evaluation instance/spawn register、root/focused/local/global time | 电影source、section、hierarchy、instance和editor controller全缺 | Cinematic主参考；提取合同，不复制UObject/Slate |
| Unreal Take/MRQ | slate/take/timecode/frame-rate metadata；source Pre/Start/Tick/Stop/Post；Recorder Start/Stop/Cancel/State；Queue/Job/Shot/config/status；temporal/spatial/warmup/tile/output/handles | Take整链和Queue/worker/output整链全缺 | Take与Movie Render唯一重型主参考 |
| Godot | value/transform/blend-shape/method/bezier/audio/animation typed track；capture与backup/restore；MovieWriter固定fps并同步audio block、检查磁盘和mix-rate整除 | Zircon事件/音频/restore与固定帧输出未接入同一时间域 | 通用typed track、restore、轻量movie writer参考，不替代shot queue |
| Fyrox | Track/Signal UUID、target binding、typed curve、Add/Remove/Replace/Move/Rebind可逆command | Zircon helper仍以Vec index寻址，插件operation无command factory | stable element identity和Editor command最低线 |
| Bevy | UUID `AnimationTargetId`、target-scoped event携带time/weight、ActiveAnimation repeat/seek/previous seek、serializable graph | Zirconsequence target为path/string，event marker无runtime context | typed animation target/event/graph辅助参考，不推断其有Sequencer |
| Unity Graphics | per-camera capture action registry；AOV request明确depth/motion/world position等输出和调用者分配buffer/callback | Zirconcapture仅RGBA/latest-frame mailbox，错误和pass metadata缺失 | 只约束下游capture/AOV边界，不推断Unity Timeline/Recorder |

这些参考共同说明“高性能”不等于删掉工程语义。稳定ID、qualified time、compiled hierarchy、pre-animated restore、typed packet和有界queue是避免frame hot path反复解析、分配、扫描和猜测的前提。Zircon若要超过Unreal，应在同语义合同完整后，通过更紧凑的immutable SoA artifact、interval-indexed evaluation field、dense binding slots、per-instance sparse override、零稳态分配和可测有界worker取得优势。

## 6. 目标架构保持不变

```text
CinematicSequenceSourceDocument
  -> canonical codec + migration + dependency graph
  -> CinematicCompileRequest(source/dependency/provider generations)
  -> immutable CinematicCompiledProgram
       stable dense IDs + hierarchy + time transforms
       binding plans + evaluation field + phased domain schedule
  -> CinematicEvaluationInstance
       root context + overrides + spawn register + pre-animated store
  -> CinematicFrameRequest(previous/current qualified time + update method)
  -> CinematicFrameReceipt(binding/domain/event/restore/diagnostic/timing)

TakeCaptureSession
  -> bounded timestamped source buffers + journal
  -> validated TakeAsset staging
  -> cross-document transaction + atomic publication receipt

MovieRenderRunManifest
  -> frozen Job/Shot/Frame/Sample/Tile/Pass plan
  -> fixed-step evaluator + bounded GPU/CPU/writer pipeline
  -> checkpoint/checksum + Editor36 encoder/muxer
  -> atomic artifact publication receipt
```

必须继续执行Editor45的硬边界：`AnimationSequenceAsset`只作为animation section adapter，不追加shot/binding/movie字段；Sound timeline只作为audio adapter底座；capture mailbox只作为readback substrate；`CameraCutOrInvalid`只保留temporal heuristic；plugin descriptor绝不等于factory或capability实现。

## 7. M0-M11当前状态与重构顺序

| 里程碑 | 当前状态 | 本轮复核后的退出条件 |
|---|---|---|
| M0 真实性封口 | Partial | 保留helper零变更修复；移除固定Sequencer结果，插件缺resource/factory/runtime时Unavailable，event_marker fail-close |
| M1 Source/identity/time | Not started | 独立versioned source、stable IDs、rational frame time、ranges、migration、canonical roundtrip |
| M2 Binding/spawn | Not started | possessable/spawnable、qualified resolver、per-instance override、spawn register、orphan diagnostic |
| M3 Track/section | Not started | executable registry、section/shot/subsequence、camera/transform/animation/audio/event adapter |
| M4 Compiler/evaluator | Not started | hierarchy、evaluation field、phase schedule、pre-animated state、completion、deterministic receipt |
| M5 Sequencer Editor | Not started | provider document、Outliner/Timeline/Curve、stable selection、transaction/save/recovery/diagnostic |
| M6 Camera/audio/event | Not started | authored cut/history、audio sync、event traversal、安全preview与runtime parity |
| M7 Take Recorder | Not started | source registry、clock/state/buffer、metadata、staging/finalize/recovery/browser |
| M8 Render Queue core | Not started | Queue/Job/Shot/Preset、expansion、fixed-step、sampling、output policy、headless worker |
| M9 Offline output | Not started | AOV/color、bounded readback、encoder/muxer、checkpoint/resume、atomic artifact |
| M10 Robustness/scale | Not started | migration、unknown provider、fault、large timeline/take/render、跨平台确定性 |
| M11 Hard cut/qualification | Not started | 删除legacy static/index authority，默认产品装配、CI、benchmark和release gates闭合 |

依赖顺序保持`M0 -> M1 -> M2/M3 -> M4 -> M5/M6 -> M7/M8 -> M9 -> M10 -> M11`。当前只允许继续M0；不得因为helper测试增加而跳到M5，也不得因为单帧PNG已原子写入而跳到M8/M9。

## 8. Current-source复验门（48项，当前全部Fail）

### Authority与Source

- [ ] CSEQ83-G-01：production Sequencer不含固定`SEQ_Intro / 12 shots / 428 keys / 1 gap`authority。
- [ ] CSEQ83-G-02：可见Open/Preview/Validate均解析到revision-qualified document/controller和terminal receipt。
- [ ] CSEQ83-G-03：缺ZUI、factory、codec、compiler、evaluator、bridge或capability时插件为Unavailable且菜单不出现。
- [ ] CSEQ83-G-04：独立`CinematicSequenceSource`拥有schema/source ID/revision/provenance，不扩写AnimationSequence。
- [ ] CSEQ83-G-05：sequence/binding/track/section/shot/folder/marker/channel/key均有持久stable ID。
- [ ] CSEQ83-G-06：tick resolution、display rate、subframe与SMPTE/drop-frame转换使用唯一有理数时间库。
- [ ] CSEQ83-G-07：playback/work/view/selection/render/section range开闭语义、finite与overflow由统一validator处理。
- [ ] CSEQ83-G-08：Windows/Linux canonical save/compile对相同source生成相同digest或显式platform key。

### Binding、Hierarchy与Track

- [ ] CSEQ83-G-09：possessable与spawnable拥有不同source、resolver、lifecycle、copy和cook合同。
- [ ] CSEQ83-G-10：qualified binding在多World、PIE、重复Level Instance和component/subobject下无碰撞。
- [ ] CSEQ83-G-11：同一compiled source可由多个evaluation instance应用不同binding override而互不污染。
- [ ] CSEQ83-G-12：spawn register在play/jump/loop/abort/error/preview teardown后无泄漏对象。
- [ ] CSEQ83-G-13：missing/orphan/ambiguous/type mismatch/stale generation diagnostic携带source location和修复动作。
- [ ] CSEQ83-G-14：track provider同时提供codec/schema/compiler/evaluator/editor factory/migration；缺项只读且不可执行。
- [ ] CSEQ83-G-15：section trim/split/slip/move/overlap/row/priority/pre-post roll/completion拥有golden matrix。
- [ ] CSEQ83-G-16：三层shot/subsequence offset/scale/trim/hierarchy bias在Editor/runtime/render逐frame一致。

### Compiler、Evaluation与Domain

- [ ] CSEQ83-G-17：artifact包含source/dependency/provider fingerprint、dense IDs、binding table、hierarchy和source map。
- [ ] CSEQ83-G-18：artifact不依赖外部source Vec index，source reorder后旧artifact被currentness拒绝。
- [ ] CSEQ83-G-19：evaluation field按interval只访问激活section，1,000 tracks/100,000 keys满足公开预算。
- [ ] CSEQ83-G-20：request区分Play/Jump/Scrub/Reverse/Loop并携带previous/current qualified time和direction。
- [ ] CSEQ83-G-21：phase schedule稳定执行spawn/pre/evaluate/blend/apply/event/post且不依赖注册偶然顺序。
- [ ] CSEQ83-G-22：pre-animated store在stop/unbind/cut/switch/error/cancel/close按completion正确restore/keep。
- [ ] CSEQ83-G-23：camera/transform/property/animation/audio/event adapter输出同一frame receipt并原子处理partial failure。
- [ ] CSEQ83-G-24：compile/apply错误不被continue或`let _`吞掉，LKG/current generation和terminal receipt可检查。

### Sequencer Editor与Transaction

- [ ] CSEQ83-G-25：Outliner/Timeline/Curve/Inspector只投影同一document generation和stable element address。
- [ ] CSEQ83-G-26：key/section/track多选、drag、trim、split、duplicate、跨row move均one transaction提交。
- [ ] CSEQ83-G-27：任何validation/save/compile失败保持source bytes/revision/dirty/selection/history不变。
- [ ] CSEQ83-G-28：undo/redo、save/reopen、reorder、rename、migration后stable IDs和selection逐项相等。
- [ ] CSEQ83-G-29：Add Key读取resolved typed property；missing/mixed/unsupported明确拒绝且不生成占位值。
- [ ] CSEQ83-G-30：Timeline只物化可见row/section/key，hover/selection不改变稳定geometry或全量clone keys。
- [ ] CSEQ83-G-31：Validate投影source-located diagnostic和artifact revision，不输出固定字符串结果。
- [ ] CSEQ83-G-32：PreviewWorld与runtime在相同artifact/request下输出等价binding/domain/event/restore receipt。

### Take Recorder

- [ ] CSEQ83-G-33：Take Source registry为camera/transform/property/animation/audio/plugin source提供typed lifecycle factory。
- [ ] CSEQ83-G-34：Idle/Preparing/Armed/CountingDown/Recording/Stopping/Finalizing/Completed/Failed/Canceled转换幂等。
- [ ] CSEQ83-G-35：engine/audio/external timecode记录rate/epoch/drop-frame/drift/lock，丢锁不静默fallback。
- [ ] CSEQ83-G-36：每source buffer有容量/backpressure/overflow政策，drop/late/duplicate/out-of-order进入receipt。
- [ ] CSEQ83-G-37：长时多source录制满足内存/I/O预算，设备断开和source crash不发布半Take。
- [ ] CSEQ83-G-38：slate/take/timecode in-out/frame rate/source/operator/provenance roundtrip并可检索。
- [ ] CSEQ83-G-39：staging/journal/finalize crash可恢复或完整回滚，checksum不足时只进入quarantine。
- [ ] CSEQ83-G-40：Take publish与sequence section写入使用cross-document transaction且完整undo/redo。

### Movie Render、Output与领先性

- [ ] CSEQ83-G-41：Queue/Job/Shot/Preset与run status分层，submit冻结source/map/content/plugin/engine/config fingerprint。
- [ ] CSEQ83-G-42：相同输入展开出相同shot/frame/sample/tile/pass plan digest，per-shot override可审计。
- [ ] CSEQ83-G-43：fixed-step worker在pause/retry/resume/headless下保持camera/event/random seed逐frame确定。
- [ ] CSEQ83-G-44：warmup、temporal/spatial sample、shutter、tile和authored cut history reset有golden结果。
- [ ] CSEQ83-G-45：beauty/alpha/depth/normal/motion/ID/AOV声明format/stride/color/premultiply和完整frame metadata。
- [ ] CSEQ83-G-46：GPU readback/CPU conversion/writer/encoder有界且错误关联job，不再静默丢弃capture failure。
- [ ] CSEQ83-G-47：manifest/checkpoint/checksum/cancel/retry/resume和atomic artifact通过crash/slow disk/fault矩阵。
- [ ] CSEQ83-G-48：与Unreal及可用Godot/Fyrox/Bevy/Graphics同语义公开测量correctness、编辑/compile/evaluate/render吞吐、内存和artifact size；只有实测领先才允许宣传超过Unreal。

## 9. Review closeout

| 项目 | 状态 | 证据 |
|---|---|---|
| Editor45 canonical owner | preserved | 本报告0新增finding，不重复5/72/12 |
| Workbench产品追踪 | review_complete | 静态ZUI、19 route、binding/navigation/allowlist与5条固定feedback已逐层定位 |
| Plugin资源/执行追踪 | review_complete | 10文件package加support batch；0 ZUI、5 descriptor operation、0 factory、0 command/bridge |
| Key move delta | partial_progress | failure-before-mutation与equal-time顺序静态闭合；index identity、transaction、budget仍缺 |
| Runtime sequence追踪 | review_complete | asset/index artifact/cache/player/apply链确认entity未消费、asset共享和receipt丢弃 |
| Sound/Capture追踪 | review_complete | 独立Sound automation、bounded viewport ring、错误丢弃和atomic single-PNG边界已确认 |
| 五套参考 | review_complete | Unreal主导电影链；Godot/Fyrox/Bevy补通用合同；Graphics只约束capture/AOV |
| P0状态 | 4 Open / 1 Partial | P0-03直接故障机制关闭，但完整finding未关闭 |
| P1状态 | 61 Open / 11 Partial / 0 Closed | partial ID和剩余合同已逐项列出 |
| P2状态 | 12 Open | 无advanced功能达到实现入口 |
| 复验门 | 48 Fail | 未发现可通过的端到端产品门 |
| 动态验证 | not_run | review-only；未运行Cargo、Editor、GUI/GPU、cook、preview、record、render、fault/soak/profile/benchmark |

实施前必须重新读取最新source、Editor45和本报告fingerprint。若当前dirty的timeline helper被继续修改、合入或撤销，至少重跑P0-03、P1-05/12/40/48和CSEQ83-G-03/07/18/24/27/46；若产品route、AnimationSequence artifact、Sound timeline或capture mailbox改变，则重算全部69文件fingerprint后再进入M0。
