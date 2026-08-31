---
title: Editor Animation Pose Library、Pose Asset、Pose Name、Curve Weight、Additive Base、Runtime Evaluation、Preview 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor202
review_date: 2026-08-28
baseline_head: 67e91cc6e970e1c5fd964a289d4d674a52854462
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_pose_library_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_runtime/src/core/framework/animation/pose_source.rs
  - zircon_runtime/src/core/framework/animation/pose_bone.rs
  - zircon_runtime/src/core/framework/animation/pose_output.rs
  - zircon_runtime/src/core/framework/animation/pose_snapshot.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/importer/ingest/import_animation_asset.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/scene/level_system/frame_state.rs
  - zircon_runtime/src/scene/level_system/animation_runtime.rs
  - zircon_editor/src/core/editing/animation_document/asset.rs
  - zircon_editor/src/core/editing/animation_document/command.rs
  - zircon_editor/src/core/editing/animation_document/compilation.rs
  - zircon_editor/src/core/editing/animation_document/document.rs
  - zircon_editor/src/core/jobs/system/submission.rs
  - zircon_plugins/animation/runtime/src/evaluation/skeleton_target_table.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/pose_buffer.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/storage.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_buffer/blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_pool.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/sample.rs
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator/animation_evaluation_error.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_blend.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_apply.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/pose_target_binding.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/animation_evaluation_pipeline.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/src/manager/pose.rs
  - zircon_runtime/src/animation/manager/pose.rs
tests:
  - zircon_plugins/animation/runtime/tests/animation_pose_buffer_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_pose_buffer_allocation_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_target_table_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/target_resolution.rs
  - zircon_editor/src/core/editing/animation_document/tests.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/198-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md
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
  - dev/godot/scene/resources/animation_library.h
  - dev/godot/scene/resources/animation_library.cpp
  - dev/godot/scene/resources/animation.cpp
  - dev/godot/scene/animation/animation_blend_tree.cpp
  - dev/godot/tests/scene/test_animation_blend_tree.cpp
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/LinearBlendSkinningNode.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/ComputeDeformNode.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/81-editor-animation-pose-library-pose-asset-pose-name-curve-weight-additive-base-runtime-evaluation-preview-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/81-editor-animation-pose-library-pose-asset-pose-name-curve-weight-additive-base-runtime-evaluation-preview-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Animation Pose Library、Pose Asset、Pose Name、Curve Weight、Additive Base、Runtime Evaluation、Preview 与 Product Integration 当前源码复核

## 1. 结论

当前 Zircon 仍没有工程级 Pose Asset 或 Pose Library 产品。唯一专用产品表面是 233 行 `workbench_extension_pose_library_workspace.zui`：`PL_Combat / PL_Locomotion / PL_Emotes`、`42 poses / 6 tags`、`Idle Ready / Aim Offset / Crouch Cover / Mirror Candidate`、`Combat.Ready` 与 `LeftRight` 均为固定文本。20 个标准化 action 最终只选择 workspace/tab/row、打开通用 dropdown 或写入固定 feedback；Preview 与 Apply 只返回 `queued`，字段 `.commit` 不产生 typed value、asset mutation、transaction、compile job、runtime request 或 operation receipt。

对 `zircon_runtime`、Animation Runtime 插件、`zircon_app` 与 `zircon_runtime_interface` 的 Pose Asset runtime/compiler 精确语义检索为 0；另扫描 2,265 个未跟踪 Rust/ZUI 文件，同样没有 `PoseAsset / PreparedPoseLibrary / PoseEvaluationRequest / PoseEvaluationReceipt / PoseByName / PoseLibraryRuntimeHandle` 命中。`ImportedAsset`、artifact cache、动画 suffix/import/load 与 shared compiler 仍只覆盖 Skeleton、Clip、Sequence、Graph、StateMachine 等既有类型，`AnimationPoseSource` 也只有 Clip、Graph、StateMachine。

旧 Editor81 有八处需要正向校正，但都只是 Pose Library 的共享前置：Animation document 已有 revision/CAS/current/LKG 与可撤销 source swap；通用 job 有 admission/merge/batch/cancel 基础；Animation 插件已有 dense `SkeletonTargetTable`、SoA `PoseBuffer`、`PosePool`、有限值与形状校验、per-bone normalization、最短弧/规范化四元数混合；clip evaluator 有 typed error；Level frame 已用 `Arc` 发布 immutable pose snapshot；Scene/Physics/GPU skinning 已有下游入口；专项测试覆盖池内混合零分配、target table 与部分数值合同。这些能力不能表达独立 Pose Asset、命名 entry、来源 provenance、curve/morph channel、base/additive conversion、prepared artifact、qualified handle 或 runtime receipt。

同时，底座仍存在不能被性能微基准掩盖的退化：`sample_compiled_pose` 每次求值都重新收集 `Vec<AnimationPoseBone>` 并克隆骨名；`tick` 又为 skeletal target 重建 `Vec` 和骨名。`GraphWeightedPose` 继续携带完整 AoS pose 与 `legacy_target_ids`；reference delta 按长度、顺序和骨名比对；scene apply 按名字解析并丢弃 `world.update_transform` 错误。`PoseBuffer` 拒绝负权重/overdrive，graph base blend 忽略非正/非有限权重并归一化，graph additive 又直接使用未统一验证的权重，三条路径没有单一数值政策。仓内还保留 Runtime 与插件两套 name-based clip sampler。

