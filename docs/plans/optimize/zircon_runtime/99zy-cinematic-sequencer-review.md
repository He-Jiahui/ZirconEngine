---
title: Runtime Cinematic Sequencer、Sequence、Shot、Track、Section、Binding、Hierarchy、Evaluation、Camera Cut、Audio、Event、Take Recorder、Movie Render Queue、Network、Save、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime150
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
runtime_child_of: Editor45
related_code:
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/animation/sequence
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_runtime/src/asset/assets/scene/animation.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
  - zircon_plugins/timeline_sequence
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/capture.rs
  - zircon_runtime/src/graphics/runtime/render_framework/viewport_record
  - zircon_runtime/src/core/framework/sound/automation.rs
  - zircon_plugins/sound/runtime/src/timeline
  - zircon_plugins/net/features/replication/runtime/src
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/operation
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
  - docs/plans/optimize/zircon_editor/83-editor-cinematic-sequencer-shot-track-section-binding-hierarchy-evaluation-camera-cut-audio-event-take-recorder-movie-render-queue-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/37-camera-endpoint-director-rig-controller-blend-shake-cinematic-cut-history-multiview-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99za-runtime-camera-endpoint-director-rig-controller-blend-shake-cut-history-multiview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zd-runtime-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99ze-runtime-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zl-runtime-animation-skeleton-clip-pose-graph-state-machine-layer-mask-blend-ik-root-motion-event-extract-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/MovieScene
  - dev/UnrealEngine/Engine/Source/Runtime/LevelSequence
  - dev/UnrealEngine/Engine/Source/Runtime/MovieSceneTracks
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/Takes
  - dev/UnrealEngine/Engine/Plugins/MovieScene/MovieRenderPipeline
  - dev/godot/scene/resources/animation.h
  - dev/godot/scene/animation/animation_mixer.h
  - dev/godot/scene/animation/animation_player.cpp
  - dev/godot/servers/movie_writer/movie_writer.cpp
  - dev/godot/tests/scene/test_animation_player.cpp
  - dev/bevy/crates/bevy_animation/src
  - dev/Fyrox/fyrox-animation/src
  - dev/Fyrox/editor/src/plugins/animation
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/CameraCaptureBridge.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/RenderPass/AOV
---

# Runtime Cinematic Sequencer、Evaluation、Take Recorder 与 Movie Render Queue 当前源码工程化差距

## 1. 结论

当前 Zircon **没有可执行的 Runtime Cinematic 产品**。排除 `dev/`、`docs/`、tooling 与测试后，对生产 Rust/TOML/ZUI 精确检索，`CinematicSequenceSource`、`LevelSequence`、`MovieScene`、`TakeRecorder`、`TakeMetaData`、`MovieRenderQueue`、`MoviePipeline`、possessable、spawnable、pre-animated state 与 cinematic evaluation field 均为 0 命中。存在的 `AnimationSequenceAsset`、`AnimationSequencePlayerComponent`、Sound automation、camera stack、temporal cut heuristic、单帧 capture、operation service、generic replication 与 Dynamic Scene archive 都是真实底座，但没有组成 `source -> compiler -> per-instance evaluator -> domain adapters -> receipt -> network/save/render output` 闭环。

当前通用 Animation Sequence 只有 `duration_seconds`、浮点 `frames_per_second`、`EntityPath`、可选字符串 `target_id` 和 property channel。compiled projection保存 `binding_index/track_index`，应用时重新回读外部 source Vec；cache只以asset ID和revision为键并按当前World解析。`PendingSequenceSample.entity`虽然传入`LoadedSequenceSample`，`apply_loaded_sequences`却从不消费它。因此同一asset的多个player不能拥有不同root context、binding override、spawn register、pre-animated state或实例receipt。compile失败会删除cache后`continue`，apply结果继续被`let _`丢弃。

`AnimationSequencePlayerComponent`只能表示sequence、speed、seconds、looping和playing。它可以随Scene project保存/重开，这是应保留的Partial；但没有Play/Jump/Scrub/Reverse语义、previous/current qualified frame、事件穿越政策、root/subsequence hierarchy、section lifecycle、completion mode或stop/error restore。Runtime time已有real/virtual/fixed clock、clock stamp和fixed-step预算，却没有tick resolution/display rate分离、rational frame time、SMPTE/drop-frame timecode或movie fixed-step authority。

Camera与Sound也没有被错误高估。`CameraSequenceReport`只是Base/Overlay render stack解析；`CameraCutOrInvalid`只是根据运动阈值推断temporal discontinuity，不能表示authoritative authored cut。`SoundTimelineSequence`可确定推进automation并返回sample report，但使用独立seconds clock，没有cinematic section、hierarchy time transform、seek/reverse/event crossing或统一frame receipt。capture mailbox受三槽readback约束，却静默丢弃`Err`并只保留最新ready generation；`CapturedFrame`缺format、stride、color space、PTS/timecode、sequence/shot/frame/sample/tile/pass身份。

产品真相仍未封闭。Sequencer Workbench固定显示`SEQ_Intro`、12 shots、428 keys、Camera Cut、Audio Theme、Event Cues和24 fps；Preview/Validate直接返回固定queued文本。`timeline_sequence`声明的`plugins://timeline_sequence/editor/authoring.zui`物理不存在，五个operation只有descriptor，native dist为`invoke_command: None`且bridge method为0。局部key move helper确已改成mutation前全量校验、有限值拒绝、二分位置和slice rotation，并有失败零变更/equal-time测试；但它仍以三个Vec index寻址，没有document revision、stable key ID、transaction、undo/redo或runtime executor，性能门还是ignored。Event marker只在插件helper/tests存在，validator仍未拒绝非有限marker time/duration。

