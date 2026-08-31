---
title: Editor Animation Montage、Section、Slot、Segment、Notify、Branching Point、Sync、Root Motion、Runtime Playback、Preview 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor201
review_date: 2026-08-28
baseline_head: 6350cb00b5a060f628c904f84703b3843ac404fa
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay/workbench_ability_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_montage_editor_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/index/workbench_extension_module_workspaces.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_runtime/src/core/framework/animation/asset/clip.rs
  - zircon_runtime/src/core/framework/animation/asset/sequence.rs
  - zircon_runtime/src/core/framework/animation/asset/state_machine.rs
  - zircon_runtime/src/core/framework/animation/clip_event_sampling.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/core/framework/animation/compiler/product.rs
  - zircon_editor/src/core/editing/animation_document/kind.rs
  - zircon_editor/src/core/editing/animation_document/asset.rs
  - zircon_editor/src/core/editing/animation_document/compilation.rs
  - zircon_editor/src/core/jobs/job.rs
  - zircon_editor/src/core/jobs/system/submission.rs
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compile.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/evaluate.rs
tests:
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/state_machine_interruption.rs
  - zircon_plugins/animation/runtime/src/state_machine/compiled/evaluate.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/198-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimCompositeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimCompositeBase.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimMontage.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimSync.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSync.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/ActiveMontageInstanceScope.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/ActiveMontageInstanceScope.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/Abilities/Tasks/AbilityTask_PlayMontageAndWait.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/Abilities/Tasks/AbilityTask_PlayMontageAndWait.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/Abilities/GameplayAbilityRepAnimMontage.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Private/GameplayAbilityRepAnimMontage.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimMontagePanel.cpp
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private/SAnimMontageSectionsPanel.cpp
  - dev/godot/scene/animation/animation_player.h
  - dev/godot/scene/animation/animation_player.cpp
  - dev/godot/scene/animation/animation_blend_tree.h
  - dev/godot/scene/animation/animation_blend_tree.cpp
  - dev/godot/scene/animation/animation_mixer.h
  - dev/godot/scene/animation/animation_mixer.cpp
  - dev/Fyrox/fyrox-animation/src/signal.rs
  - dev/Fyrox/fyrox-animation/src/machine/layer.rs
  - dev/Fyrox/fyrox-animation/src/machine/mod.rs
  - dev/Fyrox/fyrox-animation/src/machine/event.rs
  - dev/Fyrox/fyrox-animation/src/machine/node/play.rs
  - dev/bevy/crates/bevy_animation/src/animation_event.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/ComputeDeformNode.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation/LinearBlendSkinningNode.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Samples~/RendererShaderUserValue_Common/Scripts/VertexAnimationTextureBaker.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/ShaderPass/MotionVectorVertexShaderCommon.hlsl
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/80-editor-animation-montage-section-slot-segment-notify-branching-point-sync-root-motion-runtime-playback-preview-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/80-editor-animation-montage-section-slot-segment-notify-branching-point-sync-root-motion-runtime-playback-preview-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Animation Montage、Section、Slot、Segment、Notify、Branching Point、Sync、Root Motion、Runtime Playback、Preview 与 Product Integration 当前源码复核

## 1. 结论

当前 Zircon 仍没有 Animation Montage 产品，也没有可供 Gameplay、Animation Graph 或角色运行时使用的 one-shot action stack。唯一专用产品表面是 233 行 `workbench_extension_montage_editor_workspace.zui`：`AM_DashAttack / AM_Combo_01 / AM_Evade`、`Intro Section 0.00-0.38s`、`Combo Branch`、`Notify HitPause 0.46s`、`Slot UpperBody`、`Extracted 2.8m forward` 与 `4 sections / 3 notifies`均为固定文本。20 个标准化 action 最终只选择 workspace/tab/row、打开通用 dropdown 或写入固定 feedback；`.commit`、Preview 与 Apply 不产生 asset mutation、transaction、compile job、runtime request 或 operation receipt。

对 `zircon_runtime`、`zircon_plugins`、`zircon_app` 与 `zircon_interfaces` 的 Montage 专属 runtime/compiler 类型检索为 0；另扫描 2,115 个未跟踪 Rust/ZUI 文件，同样没有 `AnimMontage / AnimationMontage / MontageInstance / MontageCompile / PreparedAnimationAction / BranchingPoint / JumpToSection / SlotGroup / SyncGroup` 命中。shared Animation source/product 仍只有 Sequence、Graph、StateMachine。Clip event schema 仍只是 optional target、event string、time 与 optional payload；没有 Section、Slot、Segment、Notify State、Branching Point 或 Action 实例身份。