Editor81 的 18 项 P1 当前重判为 **10 Open / 8 Partial / 0 Closed**，5 项 P2 为 **5 Open / 0 Partial / 0 Closed**；48 个资格门为 **43 Fail / 5 Partial / 0 Pass**。canonical finding 数不重复增加。本轮不新增父报告 P0：Editor14 的 capability truth、Editor197/198 的唯一 compiler/evaluator 与 Clip 语义、Runtime08C/Plugins13 的 dense pose/stable target/consumer hard cut、Editor32 的 rig/retarget/mirror、Editor63 的 transaction、Editor69 的 PreviewWorld 仍是上游 owner。

本轮只做静态 current-source review 与文档建账，不修改 production Rust/ZUI，不运行 Cargo、Editor、GUI/GPU、import/cook、runtime Pose evaluation、preview、reload/evict、fault/soak/profile 或同语义跨引擎 benchmark。不能据此宣称 Pose Library 可用、正确、稳定或性能达标，更不能宣称性能或表现超过 Unreal。Tooling 按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

## 2. 审查边界、owner 与冻结语料

### 2.1 本报告唯一纵向边界

本报告只拥有：

```text
AnimationPoseLibrarySourceDocument
  -> PoseLibraryCompilePlan
  -> PreparedPoseLibrary
  -> PoseLibraryRuntimeHandle
  -> PoseEvaluationRequest(PoseWeightSet)
  -> PoseEvaluationReceipt
  -> Graph / Gameplay / Scene / Physics / Render / Editor Preview consumers
```

- Editor04 拥有通用 asset discovery、import/reimport、catalog 与 thumbnail。
- Editor09/131 拥有 background job admission、cancel、progress、scope、shutdown 与 durable result 总合同。
- Editor14 拥有高级动画 workspace 的 capability truth、通用 toolkit/preview/compile 真实性。
- Runtime08C 与 Plugins13 拥有唯一 Animation provider/evaluator、dense pose、stable target、scene writeback 与 deformation 主干。
- Editor32 拥有 Skeleton、Rig、Retarget、Mirror 与 compatibility。
- Editor63/184 拥有 transaction/history/savepoint/document generation 与 async CAS。
- Editor69/190 拥有 PreviewWorld、time domain、pause/step 与 subsystem admission。
- Editor196 拥有 Timeline/Curve/transport 通用交互。
- Editor197/198 拥有唯一 Graph/Clip compiler、prepared artifact、event/root-motion/sync 与 playback transaction。
- Editor78/79 分别拥有 Control Rig bake 与 Motion Matching/Pose Search；它们可消费或产生 Pose 数据，但不拥有 durable named Pose Library。

### 2.2 Currentness、负证据与已知失败

- HEAD 锚点为 `67e91cc6e970e1c5fd964a289d4d674a52854462`；结论以 2026-08-28 当前磁盘内容为准。审查期间 HEAD 多次前移，最终差异核验未发现中途提交触及冻结的 Pose 产品、Runtime 或参考语料。
- 当前工作树有大量共享在途与未跟踪文件；本轮不回退、不覆盖、不归属这些变化。旧 Editor81 本身是未跟踪 canonical 文档，本报告不修改它。
- tracked runtime/compiler 精确专用语义为 0；2,265 个未跟踪 Rust/ZUI 文件的同一检索也为 0。Editor 中 `PoseLibrary` 命中全部属于固定 ZUI、binding、navigation、feedback 与静态 action inventory。
- `docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-level-system-runtime-state-frame-snapshot.md` 仍为 Open。当前 `AnimationPoseSnapshot = Arc<BTreeMap<EntityId, Arc<AnimationPoseOutput>>>` 已关闭“每个 reader 深拷贝全部姿态”的动画子问题，但跨域互斥、锁 hold、稳定帧 clone bytes 与 1k/100k/64-reader 动态性能门尚未验收，不能擅自把 failure 标为 fixed。
- 现有 Animation 插件仍有 fallback evaluator divergence、frame diagnostics omission、Sequence caller/root drift 与 dynamic runtime module duplication 等上游失败记录。Pose Library 不得接入重复 manager/evaluator 形成第三条语义路径。

### 2.3 冻结语料与 fingerprint

统计口径：相对路径转小写、`/` 规范化并排序；每个文件取 SHA-256，再按 `path + NUL + lowercase file hash + LF` 聚合集合 fingerprint。行数、非空行与 bytes 均按当前磁盘文本计算。

