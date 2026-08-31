---
title: Editor Animation Timeline、Dope Sheet、Curve Editor、Track-Key Selection、Transport、Scrub、Snap、Clipboard、Transaction、Virtualization 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor196
review_date: 2026-08-28
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/core/editor_authoring_extension.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/retention.rs
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/timeline
  - zircon_editor/src/ui/curve
  - zircon_editor/src/ui/timeline_strip
  - zircon_editor/src/ui/binding/animation
  - zircon_editor/src/ui/binding_dispatch/animation
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution/animation_event.rs
  - zircon_editor/src/ui/host/editor_event_execution/undo_policy.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/animation_sequence.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_timeline_strip
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion
  - zircon_editor/src/ui/template_runtime/runtime/pane_payload_projection.rs
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_timeline_strip.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_plugins/timeline_sequence
tests:
  - zircon_editor/src/core/editing/animation_document/tests.rs
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/ui/timeline/tests.rs
  - zircon_editor/src/ui/curve/tests.rs
  - zircon_editor/src/ui/timeline_strip/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/binding_dispatch/animation.rs
  - zircon_editor/src/tests/host/retained_animation_template_body.rs
  - zircon_editor/src/tests/ui/binding/animation.rs
  - zircon_editor/tests/integration_contracts/workbench_animation_editor_shell.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/74-editor-scene-selection-authority-primary-active-range-filter-named-set-history-document-world-scope-lifecycle-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/190-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Public/MVVM/Selection/Selection.h
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/ToolableTimeline/ToolableTimelineKeySelection.h
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/ToolableTimeline/ToolableTimelineClipboard.h
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/ToolableTimeline/Caches/MultiChannelKeyCache.h
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Public/SequencerTimeSliderController.h
  - dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/SequencerTimeSliderController.cpp
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Public/CurveEditorSelection.h
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Public/CurveEditorSnapMetrics.h
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Public/CurveEditorCommands.h
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Private/DragOperations/CurveEditorDragOperation_MoveKeys.cpp
  - dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Private/Modification/CurveEditorTransactionObject.h
  - dev/godot/editor/animation/animation_track_editor.h
  - dev/godot/editor/animation/animation_track_editor.cpp
  - dev/godot/editor/animation/animation_bezier_editor.h
  - dev/godot/editor/animation/animation_bezier_editor.cpp
  - dev/godot/editor/animation/animation_track_editor_plugins.h
  - dev/Fyrox/editor/src/plugins/animation/selection.rs
  - dev/Fyrox/editor/src/plugins/animation/command/mod.rs
  - dev/Fyrox/editor/src/plugins/animation/toolbar.rs
  - dev/Fyrox/editor/src/plugins/animation/mod.rs
  - dev/Fyrox/editor/src/plugins/animation/track.rs
  - dev/Fyrox/editor/src/plugins/curve_editor.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/animation_curves.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/InspectorCurveEditor.cs
  - dev/Graphics/com.unity.postprocessing/PostProcessing/Editor/Utils/CurveEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/TextureCurve.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Tools/Converters/AnimationClipConverter/AnimationClipUpgrader.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Animation Timeline、Dope Sheet、Curve Editor、Track-Key Selection、Transport、Scrub、Snap、Clipboard、Transaction、Virtualization 与 Product Integration 当前源码复核

## 1. 结论

Editor75之后，Animation Timeline底层发生了四项真实变化。默认asset type registry已经为Sequence/Graph/State Machine注册产品toolkit route；持久动画source迁入`CoreEditContext`中的`AnimationAuthoringDocumentStore`，以`DocumentId + AssetUri + kind + monotonic revision`持有；Sequence的cursor/range/selected span/playback保留在per-view session中，Host的瞬态mutator不再标记document dirty；持久Add/Remove Key、Create/Remove/Rebind Track及Graph/State修改均通过`HistoryContextId::Document`、CAS revision和`AnimationEditCommand`提交，Undo/Redo与dirty/save路径已有局部测试。旧ED75-P1-01可以关闭，旧P1-05和产品入口问题得到实质修正。

这些进展仍未构成可用的Timeline/Dope Sheet/Curve Editor。新`ui::timeline`、`ui::curve`和`AnimationTimelineFoundationView`只是renderer-neutral数据词汇与只读投影；production中没有`TimelineModel`或`CurveModel`实现，没有controller、hit test、capture、drag、keyboard、clipboard、transaction adapter，也没有消费者把foundation接入`animation_timeline_slot`。产品ZUI仍只有header和空slot，pane payload继续压成frame/range/playback字符串与`Vec<String>` track items。`animation.sequence.timeline_edit=true`因此过度声明了当前产品能力。

