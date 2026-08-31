---
title: Editor Control Rig、Rig Graph、Hierarchy、Controls、Spaces、Constraints、IK、Solve、Bake、Preview、Compiler 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor78
review_date: 2026-08-23
baseline_head: f1614c5e601d0879cfa3ac1e5d4886f0d8734d97
baseline_epoch: 355
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/reference_menu_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/componentized_window.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_runtime/src/core/framework/animation/ik_command.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/ik
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
tests:
  - zircon_plugins/animation/runtime/tests/animation_ik_contract.rs
  - zircon_plugins/animation/runtime/tests/animation_pipeline_structure_contract.rs
  - zircon_plugins/animation/runtime/tests/runtime_physics_animation_tick_contract/ik_postprocess.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Animation/ControlRig/Source/ControlRig/Public/ControlRig.h
  - dev/UnrealEngine/Engine/Plugins/Animation/ControlRig/Source/ControlRig/Public/Rigs/RigHierarchy.h
  - dev/UnrealEngine/Engine/Plugins/Animation/ControlRig/Source/ControlRig/Public/Rigs/RigHierarchyElements.h
  - dev/UnrealEngine/Engine/Plugins/Animation/ControlRig/Source/ControlRig/Public/Units/Execution
  - dev/UnrealEngine/Engine/Plugins/Runtime/RigVM/Source/RigVM/Public/RigVMCore/RigVM.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/RigVM/Source/RigVMDeveloper/Public/RigVMCompiler/RigVMCompiler.h
  - dev/UnrealEngine/Engine/Plugins/Animation/ControlRig/Source/ControlRigEditor
  - dev/godot/scene/3d
  - dev/godot/editor/scene/3d/skeleton_3d_editor_plugin.cpp
  - dev/Fyrox/fyrox-animation/src/pose.rs
  - dev/Fyrox/fyrox-animation/src/machine/node/mod.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Nodes/MeshDeformation
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Control Rig、Rig Graph、Hierarchy、Controls、Spaces、Constraints、IK、Solve、Bake、Preview、Compiler 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon并非完全没有IK底座。Runtime已经有typed `AnimationIkCommand`、按World隔离且有4096条上限的命令队列、replacement epoch、Two Bone与Look At求解器、skeleton target table、可复用model-pose scratch、simulated-pose之后且scene writeback之前的post-process接入。这些实现证明Zircon已经能在已求值骨架姿态上执行有限的程序化修正，不应被静态Workbench外观掩盖，也不应在后续重构中被无理由推倒。

但当前Editor里的“Control Rig”不是一个Control Rig产品。它是一份230行固定ZUI：`CR_Hero`、`Spine_CTRL`、`Hand_IK_L`、`64 controls`、`18 constraints`、`1 warning`都写死在模板中；Preview与Validate只选择控件并把固定的“queued”文本写回状态栏。路由索引只知道workspace/tab/row/command/field，完全不知道rig asset、document、hierarchy element、control value、graph node、compiled generation、preview subject或operation receipt。字段`commit`没有资产变更路径，所谓`Weight: 1.00`甚至只是字符串。

Runtime的IK也不能直接升级命名成Control Rig。公开命令只有TwoBone/LookAt两种，并且只接受model-space位置、轴、clamp和weight；没有rig/command/node稳定身份、space provider、effector orientation、phase、priority、依赖、writer ownership或artifact generation。执行器逐命令同步load skeleton、逐命令编译target并串行原地修改pose；TwoBone先写root，再重建model pose后求mid，第二阶段失败时没有回滚root。多命令重叠的结果由插入顺序决定，错误诊断只保留entity、skeleton和粗粒度error。

本轮不新增P0。Editor14已经拥有“静态成功Workbench/无真实runtime preview与toolkit”的产品真实性P0；Editor76拥有通用Animation Graph/compiler/runtime双authority；Runtime08C P1-17已经拥有同步load、重复model-pose、排序、limits和hot path，P2-5已经拥有Control Rig/Full-Body大类平台。本报告只新增尚未被具体建账的 **14项P1、5项P2与48个资格门**：建立versioned `ControlRigSourceDocument`、typed `RigHierarchy/Control/Space/Constraint`、`RigUnitGraph`、唯一`ControlRigCompiler`、immutable `CompiledRigProgram`、分阶段且可回滚的`RigSolveTransaction`、qualified preview/bake/diagnostic链。

本轮只做current-source review与文档建账，不修改生产源码。未运行Cargo、真实Editor、GUI/GPU、save/reopen、cook、live preview、viewport manipulation、bake、fault/soak/profile或同语义跨引擎benchmark；因此不能宣称当前Control Rig正确、完整、性能达标，更不能宣称超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 本轮唯一owner

本报告只拥有“Control Rig source document如何表达Hierarchy/Control/Space/Constraint/Rig Unit Graph，经compiler生成prepared rig program，在Runtime分阶段求解，并由Editor对同一generation预览、操纵、反解与bake”的纵向边界。