| 范围 | 文件 / 行 / 非空行 / bytes | 本轮证据 | fingerprint |
|---|---:|---|---|
| Zircon Editor/product | **11 / 5,303 / 5,086 / 255,580** | 固定 ZUI、binding、route、feedback 与 action inventory | `adc086cf07fffd57508f17e6eb236e8718ab8e92ee0115e43065f809c92a2977` |
| Zircon shared foundation | **35 / 6,996 / 6,432 / 251,178** | document/CAS/LKG/jobs、asset/import、pose/snapshot、dense evaluator 与 consumer | `c06e59fbee667824f90e2ed6f947c0f6931c808770046410864488e472ca5190` |
| Focused tests | **5 / 932 / 845 / 31,056** | document history/LKG、PoseBuffer、allocation、target table 与 scene target | `156e765af3edd3321d071a00032d741e1f9e664e3529e33327214c36976e9f9d` |
| Zircon deduplicated focused set | **51 / 13,231 / 12,363 / 537,814** | 上述三组按 normalized path 去重 | `73381823e065885b44cb3adaba10eb854d6f7016b9a1253dc549cccf29864ebb` |
| Unreal selected set | **8 / 4,771 / 3,949 / 153,445** | Pose Asset/source GUID、sparse influence、runtime nodes、factory 与 transaction editor | `bfca4b015e22440be6c6d0cada8b046d7900159f8affdcea2322ce595e606f29` |
| Godot selected set | **5 / 8,843 / 7,508 / 313,416** | observable library lifecycle、typed tracks、blend tree 与 rename/remove tests | `702f47e1528746385b287780d863fb4c8d6e41ac8e7e1a08ac07f6a77c54b7c2` |
| Fyrox selected set | **3 / 1,119 / 963 / 40,371** | reusable pose、blend output 与 event policy | `14f075ed966fd2124e20d0e8453eed20cfcee099f3a3bb055a3be58e59259db5` |
| Bevy selected set | **3 / 2,027 / 1,857 / 77,603** | prepared graph、typed evaluator 与 variable-width morph | `799f6e3746cdfd356f11ae54321405821102548dbe3ba6bcfd462746057dea58` |
| Unity Graphics selected set | **2 / 359 / 325 / 17,789** | skin matrix与deformed vertex consumer | `73536e0ff89f1553836962e2a2d73fcf985e7a6f14e62fb82d4167257308345c` |
| Five-engine reference total | **21 / 17,119 / 14,602 / 602,624** | 五组显式路径去重 | `6b1b7cc60d30718225bed0f9e0c1ea46c9ac3201be6c4101eec78cbc04f133cb` |

fingerprint 只是本轮静态输入 receipt，不是未来 Pose Library build key、source revision 或 runtime generation。

## 3. 当前真实实现与旧报告校正

### 3.1 可见 workspace 仍是固定 projection

ZUI 第 152-184 行固定四个 pose row 与 `PL_Combat 42 poses 6 tags`，第 206-228 行固定三项 Library、`Combat.Ready` 与 mirror mode。这里没有 AssetId、DocumentId、SourceRevision、PoseEntryId、Skeleton/Rig generation、channel schema、CompileGeneration、RuntimeHandle 或 ReceiptId。

20 个 binding 覆盖 open、3 tab、8 row、preview/apply 与 6 个 field edit/commit。navigation 只把 action 映射到 control ID；field edit 最多打开通用 dropdown，commit 不读取 typed domain value。feedback 只有 open、preview、apply、Idle 与 Mirror Candidate 五个固定分支，其余 action 落入通用反馈。静态 action 测试只证明 wiring 存在，不证明资产、编译、事务或运行时求值。

### 3.2 资产、compiler 与 channel 链没有 Pose owner

`AnimationAuthoringDocumentKind`、`AnimationAuthoringAsset` 与 shared `AnimationCompileSource/Product` 只有 Sequence、Graph、StateMachine。`ImportedAsset`、cache payload、builtin importer 的五个 `.zranim` suffix、load/acquire API、asset type registry 和 artifact store 没有 Pose Library variant。正确实现不能把 pose 数组塞进 Graph node、Data asset 或 Editor 私有 JSON 旁路这些 owner。

公共 `AnimationPoseOutput` 只包含 source、optional active state 与 `Vec<AnimationPoseBone>`；每个 bone 只有 `String name + local Transform`。没有 scalar/vector curve、attribute、morph、root motion、sparse influence、reference/base metadata 或 source provenance。场景与渲染已有 morph weight 存储、property writer、GPU scene buffer 和 shader consumer，但 glTF Animation `MorphTargetWeights` 在 channel builder 中被忽略，随后被明确拒绝为“不受 AnimationClip bone tracks 支持”。这证明下游能力存在，动画资产链仍断开。

### 3.3 Dense/SoA 与快照是可复用底座，不是完整热路径

`SkeletonTargetTable` 把 canonical path hash 编译到 dense slot，并拒绝空名、非 canonical 名、父环、重复/歧义 target。`PoseBuffer` 以 translation/rotation/scale/weight SoA 存储，`PosePool` 复用容量；测试证明池内 128-joint override/additive 循环为零分配。`AnimationPoseOutput::clone_from_reusing_storage` 也能在目标容量与名字容量已存在时零分配。

真实 clip evaluator 仍在每次 `sample_compiled_pose` 末尾 `collect::<Vec<_>>()` 并对 bind-pose bone name `clone()`；池在这一步之前就被 release。`skeletal_pose_targets()` 随后再次 `collect::<Vec<_>>()` 和 clone bone name。`GraphWeightedPose` 继续拥有 AoS output 与 legacy string target，layer/graph 路径也在完整 pose 间移动。这些事实限定了现有 allocation test 的证明范围：它验证局部容器，不验证端到端 steady-state evaluation。

`AnimationPoseSnapshot` 和 `LevelFrameStateSnapshot` 已以共享 `Arc` 发布，presentation 更新还能保留未变化 entity 的 pose `Arc`。这是 Runtime07 的实质局部修复，也为 Render/Physics 多 reader 提供正确方向。但 map partial update 仍 clone `BTreeMap` 结构，changed pose 本身仍是骨名 AoS；发布前场景 apply、skeletal projection 与完整 evaluator 的分配/错误行为没有被该 snapshot 自动修复。

### 3.4 数值政策、错误与 consumer 仍分裂

