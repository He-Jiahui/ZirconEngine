---
title: Editor Motion Matching、Pose Search、Database、Feature Schema、Trajectory Query、Runtime Selection、Preview、Debugger 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor200
review_date: 2026-08-28
baseline_head: e2d29a4a9cbbfc2c80067f3380212d6efd730361
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_motion_matching_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_editor/src/core/jobs/spec.rs
  - zircon_editor/src/core/jobs/admission.rs
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/core/jobs/system/submission.rs
  - zircon_editor/src/core/jobs/system/lifecycle.rs
  - zircon_editor/src/core/jobs/event_journal/journal.rs
  - zircon_runtime/src/core/framework/animation/compiler/mod.rs
  - zircon_runtime/src/core/framework/animation/compiler/product.rs
  - zircon_runtime/src/core/framework/animation/compiler/schema.rs
  - zircon_runtime/src/core/framework/animation/compiler/graph.rs
  - zircon_runtime/src/core/framework/animation/pose_snapshot.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph/types.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph/compile.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_graph/evaluate.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/core/jobs/system/submission.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/190-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/198-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchSchema.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchSchema.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchFeatureChannel.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchFeatureChannel.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchFeatureChannel_Trajectory.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchFeatureChannel_Trajectory.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchDatabase.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchDatabase.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchIndex.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchIndex.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchContext.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchContext.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchResult.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchResult.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchHistory.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchHistory.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchAssetIndexer.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchAssetIndexer.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchDerivedData.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchDerivedData.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/PoseSearchDerivedDataKey.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/PoseSearchDerivedDataKey.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/AnimNode_MotionMatching.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/AnimNode_MotionMatching.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Public/PoseSearch/Trace/PoseSearchTraceLogger.h
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Runtime/Private/Trace/PoseSearchTraceLogger.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchDatabaseEditor.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchDatabaseViewModel.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchDebugger.cpp
  - dev/UnrealEngine/Engine/Plugins/Animation/PoseSearch/Source/Editor/Private/PoseSearchDebuggerDatabaseView.cpp
  - dev/godot/scene/animation/animation_tree.h
  - dev/godot/scene/animation/animation_tree.cpp
  - dev/godot/scene/animation/animation_mixer.h
  - dev/godot/scene/animation/animation_mixer.cpp
  - dev/Fyrox/fyrox-animation/src/pose.rs
  - dev/Fyrox/fyrox-animation/src/machine/mod.rs
  - dev/Fyrox/fyrox-animation/src/machine/node/mod.rs
  - dev/Fyrox/editor/src/plugins/absm/command/mod.rs
  - dev/Fyrox/editor/src/plugins/absm/command/blend.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/ComputeDeformNode.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/LinearBlendSkinningNode.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/Operators/Implementations/SkinnedMeshRendererTransform.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/Slots/Implementations/VFXSlotSkinnedMeshRenderer.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/79-editor-motion-matching-pose-search-database-feature-schema-trajectory-query-runtime-selection-preview-debugger-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/79-editor-motion-matching-pose-search-database-feature-schema-trajectory-query-runtime-selection-preview-debugger-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Motion Matching、Pose Search、Database、Feature Schema、Trajectory Query、Runtime Selection、Preview、Debugger 与 Product Integration 当前源码复核

## 1. 结论

当前 Zircon 仍没有 Motion Matching 产品，也没有 Pose Search 运行时平台。唯一专用产品表面仍是 230 行 `workbench_extension_motion_matching_workspace.zui`：`MM_Locomotion / MM_Combat / MM_Traversal`、`Idle_Breath`、`184 clips`、`1 warning`、三个 trajectory option 与 `Cost Bias: 0.42`全部写死。Preview、Rebuild、Idle 和 Pivot action 最终只把固定文本写入状态栏与 output row；field edit 最多打开通用 dropdown，field commit 没有 database mutation、transaction、build request 或 receipt。

对 `zircon_runtime`、`zircon_plugins`、`zircon_app` 与 `zircon_interfaces` 的 tracked production 精确领域检索为 0；另扫描 1,505 个未跟踪 Rust/ZUI 文件，同样为 0。当前 shared Animation compiler 的 source/product 只有 Sequence、Graph、StateMachine，插件 `CompiledGraphNode` 只有 Clip、Blend、Additive、Mask。`AnimationPoseSnapshot` 只是 sealed pose 输出，不是带 time domain、trajectory、teleport epoch 与 source generation 的搜索历史。数据库 source、feature schema、offline indexer、prepared index、query builder、search result、runtime instance、graph operation、trace 与 cook 均无生产类型或调用链。