- Editor14继续唯一拥有Animation默认toolkit可达性、静态成功UI、通用save/preview/compile真实性P0；本轮引用但不重复计数。
- Editor32继续唯一拥有Skeleton/Skin/import/reimport/retarget identity；Control Rig只能引用其稳定Skeleton artifact，不能建立第四份骨架schema。
- Editor63继续唯一拥有transaction/history/savepoint/async operation总合同。
- Editor75继续唯一拥有Timeline/Dope Sheet/Curve/Track/Key/transport交互。
- Editor76继续唯一拥有通用Animation Graph/State Machine compiler和Runtime authority硬切；Rig Graph必须共享compiler基础设施，但其Rig Unit、Hierarchy与solve event语义由本报告拥有。
- Editor77继续唯一拥有Sequence/Clip/channel/event/prepared animation artifact/playback合同；本轮只拥有Control Rig反解和bake桥。
- Runtime08C继续唯一拥有通用pose/skeleton/IK scheduling、热路径、joint limits、full-body/foot-lock/facial/cloth大类平台。本轮不把P1-17和P2-5换名字重报。

### 2.2 Currentness

- 审查HEAD：`f1614c5e601d0879cfa3ac1e5d4886f0d8734d97`。
- 协作baseline epoch：`355`；session：`optimize-editor78-control-rig-ik-review-r1-20260823`。
- working tree中`zircon_plugins/animation/runtime/src/ik/postprocess.rs`存在非本轮修改；focused diff只改变测试函数签名排版，没有改变本报告依赖的执行语义。
- 本报告按当前可达production caller、模板binding、focused test与本地参考源码裁决；测试名称、静态action白名单、固定反馈文案和ignored benchmark都不算动态资格。

### 2.3 冻结语料与可复算fingerprint

统计口径：相对路径规范化为小写并排序；每个文件取SHA-256，再拼接`path + NUL + lowercase file hash + LF`计算集合fingerprint。声明数沿用本审查系列的Rust/C++/C#函数/测试声明正则口径。

| 范围 | 文件 / 行 / 非空行 / bytes / declarations | fingerprint |
|---|---:|---|
| Zircon editor/product | **9 / 4,844 / 4,713 / 221,411 / 59** | `6ba54d223b6be317d0f19ed1180bbe8ee2fcd1f2f2150fc96c2d11b311d13967` |
| Zircon runtime/IK | **13 / 2,180 / 2,006 / 76,587 / 70** | `a9d58116a89defaa84634cf16211735acdfc6f7fcfdee4386ec0079315c53fd0` |
| Zircon focused tests | **3 / 349 / 328 / 12,206 / 10** | `6484a859cd9bc9fd738c2a76a191b004232ed7995028fe09769a6b66afddc74a` |
| Zircon deduplicated focused set | **25 / 7,373 / 7,047 / 310,204 / 139** | `e6f89f5a96599d0e8e0812829ec973392c3aa628190514217b6aecfb4663f01b` |
| Unreal selected set | **17 / 26,263 / 22,693 / 992,887 / 0** | `b4b75d54d5c5b68ffdd31fa74fab77acedf875d0bc03a1f6e14422df9ca9ab05` |
| Godot selected set | **26 / 9,890 / 8,456 / 392,691 / 0** | `fbce9f6ae05849256e601d117fce814c637f234cc1bd710469195ce266cfd990` |
| Fyrox selected set | **3 / 767 / 692 / 30,487 / 40** | `294bd20690fde6a3168df3c029f57ebbd63ecc7db87942db7b4ba4b415df79fd` |
| Bevy selected set | **2 / 1,016 / 931 / 41,097 / 30** | `f0d6092cf979af8f4ab08466340cf4c2d18cb912c6c092e56983db99a38303e1` |
| Unity Graphics selected set | **4 / 458 / 414 / 21,159 / 0** | `f087df51937d0fb6a582d70655f297e60c15d36fecffda8b84d08132b7726bde` |
| Five-engine deduplicated set | **52 / 38,394 / 33,186 / 1,478,321 / 70** | `7a209677ff73fef96d84f86ecc72a669193ca9c239213b0c934cddb2f7a03f5b` |

### 2.4 集合成员

Zircon editor/product集合为Control Rig ZUI、`gameplay_animation.rs` template bindings、navigation index与spec、`reference_menu_actions.rs`、`componentized_window.rs`、`extension_module_feedback.rs`及两份`workbench_preview_actions`。Runtime/IK集合为两份framework IK command/error、插件manager、`ik/{diagnostic,error,execution_error,look_at,mod,postprocess,two_bone}.rs`和pipeline的`tick/simulated_pose_blend/pose_apply.rs`。focused tests为frontmatter列出的3个integration/structure文件。

Unreal集合为：

```text
ControlRig/Private/Sequencer/MovieSceneControlRigParameterSection.cpp
ControlRig/Public/ControlRig.h
ControlRig/Public/Rigs/RigHierarchy.h
ControlRig/Public/Rigs/RigHierarchyElements.h
ControlRig/Public/Sequencer/MovieSceneControlRigParameterSection.h
ControlRig/Public/Units/Execution/RigUnit_BeginExecution.h
ControlRig/Public/Units/Execution/RigUnit_InteractionExecution.h
ControlRig/Public/Units/Execution/RigUnit_InverseExecution.h
ControlRig/Public/Units/Execution/RigUnit_PrepareForExecution.h
ControlRigDeveloper/Private/ControlRigBlueprintCompiler.cpp
ControlRigDeveloper/Public/ControlRigBlueprintCompiler.h
ControlRigEditor/Private/BakeToControlRigHelper.cpp
ControlRigEditor/Private/EditMode/ControlRigEditMode.cpp
ControlRigEditor/Public/BakeToControlRigHelper.h
ControlRigEditor/Public/EditMode/ControlRigEditMode.h
RigVM/Public/RigVMCore/RigVM.h
RigVMDeveloper/Public/RigVMCompiler/RigVMCompiler.h
```