旧 Editor80 有三处需要正向校正，但都只是 Montage 的共享前置：Animation document 已有 revision/CAS/current product/LKG；Clip Event 已有按事件数、字节数和播放跨度的有界可恢复采样及 replacement epoch；状态机 Trigger 只报告被选 transition 的触发器，并在目标 pose 可用、event admission 成功后才消费。集成测试覆盖 exit gate 等待、缺 pose 重试、interruption source pose 等待和 event queue backpressure 重试。这些能力不能表达 Section graph、Slot action stack、Branching Point、Root Motion、Sync、Gameplay terminal lifecycle 或 Montage handle，因此只能把 ED201-P1-09、P1-13、P1-14 标为 Partial。

Editor80 的 18 项 P1 当前重判为 **15 Open / 3 Partial / 0 Closed**，5 项 P2 为 **5 Open / 0 Partial / 0 Closed**；48 个资格门为 **46 Fail / 2 Partial / 0 Pass**。canonical finding 数不重复增加。本轮不新增父报告 P0：Editor14 的 capability truth、Editor197 的唯一 compiler/runtime authority、Editor198 的 Clip/Event/Root Motion/Sync/Playback transaction、Runtime08C 与 Runtime08G 的动画/玩法平台仍是上游 owner。

本轮只做静态 current-source review 与文档建账，不修改 production Rust/ZUI，不运行 Cargo、Editor、GUI/GPU、cook、runtime playback、Gameplay Ability、network、preview、fault/soak/profile 或同语义跨引擎 benchmark。不能据此宣称 Montage 可用、性能达标，更不能宣称性能或表现超过 Unreal。Tooling 按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

## 2. 审查边界、owner 与冻结语料

### 2.1 本报告唯一纵向边界

本报告只拥有：

```text
AnimationMontageSourceDocument
  -> MontageCompilePlan
  -> PreparedAnimationAction
  -> MontagePlayRequest
  -> MontageInstanceHandle
  -> MontageFrameReceipt
  -> Gameplay / Movement / Renderer / Editor Preview consumers
```

- Editor14 拥有高级动画 workspace 的 capability truth、通用 toolkit/preview/compile 真实性。
- Editor09/131 拥有 background job admission、cancel、progress、scope、shutdown 与 durable result 总合同。
- Editor21 与 Runtime08G 拥有 Ability task、Cue、prediction、replication 与 Gameplay authority。
- Editor63/184 拥有 transaction/history/savepoint/document scope 与 async operation 总合同。
- Editor69/190 拥有 PreviewWorld、time domain、pause/step 与 subsystem admission。
- Editor196 拥有 Timeline/Curve/transport 通用交互。
- Editor197 拥有唯一 Animation compiler/runtime authority；Montage 必须成为 typed variant，而不是第四套 Editor 私有 evaluator。
- Editor198 拥有 canonical Clip/Event/Root Motion/Sync、prepared artifact 与 playback transaction；本报告只扩展 Montage 特有的 Section/Slot/Branching/action semantics。

### 2.2 Currentness、负证据与已知验证阻塞

- HEAD 锚点为 `6350cb00b5a060f628c904f84703b3843ac404fa`；结论以 2026-08-28 当前磁盘内容为准。审查期间从 `e2d29a4a...` 前进的三次提交只修改 `tools/zircon_export`、对应测试与 `docs/plans/zircon_plugins/13`，未触及本报告冻结语料。
- 当前工作树有大量共享在途与未跟踪文件；本轮不回退、不覆盖、不归属这些变化。Montage ZUI、feedback、navigation/spec 与 preview action inventory 本身也处于共享修改状态，报告按磁盘当前内容取证。
- tracked production 的 Montage runtime/compiler 精确语义为 0；2,115 个未跟踪 Rust/ZUI 文件的同一检索也为 0。Editor 中 plain `Montage` 命中全部属于固定 ZUI、binding、route、feedback 与静态 action inventory。
- `docs/plans/optimize/zircon_editor/14/failure-2026-08-24-animation-editor-zui-deletion-closure.md` 记录旧 animation editor ZUI hard-cut 的广域 Cargo 阻塞；它不改变本轮静态 Montage 结论。
- Animation 插件仍有 fallback evaluator divergence、frame diagnostics omission 与 dynamic runtime animation module duplication 等 open failure 记录。即便未来把 Montage 接入现有 pipeline，也必须先收敛唯一运行时 authority，不能依赖当前重复 manager/evaluator 路径取得虚假通过。

### 2.3 冻结语料与 fingerprint

统计口径：相对路径转小写、`/` 规范化并排序；每个文件取 SHA-256，再按 `path + NUL + lowercase file hash + LF` 聚合集合 fingerprint。行数、非空行与 bytes 均按当前磁盘文本计算。