本报告将历史Editor45混合边界重新拆开：Editor45继续拥有作者document、transaction、Sequencer/Take/MRQ UX与产品投影；**Runtime150拥有可部署的cinematic source codec、compiler/artifact、binding/hierarchy、evaluation instance、playback authority、domain adapter、take capture执行、movie render执行、network/save/replay与runtime qualification**。这不是复制Editor45的5/72/12 finding；本文登记运行时专属 **72项P1，52 Open / 20 Partial / 0 Closed；16项P2全部Open；40项Gate为31 Fail / 9 Partial / 0 Pass**。Editor45五项父P0当前仍为4 Open / 1 Partial，不在本文重复计数。

目标链必须收敛到：

```text
Versioned Cinematic Sources
  -> canonical codec + dependency/provider/target admission
  -> deterministic CinematicCompiler
  -> immutable CinematicProgramArtifact
       stable dense IDs + hierarchy/time transforms + binding plans
       interval evaluation field + phase schedule + source map
  -> per-World CinematicPlaybackService
  -> CinematicEvaluationInstance
       root context + overrides + spawn register + pre-animated store
  -> CinematicFrameRequest(previous/current qualified time + method)
  -> atomic CinematicFrameReceipt
  -> Camera/Transform/Animation/Audio/Event/Gameplay adapters

TakeCaptureRun -> bounded timestamped source buffers -> journal/staging -> Take artifact receipt
MovieRenderRun -> frozen job/shot/frame/sample/tile/pass plan -> bounded output -> checkpoint/artifact receipt
```

本轮只做静态review与计划文档，不修改production、tests、Cargo、ABI或参考源码；没有运行Editor/App、PIE、asset cook、Scene roundtrip、network/save/replay、录制、离线渲染、GPU、fault/scale/soak/profile或同语义竞争benchmark。不能宣称Cinematic可用，更不能宣称性能或表现超过Unreal。

## 2. 审查边界、currentness与证据强度

### 2.1 Currentness

- 审查基线：`main@1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8`，协调baseline epoch 433。
- 读取时共享工作树有3,378个tracked changes、2,163个untracked paths；本文读取当前working bytes，不归因、不覆盖、不回退其他Session改动。
- `timeline_sequence`、Sound timeline与viewport surface含在途修改；本文按当前bytes裁决，实施前必须重取fingerprint。
- 当前MVP仍未完成。本轮属于MVP-00允许的C3 read-only audit，不提前实现高级Cinematic功能。
- 用户明确暂不优化tooling，tooling不在本文source、finding或里程碑范围内。

### 2.2 冻结范围

统计口径：repository-relative path转`/`并小写排序；逐文件取当前bytes SHA-256；聚合输入为`path|file_sha256`以LF连接且末尾无LF。tests统计Rust test、Unreal/Godot test macro与C# test attribute，ignored/disabled单列。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树fingerprint与证据 |
|---|---:|---|
| Zircon sequence、plugin与product truth | **35 / 9,830 / 9,324 / 402,172 / 34 / 2** | source/player/compiler/cache、timeline plugin、Workbench route/feedback；`a88a17cfcd62c02ef40e95d41d97ba900dce4da0ae323e29506b580cf934a83d` |
| Zircon time、camera、sound、capture、net/save/operation基础 | **83 / 11,745 / 10,592 / 399,007 / 93 / 5** | clock、render stack/history、automation、readback、replication、archive与operation；`eb8ef52564fbf25697b729ef6383760fba276abf83c5888452ae33b8005e1fdd` |
| Unreal MovieScene、LevelSequence、Take与MRQ | **29 / 14,465 / 11,779 / 552,342 / 20 / 2** | source/section/player/instance/evaluation/hierarchy/domain adapter及compiler/pre-animated/transform tests；`aa3e9640ef3f421f61712c247be51623ce734fcb60f2efb549a9039ba9071a7b` |
| Godot Animation/MovieWriter与tests | **9 / 9,797 / 8,316 / 365,156 / 9 / 0** | typed track、play/seek/capture/cache、fixed-fps movie/audio与player tests；`b1f54145d486d24633d7986a759c1cfbf2ff9156425c52dca7104549709b6718` |
| Bevy、Fyrox与Unity Graphics | **10 / 6,087 / 5,516 / 236,102 / 10 / 0** | stable target/event/graph、UUID track/signal/command、camera capture与AOV；`6b7b8a4652c91a292bdce61ead2281ec477c1b0f4ef8f0e0f20f53ec530f802c` |

Zircon合计118文件、21,575行、801,179 bytes、127项test declaration、7项ignored；五引擎参考合计48文件、30,349行、1,153,600 bytes、39项test declaration、2项disabled/ignored。选择集是本轮证据边界，不表示未列文件与Cinematic永远无关。

### 2.3 纵向扫描链

本轮按 package/capability -> source/schema/identity -> Scene/project persistence -> compile/cache/currentness -> player scan/time advance -> per-instance binding -> property apply -> camera/audio/event adapter -> operation -> network/save/replay -> capture/readback/output -> App/Editor/product truth -> tests逐层读取。参考侧按Unreal MovieScene/LevelSequence为主线，读取source、section、hierarchy、entity instance、player、camera/audio/event、Take、MRQ和三类测试；Godot、Bevy、Fyrox用于验证较轻运行时合同；Unity Graphics只验证capture/AOV下游边界，不推断其闭源Timeline/Recorder实现。

## 3. 当前可保留的真实基础