Godot集合为`scene/3d`下`skeleton_modifier_3d`、`skeleton_3d`、`ik_modifier_3d`、`two_bone_ik_3d`、`look_at_modifier_3d`、`chain_ik_3d`、`iterate_ik_3d`、`fabr_ik_3d`、`ccd_ik_3d`、`jacobian_ik_3d`、`bone_constraint_3d`的`.h/.cpp`，以及`editor/scene/3d`下`skeleton_3d_editor_plugin`与`skeleton_ik_3d_editor_plugin`的`.h/.cpp`，共26文件。

Fyrox集合为`fyrox-animation/src/{pose.rs,machine/mod.rs,machine/node/mod.rs}`；Bevy集合为`bevy_animation/src/{animation_event.rs,graph.rs}`；Unity Graphics集合为ShaderGraph的`ComputeDeformNode.cs`、`LinearBlendSkinningNode.cs`以及VFX Graph的`SkinnedMeshRendererTransform.cs`、`VFXSlotSkinnedMeshRenderer.cs`。

## 3. 当前真实产品链与可保留底座

### 3.1 Editor surface不是空壳文件，但只是静态surface

Control Rig ZUI已声明Controls/Hierarchy/Solve标签、rig/control/solve rows、Preview/Validate、control/space dropdown和weight field；template binding把20个事件规范化为action ID。navigation index用单个`LazyLock<HashMap>`建立500余action的O(1) route lookup，这一通用路由基础可以保留。

但ZUI第73-95行固定`CR_Hero / Spine_CTRL / Hand_IK_L`，第146-182行固定FK/IK/space switch/64 controls/18 constraints/1 warning，第199-229行把control、World/Local/Parent和`Weight: 1.00`写成模板常量。`ExtensionActionRoute`第67-77行只有control ID与`field_action`布尔值；`apply_workbench_extension_action`第196-219行只做可见性、exclusive selection、popup toggle和feedback。

`extension_module_feedback.rs:15-39`只改`WorkbenchStatusReady`、`WorkbenchStatusMessages`和output row文本；其Control Rig分支在275-300行返回固定opened/queued/selected文本。`workbench_preview_actions`对Control Rig的测试只证明open/validate/weight action出现在静态白名单中，不证明存在document mutation、compile、runtime evaluation、viewport product或receipt。

### 3.2 Runtime IK基础是真实实现

`ik_command.rs:14-40`提供TwoBone与LookAt typed command；第57-83行验证finite input、非退化axis和`[0,1]` weight。manager使用per-world队列、4096上限、replacement epoch、stale reject和deferred entity选择性drain，并有队列顺序/替换epoch测试。

`two_bone.rs:38-66`按目标、pole和weight求两段链位置；`look_at.rs:33-49`产生clamped rotation。`postprocess.rs:71-98`复用一个`ModelPoseScratch`并把错误转换为诊断；pipeline `tick.rs:311-338`在simulated pose blend之后执行IK，随后才在365行写回scene node并更新presentation snapshot。正确重构应把这些数学与有界admission迁入compiled rig execution，而不是另造临时求解器。

### 3.3 当前边界为什么仍不是Control Rig

Control Rig至少要求三种可验证事实同时存在：可持久化并可迁移的Rig source；可编译、可诊断、可按明确phase执行的Rig program；Editor操纵与Runtime/Preview消费同一artifact generation。当前代码只有静态surface和两个ephemeral IK jobs，三者之间没有任何stable identity、revision、binding或receipt连接。

## 4. 父报告校正、开放阻断与不重复计数

| 既有owner | 本轮确认仍Open的事实 | 本轮处理 |
|---|---|---|
| Editor14 | Control Rig Workbench显示成功内容但无真实toolkit/preview/compile/save产品链 | 保持原P0/P1，不重复登记“假UI” |
| Editor32 | Skeleton/import/retarget identity与artifact未闭合 | Control Rig只能依赖，不另造Skeleton finding |
| Editor63 | transaction/history/savepoint/async operation总合同未闭合 | Bake/manipulation使用其合同，不重复通用undo finding |
| Editor75 | timeline/key/channel编辑与virtualization未闭合 | 只登记Rig反解/bake bridge |
| Editor76 | 多graph/compiler/runtime authority、typed pins与last-good总平台未闭合 | Rig Unit专属schema/phase/VM仍由Editor78登记 |
| Editor77 | Sequence/Clip prepared artifact、event/root motion/playback未闭合 | Bake输出必须进入其canonical source/transaction |
| Runtime08C P1-17 | 每command同步load、重复model pose、排序/limits/热路径缺口 | 不重复计数，只作为ED78架构依赖 |
| Runtime08C P2-5 | Control Rig/Full-Body/foot lock/facial/cloth平台缺失 | 不重复登记solver品类数量 |

本轮没有证据把父P0关闭。尤其“有action binding”“Preview queued”“64 controls”都不能改变能力真实性状态。

## 5. 新增P1工程差距

### ED78-P1-01：没有canonical `ControlRigSourceDocument`、版本、revision或runtime binding identity

产品唯一“rig”身份是ZUI中的`CR_Hero`字符串；action route没有asset/document/session/generation字段，Runtime command只携带world/entity和target IDs。不存在source version、schema migration、skeleton dependency、object/subject binding、source revision、compiled generation或last-good relation。

目标建立`ControlRigAssetId + ControlRigDocumentId + RigElementId + RigNodeId + SourceRevision + SkeletonArtifactId`，持久source与runtime instance分离；Editor open/save/reload、PIE/preview binding和cook必须通过qualified identity与revision CAS，不能靠当前focus或显示文本定位。

