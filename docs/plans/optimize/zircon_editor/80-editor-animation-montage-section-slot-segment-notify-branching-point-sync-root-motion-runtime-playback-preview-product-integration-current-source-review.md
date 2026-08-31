---
title: Editor Animation Montage、Section、Slot、Segment、Notify、Branching Point、Sync、Root Motion、Runtime Playback、Preview 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor80
review_date: 2026-08-23
baseline_head: c4761b14c6748c4fb0ac7edef67183d8d5afb5eb
baseline_epoch: 357
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
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip/compile.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs
tests:
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimCompositeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimMontage.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimSync.h
  - dev/UnrealEngine/Engine/Source/Editor/Persona/Private
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities
  - dev/godot/scene/animation
  - dev/Fyrox/fyrox-animation/src
  - dev/bevy/crates/bevy_animation/src
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Samples~/RendererShaderUserValue_Common
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Animation Montage、Section、Slot、Segment、Notify、Branching Point、Sync、Root Motion、Runtime Playback、Preview 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon没有Animation Montage产品，也没有可被Gameplay、Animation Graph或角色运行时使用的one-shot action stack。仓内唯一专用产品表面是230行`workbench_extension_montage_editor_workspace.zui`：`AM_DashAttack / AM_Combo_01 / AM_Evade`、`Intro Section 0.00-0.38s`、`Combo Branch`、`HitPause 0.46s`、`UpperBody`、`Extracted 2.8m forward`、`4 sections / 3 notifies`均为固定文本。20个标准化action最终只选择workspace/tab/row/command、对`.edit`字段切换popup并写入固定feedback；`.commit`、Preview和Apply都不产生asset mutation、transaction、compile job、runtime request或operation receipt。

`zircon_runtime`、`zircon_plugins`和`zircon_app`对Montage、Branching Point、slot group/track、section jump或animation action没有生产命中。现有`AnimationClipAsset`只有bone track和`event: String + time_seconds + payload`，没有stable event ID、duration notify、branching point、section、slot或action source；`AnimationSequenceAsset`是通用property timeline。Animation插件虽有clip/graph/state machine/blend space/event/pose/IK底座，却没有Montage source、compiler、prepared artifact、instance、action arbitration、section command或Gameplay bridge。

本轮不新增P0。Editor14 P0-2继续唯一拥有“可见高级工作区用静态成功措辞声明不存在能力”；Runtime08C P2-3继续唯一拥有Montage/slot/sync group/marker/inertialization平台大类；Editor77继续拥有通用Clip/Event/Root Motion/Sync和`AnimationActionArtifact`前置。本报告只登记尚未逐项建账的 **18项P1、5项P2和48个资格门**，把Montage专属纵向合同展开为`AnimationMontageSourceDocument -> MontageCompilePlan -> PreparedAnimationAction -> MontagePlayRequest -> MontageInstanceHandle -> MontageFrameReceipt`。

本轮只做current-source review和文档建账，不修改生产源码。未运行Cargo、真实Editor、GUI/GPU、cook、runtime playback、Gameplay Ability、network prediction/replication、preview、trace、fault/soak/profile或同语义跨引擎benchmark；因此不能宣称当前Montage功能正确、可用、性能达标，更不能宣称性能或表现超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 本轮唯一owner

本报告只拥有“Montage source如何定义section graph、slot/segment layout、point/state notify与blend policy，经唯一Animation compiler生成immutable action artifact，再由Runtime action stack以qualified instance和确定事件/root motion/sync receipt运行，并由Editor编辑、预览和调试同一generation”的纵向边界。

- Editor14继续拥有高级动画workspace的capability truth、通用toolkit/preview/compile真实性；本轮不重复假UI P0。
- Editor09继续拥有background job admission、cancel、progress、shutdown和durable artifact总合同。
- Editor21与Runtime08G继续拥有Ability task、Cue、prediction、replication与Gameplay authority；Montage只能作为typed Animation provider接入。
- Editor32继续拥有Skeleton、Skin、Retarget与import/reimport identity。
- Editor63继续拥有transaction/history/savepoint/document scope和async operation总合同。
- Editor69继续拥有PreviewWorld、time domain、pause/step和可见性调度。
- Editor75继续拥有Timeline/Dope Sheet/Curve/transport通用交互。
- Editor76继续拥有Animation Graph唯一compiler/runtime authority。
- Editor77继续拥有stable Clip/Event/Root Motion/Sync source、prepared artifact和playback transaction；本轮只扩展Montage特有的section/slot/branching/action合同。
- Runtime08C P1-13继续拥有通用one-shot trigger，P1-15拥有通用event平台，P2-2拥有root motion，P2-3拥有Montage/sync大类；本轮不改名重报。

