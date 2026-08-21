---
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

# 45 · Cinematic Sequencer / Shot / Track / Binding / Take Recorder / Movie Render Queue Authoring 工程化差距

## 1. 结论

Zircon当前没有可称为工程级Cinematic Sequencer、Take Recorder或Movie Render Queue的产品链。产品Workbench展示了名为`SEQ_Intro`的固定序列、12个shots、428个keys、固定轨道和Preview/Validate动作，但这些内容来自静态ZUI和固定feedback，不来自序列资产、运行时evaluation instance、录制session或渲染job。它是当前最危险的差距之一：界面已经制造“能力存在且任务已排队”的印象，而资产、执行器、状态机、回执和产物均不存在。

仓内真实存在的`AnimationSequenceAsset`、Animation Editor、属性轨编译/应用以及单帧capture可以作为底层材料，但不能直接改名为电影系统。现有sequence只有秒制时长、浮点fps、entity path/可选字符串target、property track/channel；没有stable binding、track/section/shot/subsequence、camera cut、audio/event、spawnable、hierarchy time transform、pre/post-roll、pre-animated state或电影时钟。runtime cache又仅按asset revision和当前World编译，apply错误被忽略，无法支持同一序列在不同上下文中的binding override和可审计失败。

`timeline_sequence`插件也不是可执行产品：五项operation只是descriptor，声明的authoring ZUI不存在，dist没有command或bridge method；其`event_marker`描述与实际依赖能力不相符，运行时能力只实现`AnimationClipAsset.event_tracks`，没有把插件本地marker持久化到`AnimationSequenceAsset`或在sequence evaluator中播放。`move_timeline_keyframe`还会先原地修改/排序，再执行全序列校验；当无关既有错误使校验失败时，函数返回`Err`但对象已经变更，且索引身份在排序后漂移。

目标不是复制一个更大的时间轴界面，而是建立`versioned cinematic source -> stable hierarchy/binding/section compiler -> deterministic evaluation instance -> transactional editor -> take session -> durable render queue/artifact`闭环。Camera Director/Cut由Editor 30拥有，媒体编码/封装由Editor 36拥有，Render Graph与调试capture由Editor 22拥有，通用Animation资产/曲线编辑由Editor 14拥有；本专题只拥有电影编排和上述能力的typed orchestration。

## 2. 审查范围与证据强度

本轮逐文件冻结131个selected path，共71,741行、2,908,630 bytes、115个test attributes、0 ignored。清单按规范化路径排序，对每个文件计算小写SHA-256，再以`forward/slash/path|hash`、LF连接且末尾无LF形成manifest，当前工作树fingerprint为`ac54c4e41dc130b0d4d7ae407f3dc923906f57a44e709e544acfce588ad72f5f`。

| 子域 | 文件 / 行 / bytes | 本轮判定 |
|---|---:|---|
| Zircon静态Sequencer surface | 7 / 12,870 / 704,599 | E3逐ZUI、route、binding、navigation、preview设计和feedback；0个tests，确认固定数据与固定queued/success反馈 |
| Zircon timeline plugin/contracts | 13 / 2,330 / 81,315 | E3逐完整package、manifest、dist、resource、descriptor、helper和tests；15个tests，确认无执行工厂且事件能力错配 |
| Zircon runtime sequence/event/playback | 18 / 3,065 / 107,378 | E3逐asset、target、compiler、cache、sample、apply和capability；17个tests |
| Zircon generic Animation Editor | 25 / 3,419 / 133,463 | E3逐session、sequence/graph、transport、host lifecycle/save/sync；27个tests |
| Zircon camera/capture/output substrate | 11 / 2,922 / 111,078 | E3逐capture DTO/mailbox/readback/dynamic API、camera stack与PNG evidence；38个tests |
| Unreal参考 | 26 / 12,093 / 449,556 | E2/E3按MovieScene、Sequencer、Take Recorder和Movie Render Pipeline职责路由 |
| Godot参考 | 12 / 23,338 / 855,551 | E2/E3按typed animation tracks、player backup/restore和MovieWriter路由 |
| Fyrox参考 | 11 / 7,396 / 289,488 | E2/E3按UUID track/signal、command editor、ruler/curve/preview路由；8个tests |
| Bevy参考 | 3 / 2,882 / 110,234 | E2/E3按AnimationTargetId、clip/event/graph/player路由；10个tests |
| Unity Graphics参考 | 5 / 1,426 / 65,968 | E2/E3仅按camera capture callback、AOV buffer与completion路由，不推断Timeline/Recorder |
| 合计 | 131 / 71,741 / 2,908,630 | 115个test attributes、0 ignored；5个在途文件 |

冻结时以下selected file已有非本轮改动：`zircon_editor/src/ui/animation_editor/session/graph.rs`、`zircon_editor/src/ui/animation_editor/session/sequence.rs`、`zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs`、`zircon_runtime/src/core/framework/render/camera_stack.rs`、`zircon_runtime/src/dynamic_api/frame.rs`。本轮没有编辑它们；实施前必须重算fingerprint并复核终态。

证据等级沿用总审查约定：E3为逐实现/调用/测试追踪，E2为目标引擎局部合同阅读，E1为名字或声明命中。本报告不把resource marker、capability字符串、menu descriptor、DTO、静态preview或单帧capture算作可用电影产品。

## 3. 当前实现事实

### 3.1 静态Sequencer正在制造虚假产品状态

`workbench_extension_sequencer_workspace.zui`共230行，固定显示`SEQ_Intro`、`Camera_A`、Event Cues、四条预制track row、12 shots、428 keys、24/30/60 fps与工作范围。20个production action ID虽然进入allowlist、navigation和template binding，但没有document provider、controller、repository、compiler、evaluation instance、recording session或render job。feedback callback直接返回“opened `SEQ_Intro`”“preview queued `SEQ_Intro` 24 fps”“validation queued `12 shots 1 gap`”等固定文本。

