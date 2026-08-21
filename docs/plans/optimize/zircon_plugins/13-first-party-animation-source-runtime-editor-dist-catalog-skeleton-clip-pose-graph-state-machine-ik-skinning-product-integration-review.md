---
title: First-Party Animation Source、Runtime、Editor、Dist、Catalog、Skeleton、Clip、Pose、Graph、State Machine、IK、Skinning 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins13
review_date: 2026-08-19
baseline_head: 25e09a23178000f2e783ce2143cf70a8b118d404
baseline_epoch: 333
related_code:
  - zircon_plugins/animation/plugin.toml
  - zircon_plugins/animation/runtime/Cargo.toml
  - zircon_plugins/animation/runtime/src
  - zircon_plugins/animation/runtime/tests
  - zircon_plugins/animation/editor/Cargo.toml
  - zircon_plugins/animation/editor/src
  - zircon_plugins/animation/dist/Cargo.toml
  - zircon_plugins/animation/dist/src
  - zircon_plugins/animation_graph/plugin.toml
  - zircon_plugins/animation_graph/editor/src
  - zircon_plugins/timeline_sequence/plugin.toml
  - zircon_plugins/timeline_sequence/editor/src
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_runtime/src/animation
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
tests:
  - zircon_plugins/animation/runtime/src/tests.rs
  - zircon_plugins/animation/runtime/src/evaluation
  - zircon_plugins/animation/runtime/src/state_machine
  - zircon_plugins/animation/runtime/tests
  - zircon_plugins/animation/editor/src/tests.rs
  - zircon_plugins/animation/dist/src/lib.rs
  - zircon_plugins/animation_graph/editor/src/tests.rs
  - zircon_plugins/timeline_sequence/editor/src/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_plugins/04-animation.md
  - docs/plans/zircon_plugins/04/failure-2026-07-22-runtime-animation-fallback-evaluator-divergence.md
  - docs/plans/zircon_plugins/04/failure-2026-07-29-animation-frame-diagnostics-hardcut-omission.md
  - docs/plans/zircon_plugins/04/failure-2026-07-29-animation-sequence-caller-root-drift.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimationAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/Skeleton.h
  - dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Public/AnimationStateMachineLibrary.h
  - dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Private/AnimationStateMachineLibrary.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimationCompressionDerivedData.cpp
  - dev/UnrealEngine/Engine/Source/Editor/AnimationBlueprintEditor
  - dev/UnrealEngine/Engine/Source/Editor/Persona
  - dev/bevy/crates/bevy_animation/src
  - dev/Fyrox/fyrox-animation/src
  - dev/godot/scene/animation/animation_player.h
  - dev/godot/scene/animation/animation_tree.h
  - dev/godot/editor/animation
  - dev/godot/servers/rendering/storage/mesh_storage.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 13 · First-Party Animation Source、Runtime、Editor、Dist、Catalog、Skeleton、Clip、Pose、Graph、State Machine、IK、Skinning 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/animation` 不是空壳。包内已经有编译后的clip evaluator、严格递增key校验、`partition_point`区间选择、skeleton target table、revision cache、PosePool、graph/state-machine program、1D/2D blend space、layer/mask、two-bone/look-at IK、physics simulated pose bridge、ECS `QueryState` projection、事件准入与replacement epoch。这些机制与129项局部测试说明代码已经超过“只返回固定值”的原型，可以作为重构底座。

但它仍不是普通Zircon产品中的单一、工程级动画系统。`zircon_app`的普通Client和Editor Host只启用`zircon_runtime/animation`，没有启用`first-party-runtime-plugins`中的Animation provider；Dynamic Runtime Session因此装入core fallback `AnimationModule`。显式链接插件后，session会跳过fallback并装入插件module。两者使用同一个`animation.runtime` module、driver与manager service名称，却由两套近似复制的源码维护。它们不是同时碰撞，而是按装配条件互斥并可独立漂移；开放的fallback evaluator failure已经证明这种漂移不是理论风险。

插件内部也没有收敛成一条求值链。公开manager仍使用按`windows(2)`线性查找的legacy channel sampler与字符串骨骼绑定，较新的evaluation pipeline使用compiled evaluator，state machine/graph又各自维护缓存与采样入口。热路径会同步load skeleton/clip/graph/state-machine/sequence资产，克隆参数map、player组件和字符串骨骼名，把本可复用的dense `PoseBuffer`重新展开成`Vec<AnimationPoseBone>`。direct clip worker最多四片，但owner线程仍同步等待`sync_channel`，忽略schedule失败，worker不返回时生产代码会panic。当前测试所称“zero allocation”只覆盖PoseBuffer局部操作，不覆盖完整帧。

产品输入与输出也未闭合。首方glTF importer明确把每条animation写成“channel import is not implemented”的Data placeholder，skin和inverse-bind matrices也是generic Data；Animation自己的GPU skinning palette没有renderer consumer，所谓GPU/CPU parity测试只计算CPU matrix。pose apply依赖字符串骨骼名并在world-transform系统之后改写通用Scene node，更新错误被丢弃；physics bridge再复制一份字符串骨骼DTO。这个路径无法证明import、cook、retarget、evaluate、IK、root motion、skin deformation、render、physics与network使用同一skeleton/pose generation。

Editor与NativeDynamic同样是声明层。Animation Editor注册的四个`plugins://animation/editor/*.zui`全部不存在；Animation Graph的三个ZUI与Timeline Sequence的一个ZUI也不存在。Graph/Timeline注册open/validate/compile descriptor，却没有产品operation factory/compiler handler；first-party editor catalog只链接Navigation与Neural，不链接Animation。dist则明确声明animation evaluation仍由source runtime托管，并以`is_stateless`、空command/event、无save/restore/unload/bridge导出metadata shell。

