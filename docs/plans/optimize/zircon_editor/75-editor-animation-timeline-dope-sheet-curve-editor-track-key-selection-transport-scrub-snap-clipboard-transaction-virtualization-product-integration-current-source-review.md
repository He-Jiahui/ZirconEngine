---
title: Editor Animation Timeline、Dope Sheet、Curve Editor、Track-Key Selection、Transport、Scrub、Snap、Clipboard、Transaction、Virtualization 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor75
review_date: 2026-08-23
baseline_head: 0d70d1ac6499abcf56c3f6c3ef43cb3a7502a249
baseline_epoch: 351
related_code:
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/timeline_strip
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/ui/binding/animation
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_event_execution
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders
  - zircon_editor/src/ui/retained_host/host_contract
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_blend_space_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_sequencer_workspace.zui
  - zircon_runtime/src/animation
  - zircon_runtime/src/plugin
tests:
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/ui/timeline_strip/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime/sequence.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/retained_animation_template_body.rs
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/tests/timeline_strip.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/74-editor-scene-selection-authority-primary-active-range-filter-named-set-history-document-world-scope-lifecycle-performance-product-integration-current-source-review.md
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
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Animation Timeline、Dope Sheet、Curve Editor、Track-Key Selection、Transport、Scrub、Snap、Clipboard、Transaction、Virtualization 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon已经有若干可以保留的局部底座。`AnimationEditorSession`能加载、保存Sequence/Graph/State Machine资产，Sequence session区分资产dirty与局部cursor/range/selection/playback状态；`TimelineStripGeneration`会过滤非有限值、限制tick数量并把静态tick内容放入有界cache；Runtime已经存在`CompiledAnimationSequence`、world compile/apply函数和`AnimationSequencePlayerComponent`。这些都不是纯占位符，应作为后续hard cut的输入。

但产品链没有把这些底座接成真正的Animation Timeline。默认Animation Sequence body只有一个空`animation_timeline_slot`和固定跳到frame 0的Scrub按钮，pane payload只是current/range/playback/track字符串，`TimelineStrip`只在Blend Space演示面里用固定数据绘制。Sequence event没有document/asset地址，dispatch永远命中当前focused view；Scrub虽然进入LatestState，coalescing key却是全局`TimelineCursor`，range/span/playback又被误归类为DurableReplay。Session单测明确规定这些状态不应dirty资产，Host却把所有changed mutation统一送入`ensure_document_external_effect`，使有效的selection/range/scrub/playback变化进入document dirty/autosave/close-prompt链。

Timeline primitive自身也不是工程级Dope Sheet：它只有一条flat track、无stable key/channel identity、无hierarchy、无viewport range或projection revision；每次构造会clone并hash全部keys，static tick cache还错误依赖track/key内容。进程全局cache用`64-bit FNV digest + visual budget`作为完整key，没有内容相等校验；并发miss可重复生成，所有surface争用一个mutex。绘制侧遍历全部ticks和keys，每个diamond key拆成逐scanline quad，没有visible-key culling、row virtualization、density LOD或batch。primitive没有hit test、capture、drag、keyboard、transaction或clipboard接口，因此“interactive”wrapper并不等于可编辑时间轴。

Editor14已唯一登记默认toolkit不可达、可见空槽/静态假能力、动画mutation无transaction、无共享semantic compiler和preview不执行runtime等5项P0，以及通用track/key/time/selection/snap/curve/clipboard/transport和静态Sequencer/BlendSpace产品差距。本报告不重复计数这些父账，只登记当前源码更细的state classification、document-qualified routing/retention、projection currentness、cache正确性/并发、生成/绘制复杂度和缺失回归证据。

