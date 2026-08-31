---
related_code:
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/core/asset/type_registry/builtin.rs
  - zircon_editor/src/core/asset/source_authority.rs
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/binding/animation
  - zircon_editor/src/ui/binding_dispatch/animation
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_editor/src/ui/host/editor_event_execution/animation_event.rs
  - zircon_editor/src/ui/host/editor_manager_animation_editor.rs
  - zircon_editor/src/ui/layouts/views/animation_editor.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders
  - zircon_editor/src/ui/timeline
  - zircon_editor/src/ui/timeline_strip
  - zircon_editor/src/ui/curve
  - zircon_editor/src/ui/graph
  - zircon_editor/src/ui/preview_scene
  - zircon_editor/src/ui/weight_heatmap
  - zircon_editor/assets/ui/editor/host/animation_graph_body.zui
  - zircon_editor/assets/ui/editor/host/animation_sequence_body.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/animation
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_plugins/animation/runtime
  - zircon_plugins/animation_graph/editor
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/78-editor-control-rig-rig-graph-hierarchy-controls-spaces-constraints-ik-solve-bake-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/79-editor-motion-matching-pose-search-database-feature-schema-trajectory-query-runtime-selection-preview-debugger-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/80-editor-animation-montage-section-slot-segment-notify-branching-point-sync-root-motion-runtime-playback-preview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/81-editor-animation-pose-library-pose-asset-pose-name-curve-weight-additive-base-runtime-evaluation-preview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/82-editor-animation-blend-space-axis-sample-triangulation-interpolation-filter-per-bone-additive-sync-runtime-evaluation-preview-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/14/failure-2026-08-24-animation-editor-zui-deletion-closure.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/AnimationEditor/Private/AnimationEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/AnimationEditorPreviewScene.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/AnimTimeline/AnimModel.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/AnimTimeline/AnimModel_AnimSequenceBase.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/AnimTimeline/SAnimTimeline.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimationGraphSchema.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimGraph/Private/AnimBlueprintCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Developer/AnimationDataController/Private/AnimDataController.cpp
  - dev/godot/editor/animation/animation_track_editor.cpp
  - dev/godot/editor/animation/animation_bezier_editor.cpp
  - dev/godot/editor/animation/animation_blend_tree_editor_plugin.cpp
  - dev/godot/editor/animation/animation_state_machine_editor.cpp
  - dev/godot/editor/animation/animation_player_editor_plugin.cpp
  - dev/Fyrox/editor/src/plugins/animation/mod.rs
  - dev/Fyrox/editor/src/plugins/animation/command/mod.rs
  - dev/Fyrox/editor/src/plugins/absm/mod.rs
  - dev/Fyrox/editor/src/plugins/absm/command/mod.rs
  - dev/Fyrox/editor/src/plugins/absm/blendspace.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/InspectorCurveEditor.cs
refreshes:
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
evidence_captured_at: 2026-08-26T14:41:07+08:00
---

# 136 · Editor Animation Sequence、Graph、State Machine、Timeline/Curve、Preview 与 Compiler Authoring 当前源码复核

## 1. 结论

Editor14之后，动画创作链不再只是直接修改session vector的原型。当前源码已经建立生产内建toolkit route、canonical kind解码、typed load/target diagnostic、恢复回滚、durable conditional source write、`AnimationAuthoringDocumentStore`、document revision/CAS、`HistoryContextId::Document`事务、undo/redo、共享Sequence/Graph/State Machine compiler和last-good compilation。这些是必须保留的真实工程基础，原P0-1与P0-3可判定为Closed，P0-2与P0-4进入Partial。

但产品仍不能称为完整动画编辑器。两个当前P0保持直接阻断：没有任何生产`PreviewSceneBackend`把source/compiler产物送入runtime evaluator；已删除的`animation_editor.zui`仍被生产pane loader、integration test和template boundary引用，loader又以`unwrap_or_default`把缺失降级成空节点，导致主动画pane可能静默为空。高级Sequencer、Montage、Blend Space、Pose Library、Retarget、Control Rig、Motion Matching与Compression工作区继续用静态ZUI和“queued”反馈表达尚不存在的asset/job/compiler/preview结果。

共享compiler也尚未形成产品闭环：Editor document每次交换整份asset时同步全量编译，但diagnostic/last-good只在core测试读取；capability table仍声明`animation.compiler.semantic=false`；`zircon_plugins/animation_graph/editor`保留另一套返回字符串的Graph/State Machine validator；runtime Graph/State Machine会先消费共享compiler，而world Sequence仍走独立`compile_sequence_for_world`。因此“存在compiler”不能等同于save、preview、cook和runtime已经共享一个authoritative artifact。