1. `AnimationSequenceAsset`有binary codec、V1 fallback、asset revision、property channel与World编译writer，可作为Animation section adapter。
2. compiled property writer避免frame hot path重复解析entity/property文本，并用World binding catalog generation检查currentness。
3. animation projection使用cached ECS query、change tick和staged player update，避免每帧盲扫全部组件，并对owner admission延迟提交player time。
4. Scene project可保存`AnimationSequencePlayerComponent`五个字段，说明普通sequence player有最低持久化底座。
5. Runtime real/virtual/fixed clock、clock-domain stamp、policy transaction、fixed debt与step budget可作为movie clock下层来源。
6. Camera render endpoint、Base/Overlay stack、多target、per-camera temporal数据和view-family基础真实存在。
7. Sound automation有typed binding/target/curve、finite validation、schedule/advance/report及容量优化。
8. Render capture有RGBA8/RGBA16F、generation、target/capture report、GPU三槽readback和单PNG staging/flush/sync/rename。
9. Operation service有handler registry、admission、prepare/apply、deadline/cancel/harvest/retention/shutdown底座。
10. Generic replication、Dynamic Scene、Session archive与resource transaction提供未来adapter可复用的transport、snapshot、journal/atomic publication构件。

这些只能提升对应finding为Partial，不能把通用Animation、Sound、Capture、Replication或Archive改名为Cinematic、Take Recorder或Movie Render Queue。

## 4. 当前源码断路

### 4.1 Source、身份与时间

1. 没有独立Cinematic resource kind、versioned envelope、source ID/revision、provider identity、dependency manifest、target profile或migration graph。
2. Sequence/binding/track/channel/key均依赖Vec位置；binding的`target_id`只是可选String，可解析为当前`EntityId`或`EntityPath`，不是跨World/PIE/level instance稳定binding identity。
3. duration/fps/key使用浮点seconds。没有有理数frame time、tick resolution/display rate、subframe、qualified frame rate、SMPTE/drop-frame timecode或overflow policy。
4. 没有playback/work/view/selection/render/section range的统一开闭语义，也没有shot/subsequence offset、trim、scale、loop与hierarchy bias。
5. Timeline helper已拒绝sequence duration/fps/key非有限值，但event marker validator没有检查marker time和duration的finite/positive，NaN可绕过比较。

### 4.2 Compiler、cache与evaluation instance

1. `CompiledAnimationSequence`只保存duration、World generation、compiled writer和source Vec索引，没有source/dependency/provider/root-context fingerprint、stable dense ID、source map或artifact digest。
2. artifact不是自包含；source binding/track reorder后，如果asset revision/currentness未同步阻断，旧索引可指向不同track。
3. cache只按asset ID保存一份World解析结果。同一source的多个player不能拥有不同binding override或level instance context。
4. `LoadedSequenceSample.entity`完全未消费，player实体不是binding root或instance identity。
5. 没有evaluation field、active interval index、root/subsequence instance tree、phase schedule、spawn ledger、pre-animated store或completion mode。
6. 没有Play/Jump/Scrub/Reverse/Loop update method与previous/current time，离散事件无法定义跨越、反向、循环或seek策略。
7. compile错误被`continue`吞掉，apply返回值被`let _`丢弃；缺失binding只能进入内部stats，没有终端receipt或source-located diagnostic。

### 4.3 Camera、Animation、Audio与Event adapter

1. `CameraSequenceReport`解析render stack，不是电影sequence；Scene source也无法持久化完整stack/director/cut。
2. camera cut依赖temporal velocity阈值猜测。它无法识别同位置硬切，也会把合法高速运动误判为cut，且没有统一history epoch。
3. property channel可写通用scene字段，但没有section overlap/blend/row/priority/pre/post-roll、typed space/compose或失败原子性。
4. Clip/Graph/State Machine运行时是真实Animation域，却没有cinematic animation section adapter、start offset、play rate、root motion、sync与completion receipt。
5. Sound automation使用独立clock与sequence列表，没有shot/hierarchy/time transform、seek/reverse、fade/offset、audio device drift或movie frame receipt。
6. Event marker没有持久化、codec、compiler、runtime dispatch或event ordering；clip event capability不能替代sequence event capability。
7. 没有Gameplay/Script safe event adapter、permission、idempotency或preview/runtime隔离。

### 4.4 Take、Movie Render、Network与Save

1. 没有Take source registry、Idle/Preparing/Armed/Recording/Stopping/Finalizing状态机、timecode lock、bounded per-source buffer、slate/take metadata、staging journal、cancel或crash recovery。
2. 没有Movie Queue/Job/Shot/Preset/Run实体，没有deterministic frame/sample/tile/pass expansion、warmup、shutter、temporal/spatial sample、headless worker或resume。
3. capture packet缺movie identity与像素合同；mailbox只保留最新ready并静默丢错误，不能输出有序帧序列。
4. single PNG原子publication是有价值底座，但没有run manifest、frame/pass checksum、checkpoint、resume、encoder/muxer或whole-run atomic artifact。
5. Scene persistence只保存普通Animation sequence player。SaveGame产品本身仍缺失，没有Cinematic participant、instance state、spawn/restore、pre-animated policy或Take/MRQ run恢复。
6. Net replication代码对AnimationSequencePlayer、SoundTimelineSequence、camera director/cut或cinematic runtime均为0命中；没有server authority、late join、correction、event dedup或content compatibility。
7. 没有runtime product fixture从真实source通过cook、load、play、cut、audio/event、save/reopen、network replay和offline output闭合。

### 4.5 产品真实性与插件边界

1. Workbench的`SEQ_Intro / 12 shots / 428 keys / 1 gap`和queued文本仍是静态第二authority。
2. Workbench使用`workbench.extension.sequencer.*`，插件使用`timeline_sequence.*`，没有controller/provider桥接。
3. 插件物理只有1个Markdown、6个Rust、3个TOML，声明的authoring ZUI不存在。
4. 五个operation没有factory/executor；dist没有command invocation、state或bridge method。
5. key move helper的失败零变更与stable equal-time排序是局部正确性进展，但全量validation加Vec rotation仍是O(sequence + moved span)，release性能门被ignore。

## 5. 参考引擎的可迁移合同