旧 Editor79 只有一项需要正向校正：通用 `EditorJobSystem` 已具 estimated byte、pending age、admission key、keyed merge、batch reservation、cooperative cancellation、progress snapshot、bounded journal 与 deadline shutdown，因此 Pose Search Rebuild 的任务承载底座不再是完全空白。它没有 `PoseSearchBuildJob`、依赖 build key、领域 artifact、LKG/CAS publication、cook/install，也尚未通过 execution scope 实现 quiescent shutdown；故 ED200-P1-06 与 MM-15 只能标为 Partial，不能据此把固定 `Rebuild queued` 视为真实工作。

本轮不新增父报告 P0。Editor14 的静态成功界面 P0、Editor131 的 shutdown quiescence P0、Editor197 的 compiler/runtime authority、Editor198 的 Clip/prepared/playback、Runtime08C 的 pose/trajectory/search 大类仍由各 canonical owner 持有。Editor79 的 15 项 P1 当前重判为 **14 Open / 1 Partial / 0 Closed**，5 项 P2 为 **5 Open / 0 Partial / 0 Closed**；48 个资格门为 **47 Fail / 1 Partial / 0 Pass**。canonical finding 总数不重复增加。

本轮只做静态 current-source review 与文档建账，不修改 production Rust/ZUI，不运行 Cargo、Editor、GUI/GPU、database build、cook、runtime query、preview、trace、fault/soak/profile 或同语义跨引擎 benchmark。不能据此宣称功能可用、性能达标，更不能宣称性能或表现超过 Unreal。Tooling 按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

## 2. 审查边界、owner 与冻结语料

### 2.1 本报告唯一纵向边界

本报告拥有：`MotionMatchingDatabaseSource -> PoseFeatureSchema -> PoseSearchBuildPlan/Job -> PreparedPoseSearchDatabase -> MotionQuerySnapshot -> PoseSearchRuntime -> MotionMatchingSelectionReceipt -> CompiledAnimationProgram -> AnimationPlaybackTransaction -> Editor Preview/Debugger`。

- Editor14 拥有默认 Animation toolkit、静态成功措辞与通用 preview/save/compile 真实性 P0。
- Editor131 拥有 background job admission、scope、cancel、progress、shutdown 与 durable result 总合同；本报告只定义 Pose Search typed build job/product。
- Editor32 拥有 Skeleton、Skin、Mirror、Retarget 与 import/reimport artifact identity。
- Editor184 拥有 document transaction/history/savepoint/async operation 总合同。
- Editor190 拥有 PreviewWorld、time domain、pause/step 与 subsystem admission。
- Editor196 拥有 Timeline/Curve/transport；本报告的数据库预览只消费该时间合同。
- Editor197 拥有唯一 Animation compiler/runtime authority；Motion Matching 必须成为其 typed operation。
- Editor198 拥有 canonical Sequence/Clip、event、root motion、sync、prepared artifact 与 playback transaction。
- Runtime08C 拥有 pose/skeleton 调度、root trajectory 与 Motion Matching 平台大类；本报告不换名重复登记。

### 2.2 Currentness 与负证据

- HEAD 锚点为 `e2d29a4a9cbbfc2c80067f3380212d6efd730361`；结论以 2026-08-28 当前磁盘内容为准。审查期间 HEAD 前进未触及本报告冻结路径。
- 工作树已有大量共享在途和未跟踪文件；本轮不回退、不覆盖、不归属这些变化。
- tracked production 专用词检索覆盖 MotionMatching、Motion Matching、PoseSearch、Pose Search、PoseFeature、FeatureSchema、TrajectoryQuery 与 PoseHistory，四个生产根目录返回 0。
- 1,505 个未跟踪 production Rust/ZUI 文件的同一检索也返回 0；Editor 之外的 broad `Trajectory/root_motion/history` 命中属于已有动画或图形 frame-history 边界，不构成 Pose Search。
- `workbench_extension_panel_field_action` 当前只有定义和局部测试两个调用点；真正 dispatch 使用另一套通用 dropdown 判定，不存在 Motion Matching domain controller。

### 2.3 冻结语料与 fingerprint