SoA `PoseBuffer` 的 layer weight 统一限制为 `0..=1`，最短弧 slerp 与 shape/mask/non-finite 检查有专项测试。Graph base blend 则把非正或非有限权重当 0，按骨骼重新归一化，并通过 canonical quaternion sign 保证输入顺序测试；Graph additive 直接把 `additive.weight` 用于 translation/scale/slerp，没有同一 validation、negative/overdrive 或 failure policy。reference delta 又要求 pose/bind pose 长度、顺序和骨名完全相等。未来 PoseWeightSet 不能选择其中任一路径作为偶然语义，必须先冻结单一政策与 oracle。

`AnimationEvaluationError` 能报告 clip compile、非法 skeleton/channel、missing prepared data 与 pose shape；pipeline 也能发布部分 evaluation diagnostic。它没有 Pose asset/generation/name ambiguity/eviction/budget/fallback receipt。scene apply 编译 descendant name index 后仍按 exact/path-tail/short-name依次匹配；重复 short name 取首个，missing bone 静默跳过，`world.update_transform` 错误被 `let _ =` 丢弃。Physics target publication也只有 cloned bone name 与固定 `normalized_weight: 1.0`。

仓内 `zircon_runtime/src/animation/manager/pose.rs` 与 `zircon_plugins/animation/runtime/src/manager/pose.rs` 仍各自实现 name/path-based clip sampling；插件 legacy sampler逐 track 扫 skeleton/path，core sampler另建 HashMap。compiled clip evaluator虽然更接近目标 authority，但尚未硬切这两套 fallback。PoseByName/WeightSet 必须接入收敛后的唯一 evaluator，不能再复制第三份采样与混合代码。

### 3.5 Generic document/jobs 为何只计 Partial

Animation document 已有单一 mutable source、monotonic revision、CAS-guarded swap、transaction apply/revert、current product 与 last-good product；通用 JobSystem 有 pending admission、keyed merge、batch reservation、estimated-byte/age budget、cancel 与 progress 基础。这些是 ED202-P1-09、P1-15 的可复用实现。

但 document kind/source/product 没有 Pose Library variant，recompile 仍在 source swap 路径同步执行；没有 `PoseLibraryCompileJob`、dependency build key、artifact validator、atomic generation publication、cook/install 或 Pose transaction command。通用基础不能让固定 field `.commit` 自动成为 authoring transaction。

## 4. P1：Pose Library 生产差距

以下 18 项与 Editor81 的 `POSE-P1-001..018` 一一对应，不增加 canonical finding 数。

### ED202-P1-01 · Open · 没有 canonical Pose Library source document

建立 versioned `AnimationPoseLibrarySourceDocument`，包含 asset/document/source revision、schema version、rig/skeleton dependency、authoring metadata 与稳定序列化。ZUI row、Graph 临时节点、Data asset 和 Runtime output 都不能成为 source of truth。

### ED202-P1-02 · Open · Pose entry 没有稳定 identity 与名称生命周期

引入 `PoseLibraryEntryId`、validated display name、stable order、tags/set membership、duplicate policy 与 rename/remove/reorder reference repair。Runtime、transaction、trace 与 preview 按 ID/generation 寻址；名称只作为 validated lookup key。

### ED202-P1-03 · Open · 没有 extraction recipe 与 provenance

引入 `PoseExtractionRecipe`，覆盖 source clip/time/sample mode/source generation、reference pose、viewport snapshot、Control Rig bake、mirror/retarget 与 update policy。来源变化必须能报告 stale，并区分 re-extract、keep override 与 reject。

### ED202-P1-04 · Open · Skeleton/Rig/Retarget/Mirror dependency closure 缺失

资产与 build key 必须冻结 source/target rig identity、stable target table、retarget profile、mirror table 及各自 generation。missing/renamed bone、hierarchy mismatch、provider reload 与 world replace 必须产生 typed diagnostic，不得按名字静默跳过。

### ED202-P1-05 · Open · Pose channel schema 仍只有 bone transform DTO

Pose entry 需要 typed transform、scalar/vector curve、attribute 与 variable-width morph channel，并记录 sparse influence 与 width。未知 channel、重复 target、type/width mismatch、NaN/Inf 在 compile 前拒绝；glTF morph animation 必须进入相同 channel authority。

### ED202-P1-06 · Partial · 有 generic reference delta/additive math，但无 Pose full/base/reference contract

现有 graph/SoA 能做 reference delta、override 与 additive 数学，是可复用 kernel。仍需定义 full local/component pose、reference-relative delta、selected-base delta、mesh/retarget reference、base deletion/change 与可撤销重算；当前按骨名/顺序比对不能成为 artifact contract。

### ED202-P1-07 · Partial · 有局部 blend oracle，但权重政策在三条路径分裂

需要统一 single pose clamp、multi full normalization、additive negative/overdrive、zero-total、missing contribution、curve/morph、quaternion hemisphere/order 与 deterministic accumulation。先消除 PoseBuffer、graph base、graph additive 的不同 validation/weight 语义，再开放 PoseWeightSet。

### ED202-P1-08 · Open · 没有 Pose semantic compiler 与 sparse prepared artifact

`PoseLibraryCompilePlan` 必须验证 source/dependency，生成 dense target indices、per-target influence ranges、pose/curve/morph payload、ID/name lookup、base/additive metadata、source map 与 diagnostic。`PreparedPoseLibrary` immutable、自包含，runtime 不读取 mutable source。

