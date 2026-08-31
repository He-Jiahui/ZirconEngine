---
title: Editor Animation Pose Library、Pose Asset、Pose Name、Curve Weight、Additive Base、Runtime Evaluation、Preview 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor81
review_date: 2026-08-23
baseline_head: 9b5564b749e618475e258dd75bd3d9b34e9388a9
baseline_epoch: 358
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_runtime/src/core/framework/animation/asset/clip.rs
  - zircon_runtime/src/core/framework/animation/asset/graph.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_plugins/animation/runtime/src/manager/pose.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_pool.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/78-editor-control-rig-rig-graph-hierarchy-controls-spaces-constraints-ik-solve-bake-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/79-editor-motion-matching-pose-search-database-feature-schema-trajectory-query-runtime-selection-preview-debugger-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/PoseAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/PoseAsset.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Public/AnimNodes/AnimNode_PoseByName.h
  - dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Private/AnimNodes/AnimNode_PoseByName.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Public/AnimNodes/AnimNode_PoseBlendNode.h
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SPoseEditor.h
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SPoseEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Factories/PoseAssetFactory.cpp
  - dev/Fyrox/fyrox-animation/src/pose.rs
  - dev/Fyrox/fyrox-animation/src/machine/node/blend.rs
  - dev/Fyrox/fyrox-animation/src/machine/node/blendspace.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/animation_curves.rs
  - dev/bevy/crates/bevy_animation/src/morph.rs
  - dev/godot/scene/resources/animation.cpp
  - dev/godot/scene/animation/animation_blend_tree.cpp
  - dev/godot/tests/scene/test_animation_blend_tree.cpp
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/LinearBlendSkinningNode.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/ComputeDeformNode.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Animation Pose Library、Pose Asset、Pose Name、Curve Weight、Additive Base、Runtime Evaluation、Preview 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon没有工程级Pose Asset或Pose Library产品。唯一专用表面是230行`workbench_extension_pose_library_workspace.zui`：`PL_Combat / PL_Locomotion / PL_Emotes`、`42 poses / 6 tags`、`Idle Ready / Aim Offset / Crouch Cover / Mirror Candidate`、`Combat.Ready`与`LeftRight`均为固定文本。其3个tab、8个row action、3个command和6个field action合计20个标准化action，只执行workspace/tab/row/popup选择和固定feedback；Preview与Apply只返回“queued”，不产生asset mutation、transaction、compile job、runtime request或operation receipt。

通用动画底座并非全空。Runtime能从`AnimationClipAsset`的骨骼transform track采样`AnimationPoseOutput`；Graph可描述Clip、Blend、Additive、Mask与Output；first-party Animation插件具有SoA `PoseBuffer`、`PosePool`、dense target table、clip revision cache、按骨骼归一化和additive混合。这些基础只能证明“一个Clip可被采样并混合”，不能证明独立Pose Asset存在：`ImportedAsset`、`ArtifactCache`、`AssetKind`、import suffix、load API和插件request均没有Pose Library类型，公开输出仍退化为`Vec<AnimationPoseBone>`加骨名字符串，场景写回再次按名字解析并吞掉transform更新错误，glTF morph weight动画还被明确拒绝。

本轮不新增P0。Editor14 P0-2继续唯一拥有静态高级动画workspace的capability truth；Runtime08C、Editor76/77与Plugins13继续唯一拥有通用animation compiler/evaluator/pose buffer问题；Editor78与Editor79分别拥有Control Rig bake和Motion Matching/Pose Search。本报告只登记尚未逐项建账的 **18项P1、5项P2和48个资格门**，把Pose Asset专属合同展开为`AnimationPoseLibrarySourceDocument -> PoseLibraryCompilePlan -> PreparedPoseLibrary -> PoseLibraryRuntimeHandle -> PoseEvaluationRequest -> PoseEvaluationReceipt`。

本轮只做current-source review和文档建账，不修改生产源码。未运行Cargo、真实Editor、GUI/GPU、import/cook、runtime evaluation、preview、hot reload、fault/soak/profile或同语义跨引擎benchmark；因此不能宣称Pose Library可用、正确、稳定或性能达标，更不能宣称性能或表现超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 本轮唯一owner与去重边界