本轮保留Editor14原5个P0、60个P1、12个P2的canonical ID并重新判定，新增`E-ANIM-P0-06`和`E-ANIM-P1-61..65`。当前总表为P0：2 Open / 2 Partial / 2 Closed；P1：44 Open / 16 Partial / 5 Closed；P2：12 Open。32个验收门仍保留，当前17 Partial、15 Open、0 Closed。本文只做current-source review，不修改production、测试或asset schema。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Authoring authority与host | **53 / 6,933 / 6,451 / 251,020 / 23 / 0** | core document、session、binding、event、toolkit、save与pane链；fingerprint `a34267312b339b7cf8c9a998886161214f37e7dd48ceb11f0021188bf4ae4692` |
| Product UI与基础控件 | **41 / 8,297 / 7,487 / 337,945 / 40 / 0** | timeline/curve/graph/preview foundation、host ZUI和8个高级工作区；fingerprint `30e142df49d1c9f714e946bf2141d89f0838561db4a09e5720d9dc37979f4037` |
| Runtime/compiler/plugin | **206 / 24,717 / 22,614 / 844,569 / 105 / 0** | framework compiler、runtime sequence、animation runtime plugin、graph editor plugin与glTF接点；fingerprint `b0acc5cfb7d32c0e5cee8d57bef1864449ceb8437833a24ab04c3b88cca9c647` |
| Focused tests | **39 / 8,559 / 7,945 / 313,063 / 146 / 0** | Editor animation event/host/UI/integration与runtime plugin contract tests；fingerprint `7b69ea3b9de35ea3f35da89f183230037f80fe5dcc90d953b247bfd8518f3354` |
| Zircon selected union | **339 / 48,506 / 44,497 / 1,746,597 / 314 / 0** | 上述四组去重集合；fingerprint `39ed0c130e6f6cd3778888623145d7ad85716caced0ed1b2416c7dc77d098104` |
| Reference engines | **22 / 27,752 / 23,878 / 1,403,777 / 12 / 0** | Unreal/Godot/Fyrox/Bevy/Unity Graphics selected evidence；fingerprint `f9735de41085c1711498b9005fb6e29167ec15cab18424b2506f32b54149a8df` |
| Plan/docs evidence | **11 / 4,729 / 3,373 / 444,124 / 2 / 0** | Editor14、75-82、Runtime08C与open failure；fingerprint `104ea6c63e7a629da66c1ac2dd6dde35ca701091fb77ddd4da2af61520082b7d` |
| Zircon/reference/docs union | **372 / 80,987 / 71,748 / 3,594,498 / 328 / 0** | `2026-08-26T14:41:07+08:00`捕获的去重集合；fingerprint `1a7158b8f5554089778d4bfa85144638cef80f9677b1806857f6302aeed2b2ae` |

fingerprint算法与Editor135一致：相对路径排序，对每个文件计算SHA-256，再对`path + NUL + per-file hash + LF`的UTF-8 manifest计算SHA-256。它标识实际阅读集合，不是animation schema、compiler key或artifact identity。

### 2.2 Test inventory、在途隔离与未执行项

Zircon selected union静态包含314个`#[test]`属性、0 ignored；focused test集合包含146个。测试已覆盖默认toolkit route、kind mismatch、部分restore rollback、document revision/CAS、undo/redo、compiler错误、Graph/State runtime lowering、Graph diamond/deep-chain非递归求值和若干foundation model。它们没有证明真实ZUI主pane、production preview、compiler result UI、save/import第N步原子性、外部编辑合并、插件schema迁移、十万key/大图性能、crash recovery或完整高级工作区。

成文时animation相关tracked/untracked源码、测试和asset仍处于共享工作树在途状态，包括完整`core/editing/animation_document`、timeline/curve/graph foundation、host session、runtime compiler/plugin与旧helper删除。本报告不回退、格式化、暂存或提交这些更改；实施前必须重取source、fingerprint、failure状态和动态测试结果，因此`source_recheck_required=true`。

证据等级：

- E3：`OpenAsset -> builtin toolkit route -> restore -> core document/session/toolkit -> pane projection`逐函数闭环。
- E3：`binding/event -> prepare whole-asset replacement -> transaction apply -> commit_after_apply -> dirty/projection -> undo/redo`逐分支闭环。
- E3：document source swap、revision CAS、current/last-good compilation、save source authority与import回执逐函数闭环。
- E3：共享compiler与runtime Graph/State Machine lowering、world Sequence compiler以及graph editor plugin validator逐入口对照。
- E3：timeline/curve/graph/preview foundation的全部生产consumer搜索；高级工作区ZUI/action/feedback逐项对照。
- E3：Unreal、Godot、Fyrox的transaction、graph schema/compiler、timeline与preview逐源码对照。
- E2：Bevy只作为runtime source graph/threaded graph、mask/event/transition分层参考，不是Editor UX基准；Unity Graphics checkout只有局部`InspectorCurveEditor`，不能外推Unity核心Animation Editor。
- 未执行：Cargo、Editor启动、真实runtime preview、磁盘/崩溃/导入失败、DPI/UIA、规模benchmark与平台lane。

## 3. 当前生产链与必须保留的基础

### 3.1 入口、恢复与document authority已经实质收束