事件与性能合同仍有结构性错误。全部Sequence event仍以执行时focused view作为地址；Scrub虽为LatestState，coalescing key仍是进程全局`TimelineCursor`，range/selection/playback继续进入DurableReplay。每次持久mutation会clone整份`AnimationAuthoringAsset`，transaction持有整份before/after source，并在document write lock内同步全量compile；每次foundation projection又重新分配全部track/key/string。高频Scrub和无变化Animation event无条件发布Presentation/Reflection effect，无法形成局部、按document隔离的更新路径。

旧`TimelineStrip`的single-flight和tick预算有改进：`OnceLock`防止同key并发重复build，tick数按plot width限制并保留4096硬上限，label frame会按相邻tick区域裁剪。但cache仍以未经内容校验的64-bit FNV digest为身份，静态tick cache错误依赖track label和全部key time/label，所有surface共享容量16的全局mutex；paint仍遍历全部key并把每个diamond拆成多条1像素quad，没有visible range index、row virtualization、density LOD或instance batch。

本轮不新增P0。Editor75的13项P1刷新为 **5 Open / 7 Partial / 1 Closed**，并新增2项独立P1，合计15项为 **7 Open / 7 Partial / 1 Closed**；6项P2为 **4 Open / 2 Partial / 0 Closed**。48门为 **27 Fail / 19 Partial / 2 Pass**。两个Pass只证明持久source与per-view瞬态状态已物理分离，以及scrub/range/selection/playback的Host路径不再登记dirty；它们不代表真实Timeline产品或性能资格完成。

本轮只做静态review，不修改production Rust/ZUI，不运行Cargo、Editor、GUI/GPU、native input、save/reopen、plugin reload、fault/soak、100K/1M key profile或同语义跨引擎benchmark。Tooling按用户要求排除；没有查询、轮询、等待或实时跟踪协调器。

## 2. 审查边界与冻结语料

### 2.1 Current working tree

主仓HEAD为`681588f7a1cbfaae3147e8b93e1be6705d810f21`。报告读取2026-08-28 current working tree；animation document、session foundation、timeline/curve和timeline plugin含共享工作树中的在途修改。本报告保存当前磁盘事实，不回退、不归属也不提交这些production改动。

MVP baseline recovery仍为`in_progress`。本报告是静态差距账本，不把未执行的单元测试或源码中的capability声明当成动态产品资格；实施前必须复算全部fingerprint并读取Editor14/45/55/63/69/75及Runtime animation owner的最新终态。

### 2.2 冻结物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Animation document/session/event | **47 / 8,378 / 7,807 / 295,768 / 18** | source authority、revision/CAS、transaction、retention、route、dirty/save与trace | `4b849275bb5e98051b75625c32fa4de50076a96a2ea4f13d59171ae01dac1462` |
| Timeline/curve/product/plugin | **36 / 5,212 / 4,785 / 194,961 / 5** | typed foundation、legacy strip/cache/paint、pane/slot、extension descriptors与plugin负证据 | `47fc97caea04d5f1770ece794a8b069292ec6cfe83834b586987be61a0841189` |
| Focused tests | **32 / 6,663 / 6,213 / 242,352 / 122** | document history/LKG、session projection、event route、pane slot、cache/paint与plugin helper | `2c0385f5ffa5c64b218b8394ce352a2876ce14ce33898af4571dcbfa4a4f6a56` |
| Zircon去重合计 | **115 / 20,253 / 18,805 / 733,081 / 145** | 上述三组按normalized path去重 | `cb22b22b1e105d2502584b0988d2d4e28e7c4f205caec805e1864f43e7496ee3` |
| Unreal selected set | **11 / 5,634 / 4,692 / 204,748 / 0** | Sequencer selection/time slider/cache/clipboard与Curve Editor snap/drag/transaction | `0af1f872e9508d0d424a5d69a7b26796a04840ac028a9ec3037107398f0eb936` |
| Godot selected set | **5 / 14,193 / 11,922 / 524,896 / 0** | track/key/marker editor、Bezier editor与typed track plugin | `cd5354dc40d229bd8e74e905504030410bdaa5e07574ad3d71fbc4729168226a` |
| Fyrox selected set | **6 / 5,593 / 5,159 / 222,209 / 2** | typed animation selection、reversible command、toolbar/track与curve editor | `40f3badbff86a495b1e523b25242923dd9cc41419e6725f2837d368446610a10` |
| Bevy selected set | **4 / 3,825 / 3,476 / 144,942 / 11** | typed runtime curves/evaluator、graph、transition与player scheduling | `d3cabb357ddf0bed113be3426d94e7ffed1749d76601e34f25a82c4df0d948ed` |
| Unity Graphics selected set | **4 / 2,677 / 2,275 / 107,582 / 0** | embedded curve selection/picking/serialization、TextureCurve与clip migration | `f0aa6f2d9242c80d88f5c256393ba1c93b8c2089f327f32e0c6558d22d971857` |
| 五引擎参考合计 | **30 / 31,922 / 27,524 / 1,204,377 / 13** | 五组显式路径去重 | `08506c8fe7e523e484e469b1720abae069eb722039d94227b2df566e190f813c` |