### ED202-P1-09 · Partial · 有 generic revision/CAS/LKG/jobs，无 Pose build/publish/cook

复用 Animation document 与 JobSystem，但补 `PoseLibraryCompileJob`、source/schema/compiler/rig/retarget/mirror/channel/target-profile build key、cancel checkpoint、artifact validator、atomic CAS/LKG publish、cook strip/export/install 与 dependency provenance。旧 artifact 不能冒充 current。

### ED202-P1-10 · Open · 没有 qualified runtime handle 与 residency lifecycle

引入 world/provider/asset/artifact generation 限定的 `PoseLibraryRuntimeHandle`，覆盖 load/resident/evict/reload/unload/world replace/shutdown。旧 handle、跨 world handle、provider reload 与 asset generation mismatch 必须 fail-close。

### ED202-P1-11 · Partial · 有 compiled clip pipeline/dense scratch，但没有唯一 PoseByName/WeightSet evaluator

先硬切两套 legacy manager sampler 与 string/AoS fallback，再在唯一 evaluator 加入按 entry ID/name、multi-pose `PoseWeightSet`、mask 与 base input 的 request。输出沿用 pooled dense pose page；Editor 不得建立私有 evaluator。

### ED202-P1-12 · Partial · Clip error/diagnostic typed，Pose failure/fallback/receipt 仍为空

建立 `PoseEvaluationReceipt`，区分 accepted、missing/ambiguous pose、rig mismatch、stale artifact、evicted、provider reload、budget exceeded 与 terminal failure，并记录 resolved entry/generation、weights、fallback 与 diagnostic identity。reference pose fallback 必须显式且可观察。

### ED202-P1-13 · Partial · 已有 Arc snapshot 与 Scene/Physics/Render入口，但 consumer 仍按名字半成功

Graph、State、Gameplay、Scene、Physics、deformation、network/replay 必须消费同一 prepared generation 与 receipt。保留共享 snapshot publication；硬切骨名 AoS、重复名字 first-wins、missing 静默跳过、固定 weight 与写回错误吞掉。

### ED202-P1-14 · Open · 没有真实 Pose Library document/toolkit

建立 `PoseLibraryEditorSession` 与 dynamic projection，显示真实 asset identity、entry/source/tags/channels、base/additive、artifact currentness、diagnostics 与 runtime generation。固定 `PL_Combat` 只能作为明确 demo fixture，不能进入 production authority。

### ED202-P1-15 · Partial · 通用 Animation transaction/CAS 可复用，Pose command 全部缺失

Add/Extract/Update/Duplicate/Rename/Delete/Reorder、Set Base、Convert Additive、Mirror、Retarget、Tag 与 Update From Source 都必须使用 expected revision、document-scoped history、undo/redo、dirty/savepoint 与 atomic save。失败不能留下半更新引用或半编译 artifact。

### ED202-P1-16 · Open · Preview 仍是固定 queued 文本

PreviewWorld 必须运行同一 `PreparedPoseLibrary` 与 evaluator，支持 weight scrub、single/multi pose、full/additive、reference/base/difference、mirror/retarget 与 bone/curve/morph diagnostics；显示 source/artifact generation、fallback 与 budget，不得建立 preview-only 数学。

### ED202-P1-17 · Open · Import/Reimport/Batch Extraction 与 dependency invalidation 未闭环

需要从 Clip/FBX/glTF/Control Rig/viewport 批量提取、命名映射、mirror/retarget recipe、source revision 与可审计 reimport diff。手工 override 不能被静默覆盖；morph weights 不得继续被 bone-track importer 丢弃。

### ED202-P1-18 · Partial · 有局部数值/分配/target 测试，无 Pose 纵向资格

保留 PoseBuffer golden/allocation、target table 与 snapshot 基础测试，但补 schema/migration、full/additive/base 数值 oracle、rename/remove reference、malformed artifact、reload/evict/shutdown、1/100/1k pose/target/evaluation、end-to-end allocation-free、cook/reopen 与跨平台资格。局部容器微基准不能代替完整 pipeline profile。

## 5. P2：平台扩展与超越目标

### ED202-P2-01 · Open · Corrective pose、pose driver 与 RBF 缺失

基础 Pose Asset 稳定后再增加 driver input、RBF solver、corrective set、LOD 与 diagnostic；高级 solver 保持独立 artifact/owner，不污染 core named-pose schema。

### ED202-P2-02 · Open · 语义 taxonomy、search 与质量分析缺失

支持 tag hierarchy、similarity/dedup、thumbnail、coverage/outlier 与 authoring lint，并复用 Editor79 特征基础，不混淆 authored Pose Library 与运行时 Pose Search database。

### ED202-P2-03 · Open · Constraint-aware mirror/retarget 批处理缺失

加入 contact/constraint-aware mirror、retarget quality report、batch repair 与可撤销 automation；每次结果保留 recipe、dependency generation 与质量 diagnostic。

### ED202-P2-04 · Open · 大型 Pose Library streaming/GPU 路径缺失

用 1/100/1k/10k 数据评估 page residency、LOD、pose sharing、compressed sparse payload、GPU decompression 与 budget telemetry。当前 GPU skinning consumer 不等于 Pose Library streaming。

### ED202-P2-05 · Open · LiveLink/Capture/ControlRig/ML Deformer interchange 缺失