本轮没有新增P0，新增 **0项P0、13项P1、6项P2与48个资格门**。目标不是继续扩展单行`TimelineStrip`，而是建立per-document `AnimationTimelineViewState`、stable `AnimationElementAddress`、rational `AnimationTimeDomain`、immutable/virtualized `AnimationTimelineProjection`、typed selection/edit transaction、runtime-backed preview session、snap/clipboard/curve服务和collision-safe per-surface render cache。当前状态为`review complete / implementation not started`；未运行Cargo、真实Editor、native input、GUI/GPU、save/reopen、fault/soak/profile或同语义跨引擎benchmark，不能宣称性能与表现达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | fingerprint |
|---|---:|---|---|
| Session/event | **30 / 7,391 / 6,984 / 270,156 / 26** | sequence session、binding/event/retention、host mutation/dirty/undo route | `8d44f93a718c9d85eca7ef0c1a87162d9382c69c4b7c0f0a99a56b0a073a84c9` |
| Timeline/product | **18 / 3,883 / 3,623 / 163,235 / 13** | generation/cache/painting、pane projection、Sequence/BlendSpace/Sequencer ZUI与产品装配 | `7487fbcb42744976ad77db6f74c18cbd79633d6e05c36b6606d6a2b6336122fd` |
| Runtime/focused tests | **20 / 3,702 / 3,466 / 134,298 / 35** | compiled sequence/player、plugin runtime、session/event/host/timeline tests | `74aec279bbd1d32119755017eb70dcd604c5182187df4bd17bacee8786fb86cb` |
| Zircon去重合计 | **68 / 14,976 / 14,073 / 567,689 / 74** | 三组按normalized path去重的working-tree scan | `65d35f3580585922a5b5c259211f41e73bfa9a61ebaa4c616b860704dbe08744` |
| Unreal selected set | **11 / 5,624 / 4,682 / 214,408 / 0** | Sequencer selection/time slider/cache/clipboard与Curve Editor命令、snap、drag、transaction | `3c118f2e0374b003d209a88cf5490d02217867ee6a5d2da4a83ac732ab9c4e0e` |
| Godot selected set | **5 / 14,193 / 11,922 / 524,896 / 0** | track editor、Bezier editor与typed track plugin | `1145c075139ff7240855ec902899a1058a40f217a97cca11233a72f426a997bc` |
| Fyrox selected set | **6 / 5,593 / 5,159 / 222,209 / 2** | animation selection/command/toolbar/track与独立curve editor | `40f3badbff86a495b1e523b25242923dd9cc41419e6725f2837d368446610a10` |
| Bevy selected set | **4 / 3,825 / 3,476 / 144,942 / 11** | typed runtime curves、graph、transition与player scheduling | `ac9d9f3b57b7badcd20d451a6d81f5f2afa1f056fbda8434c6660df6ed71d343` |
| Unity Graphics selected set | **4 / 2,677 / 2,275 / 107,582 / 0** | embedded curve picking/serialization、TextureCurve与clip migration | `c939dcd2958695920d55ddc319c4d22d2667ccd592b43d45345e54078b81331b` |
| 五引擎参考合计 | **30 / 31,912 / 27,514 / 1,214,037 / 13** | 五类本地参考按path去重 | `96b4b28962a4a9c95df06b7e5fea3de745ba0ecf9ee05168aa5841ab2585f590` |

fingerprint算法沿用Editor58-74：按normalized lowercase relative path排序，把`path + NUL + lowercase per-file SHA-256 + LF`串联后再取SHA-256。指标和fingerprint冻结于完整逐文件扫描时；随后从`d714ab00de5e8d3ac3fbb0a5fe63540d3d2f21ae`到本报告HEAD的聚焦路径无提交变化。落盘前工作树中`zircon_runtime/src/animation/clip_event.rs`与`manager/mod.rs`只有测试import/断行格式差异，已逐行复核为无语义变化；实现前仍须重算全部语料。

### 2.2 当前产品链

```text
Animation Sequence view
  -> AnimationSequenceV1 pane payload (strings + frame/range/playback scalars)
  -> animation_sequence_body.zui
  -> empty animation_timeline_slot
  -> one Scrub button -> ScrubTimeline { frame: 0 }

Animation command/event
  -> EditorAnimationEvent without document address
  -> focused_animation_sequence_instance()
  -> AnimationEditorSession mutation
  -> any changed=true -> ensure_document_external_effect
  -> document dirty metadata + presentation/reflection effects

Standalone TimelineStrip demo
  -> ZUI attributes -> flat TimelineStripGeneration
  -> global static tick cache
  -> eager tick text + eager scanline-diamond key painting

Runtime animation
  -> CompiledAnimationSequence / AnimationSequencePlayerComponent
  -> plugin runtime evaluation
  x no Editor preview/session/transport bridge
```

### 2.3 已有基础必须保留

1. 保留Sequence asset/session的加载、保存和资产dirty位，但把editor-local view state移出durable document mutation链。
2. 保留invalid duration/FPS/speed的有限值防护，但把静默coercion升级为typed validation与产品诊断。
3. 保留timeline tick upper bound和静态内容复用意图，但cache key、owner、并发与观测必须重做。
4. 保留retained-host的TimelineStrip专用paint node和几何/palette模块分离，改为消费typed projection page与batched instances。
5. 保留pane payload的immutable publication方向，但不能再以字符串列表代替track/channel/key模型。
6. 保留Runtime compiled sequence/player和plugin runtime接线，Editor preview必须消费同一compiler/evaluator，不建立第二解释器。
7. 保留event retention对高频cursor采用LatestState的方向，但key必须document-qualified且其他view state不能进入durable replay。
8. 保留session测试中“selection/range/playback不dirty资产”的合同，并把它提升到Host、autosave、close和save/reopen E2E。

## 3. 父报告校正、唯一owner与不重复计数