### 2.2 Currentness

- 审查HEAD：`c4761b14c6748c4fb0ac7edef67183d8d5afb5eb`。
- 协作baseline epoch：`357`；session：`optimize-editor80-animation-montage-review-r1-20260823`。
- 16个focused Zircon文件在冻结时没有working-tree diff；共享工作区存在大量其他Session变更，本轮不回退、不整理、不纳入结论。
- 全仓专用语义检索确认Editor仅有9个Montage产品文本文件；`zircon_runtime`、`zircon_plugins`、`zircon_app`对Montage/branching point/slot group/section jump/animation action返回零生产命中。
- action白名单、navigation hash index、ZUI可加载与固定feedback测试只证明控件路由，不证明domain、asset、compiler或runtime存在。

### 2.3 冻结语料与可复算fingerprint

统计口径：路径转为小写正斜杠并排序；每个文件取SHA-256，再拼接`path + NUL + lowercase file hash + LF`计算集合fingerprint。declarations使用Rust/C++/C#的`fn/class/struct/enum/trait`行首声明正则，仅用于规模定位。

| 范围 | 文件 / 行 / 非空行 / bytes / declarations | fingerprint |
|---|---:|---|
| Zircon selected set | **16 / 5,742 / 5,468 / 269,040 / 65** | `5c772b0085c6195c04d5b485611c5f43dd50450641bb55cc4e420c6c3d9e2aa3` |
| Unreal selected set | **12 / 8,466 / 7,100 / 294,479 / 65** | `275cc9cbffed9b1b0cec8678620fcd6d443318c11b9a66393d3adc16eb865b2d` |
| Godot selected set | **5 / 3,857 / 3,143 / 150,273 / 27** | `f7f4f934f5dd796e51e371c92dd58ebe00a1dc4d750683c34fb6ddcc9dafd216` |
| Fyrox selected set | **4 / 1,227 / 1,090 / 46,503 / 75** | `bb1af5553140456246ca4087f8c960ff24333459c1dd2b72aaa706bc2f563734` |
| Bevy selected set | **3 / 2,087 / 1,877 / 78,138 / 124** | `381d9e91000eb8161ebef1c65bbf221f6b061d5992638e294015c2de89d203ee` |
| Unity Graphics selected set | **3 / 343 / 289 / 13,474 / 0** | `2f3e07f58e7995e4b97aa7264e2ab0df73e69f2ef7f2471cf70fb6e4b69293ba` |
| Five-engine deduplicated set | **27 / 15,980 / 13,499 / 582,867 / 291** | `bf7bee81966a012ee38cb6a31d8d1ab4a24434171eeac910eb45fff0088cc290` |

### 2.4 集合成员与参考限制

Zircon集合覆盖Ability入口、Montage ZUI、extension host/binding/navigation/feedback/action inventory，以及现有Clip/Sequence、compiled clip、event和request底座。没有可加入集合的Montage runtime/compiler source；这不是抽样遗漏，而是专用词和候选类型全仓检索结果。

Unreal是主参考，集合覆盖`FAnimSegment/FAnimTrack`、`FCompositeSection/FSlotAnimationTrack/UAnimMontage/FAnimMontageInstance`、sub-step/event/root motion/sync、Persona Montage editor和Gameplay Ability play/replication。Godot只用于one-shot request、fade/abort/seek、section playback；Fyrox只用于stable signal、layer mask、active state/transition与event provenance；Bevy只用于typed player和forward/reverse/loop event traversal。Graphics仓不是Mecanim源码，本轮只用其previous-position motion vector与VAT bake说明deformation consumer需要稳定的current/previous frame artifact，不能从中推断Unity Montage能力。

## 3. 当前真实产品链

### 3.1 Workspace是固定演示数据