`tools/editor-workbench-preview/design.js`又展示Sequencer/Shot/Takes/Render Queue的设计意图，但production ZUI甚至没有Render Queue命令。这份设计文件只能作为视觉原型，不能提升产品证据等级。当前Preview/Validate没有source revision、request ID、job ID、progress、cancel、diagnostic artifact或completion receipt，应在M0 fail-close。

### 3.2 通用Animation Sequence是底座，不是电影资产

`AnimationSequenceAsset`的真实优点是：有asset identity/revision、duration/fps、binding/property track/channel，compiled sequence可把property writer预编译并避免frame hot path解析路径。这些能力应保留并由Editor 14继续维护。

但电影语义缺口是结构性的：marker registry只有`AnimationSequence`，没有Level/Cinematic Sequence、Shot、Take或Movie Render Job资源；binding依赖`EntityPath`和可选字符串target，运行时把字符串尝试解析为当前`EntityId`；没有possessable/spawnable、qualified world/instance context、section和hierarchy。时间是`f32/f64 seconds + playback_speed`，不是tick resolution/display rate分离的有理数帧时间；player只有sequence/speed/time/looping/playing。

runtime sequence cache只按asset ID、revision和当前World currentness维护。`LoadedSequenceSample.entity`没有参与apply，编译失败会移除cache后继续，apply结果被`let _`丢弃。因此现有链无法为相同source的不同player建立独立binding override、root instance、pre-animated restore或错误回执。`AnimationTimelineDescriptor`只是无production caller的DTO；`CameraSequenceReport`表示render camera stack，`CameraCutOrInvalid`是temporal invalidation运动启发式，均不是电影序列或authoritative camera cut。

### 3.3 Timeline插件的声明、持久化与执行链断裂

`timeline_sequence`为editor-only experimental package，依赖`runtime.feature.animation.timeline_event_track`。它注册timeline editor和transform/property/event-marker三类track descriptor，但`EditorAuthoringContributionBatch`只收集descriptor vector，没有operation factory/executor；五个operation无法被实例化和执行。manifest声明的`plugins://timeline_sequence/editor/authoring.zui`物理缺失，dist为stateless、`invoke_command: None`、`bridge_methods: []`。

插件本地`TimelineEventMarker`只在helper/tests出现，没有进入`AnimationSequenceAsset`。所依赖的partial animation capability实际服务`AnimationClipAsset.event_tracks`并在clip runtime采样发布事件，不服务sequence。因此“注册event_marker track”不能证明保存、重开、cook或运行时播放。

`move_timeline_keyframe(binding_index, track_index, key_index, time)`以collection index作为身份，修改key时间并排序后才校验整个sequence。任何不相关既有校验错误都可能导致返回失败但mutation保留；排序后原key index也不再稳定。它不能接入Editor transaction、undo/redo或协同控制面。

### 3.4 Capture链只能证明单帧读回

Runtime capture提供RGBA8或RGBA16F像素、width/height、generation及report，mailbox以三slot保留最新ready generation并丢弃失败；dynamic API和Editor poll只校验RGBA形状。PBR viewer可以写单张PNG与provenance，这是有价值的evidence writer。

但frame没有format/stride/color space/PTS/timecode/camera/shot/sample/tile/pass/layer信息，也没有固定步进、warmup、temporal/spatial sampling、motion blur shutter、AOV、checkpoint、resume或atomic output publication。Editor 36确认通用encoder/muxer尚待建设。本专题不得用循环调用单帧capture来宣称Movie Render Queue完成。

### 3.5 参考引擎提供的最低工程合同

Unreal `MovieScene`把tick resolution与display rate分开，以frame number/range描述播放区间；binding使用stable GUID并拥有track集合；section具有range、row、overlap、pre/post-roll、completion和trim/split语义；hierarchy保存parent/children、subsequence transform和bias。Sequence Player区分Play、Jump、Scrub及其事件穿越语义，并拥有root evaluation instance、spawn register与恢复状态。Sequencer Editor维护root/focused sequence、local/global qualified time、focus transform、selection和change notification。

Take Recorder另外拥有source registry、prepare/start/tick/stop/finalize生命周期、slate/take number/timestamp/timecode/frame rate元数据。Movie Render Pipeline将queue、job、shot、preset/config、status分层，并明确spatial/temporal sample、warmup、tile、output naming/resolution、root/shot frame和authored camera cut history invalidation。

Godot证明typed value/method/bezier/audio/animation等track、marker、double time、player backup/restore和固定fps MovieWriter是成熟通用动画/输出的最低线，但它不是完整shot queue参考。Fyrox证明UUID track/signal、command-based编辑、ruler/curve/preview；Bevy证明UUID AnimationTargetId、clip/event/graph/player。Unity Graphics镜像只证明per-camera capture callback和HDRP AOV buffer/completion合同，不足以证明Unity Timeline或Recorder。

## 4. 目标架构与责任边界

```text
CinematicSequenceSource
  -> schema/migration/dependency validation
  -> CinematicCompiledArtifact
     (hierarchy + binding table + evaluation field + deterministic schedule)
  -> CinematicEvaluationInstance
     (root context + spawn register + pre-animated state + receipts)
  -> Camera/Audio/Animation/Event domain adapters

TakeSession -> recorded source fragments -> transactional TakeAsset publication

MovieRenderQueue -> Job -> Shot -> Sample/Tile/Pass
  -> capture/readback -> Editor36 encoder/muxer -> manifest/checkpoint/artifact
```