本报告只拥有“命名Pose如何从Clip/Reference/Viewport等来源提取并保持稳定identity/provenance，如何表达骨骼、曲线和morph通道以及full/additive base语义，经唯一Animation compiler生成immutable prepared artifact，再由Runtime按名称或权重集求值并由Editor事务化编辑、预览、重导入”的纵向边界。

- Editor04继续拥有通用asset discovery、import/reimport、catalog与thumbnail基础设施。
- Editor14继续拥有高级动画workspace的capability truth、通用toolkit/preview/compile真实性；本轮不重复假UI P0。
- Runtime08C继续拥有通用dense pose、prepared animation与runtime evaluation平台。
- Plugins13继续拥有first-party provider/evaluator、SoA pose buffer、stable target和scene writeback hard cut；本轮复用而不重建第二个evaluator。
- Editor32继续拥有Skeleton、Retarget、Mirror和rig compatibility。
- Editor63继续拥有transaction/history/savepoint/document generation与async CAS。
- Editor69继续拥有PreviewWorld、time domain、pause/step和可见性调度。
- Editor75继续拥有Timeline/Curve通用编辑交互；Pose Asset只拥有pose weight与source curve的domain语义。
- Editor76/77继续拥有唯一Graph/Clip compiler、event/root-motion/sync与prepared artifact主干。
- Editor78只拥有Control Rig solve/bake；其输出可成为Pose提取来源，但不拥有durable Pose Library。
- Editor79只拥有Motion Matching/Pose Search database；其搜索pose不是author-authored named Pose Asset。

### 2.2 Currentness

- 审查HEAD：`9b5564b749e618475e258dd75bd3d9b34e9388a9`。
- 协作baseline epoch：`358`；session：`optimize-editor81-pose-library-pose-asset-review-r1-20260823`。
- 从上一冻结HEAD `c4761b14c6748c4fb0ac7edef67183d8d5afb5eb`前移到本HEAD的产品源码差异未触及本轮Pose证据文件。
- 33个focused Zircon文件在冻结时没有本轮产生的production diff；共享工作区存在其他Session输入，本轮不回退、不整理、不纳入完成声明。
- 专用语义检索确认Editor只有Pose Library静态产品链；`zircon_runtime`、`zircon_plugins`、`zircon_app`没有`PoseAsset`或`PoseLibrary`类型、handle、request或artifact。
- action inventory、navigation index、ZUI load与feedback测试只证明控件路由和模板可解析，不证明domain asset、compiler、runtime evaluation或preview闭环。

### 2.3 冻结语料与可复算fingerprint

统计口径：路径转为小写正斜杠并排序；每个文件取SHA-256，再拼接`path + NUL + lowercase file hash + LF`计算集合fingerprint。declarations使用Rust/C++/C#的`fn/class/struct/enum/trait`行首声明正则，仅用于规模定位。

| 范围 | 文件 / 行 / 非空行 / bytes / declarations | fingerprint |
|---|---:|---|
| Zircon selected set | **33 / 8,970 / 8,451 / 382,822 / 241** | `1d344780de822a31c53055f32485e66df42e7b55344c595165bbfcd08a12127d` |
| Unreal selected set | **11 / 5,067 / 4,186 / 163,930 / 31** | `0777e6b33279f8de812d32eba7451198dc62de5e92bbaed97f3646d51c4e6e1d` |
| Godot selected set | **5 / 2,777 / 2,234 / 106,787 / 21** | `49a2862f49c9385efb839589c7865c123b492dff93dfcdefc8c30b41d70f8f98` |
| Fyrox selected set | **3 / 1,119 / 963 / 40,371 / 71** | `14f075ed966fd2124e20d0e8453eed20cfcee099f3a3bb055a3be58e59259db5` |
| Bevy selected set | **4 / 3,893 / 3,533 / 146,740 / 225** | `96b460cafc8b435e3d5a9c78e1057e5b9e9a34d139160b92a569b261d59db185` |
| Graphics selected set | **2 / 359 / 325 / 17,789 / 2** | `73536e0ff89f1553836962e2a2d73fcf985e7a6f14e62fb82d4167257308345c` |
| Five-engine deduplicated set | **25 / 13,215 / 11,241 / 475,617 / 350** | `3b66b9a6d200fada5b9dbeec52e4544ceb6b8b6df2195e8b61a1bcb5f70d5195` |