fingerprint按小写规范化相对路径排序，将每个`path + NUL + lowercase file SHA-256 + LF`聚合后再取SHA-256。它是本轮审查输入receipt，不是asset content key、compiler key或render cache key。

### 2.3 Owner边界

Editor196只刷新Editor75拥有的timeline view-state分类、qualified event routing/retention、typed projection、timeline/curve interaction adapter、strip cache/paint复杂度和聚焦资格门。Editor14拥有动画source/schema/compiler/preview总账；Editor45拥有Cinematic Sequencer及`timeline_sequence`插件的P0；Editor55拥有系统clipboard；Editor63/184拥有通用transaction/history/savepoint；Editor69/190拥有preview clock/session；Editor74拥有Scene Selection原则；Runtime animation拥有实际evaluator。这里引用这些父账，不重复新增同一finding。

## 3. 当前实现拓扑

```text
Asset Browser / OpenAsset
  -> builtin animation toolkit route
  -> AnimationAuthoringDocumentStore(DocumentId, AssetUri, revision, source, compilation)
  -> AnimationEditorSession(read handle + per-view Sequence transient state)

Durable event
  -> focused Sequence or locator-resolved Graph/State view
  -> clone whole source + apply mutation
  -> AnimationEditCommand whole-source CAS swap
  -> HistoryContextId::Document
  -> synchronous full compile under document write lock
  -> metadata dirty + pane string projection

Transient Sequence event
  -> execution-time focused view
  -> mutate cursor/range/span/playback in session
  -> no document dirty
  -> unconditional PresentationChanged + ReflectionChanged

Foundation
  -> rebuild Vec<TimelineTrackView<String>> and Vec<CurveView<String>>
  -> stable-looking IDs derived from track path + f32 time bits
  x no production consumer / TimelineModel / CurveModel / controller

Product
  -> AnimationSequenceV1(strings + frame scalars)
  -> animation_sequence_body.zui header + empty timeline slot anchor
  x typed foundation is not mounted

Legacy strip demo
  -> ZUI attributes -> flat one-lane TimelineStripGeneration
  -> global digest-keyed tick cache
  -> eager tick/text/key quad commands
```

### 3.1 可保留底座

1. 默认Animation Sequence/Graph/State Machine toolkit route与route-kind校验已经进入builtin registry。
2. 持久动画source只有`AnimationAuthoringDocument`可写，session只持read handle和瞬态Sequence状态。
3. source revision使用checked monotonic increment，command apply/revert用expected revision fail-close。
4. durable animation mutation进入document history，Undo/Redo、dirty和canonical save已接通。
5. invalid current source可保留last-known-good compilation，current与LKG revision分开。
6. transient scrub/range/selection/playback不修改source bytes，也不登记document dirty。
7. shared timeline已有range、track/key/section、selection、lane、ruler、snap和overlap词汇；curve已有key/tangent selection和坐标变换。
8. TimelineStrip cache已用`OnceLock`实现同key single-flight，tick生成受plot width与4096硬上限约束。
9. paint按共享palette/geometry模块拆分，并有离屏像素测试；这些可作为后续batch renderer的输入。

### 3.2 不能误判为完成

1. `AnimationTimelineFoundationView`与`AnimationCurveFoundationView`只有session tests消费，production调用为0。
2. `TimelineModel`和`CurveModel`没有实现者；trait存在不代表有authoring controller。
3. `TimelineSelection`和`CurveSelection`是独立容器，没有接Animation session、event、history或产品UI。
4. `TimelineSnapSettings`只是对传入boundary线性扫描的纯函数，没有candidate provider、pixel threshold、receipt或产品设置。
5. slot anchor只证明布局预留，不证明timeline控件、input、accessibility或rendering挂载。
6. `animation.sequence.timeline_edit=true`只反映少量event mutator，不反映产品可编辑性。
7. compilation的LKG没有接preview runtime或pane currentness，不能证明用户看到的是当前source。
8. cache的single-flight只解决同digest重复build，不解决digest collision、全局锁、错误依赖或跨surface预算。

## 4. 父报告校正