| 参考 | 本轮逐实现/测试确认 | Zircon应吸收 | 边界 |
|---|---|---|---|
| Unreal MovieScene | `MovieScene`分离tick resolution/display rate；binding以GUID拥有track；section有range、row、overlap、pre/post-roll、easing、completion、trim/split；hierarchy保存parent/children/transform/bias | stable source identity、rational time、first-class section、compiled hierarchy | 不复制UObject宏、Slate或历史兼容shim |
| Unreal runtime evaluation | `FSequenceInstance`明确说明同一asset可同时有多个实例，并持有entity ledger；SequencePlayer区分Jump/Scrub/Play、root evaluation、spawn register和network sync；evaluation field按range/entity查询 | per-instance root context、active interval field、phase schedule、typed update method、spawn/restore | 不把Unreal legacy template/entity双路径照搬为双authority |
| Unreal tests | Compiler test覆盖empty space和subsequence ID；Transform test覆盖线性/warp/scale/inverse/zero-timescale；PreAnimated tests覆盖global/entity/overlap/keep/restore/context change | hierarchy/time/restore必须有数学与lifecycle oracle，不以UI smoke代替 | disabled性能test不算性能完成证据 |
| Unreal Camera/Audio/Event | camera cut track instance处理view target/blend/restore；Audio track拥有section/row；Event track定义forward/backward与start/end/after-spawn phase | authored cut/history epoch、audio/event section与明确evaluation phase | Camera stack或clip event不能冒充这些合同 |
| Unreal Take/MRQ | Take Source有Pre/Start/Tick/Stop/Post/Finalize，metadata有slate/take/timestamp/timecode/rate；MRQ分Queue/Job/Shot/config并提供sample/warmup/tile/output/frame policy | typed recording source lifecycle、frozen render run、可恢复输出与metadata | 采用合同，不照搬编辑器对象图 |
| Godot | typed value/method/bezier/audio/animation track；AnimationPlayer区分play/seek/queue/capture/keep state；MovieWriter固定fps并同步audio、检查rate/disk；tests覆盖player行为 | 通用typed track、restore/seek、轻量fixed-fps writer最低线 | Godot不提供完整shot queue，不能降低MRQ目标 |
| Bevy | UUID `AnimationTargetId`、ActiveAnimation elapsed/seek/repeat/speed/weight、target-scoped event及serializable graph，并有inline tests | typed target、event context与Rust-native graph数据布局 | Bevy Animation不是完整Sequencer |
| Fyrox | Track/Signal使用UUID、typed curve/target；Editor以可逆command管理add/remove/replace/move/rebind | stable element identity与command最低线 | 不把普通animation editor当cinematic runtime |
| Unity Graphics | per-camera capture callback；HDRP AOV显式声明depth/motion/world position、allocator、buffer列表与completion callback | typed capture packet、AOV/output ownership与completion | 本地Graphics镜像不含Timeline/Recorder，本文不做无证据推断 |

要超过Unreal，不能先删掉这些语义。领先路径应建立在等价correctness之上：immutable SoA artifact、dense binding slot、interval-indexed evaluation field、per-instance sparse override、zero steady-state allocation、bounded worker/readback、content-addressed run plan与公开可复算benchmark。没有同源、同场景、同输出质量、同错误政策的raw evidence前，禁止宣传性能或表现领先。

## 6. 唯一Owner与硬边界

| Owner | 拥有 | 不拥有 |
|---|---|---|
| Runtime150 | cinematic codec/compiler/artifact、hierarchy/binding、evaluation instance、playback service、runtime adapters、take capture执行、movie render执行、net/save/replay/runtime gates | Sequencer widget、document transaction、codec实现细节之外的媒体产品UX |
| Editor45/83 | authoring document、selection、transaction/history、Sequencer/Take/MRQ UX、job projection与diagnostic navigation | runtime World authority、frame evaluation、GPU/readback/encoder执行 |
| Runtime126 | camera endpoint/director/rig/lens/blend/shake与authoritative cut/history epoch | shot hierarchy与sequence schedule |
| Runtime135 | animation clip/skeleton/pose graph/state machine/IK/root motion/event extract | shot/subsequence与movie binding |
| Runtime137 | sound/device/mixer/bus/effect/spatial/automation/voice | cinematic hierarchy、movie clock与job policy |
| Runtime129/146 | SaveGame/checkpoint与Network transport/session/RPC/replication | cinematic-specific participant/codec语义 |
| Runtime90/Render owners | device/submission/readback/render graph/frame output substrate | queue/shot/frame plan与movie artifact policy |
| Editor36 | encoder、muxer、media artifact authoring | runtime shot scheduling和cinematic evaluation |

硬边界：`AnimationSequenceAsset`只作为animation section adapter，不追加shot/take/movie字段；`SoundTimelineSequence`只作为audio adapter；`CameraSequenceReport`只表示render stack；capture mailbox只作readback substrate；plugin descriptor不等于factory或capability实现。旧路径不保留re-export、compat module或第二状态真相。

## 7. Editor45父P0当前状态

| 父Finding | 状态 | Runtime150当前证据 |
|---|---|---|
| P0-01 静态Sequencer假成功 | Open | 固定`SEQ_Intro / 12 shots / 428 keys / 1 gap`仍在production route/feedback |
| P0-02 缺资源/工厂/桥接不得admit | Open | authoring ZUI不存在，五operation无factory，dist无command/bridge |
| P0-03 key move失败零变更与事务 | Partial | helper失败零变更已修复；stable ID、document transaction、history/undo/receipt仍缺 |
| P0-04 Event Marker能力错配 | Open | marker未持久化/compile/dispatch，finite validation仍不完整 |
| P0-05 禁止包装普通Sequence/capture | Open | 独立source/evaluator/job为0，静态产品面仍存在 |

这五项由Editor45唯一计数。Runtime150的M0必须消费其closure receipt，不能新建重复P0编号。

## 8. P1 Runtime专属重构清单