统计口径：相对路径转小写、`/` 规范化并排序；每个文件取 SHA-256，再按 `path + NUL + lowercase file hash + LF` 聚合集合 fingerprint。行数与非空行按当前文本计算。

| 范围 | 文件 / 行 / 非空行 / bytes | 本轮证据 | fingerprint |
|---|---:|---|---|
| Zircon Editor/product | **12 / 5,956 / 5,688 / 281,614** | 固定 ZUI、binding、route、feedback 与静态 action inventory | `3a178e2ccd0112fbeb61879c139615670614ed2c743601d3facd7e40163f5da2` |
| Zircon shared prerequisite foundation | **14 / 3,395 / 3,064 / 115,042** | 通用 jobs、shared compiler、compiled graph 与 pose snapshot | `55f4bc07655fd35f2e99ad447165bc8d88b60e2c4270e8d1cbc5610bcde1fb66` |
| Zircon deduplicated focused set | **26 / 9,351 / 8,752 / 396,656** | 上述两组按 normalized path 去重 | `1dd8e2619aafa27eb17e8b5e4ae1b27200ccb667f9526da897b6e6d943b9a3e4` |
| Unreal selected set | **30 / 21,271 / 18,004 / 852,509** | Schema/channel/database/index/history/build/search/node/trace/editor/debugger | `74de5732d958f497983247069490a003243c479ce9abf61bec8784a91a526925` |
| Godot selected set | **4 / 4,868 / 4,159 / 181,576** | AnimationTree 与 Mixer/root-motion 边界 | `8a4732682e2b1ccc891a43caf1e013c72444b6a0a517c23e6dfbf2af24a1f66b` |
| Fyrox selected set | **5 / 1,828 / 1,633 / 67,123** | typed pose/machine 与 reversible ABSM command | `1b2c8ba330ad2ef0e05e41aaf0a5a7e0acd8b4d5402a2f96a86ea0298f611dab` |
| Bevy selected set | **3 / 2,979 / 2,692 / 113,433** | serialized/prepared graph、player 与 transition | `f8835c768eb054f0c2d1d8fbebd4c80e93c23a365f579e02cb636246bec9ee1a` |
| Unity Graphics selected set | **4 / 458 / 414 / 21,159** | deformation/skinning 与 VFX consumer | `f087df51937d0fb6a582d70655f297e60c15d36fecffda8b84d08132b7726bde` |
| Five-engine reference total | **46 / 31,404 / 26,902 / 1,235,800** | frontmatter 显式路径去重 | `7960a11b4bf9a03eedf6a0cab6d992455f5593da91db22f2b8fc809d5775eaf1` |

fingerprint 只是本轮静态输入 receipt，不是未来数据库 build key、asset revision 或运行时 generation。

## 3. 当前真实实现与旧报告校正

### 3.1 可见 workspace 仍是固定 projection

ZUI 第 79/87/95 行固定三个 database，152 行固定 `Idle_Breath / cost 0.04`，184 行固定 `184 clips / 1 warning`，206/217/228 行固定 database、trajectory 与 Cost Bias。这里没有 AssetId、DocumentId、SchemaId、EntryId、source revision、build generation、query frame 或 selected pose address。

feedback 第 327-353 行把 Open、Preview、Rebuild、Idle 与 Pivot 映射为固定文本。`apply_workbench_extension_action` 只做 workspace/tab/row 选择、dropdown 与 feedback；`.edit` 仅在有 options 时开 popup，`.commit` 不进行领域 mutation。静态 action 测试证明模板 wiring 存在，不证明数据库、查询或预览执行。

### 3.2 Shared Animation compiler 不是 Motion Matching compiler

`AnimationCompileSource` / `AnimationCompileProduct` 只覆盖 Sequence、Graph、StateMachine；插件 runtime 的 `CompiledGraphNode` 只覆盖 Clip、Blend、Additive、Mask，evaluation 输出 clip instance。当前没有 MotionMatching source/product/node、prepared database handle、query operation、instance state、selection receipt 或 playback handoff。

共享 compiler、dense graph 和 sealed pose snapshot 是未来可复用的 authority 边界，但没有领域 variant/consumer 时只能作为依赖，不得把它们计作 Pose Search 的 Partial 实现。

### 3.3 Background Jobs 是唯一新增的 Partial 底座