## 3. Zircon当前产品链事实

### 3.1 Pose Library workspace是静态样板，不是asset editor

`workbench_extension_pose_library_workspace.zui`没有document binding、asset handle、revision、dirty/savepoint、selection model或provider generation。列表、标签、pose count、来源、mirror状态、权重与校验结果全部写死；Apply没有目标对象/骨架/世界/实例identity。`extension_module_feedback.rs`只把动作翻译成“opened / selected / queued”文本，navigation spec只把action映射到control ID，preview action测试只清点字符串。

因此当前产品风险不是“功能较少”，而是可见UI宣称存在一套并不存在的authority。实现前应先继承Editor14 capability gate：provider缺失时隐藏或disabled，固定成功措辞不得继续进入Ready产品面。

### 3.2 Runtime只有通用Clip pose DTO

`AnimationPoseOutput`只有`source`、`active_state: Option<String>`与`bones: Vec<AnimationPoseBone>`；`AnimationPoseSource`只区分Clip、Graph、StateMachine。`AnimationClipAsset`只有skeleton、duration、bone tracks和event tracks，不承载命名pose、source curve、morph weight、reference/base pose、provenance或pose-level metadata。`AnimationManager::sample_clip_pose`从skeleton bind pose开始按target/name采样transform，并输出带骨名的Vec。

Graph的Additive节点能给Clip实例标记base/additive并计算权重，但图求值结果仍是weighted clip instance描述，不是可独立保存、按名称寻址或热重载的Pose Asset。当前`PoseBuffer.weights`又把值限制在`0..=1`，尚未定义single pose clamp、multi-pose normalization、additive负权重/overdrive和缺失贡献的统一政策。

### 3.3 资产、编译与加载链没有Pose owner

`ImportedAsset`、artifact cache、asset kind、import suffix、load API、compiled clip和Animation插件request只覆盖Skeleton、Clip、Sequence、Graph、StateMachine等既有类型。不存在Pose source schema、schema migration、semantic compiler、prepared artifact、dependency manifest、cook policy、LKG、runtime handle或generation receipt。glTF importer把`MorphTargetWeights`排除在bone track builder之外，并把该channel作为不支持的bone animation返回错误，说明面部/形变pose通道没有旁路基础。

### 3.4 插件底座应保留，但跨层边界仍退化

Animation插件的SoA `PoseBuffer`、`PosePool`、dense target table、revision cache和additive/reference-delta计算是可复用底座。问题在于`GraphWeightedPose`仍携带`AnimationPoseOutput`和legacy string target ID，`convert_pose_to_reference_delta`按骨名比对，`pose_apply.rs`又编译后代名字索引并按名字写回场景，且忽略`world.update_transform`错误。这与Plugins13的NANI-P1-026、030、031、041一致，本报告不重复计数；Pose Asset必须直接消费修复后的stable target/dense pose authority。

## 4. 参考引擎证据与可迁移原则

### 4.1 Unreal主参考：独立Pose Asset、稀疏影响与编辑生命周期

`UPoseAsset`是独立`UAnimationAsset`，而不是Clip或Graph里的临时数组。`FPoseDataContainer`保存pose names、sorted curve indices、tracks、track/bone映射、per-track pose influences、pose data与curve metadata；editor source可保留full local pose和source curves，runtime data可压为稀疏local pose与uncompressed curves。资产还记录source animation、raw-data GUID、retarget source/mesh/reference pose、additive flag与base pose index。