| Owner | 拥有 | 不拥有 |
|---|---|---|
| Editor45 Cinematic domain | sequence/shot/section/binding source、hierarchy compiler、evaluation instance、Sequencer UI、Take协调、render queue/job/shot orchestration | 通用动画曲线算法、camera实现、codec/muxer、Render Graph debugger |
| Editor14 Animation | AnimationSequence/Clip/Graph通用文档、曲线/键编辑、property writer与preview substrate | shot/subsequence、movie binding、take、render queue |
| Editor30 Camera | camera endpoint/rig/lens/director/blend/shake、authoritative cut与history epoch | sequence hierarchy和render job |
| Editor36 Media | timestamped media sample、recorder sink、encoder、muxer、durable media artifact | shot scheduling、cinematic evaluation |
| Editor22 Rendering | Render Graph、frame capture/debug、lighting/probe/post-process authoring | cinematic source与movie job policy |
| Runtime Asset/Scene | versioned source codec、dependency/cook、stable object identity、runtime install | Editor transaction和queue UX |
| Plugin SDK | typed track/source/output provider factory、capability/admission、unknown-data preservation | descriptor即执行器、未协商远程代码 |

## 5. P0：启用前必须封闭

### P0-01 静态Sequencer假成功面必须fail-close

移除production profile中的固定`SEQ_Intro`、12 shots、428 keys及queued/success feedback。只有真实provider打开revision-qualified document，Preview/Validate返回request/job ID并可查询最终receipt时，动作才可用；设计preview继续与production authority隔离。

### P0-02 Timeline插件无资源、无工厂、无桥接时不得admit

manifest资源必须物理存在且通过schema/hash验证；每个operation/track必须解析到typed factory、executor和权限声明。仅注册descriptor、`invoke_command: None`或零bridge method时，插件状态必须为Unavailable，不能出现可执行菜单。

### P0-03 Keyframe移动必须保证失败零变更

立即隔离`move_timeline_keyframe`，禁止任何产品调用。重建为stable key ID驱动的preflight + clone/staging + scoped validation + one transaction commit；任何错误都保持source、dirty、selection、history和revision完全不变，并提供undo/redo receipt。

### P0-04 Event Marker不得建立在错误能力上

在`CinematicSequenceSource`拥有可持久化event section、compiler生成interval evaluation field、runtime对play/jump/scrub/reverse/loop有明确派发策略前，移除`event_marker`可用声明。Clip event capability不能满足Sequence event capability。

### P0-05 禁止把AnimationSequence或单帧capture包装成电影产品

在独立versioned cinematic source、stable binding/hierarchy、deterministic movie clock、pre-animated restore、queue/job/shot和durable artifact authority建立前，Cinematic/Take/Movie Render入口全部保持Unavailable。循环capture不得返回movie成功，普通animation player不得返回shot/cut完成。

## 6. P1：工程级主链重构

### 6.1 Source、身份与时间合同

### P1-01 建立独立的versioned Cinematic Sequence source

新增明确schema version、source ID、revision、catalog fingerprint和editor/cook provenance的`CinematicSequenceSource`，不向现有`AnimationSequenceAsset`无界追加电影字段。两者通过typed animation section adapter复用，而不是继承式混合authority。

### P1-02 为所有可编辑元素分配stable ID

Sequence、binding、track、section、shot、folder、marker、channel和key均使用持久化ID；数组索引只作布局，不得进入selection、undo、diff、plugin API或外部引用。排序、插入、迁移和协同重放后ID保持不变。

### P1-03 分离tick resolution与display rate

源时间使用有理数frame time/subframe，分别存储内部tick resolution和显示fps；转换必须检查溢出、舍入和不兼容rate。播放、编辑、timecode、record和render共用同一转换库。

### P1-04 冻结versioned schema与migration registry

每个source/track/section/provider拥有type ID、schema version、migration chain和canonical codec。缺provider时以opaque placeholder无损保留，未知字段不得因打开、保存或cook静默删除。

### P1-05 统一范围与有限值验证

明确playback、work、view、selection、section和shot range的开闭区间语义；拒绝NaN/Inf、负tick resolution、倒置范围和不可表示变换。所有入口使用同一validator和structured diagnostic。

### P1-06 建立source revision与compiled artifact合同

编译产物记录source revision、dependency revisions、provider versions、platform/cook key和content digest。stale artifact不能安装；成功swap采用generation，失败保留最后一个已验证generation并给出receipt。

### P1-07 建立直接引用与cook依赖图

shot/subsequence、animation、audio、camera、material和output preset引用进入asset dependency index；rename/move/redirector由stable resource identity修复。缺失或循环依赖在save/cook前阻断，而非播放时临时猜测。

### P1-08 统一marker、label与timecode metadata

marker拥有stable ID、frame time、label/color/tags和可选range；源可携带start timecode、frame-rate provenance及drop-frame policy。它们必须roundtrip、可搜索，并与运行时event section严格区分。

### P1-09 区分五类编辑/播放范围

playback、work、view、selection和render range独立持久化或按workspace保存；修改view range不得改source，修改render handles不得破坏playback。UI和CLI明确显示当前range authority。

### P1-10 建立lock、read-only与checkout状态

document provider投影asset lock、source-control checkout、external change和schema/provider缺失状态。所有写动作在preflight前检查authority，read-only模式仍允许scrub、inspect和导出diagnostic。

### P1-11 统一subframe与drop-frame规则

key/section/evaluation支持subframe精度；SMPTE timecode与drop-frame仅影响合法rate的显示/换算，不改变源tick。record、render和导出必须保留rate、timecode及舍入receipt。

### P1-12 保证canonical serialization与确定性排序

map、binding、track、section和channel按stable canonical key编码；浮点/有理数、颜色、路径与provider payload有唯一表示。Windows/Linux重复save、compile和cook必须得到相同digest。

### 6.2 Binding、Spawn与实例上下文

### P1-13 区分possessable与spawnable binding