| 既有结论 | 当前源码复核 | 当前裁决 |
|---|---|---|
| Editor14 P0-1默认动画入口不可达 | builtin registry现有三类animation toolkit route，真实OpenAsset测试不再需要测试注入 | source层已修正；动态bootstrap未跑，本报告Gate 1为Partial |
| Editor14 P0-2固定Scrub/Add Node与空slot | 固定header假动作已删除；slot仍为空，capability又把timeline edit标为true | 父P0仍未关闭，虚假动作范围缩小 |
| Editor14 P0-3动画mutation绕过transaction | durable mutation已走document transaction/CAS/Undo，transient不再dirty | 原核心断点已修正；event分类/寻址/trace仍由ED75-P1-02至05登记 |
| Editor14 P0-4无compiler/LKG | document每revision同步compile并保留LKG，但capability明确semantic compiler unavailable | Partial底座，不等于完整semantic compiler/product diagnostic |
| Editor14 P0-5 preview不执行runtime | capability明确`animation.preview.runtime=false`，timeline transport仍只改session字段 | 继续Open |
| Editor45 timeline plugin | descriptor和operation名仍无真实resource/factory/executor，声明ZUI物理缺失；`move_timeline_keyframe`已改为mutation前全量preflight，旧“返回Err但source已变化”结论不再成立，但仍使用Vec index且绕过Editor transaction | resource/admission/event-marker P0继续由Editor45拥有；失败原子性进展应在其下次刷新校正 |
| Editor55 clipboard | timeline/curve没有typed clipboard adapter | 继续由Editor55拥有系统envelope，本报告只要求Animation payload adapter |
| Editor63 transaction | document history底座已被Animation复用 | 不创建私有history；drag merge/cancel/receipt仍待Timeline adapter |

## 5. 参考引擎差异与适用边界

| 参考 | 本地源码可验证事实 | Zircon应采用 | 证据限制 |
|---|---|---|---|
| Unreal | Sequencer selection分层管理outliner/track-area/keys；Toolable Timeline以channel identifier匹配clipboard和cache；Curve move drag有begin/update/finish/cancel、snapshot、scoped transaction与frame-rate snap；time slider使用frame-time/rate | stable element/channel address、qualified time、single transaction gesture、clipboard destination mapping、cache descriptor equality | 不复制UObject/Slate或全量Sequencer电影域 |
| Godot | Animation Track Editor覆盖多轨/多key/marker、typed plugin、Bezier handle、selection、clipboard、snap与UndoRedo | typed lane provider、multi-selection、curve handles、产品输入和Undo闭环 | Node/Ref单例结构不是Zircon多document终态 |
| Fyrox | animation selection独立typed domain，Add/Remove Track和selection通过可逆Command；curve resource修改用swap command | domain-owned reversible delta、selection与asset command一致、preview toolbar接真实状态 | clone/swap只证明语义，不证明百万key性能 |
| Bevy | runtime AnimationCurve/Evaluator按typed property/evaluator ID采样，graph/transition与player调度分层 | Editor preview必须消费同一typed runtime evaluator与property identity | Bevy选取范围不是完整Editor UX基准 |
| Unity Graphics | Inspector Curve Editor保留selection/picking/serialization；TextureCurve管理runtime texture bake，upgrader显式迁移clip | embedded curve adapter、selection/picking、migration/bake currentness | `dev/Graphics`不含Unity核心Animation/Timeline产品，不外推其完整能力 |

共同工程合同是：稳定身份与有理时间先于绘制，手势有明确生命周期和单一transaction，selection/clipboard/snap/curve由typed domain adapter连接，preview运行真实evaluator，projection/cache以完整revision和viewport currentness校验，规模路径按visible page工作。Zircon目前只覆盖其中的document transaction和若干只读词汇。

## 6. 差异矩阵

| 能力 | 当前Zircon | 工程级目标 | 当前判定 |
|---|---|---|---|
| Product entry | builtin route可打开document | 默认入口打开可操作timeline并投影真实source/currentness | Partial |
| Source/view split | core source + per-session transient | qualified document/view/session authority与schema persistence | Pass |
| Event target | Sequence读取执行时focus | event入队时冻结document/view/session/epoch/revision | Fail |
| Retention | Scrub Latest，其余view state Durable | asset/view/gesture/transport分型且per-session coalesce | Partial |
| Time | u32 frame与f32 seconds/fps | rational tick resolution/display rate/subframe/timecode | Fail |
| Identity | track path；key ID为path+time bits | persistent track/channel/key/tangent/section/marker IDs | Fail |
| Projection | typed只读Vec + production字符串payload | revision-qualified immutable paged projection | Partial |
| Selection | span tuple及未接线generic sets | typed multi-element authority、range/box与observer receipt | Fail |
| Editing | Add/Remove key/track/rebind event | move/value/tangent/interpolation/duplicate/delete/retime完整controller | Partial |
| Transaction | whole-source CAS command | delta/page级command、gesture merge/cancel与bounded history | Partial |
| Curve | scalar/vector只读projection | canvas/hit/handle/edit/extrapolation/bake/reduce产品闭环 | Partial |
| Preview | playback标签，无runtime evaluator | isolated runtime session、seek/evaluate/event/currentness receipt | Fail |
| Rendering | one-lane eager quads | hierarchy virtualization、visible culling、LOD、batch | Fail |
| Cache | global 16-entry digest key + mutex | collision-safe per-surface descriptor cache与telemetry | Partial |
| Performance | whole asset clone/compile/project | bounded delta compile、incremental projection与100K/1M receipts | Fail |