定义外部 capture、Control Rig bake、procedural pose 与 ML deformer interchange；Control Rig/ML 核心能力继续由各自 owner 负责，Pose Library 只拥有可验证的交换/提取合同。

## 6. 五套参考源码裁决

### 6.1 Unreal：主参考覆盖独立资产、稀疏数据、运行时节点与 Editor 生命周期

`UPoseAsset` 是独立 Animation Asset。`FPoseDataContainer` 保存 pose names、tracks、curve metadata、local/source pose、track-to-bone map 与 per-track pose influences；runtime 只遍历有 influence 的 sparse 数据。资产保留 source animation、raw-data GUID、retarget source/mesh/reference pose、additive flag 与 base pose index，并提供 full/raw/base pose 转换与 skeleton remap。

`FAnimNode_PoseByName` 在名称改变时解析并缓存 pose index，求值只更新 weight，missing 时回 reference pose；PoseBlendNode 把 source curves 映射到 Pose Asset weight，并保留 curve ignore/blend policy。`SPoseEditor` 提供真实 pose/curve list、过滤、preview override、add/update/delete/rename 与 transaction/`Modify()`；factory 从 source sequence 和 pose names 创建资产。Zircon 应学习 source/runtime/editor 分层与 failure model，不复制 UObject、宏、deprecated 字段或名字作为唯一 stable identity。

### 6.2 Fyrox：可复用 pose output 与事件政策次参考

Fyrox `AnimationPose` 保留 node/property map 与 root motion，`reset()` 清 value 而保留容器，`clone_into()` 复用已占用 entry，blend node 自持 `output_pose` 并每次 reset/reuse。Blend/BlendSpace 还显式解析 parameter weight 与 animation event collection strategy。它证明热路径输出应复用且事件/权重政策显式，但没有 Unreal 等价 Pose Asset authoring 产品。

### 6.3 Bevy：prepared graph 与 typed curve/morph evaluator 次参考

Bevy serialized graph 区分 Clip/Blend/Add 与 mask，`ThreadedAnimationGraph` 缓存 postorder、sorted edges 与 computed masks。`AnimationCurveEvaluator` 统一 stack/blend register/commit 合同；`WeightsCurveEvaluator` 为 variable-width morph weight 独立维护 packed stack、blend register、width 和 `MorphWeights` commit。它证明 transform、curve、morph 应进入同一类型化求值框架，而不是把 morph 拒绝在 bone track 之外。

### 6.4 Godot：observable library lifecycle 与 graph reference repair 次参考

`AnimationLibrary` 验证名称，支持 add/remove/rename/has/get/list，并发布 added/removed/renamed/changed signal；blend tree 测试证明 rename/remove 会修复连接。Animation 还有 blend-shape track 与 changed notification。它适合校验名称生命周期、observable mutation 与 reference repair，但不提供 Pose Asset sparse/additive/base runtime 设计。

### 6.5 Unity Graphics：只约束 deformation consumer

Graphics 仓不是 Mecanim/Pose Asset authoring 源码。Linear Blend Skinning node 要求 stable bone indices/weights 与 skin matrix buffer；Compute Deformation node读取 deformed vertex position/normal/tangent，并在可用配置下接 current/previous deformation。它只约束 Pose 输出到 renderer 的稳定 identity/history/reset，不可作为 Pose source/compiler 主参考。

## 7. 目标架构与唯一 authority

```text
Pose source / import / capture / Control Rig bake
        |
        v
AnimationPoseLibrarySourceDocument
  + PoseLibraryEntryId / validated name / tags
  + PoseExtractionRecipe / source generation
  + typed transform / curve / morph channels
  + full/additive/base/reference + rig dependencies
        |
        v
Canonical Animation Compiler
  -> PoseLibraryCompilePlan / diagnostics / deterministic build key
  -> dense target table + sparse influence ranges + source map
        |
        v
Immutable PreparedPoseLibrary
  -> validator / LKG / CAS / cook / generation install
        |
        v
Qualified PoseLibraryRuntimeHandle
        |
        v
PoseEvaluationRequest(PoseWeightSet, mask, base input)
        |
        v
Single Animation evaluator -> pooled dense PosePage
        |
        v
Atomic PoseEvaluationReceipt + shared frame snapshot
        |
        +--> Graph / State / Gameplay / Network / Replay
        +--> Scene / Physics / Deformation / Render current-previous
        +--> Editor PreviewWorld / Trace Debugger
```

必须只有一份 source schema、一份 semantic compiler、一份 artifact validator、一份 runtime evaluator 和一份 receipt ABI。Editor 只拥有 document/toolkit/job/preview projection；Gameplay 只提交 typed request；Scene/Physics/Render 只消费 stable target/dense page 与 generation snapshot。

硬切要求：真实 provider 成立前隐藏或标记 Unavailable 当前入口；删除固定 queued/Ready/count/tag 成功措辞；禁止按 focus/control ID/display name/骨名寻址 runtime；删除两套 legacy sampler、AoS string target 与错误吞掉路径，不保留兼容 shim。

## 8. 重构里程碑

### ED202-M0 · Capability truth、owner、RED corpus 与 benchmark protocol

把 Preview/Apply/field commit 降为 Unavailable/Prototype；冻结 owner、invalid corpus、reference revision、semantic scenarios、quality metrics 与 hardware/workload protocol，加入“无 artifact/receipt 不得显示成功”的 RED contract。

### ED202-M1 · Stable source schema、identity 与 provenance