运行时既有按名称缓存索引的PoseByName节点，也有把source curves映射为pose weights的PoseBlendNode；single pose、multi pose、full pose与additive pose具有明确而不同的权重/归一化/缺失贡献政策，骨骼只遍历已知influence，曲线与pose在同一evaluation中混合。Editor侧支持add/update/delete/rename pose与curve、从animation/reference/viewport提取、full/additive转换、base pose重设、invalid track清理和source GUID过期检查；`SPoseEditor`使用transaction、asset `Modify`和preview override，factory负责source animation与pose name选择。

本轮未在目标Runtime tests中找到可证明Pose Asset数值语义的直接Unreal专项测试，因此报告只采信生产合同，不把“存在类型”误写成“已被完整测试”。

### 4.2 Fyrox、Bevy与Godot稳定运行时原则

- Fyrox的`AnimationPose`是可复用pose map，支持`clone_into/reset/blend/root motion`；blend/blendspace节点持有可复用输出pose并显式计算source weight和event政策。这证明热路径结果需要复用与明确合成，而不是每帧重建带名字Vec。
- Bevy的serialized animation graph区分Clip/Blend/Add与mask，并可编译为缓存postorder的threaded graph；curve evaluator使用type-erased stack/register和显式BlendInput/additive合同，morph weights是独立variable-width typed curve evaluator并提交到`MorphWeights`。这证明transform、curve、morph应进入同一类型化求值框架而非只支持bone transform。
- Godot的`AnimationLibrary`提供add/remove/rename/has/get/list、名称校验与changed signal；blend tree显式表达Add/Blend、filter、sync及负值/越界权重范围。现有目标测试只验证节点创建、连接、rename/remove与连接重写，不证明blend数学，因此只采信observable lifecycle与结构契约。

### 4.3 Unity Graphics只约束下游消费

仓内Graphics参考没有Mecanim或PoseAsset authoring源码。`LinearBlendSkinningNode`和`ComputeDeformNode`只证明renderer需要稳定skin matrix/vertex bone weight以及current/previous deformation数据来维持motion vector。它不能作为Pose资产主参考，但要求Pose求值与场景/渲染写回保留generation、current/previous一致性，不能用Editor私有名字数组旁路deformation owner。

## 5. P1工程化差距（18项）

### POSE-P1-001：缺少canonical `AnimationPoseLibrarySourceDocument`

需要独立asset/document/source revision、schema version、rig/skeleton dependency、authoring metadata和stable serialization。它不能复用Graph临时节点数组，也不能把ZUI行当source truth。

### POSE-P1-002：Pose entry没有稳定identity与名称生命周期

引入`PoseLibraryEntryId`、validated display name、stable order、tags/set membership、duplicate policy和rename/remove/reorder reference repair。Runtime、transaction、trace与preview必须按ID/generation寻址，名称只作为可验证lookup key。

### POSE-P1-003：缺少提取来源与provenance

引入`PoseExtractionRecipe`，覆盖source clip/time/sample mode/source generation、reference pose、viewport snapshot、Control Rig bake和update policy。来源变化必须可检测out-of-date，并能选择re-extract、keep authored overrides或reject。

### POSE-P1-004：缺少Skeleton/Rig兼容、Retarget与Mirror依赖

资产必须冻结stable target table、source/target rig identity、retarget profile、mirror table及其generation。missing bone、renamed bone、different hierarchy和provider reload需要typed diagnostic，不得按名字静默跳过。

### POSE-P1-005：通道模型只覆盖骨骼transform

Pose entry需要类型化bone transform、scalar/vector curve、attribute和variable-width morph weight通道，并记录sparse influence。未知channel、重复target、width mismatch和非有限值必须在compile前拒绝。

### POSE-P1-006：Full/Additive/Base/Reference语义未定义

明确full local/component pose、reference-relative delta、selected-base-pose delta与mesh/retarget reference的合同；转换必须可复算、可诊断、可撤销，base删除或变更不能隐式重解释全部pose。

### POSE-P1-007：Pose weight与混合政策未定义