`EditorJobSpec` 已有 mutex group、cancellation token、estimated bytes、admission key、max pending age 与排序去重 dependency；submission 支持 keyed merge、batch all-or-nothing、reservation 与 current cancellation authority；lifecycle 支持 pending/active cancel、progress、deadline shutdown 与 unfinished report。

但 job descriptor 没有 project/document/plugin/scope generation，Editor task 不进入 Runtime `ExecutionScope`，deadline shutdown 不能 join/reap executor 或阻止 late commit。更没有 Pose Search build key、dependency manifest、typed result、artifact validator、LKG、CAS publication 或 cook。因此只校正 ED200-P1-06/MM-15，不改变其余领域差距。

### 3.4 父 owner 与不重复计数

| 既有 owner | 当前事实 | 本轮处理 |
|---|---|---|
| Editor14 | 固定成功 workspace 对外声明不存在能力 | 保持父 P0，不重复登记 |
| Editor131 | 通用 admission/cancel/progress 已出现；quiescent shutdown 仍 Open | 作为 build substrate；P1-06 Partial |
| Editor197 | shared Graph/State compiler/runtime 已出现，无 Motion Matching variant | 复用唯一 authority，不建立第二 compiler |
| Editor198 | Sequence source/compiler/clip cache 有进展，root motion/sync/playback 仍不完整 | 数据库 entry 只引用其 canonical artifact |
| Runtime08C | trajectory/search/budget/fallback 平台总账仍 Open | 本报告展开可验收产品链，不换名重报 |

## 4. P1：Motion Matching / Pose Search 生产差距

### ED200-P1-01 · Open · 没有 canonical source、稳定身份、版本、revision 与依赖闭包

当前 database 身份只是 `MM_Locomotion` 等显示字符串。目标建立 versioned `MotionMatchingDatabaseSource`，至少拥有 `DatabaseAssetId + DocumentId + SourceRevision + SkeletonArtifactId + FeatureSchemaId + EntryId`，并定义 load/save/migration/reimport、compiled generation 与 last-good 关系。显示名、control ID 与当前 focus 不得参与寻址。

### ED200-P1-02 · Open · 没有 typed Feature Schema、channel registry、layout 与兼容规则

不存在 pose/trajectory/curve/phase/event channel、bone/role/reference frame、sample offset、cardinality、unit、layout offset、channel version 或 missing-data policy。目标让每个 channel 声明输入、坐标系、时间域、维度、单位、offline/runtime kernel、debug label 与 migration；Schema finalize 后生成唯一不可变 `PoseFeatureLayout`。

### ED200-P1-03 · Open · weight、normalization 与 cost 不是可审计数学合同

固定 `Cost Bias: 0.42` 与 0.04/0.18/0.31 没有数据来源。目标把 raw feature、normalization group/statistics、sqrt weight、metric、bias 与 cost decomposition 固化进 artifact/receipt；finite、nonnegative、zero variance、cross-database scale 与 schema drift必须 fail-close。

### ED200-P1-04 · Open · entry 缺 source clip、采样区间、mirror/loop 与 provenance

缺 stable entry ID、clip artifact generation、sampling interval、exclude head/tail、loop/mirror/permutation、tags、enablement、reselection/transition policy。每个 indexed pose 必须可逆映射到 entry、source asset、time、mirror/permutation 与 build generation。

### ED200-P1-05 · Open · 没有唯一确定性的 offline extractor 与 build/query 等价规则

仓内没有 asset indexer 或共享 feature kernel。目标用同一 channel 定义驱动 offline `IndexAsset` 和 runtime `BuildQuery`，冻结 time quantization、boundary/extrapolation、root/reference transform、mirror、curve 与 floating-point policy，并以 golden vector 验证重复 build 与跨目标一致性。

### ED200-P1-06 · Partial · 通用 job substrate 可复用，但 Pose Search build/product/publication 仍为空

通用 jobs 已覆盖 admission、estimated bytes、max pending age、keyed merge、batch reservation、cooperative cancel、progress、bounded event journal 与 deadline shutdown。缺失项仍包括 `PoseSearchBuildJob`、source/schema/clip/skeleton/mirror/compiler/target-platform build key、transitive invalidation、domain coalesce、typed diagnostics/result、immutable artifact、LKG、CAS publication、cook/install 与 quiescent shutdown。取消/失败不得覆盖 last-good，late task 不得发布过期 generation。

### ED200-P1-07 · Open · 没有自包含 Prepared Database 与 load validator