Montage ZUI第49-95行固定Sections/Notifies/Curves和三行所谓asset；第146-184行固定Intro、Combo Branch、HitPause、Root Motion和output；第199-229行固定Montage dropdown、`Slot: UpperBody`与三个blend option。这里没有AssetId、DocumentId、SourceRevision、SectionId、SlotId、SegmentId、NotifyId、RigArtifactId、CompileGeneration、InstanceHandle或ReceiptId。

Ability workspace只增加一个`Montage Editor`按钮，extension workspace host只把ZUI挂入容器。它们证明surface可达，不证明Montage资产或运行时存在。

### 3.2 20个action只改变控件状态

template binding把20个event规范化为3个tab、8个row、3个command和6个field edit/commit action。`ExtensionActionRoute`只有workspace/tab/row/command control ID和`field_action: bool`；`apply_workbench_extension_action`只执行exclusive selection、dropdown popup与feedback。

`.commit`没有读取字段值或expected revision，Apply没有保存/编译，Preview没有创建preview session。feedback把Open、Preview、Apply、Intro和Root Motion映射为固定`AM_DashAttack`、`UpperBody`、`4 sections / 3 notifies`与`2.8m forward`。唯一Montage相关测试只断言open/apply/blend action字符串存在；没有domain mutation、undo、compile、playback、event、root motion或失败路径。

### 3.3 Runtime底座不能表达Montage

`AnimationEventTrackAsset`只保存optional target string、event string、time和payload，不能表达稳定身份、duration state、branching semantics、notify class/schema、trigger offset或消费政策。Clip binary升级只把v1 event list补为空；Sequence则是entity/property channel集合。

Animation插件的compiled clip、pipeline event/request、state machine、blend space、mask和pose是应复用的底座，但没有action source/artifact/instance。若直接把Montage塞进event字符串、临时Graph节点或Editor私有Vec，会制造第四份动画schema并绕过Editor76/77的唯一compiler与playback transaction。

## 4. 参考源码提炼

### 4.1 Unreal：Montage是资产、实例、事件和编辑器的完整纵向

`FAnimSegment`保存animation reference、montage start、source start/end、play rate和loop count；`FAnimTrack`验证/排序segment，支持正反向notify与root motion range。`FCompositeSection`保存section name、link time、next section和metadata；`FSlotAnimationTrack`把slot name连接到segment track。`UAnimMontage`再定义blend mode/profile、blend in/out、auto blend out、sync group/slot、sections和slot tracks。

`FAnimMontageInstance`不是一个bool：它持instance ID、position/play rate/weight、next/prev section arrays、active state branching points、marker record、sync leader/follower和root-motion disable count。`Advance`在section end与branching marker处分步，限制迭代，按连续range提取root motion、收集marker、排队普通notify、即时执行branching point，再观察事件是否改变position/section。事件甚至可能销毁instance，因此每次回调后都验证lifetime。

Persona对Add/Remove/Rename/Duplicate Slot、Add/Remove/Move Section、Set Next Section使用transaction和asset `Modify()`，刷新notify offset、section time、track projection、dirty state与真实preview。Gameplay Ability task绑定blended-in/blending-out/end/cancel回调，核对当前ability和Montage instance；复制记录包含animation、position或section、play rate、blend time、next section、play instance ID、prediction key、slot等字段。Zircon不必复制UObject，但必须覆盖这些责任。

### 4.2 Godot、Fyrox与Bevy：通用语义校验

Godot `AnimationNodeOneShot`显式区分Fire、Abort、Fade Out，持active/internal-active、fade剩余时间、auto restart和mix mode，并区别内部reset seek与外部seek。`AnimationPlayer`有assigned/current、queue、blend、speed、section/marker正反播放和method call mode。这证明action command必须有状态机与seek政策，不能只设selected。

Fyrox `AnimationSignal`用UUID、name、time和enabled表达稳定事件；MachineLayer持mask、active state/transition、bounded event queue，并标记事件来自state还是transition、按权重策略收集。Bevy `ActiveAnimation`区分repeat/complete/pause/speed/seek，并对forward、reverse、forward-loop、reverse-loop四种事件区间使用有序partition与测试。它们不提供完整Montage产品，但共同否定“字符串event + 一次frame比较”足够的假设。

### 4.3 Unity Graphics：仅约束下游帧一致性