定义single pose clamp、multi full-pose normalization、additive negative/overdrive、zero-total、missing contribution、curve/morph policy、quaternion hemisphere/order及deterministic accumulation。禁止由UI slider范围或`PoseBuffer.weights`偶然决定运行时语义。

### POSE-P1-008：缺少semantic compiler与稀疏prepared artifact

`PoseLibraryCompilePlan`应验证source与依赖，生成dense target indices、per-target influence ranges、pose/curve/morph payload、name/ID lookup、base/additive metadata和diagnostics；`PreparedPoseLibrary`必须immutable、自包含且runtime不读取mutable source。

### POSE-P1-009：缺少artifact currentness、job、LKG与cook合同

build key覆盖source/schema/compiler/rig/retarget/mirror/curve layout/target profile generation；接入Editor09 job admission、cancel/progress、atomic publish、LKG stale标记、cook strip与dependency provenance。旧artifact不可冒充current。

### POSE-P1-010：缺少qualified runtime handle与生命周期

引入world/provider/asset/artifact generation限定的`PoseLibraryRuntimeHandle`，覆盖load、resident、evict、reload、unload、world replace与shutdown。旧handle、跨world handle和provider generation mismatch全部fail-close。

### POSE-P1-011：唯一Animation evaluator没有PoseByName/WeightSet入口

在Editor76/77与Plugins13收束后的唯一evaluator中增加`PoseEvaluationRequest`，支持entry ID/name、multi-pose `PoseWeightSet`、mask和base input；输出继续使用共享dense `PoseBuffer`/pool，禁止建立Editor私有evaluator。

### POSE-P1-012：失败、fallback与receipt不是typed合同

`PoseEvaluationReceipt`应区分accepted、missing pose、ambiguous name、rig mismatch、stale artifact、evicted、provider reload、budget exceeded与terminal failure，并记录resolved entry/generation、weights、fallback与diagnostic identity。reference pose fallback必须显式且可观测。

### POSE-P1-013：Graph、State、Gameplay、Scene与Render没有下游闭环

Graph节点、state/action、Gameplay consumer、scene apply、deformation与network/replay只能消费同一prepared generation和receipt。stable target/dense pose修复由Plugins13拥有；Pose owner只提供类型化输入，不得重新引入骨名字符串和半成功写回。

### POSE-P1-014：缺少真实Pose Library document/toolkit

建立`PoseLibraryEditorSession`和dynamic projection，显示真实asset identity、entry、source、tags、channels、base/additive状态、artifact currentness与diagnostics。固定`PL_Combat`样例只能保留为明确标注的demo fixture，不能进入生产authority。

### POSE-P1-015：所有编辑动作缺少transaction与CAS

Add/Extract/Update/Duplicate/Rename/Delete/Reorder、Set Base、Convert Additive、Mirror、Retarget、Tag和Update From Source均需Editor63 transaction、expected document generation、undo/redo、dirty/savepoint与atomic save。失败不得留下半更新引用或半编译artifact。

### POSE-P1-016：Preview不是runtime-backed证据

PreviewWorld必须运行同一`PreparedPoseLibrary`与evaluator，支持weight scrub、single/multi pose、full/additive、reference/base/difference、mirror/retarget以及bone/curve/morph diagnostics；显示artifact/source generation和fallback，不能只返回queued文本。

### POSE-P1-017：Import/Reimport/Batch Extraction与依赖失效未闭环

需要从Clip/FBX/glTF/Control Rig等来源批量提取、命名映射、mirror/retarget recipe、source revision跟踪和可审计reimport diff。morph weights必须进入独立typed channel，不得继续因bone-track importer限制而丢失。

### POSE-P1-018：缺少跨层资格、故障、规模与性能证据

补齐schema/migration、数值oracle、additive/normalization、rename/remove reference、malformed artifact、reload/evict/shutdown、1/100/1k pose与target规模、allocation-free steady state、cook/reopen、cross-carrier和跨平台测试；固定字符串、action inventory或单一happy path均不计资格。

## 6. P2平台扩展差距（5项）

