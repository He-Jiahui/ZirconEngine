---
title: Editor Cinematic Sequencer、Shot、Track、Binding、Take Recorder 与 Movie Render Queue 当前工作树 authoring 与 preview 复审
category: zircon_editor
report_id: Editor236
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/222-editor-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/176-runtime-cinematic-current-working-tree-sequencer-evaluation-take-render-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/ui/timeline
  - zircon_plugins/timeline_sequence/editor/src/plugin.rs
  - zircon_plugins/timeline_sequence/editor/src/lib.rs
  - zircon_plugins/timeline_sequence/dist/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Public/ISequencer.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/Takes/Source/TakesCore/Public/TakeMetaData.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/Takes/Source/TakeRecorder/Public/Recorder/TakeRecorderSubsystem.h
  - dev/UnrealEngine/Engine/Plugins/MovieScene/MovieRenderPipeline/Source/MovieRenderPipelineCore/Public/MoviePipelineQueue.h
  - dev/godot/scene/resources/animation.h
  - dev/godot/servers/movie_writer/movie_writer.h
  - dev/Fyrox/editor/src/plugins/animation/command/mod.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor236 · Sequencer、Take 与 Movie Render authoring 当前工程化差距

## 1. 结论

当前 Editor Sequencer 是 layout fixture 加 descriptor plugin，不是可交付的 cinematic authoring product。workbench_extension_sequencer_workspace.zui 当前 约 300 行、14,031 bytes，根节点在第 27 行设置 visibility collapsed；第 79、152、168、176、184、206、217 行直接写入 SEQ_Intro、Camera Cut、Audio Theme、Event Cues、12 shots/428 keys 和 24 fps。Preview/Validate 两个按钮只是 route。

共享 callback 进一步给出假阳性证据：extension_module_feedback.rs:142-155 对 open、preview、validate 返回 Sequencer opened、Preview queued SEQ_Intro 24 fps、Validation queued 12 shots 1 gap；workbench_preview_actions/extensions.rs:65-84 将全部动作收录为静态 action id。它们没有 document id、revision、operation receipt、compile job、PreviewWorld/PIE session、runtime frame 或 render output。

timeline_sequence 插件存在可保留的骨架。editor/src/plugin.rs:53-67 注册 authoring extension，138-214 注册 open/create/delete/move/validate descriptor、AnimationSequence augment 和三类 track descriptor；editor/src/lib.rs:38-156 对 duration/fps/key bounds 和 keyframe move 做有限校验，失败保持 source 不变。问题是 move request 仍用 binding_index/track_index/key_index，缺 stable key id、document transaction、handler、compile artifact。dist/src/lib.rs:28-40 明确 stateless、无 command/event manifest、invoke_command 为 None、bridge_methods 为空；plugin.toml 的 authoring.zui 物理路径也不存在。

当前 Editor Sequencer 选择集为 8 个 timeline/plugin 文件、2,941 行、128,242 bytes、15 个 test declaration；Workbench、feedback、navigation、preview action 关联面还会扩大范围。旧 Editor222 的 4 Open/1 Partial P0 与 51 Open/19 Partial P1 不因这些 descriptor 或 helper 而关闭。本报告新增 24 项 P1（21 Open / 3 Partial / 0 Closed）、10 项 P2（10 Open）和 22 道资格门（20 Fail / 2 Partial / 0 Pass），不新增独立 P0。

## 2. 当前断路

### 2.1 Asset、document 与 operation

没有 CinematicSequence ResourceKind、factory、importer、asset toolkit、source ownership 或 stable subobject identity。现有 animation_document 可以维护 Sequence 的 revision/last-good，但不包含 shot、section、binding hierarchy、take metadata、render job 或 camera/audio output。descriptor operation 没有 factory/handler，因此菜单存在不代表 mutation 可以执行。

### 2.2 Graph/timeline UI 与语义

Sequencer Workbench 的 Tracks/Curves/Validation tabs、Camera_A、Event Cues、Camera Cut 和 Hero Transform 行都是固定 node/control；字段 route 只编辑 retained UI 值。没有 tree/graph model、stable track/section/key identity、range/overlap/row/priority、subsequence hierarchy、binding picker、curve compiler、event source map 或 diagnostics span。

### 2.3 Preview、PIE、Take 与 Render

Preview 不创建隔离 PreviewWorld，不安装 runtime artifact，不保存 pre-animated state，也没有 camera/animation/audio/gameplay/VFX adapters。Take Recorder 没有 source discovery、metadata、timestamp buffer、journal 或 crash recovery；Movie Render Queue 没有 queue/job/shot/frame/sample/tile/output/checkpoint。固定 queued 文案会在 backend 不存在时产生产品假成功。

### 2.4 Catalog 与调试

first-party editor catalog 当前 provider 分支主要覆盖 Navigation、Neural；timeline_sequence standalone manifest 没有进入默认 Gameplay/Animation editor composition 的证据。没有 runtime debug mirror、frame receipt、camera cut inspector、event cursor、network/save/replay diff、render failure diagnostics 或 benchmark dashboard。

## 3. 参考编辑器差异

Unreal Sequencer/ISequencer、MovieScene sections/hierarchy、TakeRecorderSubsystem 和 MoviePipelineQueue 将 asset action、graph/schema、evaluation、recording、render queue 分成明确 owner；Godot Animation/MovieWriter、Fyrox animation commands、Bevy animation graph 与 Unity camera capture 都把 stable target、transaction、playback/capture context 和 output provenance 作为合同。Zircon 当前只有 layout、route、descriptor 和局部 validation。