possessable绑定外部World对象，spawnable由sequence拥有template和生命周期；两者拥有不同解析、保存、复制和cook规则。禁止用当前`EntityId`字符串同时承担两种语义。

### P1-14 使用qualified object identity

binding target至少包含project/world/level-or-instance/object/component identity和resolver namespace；运行时`EntityId`只能是某generation内的解析结果。PIE、重复Level Instance和多World下不得碰撞。

### P1-15 支持per-instance binding override

同一compiled source可在不同player/root context中绑定不同对象，override存于evaluation instance或明确的binding override asset。cache key必须包含source generation、root context和provider compatibility。

### P1-16 提供binding tag、display name与查询索引

稳定ID负责引用，tag/name负责作者检索和批量替换。重命名不改变identity；重复tag和歧义查询返回完整候选，不得first-match静默成功。

### P1-17 支持component与subobject binding

binding resolver能从对象稳定定位component/subobject，并校验expected type/schema。组件替换、reparent和class migration产生rebind/orphan diagnostic，不退回字符串property path猜测。

### P1-18 建立spawn register与生命周期

evaluation instance拥有spawn register，处理create、reuse、destroy、ownership、pre/post-roll、loop、jump和abort。失败或停止后不得遗留对象；编辑preview与runtime使用相同状态机。

### P1-19 对missing/orphan target输出结构化诊断

区分missing world、missing object、missing component、type mismatch、ambiguous、provider unavailable和stale generation。诊断携带binding/section/time/source location和修复动作，不能只跳过apply。

### P1-20 Rebind必须是事务动作

Rebind预览受影响track/section和类型兼容性，使用stable binding ID更新引用，支持undo/redo和CAS revision。失败后source、selection和compiled generation零变化。

### P1-21 编译property schema而非每帧解析path

binding+track编译为typed access plan，记录target type ID、field stable ID、codec和writer generation。schema变化使artifact失效并触发migration/diagnostic，frame loop不解析字符串路径。

### P1-22 支持明确的multi-object binding policy

一个binding可按provider policy解析单对象或有序对象集合；集合排序、缺项和动态变化语义必须确定。track声明是否支持fan-out，不能由偶然查询结果决定。

### P1-23 将hierarchy context传入嵌套序列

subsequence实例携带parent sequence ID、instance ID、time transform、binding remap和hierarchical bias。递归cycle、过深层级和实例爆炸在编译期有预算和完整诊断链。

### P1-24 冻结PIE、network与runtime binding边界

Editor对象、PIE副本、remote game process和network entity之间通过显式resolver/adaptor映射。电影序列不直接广播Editor handle，也不把gameplay replication误作authoring binding authority。

### 6.3 Track、Section与Evaluation

### P1-25 建立可执行typed track registry

track type注册必须同时提供source codec、section/channel schema、compiler、runtime evaluator、Editor factory、migration和capability contract。缺任一关键部分时只以只读unknown track显示，不得注册为可创建类型。

### P1-26 将section设为一等对象

section拥有stable ID、range、row、priority、active/locked、overlap/blend policy、pre/post-roll和completion mode。trim、split、slip、duplicate、move和resize由domain command实现，并共享range validator。

### P1-27 建立shot与subsequence track

shot引用子sequence并携带display metadata、handles、camera binding/override和render enable；普通subsequence提供可复用嵌套编排。两者都编入hierarchy，不能用UI row文本或animation clip替代。

### P1-28 编译嵌套时间变换

支持parent-to-child offset、scale、start/end trim及必要的loop/warp扩展，生成可逆或可诊断的qualified time transform。root/local/global time在Editor、runtime和render中必须一致。

### P1-29 接入authoritative Camera Cut

camera cut section绑定Editor 30的camera endpoint/director，明确cut与blend、viewport/player、lens state和history epoch。不得依赖`CameraCutOrInvalid`运动启发式猜测作者cut。

### P1-30 完成transform/property track语义

transform track定义空间、轴、插值、权重和component composition；property track只接受已编译typed channel。多section blend、缺target、只读field和类型迁移都有确定结果和diagnostic。

### P1-31 完成animation与audio section adapter

animation section引用Editor 14资产并定义offset/rate/loop/root motion/blend；audio section引用Editor 36媒体/音频能力并定义start offset、volume/pitch/fade和clock sync。adapter不得复制底层decoder或pose evaluator。

### P1-32 定义event section派发策略

event拥有stable ID、payload schema、direction/seek policy、fire-once/loop policy和权限域。Play、Jump、Scrub、reverse、loop wrap、subsequence和network replay分别有golden behavior，Editor preview默认不得执行危险副作用。

### P1-33 提供核心domain track adapters

按需求加入visibility、material parameter、fade、time dilation、spawn和domain-specific adapter；每类都走registry及版本化payload。禁止再次以任意字符串method/property构造无权限执行面。

### P1-34 建立阶段化evaluation schedule

编译为spawn/instantiate、pre-evaluate、evaluate、blend/resolve、apply、event、post-evaluate等稳定phase，并显式声明依赖和冲突。执行顺序不依赖HashMap、文件或插件注册偶然顺序。

### P1-35 构建interval evaluation field

按section有效区间、hierarchy transform和track template生成可查询evaluation field，seek或frame advance只访问受影响段。编译结果可检查、可profile，并有source location映射。

### P1-36 捕获pre-animated state

首次接管property/object前记录原状态和ownership，按completion mode选择restore或keep。停止、切换sequence、unbind、error、cancel和Editor preview teardown均经过统一restore路径。

### P1-37 明确completion与ownership mode

section/track/sequence层定义restore、keep和project default policy，嵌套冲突按显式precedence解决。不同evaluation instance不互相覆盖备份，最后owner退出时才恢复。

### P1-38 支持pre-roll、post-roll与warmup