### ED78-P1-02：Hierarchy没有typed element、稳定身份、拓扑与initial/current transform合同

当前Editor把rig、control和solve row都表达成通用control ID；Runtime只认识skeleton target slot。没有Bone/Control/Null/Curve/Connector等element kind，没有稳定parent/multi-parent topology、local/global、initial/current pose、topology/pose version或metadata owner。

Unreal `RigHierarchyElements.h:232-236`明确区分Single/MultiParent、Bone、Null、Control，348-379行保存Current/Initial且Local/Global的transform与dirty state；Zircon需要同等清晰但Rust/data-oriented的`RigHierarchySource -> CompiledRigHierarchy`，并与Editor32的Skeleton artifact建立显式引用而非复制骨架。

### ED78-P1-03：Control value/settings只是显示文本，没有typed value、limits、shape和initial/current语义

ZUI第199-229行只有固定dropdown与`Weight: 1.00`字符串。没有Bool/Float/Integer/Vector/Rotator/Transform/Euler policy、min/max/limit enable、display name、shape/color/scale、animatable/transient、offset/shape transform、initial/current value或control visibility/selectability。

目标用`RigControlDefinition`与`RigControlValue`分离定义和值，所有Editor field必须由schema生成typed editor；invalid value fail-close，limits同时在authoring、preview、bake和runtime执行，shape只属于Editor visualization metadata而不污染solver ABI。

### ED78-P1-04：World/Local/Parent只是三个option，没有Space、multi-parent、constraint与maintain-offset图

ZUI第210-218行静态列出World/Local/Parent，第162-168行宣称“Space switch keyed Warning”，但仓内没有对应space element、parent weight channel、constraint source、maintain offset、compensation key、cycle detection或evaluation order。Runtime command又强制target/pole为skeleton model space（`ik_command.rs:8-12`）。

目标建立stable `RigSpaceId / RigParentConstraint / RigConstraintId`，明确source/target space、offset、weight、active interval、priority与cycle policy；切换space必须在同一transaction中计算补偿并交给Editor75/77生成canonical keys，不能只改dropdown文本。

### ED78-P1-05：没有Rig Unit registry、typed pin、node/edge、external variable或upgrade schema

Workbench标题写着“Solve Graph”，中心区域实际只有四个table row。通用navigation spec只列control ID；Runtime enum把整个可执行域封死为TwoBone/LookAt，没有unit descriptor、pin direction/type/default、wildcard resolution、execution/data edge、external variable、function/library、node version或upgrade hook。

目标在Editor76共享graph infrastructure上建立`RigUnitDescriptorRegistry + RigGraphSource + RigPinSchema + RigGraphUpgrade`。Math/transform/hierarchy query/set/solver/event unit应按feature注册；schema validation先于mutation，plugin卸载必须使旧节点Unavailable而不是静默改语义。

### ED78-P1-06：solve没有Construction/Forward/Backward/Interaction等typed phase与依赖合同

Runtime `tick.rs`只有固定“animation/simulated pose -> all IK commands -> scene apply”顺序。command没有phase、entry、priority、dependency或read/write set，因此无法表达construction、forwards solve、backwards solve、interaction、pre/post solve，也无法安全组合physics/contact或Editor direct manipulation。

Unreal四类execution unit显式提供Construction、Forwards Solve、Backwards Solve和Interaction事件。Zircon目标不是复制名称，而是建立typed `RigSolvePhase`、entry DAG、read/write declaration、phase barrier和deterministic schedule；无效跨phase依赖必须在compile期拒绝。

### ED78-P1-07：没有compiled Rig VM/program、memory layout、debug map、artifact currentness与last-good

当前每条command在`postprocess.rs:146-170`临时把target ID解析成slot，执行完即丢；不存在bytecode/instruction stream、constant/work/external/debug memory、dense operands、entry table、watched pins、source map、artifact digest、ABI/version或last-good install。

目标由唯一`ControlRigCompiler`输出immutable `CompiledRigProgram`：包含compiled hierarchy layout、entry/phase schedule、dense operand/memory pages、external binding table、source debug map、capability requirements和deterministic digest。Editor preview、PIE和cooked Runtime只能执行同一artifact格式；compile失败保留明确标记的last-good，不得把旧结果伪装成current。

### ED78-P1-08：Runtime IK command ABI丢失rig、command、source、space、orientation和generation

`ik_command.rs:20-40`只有world/entity/bones/model-space target/pole/axis/clamp/weight。它没有`RigInstanceId`、`CommandId`、node/pin source address、artifact generation、phase、space/provider identity、maintain-offset、effector rotation、preferred angle、twist/stretch policy或result slot。

这不是要求继续扩充一个巨型enum。基础TwoBone/LookAt应成为compiled unit，外部动态目标通过generation-qualified parameter/input buffer提交；一次性command仅保留为受限低级API，并必须携带source/phase/order/currentness与typed receipt。

### ED78-P1-09：串行原地求解非原子，TwoBone存在可观察的半写pose

`apply_ik_commands`按Vec插入顺序直接修改共享pose。`apply_two_bone`在`postprocess.rs:204-227`计算并写入root rotation，随后230-237行重建model pose并计算mid；如果第二次model reconstruction或rotation arc失败，root已经改变，调用方只收到error diagnostic，没有rollback。前一命令成功、后一命令失败时也没有batch atomicity或明确partial-commit receipt。