| 范围 | 文件 / 行 / 非空行 / bytes | 本轮证据 | fingerprint |
|---|---:|---|---|
| Zircon Editor/product | **11 / 5,303 / 5,086 / 255,909** | 固定 ZUI、binding、route、feedback 与静态 action inventory | `393548fb082c827bfcf2203f4612ec8917b6f482ce39413be9cc12ed08c63c34` |
| Zircon shared prerequisite foundation | **17 / 3,765 / 3,443 / 136,809** | source schema、LKG、jobs、event budget、Trigger evaluation/commit 与 integration tests | `7b3f557ea42a7c5ef3d09541a033a122af4088ee2886504c8844b4cf3c6b6089` |
| Zircon deduplicated focused set | **28 / 9,068 / 8,529 / 392,718** | 上述两组按 normalized path 去重 | `173d74e4536f6e21d3af8c05650bf76e9961c2923ff99e8f0c77857ba9da4ba6` |
| Unreal selected set | **16 / 15,925 / 13,374 / 608,443** | Segment/Section/Slot/Montage instance、sub-step、sync、ability、replication 与 Editor transaction | `276be7b3a0d450981c5bb87e3e534760e9fd13d86377c0441b17ac931e4b7995` |
| Godot selected set | **6 / 6,929 / 5,830 / 261,884** | one-shot、section/marker 正反播放、queue/blend/seek 与 root-motion consumer | `a783e9b7c86c6b720f92161e7b70b8a64b58bb93d1552d747779212432bbdeb7` |
| Fyrox selected set | **5 / 1,391 / 1,244 / 53,942** | signal、layer/mask、active transition、event provenance 与 play node | `e2ad696ed9a9a00ed9731a972d379287cb8734afb11b590e42ca1cdae0ee7aba` |
| Bevy selected set | **3 / 2,087 / 1,877 / 78,138** | typed player、transition set 与 animation event traversal | `381d9e91000eb8161ebef1c65bbf221f6b061d5992638e294015c2de89d203ee` |
| Unity Graphics selected set | **4 / 835 / 720 / 34,510** | current/previous deformation、skinning、VAT 与 motion-vector consumer | `66b3099de28325ae88d8812aa69ff2311c97e1b0f63df99295f28873b02028a0` |
| Five-engine reference total | **34 / 27,167 / 23,045 / 1,036,917** | 五组显式路径去重 | `b515c56e87ace909dec50017ce694a2e3208abd608b62f2a987287b299badf45` |

fingerprint 只是本轮静态输入 receipt，不是未来 Montage build key、source revision 或 runtime generation。

## 3. 当前真实实现与旧报告校正

### 3.1 可见 workspace 仍是固定 projection

ZUI 第 77-97 行固定三行列表，第 150-186 行固定 Intro、Combo Branch、HitPause、Root Motion 与 output，第 204-232 行固定 Montage dropdown、`Slot: UpperBody` 和三种 blend option。这里没有 AssetId、DocumentId、SourceRevision、SectionId、SlotId、SegmentId、NotifyId、CompileGeneration、InstanceHandle 或 ReceiptId。

20 个 binding 覆盖 open、3 tab、8 row、preview/apply 和 6 个 field edit/commit。navigation spec 只把 action 映射到 control ID；field edit 最多打开通用 dropdown，field commit 不读取 typed domain value，也没有 expected revision。feedback 第 169-193 行固定发布 `AM_DashAttack`、`UpperBody slot`、`4 sections / 3 notifies` 与 `2.8m forward`。静态 action 测试只证明 wiring 存在，不证明资产、编译或播放执行。

### 3.2 Shared source/compiler 尚无 Montage variant

`AnimationAuthoringDocumentKind` 与 `AnimationAuthoringAsset` 只有 Sequence、Graph、StateMachine；`AnimationCompileSource` / `AnimationCompileProduct` 也只有三类。`AnimationClipAsset` 有 bone tracks 与 point-like event tracks，`AnimationSequenceAsset` 是 entity/property timeline。它们不能表达 Section graph、Slot Group、Segment time mapping、Notify State、Branching Point、action blend/interruption 或 gameplay window。

因此不能把 Montage 临时塞进 `event: String`、状态机 Trigger、Graph node label 或 Editor 私有 `Vec`。正确做法是先扩展 canonical source/schema/compiler，再生成 immutable prepared action，运行时只消费 artifact 与 qualified command。

### 3.3 当前三项共享基础为何只能是 Partial

`AnimationDocumentCompilation` 在 source revision 编译失败时保留 last-good product，通用 `EditorJobSystem` 有 admission、keyed merge、batch reservation、cancel 与生命周期事件。这可以承载未来 Montage compile job，但当前 compilation 同步发生在 document write path，source/product 没有 Montage variant，job 也没有 dependency build key、领域 artifact、atomic publication 或 cook install。

Clip Event sampler 有 `max_events / max_event_bytes / max_playback_span_seconds`、cursor 与 heap 顺序；queue admission 区分 Admitted/Deferred/RejectedOversized，并使用 replacement epoch。它只支持 `to > from` 的正向范围；record publication 把 `clip` 写成 `None`，stable EventId、source generation、reverse/seek policy、Notify State、Branching Point 与 single-delivery receipt 仍缺失。

compiled state machine 只消费被选 transition 的 Trigger；pipeline 对 deferred entity 恢复旧 graph/state-machine time，并推迟 active-state/Trigger update。现有集成测试证明缺 target pose 或 event admission backpressure 时可重试。然而 `tick_animation_world` 仍分阶段发布 event、player state、scene transform、pose snapshot 与 playback time，不是包含 pose/event/root-motion/sync/gameplay 的单一原子 frame receipt；它也没有 Montage action handle 或 terminal disposition。