HDRP motion vector使用current/previous clip position并对skin/deformation路径给出专门政策；VAT baker把多个clip按frame烘焙为texture array并保存frame count。Montage若改变section、seek、loop、root motion或slot composition，必须向renderer提供同generation的current/previous pose与teleport/reset标志。本报告不把Graphics样例误当作Unity Animation authoring参考。

## 5. 父报告校正与不重复计数

| 既有owner | 仍Open事实 | 本轮处理 |
|---|---|---|
| Editor14 P0-2 / P1-26 / P1-52 | 静态成功workspace；缺通用notify/montage轨；高级动画产品没有asset/job/compiler/runtime preview | 保持原账，本轮只展开Montage专属合同 |
| Runtime08C P1-13 / P1-15 / P2-2 / P2-3 | one-shot trigger、event、root motion、Montage/slot/sync大类未闭合 | 保持Runtime owner，不重复平台缺失 |
| Editor77 ED77-P1-08/09/10、M7 | 通用事件遍历/身份/发布、root motion/sync/action artifact未闭合 | 本轮定义Montage consumer，不重报通用event算法 |
| Editor21 / Runtime08G | Ability task、Cue、prediction、replication与Gameplay trace缺失 | Montage只定义provider bridge和输入输出 |
| Editor63 / 69 / 75 / 76 | transaction、preview world、timeline、animation compiler/runtime各有唯一owner | 作为实现依赖，不建立平行authority |

没有新证据可以关闭父P0/P1/P2。固定`Ready`、`Warning`、`queued`、action binding或能打开ZUI都不是动态产品证据。

## 6. 新增P1工程差距

### ED80-P1-01：没有canonical `AnimationMontageSourceDocument`、稳定身份、版本与依赖闭包

当前Montage身份只是dropdown显示字符串。目标source必须有`MontageAssetId / DocumentId / SourceRevision / RigArtifactId / SectionId / SlotId / SegmentId / NotifyId`，并声明clip/mask/blend profile/curve依赖、schema version、migration和reimport政策；显示名不得参与寻址。

### ED80-P1-02：Section不是稳定图，无法表达合法跳转、循环和终止

固定`Intro/Combo Branch`没有source对象。目标`MontageSectionGraph`保存stable section ID、sorted start boundary、default/explicit next edge、metadata与terminal policy；禁止重名、零长、越界、dangling edge和无预算无限环。rename/reorder不得破坏runtime command或diff identity。

### ED80-P1-03：没有Slot Group、Slot、Layer、Mask与Skeleton兼容合同

`Slot: UpperBody`只是一段文本。目标`MontageSlotLayout`引用rig-scoped slot/group catalog、compiled bone mask、blend/additive policy、priority和root-motion ownership；同group冲突、未知slot、mask越界、rig generation不一致必须compile fail-close。

### ED80-P1-04：没有Segment的clip引用、source range、rate、loop和time mapping

当前timeline没有任何clip。目标segment保存stable ID、prepared clip generation、montage start、source start/end、signed play rate、loop count、mirror/additive/retarget option和provenance，并预编译montage time到clip time的正反/loop映射；零速率、空range、gap/overlap政策和recursive action引用必须显式验证。

### ED80-P1-05：多Slot组合、additive、root motion与deformation ownership未定义

多slot并行时谁产生base pose、additive、curve、notify、root motion和previous pose完全没有合同。目标compiler生成每slot evaluation order与composition plan；每帧只能有明确root-motion owner，renderer current/previous pose和movement consumption共享同一action frame generation。

### ED80-P1-06：Point Notify、Notify State、Branching Point与Cancel Window没有typed schema

`HitPause gameplay cue`和`Cancel Window`没有数据结构。目标区分deferred point notify、duration state begin/tick/end、必须在推进边界即时执行的branching point，以及由Gameplay消费的typed window；事件带stable ID、payload schema、role/target、weight/filter、authority/prediction和seek/reverse/loop政策。

### ED80-P1-07：Blend In/Out、Blend Profile、Inertialization与Interrupt政策缺失

一个`0.15` dropdown不能表达blend curve/profile、standard/inertial mode、auto blend out、trigger time、stop blend override、interrupted/completed语义或per-bone profile。目标source与play/stop request共同解析成确定blend plan，illegal mode/provider missing必须返回typed reject。

### ED80-P1-08：没有唯一Montage semantic compiler与immutable `PreparedAnimationAction`