### 8.1 Source、Schema、Asset与Cook

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-001 | Open | 新建独立`CinematicSequenceSource`、source ID、revision、schema version与provenance；禁止扩写AnimationSequence。 |
| CINE-P1-002 | Open | sequence/binding/track/section/shot/folder/marker/channel/key全部使用持久stable ID，Vec index只作布局。 |
| CINE-P1-003 | Partial | Animation binary已有kind与V1 fallback；建立Cinematic envelope、连续migration、unknown provider/data preservation。 |
| CINE-P1-004 | Open | 编译显式dependency manifest，覆盖subsequence、animation/audio/camera/map/plugin/output preset。 |
| CINE-P1-005 | Open | Track provider必须同时提供codec/schema/compiler/evaluator/editor factory/migration；缺项只读且不可执行。 |
| CINE-P1-006 | Partial | runtime已有product/target policy底座；Cinematic cook需冻结target/profile/capability/provider closure。 |
| CINE-P1-007 | Partial | Animation已有binary roundtrip；补canonical order、NaN/Inf/negative-zero、cross-platform digest与loss receipt。 |
| CINE-P1-008 | Partial | sequence helper已有部分finite/range检查；统一source validator并覆盖全部range/marker/section/ratio。 |
| CINE-P1-009 | Partial | asset revision/resource snapshot存在；补dependency/provider generation、LKG/CAS install与publication receipt。 |

### 8.2 Frame Time、Range、Section与Hierarchy

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-010 | Open | 唯一有理数`FrameRate/FrameTime/SubFrame`库，checked转换、overflow与canonical compare。 |
| CINE-P1-011 | Open | tick resolution与display rate分离，禁止以浮点fps承担storage/evaluation identity。 |
| CINE-P1-012 | Open | playback/work/view/selection/render/section六类range统一开闭、trim与空范围语义。 |
| CINE-P1-013 | Open | qualified time携rate/sequence instance/domain，禁止裸seconds跨边界。 |
| CINE-P1-014 | Open | SMPTE/drop-frame/timecode包含rate、epoch、source、lock与drift disposition。 |
| CINE-P1-015 | Open | section成为first-class source，拥有range/row/priority/overlap/pre-post-roll/completion/easing。 |
| CINE-P1-016 | Open | shot/subsequence保存offset/scale/trim/loop/warp/hierarchy bias与parent/children。 |
| CINE-P1-017 | Open | 编译root-to-local/local-to-root transform，支持inverse失败与zero-timescale测试。 |
| CINE-P1-018 | Partial | 现有BTree/稳定遍历可复用；hierarchy、equal-time、provider/domain排序必须生成canonical schedule。 |

### 8.3 Binding、Instance、Spawn与Restore

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-019 | Open | binding使用stable GUID/typed kind/source location，不再以path/string/index为身份。 |
| CINE-P1-020 | Open | possessable resolver声明World/Level instance/component/subobject/owner scope与ambiguity policy。 |
| CINE-P1-021 | Open | spawnable source、template、outer/ownership/lifecycle/cook与copy语义独立于possessable。 |
| CINE-P1-022 | Partial | EntityPath与compiled property writer可作resolver底座；补qualified context、generation与typed diagnostic。 |
| CINE-P1-023 | Open | 同一artifact的多个evaluation instance拥有互不污染的sparse binding override。 |
| CINE-P1-024 | Open | per-instance spawn register在play/jump/loop/abort/error/teardown后不泄漏对象。 |
| CINE-P1-025 | Open | pre-animated store按object/property/group捕获原值，支持transient/persistent与nested contributor。 |
| CINE-P1-026 | Open | Keep/Restore completion在stop/unbind/cut/switch/error/cancel/close有确定政策与receipt。 |
| CINE-P1-027 | Open | World/source/instance owner lease、replacement epoch和teardown fence统一管理生命周期。 |

### 8.4 Compiler、Evaluation Field与Frame Receipt

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-028 | Partial | asset revision与World writer generation已存在；compile request补source/dependency/provider/root-context generations。 |
| CINE-P1-029 | Open | artifact自包含stable dense IDs、binding plans、hierarchy、time transforms、evaluation field、source map。 |
| CINE-P1-030 | Partial | compiled writer已避免文本热解析；删除外部source Vec index依赖并用dense immutable channel storage。 |
| CINE-P1-031 | Open | evaluation field按time interval只物化active section/entity，支持incremental invalidation。 |
| CINE-P1-032 | Open | 固定phase为spawn/pre/evaluate/blend/apply/event/post，不依赖注册偶然顺序。 |
| CINE-P1-033 | Open | FrameRequest明确Play/Jump/Scrub/Reverse/Loop、previous/current time、direction与discontinuity。 |
| CINE-P1-034 | Open | 一帧domain outputs以staging/commit原子发布，partial failure不留半camera/半property/半event。 |
| CINE-P1-035 | Open | FrameReceipt包含instance/artifact/time/binding/domain/event/restore/diagnostic/timing与terminal disposition。 |
| CINE-P1-036 | Partial | helper/sound有ignored局部性能门；建立release gate、zero steady-state allocation和公开规模预算。 |

### 8.5 Domain Adapter

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-037 | Open | typed Camera Section消费Runtime126 CameraViewResult/director，不直接写裸camera transform。 |
| CINE-P1-038 | Open | authored camera cut发布cut identity/history epoch，并在render/audio/AI一致消费。 |
| CINE-P1-039 | Partial | property channel/writer真实存在；补stable field ID、typed space/compose/blend与readonly/migration receipt。 |
| CINE-P1-040 | Partial | Clip/Graph/State Machine可复用；补section offset/rate/weight/root motion/sync/completion adapter。 |
| CINE-P1-041 | Partial | Sound automation可复用；补qualified clock、offset/fade/seek/reverse/device drift与frame receipt。 |
| CINE-P1-042 | Open | Event section持久化/compile interval，定义forward/backward/jump/scrub/loop与phase策略。 |
| CINE-P1-043 | Open | Gameplay/Script adapter使用allowlisted typed command、authority、idempotency、preview隔离与error receipt。 |
| CINE-P1-044 | Open | Shot/Subsequence adapter管理nested instance、hierarchy transform、bias、preroll与camera/audio boundary。 |
| CINE-P1-045 | Open | adapter failure按required/optional/fallback处理，禁止`continue`或`let _`吞错。 |