目标使用`RigSolveTransaction`：从sealed input pose读取，在scratch/output page求解，验证finite/topology/writer conflicts后原子publish generation。命令/phase失败策略必须是fail entry、fail rig或explicit partial policy之一；任何policy都要有typed receipt与测试，不能依赖“通常第二步不会失败”。

### ED78-P1-10：model/local rotation数学没有定义non-uniform scale、negative scale与shear语义

`resolve_model_bone`用完整matrix计算position，却用`parent_rotation * local.rotation`单独累计rotation（`postprocess.rs:332-346`）；`local_rotation`又只乘parent quaternion inverse（278-288行）。父级non-uniform/negative scale或shear下，matrix方向与独立quaternion方向可能不再代表同一frame，solver pole、look-at axis和写回结果没有明确合同。

目标冻结transform policy：允许的TRS范围、negative scale、shear、orthonormalization、mirror chain、decomposition failure与diagnostic。若Rig只支持rigid/orthogonal solve frame，compiler应在artifact admission拒绝或生成显式correction，不能在运行时默默混用两套空间。

### ED78-P1-11：诊断没有稳定source address、generation、phase、输入快照或成功receipt

`AnimationIkDiagnostic`仅有entity、optional skeleton与`AnimationIkExecutionError`（`diagnostic.rs:6-10`）；error能说MissingPose/UnresolvedTarget/InvalidChain，却不能定位rig、entry、node、unit、pin、control、space、constraint、source revision、artifact generation或phase。成功命令完全无receipt，Editor无法证明Preview/Validate对应哪次执行。

目标统一`RigDiagnosticAddress`与`RigExecutionReceipt`，至少包含document/rig instance/artifact generation/entry/phase/node/pin/element、severity/code、bounded context、input/output generation、duration和terminal disposition；Editor必须按generation拒绝stale结果。

### ED78-P1-12：validation只覆盖finite/weight/direct chain，未覆盖完整Rig可执行性

现有validation检查非有限数、weight、axis、pose shape、skeleton hierarchy和direct root-mid-tip chain。没有stable-ID唯一性、parent/space/constraint cycle、pin type/default/link、entry reachability、uninitialized external variable、phase crossing、multiple writer conflict、control limit、missing object binding、unsupported transform或artifact capability validation。

目标分为source schema、semantic compile、runtime admission三层；同一diagnostic code在Editor、cook和Runtime保持一致。Validate按钮必须执行这套validator并返回真实generation-qualified report，而不是`64 controls / 1 warning`固定文本。

### ED78-P1-13：没有viewport control shape、picking、selection、gizmo与direct manipulation bridge

当前Control Rig页面只允许选择静态row和打开dropdown；没有把control shape发布到Scene Viewport，没有qualified picking target、multi-control selection、local/global manipulator、interaction begin/update/cancel/commit、temporary control、hover/highlight或undo bracket。通用Scene gizmo不能凭control显示名直接复用，因为其target/transaction/space identity不同。

目标由`RigViewportProduct`发布generation-qualified shapes与pick proxies，Editor59/67的input/gizmo基础通过`RigManipulationSession`适配；interaction phase在scratch preview运行，commit生成typed Control value command，cancel恢复pre-interaction pose且不污染document dirty。

### ED78-P1-14：没有Backwards Solve与Sequence bake桥，无法把求解结果工程化落回动画数据

Zircon没有inverse/backwards entry、control-to-bone反解合同、sample range、display/tick rate转换、control mask、space/constraint/weight channel、key reduction或bake receipt。Editor75/77已有时间轴与Sequence owner，但Control Rig没有任何桥接类型，因而“space switch keyed”只是文案。

Unreal Bake helper会要求可反解Control Rig、创建Sequencer section、收集导入前key times、加载animation sequence并对新增范围执行smart reduce或fixed bake；Zircon应建立`RigBakePlan -> RigBakeScratch -> AnimationEditTransaction -> RigBakeReceipt`，支持cancel/rollback、range/rate、selection mask、tolerance与error report，最终只写Editor77 canonical source。

## 6. 新增P2扩展差距

### ED78-P2-01：Modular Rig、Connector、Function Library、Template与versioned unit upgrade尚未建立

工程规模需要可嵌套rig module、connector/resolve rule、function library、template parameter与dependency package。它们必须建立在P1 stable identity/compiler上；禁止先用字符串include、复制节点或不可迁移宏临时实现。

### ED78-P2-02：缺少watch pin、breakpoint、single-step、control influence、phase timing与可回放trace

当前只有粗错误和ignored micro benchmark。目标为compiled source map提供bounded watch/breakpoint/trace，按entry/unit/phase统计CPU、scratch、cache、solver iteration，并可冻结输入重放；disabled时必须接近零成本。

### ED78-P2-03：没有Rig source migration、structural diff/merge、multi-user conflict与review artifact

stable ID建立后还需version migration、hierarchy/graph semantic diff、rename/move/link/setting冲突、review annotation和multi-user lock/merge。不能把序列化文本行diff当作Rig语义合并。

### ED78-P2-04：大Hierarchy/Graph的搜索、过滤、virtualization、incremental compile与批量编辑没有预算

当前64 controls只是固定数字。真实产品要对10K element、数千unit、多选属性和大型function library建立paged projection、query index、visible virtualization、incremental validation/compile、cancel与memory budget，并与Editor56通用搜索结果使用qualified address。

### ED78-P2-05：专项测试、fault、soak、profile与跨引擎同语义资格矩阵缺失