Apply只写固定字符串。目标由Editor76/77唯一compiler消费source snapshot，输出dense section/slot/segment tables、time map、cooked event/branch index、root-motion/sync metadata、blend plan、scratch/budget和dependency generations；runtime不得读取mutable Editor source或按字符串查找。

### ED80-P1-09：没有build key、增量cook、LKG、publication与currentness

目标build key至少包含source hash、rig/clip/mask/profile generations、compiler/schema/toolchain/target profile；compile作为Editor09 typed job运行，支持cancel/progress/diagnostics。成功后原子publish新generation，失败保留LKG但明确stale，cook/export拒绝missing或错误generation。

### ED80-P1-10：没有qualified `MontagePlayRequest`、instance handle和terminal lifecycle

Preview/Gameplay都没有可调用runtime入口。目标request包含world/subject/animation instance/action artifact generation、start section/time、rate、slot scope、priority/concurrency、blend override、authority/prediction和caller token；返回generation-safe handle及Accepted/Rejected receipt，实例必须最终Completed/Interrupted/Cancelled/Failed/Retired之一。

### ED80-P1-11：Jump/SetNext/Seek/Rate/Pause/Stop命令没有原子与代际语义

目标`MontageControlCommand`按instance handle、expected generation和command sequence寻址，支持jump section、set-next edge、seek with/without events、rate、pause/resume、stop/interrupt。过期handle、已终止实例、非法section和回调重入返回typed outcome；命令对section/event/root motion推进要么同frame原子生效，要么明确排入下一frame。

### ED80-P1-12：没有Action Stack、Slot所有权、并发组与仲裁

工程角色会同时有locomotion、attack、reload、hit reaction和interaction。目标action stack按slot group/concurrency key/priority/activation group仲裁，定义replace、queue、reject、blend-out、coexist与cancel；相同action的多实例必须由handle区分，不能靠asset名停止错误实例。

### ED80-P1-13：没有section/branch边界的确定性sub-step与预算

单次大delta可能跨segment loop、notify、branching point和section edge。目标runner在已排序边界处分步，支持正反、loop、hitch与零delta state tick，并有max substeps/time span/event count预算；超限必须产生continuation或typed fault，不能跳过Gameplay关键branch或无界循环。

### ED80-P1-14：Pose、Event、Root Motion、Sync与Gameplay输出不是同一frame receipt

目标`MontageFrameReceipt`绑定world/instance/artifact/frame generation，包含previous/current position、section transition、pose contribution、ordinary events、branch events、active states、root-motion delta、sync markers和terminal change。消费顺序确定且可回放；callback能销毁/跳转instance时必须以lease与generation重新验证。

### ED80-P1-15：Montage Sync、leader/follower与marker join没有实例级集成

通用sync平台由Runtime08C/Editor77拥有，但Montage必须声明group/role/sync slot、join/leave、leader loss、section jump和branch回调后的重同步政策。目标action artifact只引用canonical sync metadata，instance receipt报告marker pass和correction；不得另建Editor私有sync clock。

### ED80-P1-16：Gameplay Ability、prediction、replication、replay与save/load桥缺失

目标Animation provider向Runtime08G暴露play/control/terminal/event receipt；Ability task绑定handle而不是asset名，cancel/interrupt/complete只终结自己的instance。网络记录至少带action generation、play instance sequence、position或section、rate、next edge、prediction key和correction policy；client/server asset generation不一致必须拒绝，而不是静默播放不同动作。

### ED80-P1-17：Editor没有真实asset toolkit、transactional operation与动态projection

目标`MontageEditorSession`绑定qualified document和expected revision，提供section graph、slot/segment tracks、notify/state/curve/details、selection和diagnostics的typed projection。Add/Remove/Rename/Reorder/Link Section、Add/Remove/Duplicate Slot、Insert/Trim/Move Segment和Notify编辑全部走Editor63 transaction；undo/redo/save/reopen保持stable identity并刷新compile currentness。

### ED80-P1-18：没有runtime-backed preview、trace、fault与性能资格

Preview必须在Editor69隔离world中加载同一prepared artifact和rig/mesh，以统一clock执行play/jump/interrupt/seek/reverse/loop，显示active instance、section/next edge、slot weights、events/states、root motion、sync、budget与diagnostics。测试必须覆盖source/compiler/runtime/editor/Gameplay组合链、malformed asset、provider reload、stale generation、callback销毁、hitch/loop storm和1/100/1k action instance性能；固定字符串断言不再算产品资格。