Animation runtime本体由Runtime08C管理，authoring/compiler由Editor14管理，import/cook由Plugins07管理，通用catalog/native由Plugins01/06管理，Animation Graph/Timeline假authoring面已由Plugins08登记。本篇不重复累计这些父报告的最高优先级问题，登记 **0项新增P0、48项P1、12项P2**。本篇唯一拥有Animation单包从manifest、source runtime、fallback、editor、dist、catalog、ordinary App、imported asset到render/physics consumer的纵向交付合同。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes | 冻结事实 |
|---|---:|---|
| `zircon_plugins/animation`全包 | 170 / 18,172 / 634,036 | 166个Rust、4个TOML；包内无tracked working-tree差异 |
| runtime | 161 / 17,794 / 620,461 | manager、compiled evaluation、state machine、IK、GPU skinning、mask、systems与integration tests |
| editor | 6 / 208 / 7,507 | 只含registration/capability/test源码；注册引用的四份ZUI均不存在 |
| dist | 2 / 115 / 4,123 | Native ABI v3 descriptor与registration manifest projection |
| test-bearing inventory | 38 / 8,399 / 305,384 | 共129项`#[test]`；无Criterion、bench、property、fuzz或soak证据 |
| package fingerprint | `9fb8c3491df494af7b883afef0c2836a4910a595ef2ee60a91566a24b7877f34` | tracked path排序，以小写path、空格和文件SHA-256组成LF串，无末尾LF后再算SHA-256 |
| 相邻authoring包 | Animation Graph 10 / 1,015 / 38,720；Timeline 10 / 864 / 32,492 | 各11项registration/structure测试；合计另有4份缺失ZUI |
| core fallback | `zircon_runtime/src/animation` 17 / 2,202 / 76,471；framework contracts 38 / 3,210 / 101,160 | fallback manager/module与插件同名近复制，产品按linked plan二选一 |

源revision为`25e09a23178000f2e783ce2143cf70a8b118d404`，coordinator baseline epoch为333。Animation主包冻结时干净；App、catalog、Runtime fallback和共享计划有其他会话或用户改动，所以本文按当前工作树读取并保留`source_recheck_required`。实施前必须在同一generation重算主包、core fallback、Graph/Timeline、glTF importer、App features、runtime/editor catalog、builtin row与native dist。

### 2.2 测试库存不等于产品资格

129项测试覆盖clip key校验与采样、pose blend、graph/state-machine求值、layer/mask、IK、事件背压、replacement epoch、ECS projection、manager contract和registration shape。这些局部回归有价值，尤其是compiled target table、缓存revision、PoseBuffer复用和事件公平游标。

但测试同时固化了临时边界。allocation contract只验证PoseBuffer blend和`clone_from`保留容量，不运行asset load、graph/state-machine、pose publication和Scene writeback。`gpu_cpu_skinning_parity_within_tolerance`只验证两个骨骼的CPU矩阵运算，没有GPU dispatch、readback或draw。Editor测试只比较ID、URI与registration，dist测试只检查descriptor/manifest。structure contract还直接读取源码断言结构。仓内没有普通App provider选择、真实glTF clip/skin导入、platform cook、native/source parity、Editor preview/save/undo、renderer deformation、root motion、retarget、1k character、long soak、fault injection或跨平台SIMD资格。

### 2.3 本轮纵向追踪

1. `plugin.toml`、Cargo feature、runtime/editor/dist registration、capability与target声明。
2. core fallback和插件manager/module的装配选择、legacy/compiled evaluator、ECS projection、worker、cache、pose apply与event路径。
3. graph、state machine、blend space、layer/mask、sequence/timeline、IK、physics bridge与GPU skinning。
4. glTF animation/skin输入、runtime/editor catalog、App target feature、builtin catalog与NativeDynamic载体。
5. Animation Editor、Animation Graph、Timeline Sequence的resource、operation、compiler、preview与document closure。
6. Runtime08C、Editor14、Plugins01/06/07/08、Runtime22/24/42与三份开放failure的唯一owner边界。
7. Unreal、Bevy、Fyrox、Godot的适用源码；Unity Graphics只用于animation-to-render instance/deformation边界。

本轮为E3静态源码审查。没有修改production/tests，没有运行Cargo、App、Editor、NativeDynamic、真实import/cook/render、GPU readback、soak或性能测试。测试数量是源码库存，不是本轮通过数。

## 3. 当前真实产品链与断点

~~~text
ordinary zircon_app client / editor host
  -> enables zircon_runtime/animation contracts and dynamic-api
  -> does not enable first-party-runtime-plugins base catalog
  -> Dynamic Runtime Session sees no linked Animation package
  -> installs zircon_runtime::animation::AnimationModule fallback

explicit source-linked Animation provider
  -> first-party runtime catalog can return plugin registration
  -> Dynamic Runtime Session skips core fallback
  -> installs near-copy plugin AnimationModule/DefaultAnimationManager
  -> newer evaluation pipeline exists beside legacy manager sampler

source assets
  -> first-party glTF importer emits AnimationN Data placeholder
  -> SkinN and inverse bind matrices remain generic Data assets
  -> runtime hot path synchronously loads raw animation contracts
  -> compiled evaluator emits string-bearing pose DTO

frame execution
  -> PostUpdate after zircon.scene.world_transform
  -> worker shards schedule then owner blocks on sync_channel
  -> pose applies by bone-name lookup to generic Scene nodes
  -> physics receives another full string-bearing pose copy
  -> Animation SkinningPalette has no renderer consumer

Animation / Graph / Timeline Editor
  -> registration descriptors and URIs exist
  -> eight referenced ZUI resources do not exist
  -> no ordinary editor catalog link for Animation
  -> graph/timeline operation descriptors have no product handler

NativeDynamic dist
  -> exports ABI v3 descriptor and registration manifest
  -> is_stateless, empty command/event, no state/lifecycle/bridge
  -> explicitly leaves animation evaluation in source runtime
~~~