### 8.6 Playback、World、Network、Save与Replay

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-046 | Open | 建立per-World `CinematicPlaybackService`，process registry只拥有provider，不拥有World状态。 |
| CINE-P1-047 | Open | root evaluation instance持artifact、playback state、hierarchy updater、binding/spawn/restore与receipt cursor。 |
| CINE-P1-048 | Open | 同一asset多实例、多World、PIE/preview/runtime隔离与independent teardown。 |
| CINE-P1-049 | Partial |普通player已有play/speed/time/looping；补Pause/Stop/PlayTo、serial、range、status和terminal callback。 |
| CINE-P1-050 | Open | Jump/Scrub/Reverse/Loop分别定义事件穿越、pre/post-roll、restore与history invalidation。 |
| CINE-P1-051 | Open | 循环边界、负速率、超大delta、frame skip、pause/resume与clock discontinuity有golden oracle。 |
| CINE-P1-052 | Partial | Scene可保存普通player字段；新增artifact-qualified instance state、override与明确non-persistent spawn/restore。 |
| CINE-P1-053 | Open | network codec复制artifact/content compat、instance/time/status/cut/event cursor，服务端authority可纠正。 |
| CINE-P1-054 | Open | SaveGame/checkpoint/replay participant保存deterministic state、seed/event cursor与restore policy。 |

### 8.7 Take Capture Runtime

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-055 | Open | typed Take Source registry覆盖camera/transform/property/animation/audio/plugin source与factory lifecycle。 |
| CINE-P1-056 | Open | Idle/Preparing/Armed/CountingDown/Recording/Stopping/Finalizing/Completed/Failed/Canceled幂等状态机。 |
| CINE-P1-057 | Open | engine/audio/external timecode保存rate/epoch/drop-frame/drift/lock，丢锁不静默fallback。 |
| CINE-P1-058 | Open | 每source bounded buffer、byte/time budget、backpressure、overflow/drop/late/out-of-order receipt。 |
| CINE-P1-059 | Open | slate/take/operator/source/timecode/frame rate/provenance metadata canonical roundtrip与检索。 |
| CINE-P1-060 | Open | recording journal、staging chunk/checksum与periodic checkpoint，不在内存积累整段Take。 |
| CINE-P1-061 | Open | device/source crash、disk full、cancel和process kill可完整rollback、resume或quarantine。 |
| CINE-P1-062 | Open | finalize做validation/reduction/compression/dependency closure，失败不发布半Take。 |
| CINE-P1-063 | Open | Take artifact与sequence section写入通过cross-document transaction发布并完整undo/redo。 |

### 8.8 Movie Render Runtime与输出

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| CINE-P1-064 | Partial | operation service可作异步执行底座；建立Movie Queue专用typed handler、quota、priority、公平与terminal status。 |
| CINE-P1-065 | Open | Queue/Job/Shot/Preset/Run身份分层，submit冻结source/map/content/plugin/engine/config fingerprint。 |
| CINE-P1-066 | Open | deterministic expansion生成shot/frame/sample/tile/pass plan digest与per-shot override receipt。 |
| CINE-P1-067 | Partial | fixed-step clock/budget已存在；建立movie-owned fixed frame、seed、event/camera一致性和headless worker。 |
| CINE-P1-068 | Open | warmup、temporal/spatial sample、shutter、handle、tile/overlap与authored cut reset有golden oracle。 |
| CINE-P1-069 | Partial | RGBA8/RGBA16F capture真实存在；定义format/stride/color/premultiply/PTS/shot/frame/sample/tile/pass/AOV packet。 |
| CINE-P1-070 | Partial | readback ring有界；改为ordered packet、byte quota、backpressure和exactly-once success/error receipt。 |
| CINE-P1-071 | Partial | single PNG staging/flush/sync/rename可复用；接Editor36 encoder/muxer与whole-run artifact policy。 |
| CINE-P1-072 | Open | run manifest/checkpoint/checksum/cancel/retry/resume/atomic publication通过slow disk/crash/fault矩阵。 |

P1状态复算：Partial为03、06、07、08、09、18、22、28、30、36、39、40、41、49、52、64、67、69、70、71，共20项；其余52项Open，0项Closed。

## 9. P2 领先性与高级产品

| ID | 状态 | 改进项 |
|---|---|---|
| CINE-P2-001 | Open | 多用户实时Sequencer协作、稳定元素CRDT/锁与冲突可视化。 |
| CINE-P2-002 | Open | 分布式render farm、worker capability matching、lease/retry与artifact aggregation。 |
| CINE-P2-003 | Open | OTIO/EDL/FCPXML/ALE interchange、loss report与roundtrip fixture。 |
| CINE-P2-004 | Open | USD stage/camera/animation/timecode交换与layer composition。 |
| CINE-P2-005 | Open | Genlock/PTP/LTC/VITC、虚拟制片stage clock与多机同步。 |
| CINE-P2-006 | Open | LiveLink/mocap多源校准、漂移、重采样、丢包与take provenance。 |
| CINE-P2-007 | Open | 立体、360、XR、多视图shot与per-eye history/output。 |
| CINE-P2-008 | Open | ACES/OCIO、HDR metadata、working/display/output transform与AOV色彩政策。 |
| CINE-P2-009 | Open | GPU direct encode/zero-copy pipeline与显式device/queue/fence ownership。 |
| CINE-P2-010 | Open | procedural editorial rules、shot validation和AI-assisted edit，所有变更可审计/撤销。 |
| CINE-P2-011 | Open | advanced nonlinear time warp、nested retime、audio stretch与deterministic inverse。 |
| CINE-P2-012 | Open | virtual camera、lens calibration/distortion、focus/zoom metadata与physical camera parity。 |
| CINE-P2-013 | Open | render checkpoint自适应分片、跨设备resume与cost-aware scheduling。 |
| CINE-P2-014 | Open | sequence diff/merge、semantic review、approval/signature与release provenance。 |
| CINE-P2-015 | Open | runtime cinematic streaming、large-world shot dependency prefetch与budget feedback。 |
| CINE-P2-016 | Open | 同语义下公开超过Unreal的compile/evaluate/edit/render吞吐、内存、artifact size与画质证据。 |