| 既有owner | 当前源码重验 | 本报告裁决 |
|---|---|---|
| Editor14 P0-1 | production asset registry仍只给UI asset装toolkit；动画toolkit主要由测试显式注册 | 继续由Editor14唯一计数；Editor75 Gate 1只消费其产品入口终态 |
| Editor14 P0-2 | Sequence body仍为空slot，BlendSpace/Sequencer仍是固定数据表面 | 不重复P0；Editor75只拥有typed projection与真实Timeline接线细节 |
| Editor14 P0-3 | Animation event仍宣称delegated transaction，但trace没有transaction ID | 通用缺transaction保持Editor14；本报告新增的是transient/durable分类与event contract失真 |
| Editor14 P0-4/P0-5 | Editor session与Runtime compiler/player仍无共享preview闭环 | 不重复；Editor75定义Timeline/transport如何消费父compiler和runtime preview authority |
| Editor14 P1-13至P1-27 | stable track/key identity、多选、编辑、snap、curve、clipboard、hierarchy、event lane等通用能力仍缺 | 保持Editor14登记；Editor75不把同一功能列表重新加13项 |
| Editor14 P1-51至P1-60 | BlendSpace/Sequencer静态假数据、空slot、persistence/performance/test/a11y仍成立 | 产品真值由Editor14计数；本报告只登记新证明的cache/paint复杂度和精确回归缺口 |
| Editor55 | 通用clipboard schema、跨document cut/copy/paste和security由其拥有 | Animation clipboard只定义typed channel/key destination adapter，不创建第二系统clipboard |
| Editor63 | transaction/history/savepoint/document scope/journal由其拥有 | Timeline drag/edit只提供begin/update/commit/cancel domain adapter和receipt，history继续归Editor63 |
| Editor69 | preview clock/cadence/transport跨产品父合同由其拥有 | Animation preview只定义runtime evaluator、animation event policy和timeline currentness adapter |
| Editor74 | 通用scene selection authority/query/named set/history由其拥有 | Animation key/track/tangent/marker selection使用相同authority原则，但为独立typed domain，不复用Scene Node状态 |

## 4. P0：本轮没有新增

Editor14的5项Animation P0均经当前源码重验仍成立，但没有发现比父账更窄且语义独立的新数据破坏/产品不可达P0。Host把transient变化送入dirty链是当前可达的严重产品正确性问题，但它不会直接修改Sequence asset payload，风险主要是错误autosave/close prompt、无意义写回与import，因此按P1登记。Cache hash collision理论上可返回错误ticks，但当前未证明可控碰撞或资产破坏路径，同样按P1正确性缺陷处理。

## 5. P1：本轮新增的工程差距

### ED75-P1-01 · Editor-local Timeline状态污染durable document dirty/autosave/save链

`session/sequence.rs`中的scrub、visible range、selected span与playback只修改`AnimationSequenceDocument`的局部字段，session tests又明确断言它们不应使Sequence asset dirty；`document_bytes()`也只序列化资产。Host的`with_animation_session_mut`却把任意`changed=true`统一送入`ensure_document_external_effect`并刷新document dirty metadata。有效的selection/range/scrub/playback因此会触发dirty、autosave/close prompt及可能的未变化资产重写/import。必须把`AnimationTimelineViewState`从durable mutation中物理分离，并让Host按mutation class决定dirty/effect。

### ED75-P1-02 · Range、Span Selection与Playback被错误保留为DurableReplay

`retention_class`只专门把`ScrubTimeline`列为LatestState；`SetTimelineRange`、`SelectTimelineSpan`和`SetPlayback`落入默认DurableReplay。它们既未进入asset bytes，也不应在crash/reconnect后作为durable authoring操作重放。当前分类会让editor-local状态占用durable队列，并在focus变化后作用于错误session。需要明确`DurableAssetEdit / ViewLatestState / FrameGesture / TransportIntent`等typed类别及各自retention上限。

### ED75-P1-03 · Scrub coalescing key是进程全局，多个document会互相覆盖

`EditorEventLatestStateKey::TimelineCursor`不携document、view instance或session epoch；所有Animation Sequence文档的Scrub共享一个latest-state槽。两个文档并发拖动或late delivery时，后一个cursor可淘汰前一个仍待处理的状态。key必须至少包含`DocumentSessionId + ViewInstanceId + TimelineSessionEpoch`，coalescing receipt要能报告superseded sequence和最终applied revision。

### ED75-P1-04 · Sequence event按当前focus寻址，晚到/replay可命中另一文档

`ScrubTimeline`、range、span、playback和durable track/key事件都不带asset/document locator；`apply_animation_event`逐分支调用`focused_animation_sequence_instance()`。事件入队后只要用户切换focus，执行目标就可能改变；没有expected asset revision、view generation或closed-session拒绝。必须把qualified document target写入command/event/journal，dispatch按地址resolve并在stale/closed时fail-close，不能把focus当身份。

### ED75-P1-05 · Blanket undo policy把transient与durable Animation事件伪装成同一transaction合同

`undo_policy_for_event`将所有`EditorEvent::Animation(_)`标成`DelegatedToTransactionEngine`，但`authoring_trace`只为Inspector和Operation读取transaction，Animation record得到`transaction_id=None`；Scrub/selection/playback本就不应undo，Add/Remove Key等durable edit则必须有transaction。应按domain operation分类：view/transport non-undoable，durable edit必须携transaction token和before/after receipt。缺失的通用transaction实现仍归Editor14/63，本项只登记分类与可观测合同错误。