编译器将pre/post-roll传播到subsequence和domain adapter；camera、particles、audio和render temporal histories可声明warmup需求。warmup evaluation与正式输出帧严格区分，不误派发普通事件。

### P1-39 区分Play、Jump、Scrub、Reverse与Loop

evaluation request携带update method、previous/current qualified time、direction和traversed intervals。事件、spawn、audio、restore和motion blur依据该合同处理，不能只有`time += delta`。

### P1-40 建立cache invalidation、并发与预算

cache key覆盖source/dependency/provider/schema/root-context generation；编译在worker staging，安装在明确同步点。限制hierarchy深度、track/section/key数量、evaluation fan-out、compile time和artifact memory，并支持取消。

### 6.4 Sequencer Editor与事务

### P1-41 用provider-backed document替换静态Workbench

Sequencer打开真实source并投影revision、dirty、read-only、compile status、selected IDs和diagnostics。所有按钮通过controller调用typed command/job，UI不自造资产数据或完成状态。

### P1-42 建立可扩展Track Outliner

Outliner支持binding/track/folder层级、rename、mute/solo/lock、filter、pin和type icon；行由registry factory生成。展开/选择状态按workspace保存，不污染source semantic data。

### P1-43 虚拟化Timeline Canvas

仅布局和绘制可见row、section、key与label，稳定row geometry避免hover/selection改变布局。十万key、千track和深层folder下内存、输入延迟与frame time满足预算。

### P1-44 完成Ruler、Zoom、Scroll与Snap

ruler显示frame/timecode，zoom以pointer/focus为锚；scroll、playhead、work/view range同步。snap registry覆盖frame、key、section edge、marker和custom provider，并显示命中来源。

### P1-45 完成多选与直接操作

key/section/track支持框选、toggle、range select、drag、trim、slip、split、duplicate和跨row移动。操作使用stable IDs、preview delta和one transaction commit，取消恢复原状态。

### P1-46 复用工程级Curve Editor

float/vector/transform channel接入Editor 14 curve editor，支持tangent、weighted tangent、interpolation、extrapolation、normalize和multi-channel overlay。曲线编辑和timeline selection共享identity与transaction。

### P1-47 从当前typed property插入key

Add Key读取resolved target当前值和field codec，明确无target/mixed/unsupported状态；禁止继续复制最后一个key或使用`Scalar(0)`伪造值。auto-key只在已授权track和transaction内触发。

### P1-48 建立domain command与undo coalescing

每个编辑动作包含before/after、source revision、affected IDs、validation和receipt。连续drag可合并为一次undo，结构变更、外部change或失败终止coalescing；undo/redo后compile和selection一致。

### P1-49 接入dirty、autosave与recovery

Sequencer document复用Editor 02保存主链，支持revision CAS、atomic save、autosave snapshot、crash recovery和external-change conflict。plugin unknown payload和未加载引用也必须无损roundtrip。

### P1-50 投影结构化diagnostic与修复

错误按sequence/binding/track/section/key/time定位，可在Outliner和Timeline跳转；修复动作必须是可预览、可撤销的typed command。Validate输出artifact和source revision，不用固定“1 gap”。

### P1-51 建立隔离Preview与运行时一致性

Preview使用明确World/viewport、root context、camera/audio/event安全策略和evaluation artifact，stop时完整restore。相同source/time/request在Preview、PIE和runtime产生等价domain sample，差异有receipt。

### P1-52 为插件提供真实factory边界

插件按能力注册track/section editor factory、compiler/evaluator和optional detail customization；卸载时保留opaque source并关闭编辑。ABI/version/capability不匹配时fail-close，descriptor文字不构成产品。

### 6.5 Take Recorder

### P1-53 建立Take Source registry

可录制对象、camera、transform/property、animation、audio和插件source通过typed factory注册，声明schema、clock、buffer、prepare/start/tick/stop/finalize能力。缺provider或权限时source不能arm。

### P1-54 实现明确的录制状态机

状态至少包括Idle、Preparing、Armed、CountingDown、Recording、Stopping、Finalizing、Completed、Failed和Canceled；转换由controller拥有并持久化receipt。重复start/stop、cancel和异常退出均幂等。

### P1-55 接入authoritative timecode与frame source

Take session选择engine fixed clock、audio clock或外部timecode provider，并记录rate、epoch、drop-frame、drift和lock status。provider丢锁、rate变化和回拨按policy暂停/失败，不静默继续。

### P1-56 使用有界采样buffer与drop receipt

每个source声明采样率、最大buffer、backpressure和overflow policy；内存按session预算。任何drop、late、duplicate或out-of-order sample进入结构化receipt和UI告警。

### P1-57 提供核心录制source adapters

transform/property/camera/animation/audio adapter分别把timestamped sample归一化为typed recorded channel/section；低频控制与高频sample分离。插件source同样必须通过schema和buffer合同。

### P1-58 建立Take metadata资产

Take记录slate、take number、description、timestamp、timecode in/out、frame rate、level/sequence、source list、operator和provenance。编号分配具备collision/CAS规则，元数据随资产roundtrip与检索。

### P1-59 实现staging、finalize与故障恢复

录制先写session staging/journal，stop后校验并原子发布TakeAsset及sequence sections。任一source/finalize/save失败保留可恢复staging或完整回滚，不留下半资产或虚假成功。

### P1-60 建立Take Browser与命名策略

按slate/take/date/source/status搜索、比较、打开和替换；命名模板使用typed token并预检路径、冲突和长度。删除/重录遵循asset dependency与回收策略。

### P1-61 录制写入必须非破坏且可撤销

默认生成新Take或新section，不覆盖原始曲线；replace/merge先展示diff和范围。最终sequence修改通过one cross-document transaction，失败或undo恢复原引用与资产状态。

### P1-62 提供录制监控、诊断与性能预算