目标不是继续给每条断链增加fallback。目标是一个`AnimationActivationPlan`选择唯一runtime provider和artifact set；`SkeletonArtifact`、`ClipArtifact`、`GraphProgram`、`RigProgram`和`SkinBindingArtifact`共享稳定identity/revision；per-world `AnimationRuntimeInstance`按明确phase输出generation-qualified pose、root motion、events和deformation handle。Editor预览、source-linked runtime与NativeDynamic必须消费同一compiler/evaluator，Renderer/Physics/Network只通过typed adapter读取同代结果。

## 4. 可保留基础

| 基础 | 当前价值 | 重构约束 |
|---|---|---|
| compiled clip evaluator | key有序校验、target table、revision snapshot、`partition_point`采样比legacy线性扫描更接近生产路径 | 成为唯一采样owner；raw/editor数据与runtime artifact分离 |
| PoseBuffer/PosePool | dense buffer和pool已为稳定帧分配打下基础 | 全帧保持slot/handle表示，不在publication重新克隆bone name和Vec |
| graph/state-machine program | 已有编译结构、nested machine、layer/mask和blend-space节点 | compiler产物必须versioned/cooked，runtime不得重复解释source graph |
| ECS QueryState projection | parameter/player投影不再每帧无差别扫描全部node | 增量dirty set、per-frame receipt和generation retirement必须继续收敛 |
| event admission | 有bounded admission、backpressure与公平游标 | 扩展items/bytes/time预算、事件schema、drop原因和跨载体parity |
| replacement epoch | 能拒绝旧evaluation覆盖新player generation | 扩展到asset/program/world/render/physics publication generation |
| target mask/layer/IK contracts | 提供骨骼局部混合和后处理雏形 | 统一进rig program与stable target slot，不保留字符串旁路 |
| manifest partial状态 | 两项capability均为partial，没有冒充stable/complete | 在产品闭环和资格门前继续default-off/partial |

## 5. 参考实现给出的工程边界

### 5.1 Unreal Engine

`UAnimSequence`明确区分source data model、compressed runtime data、curve compression、frame stripping、retarget source、root motion与dedicated-server strip policy。`AnimationCompressionDerivedData`通过DDC key、后台build、cancel/wait/poll、压缩数据有效性和cook统计管理prepared artifact，而不是在帧循环同步解释source curve。

`UAnimInstance`和proxy边界覆盖parallel update/evaluation、notify queue、montage/section/interruption、sync group、root motion与linked graph。Animation Blueprint/Persona又拥有独立compiler、preview scene、skeleton/mesh/sequence/graph authoring和diagnostics。Zircon不必复制Unreal类数量，但必须达到相同的职责分离：source/editor model、platform artifact、parallel runtime instance、gameplay output、deformation output和authoring product不能由同一组字符串DTO临时串联。

### 5.2 Bevy Animation

Bevy以UUID实现`AnimationTargetId`，让clip curve和armature target共享稳定身份，而不是每帧按bone name解析。`AnimationGraph`是可序列化asset，mask映射到target ID；`ThreadedAnimationGraph`缓存postorder traversal作为求值加速结构。curve evaluator、event、morph与transition均进入同一动画crate合同。

Bevy并不提供Unreal级完整Anim Blueprint，但给出了较低的工程下界：稳定target identity、可保存graph asset、显式预计算遍历和可扩展curve target是基础能力。Zircon已有compiled graph雏形，却在最终pose、Scene apply、physics和GPU skinning边界退回字符串，因此尚未达到这个下界。

### 5.3 Fyrox Animation

Fyrox `AnimationContainer`长期持有animation，`AnimationPose`同时承载root motion；ABSM由Machine、Layer、LayerMask、State、Transition和pose node组成，animation signal也作为runtime事件进入模型。它说明中型Rust引擎也能把clip container、pose、root motion、layer/state-machine和event定义成连贯的runtime ownership，而不是互不一致的manager、compiled pipeline和sequence旁路。

### 5.4 Godot Animation

Godot `AnimationPlayer`明确physics/idle/manual callback与manual `advance`，维护playback/cache并提供blend hook。`AnimationTree`通过thread-local `ProcessState`传递求值状态，缓存filter和animation version，并把常用parameter绑定到slot以避免每次hash lookup。其Editor提供Animation Player、track、Bezier、blend space、blend tree和state machine的真实交互；状态转换与key编辑进入UndoRedo transaction。

Godot rendering storage又为skeleton分配RID/data，按bone更新transform并把mesh instance绑定到skeleton。适用结论不是复制Godot singleton，而是动画求值phase、parameter slot、editor transaction和render skeleton resource必须有明确owner。Zircon当前缺失的ZUI URI和未接入renderer的palette不能视为同等级实现。

### 5.5 Unity Graphics参考边界

本地`dev/Graphics`是Unity SRP/Graphics package，不含Animator或完整Animation Editor源码，因此不能用于证明clip、state machine、retarget或IK语义。它适合对照GPU product boundary：GPU Resident Drawer长期持有`InstanceDataSystem`、GPU buffer、per-frame visibility/culling update、platform admission和debug/readback接口。Zircon Animation若声称GPU skinning，至少必须把deformation artifact和palette generation接入renderer-owned buffer/update/draw/readback链，而不是只有一个未被消费的CPU palette helper。

## 6. P0归属：本文不新增最高优先级finding

| 已证实现象 | Canonical owner | 本篇责任 |
|---|---|---|
| fallback evaluator与插件evaluator分叉 | `failure-2026-07-22-runtime-animation-fallback-evaluator-divergence.md`、Runtime08C | 保持failure open，要求唯一runtime owner和迁移门 |
| 十项frame diagnostic未迁移 | `failure-2026-07-29-animation-frame-diagnostics-hardcut-omission.md` | 保持failure open，纳入G19/G31，不用累计counter冒充frame receipt |
| sequence caller root漂移 | `failure-2026-07-29-animation-sequence-caller-root-drift.md` | 保持failure open，等待managed Cargo/current support gate |
| imported clip/skeleton/cook/residency | Plugins07、Runtime08C | 记录placeholder如何阻断Animation纵向闭环，不复制importer P0 |
| Animation Graph/Timeline缺resource、operation与compiler product | Plugins08、Editor14/50 | 记录主Animation provider依赖关系，不重复authoring P0 |
| source/native catalog、ABI与lifecycle parity | Plugins01/06、Runtime42 | 定义Animation载体gate，不重造通用loader/ABI P0 |