### 3.4 父 owner 与不重复计数

本报告不把 shared Clip、state-machine、job 或 Gameplay 的既有缺陷改名重复登记。Editor80 保持 canonical owner；Editor201 只刷新事实、优先级状态、目标架构和资格门。实施时必须按 Editor197 -> Editor198 -> Runtime08C/08G -> Editor201 的 dependency 顺序收敛。

## 4. P1：Montage 生产差距

### ED201-P1-01 · Open · 没有 canonical Montage source、稳定身份、版本与 revision

必须建立 versioned `AnimationMontageSourceDocument`，为 Section、Slot、Segment、Notify、Curve 与 metadata 分配稳定 ID；序列化、migration、clone、rename、duplicate、diff 与 merge 不得依赖显示名或数组下标。当前固定文本不能成为 source of truth。

### ED201-P1-02 · Open · Section 不是可验证的有向图

Section 需要 start/end、default next edge、runtime override、loop/cycle policy、entry/terminal 与 stable edge identity。compiler 必须拒绝重名、零长、越界、dangling edge 和无预算循环，并把 source ID 映射保留到 artifact/trace。

### ED201-P1-03 · Open · Slot Group、Slot、Layer、Mask 与 Skeleton 兼容合同缺失

当前 `Slot UpperBody` 只是字符串。需要 canonical catalog、stable SlotId/GroupId、Skeleton/Rig/AvatarMask generation、additive/base policy、bone ownership 与 reload invalidation；Editor、compiler、runtime 和 Gameplay 必须消费同一 catalog。

### ED201-P1-04 · Open · Segment 的 clip、range、rate、loop 与 time map 缺失

每个 Segment 需要 Clip generation、source start/end、action start、play rate、loop count、mirror/retarget、trim/slip 与正反向映射。compiler 生成 dense time map 和边界索引；非法 duration、NaN、负范围、zero rate 与越界 clip 必须 fail-close。

### ED201-P1-05 · Open · 多 Slot composition、additive、root-motion 与 deformation owner 未定义

需要明确同一 action 多 Slot 的 pose/curve composition 次序、base/additive normalization、mask overlap、root-motion 单一 owner 与 renderer current/previous pose reset。`Extracted 2.8m forward` 不能替代可消费的 root-motion delta contract。

### ED201-P1-06 · Open · Point Notify、Notify State、Branching Point 与 Gameplay Window 未类型化

当前 event string 不能表达 EventId、schema/version、begin/tick/end、branch-immediate、cancel window、trigger offset、mirror/loop policy、authority 或 dedup key。必须区分普通 deferred event、state lifetime 与能改变推进控制流的 branching event。

### ED201-P1-07 · Open · Blend、Profile、Inertialization 与 Interruption policy 缺失

需要 blend in/out、auto blend out、blend profile、inertialization、desired/actual weight、interruption source/target policy、callback order 与 failure disposition。所有 policy 必须在 artifact 中冻结并由同一 runtime state machine 执行。

### ED201-P1-08 · Open · 没有唯一 Montage semantic compiler 与 Prepared Action

Montage 必须成为 Editor197/198 canonical compiler 的 typed source/product，输出 self-contained dense section/slot/segment/event/root-motion/sync plan、source map 与 diagnostic。runtime 禁止读取 mutable source 或重新实现 Editor compiler 数学。

### ED201-P1-09 · Partial · 有通用 document LKG/jobs，但无 Montage build key、publication 与 cook

现有 revision/CAS/current/LKG 和通用 job admission 可复用。仍需 `MontageCompileJob`、dependency closure、target profile、deterministic build key、cancel checkpoint、CAS/LKG atomic publication、artifact validator、cook/export/install 与 generation-qualified hot reload。

### ED201-P1-10 · Open · 没有 Play Request、Instance Handle 与 terminal lifecycle

需要按 world/subject/artifact generation/slot/policy 寻址的 typed request，返回 accepted/rejected receipt 与 generational handle。实例必须有 Playing/BlendingOut/Completed/Interrupted/Cancelled/Failed terminal disposition，旧 handle 永不命中新实例。

### ED201-P1-11 · Open · Jump/SetNext/Seek/Rate/Pause/Stop 命令没有 ordered receipt

命令必须携带 instance generation、expected sequence、time domain 与 source ID，形成可重放的 ordered command log。按 asset 名、focused row 或 control ID 操作运行时实例都不允许。

### ED201-P1-12 · Open · Action stack、并发与 Slot 仲裁缺失

需要 bounded active registry、priority、concurrency group、replace/queue/reject/coexist、per-slot claims、interrupt propagation 与 cleanup。多个相同 asset 实例必须独立，world replace/provider unload/shutdown 必须终结所有 claims。