现有focused tests覆盖TwoBone/LookAt基础、scratch复用和pipeline顺序，没有source roundtrip、migration、space switch、constraint cycle、overlapping writer、atomic rollback、non-uniform scale、direct manipulation、bake、hot reload、plugin revoke、large-rig或1小时soak。性能超过Unreal只能在同骨数、同unit/solver、同更新率、同输出精度、同线程/硬件/warmup的profile中声明。

## 7. 五套参考源码裁决

### 7.1 Unreal：Control Rig产品与Rig VM的主架构参考

`ControlRig.h`把DynamicHierarchy、RigVM execute context、object binding、construction/forward/backward support、control selection、undo bracket、pre/post events、transient viewport controls和trace放在同一host生命周期中。`RigHierarchyElements.h`定义typed element与current/initial local/global transform；四类Execution Unit显式声明Construction、Forwards、Backwards与Interaction事件。

`RigVM.h`拥有bytecode、literal/work/debug memory、external variables、entry/instruction；compiler把AST、memory、operand、watched pin和instruction build分阶段。Blueprint compiler复制并初始化hierarchy；ControlRig Edit Mode拥有shape/gizmo/selection/Sequencer binding；MovieScene section拥有control/space/constraint/weight channels；Bake helper用transaction、反解、range和key reduction闭环。

裁决：Unreal是Editor78的产品边界主参考，但Zircon不复制UObject/反射布局。Zircon采用Rust typed source、immutable artifact、dense memory、explicit generation和transactional output，并以数据并行/缓存友好的program执行作为性能目标。

### 7.2 Godot：Skeleton modifier、IK品类与轻量Editor操纵参考

`SkeletonModifier3D`有active/influence与process hook；`Skeleton3D`保存rest/local/global pose、dirty cache和physics/idle/manual modifier callback。TwoBone支持多setting、virtual/extended end、pole node/direction和cached joint rotation；LookAt提供origin/target、forward axis及主/次轴角度限制。Chain/Iterate/FABR/CCD/Jacobian与BoneConstraint展示solver family、iteration/error threshold和joint axis/constraint边界。

Skeleton Editor有rest/pose/meta字段、UndoRedo、reset、pose-to-rest、insert keys及gizmo/subgizmo transform/commit。裁决：Godot证明“即使不建完整Rig VM，也必须有真实modifier lifecycle、typed settings、pose cache和Editor操纵”；但Zircon目标高于Godot，不能以Node列表替代compiled Rig program。

### 7.3 Fyrox：typed pose/blend与Editor command纪律的受限参考

Fyrox `pose.rs`提供typed `NodePose / AnimationPose`、root motion与复用式clone/blend；machine `PoseNode`明确只有Play/Blend/BlendByIndex/BlendSpace。聚焦源码没有第一类Control Rig/IK authoring产品。

裁决：借鉴其typed handle、pose ownership和command execute/revert纪律；不得把普通AnimationPose或blend tree称作Control Rig，也不得从缺失实现反推出Zircon可以降低目标。

### 7.4 Bevy：serialized graph asset与prepared traversal的受限参考

Bevy `AnimationGraph`是可序列化/加载/保存的Asset，区分serialized path reference与runtime handle，并构建`ThreadedAnimationGraph`、threaded nodes与sorted edges。聚焦版本没有Control Rig hierarchy、controls、spaces、constraints或IK authoring。

裁决：借鉴source/runtime graph分离、asset path迁移和prepared traversal；Rig Unit语义、phase、VM与Editor闭环仍以Unreal/Godot证据设计。

### 7.5 Unity Graphics：只作为deformation consumer边界

ShaderGraph Linear Blend Skinning node拥有typed position/normal/tangent slots并加载skin matrix；Compute Deform node消费structured deformed vertex buffer；VFX Graph暴露SkinnedMeshRenderer与root transform。该仓内Graphics快照不是Unity Animation Rigging包，没有Control Rig authoring证据。

裁决：它只证明最终Rig/Pose产物必须以稳定palette/deformed stream供Renderer/VFX消费。不得用它证明Control Rig source、solver或Editor功能已经有参考实现。

## 8. 目标架构与唯一authority

### 8.1 Source、compile、runtime与product分层

```text
ControlRigSourceDocument
  RigHierarchySource
    BoneRef | Control | Null | Curve | Connector
    Initial/Current Value | Local/Global Transform | Metadata
    Parent/Space/Constraint Definitions
  RigGraphSource
    Entry/Phase | RigUnit Node | Typed Pin/Edge | External Variable
  Binding/Dependency/Version/Revision
              |
              v
ControlRigCompiler
  Schema -> Topology -> Types -> Phases -> Read/Write Conflicts
  Dense Layout -> Instruction/Kernel Plan -> Debug Map -> Digest
              |
              v
CompiledRigProgram (immutable, generation-qualified)
  CompiledHierarchy | EntryTable | Operand/Memory Pages
  Parameter Binding | Capability | SourceMap | LastGood Relation
              |
              v
RigRuntimeService
  RigInstance + InputSnapshot + SealedPoseGeneration
  RigSolveTransaction(Construction/Forward/Backward/Interaction)
  Atomic Pose/Control Output + Diagnostic/Execution Receipt
              |
        +-----+------------------+
        v                        v
RigPreviewSession           RigBakeCoordinator
Viewport Shapes/Gizmos      AnimationEditTransaction
Selection/Manipulation      Sequence/Control Channels
```

### 8.2 Owner合同