0项新增P0不表示Animation接近完成，只表示最高优先级问题已经有唯一owner。任何实施都必须先处理这些父依赖，不能用Plugins13新编号绕过它们。

## 7. P1：Package、Catalog、Asset 与 Carrier纵向闭环

### NANI-P1-001 · 普通Client不链接Animation plugin provider

`target-client`启用core `animation`和`dynamic-api`，却不启用`first-party-runtime-plugins`。产品因此运行fallback module，而不是本包的新evaluation pipeline。目标是由profile/manifest解析出唯一`AnimationActivationPlan`，并在load report中明确provider/package/artifact generation。

### NANI-P1-002 · Editor Host同样运行fallback而非Animation plugin

`target-editor-host`链接advanced render、Navigation runtime/editor和Neural editor，但不链接base runtime plugins。Editor preview即使未来可见，也不会天然使用被审查的Animation runtime。必须让Editor runtime selection与game/export selection来自同一plan。

### NANI-P1-003 · First-party editor catalog没有Animation分支

editor catalog只有Navigation与Neural dependency/feature，Animation Editor crate无法进入普通host。注册源码存在不能证明产品可达；必须增加capability-gated package selection、factory mount和卸载/reload receipt。

### NANI-P1-004 · Animation Graph与Timeline同样没有catalog closure

两个authoring package的descriptor不能被普通Editor选择，且其runtime依赖只停留在manifest capability字符串。主Animation closure必须显式包含兼容的graph/timeline authoring/compiler artifact版本，或明确保持Unavailable。

### NANI-P1-005 · Builtin catalog广告与effective provider不一致

builtin row持续投影Animation与timeline event track为partial，但ordinary App实际装入core fallback，插件capability与实现版本没有进入effective receipt。能力查询必须回答“哪个provider、哪个artifact、哪种载体、哪个generation”，不能只回答row存在。

### NANI-P1-006 · Core fallback与插件module形成双源码权威

两套`AnimationModule`和`DefaultAnimationManager`使用相同module/driver/service名称，session按linked package二选一。互斥装配避免同时注册，却无法避免行为漂移。必须硬切为单一实现或让fallback成为对同一crate/artifact的薄adapter。

### NANI-P1-007 · Server target声明没有strip/evaluation policy

manifest把Animation支持到server runtime，但没有说明clip track、curve、event、root motion、pose、render-only deformation的server strip规则。参考Unreal的dedicated-server policy，构建artifact必须按用途保留gameplay曲线/事件并剥离无用变形数据。

### NANI-P1-008 · Animation Editor四份资源全部缺失

`authoring.zui`、两份blend-space ZUI和avatar-mask bone tree均不存在，manifest的`asset_roots`/`content_roots`又为空。注册URI不构成UI。resource必须进入package manifest、hash、mount、locale/theme和missing-resource fail-close gate。

### NANI-P1-009 · Graph/Timeline另有四份缺失资源

Animation Graph的authoring/player/state-machine ZUI与Timeline authoring ZUI均不存在。该问题由Plugins08拥有authoring P0，本篇要求Animation activation不能把依赖缺失的toolkit投影为可用。

### NANI-P1-010 · NativeDynamic dist只是stateless metadata shell

dist声明`is_stateless: true`，command/event manifest为空，invoke/save/restore/unload/bridge/host-ready均为None。诊断字符串还明确evaluation留在source runtime。必须实现同语义native provider或在载体选择阶段fail-close为Unsupported，不能报告已加载Animation行为。

### NANI-P1-011 · Source与NativeDynamic没有行为parity合同

source链接时可注册system、manager和复杂evaluator，native只导出registration metadata。需要同一scenario corpus验证clip、graph、state、event、pose、failure和lifecycle，或删去native行为能力声明。

### NANI-P1-012 · 两项粗粒度capability不足以描述真实支持面

`runtime.plugin.animation`和timeline event track均为partial，却无法表达import format、compression、retarget、root motion、graph node、IK solver、skin path、server policy和editor/compiler版本。建立typed support matrix，但不得用更多布尔位替代qualification receipt。

### NANI-P1-013 · Activation没有单一可观察receipt

当前load report能选择linked package或fallback，却没有Animation专属receipt说明manager实现、program/artifact schema、worker mode、render/physics adapters和degraded reason。必须让diagnostics、Editor和export消费同一不可伪造的activation结果。

### NANI-P1-014 · glTF animation仍被首方importer产出为placeholder

`add_gltf_animation_placeholders_and_skin_subassets`明确写出channel import未实现。Plugins07已拥有importer遮蔽与artifact P0，本篇要求Animation产品gate必须使用真实clip oracle，拒绝Data placeholder进入Ready evaluator。

### NANI-P1-015 · Skin与inverse-bind matrices没有typed Animation artifact

glTF skin、joint关系和inverse-bind matrices作为generic Data subasset发布，无法与SkeletonArtifact、mesh skin binding和runtime deformation建立versioned关系。必须生成稳定joint mapping、bind pose、inverse bind、mesh primitive和LOD兼容key。

### NANI-P1-016 · Package、import、cook、runtime与editor没有一张artifact graph

manifest module、raw runtime asset loader、glTF Data、Graph/Timeline descriptor与GPU palette各自解释资产。目标图必须从source snapshot到Skeleton/Clip/Graph/Rig/SkinBinding platform artifact，再到runtime instance和editor document，共享BuildSet与dependency digest。

## 8. P1：Evaluator、Scheduling、Cache 与 Pose Pipeline

### NANI-P1-017 · 插件内legacy manager与compiled evaluator重复