| ID | 差距 | 目标 |
|---|---|---|
| POSE-P2-001 | Corrective pose、pose driver与RBF缺席 | 在基础Pose Asset稳定后增加driver input、RBF solver、corrective set、LOD与diagnostic，不把高级solver塞入core schema。 |
| POSE-P2-002 | 语义taxonomy、search与质量分析缺席 | 支持tag hierarchy、similarity/dedup、thumbnail、coverage/outlier与authoring lint，并复用Editor79特征基础而不混淆两类资产。 |
| POSE-P2-003 | Constraint-aware mirror/retarget批处理缺席 | 加入contact/constraint-aware mirror、retarget quality report、batch repair与可撤销authoring automation。 |
| POSE-P2-004 | 大型Pose Library streaming与GPU路径缺席 | 评估page residency、LOD、pose sharing、compressed sparse payload、GPU decompression与budget telemetry，先以数据证明需要。 |
| POSE-P2-005 | LiveLink/Capture/ControlRig/ML Deformer交换缺席 | 定义外部capture、Control Rig bake、procedural pose和ML deformer interchange；核心Control Rig与ML能力继续由各自owner负责。 |

## 7. 目标架构与关键合同

```text
Pose source/import/capture
        |
        v
AnimationPoseLibrarySourceDocument
  - PoseLibraryEntryId / validated name / tags
  - PoseExtractionRecipe / provenance / source generation
  - typed transform / curve / morph channels
  - full/additive/base/reference + rig dependencies
        |
        v
PoseLibraryCompilePlan -- Editor09 job / dependency key / diagnostics
        |
        v
PreparedPoseLibrary
  - dense target table + sparse influence ranges
  - immutable pose/curve/morph payload
  - ID/name lookup + base/additive metadata
  - artifact generation / provenance / cook profile
        |
        v
PoseLibraryRuntimeHandle
        |
        v
PoseEvaluationRequest(PoseWeightSet, mask, base input)
        |
        v
Shared Animation evaluator -> pooled dense PoseBuffer
        |
        v
PoseEvaluationReceipt -> Graph / Gameplay / Scene / Render / Preview
```

关键类型建议：

- `AnimationPoseLibrarySourceDocument`：唯一可保存authoring truth。
- `PoseLibraryEntryId`：跨rename、transaction、trace与runtime稳定。
- `PoseExtractionRecipe`：来源、采样、mirror/retarget与更新政策。
- `PoseLibraryCompilePlan`：输入generation、依赖、target profile和diagnostics。
- `PreparedPoseLibrary`：runtime唯一可消费immutable artifact。
- `PoseLibraryRuntimeHandle`：world/provider/artifact generation限定句柄。
- `PoseWeightSet`：ID/weight/mode/mask的validated deterministic输入。
- `PoseEvaluationRequest`与`PoseEvaluationReceipt`：求值边界和可观测结果。
- `PoseLibraryEditorSession`：document generation、selection、transaction和preview绑定。

## 8. 实施里程碑

### ED81-M0：Owner、capability truth与currentness

冻结父owner，关闭固定Ready/queued措辞；定义Pose capability manifest、source/artifact/runtime generation、错误taxonomy和baseline tests。

### ED81-M1：Source schema、stable identity与provenance

实现source document、entry ID/name lifecycle、typed channels、extraction recipe、rig/retarget/mirror依赖、migration与roundtrip。

### ED81-M2：Compiler、prepared artifact与数值语义

实现full/additive/base/reference、weight policy、dense/sparse layout、curve/morph、deterministic oracle、build key与diagnostic；接入job、LKG和atomic publication。

### ED81-M3：Runtime handle与唯一evaluator

在共享Animation pipeline接入PoseByName/WeightSet、qualified handle、pooled dense output、typed receipt、fallback与budget；不保留名字Vec旁路。

### ED81-M4：Asset pipeline、cook与reload

接入asset registry/import/reimport/artifact cache/cook/package，完成dependency invalidation、resident/evict、provider reload、LKG和generation fencing。

### ED81-M5：真实Editor document与transaction