| Owner | 必须拥有 | 禁止拥有 |
|---|---|---|
| Control Rig source | hierarchy/control/space/constraint/rig graph/version | runtime scratch、viewport node |
| ControlRigCompiler | validation、typed lowering、artifact、debug map | Editor widget状态、World mutation |
| RigRuntimeService | instance/input/phase solve/atomic publish/receipt | source migration、文件I/O |
| Animation Runtime | sealed pose、schedule、prepared artifact cache | Rig Editor document |
| Rig Editor | document/view/selection/manipulation/diagnostic projection | 自造solver、绕过artifact写pose |
| Bake Coordinator | sample plan、backward solve、atomic sequence edit | 直接改Runtime pose cache |
| Renderer/VFX | palette/deformation消费 | hierarchy/graph求解语义 |

### 8.3 必须硬切的旧路径

1. Control Rig Workbench在真实capability接线前必须显示Unavailable/Prototype，不得继续返回固定Preview/Validate成功文案。
2. 删除以显示字符串`CR_Hero / Hand_IK_L / Weight: 1.00`充当asset/control/value identity的路径。
3. TwoBone/LookAt迁入compiled rig unit/runtime kernel后，禁止继续扩张巨型`AnimationIkCommand`作为Rig authoring ABI。
4. 禁止逐command同步load skeleton、逐commandresolve target和原地半写pose；Runtime08C P1-17与ED78-P1-09必须共同关闭。
5. Preview、PIE、cook与bake禁止执行不同schema或临时solver；全部只接受同一`CompiledRigProgram`版本。
6. 禁止Editor直接写骨骼scene node冒充Control Rig output；只发布sealed pose generation并由既有projection/renderer owner消费。

## 9. 重构里程碑

### ED78-M0：Owner、truth table、corpus与RED证据

- 锁定Editor14/32/63/75/76/77、Runtime08C边界与hard-cut列表。
- 增加static product truth test，证明当前Preview/Validate只有文本反馈。
- 增加atomic rollback、overlapping writer、non-uniform scale和stale generation RED测试。

### ED78-M1：Stable source identity、Hierarchy与Control schema

- 建立versioned `ControlRigSourceDocument`与migration envelope。
- 建立typed hierarchy/control/value/settings/initial-current/local-global模型。
- 引用Editor32 skeleton artifact，拒绝复制骨架schema。

### ED78-M2：Space、Constraint与transactional authoring

- 建立space/multi-parent/constraint/maintain-offset/cycle模型。
- 接入Editor63 transaction、savepoint、undo/redo与revision CAS。
- space switch输出补偿计划，不直接写显示值。

### ED78-M3：Rig Unit Graph与唯一compiler

- 建立unit registry、typed pins/edges/external variables/version upgrades。
- 定义Construction/Forward/Backward/Interaction phase、read/write set和conflict validation。
- 与Editor76共享compiler foundation，不建立第三个graph compiler。

### ED78-M4：Compiled program、memory与last-good

- 输出immutable program、dense hierarchy/operand/memory、entry schedule和source map。
- 建立artifact digest、capability、dependency/currentness与last-good install。
- cook/preview/PIE加载同一artifact格式。

### ED78-M5：Atomic Rig Runtime与基础solver收敛

- 把TwoBone/LookAt变成compiled units，复用Runtime08C prepared pose与scratch。
- 输入快照、phase output page、writer conflict与atomic publish闭环。
- 补orientation/space/transform policy；limits/性能热路径依Runtime08C P1-17实施。

### ED78-M6：Diagnostics、debug与Runtime-backed preview

- 建立source-qualified diagnostic/execution receipt与stale reject。
- Preview/Validate执行真实artifact和validator，删除固定反馈。
- 建立watch/trace/phase timing最小闭环。

### ED78-M7：Viewport Control与direct manipulation

- 发布control shapes/pick proxies，接入qualified selection/gizmo/input capture。
- interaction preview与commit/cancel transaction闭环。
- 多control、space、limits和hot reload恢复通过动态测试。

### ED78-M8：Backwards Solve与Bake

- 建立inverse entry、bake plan/range/rate/mask/tolerance/cancel。
- 通过Editor77 canonical Sequence source执行atomic edit与key reduction。
- Sequencer/Control channel、space/constraint/weight channel有稳定ID与receipt。

### ED78-M9：规模、fault、soak与性能资格

- 大Hierarchy/Graph virtualization、incremental compile、batch solve和memory budget。
- plugin revoke、asset reload、compile failure、device loss相关consumer、cancel/rollback/soak。
- 在同语义同质量条件下与参考实现profile；无证据不得写“超过Unreal”。

## 10. 48个资格门

当前静态状态：**ED78-G01至ED78-G48全部Fail**。已有TwoBone/LookAt unit test只证明局部数学与scratch，不等于Control Rig source -> compile -> runtime -> Editor -> bake产品链通过。