公开manager的`sampling.rs`/`pose.rs`继续按channel和bone name求值，新的`evaluation/clip_evaluator`走target table和binary interval lookup。所有caller必须迁到唯一compiled evaluator，并删除旧实现，而不是长期做双向parity维护。

### NANI-P1-018 · 第三条graph/state sampling入口继续分裂语义

manager graph递归求值、compiled graph pipeline与state-machine sampling分别维护参数、clip选择和缓存。需要唯一`AnimationProgramEvaluator`，graph/state/layer/sequence只编译为节点程序，不直接拥有资产加载和pose publication。

### NANI-P1-019 · 热路径同步加载Skeleton与Clip

clip sample在每个request内调用同步asset load，再取snapshot；加载错误压成None。运行时实例必须只消费admitted resident artifact lease，missing/stale/evicted要产生typed disposition而不是帧内阻塞I/O。

### NANI-P1-020 · Graph、State Machine与Sequence也同步load

graph timing、state machine cache和sequence tick各自执行load/cached compile。建立prepare/cook/residency阶段与atomic program publication；frame loop只能做bounded lookup和evaluate。

### NANI-P1-021 · Asset错误被静默折叠为空pose

缺manager、disabled、无asset manager、load失败和apply失败多处early return或`let _ =`，runtime system仍总是`Ok(())`。必须返回per-player/per-frame disposition，并区分Disabled、NotResident、InvalidArtifact、BudgetExceeded与EvaluatorFault。

### NANI-P1-022 · Direct clip worker忽略schedule失败

worker调用scheduler后不检查返回状态，随后阻塞接收channel。schedule拒绝时可能等待不存在的结果。submit必须产生JobHandle/receipt，失败立即回滚本帧publication并保留last-good pose。

### NANI-P1-023 · Worker不返回会触发生产panic

`sync_channel.recv`失败使用panic描述worker terminated。Animation frame不能因单个player/shard让整个host终止；需要task supervision、cancel/deadline、fault isolation和terminal shard result。

### NANI-P1-024 · Owner线程阻塞等待，不是真正phase DAG

最多四个shard仍由owner同步等待全部返回，单shard也会schedule再wait。建立Prepare -> Parallel Evaluate -> Merge -> Publish DAG，明确work stealing/fairness、frame deadline、late result discard和generation fence。

### NANI-P1-025 · 每帧克隆parameter/player/string状态

parameter apply会克隆graph/state parameter map、active state字符串和player组件；public pose又克隆每个bone name。改为compiled parameter slots、interned stable IDs、SoA player state和scratch arena，steady frame不得随bone/key数量产生heap allocation。

### NANI-P1-026 · Dense PoseBuffer在公开边界退化为Vec/String

compiled evaluator内部已使用dense pose，输出时却重建`Vec<AnimationPoseBone>`并复制String。Renderer、Physics和debug应读取generation-qualified pose buffer/slot view；只有诊断导出才按需解析名称。

### NANI-P1-027 · Cache容器与淘汰不满足规模路径

多个cache使用BTreeMap、全表选择淘汰，graph frame cache是最多256项Vec、线性find和`remove(0)`，还把完整parameter map作为key的一部分。需要按workload证明的O(1)/amortized结构、bytes budget、lease-aware retirement和hit/miss/eviction telemetry。

### NANI-P1-028 · Raw channel仍进入runtime artifact

compiled track保留raw channel/value并在采样时clone，没有platform compression、quantization、segment/seek table、LOD或quality ladder。source/editor curve与runtime compressed blob必须分离，并由cook/DDC identity管理。

### NANI-P1-029 · Graph求值重复构造Vec和参数快照

compiled graph每次evaluation组装parameter values和clip集合，state machine也把参数重新收集成Vec。compiler应分配稳定slot/layout，instance只更新dirty slot并复用evaluation stack。

### NANI-P1-030 · Masked blend的全局归一化会衰减未共同覆盖骨骼

base input先全局归一化权重，再按clip bone mask应用；当某输入不覆盖某bone时，该bone仍被其他输入的全局权重稀释。blend必须按target contribution归一化，定义base/additive/missing-track/reference-pose语义并用oracle覆盖。

### NANI-P1-031 · Pose写回依赖不稳定字符串匹配

pose apply建立descendant name index，接受exact/short name并在重名时选择首项。必须使用import/cook生成的stable `AnimationTargetId`/skeleton slot与scene binding generation，ambiguous/missing mapping在admission期失败。

### NANI-P1-032 · Animation在world-transform之后改写local transform

system位于PostUpdate且after `zircon.scene.world_transform`，随后修改generic Scene node local transform。若没有同帧第二次传播，render/physics可能读取旧derived transform。必须定义Animation Evaluate、Root Motion、Local Pose Publish、World Propagation、Physics、Render Extract的唯一phase order。

## 9. P1：State、Events、IK、Skinning、Editor 与 Qualification

### NANI-P1-033 · Trigger参数没有consume/reset语义

trigger只是parameter value，transition后不会自动消费，可能重复触发。编译程序必须区分bool/value/trigger，按tick和transition transaction消费，并在rollback/replay中保持确定性。

### NANI-P1-034 · State Machine arbitration与同步模型过薄

当前主要按首个匹配transition执行，缺少成熟的priority/interrupt window、sync group/marker、transition profile、inertialization和可解释decision trace。先定义确定性仲裁与状态receipt，再扩节点数量。

### NANI-P1-035 · Layer求值每层每实体分配PoseBuffer

layer blend为每层构造两个新PoseBuffer并最终再次生成name-cloned pose。使用预分配layer stack、target bitset和in-place/blend scratch，建立bone/player/layer规模预算。

### NANI-P1-036 · BlendSpace2D cook是高阶暴力枚举

实现枚举所有点三元组并检查其余点circumcircle，复杂度接近O(n^4)，重叠triangle选择也缺成熟triangulation/admission。应在compiler使用稳健Delaunay或等价库，处理duplicate/collinear/outside hull并输出确定性artifact与authoring诊断。