### ED75-P1-06 · Pane payload与Timeline primitive是两套互不相连的数据authority

`AnimationSequenceV1`发布frame/range/playback label和`Vec<String>` tracks；真实Sequence body只放anchor，不把payload转换为`TimelineStripGeneration`。Timeline primitive的输入来自component attributes，而Blend Space workspace又硬编码另一套duration/current/track/keys。没有source asset revision、projection generation、track/key identity或stale/last-good状态，UI无法证明画面对应哪次资产/compile/preview。必须只有一个typed immutable `AnimationTimelineProjection`，产品slot、curve/dope sheet和automation共同消费。

### ED75-P1-07 · Static cache以64-bit digest冒充完整内容身份，collision会返回错误ticks

`StaticContentCacheKey`只有`static_generation: u64`和`visual_budget`，cache命中不比较duration/tick interval或canonical source identity。`static_generation`是本地FNV式hash，理论collision会把另一文档内容直接当成当前静态内容返回。Cache key必须是collision-safe revision/content address，或在digest命中后验证完整canonical descriptor；错误内容不得以“概率低”作为工程合同。

### ED75-P1-08 · 进程全局cache mutex造成跨surface争用，并发miss重复生成

所有Timeline surface共享`OnceLock<Mutex<StaticContentCache>>`。读取先锁一次，miss后在锁外生成candidate，再二次加锁；同key并发miss会重复做tick/label allocation，所有document又在同一mutex和16-entry全局容量上竞争。LRU touch用`VecDeque::retain`线性扫描。应改为per-surface/per-render-context revision cache，single-flight generation、bounded memory和显式eviction，不让一个Timeline的宽度/抖动淘汰所有其他文档。

### ED75-P1-09 · Tick策略按像素宽度膨胀到4096个文本标签

`visual_tick_budget`取`ceil(plot_width)+1`并限制到4096；当requested interval更密时，tick数量可接近每个像素一个。Painter随后为每个tick格式化/测量/提交文本，没有major/minor分层、label overlap计算、帧率/timecode unit或zoom级别稳定策略。需要基于字体measure、最小label spacing和time domain选择major/minor ticks，只绘制可读标签并保证pan/zoom时标尺稳定。

### ED75-P1-10 · Cursor/selection变化仍clone、扫描、hash全部keys，static tick cache错误依赖track/key数据

`TimelineStripGeneration::new`接收并消费`Vec<TimelineStripKey>`，重新filter/clamp/collect；`static_generation`扫描duration、interval、track label以及每个key的time/label，`dynamic_generation`又扫描current time与全部selected flags。实际`TimelineStripStaticContent`只有ticks，track/key内容不应使tick cache失效。百万key下cursor或selection刷新仍是O(keys) hash/clone。应以asset/projection revision和paged key storage建立增量generation，cursor/selection是独立小revision，tick cache只依赖time-domain/view descriptor。

### ED75-P1-11 · Painter eager遍历全部ticks/keys，diamond按scanline拆quad，无virtualization/LOD/batch

Timeline painter每帧调用text和keys/playhead路径；key loop没有visible range/row culling，每个diamond按`2 * radius + 1`条scanline产生quads。10万至100万keys或多track时command数、CPU和allocation线性增长，即使绝大多数key在viewport外。必须建立row virtualization、visible time-page query、density bin/aggregate LOD、batched/instanced key glyph与bounded command budget，并记录culled/visible/dropped/overload receipt。

### ED75-P1-12 · Timeline primitive没有编辑输入语义，所谓interactive只会generic click

primitive只负责generation和paint，没有stable key hit target、pointer capture、drag threshold、box select、keyboard focus、context action、scroll/zoom或transaction callback。Blend Space wrapper设置interactive/clickable/focusable，但点击只走通用select；Sequence body唯一真实binding又固定`ScrubTimeline { frame: 0 }`。需要`AnimationTimelineInteractionController`把native input映射成qualified hit/gesture/request，并支持begin/update/commit/cancel和capture loss，不能用控件flag代替产品行为。

### ED75-P1-13 · 现有测试验证hash/anchor/codec，却没有覆盖上述跨层正确性与规模风险

Session测试只证明直接调用时transient不dirty；Host/event没有测试一次有效scrub/range/span/playback后document仍clean。Timeline测试验证normalization、generation和cache容量，hybrid测试只断言anchor存在，binding测试主要做codec round-trip。没有document切换后的late event、per-document coalescing、transaction ID、cache collision/single-flight、real hit/drag/cancel、projection currentness、100K/1M key或render command budget测试。必须先以RED集成/性质/并发/规模测试固定这些当前缺陷，再进入重构。

## 6. P2：测试、维护与产品完整性缺口