不存在 dense feature table、pose metadata、pose-to-source map、normalization、backend payload、alignment、endianness、version、checksum、memory estimate 或 load validator。artifact 必须在不读取 Editor source/Vec index/对象地址的情况下完成运行时搜索，并验证 layout、range、finite、tree/index、dependency generation 与预算。

### ED200-P1-08 · Open · 没有 qualified PoseHistory 与 Trajectory Query ABI

`Forward 0.8s` 只是字符串。目标建立携带 world/entity/frame/time domain、history/current/future samples、timestamp、coordinate frame、position/facing/velocity、confidence、controller generation、teleport/cut/reset epoch 的 `MotionQuerySnapshot`；缺数据政策必须 typed 且可回放。

### ED200-P1-09 · Open · runtime query builder 没有 schema-driven kernel、cache 与失败语义

不存在 schema-driven query vector、bone compatibility、history interpolation、trajectory sampling、normalization、bounded scratch 或 channel cache。缺骨、历史不足、non-finite、teleport、stale artifact 必须产生 typed disposition，不能静默填零或让每个 node 自己拼 `Vec<f32>`。

### ED200-P1-10 · Open · 没有 search continuity、filter、tie、budget、fallback 与解释 receipt

不存在 candidate filter、continuing pose、jump threshold、reselect history、block transition、cost addend、early-out、deterministic tie、deadline/shortlist 或 fallback graph。先建立 exact bounded oracle；receipt 必须记录 query/database generation、候选/过滤数、选中 pose address/time、cost decomposition、continuing/new、elapsed/budget 与 fallback 原因。

### ED200-P1-11 · Open · 没有 runtime instance、Animation Program operation 与 atomic playback handoff

没有 per-character state、elapsed search time、selected asset/time、play rate、blend/interrupt、reset-on-relevance、selection history、root motion/event/sync handoff 或 completion。Motion Matching 必须成为唯一 compiled Animation Program 的 typed operation，由单一 Animation Runtime 持有实例状态并通过 canonical playback transaction 原子提交。

### ED200-P1-12 · Open · 多数据库搜索没有 schema/normalization 可比性与结果合并合同

Locomotion/Combat/Traversal 只是三个静态名字。目标显式编译 search set；不可比较 schema 必须拒绝或经声明的 projection，跨库 cost 必须同尺度，稳定 tie key 记录每库 shortlist、bias 与 skip reason，query cache 只能在兼容 generation 间共享。

### ED200-P1-13 · Open · Editor 没有真实 toolkit、transactional tree、details、statistics 与 diagnostics

缺 asset create/open/save、entry add/remove/reorder、drag/drop、sampling range、schema/channel editor、dependency browser、statistics、build status、invalid pose navigation 与 undo/redo。所有编辑进入 qualified document transaction；UI 只投影 document/build snapshot，不保留第二份 domain truth。

### ED200-P1-14 · Open · Preview/Debugger 没有 runtime-backed generation、query/candidate 与 replay trace

Preview 只写固定文本，Debug tab 没有运行数据。目标让 PreviewWorld 与 runtime evaluator 消费同一 prepared generation；bounded/versioned trace 记录 query、candidate flags/cost、selection 与 playback handoff，Debugger 按 frame/node/database 显示 cost breakdown、reject reason、trajectory/history、timing 并定位 source pose。

### ED200-P1-15 · Open · 没有 determinism、correctness、fault、quality 与 performance 资格体系

现有测试只覆盖通用 action/hash route 和 jobs substrate。缺 schema migration、feature golden、index determinism、exact-search oracle、continuity、teleport/reset、cancel/publication、corrupt artifact、float drift、1/100/1k 角色与 quality/performance 曲线。任何加速 backend 都必须与 exact oracle 比较 recall、cost regret 与选择稳定性。

## 5. P2：规模、交互与超越目标

### ED200-P2-01 · Open · shard、streaming、residency、prefetch 与 hot-swap 未建立

基础 artifact/budget 闭合后，按 gameplay set 分片并建立 resident generation、prefetch、memory pressure、eviction、miss fallback 与无停顿 hot-swap；animation update 热路径不得同步 I/O。

### ED200-P2-02 · Open · multi-role interaction、同步 selection 与 warping 未建立

多角色交互需要 role-qualified skeleton/history、共同 candidate、availability、root alignment、原子多实例 commit、网络 authority 与失败 rollback，不是并排运行多个单人查询。