## 7. Findings

### 7.1 P0

本轮不新增P0。空产品slot与capability truth仍由Editor14 P0-2拥有；timeline plugin无resource/factory/executor和event-marker能力错配仍由Editor45拥有。插件key move已经先preflight再mutation，旧失败原子性问题不再成立；剩余Vec index身份和绕过Editor transaction按其父owner刷新。Editor196新增的是这些父问题下面的timeline state/routing/cache/performance精确合同。

### 7.2 P1

| ID | 状态 | 当前源码证据与需要重构的内容 |
|---|---|---|
| ED75-P1-01 | Closed | `AnimationAuthoringDocument`持久source与`AnimationSequenceSessionState`瞬态状态已物理分离；Host的`with_animation_transient_session_mut`不登记dirty。保留该边界并补Host级positive/negative回归。 |
| ED75-P1-02 | Open | 只有Scrub是LatestState；SetTimelineRange、SelectTimelineSpan、SetPlayback仍落入DurableReplay。建立`DurableAssetEdit / ViewLatest / Gesture / TransportIntent`typed retention。 |
| ED75-P1-03 | Open | `EditorEventLatestStateKey::TimelineCursor`仍是进程全局单值；多document scrub可互相淘汰。key必须包含Document/View/SessionEpoch并返回supersession receipt。 |
| ED75-P1-04 | Open | 全部Sequence event无locator，执行时调用`focused_animation_sequence_instance()`；late delivery/replay会命中另一document。冻结qualified target和expected revision，closed/stale fail-close。 |
| ED75-P1-05 | Partial | durable Animation已有document transaction和event transaction ID测试，transient本身无transaction；但`undo_policy_for_event`仍 blanket delegated，trace又从focused history采样，locator定向非focused Graph时可能记录错误transaction。改为execution receipt直接携transaction identity。 |
| ED75-P1-06 | Partial | 新typed timeline/curve foundation是正确方向，但production payload仍为字符串，slot无consumer，legacy strip仍读ZUI attributes。硬切为唯一`AnimationTimelineProjection`并删除双authority。 |
| ED75-P1-07 | Open | `static_generation`是64-bit FNV digest，cache命中不比较完整descriptor；碰撞可返回另一timeline ticks。使用结构化key或digest+equality validation。 |
| ED75-P1-08 | Partial | `OnceLock`已实现同key single-flight且build在mutex外；所有surface仍争用进程全局mutex，pending miss可暂时突破容量，capacity/eviction无surface或budget隔离。迁移为per-surface sharded cache。 |
| ED75-P1-09 | Partial | tick按plot width限制并有4096 cap，shared ruler能生成nice step，label frame会裁剪；两套tick算法未统一，4096 text/line command仍可一次生成，也没有major/minor层级和测量后稀疏化。 |
| ED75-P1-10 | Partial | TimelineStrip用Arc保存keys，但构造仍clone/filter全部keys并全量hash；foundation每次重新分配所有track/key/ID/label，Curve projection也复制selected track全部component keys。改为revisioned page/COW projection和增量diff。 |
| ED75-P1-11 | Open | paint遍历全部key，每个diamond生成多条scanline quad；无visible-time culling、row virtualization、density LOD、GPU/host instance batch或command budget。 |
| ED75-P1-12 | Partial | durable Add/Remove/Create/Rebind mutation与generic selection/snap模型存在，但无Timeline/Curve hit test、capture、drag、keyboard、clipboard/controller；`timeline_edit=true`应降级到精确子能力。 |
| ED75-P1-13 | Partial | 新增document history/LKG、foundation、single-flight、tick budget、paint像素测试；仍缺Host transient no-dirty、focus switch/late event、per-document coalescing、collision、native gesture、cancel、Undo selection、scale/fault/soak和真实产品E2E。 |
| ED75-P1-14 | Open | 新document path每次`prepare_mutation`clone整份asset，command保存整份replacement，apply/revert whole-source swap后在write lock内同步全量compile。百万key下编辑延迟、history内存和锁占用均为O(asset)。改为stable-ID delta/page COW、staged incremental compile和异步generation install。 |
| ED75-P1-15 | Open | `execute_animation_event`无论changed/no-op/transient均返回PresentationChanged与ReflectionChanged；高频scrub会触发全Workbench projection/reflection。建立per-view dirty channel、effect diff和frame-coalesced local publication。 |

### 7.3 P2