- **ED75-P2-01**：flat input保留未排序keys，并把负值、超duration值全部clamp到边界；重复/重叠key没有canonical order、collision policy或用户诊断。
- **ED75-P2-02**：key label参与static hash并占用内存，但paint、hit target、tooltip和accessibility均不消费它，当前字段成本与产品语义不一致。
- **ED75-P2-03**：selected key只把diamond radius增加1px且沿用同一palette；密集关键帧、高DPI、色觉缺陷和keyboard focus下缺少可靠多通道反馈。
- **ED75-P2-04**：ruler固定`{time:.1}`秒，footer使用seconds/percent；没有frame、subframe、SMPTE timecode、display rate、tick resolution或locale formatting。
- **ED75-P2-05**：invalid duration/tick/current被静默改为1.0/0.25/0.0，session对invalid FPS/duration也静默fallback；没有source diagnostic、degraded badge或repair action。
- **ED75-P2-06**：全局cache容量固定16且没有hit/miss/eviction/contention/collision/duplicate-build telemetry，也没有按surface/device/zoom/key-count分配预算的配置与profile证据。

## 7. 五引擎参考结论

| 参考 | 可采用的工程事实 | 不应照搬/证据限制 |
|---|---|---|
| Unreal | Sequencer把Outliner/Track Area/Key/Marked Frame selection分域；typed channel/clipboard保留destination identity；per-channel key cache与sorted handles支撑范围查询；time slider区分display/tick rate及view/clamp/play/selection ranges；Curve Editor分离time/value snap，drag支持begin/update/cancel/restore与transaction，命令覆盖tangent/interpolation/bake/reduce/extrapolation | UObject、Slate、MovieScene transaction与全局Editor体系不能照搬；应抽取stable identity、typed time、currentness、interaction lifecycle和budget合同 |
| Godot | Animation Track Editor拥有真实multi-key/box selection、move preview/commit/cancel、snap、copy/paste/cut/duplicate/delete、marker与typed track rendering；Bezier Editor直接编辑handles；UndoRedo包围authoring mutation | 单体Editor和Node/Animation专用分支较多，不是Zircon多document/plugin隔离或百万key性能终态 |
| Fyrox | typed animation selection、可逆command、preview mode进入时备份并退出恢复、真实time position/track tree；独立Curve Editor有自己的command history和曲线交互 | 局部command stack证明Rust闭环可行，但Zircon必须接Editor63统一history，不能再建私有第二stack |
| Bevy | runtime curve/graph/transition以typed data和runtime evaluator为中心，AnimationPlayer/Transitions体现执行状态与调度边界 | 本地Bevy没有完整authoring timeline/curve UX；只能作为Runtime typed evaluator参考，不能用它证明Editor产品完整性 |
| Unity Graphics | Inspector/Post Processing embedded curve editor提供serialized curve、picking和局部编辑参考；TextureCurve体现runtime curve storage/evaluation；AnimationClip upgrader强调schema migration必须显式 | 本地`dev/Graphics`不是完整Unity Animation Window/Timeline源码，不能据此推断track/key selection、transport或Undo产品，只能采用embedded curve和migration证据 |

### 7.1 综合判断

成熟实现共同把Timeline视为“typed document projection + interaction transaction + runtime evaluation”的产品，而不是一张按秒绘制diamond的图。稳定element identity、rational time domain、分层selection、可取消drag、document-qualified event、typed clipboard/snap、曲线handles、runtime-backed scrub和可视范围分页是正确性合同；virtualization、LOD、batch和可观测cache才是规模合同。Zircon当前分别有session、retained painter和runtime evaluator的碎片，但没有单一currentness链把三者连接起来。

## 8. 目标架构

### 8.1 Durable Document与Ephemeral View State

`AnimationDocumentSession`只拥有asset、source revision、schema/compile status和durable dirty/savepoint；`AnimationTimelineViewState`按document/view保存cursor、view/work/selection range、zoom/pan、expanded rows、selection、transport UI与focus。两者通过typed mutation class交互，view state不得触发asset dirty或durable replay。Close/reopen/profile persistence由Editor61/13提供scope和migration，不能把view字段重新塞回asset session。

### 8.2 Stable Address与Rational Time Domain

```rust
struct AnimationElementAddress {
    document: DocumentSessionId,
    binding: AnimationBindingId,
    track: AnimationTrackId,
    channel: Option<AnimationChannelId>,
    element: AnimationElementId, // key / marker / section / tangent
    generation: AnimationElementGeneration,
}

struct AnimationTimeDomain {
    display_rate: RationalFrameRate,
    tick_resolution: RationalFrameRate,
    view_range: TimeRange,
    work_range: TimeRange,
    playback_range: TimeRange,
    selection_range: Option<TimeRange>,
}
```

时间不得以`f32 seconds + EPSILON`作为身份或duplicate判定。Key使用stable ID，时间使用integer tick/subframe或有界rational；display formatting、snap、serialization和runtime conversion共享同一domain及显式rounding policy。