内建asset registry现在为Sequence、Graph和State Machine直接注册toolkit route；测试不再依赖注入helper才能打开。route按canonical open operation选择`AnimationEditorDocumentKind`，binary envelope kind冲突返回typed diagnostic，恢复在metadata/toolkit后置失败时会移除session/core document并恢复metadata。

`AnimationAuthoringDocumentStore`以`DocumentId + AssetUri + kind + monotonic revision`持有唯一持久source。`AnimationEditorSession`只持document read handle和timeline/playback/selection等瞬态状态。持久修改先克隆asset并构造`AnimationEditCommand`，再由document history context执行CAS swap；undo/redo交换完整replacement并推进revision。`commit_after_apply`负责dirty和projection，后置失败会调用command rollback。这关闭了原先“先原地改session、再尝试登记dirty”的P0。

### 3.2 Compiler基础存在，但产品仍未消费结果

framework compiler已经覆盖三类source。Graph检查空/重复ID、output数量与引用、cycle、unreachable、parameter类型和有限数；Sequence检查duration/FPS、target、track重复、key范围/单调性、value/interpolation/tangent；State Machine检查state/transition/condition/layer/blend-space等结构。document为每个revision保存current product和last-good product。

但production搜索只发现document内部构造与测试读取`AnimationDocumentCompilation`；pane、save、preview和cook不读diagnostic/last-good。capability表反而继续把semantic compiler标成unavailable。Graph runtime会lower共享artifact并按显式evaluation order非递归求值，State Machine runtime也先调用共享compiler，这是正确方向；Sequence world compiler仍是另一入口。`zircon_plugins/animation_graph/editor`又有一套`Result<String, Vec<String>>` validator，只被自身测试消费，形成无owner的重复语义面。

### 3.3 UI foundation没有成为产品surface

新的`ui/timeline`、`ui/curve`、`ui/graph`和`ui/preview_scene`提供range/key/section/selection、curve view、typed port/connection/cycle、canvas state和preview lifecycle等基础。实际生产consumer搜索显示：timeline/curve只被animation session转换函数和测试读取；`GraphCanvasState`只在自己的测试出现；`PreviewSceneBackend`只有fake test实现。pane projection仍输出字符串数组，host body仍只是空timeline/canvas slot。

旧`animation_editor.zui`已删除，但`ui/layouts/views/animation_editor.rs`仍以绝对asset path加载并`unwrap_or_default`；`scene_projection.rs`仍调用该builder，integration contract仍直接读取缺失文件，template boundary仍要求该名字。这不是普通“样式缺失”，而是production fallback把owner删除错误伪装成合法空pane。

### 3.4 Save比旧实现可靠，但回执边界仍不完整

canonical save现在从document取得expected disk source，走conditional write和durable publication receipt，外部source冲突可typed返回，`import_asset`错误不再被吞掉，dirty save receipt只在callback成功后登记。这关闭了raw `fs::write`和忽略import错误。

不过`disk_source`在source write closure内先更新，而durability确认与import位于其后。若durable receipt或import失败，磁盘可能已经发布、session source baseline也已推进，但命令仍报失败且dirty未清。当前模型没有把“source published / directory durable / catalog imported / runtime artifact current”作为可恢复stage暴露给用户和retry owner。

## 4. Currentness状态总表

| 等级 | Open | Partial | Closed | 结论 |
|---|---:|---:|---:|---|
| P0 | 2 | 2 | 2 | document/transaction已收束；真实preview与缺失主ZUI仍阻断产品 |
| P1 | 44 | 16 | 5 | 原P1有5项关闭，新增5项全部Open |
| P2 | 12 | 0 | 0 | 大型团队、DCC、协作和可运维成熟度仍未建立 |

`Closed`表示源码责任链已被新authority替代，不表示全部动态发布门已通过；`Partial`表示存在可复用实现但finding的完整产品合同未闭环。canonical optimize总数不因currentness刷新重复增加。

## 5. P0：产品可达性、事务真实性与预览闭环

### E-ANIM-P0-01 · Closed · 默认产品无法打开动画资源

Sequence、Graph、State Machine已由builtin asset type registry声明toolkit route，并有默认bootstrap测试。Skeleton/Clip只读/可编辑行为仍缺失，但降为P1-01，不再使三种authoring document整体不可达。

### E-ANIM-P0-02 · Partial · 可见能力仍超过真实实现

固定frame-0 Scrub和无效State Add Node按钮已删除，Graph capability table也能拒绝未实现node kind。未关闭部分是8个高级动画工作区仍可导航，action继续只改变静态control/status并返回“queued”；compiler真实存在却被capability标为false。需要单一capability registry驱动导航、command、pane和自动化，未接asset/job/artifact/preview的入口必须隐藏、禁用或明确prototype。

### E-ANIM-P0-03 · Closed · 持久mutation绕过transaction/history