## 7. 新增P2工程差距

### ED80-P2-01：缺少Motion Warping、contact window与environment alignment消费

在基础root-motion receipt稳定后，Montage可声明warp target、contact phase、translation/rotation policy和failure fallback，由独立movement/physics provider求解；不得在Editor timeline中直接修改root curve冒充运行时warping。

### ED80-P2-02：缺少multi-role contextual action与同步section协议

抓取、处决、协作开门等动作需要多个actor共享role binding、entry condition、anchor、leader clock、section barrier、cancel/rollback和partial failure。每个actor仍持独立generation-safe instance，协调器只消费typed receipt。

### ED80-P2-03：缺少模板、批量编辑、semantic diff/merge与协作评审

支持从action template生成slot/section/notify布局、批量替换clip/profile、按stable ID显示section graph与segment/notify semantic diff，并对并发rename/reorder/link冲突显式解决；禁止以数组位置三方合并。

### ED80-P2-04：缺少大型Action的segment/page streaming、prefetch与residency预算

长过场或组合动作需要artifact page table、section-aware prefetch、lease、eviction、warmup和missing-page fallback。frame tick不得同步加载clip；性能报告按action/slot/segment记录resident bytes、decode time和miss。

### ED80-P2-05：缺少质量感知的自动blend/inertialization与动作优化

在确定性与可审计前提下，离线分析可建议blend profile、inertialization、section boundary、foot/contact continuity和compression quality；建议必须给出pose/root-motion误差与成本证据，由作者确认后写source，不能成为运行时黑盒自改。

## 8. 目标架构与owner边界

```text
Montage Editor Session
  -> AnimationMontageSourceDocument
     -> SectionGraph + SlotLayout + SegmentSource + NotifySchema + BlendPolicy
  -> AnimationSemanticCompiler
     -> MontageCompilePlan
     -> PreparedAnimationAction (immutable generation)
  -> AnimationRuntimeService / ActionStack
     -> MontagePlayRequest -> MontageInstanceHandle
     -> MontageControlCommand
     -> MontageFrameReceipt / MontageTraceFrame
  -> Gameplay Ability provider + Preview World + renderer/movement consumers
```

关键类型建议：

- `AnimationMontageSourceDocument`：durable source、stable IDs、revision、dependencies与migration。
- `MontageSectionGraph`：sorted boundaries、default/override edges、loop/terminal validation。
- `MontageSlotLayout`：rig-scoped group/slot/mask/composition/root-motion owner。
- `MontageSegmentSource`：clip generation、source range、rate/loop/mirror/additive与time map input。
- `MontageNotifySchema`：point/state/branch/window、payload、authority和traversal policy。
- `PreparedAnimationAction`：dense immutable runtime artifact，不含Editor control/display identity。
- `MontagePlayRequest / MontageControlCommand / MontageInstanceHandle`：qualified control plane。
- `MontageFrameReceipt / MontageTraceFrame`：同generation pose/event/root motion/sync/terminal evidence。
- `MontageEditorSession`：transactional source controller和dynamic projection，不实现runtime semantics。

禁止的捷径：

- 不得把Montage实现为`AnimationEventTrackAsset.event`字符串约定。
- 不得让ZUI control ID、display name或当前focus充当asset/section/instance identity。
- 不得让Editor维护第二套section traversal、notify触发或root-motion算法。
- 不得在frame中解析mutable source、按字符串查slot/section或同步加载clip。
- 不得用`queued/Ready/Warning`文本代替job、compile、runtime或diagnostic receipt。

## 9. 依赖顺序与重构里程碑

### ED80-M0：Capability hard gate与RED tests

先把当前Preview/Apply/field commit降级为Unavailable或接入真实provider；增加证明“固定反馈不是产品结果”的RED测试，冻结20个action的真实ownership。

### ED80-M1：Source schema与stable identity

实现Montage source、section/slot/segment/notify/blend typed schema、migration、serialization和invalid corpus；接入Rig/Clip/Mask/Profile稳定引用。

### ED80-M2：Semantic compiler与prepared artifact

在Editor76/77唯一compiler中生成dense tables、time map、branch/event index、composition/root-motion/sync plan、build key和diagnostics；接入Editor09 job、LKG与atomic publication。