### ED200-P2-03 · Open · SIMD/PCA/KD/VPTree/ANN/GPU 缺 quality-bounded backend 合同

加速后端必须替换 prepared payload 而不替换语义，保留 exact oracle、deterministic mode、recall/regret budget、fallback 与 backend telemetry；不得通过减少 feature、pose 或搜索频率制造速度优势。

### ED200-P2-04 · Open · 大规模 authoring 的 virtualization、统计、outlier 与 diff/merge 缺失

数万 clip/百万 pose 需要分页 tree、批量 tag/range/mirror rule、feature distribution、outlier、coverage、duplicate/prune explanation、artifact diff 与 multi-user conflict。不能把所有 pose 物化成 retained row。

### ED200-P2-05 · Open · 自动调参与同语义质量/性能实验室缺失

必须固定输入录制、内容、硬件、线程、数据库 pose 数、query 维度、搜索频率与质量目标，同时衡量 foot sliding、trajectory error、transition pop、recall/regret、CPU/memory/latency，才能比较 schema/weight/backend Pareto 前沿。没有这些证据不得宣传“超过 Unreal”。

## 6. 五套参考源码裁决

### 6.1 Unreal：主参考，不复制历史包袱

Unreal PoseSearch 将 source/schema/channel、offline index、derived-data key/cache、runtime search/node、trace 与 Editor toolkit/debugger 分层。Schema/FeatureChannel 明确 `Finalize / BuildQuery / FillWeights / IndexAsset`；Trajectory channel 结构化 time offset、position/velocity/facing 与 subchannels。DerivedData 的 New/Continue/Wait、cancel previous request、BLAKE3 partial key/dependency、cook tick 与 determinism stress test证明 Rebuild 不是按钮反馈。

`AnimNode_MotionMatching` 从 qualified PoseHistory 构造 search context，处理多 database、interrupt、search throttle、continuing pose、reselect history、play rate、blend 与 root-motion/interaction。Rewind debugger读取 trace timeline，展示 best/brute-force cost；database view按 channel cardinality重建 cost breakdown、candidate flags、pose/source/time/mirror。Zircon应学习责任边界与证据链，不复制 UObject、宏、默认参数或实验 API。

### 6.2 Godot：AnimationTree 与 root-motion handoff 边界

选定源码没有一等 Motion Matching/Pose Search。AnimationTree/StateMachine 提供 typed playback/travel，AnimationMixer 提供 root-motion track、local policy 与 position/rotation/scale delta。它只证明 selection 必须进入正式 animation/root-motion pipeline，不提供 feature database/search 设计依据。

### 6.3 Fyrox：typed pose、machine 与 reversible authoring

选定源码没有一等 Motion Matching/Pose Search。`AnimationPose`/machine 定义 pose ownership/evaluation，ABSM command 提供 execute/revert。它适合交叉校验 selection handoff 与 Editor transaction，不替代 Unreal 的 offline database、query 或 budget 架构。

### 6.4 Bevy：source/prepared graph 与 transition 边界

`SerializedAnimationGraph`、asset loader 与 `ThreadedAnimationGraph` 分离，asset event 后重建 prepared traversal/mask；transition component拥有主动画与 fade-out 集合。这支持 source 不直接作为每帧执行结构、selection 进入正式 player/transition，但不提供 Pose Search 算法证据。

### 6.5 Unity Graphics：deformation consumer 边界

选定 ShaderGraph/VFX 源码只消费 position/normal/tangent、bone matrix/weight 与 SkinnedMeshRenderer/root transform，没有 Pose Search 平台。它只能约束最终 pose/current-previous history稳定进入 deformation/motion-vector consumer，不能反向定义数据库、trajectory 或 search。

## 7. 目标架构与唯一 authority

```text
MotionMatchingDatabaseSource + PoseFeatureSchema
  -> PoseSearchBuildPlan / EditorJobAuthority
  -> shared PoseFeatureCompiler + exact oracle
  -> immutable PreparedPoseSearchDatabase + dependency manifest
  -> generation-qualified install / last-good
  -> MotionQuerySnapshot + PoseHistory + Trajectory
  -> bounded PoseSearchRuntime
  -> MotionMatchingSelectionReceipt
  -> CompiledAnimationProgram / AnimationPlaybackTransaction
  -> Pose + RootMotion + Event + Sync
  -> Editor Toolkit / PreviewWorld / Rewind Debugger consume the same generation
```