持久source已迁到core document store，全部当前Sequence/Graph/State Machine mutation通过expected revision的可逆command和document history context执行；transient scrub/selection/playback留在session且不标dirty。`commit_after_apply`失败会回滚command。后续性能与细粒度diff由P1-61追踪，不重开该正确性P0。

### E-ANIM-P0-04 · Partial · 共享semantic compiler未形成统一产品合同

三类共享compiler、stable element diagnostic和last-good compilation已存在，runtime Graph/State Machine也消费它后再lower。未关闭部分是save/preview/cook/pane不消费compiler result，capability truth落后，Sequence runtime另有world compiler，graph editor plugin另有字符串validator。必须收束为一个source compiler、一个versioned artifact合同和允许目标特化的lowering层。

### E-ANIM-P0-05 · Open · Preview不执行runtime动画

scrub/play仍只修改session瞬态字段；生产没有`PreviewSceneBackend`、subject/world、compiled artifact lease、clock tick、pose、root motion、event、active node/state或runtime diagnostic consumer。fake backend测试只能证明抽象生命周期。任何“playing/preview/apply”状态在真实gateway完成前都不能表示运行时结果成功。

### E-ANIM-P0-06 · Open · 已删除主ZUI仍被生产加载并静默降级为空pane

`zircon_editor/assets/ui/editor/animation_editor.zui`不存在，但production builder仍引用它并对加载失败`unwrap_or_default`，scene projection仍调用builder；integration contract直接读取缺失文件，template asset boundary仍要求该名字。必须选择唯一owner：若host Sequence/Graph body是正式surface，就删除旧loader、旧boundary和旧integration假设并为native slot装配真实typed pane；若独立layout仍是正式surface，则恢复非占位、可交互且有真实consumer的asset。禁止恢复一个空壳只让测试变绿。

## 6. P1：工程级完整性差距

### 6.1 Toolkit、lifecycle、save与diagnostic

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-ANIM-P1-01 | Open | Skeleton/Clip仍只有resource kind，没有声明一致的只读/编辑toolkit、preview、details与command capability。 |
| E-ANIM-P1-02 | Partial | 三类toolkit route已内建，但strategy、supported commands与pane composition仍由operation/view常量和host分支拼装。 |
| E-ANIM-P1-03 | Partial | Graph/State Machine同view的wrong-document gate已补齐；descriptor capability和pane composition仍无法按kind独立声明。 |
| E-ANIM-P1-04 | Closed | production restore按canonical route kind解码，不再以filename suffix决定document kind，并保留typed binary-kind mismatch。 |
| E-ANIM-P1-05 | Open | restore仍先`std::fs::read`完整文件，decode没有明确bytes/depth/count/allocation budget。 |
| E-ANIM-P1-06 | Closed | metadata sync或toolkit registration失败会回滚session、core document、toolkit与metadata。仍需更广的Nth-step测试。 |
| E-ANIM-P1-07 | Closed | canonical save已改为conditional durable source write，不再直接raw `fs::write`。stage receipt缺口由P1-63追踪。 |
| E-ANIM-P1-08 | Partial | dirty receipt延后到完整save callback成功；但source发布、`disk_source`更新、durability与import不是一个可恢复commit receipt。 |
| E-ANIM-P1-09 | Closed | `import_asset`错误不再被忽略，会使save返回失败。仍缺独立retry/job UX。 |
| E-ANIM-P1-10 | Open | autosave仍主要是source path与bytes，缺kind/schema/document revision/dependency revision/checksum和兼容恢复决策。 |
| E-ANIM-P1-11 | Partial | conditional write提供expected-source冲突检测；three-way merge、read-only/source-control checkout、rename与另存决策仍缺。 |
| E-ANIM-P1-12 | Partial | target/load/unavailable node已有typed code；transaction内部错误仍多次折叠为`UiAsset(error.to_string())`，部分无效操作仍只返回no-change。 |

### 6.2 Sequence、Timeline、Dope Sheet与Curve

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-ANIM-P1-13 | Open | track path仍不解析preview/project scene、component schema、property existence、writability与channel type。 |
| E-ANIM-P1-14 | Open | 新track仍固定Scalar(0)+Step，不能按Bool/Integer/Vector/Quaternion/Transform/离散属性初始化。 |
| E-ANIM-P1-15 | Open | 新binding `target_id`仍为None，缺stable entity/bone/component identity、rename remap与missing placeholder。 |
| E-ANIM-P1-16 | Open | 中间插key仍复制vector末尾key，不采样插入时间曲线或preview property。 |
| E-ANIM-P1-17 | Open | source key identity仍依赖seconds与`f32::EPSILON`，没有stable KeyId和timebase quantization。 |
| E-ANIM-P1-18 | Open | 没有typed key value/batch numeric/quaternion-euler/unit/color metadata与Inspector integration。 |
| E-ANIM-P1-19 | Open | move/scale/duplicate/copy/paste/cut/ripple/stretch/reverse/retime及drag transaction coalescing未实现。 |
| E-ANIM-P1-20 | Open | runtime有Linear/Hermite/tangent，产品没有插值模式、切线break/unify/weight与handle editing。 |
| E-ANIM-P1-21 | Open | authoritative session仍是单span选择；foundation选择模型未成为产品，多key/track与跨pane同步缺失。 |
| E-ANIM-P1-22 | Open | timeline range仍是瞬态状态，可与asset duration分离，超界key和duration extension policy未定义。 |
| E-ANIM-P1-23 | Open | scrub不采样channel或preview，不定义subframe/timecode/drop-frame与event suppression。 |
| E-ANIM-P1-24 | Open | playback无生产tick owner；zero/negative speed、reverse、loop boundary和event traversal规则未定义。 |
| E-ANIM-P1-25 | Partial | generic timeline有section/range/selection/snap基础；产品仍无完整hierarchy、lock/mute/solo/color/layer与大列表virtualization。 |
| E-ANIM-P1-26 | Open | event/notify/audio/montage/sync marker轨道与payload schema缺失，glTF仍固定空event tracks。 |
| E-ANIM-P1-27 | Partial | generic curve model和animation projection函数已出现；没有生产Curve Editor/dope切换、root motion、compression误差与source/compiled对比。 |