UI实时显示clock lock、duration、每source sample/drop/buffer、I/O和finalize进度；日志关联session/source ID。长时录制、设备断开、磁盘不足和source crash有fault tests与资源预算。

### 6.6 Movie Render Queue

### P1-63 建立Queue、Job、Shot与Preset source

Queue拥有有序job；job引用sequence/map/config；shot来自编译后的shot hierarchy并可enable/disable；preset/config版本化且可继承。status、attempt和output artifact属于运行记录，不污染authoring preset。

### P1-64 编译shot expansion与per-shot override

提交时冻结source/dependency revisions，把job展开为可审计shot plan，应用camera、range/handles、resolution、quality和output override。重复展开在相同输入下得到相同digest。

### P1-65 使用deterministic fixed-step movie clock

每个输出frame由qualified frame time驱动simulation/evaluation，不依赖wall-clock或viewport delta。暂停、重试、分片和headless执行保持帧、事件和随机种子确定。

### P1-66 支持warmup、采样与shutter合同

配置engine/render warmup、temporal/spatial sample count、shutter timing和motion blur delta；每个sample携带frame/shot/sample index。authored camera cut显式清空相应history，不能靠运动阈值猜测。

### P1-67 建立输出pass、layer、AOV与color pipeline

输出声明beauty、alpha、depth、normal、motion、object/material ID或provider AOV，定义pixel format、color space/transfer、premultiply和metadata。能力不支持时在queue admission失败，不降级成RGBA截图。

### P1-68 建立安全output token与磁盘预检

路径模板只接受注册token，如project/sequence/job/shot/camera/frame/pass；规范化后必须位于授权root。提交前检查collision、覆盖policy、预计容量、free space和文件名限制。

### P1-69 构建有界异步readback pipeline

render、GPU readback、CPU conversion和writer使用有界队列与backpressure，frame packet包含format/stride/color/frame/timecode/shot/sample/tile/pass。失败不得被mailbox丢弃，必须关联job receipt。

### P1-70 支持headless worker、cancel、retry与resume

worker加载冻结manifest并验证engine/plugin/content fingerprint；job/shot可取消，transient failure按policy重试。resume只消费已校验checkpoint，不重复发布或跳过未知帧。

### P1-71 建立manifest、checkpoint与atomic artifact

每次运行生成不可变manifest、resolved config、shot plan、frame/pass receipt、diagnostics、checksums和provenance。输出先写staging，完整校验后atomic publish；crash可清理或恢复。

### P1-72 复用Editor36编码封装并提供真实队列UX

图片序列、视频和音频交给Editor 36的provider-neutral encoder/muxer，不在本专题复制codec。Queue UI展示pending/running/finalizing/completed/failed/canceled、进度、ETA、日志、产物和retry，只有artifact发布后才显示成功。

## 7. P2：主链稳定后的增强项

### P2-01 非线性time warp与retiming

在有理数基础时间、hierarchy transform和事件语义稳定后，引入可版本化time-warp curve、freeze/reverse段和可诊断逆映射。

### P2-02 Rig、Constraint与Control集成

以typed track/provider接入camera/character rig、constraint和control channel，不把rig evaluator塞入Cinematic核心。

### P2-03 Live Link与Virtual Production

增加设备发现、校准、时码锁定、数据路由和可恢复录制session，所有外部输入保留设备/provenance metadata。

### P2-04 OTIO、EDL与AAF式交换

通过明确支持矩阵导入/导出镜头、timecode、handles和media reference；无法表示的track生成loss report而非静默flatten。

### P2-05 分布式Render Farm

在单机headless/checkpoint确定性通过后，增加worker capability匹配、lease、分片、重试、artifact合并和供应链验证。

### P2-06 Path Tracing与Denoise Accumulation

提供路径追踪sample accumulation、AOV、denoiser input/output和跨帧重置合同，并以实际收敛/噪声指标验收。

### P2-07 Stereo、360与Panorama输出

扩展camera view family、tile overlap、seam处理和metadata，保证每eye/face/pass的时间与采样一致。

### P2-08 Render Layer与Custom Pass生态

允许受能力约束的layer/pass provider声明输入、输出、格式、排序和资源预算，unknown pass无损保留且不可误执行。

### P2-09 Editorial Handle与Conform

支持shot handles、版本替换、offline/online conform和source timecode匹配，并生成可审计变更集。

### P2-10 Scripting与插件Track生态

公开受版本、权限和资源预算约束的sequence query/command/job API；远程和脚本入口复用Editor 08安全gateway。

### P2-11 Collaborative Review与Annotation

在Editor 43协同authority稳定后增加frame/shot annotation、review status和presence，annotation不混入evaluation source。

### P2-12 跨引擎可复现实测基准

建立相同镜头、轨数、key数、采样和输出条件下的正确性、编辑延迟、compile/evaluate/render吞吐、内存和artifact size基准；没有数据不得声称优于Unreal。

## 8. 关键合同与迁移策略

### 8.1 最小source model

| 合同 | 必要字段 | 禁止替代 |
|---|---|---|
| `CinematicSequenceSource` | source/schema/revision、tick/display rate、ranges、bindings、master tracks、markers、folders、metadata | `AnimationSequenceAsset`加几个optional字段 |
| `CinematicBinding` | stable binding ID、possessable/spawnable kind、expected type、display/tag、spawn template或resolver spec | `EntityId`或path字符串 |
| `CinematicTrack` | stable track ID、type/schema/provider、binding owner、sections、evaluation policy | UI row或descriptor title |
| `CinematicSection` | stable section ID、range/row/priority、pre/post-roll、completion、typed payload | keys数组本身 |
| `CinematicShot` | child sequence、time transform、camera binding、handles、render metadata | 固定“shot count”文本 |
| `CinematicCompiledArtifact` | source/dependency/provider fingerprint、binding table、hierarchy、evaluation field、schedule、source map | 当前World上的临时property writer cache |
| `CinematicEvaluationInstance` | root context、binding overrides、spawn register、pre-animated state、qualified time、diagnostics | 全局player time字段 |