### 8.3 Typed Immutable Projection与Selection

`AnimationTimelineProjection`携document/source/schema/compile/preview revisions、hierarchical row descriptors、visible time range、paged keys、density bins、marker/event/audio lanes和last-good/stale状态。`AnimationSelectionState`分track/channel/key/tangent/marker/section domain，支持stable multi-selection、primary/anchor/hover/preview；设计原则消费Editor74，但状态与provider是Animation专属，不能拿Scene Node集合代替。

### 8.4 Edit Transaction、Snap、Clipboard与Curve

`AnimationEditTransaction`具有begin/update/commit/cancel，保存before/after element revisions、drag origin、snap receipt和affected range；只在commit接Editor63 history。`AnimationSnapService`组合grid/key/marker/section/audio/guide candidates并返回chosen candidate/reason。Clipboard使用Editor55 envelope，payload保留source time domain、channel type、key/tangent/interpolation和destination compatibility。Curve Editor消费相同projection/selection/transaction，不复制资产或history。

### 8.5 Runtime-backed Preview与Transport

`AnimationPreviewSession`绑定document、compiled artifact revision、isolated preview world、runtime player/evaluator和Editor69 transport clock。Scrub提交time request并获得evaluation receipt；play/pause/step/reverse/loop定义event firing policy、range crossing和fixed/display rate转换。Source变更后compile/install必须以generation/currentness提交，UI清楚区分Current/Compiling/LastGood/Stale/Failed，不能用本地cursor字符串伪装预览。

### 8.6 Virtualized Renderer与Cache

Timeline只请求visible rows和time pages；数据层提供sorted key index、range query和density pyramid。Renderer为ticks、keys、selection、tangent和markers生成bounded batched instances；zoom-out使用density LOD，zoom-in才materialize individual hit targets。Cache按surface/device/theme/font/time-domain/projection revision分层，collision-safe、single-flight、bounded且可观测；cursor/selection变更不重建static ticks或全部key pages。

## 9. 分层里程碑

### ED75-M0：Truthfulness、RED Guards与Hard-Cut Inventory

- 固化valid transient导致Host dirty、global scrub coalescing和focus-retarget的RED tests。
- 枚举所有Animation event、session mutation、timeline projection/cache/paint和静态workspace入口。
- 把Editor14/55/63/69/74 owner边界写成禁止重复实现清单，撤销未接线的产品能力声明。

### ED75-M1：Identity、State Classification与Time Domain

- 分离durable document与per-view timeline state，建立typed mutation/retention class。
- 建立qualified document/view/session target与stable track/channel/key/marker IDs。
- 用rational display rate/tick resolution/subframe替换`f32 + EPSILON`身份语义。

### ED75-M2：Typed Projection与真实Product Slot

- 建立immutable projection、revision/currentness和last-good/failure状态。
- 让Animation Sequence body真实消费hierarchical rows/keys/ticks，不再为空anchor。
- 删除BlendSpace/Sequencer固定数据authority或明确降级为fixture-only。

### ED75-M3：Selection与Track-Key Interaction

- 建立typed multi-domain selection、hit target、range/box selection和keyboard focus。
- 实现qualified pointer capture、drag threshold、pan/zoom/scrub与cancel。
- 所有interaction返回terminal disposition/receipt，不以generic clickable flag代替。

### ED75-M4：Transaction、Snap、Clipboard与Curve

- 建立edit begin/update/commit/cancel并接Editor63 transaction/history。
- 建立typed snap candidate/receipt和Editor55 clipboard destination adapter。
- Curve Editor共享key identity、selection、time domain、transaction与runtime data。

### ED75-M5：Runtime Preview、Transport与Currentness

- 接共享compiler、compiled artifact和isolated runtime preview world。
- Scrub/step/play/reverse/loop真正evaluate，并定义event firing/range crossing policy。
- 处理compile/install supersession、stale/last-good、close/reopen和Play transition。

### ED75-M6：Marker、Notify、Audio与高级Lanes

- 建立marker/notify/event/audio/section lanes和typed payload editor。
- 定义lane-specific hit/render/snap/clipboard/transaction及runtime delivery。
- 支持typed track plugin/provider capability、owner lease与reload降级。

### ED75-M7：Virtualization、Cache与Render Budgets

- 建立visible row/time page、sorted range index、density pyramid和LOD。
- key/tick/tangent/marker使用batched instances与bounded command/allocation预算。
- 改为collision-safe per-surface single-flight cache并输出命中/争用/淘汰指标。

### ED75-M8：Lifecycle、Persistence、Accessibility与Diagnostics

- view/profile state schema-versioned persistence，asset savepoint与view dirty严格隔离。
- 完成keyboard、screen reader、focus、high-DPI、reduced-motion/contrast产品矩阵。
- 输出脱敏interaction/transaction/preview/cache/performance receipts和replay corpus。

### ED75-M9：Scale、Fault与跨引擎资格