## 10. 依赖顺序与实施里程碑

| Milestone | 当前状态 | 退出条件 |
|---|---|---|
| M0 产品真实性与owner hard cut | Partial | 关闭Editor45五项父P0；静态结果移除，插件缺resource/factory/runtime时Unavailable；冻结Runtime150/Editor45边界。 |
| M1 Source、identity、frame time | Not started | 独立versioned source、stable IDs、rational time、ranges、canonical migration/roundtrip。 |
| M2 Binding、spawn与restore | Not started | possessable/spawnable、qualified resolver、per-instance override、spawn ledger、pre-animated completion。 |
| M3 Compiler与artifact | Not started | self-contained immutable program、hierarchy/evaluation field/phase schedule/source map与currentness。 |
| M4 Playback与domain adapters | Not started | per-World service、multi-instance、Play/Jump/Scrub/Reverse/Loop、camera/audio/event/animation receipt。 |
| M5 Network、Save与Replay | Not started | typed codec/participant、late join/correction、save/reopen、checkpoint/replay determinism。 |
| M6 Take capture runtime | Not started | source registry、clock/state/buffer/journal/finalize/recovery与artifact publication。 |
| M7 Movie render runtime | Not started | Queue/Job/Shot/Preset、deterministic expansion、fixed-step worker、sampling/tile/AOV packet。 |
| M8 Output与resilience | Not started | ordered bounded readback、encoder/muxer、manifest/checkpoint/resume与atomic artifact。 |
| M9 Scale、fault与performance | Not started | 100K keys/1K tracks、long take、4K/8K render、slow disk/device loss/network correction及zero steady-state allocation。 |
| M10 Product qualification/hard cut | Not started | 删除legacy static/index authority，默认Client/Editor/Server/worker装配、CI和同语义benchmark全部通过。 |

依赖顺序固定为`M0 -> M1 -> M2 -> M3 -> M4 -> M5/M6/M7 -> M8 -> M9 -> M10`。在MVP-00未完成期间只允许继续review、truth封口与测试设计；不能跳到UI扩张、Take或MRQ功能实现。

## 11. Runtime150复验门（40项）

### Authority、Source与Time

- [ ] CINE-G01 `Fail`：production Sequencer不再含固定`SEQ_Intro / 12 shots / 428 keys / 1 gap`authority。
- [ ] CINE-G02 `Fail`：缺resource/factory/codec/compiler/evaluator/bridge/capability时入口Unavailable且无假queued。
- [ ] CINE-G03 `Partial`：Animation binary/V1 fallback可复用；独立Cinematic envelope/migration/canonical digest仍缺。
- [ ] CINE-G04 `Fail`：所有可编辑元素有持久stable ID，reorder/save/reopen/migration后不变。
- [ ] CINE-G05 `Partial`：runtime real/virtual/fixed clock真实；rational movie time、tick/display rate与timecode仍缺。
- [ ] CINE-G06 `Fail`：六类range、section与hierarchy transform通过overflow/warp/inverse golden matrix。
- [ ] CINE-G07 `Partial`：EntityPath/compiled writer可解析普通Scene；qualified binding与多World/PIE隔离仍缺。
- [ ] CINE-G08 `Fail`：possessable/spawnable lifecycle、override与spawn register通过play/jump/abort/teardown。

### Compiler、Evaluation与Domain

- [ ] CINE-G09 `Fail`：artifact自包含source/dependency/provider fingerprint、dense IDs、hierarchy、field与source map。
- [ ] CINE-G10 `Partial`：compiled property writer避免文本热解析；artifact仍依赖source Vec index且无instance context。
- [ ] CINE-G11 `Fail`：同一asset多个evaluation instance拥有独立root/binding/spawn/restore状态。
- [ ] CINE-G12 `Fail`：evaluation field只访问active interval，1K tracks/100K keys满足公开预算。
- [ ] CINE-G13 `Fail`：phase schedule稳定执行spawn/pre/evaluate/blend/apply/event/post。
- [ ] CINE-G14 `Fail`：FrameRequest区分Play/Jump/Scrub/Reverse/Loop并携previous/current qualified time。
- [ ] CINE-G15 `Fail`：pre-animated state在stop/unbind/cut/error/cancel按completion正确restore/keep。
- [ ] CINE-G16 `Fail`：compile/apply错误不被`continue`/`let _`吞掉，terminal receipt可检查。
- [ ] CINE-G17 `Partial`：普通sequence player可保存并推进；电影status/range/method/root instance仍缺。
- [ ] CINE-G18 `Fail`：authoritative camera cut identity/history epoch在render/audio/AI一致消费。
- [ ] CINE-G19 `Partial`：Sound automation有typed advance/report；cinematic clock/section/seek/reverse仍缺。
- [ ] CINE-G20 `Fail`：Event section在forward/backward/jump/scrub/loop及phase位置有golden结果。

### World、Network、Save与Take