source与artifact分离是性能前提，不是额外负担。Editor保存可迁移source；cook/background compiler生成紧凑、generation-qualified artifact；frame loop只访问编译后的区间、typed writer和已解析binding。debug provenance可裁剪，但错误、ownership和restore语义不能为性能省略。

### 8.2 Evaluation request与receipt

一次evaluation request至少包含root instance、previous/current qualified frame time、update method、direction、world generation、binding override generation和execution policy。输出不是`()`：它应包含activated/deactivated sections、binding/spawn changes、domain sample counts、events、warnings/errors、timings及restore actions。Preview、PIE、runtime和movie render共享格式，使“看起来播放了”可被替换为可比较证据。

### 8.3 Take publication

```text
preflight sources/clock/path/budget
  -> create staging + session manifest
  -> prepare/arm/countdown
  -> timestamped bounded capture
  -> stop all sources
  -> validate + reduce samples + build TakeAsset
  -> cross-document transaction adds/replaces sections
  -> atomic publish assets + receipt
```

任何步骤失败都不能把部分section写入原sequence。进程崩溃后，recovery只可基于完整journal和checksum继续finalize；无法证明完整性时保留quarantine staging供诊断，不自动发布。

### 8.4 Movie render execution

Queue submit冻结source、map、content、plugin、engine和config fingerprint，并先扩展出shot plan。worker每个output frame依次执行warmup、temporal/spatial samples、GPU readback、pass assembly和writer提交；只有所有预期frame/pass及可选audio完成校验后才发布artifact。重试以shot/frame/pass checkpoint为单位，禁止把已存在文件名当作完成证据。

### 8.5 现有实现处置

| 现有实现 | 决策 | 迁移条件 |
|---|---|---|
| `AnimationSequenceAsset`与compiled property writer | 保留为通用animation和cinematic adapter底座 | stable typed field/binding adapter，不直接扩成Level Sequence |
| Generic Animation Editor | 保留并复用curve/key基础 | stable IDs、typed key insertion、transaction合同先收敛 |
| `timeline_sequence` descriptors | package保持experimental/Unavailable | 资源、factory、codec/compiler/evaluator和tests齐备后重新admit |
| `TimelineEventMarker` | 删除或迁入versioned cinematic event payload | persistence/runtime event semantics同时交付 |
| `move_timeline_keyframe` | 隔离并由domain command替换 | 失败零变更、stable key ID、undo/redo门通过 |
| 静态Sequencer ZUI/feedback | 仅可留作design fixture | production绑定真实provider并删除固定结果 |
| RGBA capture/mailbox | 保留为单帧readback substrate | frame packet metadata、错误回执、bounded render writer pipeline |
| PBR viewer PNG evidence | 保留为开发证据工具 | 不进入Movie Render产品资格声明 |
| `CameraCutOrInvalid` heuristic | 保留为非电影temporal保护 | authored cut由Editor 30 adapter显式输入 |

## 9. 里程碑

| 里程碑 | 交付与退出条件 |
|---|---|
| M0 | 真实性封口：静态Sequencer固定结果移除，Timeline插件fail-close，危险key helper隔离，Cinematic/Take/MRQ入口诚实Unavailable |
| M1 | Source/identity/time：versioned schema、stable IDs、有理数frame time、ranges、migration与canonical roundtrip通过 |
| M2 | Binding/spawn：possessable/spawnable、qualified resolver、override、spawn register、orphan diagnostic通过 |
| M3 | Track/section：registry、section、shot/subsequence、camera/transform/animation/audio/event adapters及time transform通过 |
| M4 | Compiler/evaluator：hierarchy、evaluation field、phase schedule、pre-animated state、completion和deterministic receipt通过 |
| M5 | Sequencer Editor：provider-backed document、Outliner/Timeline/Curve、transaction/save/recovery/diagnostic和preview parity通过 |
| M6 | Camera/audio/event集成：authoritative cut/history、audio sync、event traversal与安全preview矩阵通过 |
| M7 | Take Recorder：source registry、clock/state/buffer、metadata、staging/finalize/recovery与browser通过 |
| M8 | Render Queue core：Queue/Job/Shot/Preset、expansion、fixed-step、sampling、output policy和headless worker通过 |
| M9 | Offline output：AOV/color、bounded readback、Editor36 encoder/muxer、checkpoint/resume、atomic artifact通过 |
| M10 | Cook/robustness/scale：migration、unknown provider、fault injection、large timeline/long take/large render、跨平台确定性通过 |
| M11 | 硬切与资格：删除legacy静态authority和索引helper，默认产品装配、文档、CI、benchmark与release gates闭合 |

依赖顺序为M0 -> M1 -> M2/M3 -> M4 -> M5/M6 -> M7/M8 -> M9 -> M10 -> M11。M1前不得开放source写入；M4前不得声称runtime playback；M7前不得录制真实take；M8前不得显示可提交render job；M9前不得显示artifact成功。

## 10. 产品资格门