- 完成100K/1M keys、10K tracks、多document、多surface、high-frequency scrub/drag profile。
- 注入cache collision/stampede、compile supersession、plugin revoke、capture loss和save/reopen故障。
- 与Unreal/Godot/Fyrox做同语义功能/延迟/内存/command count比较；Bevy和Unity Graphics只在其证据边界内比较。

## 10. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED75-G01 | production可从默认资产入口打开真实Animation Sequence toolkit | Fail |
| ED75-G02 | 产品不再显示空slot、固定数据或不可执行控制 | Fail |
| ED75-G03 | durable asset state与per-view timeline state物理分离 | Fail |
| ED75-G04 | scrub/range/selection/playback变化不dirty、不autosave资产 | Fail |
| ED75-G05 | durable key/track edit必然dirty并产生savepoint关联 | Fail |
| ED75-G06 | track/channel/key/marker/section/tangent均有stable typed ID | Fail |
| ED75-G07 | 每个command/event携qualified document/view/session target | Fail |
| ED75-G08 | retention class按asset edit/view state/gesture/transport明确区分 | Fail |
| ED75-G09 | latest-state coalescing按document/session隔离且有supersession receipt | Fail |
| ED75-G10 | time domain使用rational display rate/tick resolution/subframe | Fail |
| ED75-G11 | view/work/playback/selection ranges语义独立且可迁移 | Fail |
| ED75-G12 | Timeline消费typed immutable projection而非字符串/attributes双authority | Fail |
| ED75-G13 | Sequence product slot显示真实hierarchy、channels、keys与status | Fail |
| ED75-G14 | projection携source/schema/compile/preview revisions与currentness | Fail |
| ED75-G15 | row hierarchy支持binding/track/channel/lane及稳定expansion | Fail |
| ED75-G16 | track/channel/key/tangent/marker/section支持typed multi-selection | Fail |
| ED75-G17 | key/handle/marker hit test返回stable address与距离/priority receipt | Fail |
| ED75-G18 | box/range selection在pan/zoom/virtualization下语义稳定 | Fail |
| ED75-G19 | move/duplicate/delete/value edit使用同一qualified mutation path | Fail |
| ED75-G20 | curve tangent/interpolation/extrapolation/bake/reduce具有typed命令 | Fail |
| ED75-G21 | snap组合grid/key/marker/section/audio并返回candidate receipt | Fail |
| ED75-G22 | clipboard保留source time/channel/key/tangent和destination identity | Fail |
| ED75-G23 | drag/edit具有begin/update/commit/cancel及single transaction | Fail |
| ED75-G24 | undo/redo恢复精确key IDs、times、values、tangents与selection | Fail |
| ED75-G25 | capture loss/Escape/close/cancel精确恢复before state | Fail |
| ED75-G26 | preview使用共享compiler和Runtime evaluator/isolated world | Fail |
| ED75-G27 | scrub/step会实际evaluate并回传evaluated revision/time | Fail |
| ED75-G28 | play/pause/reverse/speed/loop/range crossing/event policy完整 | Fail |
| ED75-G29 | Current/Compiling/LastGood/Stale/Failed状态真实且generation-qualified | Fail |
| ED75-G30 | marker/notify/event/audio lanes有authoring与runtime delivery闭环 | Fail |
| ED75-G31 | tick采用adaptive major/minor spacing且label不重叠 | Fail |
| ED75-G32 | frame/subframe/timecode/seconds格式与display rate一致 | Fail |
| ED75-G33 | rows按viewport虚拟化且scroll不会materialize全树 | Fail |
| ED75-G34 | keys按visible time range/page cull | Fail |
| ED75-G35 | zoom-out使用density LOD并保留selection/currentness语义 | Fail |
| ED75-G36 | key/tick/tangent/marker绘制批处理且command count有界 | Fail |
| ED75-G37 | cache key collision-safe或命中后验证完整descriptor | Fail |
| ED75-G38 | 无进程全局timeline cache锁瓶颈与并发重复build | Fail |
| ED75-G39 | memory/key pages/render commands/labels有硬预算与overload结果 | Fail |
| ED75-G40 | 多document并发scrub/edit/cache/preview完全隔离 | Fail |
| ED75-G41 | keyboard/focus/screen reader/high-DPI/contrast产品矩阵通过 | Fail |
| ED75-G42 | disabled/stale/error/selection/preview反馈真实且可操作 | Fail |
| ED75-G43 | production bootstrap和默认asset open E2E通过 | Fail |
| ED75-G44 | transient dirty、transaction、undo、retention/replay regression通过 | Fail |
| ED75-G45 | compile/preview currentness、close/reopen、fault injection矩阵通过 | Fail |
| ED75-G46 | 100K/1M keys、10K tracks、多surface profile与soak通过 | Fail |
| ED75-G47 | 真实GUI/native input/capture/render golden与a11y自动化通过 | Fail |
| ED75-G48 | 同语义跨引擎功能、性能、内存和command-count receipt可复现 | Fail |