- [ ] CINE-G21 `Fail`：per-World playback service支持多World/PIE/preview/runtime隔离和generation teardown。
- [ ] CINE-G22 `Partial`：Scene保存普通player字段；Cinematic artifact/instance/override/spawn/restore participant仍缺。
- [ ] CINE-G23 `Fail`：network复制artifact compat、instance time/status/cut/event cursor并支持late join/correction。
- [ ] CINE-G24 `Partial`：fixed-step与clock stamp可复用；movie/replay确定帧、seed和event cursor仍缺。
- [ ] CINE-G25 `Fail`：save/reopen/checkpoint/replay逐帧输出与未中断运行一致。
- [ ] CINE-G26 `Fail`：Take source registry提供typed factory和完整Pre/Start/Tick/Stop/Post/Finalize lifecycle。
- [ ] CINE-G27 `Fail`：Take状态机幂等，timecode丢锁/设备断开/disk full/cancel不发布半Take。
- [ ] CINE-G28 `Fail`：每source buffer有entries/bytes/age/backpressure/overflow receipt并通过长时录制。
- [ ] CINE-G29 `Fail`：Take journal/staging/checksum可恢复或quarantine，final publication原子且可undo。

### Movie Render、Output与领先性

- [ ] CINE-G30 `Partial`：operation service可承载异步任务；Movie Queue typed handler/job/shot/status尚未建立。
- [ ] CINE-G31 `Fail`：submit冻结全部fingerprint并生成确定shot/frame/sample/tile/pass plan digest。
- [ ] CINE-G32 `Fail`：headless fixed-step worker在pause/retry/resume下保持camera/event/random逐帧一致。
- [ ] CINE-G33 `Fail`：warmup、temporal/spatial sample、shutter、tile、handle与cut reset通过图像golden。
- [ ] CINE-G34 `Fail`：packet声明format/stride/color/premultiply/PTS和完整run/job/shot/frame/sample/tile/pass/AOV身份。
- [ ] CINE-G35 `Fail`：GPU readback/CPU conversion/writer/encoder全链有界，错误exactly-once关联job。
- [ ] CINE-G36 `Fail`：manifest/checkpoint/checksum/cancel/retry/resume和atomic artifact通过kill-point矩阵。
- [ ] CINE-G37 `Fail`：4K/8K、多AOV、长shot、slow disk/device loss满足公开内存、VRAM、queue与吞吐预算。
- [ ] CINE-G38 `Fail`：Editor与runtime消费同一artifact/request并得到等价binding/domain/event/restore receipt。
- [ ] CINE-G39 `Fail`：默认Client/Editor/Server/render worker装配required capability，缺失时启动fail-close。
- [ ] CINE-G40 `Fail`：与Unreal及可用Godot/Fyrox/Bevy/Graphics做同源同画质correctness/吞吐/内存/artifact size竞争；只有raw evidence领先才可宣传超过Unreal。

Gate复算：Partial为G03、G05、G07、G10、G17、G19、G22、G24、G30，共9项；其余31项Fail，0项Pass。

## 12. 首个允许实施的测试设计

MVP-00和Editor45 M0关闭后，第一批不能从扩大Timeline UI开始，应先提交runtime RED oracle：

1. `cinematic_source_canonical_roundtrip`：stable IDs、rational time、unknown provider与cross-platform digest。
2. `cinematic_same_artifact_two_instances_binding_override`：同一artifact在同World两个root绑定不同对象，结果互不污染。
3. `cinematic_three_level_subsequence_time_transform`：offset/scale/trim/loop/warp/bias及inverse失败矩阵。
4. `cinematic_preanimated_keep_restore_matrix`：overlap contributor、nested instance、stop/error/cancel/close。
5. `cinematic_event_traversal_matrix`：forward/backward/jump/scrub/loop/large delta/equal-time phase。
6. `cinematic_camera_cut_history_epoch`：同位置hard cut与高速非cut都得到正确history disposition。
7. `cinematic_save_network_replay_oracle`：save/reopen、late join/correction和replay逐帧receipt一致。
8. `take_capture_kill_point_matrix`：prepare/record/stop/finalize每阶段kill后完整recover/rollback/quarantine。
9. `movie_render_plan_and_resume_oracle`：plan digest、checkpoint、partial frame/pass、slow disk和resume一致。
10. `cinematic_100k_key_release_budget`：compile/evaluate无稳态分配，记录P50/P95/P99、RSS、artifact bytes和active interval visit数。

任何测试不得以静态ZUI文本、descriptor数量、DTO roundtrip、单帧PNG或ignored benchmark作为产品通过证据。

## 13. Review closeout

| 项目 | 状态 | 证据 |
|---|---|---|
| Runtime owner split | review_complete | Runtime150拥有执行链，Editor45/83保留authoring/product projection，未复制父P0计数 |
| Zircon current source | review_complete | 118文件、21,575行；source/cache/entity/time/camera/audio/capture/net/save/operation/product逐层追踪 |
| Unreal primary reference | review_complete | 29文件，含compiler/subsequence/pre-animated tests、runtime instance、Take与MRQ |
| Other four references | review_complete | Godot typed player/writer/tests；Bevy/Fyrox stable runtime/editor基础；Graphics仅capture/AOV边界 |
| Editor45父P0 | 4 Open / 1 Partial | helper局部atomic move为Partial，其余产品真实性/执行链仍Open |
| Runtime150 P1 | 52 Open / 20 Partial / 0 Closed | Partial只表示通用底座可复用 |
| Runtime150 P2 | 16 Open | 无advanced/领先功能达到实现入口 |
| Runtime150 Gates | 31 Fail / 9 Partial / 0 Pass | 无端到端runtime cinematic、take或movie render gate通过 |
| 动态验证 | not_run | review-only；未运行Cargo、Editor/App、PIE、cook、GPU、network/save、take、render、fault/scale/benchmark |

实施前必须重新读取本报告、Editor45/83与最新source，重算五个selected-set fingerprint，并查询相关failure handoff。任何AnimationSequence cache、timeline plugin、World time、camera cut/history、Sound timeline、capture packet、replication、archive或operation变更，都必须至少重跑对应P1、Gate与首批RED oracle，不能依靠报告日期推断currentness。