### NANI-P1-037 · Event预算与观测仍不完整

已有有界admission和公平cursor，但neutral event继续克隆target/name/payload String，缺少per-frame总bytes/time、schema version、drop原因与producer generation。开放diagnostic failure必须先迁移frame counters，再建立完整EventBatchReceipt。

### NANI-P1-038 · IK只是逐command局部solver

仅two-bone/look-at，命令逐项同步load skeleton并反复重建model pose。建立compiled RigProgram、solver phase、batch target input、constraint limits、pole/singularity策略和per-character scratch；高级solver保持Unavailable直至qualified。

### NANI-P1-039 · Animation到Physics仍复制字符串全pose

skeletal targets以bone name DTO发布，simulated pose又按唯一bone name混合，无法证明skeleton/physics body与pose同代。Physics adapter应消费stable joint/body binding与只读pose view，输出generation-qualified simulated subset。

### NANI-P1-040 · GPU skinning是未接入产品的false-green

`SkinningPalette`、decision与double buffer只在Animation模块和测试被引用，没有renderer consumer。必须有upload/compute-or-vertex path、draw binding、current/previous deformation、completion retirement、device-loss和readback parity；否则capability保持Unavailable。

### NANI-P1-041 · Inverse bind、mesh skin、morph与deformation关系不完整

palette按bone name构建BTreeMap，并从当前skeleton bind推导inverse bind；imported skin matrices未进入typed artifact。mesh primitive/LOD、joint remap、inverse bind、morph target、cloth/previous pose和bounds必须由同一个SkinBindingArtifact描述。

### NANI-P1-042 · Root Motion没有进入产品authority

参考Unreal/Fyrox都把root motion作为显式pose/gameplay输出；Zircon当前graph/state/sequence主要输出骨骼pose，没有world movement、collision、network prediction、rollback和consume receipt。Runtime08C拥有本体，本篇要求载体与App caller不能把缺失root motion的状态报告为完整动画。

### NANI-P1-043 · Animation Editor注册存在但产品不可达

editor crate只发布view/drawer/template和inspector customization，catalog不链接它。需要真实document/toolkit factory、selection/focus、runtime session binding、resource mount、reload和capability withdrawal，而不只是registration report。

### NANI-P1-044 · Editor surface没有可渲染resource

四个主包ZUI缺失，Graph/Timeline另四个缺失。必须用真实skeleton/clip/graph asset打开surface，并验证layout、input、large skeleton、missing asset、locale/theme和render screenshot；测试URI字符串不是证据。

### NANI-P1-045 · Graph/Timeline operation descriptor没有执行者

open/validate/compile命令只存在descriptor与routing测试，没有product handler/compiler结果。应接入Editor operation/transaction service，返回source/build-bound artifact和diagnostics，cancel/failure不修改document或last-good artifact。

### NANI-P1-046 · Preview、save、undo与runtime evaluator没有同语义闭环

当前没有证据证明Editor graph/blend-space/mask/sequence修改可undo、save、reopen、compile并由同一runtime evaluator预览。Editor14拥有authoring P0；Plugins13要求provider/version/artifact closure作为Animation可见性的前置条件。

### NANI-P1-047 · 没有跨载体同场景scenario parity

普通App fallback、显式source plugin、Editor preview、generated export和NativeDynamic没有共同输入/期望pose/event/state/failure corpus。必须建立同一BuildSet、同一artifact和容差规则下的differential test，避免互斥实现继续漂移。

### NANI-P1-048 · 没有工程级规模、稳定性与性能资格

无真实imported/cooked character、1k+ actor、长clip、深graph、layer/IK/event storm、streaming/reload、worker fault、device loss、8h soak、CPU/GPU frame/RSS/VRAM统计。129个局部test不能支持“性能与表现优于Unreal”的结论；竞争性宣称必须使用同场景同画质同硬件协议。

## 10. P2：先进产品能力差距

| ID | 差距 | 目标边界 |
|---|---|---|
| NANI-P2-001 | Retarget、IK Rig与骨架兼容配置缺失 | source skeleton映射、retarget pose、chain solver、quality/error receipt与Editor preview |
| NANI-P2-002 | Root Motion高级策略缺失 | extraction、warping、motion matching、physics/network/rollback authority与consume protocol |
| NANI-P2-003 | Montage、Slot、Sync Marker与Inertialization缺失 | gameplay interruption、section、notify、sync group和高质量过渡 |
| NANI-P2-004 | Motion Matching/Pose Search缺失 | offline feature database、query budget、streaming、determinism和debug visualization |
| NANI-P2-005 | Control Rig、Full-Body IK与constraint graph缺失 | compiled rig VM、parallel solver、editor manipulation、runtime/debug parity |
| NANI-P2-006 | Facial、Morph、Corrective Pose与Cloth联动缺失 | curve target identity、deformer graph、LOD、previous deformation与render integration |
| NANI-P2-007 | 工业压缩codec与质量阶梯缺失 | ACL/Oodle或等价codec评估、error metric、platform cook、DDC与runtime decompression budget |
| NANI-P2-008 | Update Rate、Visibility、LOD与Crowd策略缺失 | significance、budget allocator、pose sharing、distance/visibility tick policy和quality receipt |
| NANI-P2-009 | Network determinism与rollback animation缺失 | state/input snapshot、root motion/event reconciliation、prediction和correction policy |
| NANI-P2-010 | Live debug trace与专业profile缺失 | node/transition/weight/notify/root-motion trace、CPU/GPU cost、asset/cache/worker timeline |
| NANI-P2-011 | SIMD/GPU/CPU自适应执行缺失 | platform feature admission、deterministic fallback、batch threshold和同语义parity |
| NANI-P2-012 | 长期schema迁移与兼容窗口缺失 | skeleton/clip/graph/rig artifact version、reader/writer matrix、migration与last-good rollback |