### 6.3 Graph、Parameter、Pin与Compiler

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-ANIM-P1-28 | Open | Editor node creation仍只覆盖Output/Blend，runtime Clip/Additive/Mask被明确标为unavailable。 |
| E-ANIM-P1-29 | Closed | 固定发送unsupported State的可见Add Node action已删除，typed resolver会拒绝unknown/unavailable kind。 |
| E-ANIM-P1-30 | Partial | shared compiler会拒绝空/重复ID；mutation与load admission仍未共享统一canonical ID/Unicode/reserved-name策略。 |
| E-ANIM-P1-31 | Partial | generic graph有typed port/connection/cycle validator，共享compiler有引用/cycle检查；production mutation仍按node ID猜连接且无pin/cardinality。 |
| E-ANIM-P1-32 | Partial | sharedcompiler已有cycle、topology与unreachable diagnostic；结果未进入pane，source/runtime debug map仍缺。 |
| E-ANIM-P1-33 | Open | Additive连接仍总写base，无法选择additive pin；Mask/Output也未使用typed pin identity。 |
| E-ANIM-P1-34 | Open | disconnect仍以from/to字符串批量移除，没有stable EdgeId、pin slot和edge metadata。 |
| E-ANIM-P1-35 | Partial | remove仍把必需引用清为空字符串；compiler能报告悬空，但mutation不阻止或以placeholder/edge command保持有效拓扑。 |
| E-ANIM-P1-36 | Open | parameter缺失时仍从文本猜Bool/Integer/Scalar/Vector，输入格式意外决定schema。 |
| E-ANIM-P1-37 | Open | parameter rename/remove/type/default/range/unit/category/tooltip/runtime exposure与引用迁移未实现。 |
| E-ANIM-P1-38 | Open | source不保存node position/size/comment/group/reroute/collapse/zoom/pan等authoring metadata。 |
| E-ANIM-P1-39 | Open | 没有owner-versioned node descriptor/pin registry、unknown placeholder、plugin lease、upgrade与opaque preservation。 |
| E-ANIM-P1-40 | Partial | document已有current/last-good compilation；仍是同步全量编译，缺dependency key、incremental compile、diagnostic pane、debug map与profiling overlay。 |

### 6.4 State Machine、Blend Space与高级工具

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-ANIM-P1-41 | Open | runtime支持Clip/BlendSpace1D/2D/SubMachine/GraphRef，Editor仍只能创建GraphRef state。 |
| E-ANIM-P1-42 | Open | state name只做弱重复检查，空名/canonical case/Unicode/reserved/rename/reference update合同缺失。 |
| E-ANIM-P1-43 | Open | 删除entry后仍按vector首项替换，没有用户决策、stable order与invalid-entry workflow。 |
| E-ANIM-P1-44 | Open | transition仍按from/to唯一并覆盖duration，不能表达同pair不同condition/priority的多边。 |
| E-ANIM-P1-45 | Open | transition frame换秒仍硬编码30 FPS，不服从asset/project display rate与tick resolution。 |
| E-ANIM-P1-46 | Open | exit time、interruption、priority、self transition、blend profile与sync policy不可完整编辑。 |
| E-ANIM-P1-47 | Partial | sharedcompiler会验证部分parameter/operator/value类型；mutation/UI仍不以typed condition schema限制输入。 |
| E-ANIM-P1-48 | Open | condition remove/reorder、AND/OR group、hysteresis、trigger consume与runtime debug缺失。 |
| E-ANIM-P1-49 | Open | layer、Override/Additive、weight与mask weights没有Editor authoring/validation/preview。 |
| E-ANIM-P1-50 | Open | 没有真实1D/2D sample增删、axis、triangulation、duplicate/outside-hull policy、analysis与weight preview。 |
| E-ANIM-P1-51 | Open | Blend Space工作区仍以硬编码条目和静态transport反馈为主，WeightHeatmap未绑定runtime weights。 |
| E-ANIM-P1-52 | Open | Sequencer/Montage/Pose Library/Retarget/Control Rig/Motion Matching/Compression仍无对应authoritative asset/job/artifact/preview闭环。 |