### ED201-P1-13 · Partial · 通用 event 有预算/continuation，但无 Section/Branching sub-step engine

现有 Clip Event 有界 cursor 与 admission backpressure 是可复用基础。Montage 仍需按最近 Section/Segment/Event/Branch boundary 分步推进，覆盖 large delta、reverse、loop、seek、rate flip、callback 改 position 与 branch storm，并同时限制 sub-step/event/time budget。

### ED201-P1-14 · Partial · Trigger/pose/event 重试有局部一致性，但无 MontageFrameReceipt

现有 Trigger 在缺 pose 或 deferred event admission 时不提前消费，是重要的 failure-retry 证据。它没有把 Slot pose、curve、event、root motion、sync、active state、terminal change 与 gameplay ack 封装为同 generation 的原子 receipt；当前分阶段 world mutation 也不能满足 callback 销毁/重入后的全链一致性。

### ED201-P1-15 · Open · Sync Group、leader/follower 与 marker handoff 缺失

需要复用 Editor198 canonical marker/sync clock，定义 role、leader selection、join/leave、leader loss、section jump/branch 后 resync、marker fallback 与 diagnostic。Montage 不得建立私有时钟或只同步 normalized time。

### ED201-P1-16 · Open · Gameplay Ability、network、prediction、replay 与 save/load 缺失

Ability task 必须绑定 instance handle 并精确一次终结；Cue/window 消费 typed event。复制与预测至少携带 artifact generation、instance sequence、section/position/rate/next edge/prediction key；generation mismatch、reject/correction/rollback 必须 fail-close 并最终收敛 pose/root motion/event/ability state。

### ED201-P1-17 · Open · Editor toolkit 没有 document projection 与 transaction

需要真实 asset browser、Section graph、Slot/Segment timeline、Notify/State/Curve tracks、Details、selection、diagnostic 定位、transaction/history/savepoint/dirty/save/reopen。所有 UI 投影绑定同一 document revision；不能从固定 ZUI 文本反向构造 domain state。

### ED201-P1-18 · Open · Preview、trace、fault 与 performance 资格体系缺失

PreviewWorld 必须运行同一 prepared artifact/runtime instance，支持 play/pause/step/jump/interrupt/seek/reverse/loop；debugger 展示 instance/section/next/slot weights/events/root motion/sync/budget/terminal state，并能从 source ID 回跳。还需 malformed/stale/reload/shutdown/network/hitch/loop storm fault matrix 与 1/100/1k instance profile。

## 5. P2：规模、协作与超越目标

### ED201-P2-01 · Open · Motion Warping、Target Alignment 与 Root Motion Modifier 缺失

必须建立 target snapshot、window、authority、constraint、failure fallback 与 prediction/correction contract，并保持原始 root-motion provenance；不能在 Gameplay callback 中直接修改 pose/root delta。

### ED201-P2-02 · Open · 多角色 action、shared marker 与 network choreography 缺失

多角色 interaction 需要 role-qualified instance、共同时间/marker、admission barrier、partial failure rollback 与 replication convergence，不能用多个独立 Montage request 偶然对齐。

### ED201-P2-03 · Open · Template、child override、diff/merge 与批量迁移缺失

模板继承必须以 stable ID override、冲突检测、版本迁移和 deterministic flattening 为基础；禁止靠显示名覆盖或复制整份数组。

### ED201-P2-04 · Open · Streaming、residency、prefetch 与 hot-swap 缺失

长 action 需要按 Section/Segment 依赖预取，frame tick 无同步 I/O；artifact/clip/mask/profile residency、miss、fallback、eviction 与 generation hot-swap 必须有预算和 trace。

### ED201-P2-05 · Open · 自动 blend/branch 优化与同语义质量实验室缺失

任何自动调参或优化只能在 exact semantics oracle、quality metric、deterministic corpus 与可复现硬件协议上比较；未完成同语义 benchmark 前不得宣称超过 Unreal。

## 6. 五套参考源码裁决

### 6.1 Unreal：主参考覆盖资产、实例、推进、玩法与编辑器

`FAnimSegment/FAnimTrack` 保存 source range、rate、loop 并把 root motion 拆为连续 extraction steps；`FCompositeSection`、`FSlotAnimationTrack` 与 branching marker 构成 Montage source。`FAnimMontageInstance` 持 position/rate/weight、next/prev section、active branching state、marker tick、sync leader/follower 与 root-motion disable state。推进在 Section/Branching 边界 sub-step，事件可改变 position 或销毁 instance，因此 callback 后必须重新验证 lifetime。

`UAnimInstance` 暴露 play/stop/jump/set-next/set-rate 等实例操作，`AnimSync` 负责 leader/follower marker tick 与 root-motion accumulation。Gameplay Ability task 区分 completed/blend-out/interrupted/cancelled 并停止当前 instance；replicated montage record 携带 montage、position/section、rate、blend 与 play instance identity。Persona 对 Section link、Slot/Segment 排序与修改使用 transaction、asset `Modify()`、PostEditChange、preview restart 与 track refresh。Zircon 应学习职责边界与 failure model，不复制 UObject、宏、历史兼容字段或默认参数。