| ID | 状态 | 当前源码证据与需要重构的内容 |
|---|---|---|
| ED75-P2-01 | Partial | AddKey会排序并拒绝EPSILON近似重复，但用last key value而非插入时采样值；空track默认为Scalar，RemoveKey可删除多个近邻key，rebind冲突静默no-op。定义channel schema、collision/merge和duration extension policy。 |
| ED75-P2-02 | Open | key label只参与generation hash；paint/hit/a11y均不消费label。stable label/tooltip/accessible name应由typed projection提供。 |
| ED75-P2-03 | Open | selected key只增加1像素radius且仍用同一key palette；无primary/hover/disabled/stale/error/curve handle区别。建立状态token和高对比/色弱矩阵。 |
| ED75-P2-04 | Open | frame输入为u32，source仍是f32 seconds/fps，key ID依赖f32 bits；没有rational display rate、tick resolution、subframe、timecode或舍入receipt。 |
| ED75-P2-05 | Open | invalid duration/fps/tick/speed多处静默fallback或no-op；pane没有typed diagnostic说明coercion。统一validator并在source/projection/command返回稳定诊断。 |
| ED75-P2-06 | Partial | global static cache已有16项硬上限和entry-count测试，但容量不可配置、无hit/miss/contention/eviction/bytes telemetry；唯一性能测试仍`#[ignore]`且不覆盖多surface。 |

## 8. 目标架构

```text
AnimationAuthoringDocument
  -> stable-ID delta transaction
  -> structural/semantic incremental compiler
  -> immutable AnimationCompiledArtifact(source/dependency/provider revisions)

AnimationTimelineSessionKey
  (DocumentId, ViewInstanceId, SessionEpoch)
  -> AnimationTimelineViewState
  -> qualified event retention/coalescing
  -> AnimationTimelineProjection pages
     (rows, visible time pages, keys, curves, diagnostics, currentness)

Timeline/Curve controller
  -> hit/capture/begin/update/commit/cancel
  -> snap candidates + receipt
  -> Editor63 transaction adapter
  -> Editor55 clipboard adapter

Preview session
  -> Editor69 clock
  -> runtime compiler/evaluator in isolated world
  -> evaluated revision/time/event/pose receipt

Retained renderer
  -> row virtualization + visible range index + density LOD
  -> batched key/tick/tangent/marker instances
  -> per-surface collision-safe cache + budgets/telemetry
```

关键约束：source只有document authority可写；view state不进asset dirty；event target在入队时冻结；stable ID不由time/index/display string派生；projection只读且revision-qualified；preview不得另写简化evaluator；gesture只产生一个可撤销transaction；render/cache只处理visible page并有明确过载结果。

## 9. 重构里程碑

| 里程碑 | 交付内容 | 前置 |
|---|---|---|
| ED75-M0 | RED guards：empty slot、focus late delivery、global scrub key、durable view-state retention、whole-source clone/compile、global effects、cache collision与eager paint | 无 |
| ED75-M1 | `AnimationTimelineSessionKey`、qualified event target、typed retention与supersession receipt；transient Host no-dirty E2E | M0 |
| ED75-M2 | persistent track/channel/key/tangent/marker/section IDs与rational time/tick/display-rate migration | M1 + Editor14 schema |
| ED75-M3 | stable-ID delta/COW document command、bounded history、incremental compiler与async generation install | M2 + Editor63 |
| ED75-M4 | 唯一immutable paged Timeline/Curve projection，接入真实Sequence slot并删除字符串/attribute双authority | M2-M3 |
| ED75-M5 | Timeline/Curve controller：hit、capture、multi-selection、drag、snap、keyboard、clipboard、begin/update/commit/cancel | M4 + Editor55/63 |
| ED75-M6 | isolated runtime preview、seek/play/pause/step/reverse/loop/event policy和current/LKG/stale feedback | M3-M5 + Editor69/Runtime |
| ED75-M7 | row/time virtualization、visible index、density LOD、batched instances、per-surface cache与硬预算/telemetry | M4 |
| ED75-M8 | persistence/a11y/diagnostic/fault：workspace schema、focus/screen reader/high-DPI、capture loss、plugin revoke、save/reopen | M5-M7 |
| ED75-M9 | 100K/1M keys、10K tracks、多document/surface soak与同语义跨引擎功能/latency/memory/command receipt | M8 |

不得把M4空slot接线提前于M1-M3身份、时间与delta合同；否则只会把当前字符串prototype替换成另一套不可迁移的数据authority。不得以旧flat `TimelineStrip`逐项追加交互来替代shared timeline/curve controller。