这些能力只能在P1单一authority、artifact、phase、pose identity和产品闭环完成后分层加入。用名称相同的DTO、空node、布尔capability或CPU proxy预占接口，不计为P2进度。

## 11. 目标架构与Owner收敛

~~~text
AnimationSourceSet
  SkeletonSource / ClipSource / Curves / Events / Graph / Rig / Masks / Skin
        |
        v
AnimationCompiler (Editor14 + Runtime08C contracts)
  -> SkeletonArtifact
  -> ClipArtifact(compressed/segmented/platform/server policy)
  -> GraphProgram / StateProgram / RigProgram
  -> SkinBindingArtifact(mesh/LOD/joint/inverse-bind/morph)
        |
        v
AnimationActivationPlan (Plugins06 / Runtime42 / App)
  provider + carrier + BuildSet + artifact schema + capability qualification
        |
        v
AnimationRuntimeInstance per World
  Prepare resident leases
  -> Parallel Evaluate DAG
  -> Rig/IK/Physics adaptation
  -> Atomic Pose/RootMotion/Event publication
        |
        +-> Gameplay/Network root-motion and event adapters
        +-> Physics stable joint binding
        +-> Renderer deformation handle/current+previous palette
        +-> Editor preview/debug reader
        |
        v
FrameReceipt + QualificationReceipt
~~~

| Owner | 唯一职责 | 禁止继续承担 |
|---|---|---|
| Runtime08C | animation contracts、compiler runtime schema、evaluator、per-world instance、phase与pose/event publication | Editor document、package selection、native ABI、renderer内部buffer |
| Editor14 | skeleton/clip/graph/state/timeline/curve/rig authoring、transaction、compiler UX、preview与diagnostics | 私有runtime evaluator、静态fixture成功面 |
| Plugins13 | Animation package/carrier/catalog/App纵向closure与source/native/editor parity gate | 复制runtime/editor父finding或实现第二manager |
| Plugins07 | source snapshot、glTF/other importer、subasset/dependency/cook graph | frame evaluation和editor graph语义 |
| Plugins01/06 + Runtime42 | package/native admission、profile/catalog selection、activation/load receipt | domain evaluator与asset compiler |
| Renderer Runtime owners | deformation buffer、skin/morph execution、draw binding、GPU lifetime、bounds/velocity | 解释Animation source graph或bone String |
| Physics Runtime owners | stable ragdoll/joint binding、simulated pose output | 复制全pose或按bone name猜映射 |

## 12. 分层重构里程碑

### M0 · 冻结事实与失败基线

- 重算本报告fingerprint与App/catalog/fallback/Graph/Timeline/importer scope；
- 三份Animation failure保持open，给每项补owner、generation和required gate；
- 建立ordinary client、editor host、source plugin、native dist的effective provider矩阵；
- 禁止新增fallback evaluator、字符串pose边界和无执行者operation。

### M1 · 唯一Provider与Activation Plan

- 将core fallback与plugin runtime硬切到单一implementation crate；
- 建立`AnimationActivationPlan/Receipt`，包含carrier、artifact schema、provider generation和degraded reason；
- App、runtime/editor catalog、profiles和builtin row只投影effective plan；
- NativeDynamic不能执行同语义前明确Unsupported，不以metadata shell充当Ready。

### M2 · Source到Artifact闭环

- 修复glTF animation placeholder和generic skin Data，生成typed Skeleton/Clip/SkinBinding；
- 分离source/editor curve与compressed runtime artifact；
- graph/state/rig/mask/sequence编译为versioned program；
- 建立dependency digest、platform/server strip、DDC、atomic publish和last-good。

### M3 · Evaluator与Phase硬切

- 删除legacy manager sampler和分裂graph/state采样入口；
- 建立parameter/target slot、resident lease、parallel DAG、deadline/cancel/fault result；
- 统一Animation Evaluate -> Rig/IK -> Root Motion -> Local Pose -> World Transform -> Physics/Render phase；
- 全帧steady path保持dense pose和bounded scratch，不重建String/Vec。

### M4 · Consumer闭环

- Renderer消费SkinBinding与generation-qualified deformation handle，闭合current/previous pose、bounds与GPU lifetime；
- Physics消费stable joint mapping并发布simulated subset；
- Gameplay/Network消费root motion与event receipt；
- missing/stale/budget/fault保留last-good并给出typed disposition。

### M5 · Editor产品闭环

- Animation、Graph、Timeline进入editor catalog并提供真实resource/toolkit factory；
- create/open/edit/undo/redo/save/reopen/compile/preview/debug/error/recovery使用同一artifact/evaluator；
- blend space、mask、state transition、timeline event与skeleton mapping有真实交互、validation和visual evidence；
- reload/disable会撤回capability并安全retirepreview generation。

### M6 · Carrier、Failure与Compatibility

- source/native/editor/generated export运行同一scenario corpus；
- worker fault、asset eviction/reload、bad artifact、world unload、device loss和shutdown有明确terminal receipt；
- artifact reader/writer/migration/rollback与server strip进入release gate；
- 删除core/plugin双实现和所有临时兼容alias。

### M7 · 规模与竞争性资格

- 建立真实imported/cooked角色、深graph、多layer/IK/event与1k actor workload；
- 记录CPU/GPU frame、worker critical path、allocation、RSS/VRAM、cache、streaming和pose latency；
- 执行长时间soak、deterministic replay、native/source differential和Editor/runtime parity；
- 只有correctness/failure/soak/quality/performance同时通过，才允许与Unreal作同口径比较。