必须只有一份 Feature Schema、channel kernel、prepared artifact validator、runtime search authority 与 playback bridge。`zircon_editor` 拥有 document/toolkit/job orchestration/preview projection，不拥有运行时搜索数学副本；animation runtime owner拥有 shared kernel、artifact、query/search/instance；renderer只消费已发布 pose。

必须硬切以下路径：真实 capability 成立前隐藏或标记 Unavailable 当前 workspace；删除固定 queued/clip/warning/cost 成功反馈；禁止以显示文本/control ID/focus寻址；禁止帧内全 clip 扫描、同步 source load/build；禁止 offline/runtime 两份 feature 数学；禁止失败覆盖 LKG、stale generation playback 与只报平均耗时的 benchmark。

## 8. 重构里程碑

### ED200-M0 · Capability truth、owner、RED corpus 与 benchmark protocol

先把 workspace 降为 Unavailable/Prototype，冻结 owner、内容 corpus、reference revision、quality metrics 与 hardware/workload protocol；加入“无真实 artifact 不得显示 Rebuild/Preview 成功”的 RED contract。

### ED200-M1 · Stable source、entry 与 Feature Schema

建立 IDs、version/migration、typed channels/layout、Skeleton/Clip/Mirror dependency 与 transactional document。

### ED200-M2 · Deterministic sampler 与 offline compiler

实现统一 time/space/mirror/extrapolation kernel、entry expansion、feature golden、normalization、provenance 与 exact table oracle。

### ED200-M3 · Build key、job、artifact、LKG/CAS 与 cook

接入 EditorJobAuthority，完成 scope/dependency key、cancel/coalesce、typed result、validator、last-good CAS、target cook 与 generation install；先解决 Editor131 quiescent shutdown/late commit gate。

### ED200-M4 · PoseHistory、Trajectory 与 query ABI

建立 qualified snapshot、teleport/reset/cut epoch、schema-driven builder、bounded scratch/cache 与 typed invalid disposition。

### ED200-M5 · Exact search、continuity、budget、fallback 与 receipt

完成 filter、continuing pose、jump/reselect、tie、deadline/early-out、fallback graph 与完整 cost/result receipt。

### ED200-M6 · Runtime instance 与 atomic playback

把 Motion Matching 编译为 Animation Program operation，闭合 throttle、interrupt、play rate、blend、sync/event/root-motion 与 generation-safe handoff。

### ED200-M7 · 真实 toolkit、preview 与 debugger

实现 asset tree/schema/details/statistics、diagnostic navigation、PreviewWorld、query/pose/trajectory draw、candidate cost table 与 rewind trace。

### ED200-M8 · Scale、streaming 与加速 backend

exact oracle 通过后再加入 SIMD/PCA/KD/VPTree/ANN、shard/residency/prefetch/hot-swap、virtualized Editor 与 quality-bounded telemetry。

### ED200-M9 · Fault、soak、profile 与同语义跨引擎资格

覆盖 corrupt/stale/cancel/reload/teleport/stream miss，建立 1/100/1k 角色与 pose/dimension/frequency 曲线，固定质量和硬件对比 Unreal；未通过不得宣称超越。

## 9. 48 个资格门