## 10. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED75-G01 | production可从默认资产入口打开真实Animation Sequence toolkit | Partial |
| ED75-G02 | 产品不再显示空slot、固定数据或不可执行控制 | Fail |
| ED75-G03 | durable asset state与per-view timeline state物理分离 | Pass |
| ED75-G04 | scrub/range/selection/playback变化不dirty、不autosave资产 | Pass |
| ED75-G05 | durable key/track edit必然dirty并产生savepoint关联 | Partial |
| ED75-G06 | track/channel/key/marker/section/tangent均有stable typed ID | Fail |
| ED75-G07 | 每个command/event携qualified document/view/session target | Fail |
| ED75-G08 | retention class按asset edit/view state/gesture/transport明确区分 | Partial |
| ED75-G09 | latest-state coalescing按document/session隔离且有supersession receipt | Fail |
| ED75-G10 | time domain使用rational display rate/tick resolution/subframe | Fail |
| ED75-G11 | view/work/playback/selection ranges语义独立且可迁移 | Partial |
| ED75-G12 | Timeline消费typed immutable projection而非字符串/attributes双authority | Partial |
| ED75-G13 | Sequence product slot显示真实hierarchy、channels、keys与status | Fail |
| ED75-G14 | projection携source/schema/compile/preview revisions与currentness | Partial |
| ED75-G15 | row hierarchy支持binding/track/channel/lane及稳定expansion | Fail |
| ED75-G16 | track/channel/key/tangent/marker/section支持typed multi-selection | Fail |
| ED75-G17 | key/handle/marker hit test返回stable address与距离/priority receipt | Fail |
| ED75-G18 | box/range selection在pan/zoom/virtualization下语义稳定 | Fail |
| ED75-G19 | move/duplicate/delete/value edit使用同一qualified mutation path | Partial |
| ED75-G20 | curve tangent/interpolation/extrapolation/bake/reduce具有typed命令 | Fail |
| ED75-G21 | snap组合grid/key/marker/section/audio并返回candidate receipt | Partial |
| ED75-G22 | clipboard保留source time/channel/key/tangent和destination identity | Fail |
| ED75-G23 | drag/edit具有begin/update/commit/cancel及single transaction | Fail |
| ED75-G24 | undo/redo恢复精确key IDs、times、values、tangents与selection | Partial |
| ED75-G25 | capture loss/Escape/close/cancel精确恢复before state | Fail |
| ED75-G26 | preview使用共享compiler和Runtime evaluator/isolated world | Fail |
| ED75-G27 | scrub/step会实际evaluate并回传evaluated revision/time | Fail |
| ED75-G28 | play/pause/reverse/speed/loop/range crossing/event policy完整 | Fail |
| ED75-G29 | Current/Compiling/LastGood/Stale/Failed状态真实且generation-qualified | Partial |
| ED75-G30 | marker/notify/event/audio lanes有authoring与runtime delivery闭环 | Fail |
| ED75-G31 | tick采用adaptive major/minor spacing且label不重叠 | Partial |
| ED75-G32 | frame/subframe/timecode/seconds格式与display rate一致 | Fail |
| ED75-G33 | rows按viewport虚拟化且scroll不会materialize全树 | Fail |
| ED75-G34 | keys按visible time range/page cull | Partial |
| ED75-G35 | zoom-out使用density LOD并保留selection/currentness语义 | Fail |
| ED75-G36 | key/tick/tangent/marker绘制批处理且command count有界 | Fail |
| ED75-G37 | cache key collision-safe或命中后验证完整descriptor | Fail |
| ED75-G38 | 无进程全局timeline cache锁瓶颈与并发重复build | Partial |
| ED75-G39 | memory/key pages/render commands/labels有硬预算与overload结果 | Partial |
| ED75-G40 | 多document并发scrub/edit/cache/preview完全隔离 | Fail |
| ED75-G41 | keyboard/focus/screen reader/high-DPI/contrast产品矩阵通过 | Fail |
| ED75-G42 | disabled/stale/error/selection/preview反馈真实且可操作 | Partial |
| ED75-G43 | production bootstrap和默认asset open E2E通过 | Partial |
| ED75-G44 | transient dirty、transaction、undo、retention/replay regression通过 | Partial |
| ED75-G45 | compile/preview currentness、close/reopen、fault injection矩阵通过 | Partial |
| ED75-G46 | 100K/1M keys、10K tracks、多surface profile与soak通过 | Fail |
| ED75-G47 | 真实GUI/native input/capture/render golden与a11y自动化通过 | Partial |
| ED75-G48 | 同语义跨引擎功能、性能、内存和command-count receipt可复现 | Fail |

## 11. 测试与动态证据矩阵