| Gate | 资格 | 当前 |
|---|---|---|
| ED78-G01 | versioned ControlRig source roundtrip byte/semantic稳定 | Fail |
| ED78-G02 | schema migration与unknown field/version fail-close | Fail |
| ED78-G03 | stable asset/document/element/node identity跨save/reopen稳定 | Fail |
| ED78-G04 | skeleton dependency与source revision CAS明确 | Fail |
| ED78-G05 | Bone/Control/Null/Curve/Connector typed hierarchy | Fail |
| ED78-G06 | initial/current、local/global transform/value语义完整 | Fail |
| ED78-G07 | parent/multi-parent topology与cycle diagnostic | Fail |
| ED78-G08 | typed Control value/settings/limits/shape schema | Fail |
| ED78-G09 | Space identity、maintain-offset与补偿切换 | Fail |
| ED78-G10 | Constraint identity、weight、order与cycle policy | Fail |
| ED78-G11 | Rig Unit registry支持plugin generation/revoke | Fail |
| ED78-G12 | typed pin/default/link/external variable validation | Fail |
| ED78-G13 | node/function/unit version upgrade deterministic | Fail |
| ED78-G14 | Construction/Forward/Backward/Interaction phase明确 | Fail |
| ED78-G15 | phase DAG/read-write conflict compile期拒绝 | Fail |
| ED78-G16 | unique ControlRigCompiler，无第三authority | Fail |
| ED78-G17 | immutable CompiledRigProgram自包含且可cook | Fail |
| ED78-G18 | dense hierarchy/operand/memory布局有版本合同 | Fail |
| ED78-G19 | source debug map与artifact digest/currentness | Fail |
| ED78-G20 | compile失败last-good明确标stale | Fail |
| ED78-G21 | Preview/PIE/cook执行同一artifact | Fail |
| ED78-G22 | Rig instance/object binding有qualified lifecycle | Fail |
| ED78-G23 | dynamic input buffer带generation与space provider | Fail |
| ED78-G24 | TwoBone/LookAt由compiled unit执行 | Fail |
| ED78-G25 | solve frame内零同步asset load | Fail |
| ED78-G26 | 每rig每phase复用prepared pose/scratch | Fail |
| ED78-G27 | overlapping writer结果确定或compile拒绝 | Fail |
| ED78-G28 | command/entry失败不会留下半写pose | Fail |
| ED78-G29 | batch publish为原子sealed generation | Fail |
| ED78-G30 | negative/non-uniform scale/shear policy有oracle | Fail |
| ED78-G31 | invalid chain/axis/space/constraint不产生NaN | Fail |
| ED78-G32 | diagnostic可定位document/entry/node/pin/element | Fail |
| ED78-G33 | diagnostic与receipt携source/artifact generation | Fail |
| ED78-G34 | stale preview/validate结果被Editor拒绝 | Fail |
| ED78-G35 | Validate按钮运行真实三层validator | Fail |
| ED78-G36 | Preview按钮产生真实pose/frame/receipt | Fail |
| ED78-G37 | 静态64 controls/18 constraints反馈已删除 | Fail |
| ED78-G38 | control shape/pick proxy与frame generation一致 | Fail |
| ED78-G39 | hover/selection/multi-selection跨Hierarchy/Viewport同步 | Fail |
| ED78-G40 | gizmo begin/update/cancel/commit无dirty泄漏 | Fail |
| ED78-G41 | interaction phase obeys space/limits/constraints | Fail |
| ED78-G42 | backwards solve有roundtrip tolerance oracle | Fail |
| ED78-G43 | bake range/rate/mask/cancel/rollback正确 | Fail |
| ED78-G44 | bake只写canonical Sequence/Control channels | Fail |
| ED78-G45 | source reload/plugin revoke/session close无悬挂instance | Fail |
| ED78-G46 | 10K element/数千unit projection与compile有预算 | Fail |
| ED78-G47 | 1小时preview/manipulate/reload/bake soak无增长 | Fail |
| ED78-G48 | 同语义同质量同硬件benchmark与profile可复现 | Fail |

## 11. 实现顺序、依赖与停止条件

1. 先执行ED78-M0/M1；禁止直接给现有ZUI加更多固定control、solver或成功反馈。
2. ED78-M1依赖Editor32稳定Skeleton artifact identity；M2依赖Editor63 transaction；M3/M4依赖Editor76唯一compiler基础。
3. Runtime08C P1-17必须与ED78-M5协同：先去掉frame内load和重复model pose，再谈更多solver品类。
4. M6只有在同一artifact已由Runtime执行后才允许把Workbench从Unavailable改为Experimental。
5. M7不得直接复用通用Scene entity identity充当Rig control identity，必须经过qualified adapter。
6. M8必须写Editor77 canonical Sequence source；任何旁路channel/file writer都是停止条件。
7. P2 modular/debug/collaboration/规模工作不得绕过P1 source/compiler/runtime transaction。
8. 只有G01-G45全部Pass且G46-G48有可复现证据后，Control Rig才可标Stable；只有同语义profile持续胜出，才允许讨论超过参考引擎。

## 12. 验证边界与实施前重检

本报告的“complete”只表示本轮静态审查、owner去重、参考源码对照、差距登记和路线设计完成。当前动态状态仍是G01-G48全Fail。

开始任何production实现前必须重新执行：

1. 重取HEAD、baseline epoch、focused diff与三条共享路径租约。
2. 重新搜索ControlRig/RigGraph/IK production caller、asset type、template binding和focused tests，防止新Session改变事实。
3. 重新核对Editor14/32/63/75/76/77、Runtime08C finding状态，禁止重复修复或双authority。
4. 重算本报告25个Zircon文件fingerprint；参考树若变更则重算52文件集合并注明revision。
5. 先写RED contract/behavior/fault test，再实现source、compiler、runtime或Editor adapter。
6. 使用Windows-native验证与独立approved target目录；只有明确Linux-specific问题才进入WSL。
7. 动态验证必须分别报告unit、integration、save/reopen、cook、GUI、fault、soak、profile结果；任何未运行项明确写Not Run。