### 6.2 Godot：one-shot、section playback 与 callback ordering 的次参考

`AnimationPlayer` 有 play/section/marker、正反播放、queue、blend、stop、seek、finished signal 与 method-call policy；`AnimationNodeOneShot` 区分 fire/abort/fade 与内部/外部 seek；Mixer 提供 root-motion track/delta consumer。它证明 command/state/seek/terminal callback 必须明确，但不提供 Slot action stack、Branching Point 或 Gameplay replication 设计依据。

### 6.3 Fyrox：Signal、Layer、Transition 与 provenance 次参考

Fyrox 的 Signal、machine layer/mask、active state/transition、event 与 play node 提供 typed identity、pose composition 与状态来源边界。它可交叉校验 Notify provenance、layer arbitration 与 transition 生命周期，但没有 Unreal 等价 Montage product，不可替代主参考。

### 6.4 Bevy：typed player、transition set 与 event traversal 次参考

Bevy `AnimationPlayer`/active animation、`AnimationTransitions` 和 `AnimationEvent` 展示 typed player state、fade-out set、seek/speed/repeat 与事件 traversal。它适合校验 source/prepared/player 分层和事件遍历方向，不提供 Section graph、Slot、Branching 或 Ability bridge。

### 6.5 Unity Graphics：只约束 deformation/current-previous consumer

Graphics 仓不是 Mecanim/Animator 源码。选定 ShaderGraph、VAT 与 HDRP motion-vector 源码只证明骨骼/顶点形变必须稳定发布 position/normal/tangent 与 previous-frame deformation，jump/teleport/hot-swap 要有 reset policy。它不能用于推断 Unity Montage 能力，也不能反向定义 Zircon action runtime。

## 7. 目标架构与唯一 authority

```text
Montage Source Document
  + stable Section/Slot/Segment/Notify IDs
  + Clip/Mask/Rig/Profile dependency generations
        |
        v
Canonical Animation Compiler
  -> dense section graph / segment time map / event index
  -> slot composition / root-motion / sync plan
  -> source map + diagnostics + deterministic build key
        |
        v
Immutable PreparedAnimationAction
  -> validator / LKG / CAS / cook / generation install
        |
        v
Animation Action Runtime
  -> request admission / generational instance / action stack
  -> bounded sub-step / event / interruption / sync
  -> atomic MontageFrameReceipt
        |
        +--> Movement / Gameplay / Network / Replay
        +--> Pose / Deformation / Renderer current-previous
        +--> Editor PreviewWorld / Trace Debugger
```

必须只有一份 source schema、一份 semantic compiler、一份 prepared artifact validator、一份 action instance runtime 和一份 frame receipt ABI。Editor 只拥有 document/toolkit/job/preview projection，不拥有运行时 evaluator；Gameplay 只通过 typed provider/request/receipt 操作实例；renderer 只消费已发布 pose/deformation。

硬切要求：真实 provider 成立前隐藏或标记 Unavailable 当前入口；删除固定 queued/Ready/section/notifies/root-motion 成功措辞；禁止按 focus/control ID/display name 寻址 domain/runtime；禁止 runtime 读 mutable source、帧内同步 I/O、Editor/Runtime 两份 event/branch 数学和 failure 覆盖 LKG。

## 8. 重构里程碑

### ED201-M0 · Capability truth、owner、RED corpus 与 benchmark protocol

把 Preview/Apply/field commit 降为 Unavailable/Prototype；冻结 owner、invalid corpus、reference revision、semantic scenarios、quality metrics 与 hardware/workload protocol，加入“无 artifact/receipt 不得显示成功”的 RED contract。

### ED201-M1 · Stable source schema 与 dependency closure

实现 Montage、Section、Slot、Segment、Notify/State/Branch、blend typed schema、stable IDs、serialization/migration 与 Clip/Mask/Rig/Profile generation references。

### ED201-M2 · Canonical compiler 与 PreparedAnimationAction

在唯一 Animation compiler 中实现 validation、dense graph/time/event index、composition/root-motion/sync plan、source map、build key 与 validator；删除任何 Editor/private runtime duplicate compiler 方案。

### ED201-M3 · Job、LKG/CAS、cook 与 generation publication

把领域 compile 接入 EditorJob authority，补 dependency-aware keyed admission、cancel、progress、atomic publish、last-good、cook/export/install 与 stale generation fail-close。

### ED201-M4 · Runtime instance 与 action stack

实现 qualified request/receipt、generational handle、terminal lifecycle、slot/concurrency arbitration、blend/interruption 和 bounded active registry。

### ED201-M5 · Deterministic sub-step 与 atomic frame receipt

实现 Section/Segment/Branch/Event 边界推进、正反/loop/seek/rate flip、Notify State lifetime、root motion、sync 与 pose/gameplay 的原子 receipt；覆盖 callback 重入/销毁和 budget continuation。