## 13. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | ordinary Client、Editor Host、source export与NativeDynamic均给出同schema `AnimationActivationReceipt` |
| G02 | 任一产品generation最多一个Animation provider；core fallback与plugin不再维护两套manager/module实现 |
| G03 | capability row可回溯package、carrier、BuildSet、artifact schema、provider generation和qualification |
| G04 | server artifact具有明确track/curve/event/root-motion/deformation strip policy与测试 |
| G05 | glTF真实clip、skeleton、inverse bind、skin mapping进入typed artifact；Data placeholder被产品拒绝 |
| G06 | source/editor数据不进入frame evaluator；runtime只消费admitted resident artifact lease |
| G07 | Clip/Graph/State/Rig/Sequence只有一个compiled evaluator owner，legacy sampler已删除 |
| G08 | parameter和target使用stable slot/ID，steady frame不按bone name解析或克隆String |
| G09 | schedule拒绝、worker panic/hang、deadline/cancel均返回terminal result，host不panic/死等 |
| G10 | Prepare/Evaluate/Merge/Publish是可观察DAG，late generation不能覆盖新player/world |
| G11 | missing/stale/evicted/invalid asset产生typed disposition并保留或明确撤回last-good pose |
| G12 | graph/state/layer/mask/blend-space有确定性oracle；masked blend按target贡献语义正确 |
| G13 | BlendSpace2D处理duplicate/collinear/outside hull并满足compiler复杂度预算 |
| G14 | trigger consume、transition priority/interruption/sync与replay语义有测试和decision trace |
| G15 | Root Motion进入Gameplay/Physics/Network明确authority、consume与rollback协议 |
| G16 | IK/Rig使用compiled program和bounded scratch；singularity/limit/missing target有typed结果 |
| G17 | Animation -> Physics使用stable binding generation，不复制全量字符串pose |
| G18 | Animation -> Renderer闭合SkinBinding、palette/deformer、current/previous、bounds、GPU lifetime与draw |
| G19 | 每帧receipt包含players/bones/clips/events/jobs/bytes/time/cache/drop/fault，十项diagnostic failure完成迁移 |
| G20 | 事件使用versioned schema与items/bytes/time预算，drop/backpressure/fairness可复核 |
| G21 | ordinary Editor能挂载Animation/Graph/Timeline；缺provider/resource/artifact时fail-close |
| G22 | 八份当前缺失ZUI由真实package resource替代，并通过mount/hash/render/input/locale/theme gate |
| G23 | operation descriptor都有factory/handler；validate/compile/cancel/failure返回source-bound receipt |
| G24 | Editor支持create/open/edit/undo/redo/save/reopen/compile/preview/debug/error/recovery完整闭环 |
| G25 | Editor preview、source runtime、native runtime与generated export运行同一artifact/evaluator scenario corpus |
| G26 | NativeDynamic若宣称Animation行为，具备command/event/state/lifecycle/bridge和unload quiescence；否则明确Unsupported |
| G27 | asset reload、program replacement、world unload、plugin disable/reload不会发布陈旧pose/event/deformation |
| G28 | allocation profile覆盖完整帧；steady workload满足零或明确有界分配，非仅PoseBuffer局部测试 |
| G29 | 1k actor、深graph、多layer/IK/event storm与streaming workload满足声明CPU/GPU/RSS/VRAM预算 |
| G30 | GPU parity执行真实dispatch/readback/draw；CPU-only matrix test不得命名为GPU parity证据 |
| G31 | 三份开放Animation failure按各自required gate关闭，不能由文档、source grep或局部test替代 |
| G32 | `git diff --check`、frontmatter/path、finding唯一性、fingerprint、索引/coverage与plan-output audit通过 |

## 14. 明确禁止的临时修复

1. 不在core fallback和plugin之间继续复制文件或同步粘贴patch；迁移后删除旧owner。
2. 不以新增布尔capability、Ready字符串或registration snapshot代替provider实例和行为资格。
3. 不把glTF animation placeholder、generic Data skin或测试手造clip当作import/cook完成。
4. 不把CPU matrix helper命名为GPU skinning parity，也不在没有renderer consumer时报告GPU path。
5. 不在frame loop增加更多同步asset load、String bone lookup、BTreeMap clone或临时Vec修补功能。
6. 不用更大的cache、channel或event上限掩盖无bytes/time/lease预算。
7. 不捕获worker panic后静默返回空pose；failure必须有terminal receipt与last-good策略。
8. 不在world-transform之后偷偷再写Scene local transform而没有明确phase与重新传播。
9. 不创建空ZUI、Space占位、disabled menu或只OpenView命令来关闭Editor finding。
10. 不为Graph/Timeline descriptor增加成功字符串；必须有document transaction、compiler artifact和runtime preview。
11. 不以source-only实现声称NativeDynamic parity；不能实现时明确不支持该载体。
12. 不在缺少同场景同画质同硬件数据时宣称性能或表现达到、超过Unreal。

## 15. 状态与产出边界

| 项目 | 状态 | 证据 |
|---|---|---|
| Animation主包逐文件审查 | review_complete | 170文件、18,172行、634,036 bytes、129项test attribute、fingerprint `9fb8c3491df494af7b883afef0c2836a4910a595ef2ee60a91566a24b7877f34` |
| fallback/App/catalog/Graph/Timeline/importer/consumer追踪 | review_complete | core/plugin双实现、ordinary App fallback、8份缺失ZUI、glTF placeholder、未接入GPU palette与stateless dist均已定位 |
| 参考引擎E3对照 | review_complete | Unreal artifact/parallel/editor、Bevy stable target/graph、Fyrox pose/ABSM、Godot phase/editor/render skeleton、Unity Graphics GPU product boundary |
| 新增finding | review_complete | 0 P0 / 48 P1 / 12 P2；32项qualification gate |
| Production与tests修改 | pending | 本篇未修改任何production/tests/Cargo/manifest |
| Cargo/App/Editor/GPU/Native/soak/performance执行 | not_run | review-only；测试库存不作为本轮通过证据 |

本篇完成的是Animation纵向差距清单与重构依赖，不是实现完成。三份开放failure继续保持open；Runtime08C、Editor14、Plugins01/06/07/08与Runtime42仍是父owner。下一步实施必须从M0/M1的唯一provider、activation truth和source artifact开始，不能从添加高级node、按钮或动画效果开始。