### 6.5 Product projection、import、性能与测试

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-ANIM-P1-53 | Partial | typed timeline/curve/graph model已出现；production pane仍把track/node/parameter/state/transition压成字符串数组。 |
| E-ANIM-P1-54 | Partial | 无效header action已删除且host保留typed slot；slot没有生产consumer，旧主layout删除又形成P0-06悬空引用。 |
| E-ANIM-P1-55 | Open | 没有真实viewport、skeleton tree、details、notifies、mesh/LOD/material、camera/floor/light与runtime debug pane。 |
| E-ANIM-P1-56 | Open | selection、canvas layout、timeline zoom/scroll、expanded tracks、preview subject与per-asset/user workspace无持久化/migration。 |
| E-ANIM-P1-57 | Open | animation compile/import/compression没有background job、cancel acknowledgement、progress、budget、DDC和large-document benchmark。 |
| E-ANIM-P1-58 | Open | glTF morph仍未进入track，event固定空；reimport dependency失效、selection remap、last-good preview与未保存冲突未闭环。 |
| E-ANIM-P1-59 | Partial | 新测试覆盖production registry、kind、transaction、undo和compiler；缺真实ZUI、preview parity、Nth-step save/import、malformed/large与plugin lifecycle。 |
| E-ANIM-P1-60 | Open | 大量文本仍硬编码英文，canvas/timeline缺完整keyboard/focus/UIA/screen reader/high contrast/reduced motion/i18n revision。 |
| E-ANIM-P1-61 | Open | 每次persistent edit克隆整份asset，history保存整份before/after，再在document write lock内同步全量编译；大序列/大图是O(asset)延迟和内存放大。需要element delta、copy-on-write arena、增量compiler、后台generation与明确budget。 |
| E-ANIM-P1-62 | Open | compiler truth有三处漂移：capability仍称semantic compiler不可用、pane/save不消费结果、graph editor plugin保留第二套字符串validator。必须指定framework compiler为唯一source semantic authority。 |
| E-ANIM-P1-63 | Open | source publication、directory durability、session `disk_source`、catalog import与runtime artifact没有统一stage receipt；部分成功会留下“磁盘已新、命令失败、document仍dirty”的不可解释状态。 |
| E-ANIM-P1-64 | Open | Timeline/Curve/Graph/Preview foundation没有生产host consumer；部分模型使用`String`/`f32` identity和线性visible filtering，未经10万key/1万node预算就可能成为第二套不可扩展抽象。 |
| E-ANIM-P1-65 | Open | runtime Graph/State Machine已走共享source compiler，Sequence world compiler仍独立，Editor plugin validator也分叉；需要source validate -> versioned artifact -> target lowering的单向收束与parity corpus。 |

## 7. P2：成熟度与大型团队工作流

| ID | 状态 | 当前差距 |
|---|---|---|
| E-ANIM-P2-01 | Open | 缺per-user timeline/grid/key/curve/node/preview视觉偏好。 |
| E-ANIM-P2-02 | Open | 缺viewport camera、timeline range、Graph位置与state debug bookmark。 |
| E-ANIM-P2-03 | Open | 缺近期asset、最近编辑element与跨document navigation history。 |
| E-ANIM-P2-04 | Open | 缺drag ghost、snap guide、edge auto-pan、minimap、overview ruler与selection breadcrumb。 |
| E-ANIM-P2-05 | Open | 缺key/curve/node/state批量标签、颜色、comment与review note。 |
| E-ANIM-P2-06 | Open | 缺animation revision结构diff、curve overlay与compiled artifact比较。 |
| E-ANIM-P2-07 | Open | 缺bounded compiler/debug support bundle导出。 |
| E-ANIM-P2-08 | Open | 缺authoring latency、compile cache hit与preview frame budget的opt-in telemetry。 |
| E-ANIM-P2-09 | Open | 缺批量rebind/compression/retarget与CLI dry-run报告。 |
| E-ANIM-P2-10 | Open | 缺DCC round-trip preset、source take/clip range与import recipe复用。 |
| E-ANIM-P2-11 | Open | 缺Editor scripting/remote automation的typed animation command/query surface。 |
| E-ANIM-P2-12 | Open | 缺多人asset lock、元素级冲突提示、review-only与协作审计。 |

## 8. 参考引擎逐源码对照