### ED201-M6 · Gameplay、network、prediction 与 replay

接入 Runtime08G typed Ability/Cue provider，完成 replication/prediction/correction、record/replay/save/load 与 generation mismatch diagnostics。

### ED201-M7 · 真实 Editor toolkit、PreviewWorld 与 debugger

动态 document projection 替换固定数据，接入 transaction/history/save；PreviewWorld 运行同一 artifact/runtime；trace 以 stable source ID 映射 Section/Slot/Segment/Notify。

### ED201-M8 · Streaming、fault、scale 与跨引擎资格

完成 malformed/stale/reload/shutdown/network/hitch/loop storm fault matrix、Section 依赖预取、1/100/1k instance scale、cook/PIE 用户流与同语义跨引擎 benchmark。

## 9. 48 个资格门

状态说明：`Partial` 只承认可复用共享基础，不表示 Montage gate 已通过；`Pass` 必须有 Montage vertical slice 的动态证据。

### 9.1 Source 与 Compiler

| Gate | 当前 | 资格条件 |
|---|---|---|
| MONT-G-01 | Fail | Montage 可 Create/Open/Save/Reopen，asset/document/source revision 与 stable element ID 不变。 |
| MONT-G-02 | Fail | Section 重名、零长、越界、dangling edge 和无预算循环被 typed diagnostic 拒绝。 |
| MONT-G-03 | Fail | Slot/group/mask/rig/additive 兼容性由同源 catalog 验证，provider reload 后 generation 正确。 |
| MONT-G-04 | Fail | Segment clip/range/rate/loop/mirror 映射覆盖正反与边界，非法输入 fail-close。 |
| MONT-G-05 | Fail | Point/state/branch/window notify schema 可迁移、可定位且 payload 类型受验证。 |
| MONT-G-06 | Fail | Compiler 输出自包含 dense `PreparedAnimationAction`，runtime 不读取 mutable source。 |
| MONT-G-07 | Fail | Build key 覆盖所有 dependency generation 与 target profile，缓存命中/失效可复算。 |
| MONT-G-08 | Partial | 通用 Animation document 编译失败可保留 LKG；Montage compile/cancel/cook/stale generation 仍不存在。 |

### 9.2 Runtime Instance 与 Action Stack

| Gate | 当前 | 资格条件 |
|---|---|---|
| MONT-G-09 | Fail | Play request 按 world/subject/instance/artifact generation 寻址并返回 typed accepted/rejected receipt。 |
| MONT-G-10 | Fail | 每个 instance handle 有 generation 与 terminal disposition，旧 handle 不能命中新实例。 |
| MONT-G-11 | Fail | 同 Slot 并发按 priority/concurrency policy 确定 replace/queue/reject/coexist。 |
| MONT-G-12 | Fail | 同一 asset 多实例可独立 jump/stop/cancel，禁止按 asset 名误杀。 |
| MONT-G-13 | Fail | Blend in/out/profile/inertial/auto-out/interruption 的状态与回调顺序确定。 |
| MONT-G-14 | Fail | Jump/set-next/seek/rate/pause/stop 命令有 expected generation 与 ordered receipt。 |
| MONT-G-15 | Fail | Provider unload/world replace/shutdown 终结全部实例且不遗留 Slot/root-motion owner。 |
| MONT-G-16 | Fail | Paused/clean instance 不进入无意义 full scan，active registry 与 scratch allocation 有界。 |

### 9.3 Time、Section、Event 与 Root Motion

| Gate | 当前 | 资格条件 |
|---|---|---|
| MONT-G-17 | Fail | 大 delta 跨多个 Section/Segment/Notify/Branch 边界仍按确定顺序处理。 |
| MONT-G-18 | Fail | Forward/reverse/loop/seek/zero-delta 下普通 Notify 不漏、不重，政策可测试。 |
| MONT-G-19 | Fail | Notify State begin/tick/end 在 jump、interrupt、destroy 与 reverse 下成对且 lifetime 安全。 |
| MONT-G-20 | Fail | Branching Point 在推进边界即时执行，能改变 next Section 且不会使用失效实例。 |
| MONT-G-21 | Fail | Section loop 与 branch storm 受 sub-step/event/time 预算控制并提供 continuation/fault。 |
| MONT-G-22 | Fail | 每帧 root-motion delta 只有明确 owner，movement consume 与 action receipt 同 generation。 |
| MONT-G-23 | Partial | 通用 Trigger/pose/event admission 有失败重试局部证据；统一 Slot/pose/curve/event/root-motion/sync/terminal receipt 仍不存在。 |
| MONT-G-24 | Fail | Renderer current/previous pose 在 jump/teleport/loop 时有明确 reset，motion vector 不污染。 |

### 9.4 Sync、Gameplay 与 Network