## 4. P1 重构任务

| ID | 当前问题 | 必须完成 |
|---|---|---|
| ED-CINE-01 | 无 provider/catalog/App closure | 增加 manifest、feature、first-party provider、dist/ABI registration，缺 provider 时隐藏或 fail-closed。 |
| ED-CINE-02 | 无 Cinematic resource/factory | 增加 ResourceKind、factory、import/reimport、dependency scan、subasset identity、migration。 |
| ED-CINE-03 | fixture workspace | workspace 改为 document/query snapshot 驱动，去掉固定 SEQ_Intro、shots、keys、fps 和 collapsed 默认。 |
| ED-CINE-04 | 无 authoring document | 建立 CinematicDocument、selection lease、revision、dirty/save/reopen、LKG source/artifact。 |
| ED-CINE-05 | descriptor 无 handler | 每个 open/create/delete/move/validate/preview 绑定 typed operation factory、payload schema、receipt。 |
| ED-CINE-06 | Vec index key move | 使用 stable track/section/key IDs，支持重排、复制、删除和 stale selection rejection。 |
| ED-CINE-07 | 无 graph/schema | 定义 track/section/pin/property schema、range/row/overlap、subsequence hierarchy 与 source map。 |
| ED-CINE-08 | 无 binding authoring | 提供 possessable/spawnable/level-instance picker、binding override、generation/authority diagnostics。 |
| ED-CINE-09 | 无 rational time UI | 支持 tick/display rate、qualified frame、SMPTE/drop-frame、range policy 和 overflow diagnostics。 |
| ED-CINE-10 | 无 curve compiler | 曲线/section 编译与 runtime artifact 同源，显示 interpolation/weight/easing 诊断。 |
| ED-CINE-11 | 无 validation model | 去掉固定 1 gap；诊断含 code、severity、span、object id、fix-it 和 provider generation。 |
| ED-CINE-12 | Preview 静态反馈 | Preview 只在收到 compile/runtime receipt 后显示 success；失败显示真实 diagnostic。 |
| ED-CINE-13 | 无 PreviewWorld | 创建隔离 World、clock、target mapping、pre-animated restore、cancel/timeout/teardown。 |
| ED-CINE-14 | 无 PIE authority | Play/Simulate 区分 editor/preview/server/client，显示 frame/camera/audio/event receipt。 |
| ED-CINE-15 | 无 Take Recorder | 实现 source discovery、metadata、capture transaction、journal、staging、recoverable artifact。 |
| ED-CINE-16 | 无 Movie Render Queue | 实现 queue/job/shot/frame/sample/tile/output/checkpoint、cancel/resume 和 output receipt。 |
| ED-CINE-17 | 无 runtime debug mirror | 提供 per-World artifact generation、qualified time、binding、event cursor、trace loss、bounded history。 |
| ED-CINE-18 | 无 prediction/network/save debug | 显示 camera/animation/audio/gameplay replication、save participant、replay divergence。 |
| ED-CINE-19 | 无 job lifecycle | import/compile/validate/preview/record/render 进入 JobService，支持 cancel/deadline/shutdown drain。 |
| ED-CINE-20 | plugin dist stateless | 增加 command/event/bridge manifest、handler、state schema、unload 和 ABI conformance。 |
| ED-CINE-21 | route namespace 污染 | 用 typed domain action/property path 替代字符串 route 直接 mutation retained control。 |
| ED-CINE-22 | 无 search/reference navigation | track/binding/event/shot usage 索引到 source span、revision 和 asset path。 |
| ED-CINE-23 | 无 scale/fault tests | 100K keys、1K tracks、深层 subsequence、long take、render cancel/device loss、crash/reopen、UI automation。 |
| ED-CINE-24 | 产品状态假阳性 | Open/Preview/Validate/Take/Render 只有真实 operation/runtime/output receipt 才呈成功。 |

## 5. P2 与资格门

P2 共 10 项：minimap/layout persistence、shot thumbnail cache、camera path diagnostics、take metadata review、render farm scheduling、AOV/cryptomatte output view、replay diff、telemetry redaction、artifact eviction、localization/accessibility command coverage。

E1 provider/catalog、E2 resource/factory、E3 document/operation、E4 stable IDs、E5 graph/schema、E6 binding、E7 rational time、E8 compiler parity、E9 diagnostics、E10 PreviewWorld、E11 PIE authority、E12 Take、E13 Render Queue、E14 runtime mirror、E15 job lifecycle、E16 dist execution、E17 search、E18 stale/conflict、E19 scale、E20 fault recovery 当前 Fail；E21 shared document/undo 与 E22 retained route 底座可复用但为 Partial。无 Pass。

## 6. 实施顺序

先完成 provider/resource/document/operation/ID/compiler/diagnostics；再完成 graph/section/binding/time/curve semantic authoring；随后接 PreviewWorld/PIE/runtime mirror/Take/Render/JobService；最后收紧 route、删除 fixture、加入 scale/fault/UI automation。Runtime176 的 CinematicProgramArtifact 和 frame/output receipt 是本报告 Preview、Take、Render 的前置依赖。

本轮仅修改 review/index/coverage 文档，没有修改 editor、runtime、tests、Cargo、ABI 或 ZUI，也没有运行 Editor、Cargo、UI automation、PreviewWorld、PIE、录制、离线渲染、fault、scale、soak 或 benchmark；按用户要求未查询、轮询、等待或实时跟踪协调器。