1. **G01** production Sequencer所有可见sequence/shot/track/key/status均来自revision-qualified provider，仓内不再存在固定`SEQ_Intro`完成反馈。
2. **G02** timeline插件缺ZUI、factory、compiler、evaluator、bridge或capability任一项时admission为Unavailable，菜单和Create Track不出现。
3. **G03** 任一key/section编辑在validation、save或compile失败时source bytes、revision、dirty、selection和history保持不变。
4. **G04** sequence/binding/track/section/shot/channel/key经过reorder、rename、save/reopen和migration后stable ID逐项相等。
5. **G05** 23.976、24、25、29.97 DF、30、59.94与60 fps长时播放/转换无累计漂移，tick/display rate转换有golden结果。
6. **G06** root/local/global qualified time和三层subsequence offset/scale/trim在Editor、runtime和render逐frame一致。
7. **G07** possessable在重复Level Instance、PIE copy和多World中解析到正确qualified target，不使用当前`EntityId`持久化。
8. **G08** spawnable在play、jump、loop、abort、error和preview teardown后生命周期正确且无泄漏对象。
9. **G09** missing/orphan/ambiguous/type-mismatch binding产生source-located diagnostic，其他有效track按明确policy执行或整体fail。
10. **G10** typed track注册缺任一codec/compiler/evaluator/editor contract时source无损只读，不能创建或执行。
11. **G11** section trim/split/slip/move、row overlap、priority、pre/post-roll和completion有完整golden matrix及undo/redo。
12. **G12** 相同source/dependency/provider fingerprint在Windows/Linux重复compile得到相同artifact digest和evaluation order。
13. **G13** Play、Jump、Scrub、Reverse、Loop与subsequence边界的event派发数量、顺序和安全策略逐项确定。
14. **G14** stop、unbind、cut、sequence switch、runtime error和Editor close均按ownership/completion恢复pre-animated state。
15. **G15** authoritative camera cut准确切换endpoint/lens并递增history epoch；普通大幅运动不会被伪装成authored cut。
16. **G16** audio section在preview、runtime和movie render的start offset、loop、fade与timecode drift满足预算，seek行为可解释。
17. **G17** timeline多选、drag、trim和curve编辑以one transaction提交，cancel、undo、redo和external-change conflict不损坏source。
18. **G18** Add Key读取当前typed property；无target、mixed或unsupported type明确拒绝，不生成`Scalar(0)`占位key。
19. **G19** Preview、PIE与runtime在相同artifact/time/request下输出等价binding、domain sample和event receipt。
20. **G20** 1,000 tracks/100,000 keys/deep folder timeline下滚动、缩放、选择、drag、compile、evaluate和内存满足公开预算。
21. **G21** compile/evaluation/cache/preview所有队列有上限、取消和generation检查，连续source change不安装stale结果。
22. **G22** Take Recorder状态机对重复start/stop/cancel、provider failure和process crash幂等，UI状态只来自controller。
23. **G23** 外部timecode锁定、丢锁、rate变化、drift和drop-frame均生成准确metadata/receipt，不静默改用wall clock。
24. **G24** 长时多source录制在buffer overflow、磁盘不足、设备断开和finalize crash后可恢复或完整回滚，无半Take发布。
25. **G25** 相同Queue输入展开出的job/shot/frame/pass计划digest相同，source在submit后变化不会污染当前run。
26. **G26** headless与Editor worker在固定seed/fixed-step下逐frame camera、event、sample index和输出checksum满足确定性阈值。
27. **G27** warmup、temporal/spatial sample、shutter和authored cut history reset有golden图像/metadata测试，无重复或漏采样。
28. **G28** beauty/alpha/depth/normal/motion/ID/AOV输出的format、stride、color、frame/shot/timecode metadata逐项可验证。
29. **G29** GPU readback、CPU conversion、writer和encoder在慢盘/fault下保持有界内存、backpressure与完整错误回执。
30. **G30** worker crash、cancel、retry和resume只消费校验checkpoint，最终artifact原子发布且无重复、缺失或混代文件。
31. **G31** 真实产品端到端覆盖create sequence、bind、author shot/cut/event、preview、save/reopen/cook/runtime、record take、submit/render/resume/artifact。
32. **G32** 与Unreal及可用Godot/Fyrox/Bevy/Unity Graphics基线以相同数据公开测量correctness、编辑/compile/evaluate/render吞吐、内存和artifact size；只有实测达标才可声称领先。

## 11. 验证说明

本轮是review-only，没有修改production Runtime、Editor、Interface、Plugin、App代码或tests，也没有运行新的动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断；本轮没有重复相同且无法抵达Cinematic产品行为的lane，不能根据115个静态test attributes宣称行为通过。

本报告静态验证要求：131个selected path存在且无重复；fingerprint匹配；P0/P1/P2分别为5/72/12；M0-M11连续；资格门为32；frontmatter、Editor索引、根索引与coverage链接无断链；Markdown为LF、无trailing whitespace、BOM或占位标记。动态实施阶段必须补schema/migration roundtrip、frame-time precision、binding/spawn、hierarchy/evaluation、event/pre-animated restore、transaction fault、preview parity、Take fault/recovery、deterministic render、AOV/color/output、checkpoint/resume、security、跨平台和scale资格。

## 12. 审查决策

1. 保留`AnimationSequenceAsset`、compiled property writer和Animation Editor作为通用底座，由Editor 14继续收敛；不直接把它们宣称为Cinematic Sequencer。
2. 新建独立cinematic source/artifact/evaluation instance，所有电影binding、section、shot和hierarchy从stable identity开始设计。
3. `timeline_sequence`保持experimental且fail-close，直到resource、factory、persistence、compiler、evaluator和Editor transaction垂直链齐备。
4. `event_marker`在sequence持久化与runtime interval semantics交付前不可用；clip event capability只按其真实范围保留。
5. 静态Sequencer与preview design只可作视觉fixture；production route不得输出固定queued、validation或selection成功。
6. Camera Cut消费Editor 30的authoritative camera contract；temporal movement heuristic不升级为电影cut authority。
7. Movie Render消费Editor 22/Runtime capture底座和Editor 36 encoder/muxer，不复制Render Graph、media clock或codec实现。
8. 性能领先通过compiled evaluation field、typed binding、虚拟化timeline、有界pipeline、deterministic worker及公开benchmark取得，不通过省略恢复、诊断、事务或产物完整性取得。