用dynamic projection替换固定ZUI数据，完成entry/source/tag/channel/details操作、transaction/CAS、dirty/save/undo/redo、diagnostic定位与currentness。

### ED81-M6：Preview、import、retarget与产品集成

PreviewWorld运行同一artifact/evaluator；打通batch extraction、mirror/retarget、Graph/Gameplay/Scene/Render消费与trace映射。

### ED81-M7：Fault、scale、performance与跨平台资格

完成malformed/stale/reload/evict/shutdown fault matrix、数值oracle、1/100/1k scale、allocation/residency/profile、cook/reopen和cross-carrier验证。

### ED81-M8：Hard cutover与旧表面退役

删除固定domain fixture和legacy string pose path；只在48项门禁全部有证据后恢复Ready入口，并以可复现benchmark决定性能声明。

## 9. 资格门（当前均Fail）

### 9.1 Source、Identity与Compiler

- [ ] POSE-G-01：Pose Library可Create/Open/Save/Reopen，asset/document/source revision与entry ID稳定。
- [ ] POSE-G-02：Add/Duplicate/Rename/Delete/Reorder在引用、tag、selection和undo/redo中保持identity正确。
- [ ] POSE-G-03：重名、空名、非法tag、dangling base、重复target和非有限值被typed diagnostic拒绝。
- [ ] POSE-G-04：Clip/reference/viewport/Control Rig提取保存source generation与recipe，可检测out-of-date。
- [ ] POSE-G-05：Skeleton/Rig/Retarget/Mirror依赖generation进入source和build key，mismatch fail-close。
- [ ] POSE-G-06：transform、scalar/vector curve与variable-width morph通道均可roundtrip和编译。
- [ ] POSE-G-07：full/additive/base/reference转换有数值oracle且base变化不会隐式破坏资产。
- [ ] POSE-G-08：compiler输出自包含`PreparedPoseLibrary`，runtime不读取mutable source或Editor state。

### 9.2 Artifact、Load与Lifecycle

- [ ] POSE-G-09：build key覆盖schema/compiler/source/rig/retarget/mirror/channel/target profile generation。
- [ ] POSE-G-10：compile cancel/failure保留LKG但明确stale，错误generation不能进入cook或Ready。
- [ ] POSE-G-11：artifact包含dense target、sparse influence、ID/name lookup、curve/morph与provenance。
- [ ] POSE-G-12：malformed/unknown schema/truncated payload/invalid range在publish前fail-close。
- [ ] POSE-G-13：load/resident/evict/reload/unload状态有qualified handle和ordered receipt。
- [ ] POSE-G-14：旧handle、跨world handle、provider reload与asset generation mismatch不能误命中新对象。
- [ ] POSE-G-15：world replace/shutdown终结handle、lease与pending request，不遗留幽灵residency。
- [ ] POSE-G-16：cook/export/install/reopen只发布validated generation并保留dependency provenance。

### 9.3 Evaluation与数值语义

- [ ] POSE-G-17：PoseByID与PoseByName命中同一entry，missing/ambiguous name产生typed receipt。
- [ ] POSE-G-18：single full pose clamp与reference contribution符合冻结政策和数值oracle。
- [ ] POSE-G-19：multi full pose normalization、zero-total与missing influence结果确定。
- [ ] POSE-G-20：additive negative/overdrive、base input与reference delta结果确定。
- [ ] POSE-G-21：quaternion hemisphere/order、scale/translation与curve accumulation跨平台可复现。
- [ ] POSE-G-22：morph variable width、curve type mismatch与mask组合不越界、不静默丢通道。
- [ ] POSE-G-23：稳态求值复用PosePool/dense target，无每帧骨名查找与无界heap allocation。
- [ ] POSE-G-24：request与receipt原子记录resolved generation、weight、fallback、budget和diagnostic。

### 9.4 Runtime Integration与Failure