| 层级 | 当前静态证据 | 缺失的最低资格 |
|---|---|---|
| Document/history | CAS source swap、Undo/Redo、dirty、LKG、no-op mutation | delta memory bound、stale concurrent edit、compile supersession、rollback/fault、savepoint全链 |
| Session | transient source-byte隔离、timeline/curve projection、invalid speed no-op | Host scrub/range/playback不dirty/autosave、multi-view lifecycle、workspace persistence |
| Event/retention | durable transaction ID、typed target errors、Scrub LatestState | Sequence qualified target、focus switch、per-doc coalesce、view-state retention、direct execution receipt |
| Product | builtin route、pane payload、slot anchor | typed foundation mount、真实keys/tracks/curves、native input、no-fixture default bootstrap |
| Timeline/curve | range/ruler/snap/selection/curve transform纯函数 | domain model implementation、controller、hit/capture/drag/cancel/clipboard/a11y |
| Cache/render | single-flight、capacity、tick bound、label geometry、离屏像素 | digest collision、multi-surface contention、visible culling、batch、command/bytes budget、GPU/native golden |
| Performance | 一项`#[ignore]`的cache hit计时 | 100K/1M key edit/project/render、10K rows、history/compile memory、scrub frame budget与soak |
| Preview | document current/LKG compile state | Editor-to-runtime install/evaluate、pose/event/currentness、seek/loop/reverse与fault |
| Plugin | descriptor/helper/tests | physical ZUI、factory/executor、transaction/persistence/runtime event闭环；由Editor45验收 |
| Cross-engine | 本轮逐文件静态对照 | 相同fixture/time range/zoom/selection/硬件下的可复现功能、latency、memory、command receipt |

本轮没有执行Cargo或动态产品矩阵。仓内test declaration和源码中的`#[ignore]`性能用例只说明测试意图；未执行不能表述为通过。

## 12. Owner路由与禁止重复实现

| 合同 | 唯一owner/消费者关系 |
|---|---|
| Animation source/schema/compiler/runtime preview总账 | Editor14 + Runtime animation；Editor75只接Timeline projection/interaction/transport adapter |
| Cinematic Sequencer与timeline plugin P0 | Editor45/Runtime Cinematic；普通Animation Timeline不得吸收shot/render/take域 |
| Document lifecycle/dirty/autosave/close/recovery | Editor61；Animation提供durable/view分类和document receipt |
| Transaction/history/savepoint/journal | Editor63/184；Animation不得建立私有history stack |
| Clipboard envelope/security/cross-document transfer | Editor55；Animation只注册typed key/channel payload与destination adapter |
| Preview cadence/clock/session | Editor69/190；Animation定义evaluator、seek和event policy adapter |
| Timeline/Curve shared foundation | `zircon_editor::ui::{timeline,curve}`；各domain实现model/controller，不复制selection/snap/input框架 |
| Renderer/cache | retained host只消费paged projection，不得读取asset/session成为第二authority |

禁止项：禁止继续用空slot、字符串列表或静态workspace表示Timeline完成；禁止以focus作为Sequence event身份；禁止把time bits、Vec index或display string当stable key ID；禁止whole-source snapshot成为大资产长期transaction格式；禁止在document write lock内执行无界全量compile；禁止用未经验证的64-bit digest作为cache内容身份；禁止为Curve、BlendSpace、Sequencer各建一套selection/transaction/clipboard/preview authority；禁止在没有100K/1M profile和同语义receipt时宣称性能超过Unreal。

## 13. 状态与产出记录

- review状态：`current_source_refresh_complete`。
- implementation状态：`pending`。
- canonical owner：Editor75；本报告不增加跨报告canonical finding总数。
- P1：15项，7 Open / 7 Partial / 1 Closed；其中ED75原13项为5 Open / 7 Partial / 1 Closed，新增ED75-P1-14与P1-15均Open。
- P2：6项，4 Open / 2 Partial / 0 Closed。
- 资格门：48项，27 Fail / 19 Partial / 2 Pass。
- 动态验证：未运行Cargo、Editor、GUI/GPU、native input、save/reopen、plugin reload、fault/soak/profile或跨引擎benchmark。
- Tooling：按用户要求排除；未查询、轮询、等待或实时跟踪协调器。
- 后续实施前：复算115个Zircon文件与30个参考文件fingerprint，复核共享工作树在途修改及父owner终态，从ED75-M0 RED guards开始。

## 14. 最终判断

Zircon已经从“session直接改asset并误标dirty”的临时实现，前进到“core document + revision + transaction + transient session”的可保留架构骨架。这一变化真实且重要，不能继续沿用Editor75中瞬态dirty污染和完全无transaction的旧结论。

但当前还不是Animation Timeline产品。typed timeline/curve只是无人消费的只读词汇，真实Sequence slot为空；event仍靠focus和全局coalescing key；key没有持久身份；编辑、selection、snap、clipboard、curve handle和runtime preview没有垂直闭环；每次编辑又以整资产clone、整资产history和锁内全量compile支付成本，渲染仍是全key逐quad生成。按用户要求的工程级目标，下一步应优先封闭qualified identity、retention、delta transaction和typed product projection，再实现交互/preview，最后以virtualization、batch、fault与规模证据完成资格。继续给空slot或flat TimelineStrip追加零散按钮，只会制造第三套临时authority。