## 11. 测试与动态证据矩阵

| 层级 | 当前证据 | 缺失的最低资格 |
|---|---|---|
| Session unit | add/remove/create/rebind、range clamp、invalid timing/speed、save dirty | Host-valid transient cleanliness、durable/view state split、rational time与stable key identity |
| Event/retention | codec、部分runtime sequence dispatch、Scrub LatestState | qualified document target、per-document coalescing、late focus switch、durable/view classification、transaction trace |
| Product projection | pane payload、slot anchor、TimelineStrip attribute conversion | 默认asset open、真实Sequence projection、currentness/stale/LKG、no-fixture product assertion |
| Interaction | 无真实Timeline hit/drag/capture测试 | native pointer/keyboard、box/range selection、pan/zoom/scrub、cancel/undo/redo、clipboard/snap/curve |
| Cache/concurrency | normalization、generation、static reuse与capacity | digest collision、single-flight、multi-surface contention、eviction fairness、bounded telemetry |
| Performance/render | 无规模profile或command budget | 100K/1M keys、10K rows、density LOD、visible culling、batch command/allocation、high-DPI golden |
| Runtime preview | Runtime compiler/player各自有局部测试 | Editor-to-runtime compile/install/evaluate、event policy、generation supersession、close/reopen/Play/fault |
| Cross-engine | 本轮逐文件静态对照 | 同语义fixture、相同硬件/viewport/zoom/key density、可复现latency/memory/command receipt |

本轮按用户要求只做review，没有运行Cargo或动态产品测试。表中“当前证据”只说明仓内已有静态/单元测试意图，不代表这些测试在当前共享工作树通过。

## 12. Owner路由与禁止重复实现

| 合同 | 唯一owner/消费者关系 |
|---|---|
| Animation asset/schema/compiler/runtime preview总账 | Editor14 + Runtime animation；Editor75只接Timeline projection/interaction/transport adapter |
| Document lifecycle/dirty/autosave/close/recovery | Editor61；Editor75提供durable/view mutation classification与animation-specific receipts |
| Transaction/history/savepoint/journal | Editor63；Animation edit不得建立私有history stack |
| Clipboard envelope、security、cross-document transfer | Editor55；Animation只注册typed payload/destination compatibility |
| Preview cadence/clock/process/currentness父合同 | Editor69；Animation preview使用其clock/session并定义evaluator/event policy |
| Selection authority通用原则 | Editor74；Animation拥有独立element provider和typed selection state，不复用Scene Node selection |
| Timeline layout/render primitives | zircon_editor retained host；只能消费typed paged projection，不得读取asset/session成为第二authority |
| Runtime curve/graph/player | zircon_runtime animation；Editor不得实现一套仅供预览的替代evaluator |

禁止项：禁止继续给空Sequence slot添加静态label冒充功能；禁止以focus作为event target；禁止把cursor/range/selection/playback写入asset dirty链；禁止用`f32 + EPSILON`作为key identity；禁止以64-bit digest作为未经验证的cache内容身份；禁止为Curve Editor、BlendSpace或Sequencer复制selection/transaction/clipboard/preview authority；禁止在没有100K/1M profile和同语义receipt时宣称超过Unreal。

## 13. 状态与产出记录

- review状态：complete。
- implementation状态：not started。
- 新增finding：0 P0 / 13 P1 / 6 P2。
- 资格门：48项，当前全部Fail。
- Session：`optimize-editor75-animation-timeline-review-r2-20260823`；因共享`docs/plans/optimize/00`已有可执行primary且并发`failure-*.md`持续变化，本轮使用无plan-family但精确四路径write scope的独立review session。
- Coordinator baseline：HEAD `0d70d1ac6499abcf56c3f6c3ef43cb3a7502a249`，epoch 351。
- 动态验证：未运行Cargo、真实Editor、GUI/GPU、native input、save/reopen、fault/soak/profile或benchmark。
- 后续实施前：重算全部fingerprint，重验Editor14/55/61/63/69/74终态，重新声明源码write scope并按M0 RED guards开始。

## 14. 最终判断

Zircon当前拥有“可保存Animation资产的session”“能画一行tick/key的retained primitive”和“能执行compiled sequence的Runtime”三个局部事实，但它们之间没有document-qualified、revision-qualified、transactional、runtime-truthful的产品链。最危险的不是按钮少，而是view state被当成durable edit、event被focus重定向、全局coalescing/cache跨document混用，以及所有key数据在生成和绘制热路径上被全量扫描。

因此下一步不能继续给现有`TimelineStrip`追加零散控件。必须先按ED75-M0至M2硬切state classification、qualified identity、rational time和typed projection，再接interaction/transaction/preview，最后用virtualization、batch、fault与跨引擎同语义receipt完成规模资格。任何跳过这些前置合同的“Dope Sheet/Curve Editor已完成”都只能增加第三套临时authority，不能接近用户要求的工程级引擎。