### ED80-M3：Runtime instance与action stack

实现qualified play request、generation handle、terminal lifecycle、slot/concurrency arbitration、blend/interruption和bounded active instance registry。

### ED80-M4：Deterministic advance与frame receipt

实现section/branch sub-step、正反/loop/seek、ordinary/state/branch event、root motion、sync与pose contribution的原子receipt，覆盖callback重入/销毁和budget continuation。

### ED80-M5：Gameplay、network与replay bridge

接入Runtime08G/Editor21 Ability task、Cue、prediction/replication/correction、record/replay/save/load；generation mismatch和authority failure全部fail-close。

### ED80-M6：真实Editor toolkit与transaction

用dynamic document projection替换固定ZUI数据，接入section graph、slot/segment timeline、notify/state/curve/details、transaction、dirty/save/undo/redo、compile currentness和diagnostic定位。

### ED80-M7：Runtime-backed preview与debugger

在PreviewWorld运行同一artifact/instance，支持play/pause/step/jump/interrupt/seek/reverse/loop；展示slot weight、section edge、event/root motion/sync、budget和trace，并与source stable ID映射。

### ED80-M8：Fault、scale、cook与产品资格

完成malformed/stale/reload/shutdown/network/hitch/loop storm fault matrix，1/100/1k instances和长action residency/profile，cook/export/install/reopen/PIE完整链；全部门禁通过后才恢复生产入口和成功措辞。

## 10. 资格门（当前均Fail）

### 10.1 Source与Compiler

- [ ] MONT-G-01：Montage可Create/Open/Save/Reopen，asset/document/source revision和stable element ID不变。
- [ ] MONT-G-02：section重名、零长、越界、dangling edge和无预算循环被typed diagnostic拒绝。
- [ ] MONT-G-03：slot/group/mask/rig/additive兼容性由同源catalog验证，provider reload后generation正确。
- [ ] MONT-G-04：segment clip/range/rate/loop/mirror映射覆盖正反与边界，非法输入fail-close。
- [ ] MONT-G-05：point/state/branch/window notify schema可迁移、可定位且payload类型受验证。
- [ ] MONT-G-06：compiler输出自包含dense `PreparedAnimationAction`，runtime不读取mutable source。
- [ ] MONT-G-07：build key覆盖所有依赖generation与target profile，缓存命中/失效可复算。
- [ ] MONT-G-08：compile cancel/failure保留LKG但明确stale，cook拒绝错误或missing generation。

### 10.2 Runtime Instance与Action Stack

- [ ] MONT-G-09：play request按world/subject/instance/artifact generation寻址并返回typed accepted/rejected receipt。
- [ ] MONT-G-10：每个instance handle有generation和terminal disposition，旧handle不能命中新实例。
- [ ] MONT-G-11：同slot并发按priority/concurrency policy确定replace/queue/reject/coexist结果。
- [ ] MONT-G-12：同一asset多实例可独立jump/stop/cancel，禁止按asset名误杀。
- [ ] MONT-G-13：blend in/out/profile/inertial/auto-out/interruption的状态和回调顺序确定。
- [ ] MONT-G-14：jump/set-next/seek/rate/pause/stop命令有expected generation和ordered receipt。
- [ ] MONT-G-15：provider unload/world replace/shutdown终结全部instance且不遗留slot/root-motion owner。
- [ ] MONT-G-16：paused/clean instance不进入无意义full scan，active registry和scratch allocation有界。

### 10.3 Time、Section、Event与Root Motion

- [ ] MONT-G-17：大delta跨多个section/segment/notify/branch边界仍按确定顺序处理。
- [ ] MONT-G-18：forward/reverse/loop/seek/zero-delta下普通notify不漏、不重，政策可测试。
- [ ] MONT-G-19：notify state begin/tick/end在jump、interrupt、destroy和reverse下成对且lifetime安全。
- [ ] MONT-G-20：branching point在推进边界即时执行，能改变next section且不会使用失效instance。
- [ ] MONT-G-21：section loop与branch storm受sub-step/event/time预算控制并提供continuation/fault。
- [ ] MONT-G-22：每frame root-motion delta只有明确owner，movement consume与action receipt同generation。
- [ ] MONT-G-23：slot pose、curve、event、root motion、sync和terminal change进入同一原子frame receipt。
- [ ] MONT-G-24：renderer current/previous pose在jump/teleport/loop时有明确reset，motion vector不污染。