实现 Pose Library、entry ID/name lifecycle、typed channels、extraction recipe、rig/retarget/mirror dependency、serialization/migration 与 roundtrip。

### ED202-M2 · Canonical compiler、sparse artifact 与数值语义

在唯一 Animation compiler 中实现 full/additive/base/reference、统一 weight policy、dense/sparse layout、curve/morph、source map、build key、diagnostic 与 validator。

### ED202-M3 · Job、LKG/CAS、cook 与 generation publication

接入 EditorJob authority，补 dependency-aware admission、cancel、progress、atomic publish、last-good、cook/export/install、resident metadata 与 stale generation fail-close。

### ED202-M4 · Runtime handle、唯一 evaluator 与 dense PosePage hard cut

实现 qualified handle/request/receipt、PoseByID/Name/WeightSet、pooled dense page、budget/fallback；删除 legacy manager sampler、骨名 AoS、string target 与 scene first-wins。

### ED202-M5 · Atomic consumers、snapshot 与 deformation history

把 Graph/State/Gameplay/Scene/Physics/Render 接入同一 receipt/generation，传播 partial/missing/writeback failure，定义 current/previous deformation reset 与 network/replay identity。

### ED202-M6 · 真实 Editor toolkit、transaction 与 PreviewWorld

动态 document projection 替换固定数据，接入 command/history/save；PreviewWorld 运行同一 artifact/evaluator；trace 以 stable source ID 映射 entry/channel。

### ED202-M7 · Import/Reimport、mirror/retarget 与批量提取

打通 Clip/FBX/glTF/Control Rig/viewport recipe、morph/curve、source diff、override policy 与 dependency invalidation。

### ED202-M8 · Fault、scale、performance 与跨引擎资格

完成 malformed/stale/reload/evict/shutdown、1/100/1k/10k entry/target/evaluation、end-to-end allocation/residency、cook/reopen、cross-platform determinism 与同语义 benchmark。

## 9. 48 个资格门

状态说明：`Partial` 只承认可复用共享基础，不表示 Pose Library gate 已通过；`Pass` 必须有 Pose Library vertical slice 的动态证据。

### 9.1 Source、Identity 与 Compiler

| Gate | 当前 | 资格条件 |
|---|---|---|
| POSE-G-01 | Fail | Pose Library 可 Create/Open/Save/Reopen，asset/document/source revision 与 entry ID 稳定。 |
| POSE-G-02 | Fail | Add/Duplicate/Rename/Delete/Reorder 在引用、tag、selection 与 undo/redo 中保持 identity。 |
| POSE-G-03 | Fail | 重名、空名、非法 tag、dangling base、重复 target 与非有限值被 typed diagnostic 拒绝。 |
| POSE-G-04 | Fail | Clip/reference/viewport/Control Rig 提取保存 source generation 与 recipe，可检测 stale。 |
| POSE-G-05 | Fail | Skeleton/Rig/Retarget/Mirror generation 进入 source/build key，mismatch fail-close。 |
| POSE-G-06 | Fail | transform、scalar/vector curve 与 variable-width morph channel 可 roundtrip/compile。 |
| POSE-G-07 | Fail | full/additive/base/reference 转换有 Pose Asset 数值 oracle，base 变化不隐式破坏资产。 |
| POSE-G-08 | Fail | compiler 输出自包含 `PreparedPoseLibrary`，runtime 不读 mutable source/Editor state。 |

### 9.2 Artifact、Load 与 Lifecycle

| Gate | 当前 | 资格条件 |
|---|---|---|
| POSE-G-09 | Fail | build key 覆盖 schema/compiler/source/rig/retarget/mirror/channel/target profile generation。 |
| POSE-G-10 | Partial | generic Animation document 能保留 LKG；Pose compile/cancel/stale/cook/currentness 仍不存在。 |
| POSE-G-11 | Fail | artifact 含 dense target、sparse influence、ID/name lookup、curve/morph 与 provenance。 |
| POSE-G-12 | Fail | malformed/unknown schema/truncated payload/invalid range 在 publish 前 fail-close。 |
| POSE-G-13 | Fail | load/resident/evict/reload/unload 有 qualified handle 与 ordered receipt。 |
| POSE-G-14 | Fail | 旧 handle、跨 world、provider reload 与 asset generation mismatch 不误命中新对象。 |
| POSE-G-15 | Fail | world replace/shutdown 终结 handle、lease、pending request 与 residency。 |
| POSE-G-16 | Fail | cook/export/install/reopen 只发布 validated generation 并保留 provenance。 |

### 9.3 Evaluation 与数值语义

| Gate | 当前 | 资格条件 |
|---|---|---|
| POSE-G-17 | Fail | PoseByID/Name 命中同一 entry，missing/ambiguous name 产生 typed receipt。 |
| POSE-G-18 | Fail | single full pose clamp 与 reference contribution 符合冻结政策和 Pose oracle。 |
| POSE-G-19 | Partial | generic graph 有 per-bone normalization/zero-valid-input行为；Pose multi-weight/missing policy 未冻结。 |
| POSE-G-20 | Fail | additive negative/overdrive、base input 与 reference delta 结果确定。 |
| POSE-G-21 | Partial | generic graph/SoA 有 quaternion sign/shortest-path测试；Pose accumulation/cross-platform oracle 未建立。 |
| POSE-G-22 | Fail | morph variable width、curve type mismatch 与 mask 组合不越界、不丢 channel。 |
| POSE-G-23 | Partial | dense target/PosePool局部稳态零分配；完整 evaluator/consumer 仍每帧 Vec/name allocation。 |
| POSE-G-24 | Fail | request/receipt 原子记录 resolved generation、weight、fallback、budget 与 diagnostic。 |