- [ ] POSE-G-25：Animation Graph使用同一prepared artifact/evaluator，不复制pose数组或混合数学。
- [ ] POSE-G-26：State/Gameplay consumer按qualified handle消费并能区分reject、fallback与terminal failure。
- [ ] POSE-G-27：Scene apply使用stable target并传播写回错误，不按骨名吞掉半失败。
- [ ] POSE-G-28：renderer current/previous deformation在pose jump/reload/teleport时有明确reset政策。
- [ ] POSE-G-29：network/replay/save/load携带asset/artifact generation和entry ID，rename不破坏重放。
- [ ] POSE-G-30：provider reload、artifact evict与callback重入期间无UAF、旧generation写回或半publish。
- [ ] POSE-G-31：missing bone/channel、rig mismatch与unavailable morph consumer的fallback可配置且可观测。
- [ ] POSE-G-32：budget exceeded返回continuation或failure，不静默提交不完整pose。

### 9.5 Editor、Transaction与Preview

- [ ] POSE-G-33：workspace显示真实asset/document/revision/currentness，不再显示固定`PL_Combat`结果。
- [ ] POSE-G-34：所有authoring command走transaction/CAS，失败不产生半更新或脏artifact。
- [ ] POSE-G-35：selection、list、details、tags、source、curve与diagnostic绑定同一document generation。
- [ ] POSE-G-36：Set Base/Convert Additive/Mirror/Retarget/Update Source可undo/redo/save/reopen。
- [ ] POSE-G-37：rename/delete会更新或拒绝Graph/Gameplay/asset引用，并提供可审计reference report。
- [ ] POSE-G-38：PreviewWorld运行同一prepared generation，single/multi/full/additive与Runtime一致。
- [ ] POSE-G-39：preview可显示reference/base/difference、bone/curve/morph贡献和fallback diagnostic。
- [ ] POSE-G-40：source stable ID可从runtime trace定位回entry/channel，stale trace明确标记。

### 9.6 Import、Fault、Scale与交付

- [ ] POSE-G-41：batch extraction/reimport保留recipe、命名映射、override与source diff，不静默覆盖手工编辑。
- [ ] POSE-G-42：glTF/FBX morph与custom curve进入typed channel，不能因bone-track限制而丢失。
- [ ] POSE-G-43：malformed source、missing dependency、reload/evict/shutdown矩阵无panic/UAF/幽灵handle。
- [ ] POSE-G-44：1/100/1k entries与targets有compile time、artifact size、resident memory和query curve。
- [ ] POSE-G-45：1/100/1k active evaluations有CPU、allocation、cache miss和scene apply scale curve。
- [ ] POSE-G-46：真实用户流Create/Extract/Edit/Undo/Save/Compile/Preview/Reload/Reopen通过，字符串测试不计资格。
- [ ] POSE-G-47：Windows与目标平台的serialization、determinism、cook和runtime receipts一致。
- [ ] POSE-G-48：与Unreal同语义场景的质量/CPU/内存/延迟benchmark可复现；未测不得宣称超越。

## 10. 实施约束与退出条件

1. 先关闭Editor14 capability truth：真实provider缺失时隐藏或disabled Pose Library入口，不得继续返回`queued/Ready`。
2. Source、compiler、artifact与runtime evaluation必须落入Editor76/77、Runtime08C和Plugins13的唯一Animation authority，不新增Editor私有求值器。
3. 保留并强化SoA `PoseBuffer`、`PosePool`与dense target底座；硬切legacy骨名Vec和按名字scene writeback，不留compat shim。
4. transform、curve与morph通道必须共同进入typed compiler/evaluator；Graphics只消费稳定deformation结果，不反向拥有动画资产。
5. 每个里程碑先补RED test，再实现最小vertical slice；数值、lifetime、reload和跨模块风险决定测试宽度。
6. 开始实现前重算本报告fingerprint与baseline，复核共享工作区相关animation文件、父owner状态和lease。

本报告退出条件不是“Pose Library ZUI能打开”或“Apply显示成功”，而是POSE-G-01至POSE-G-48全部有动态证据，并且父报告的asset、compiler、stable target、transaction、preview与runtime lifecycle前置同时满足。在此之前，Pose Library只能标记为Unavailable/Experimental，不得进入Ready产品能力表。