### 10.4 Sync、Gameplay与Network

- [ ] MONT-G-25：sync group/role/slot/marker来自canonical artifact，Montage不建立私有clock。
- [ ] MONT-G-26：leader/follower join/leave、leader loss、section jump和branch后的重同步可复现。
- [ ] MONT-G-27：Ability task绑定instance handle，complete/interrupted/cancelled/blend-out不会重复终结。
- [ ] MONT-G-28：Gameplay Cue/window消费typed event并按authority/prediction去重。
- [ ] MONT-G-29：replication携带action generation、instance sequence、position/section、rate、next edge与prediction key。
- [ ] MONT-G-30：client/server artifact generation不一致时拒绝并产生可定位diagnostic。
- [ ] MONT-G-31：prediction reject/correction/rollback后pose、root motion、event与ability state最终收敛。
- [ ] MONT-G-32：record/replay/save/load恢复同一section/action state且不重发已消费branch event。

### 10.5 Editor、Preview与Debugger

- [ ] MONT-G-33：workspace显示真实asset/document/revision/currentness，不再显示固定`AM_DashAttack`样例结果。
- [ ] MONT-G-34：section/slot/segment/notify所有编辑均走transaction，undo/redo/save/reopen保持stable ID。
- [ ] MONT-G-35：selection、details、timeline、section graph和diagnostics绑定同一document generation。
- [ ] MONT-G-36：Add/Remove/Rename/Reorder/Link Section会同步修复或拒绝引用，不留dangling edge。
- [ ] MONT-G-37：Add/Remove/Duplicate Slot与segment trim/move能刷新compiler currentness和preview。
- [ ] MONT-G-38：PreviewWorld运行同一prepared artifact，play/jump/interrupt/seek/reverse/loop与Runtime一致。
- [ ] MONT-G-39：debugger显示instance/section/next/slot weights/events/root motion/sync/budget和terminal state。
- [ ] MONT-G-40：source stable ID可从runtime trace定位回section/slot/segment/notify，stale trace明确标记。

### 10.6 Fault、Scale与交付

- [ ] MONT-G-41：malformed binary、未知schema、missing clip/mask/profile和rig mismatch均fail-close。
- [ ] MONT-G-42：compile/provider reload/callback destroy/world replace/shutdown故障不产生UAF、幽灵instance或半publish。
- [ ] MONT-G-43：1/100/1k active action instances有CPU、allocation、event、root-motion和slot scale curve。
- [ ] MONT-G-44：长action按section预取，frame tick无同步I/O，residency/miss/fallback有预算和证据。
- [ ] MONT-G-45：hitch、极高rate、loop storm和branch storm有有界工作量且不静默丢Gameplay关键事件。
- [ ] MONT-G-46：cook/export/install/reopen/PIE只使用validated artifact generation并保留dependency provenance。
- [ ] MONT-G-47：真实用户流Create/Edit/Undo/Save/Compile/Preview/Play/Interrupt/Reload通过，固定字符串测试不计资格。
- [ ] MONT-G-48：与Unreal同语义场景的质量/CPU/内存/延迟benchmark使用可复现实证，未测不得宣称超越。

## 11. 实施约束与退出条件

1. 先关闭Editor14 capability truth：没有真实provider时隐藏或disabled Montage入口，不能继续显示`queued/Ready`。
2. Source、compiler、artifact和runtime instance必须落入Editor76/77与Runtime08C的唯一Animation authority，不新增Editor私有evaluator。
3. Gameplay bridge只能消费Runtime08G typed provider；不得让Ability workspace直接操纵Montage内部Vec或当前focus。
4. 每个里程碑先补RED test，再实现最小vertical slice；测试规模随跨模块、网络和lifetime风险扩大。
5. 开始实现前重算本报告fingerprint与baseline，复核共享工作区相关animation文件的在途修改和owner lease。

本报告退出条件不是“ZUI能打开”或“Apply显示成功”，而是MONT-G-01至MONT-G-48全部有动态证据，并且父报告的compiler/runtime/event/root-motion/sync/Gameplay前置同时满足。在此之前，Montage只能标记为Unavailable/Experimental，不得进入Ready产品能力表。