| 参考 | 逐源码确认的工程原则 | Zircon采用边界 |
|---|---|---|
| Unreal Animation Editor / Persona / AnimGraph | toolkit组合document、viewport、skeleton、details、asset browser与timeline；`AnimationEditorPreviewScene`持有真实preview world/controller；AnimDataController以transaction/bracket组织编辑；Graph Schema和AnimBlueprint Compiler分离交互合法性与编译执行数据。 | 吸收toolkit composition、controller transaction、schema/compiler/preview分层和source-to-runtime debug mapping；不复制UObject、Slate和Blueprint VM细节。 |
| Godot Animation Track/Bezier/Tree/State Machine | track/key/bezier/blend tree/state machine有专用surface；用户修改密集调用`EditorUndoRedoManager::create_action/add_do_method/add_undo_method`；AnimationPlayer负责seek/play结果。 | 吸收“每个用户动作可逆”、按轨道类型扩展编辑器、authoring与实际播放器连接；不照搬NodePath/ObjectDB。 |
| Fyrox Animation / ABSM | track、curve、signal、target binding以及ABSM state/transition/layer都有`execute/revert` command；preview进入时保存scene node state，离开或执行undo/redo前恢复，避免预览污染authoritative scene。 | 吸收细粒度command、preview rollback barrier、ABSM分层与property editor integration；避免把整asset clone当长期history格式。 |
| Bevy Animation | `AnimationGraph` source asset与`ThreadedAnimationGraph`执行结构分离，显式处理node index、sorted edges、computed masks、target ID、events和transition。 | 用作runtime artifact/lowering与数据导向执行参考；Bevy checkout不是Editor UX或durable save基准。 |
| Unity Graphics checkout | 只包含package内`InspectorCurveEditor`的局部curve wrapper/state/GUI惯例，不含Unity核心Animator/Animation/Timeline editor source。 | 仅吸收局部curve控件封装材料；不得用该checkout的缺失推断Unity完整动画能力，也不得把它当Zircon上限。 |

共同结论是：工程级动画创作不能由“能序列化一个asset + 若干静态pane”定义。authoring source、element transaction、schema registry、semantic compiler、versioned artifact、isolated preview和typed UI projection必须是单向数据流，并由同一capability truth对用户可见。

## 9. 重构路线

### M0 · 修复产品壳与能力真值

- 硬切`animation_editor.zui`悬空owner：不得以空asset恢复测试，必须让生产pane、template boundary和integration contract指向同一真实surface。
- 建立单一AnimationCapabilityRegistry，驱动asset toolkit、pane、command、导航、自动化和diagnostic。
- 将8个高级工作区逐项标为production或prototype；没有真实owner的从生产导航退出。
- 为Skeleton/Clip声明只读或可编辑toolkit，不再只注册resource kind。

### M1 · 唯一Schema/Compiler/Artifact链

- 指定framework animation compiler为唯一source semantic authority，删除或改造graph editor plugin重复validator。
- 为track/node/pin/state/parameter注册owner/version/schema/migration/unknown placeholder。
- 形成`source revision + dependency revisions + schema versions + target options`的artifact key和stable diagnostic map。
- Sequence/Graph/State Machine runtime均只消费共享artifact或明确的target lowering，不再重复解释source。

### M2 · 细粒度Document与Durable Save State Machine

- 保留现有document ID/revision/transaction/CAS，替换whole-asset replacement为stable element delta与结构共享。
- compiler移出document write lock，以generation fence后台编译；stale结果不得覆盖current/last-good。
- save显式建模source publish、durability、catalog import、artifact build与dirty acknowledgement，每步可重试和恢复。
- 完成autosave envelope、external edit three-way merge、read-only/source-control与Nth-step failure injection。

### M3 · Sequence / Timeline / Curve产品化

- typed property binding、stable TrackId/KeyId、正确channel初始化和schema-aware Inspector。
- multi-selection、move/scale/copy/paste、snap/timebase、interpolation/tangent、Dope Sheet/Curve切换。
- event/notify/audio/root motion/sync marker、hierarchy/mute/solo/lock与大列表virtualization。
- 将foundation变为host slot的唯一typed projection，建立10万key frame/memory benchmark。

### M4 · Graph / State Machine / Blend Space产品化

- 完成全runtime node/state kind、typed pins/edges、parameter lifecycle、canvas metadata与plugin migration。
- 完成transition priority/group/exit/interruption、layer/mask与nested machine authoring。
- 完成1D/2D sample/axis/triangulation/filter/weight heatmap，并与runtime结果golden对齐。
- 建立1万node/transition的incremental compile、canvas virtualization与diagnostic定位门。

### M5 · Isolated Runtime Preview

- 为`PreviewScene`实现production backend、world/subject lease、clock、artifact generation和安全close。
- scrub/play/pause/step/loop/reverse驱动真实pose、event、root motion与active graph/state snapshot。
- current source失败时明确显示last-good revision，不把旧preview伪装成当前source成功。
- 支持受控attach Play/PIE instance，并以session/generation fence阻止stale回调污染。

### M6 · Import、Cook与高级动画工具

- 补齐morph/event/import recipe、reimport dependency invalidation、selection remap与冲突解决。
- compile/compression/retarget/motion database进入有budget、cancel、progress和durable artifact的job系统。
- Montage/Sequencer/Pose Library/Control Rig/Motion Matching分别定义source schema、compiler/runtime consumer和preview验收。
- 通过source-vs-runtime/compiled质量、determinism与性能比较后再提升production capability。