| Gate | 资格 | 当前 |
|---|---|---|
| MM-01 | Database/Schema/Entry 有 stable ID、version 与 migration | Fail |
| MM-02 | source revision 与 qualified document target 可验证 | Fail |
| MM-03 | Skeleton/Clip/Mirror dependency 使用稳定 artifact identity | Fail |
| MM-04 | feature channel registry typed、可扩展且有 owner/version | Fail |
| MM-05 | channel 声明 time/space/unit/cardinality/missing policy | Fail |
| MM-06 | schema finalize 产生唯一 layout 与 compatibility result | Fail |
| MM-07 | weight/normalization/metric/bias 数学可审计 | Fail |
| MM-08 | source save/reopen/migration/reimport 无损且 transactional | Fail |
| MM-09 | entry interval/loop/mirror/permutation 语义完整 | Fail |
| MM-10 | pose 可逆映射到 entry/source/time/mirror/generation | Fail |
| MM-11 | offline sampler 的 time/space/extrapolation policy 唯一 | Fail |
| MM-12 | offline IndexAsset 与 runtime BuildQuery 共享 channel kernel | Fail |
| MM-13 | feature vector 重复 build 得到 golden 一致结果 | Fail |
| MM-14 | build key 覆盖 source 与全部 transitive dependencies/version | Fail |
| MM-15 | build job 支持 admission/cancel/coalesce/progress/shutdown | Partial |
| MM-16 | failed/cancelled build 保留 LKG 并原子 CAS publication | Fail |
| MM-17 | prepared artifact 自包含、versioned、checksummed、cooked | Fail |
| MM-18 | artifact load 验证 layout/range/finite/tree/dependency | Fail |
| MM-19 | artifact 给出 memory/pose/dimension/backend statistics | Fail |
| MM-20 | source 变更只失效正确 database 且无 stale install | Fail |
| MM-21 | PoseHistory 定义 frequency/capacity/space/interpolation | Fail |
| MM-22 | Trajectory 有 history/current/future typed samples/timestamps | Fail |
| MM-23 | query 携带 world/entity/frame/time-domain/source generation | Fail |
| MM-24 | teleport/cut/reset/missing/non-finite 有 typed disposition | Fail |
| MM-25 | query builder 由 prepared schema 驱动且 scratch 有界复用 | Fail |
| MM-26 | continuing pose 与 current pose 选择 policy 明确 | Fail |
| MM-27 | 多 database schema/normalization 可比性被验证 | Fail |
| MM-28 | query/cache 不读 Editor source、focus 或隐式全局状态 | Fail |
| MM-29 | exact search 是 correctness oracle 且 tie-break 确定 | Fail |
| MM-30 | filters/continuity/jump/reselect/block-transition 正确 | Fail |
| MM-31 | per-instance deadline/candidate/shortlist/early-out 有预算 | Fail |
| MM-32 | budget miss/index unavailable 有 typed fallback graph | Fail |
| MM-33 | result 记录 database/pose/source/time/generation | Fail |
| MM-34 | result 记录 per-channel cost/bias/reject reason | Fail |
| MM-35 | stale query/artifact/result 在 playback 前 fail-close | Fail |
| MM-36 | selection receipt 可供 runtime/debugger/trace/replay 共用 | Fail |
| MM-37 | Motion Matching 是唯一 Animation Program 的 typed operation | Fail |
| MM-38 | instance 处理 relevance/reset/throttle/interrupt/history | Fail |
| MM-39 | selected pose 原子进入 blend/play rate/sync/event | Fail |
| MM-40 | root motion 与 movement trajectory 无双 writer/反馈环 | Fail |
| MM-41 | Editor toolkit 可真实 create/open/save/add/remove/reorder | Fail |
| MM-42 | schema/entry/details/statistics/diagnostics 来自真实 snapshot | Fail |
| MM-43 | preview 消费同一 prepared artifact/runtime evaluator | Fail |
| MM-44 | debugger 显示 query/pose/trajectory/cost/candidate/timing | Fail |
| MM-45 | trace bounded、versioned、frame-qualified 且可 rewind | Fail |
| MM-46 | determinism/property/golden/fault/soak 矩阵动态通过 | Fail |
| MM-47 | 1/100/1k 角色与 pose/dimension/frequency 预算通过 | Fail |
| MM-48 | 同质量/内容/硬件跨引擎证据支持性能与表现声明 | Fail |

## 10. 实施顺序、停止条件与复核边界

实施顺序必须先完成 M0-M7 的 source/build/query/runtime/product 闭环，再进入 M8/M9。P2 不反向阻塞 P1；但 Editor131 的 scope/quiescence、Editor197 的唯一 compiler/runtime authority、Editor198 的 stable clip/playback、Runtime08C 的 pose/trajectory ownership 是 M3-M6 的硬依赖。

出现第二份 Feature Schema、第二份 extractor/query math、显示文本寻址、frame 内全 clip 扫描、同步 load/build、失败覆盖 LKG、stale generation commit、无 typed fallback 或通过降低质量赢 benchmark 时，必须停止并回到 owner/architecture 裁决。

`review_status: current_source_refresh_complete` 只表示静态取证、差距重判和路线建账完成。实施前必须重取 HEAD、focused diff、父 owner 状态、专用词命中、animation runtime/compiler 装配、Clip/Skeleton/RootMotion/PreviewWorld/Jobs 合同与参考源码 fingerprint。Cargo、Editor、真实 build/query/preview、fault/soak/profile 均未运行；48 门没有任何 Pass。