### 9.4 Runtime Integration 与 Failure

| Gate | 当前 | 资格条件 |
|---|---|---|
| POSE-G-25 | Fail | Animation Graph 使用同一 prepared artifact/evaluator，不复制 pose 数组/混合数学。 |
| POSE-G-26 | Fail | State/Gameplay 按 qualified handle 消费并区分 reject/fallback/terminal failure。 |
| POSE-G-27 | Fail | Scene apply 使用 stable target并传播写回错误，不按骨名吞掉半失败。 |
| POSE-G-28 | Fail | renderer current/previous deformation 在 pose jump/reload/teleport 有明确 reset policy。 |
| POSE-G-29 | Fail | network/replay/save/load 携带 asset/artifact generation 与 entry ID，rename 不破坏重放。 |
| POSE-G-30 | Fail | provider reload、evict 与 callback 重入期间无 UAF、旧 generation 写回或半 publish。 |
| POSE-G-31 | Fail | missing bone/channel、rig mismatch 与 unavailable morph consumer fallback 可配置、可观察。 |
| POSE-G-32 | Fail | budget exceeded 返回 continuation/failure，不提交不完整 pose。 |

### 9.5 Editor、Transaction 与 Preview

| Gate | 当前 | 资格条件 |
|---|---|---|
| POSE-G-33 | Fail | workspace 显示真实 asset/document/revision/currentness，不再显示固定 `PL_Combat`。 |
| POSE-G-34 | Fail | 所有 Pose authoring command 走 transaction/CAS，失败不产生半更新或脏 artifact。 |
| POSE-G-35 | Fail | selection/list/details/tags/source/curve/diagnostic 绑定同一 document generation。 |
| POSE-G-36 | Fail | Set Base/Convert Additive/Mirror/Retarget/Update Source 可 undo/redo/save/reopen。 |
| POSE-G-37 | Fail | rename/delete 更新或拒绝 Graph/Gameplay/asset 引用并生成 reference report。 |
| POSE-G-38 | Fail | PreviewWorld 运行同一 prepared generation，single/multi/full/additive 与 Runtime 一致。 |
| POSE-G-39 | Fail | preview 显示 reference/base/difference、bone/curve/morph contribution 与 fallback。 |
| POSE-G-40 | Fail | source stable ID 可从 runtime trace 定位回 entry/channel，stale trace 明确。 |

### 9.6 Import、Fault、Scale 与交付

| Gate | 当前 | 资格条件 |
|---|---|---|
| POSE-G-41 | Fail | batch extraction/reimport 保留 recipe、命名映射、override 与 source diff。 |
| POSE-G-42 | Fail | glTF/FBX morph 与 custom curve 进入 typed channel，不因 bone-track 限制丢失。 |
| POSE-G-43 | Fail | malformed source、missing dependency、reload/evict/shutdown 矩阵无 panic/UAF/幽灵 handle。 |
| POSE-G-44 | Fail | 1/100/1k/10k entries/targets 有 compile time、artifact size、resident memory 与 query curve。 |
| POSE-G-45 | Partial | 局部 PoseBuffer 测试证明池内 blend 零分配；完整 active evaluation/scene apply scale 未测且仍分配。 |
| POSE-G-46 | Fail | Create/Extract/Edit/Undo/Save/Compile/Preview/Reload/Reopen 真实用户流通过。 |
| POSE-G-47 | Fail | Windows/目标平台 serialization、determinism、cook 与 runtime receipts 一致。 |
| POSE-G-48 | Fail | 与 Unreal 同语义质量/CPU/内存/延迟 benchmark 可复现；未测不得宣称超越。 |

## 10. 实施顺序、停止条件与复核边界

1. 先关闭 capability truth：没有真实 provider 时隐藏或 disabled Pose Library 入口，固定 `queued/Ready/count/tag` 不得继续作为成功反馈。
2. 先收敛 Editor197/198 与 Runtime08C/Plugins13 的唯一 compiler/evaluator、dense pose/stable target/consumer，再引入 Pose typed variant。
3. Source -> compiler -> artifact -> qualified handle -> request/receipt -> consumers 顺序不可颠倒；不允许先给 ZUI action 接临时 Vec、Data asset 或名字查找。
4. transform、curve、morph 必须进入同一 typed compiler/evaluator；Scene/Physics/Render 只消费稳定 dense output，不反向拥有动画资产。
5. 每个里程碑先补 RED contract 与 invalid/fault corpus，再实现最小 vertical slice；数值、lifetime、reload、跨模块与 scale 风险决定测试扩张范围。
6. 开始实现前重算本报告 fingerprint 与 HEAD，复核 Runtime07 frame snapshot 及 Animation 插件 open failure；不以协调器状态作为推进前提。

本报告退出条件不是 ZUI 可打开、Apply 显示成功、PoseBuffer 局部测试通过或 frame snapshot 已用 `Arc`，而是 POSE-G-01 至 POSE-G-48 全部获得 Pose Library vertical slice 的动态证据，且父报告 compiler/runtime/stable-target/transaction/preview 前置同时满足。在此之前，Pose Library 只能标为 Unavailable/Experimental。