### M7 · 发布资格

- 完成malformed/oversized/cross-version/plugin unload/crash/disk-full/import-failure corpus。
- 完成keyboard/focus/UIA/screen reader/i18n/high contrast/reduced motion。
- 建立十万key、大骨骼、1万node/transition、批量compile的可复现release benchmark。
- production bootstrap、transaction、compiler parity、preview parity、durability和平台lane同时通过后才宣称工程级Animation Editor。

## 10. 验收门当前状态

| Gate | 状态 | 验收要求与当前缺口 |
|---:|---|---|
| 1 | Partial | 默认bootstrap可打开Sequence/Graph/State Machine；Skeleton/Clip行为未声明完整。 |
| 2 | Partial | animation restore有回滚；尚缺所有route/resolve/projection第N步动态证明。 |
| 3 | Partial | canonical route与typed kind mismatch已实现；未运行完整malformed/live no-op动态门。 |
| 4 | Open | oversized/depth/count/allocation budget未建立。 |
| 5 | Partial | conditional durable write存在；完整stage failure与旧/新source原子性未证明。 |
| 6 | Partial | import错误会返回且dirty不清；retry job/notification/receipt缺失。 |
| 7 | Partial | 有expected-source CAS；merge、另存与autosave revision workflow缺失。 |
| 8 | Partial | 当前persistent mutation有transaction ID/history；需覆盖全部command和production dispatch。 |
| 9 | Partial | Sequence基础undo/redo存在；selection、dirty与完整bytes矩阵未闭环。 |
| 10 | Partial | Graph基础undo/redo存在；typed edge/layout/parameter全操作未覆盖。 |
| 11 | Partial | State基础undo/redo存在；layer与preview compile generation未闭环。 |
| 12 | Partial | `commit_after_apply`支持rollback；compile/projection/save所有Nth-step不变量未证明。 |
| 13 | Partial | shared compiler覆盖主要结构错误；unsupported schema与统一consumer仍缺。 |
| 14 | Partial | Graph cycle/reachability已覆盖；typed pin/cardinality与stable edge定位不足。 |
| 15 | Partial | State compiler覆盖主要模型；nested cycle与Editor可编辑字段未完整闭环。 |
| 16 | Partial | document有last-good；dependency revision与preview invalidation未实现。 |
| 17 | Open | track创建未解析真实property schema。 |
| 18 | Open | stable KeyId与完整multi-edit操作未实现。 |
| 19 | Open | tangent/quaternion authoring和runtime sampling golden未建立。 |
| 20 | Open | display rate/tick resolution/subframe/reverse语义未统一。 |
| 21 | Open | 没有生产runtime preview pose。 |
| 22 | Open | event/notify/root motion/sync marker预览规则未建立。 |
| 23 | Partial | generic Graph canvas foundation存在；未接生产且无大图budget。 |
| 24 | Open | runtime state kind/layer字段和active debug不能完整编辑/显示。 |
| 25 | Open | Blend Space authoring与runtime weight parity未建立。 |
| 26 | Open | 高级工作区仍未真实实现或退出生产导航。 |
| 27 | Open | plugin schema owner/lease/placeholder/migration未建立。 |
| 28 | Open | 十万key/1万node/大骨骼/批量compile budget与benchmark未建立。 |
| 29 | Open | morph/reimport/dependency/selection/冲突闭环未建立。 |
| 30 | Open | accessibility/i18n/high contrast/reduced motion未验收。 |
| 31 | Partial | production registry测试已出现；真实ZUI与fixture双矩阵被缺失asset阻断。 |
| 32 | Open | crash、磁盘满、plugin卸载、preview失败与跨版本failure injection未建立。 |

## 11. 非目标与边界

- 本报告不重写runtime evaluator；Runtime08C继续拥有pose/evaluation/GPU skinning性能，Editor负责消费同一artifact并显示真实结果。
- 本报告不要求复制Unreal Persona、Godot或Fyrox界面；只采用其transaction、schema/compiler、preview isolation和typed surface原则。
- 本报告不把静态test attribute inventory写成动态通过，也不因共享工作树在途状态降低验收门。
- 本轮不修改production、测试、asset schema、插件manifest或旧failure状态。

## 12. 完成定义

Animation Editor达到工程级资格，必须同时满足：production surface无缺失asset或静默空fallback；能力声明只暴露真实owner；所有持久修改细粒度可撤销且可扩展；一个共享compiler产生versioned artifact和stable diagnostic；save/cook/runtime/preview消费一致语义；Sequence/Graph/State Machine覆盖runtime模型；真实preview与runtime逐帧一致；import、插件、升级、崩溃、外部编辑和大资产均有budget与failure evidence。在这些事实成立前，当前实现应描述为“document/transaction/compiler基础已建立，但产品authoring与preview仍处于早期集成阶段”。