| Gate | 当前 | 资格条件 |
|---|---|---|
| MONT-G-25 | Fail | Sync group/role/slot/marker 来自 canonical artifact，Montage 不建立私有 clock。 |
| MONT-G-26 | Fail | Leader/follower join/leave、leader loss、Section jump 与 branch 后重同步可复现。 |
| MONT-G-27 | Fail | Ability task 绑定 instance handle，complete/interrupted/cancelled/blend-out 不会重复终结。 |
| MONT-G-28 | Fail | Gameplay Cue/window 消费 typed event 并按 authority/prediction 去重。 |
| MONT-G-29 | Fail | Replication 携带 action generation、instance sequence、position/Section、rate、next edge 与 prediction key。 |
| MONT-G-30 | Fail | Client/server artifact generation 不一致时拒绝并产生可定位 diagnostic。 |
| MONT-G-31 | Fail | Prediction reject/correction/rollback 后 pose、root motion、event 与 ability state 最终收敛。 |
| MONT-G-32 | Fail | Record/replay/save/load 恢复同一 Section/action state 且不重发已消费 branch event。 |

### 9.5 Editor、Preview 与 Debugger

| Gate | 当前 | 资格条件 |
|---|---|---|
| MONT-G-33 | Fail | Workspace 显示真实 asset/document/revision/currentness，不再显示固定 `AM_DashAttack` 结果。 |
| MONT-G-34 | Fail | Section/Slot/Segment/Notify 编辑均走 transaction，undo/redo/save/reopen 保持 stable ID。 |
| MONT-G-35 | Fail | Selection、details、timeline、Section graph 与 diagnostics 绑定同一 document generation。 |
| MONT-G-36 | Fail | Add/Remove/Rename/Reorder/Link Section 会同步修复或拒绝引用，不留 dangling edge。 |
| MONT-G-37 | Fail | Add/Remove/Duplicate Slot 与 Segment trim/move 刷新 compiler currentness 和 preview。 |
| MONT-G-38 | Fail | PreviewWorld 运行同一 prepared artifact，play/jump/interrupt/seek/reverse/loop 与 Runtime 一致。 |
| MONT-G-39 | Fail | Debugger 显示 instance/Section/next/Slot weights/events/root motion/sync/budget/terminal state。 |
| MONT-G-40 | Fail | Source stable ID 可从 runtime trace 定位回 Section/Slot/Segment/Notify，stale trace 明确标记。 |

### 9.6 Fault、Scale 与交付

| Gate | 当前 | 资格条件 |
|---|---|---|
| MONT-G-41 | Fail | Malformed binary、未知 schema、missing clip/mask/profile 与 rig mismatch 均 fail-close。 |
| MONT-G-42 | Fail | Compile/provider reload/callback destroy/world replace/shutdown 不产生 UAF、幽灵实例或半 publish。 |
| MONT-G-43 | Fail | 1/100/1k active action instances 有 CPU、allocation、event、root-motion 与 Slot scale curve。 |
| MONT-G-44 | Fail | 长 action 按 Section 预取，frame tick 无同步 I/O，residency/miss/fallback 有预算与证据。 |
| MONT-G-45 | Fail | Hitch、极高 rate、loop storm 与 branch storm 有有界工作量且不静默丢 Gameplay 关键事件。 |
| MONT-G-46 | Fail | Cook/export/install/reopen/PIE 只使用 validated artifact generation 并保留 dependency provenance。 |
| MONT-G-47 | Fail | Create/Edit/Undo/Save/Compile/Preview/Play/Interrupt/Reload 真实用户流通过；固定字符串测试不计资格。 |
| MONT-G-48 | Fail | 与 Unreal 同语义场景的质量/CPU/内存/延迟 benchmark 可复现；未测不得宣称超越。 |

## 10. 实施顺序、停止条件与复核边界

1. 先关闭 capability truth：没有真实 provider 时隐藏或 disabled Montage 入口，固定 `queued/Ready` 不得继续作为成功反馈。
2. 先收敛 Editor197 的唯一 compiler/runtime authority 与 Editor198 的 Clip/Event/Root Motion/Sync/playback contract，再引入 Montage typed variant。
3. Source -> compiler -> artifact -> runtime instance -> frame receipt 顺序不可颠倒；不允许先给 ZUI action 接临时 `Vec`、字符串 event 或 focus-based runtime command。
4. Gameplay bridge只能消费 Runtime08G typed provider；Ability workspace 不得直接操纵 action 内部状态。
5. 每个里程碑先补 RED contract 与 invalid/fault corpus，再实现最小 vertical slice；跨模块、network、callback lifetime 与 scale 风险决定测试扩张范围。
6. 开始实现前重算本报告 fingerprint 与 HEAD，复核 Animation 插件 open failure 和共享工作树相关路径。

本报告退出条件不是 ZUI 可打开、Apply 显示成功或 Trigger 测试通过，而是 MONT-G-01 至 MONT-G-48 全部获得 Montage vertical slice 的动态证据，且父报告 compiler/runtime/event/root-motion/sync/gameplay 前置同时满足。在此之前，Montage 只能标为 Unavailable/Experimental。
