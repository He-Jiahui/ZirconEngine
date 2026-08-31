---
title: Editor Control Rig、Rig Graph、Hierarchy、Controls、Spaces、Constraints、IK、Solve、Bake、Preview、Compiler 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor199
review_date: 2026-08-28
baseline_head: a721407083c5652619eb4b8743ae063fbb11fccf
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/animation/workbench_extension_control_rig_workspace.zui
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/gameplay_animation.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_runtime/src/core/framework/animation/runtime_status.rs
  - zircon_plugins/animation/runtime/src/ik
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
tests:
  - zircon_plugins/animation/runtime/tests/animation_ik_contract.rs
  - zircon_runtime/src/core/framework/animation/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/198-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/78-editor-control-rig-rig-graph-hierarchy-controls-spaces-constraints-ik-solve-bake-preview-compiler-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/78-editor-control-rig-rig-graph-hierarchy-controls-spaces-constraints-ik-solve-bake-preview-compiler-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Control Rig、Rig Graph、Hierarchy、Controls、Spaces、Constraints、IK、Solve、Bake、Preview、Compiler 与 Product Integration 当前源码复核

## 1. 结论

当前源码中的 Control Rig 产品仍是静态演示面，而不是可创作、可编译、可执行的工程系统。230 行 ZUI 继续把 `CR_Hero`、`Spine_CTRL`、`Hand_IK_L`、`64 controls`、`18 constraints` 和 `1 warning` 写死；20 个 action 只进入通用 workspace/tab/row/command/field 路由。Preview 和 Validate 的最终效果是把固定 `queued` 文本写入状态栏与 output row，control、space、weight 的 edit/commit 没有 Control Rig document、typed mutation、transaction、compile、runtime request 或 receipt。

旧 Editor78 对 Runtime 底座的正面判断已经失效。当前工作树删除了 framework `AnimationIkCommand`/error、插件 IK diagnostic/execution error/postprocess 以及 runtime integration test，共 6 个文件、1,081 行；`AnimationManager` 也移除了 queue/drain API，animation tick 不再调用 IK postprocess。保留下来的只有纯值 `TwoBoneIkJob::solve_positions` 和 `LookAtJob::solve_rotation`，生产源码无调用方，只有 3 个局部数学测试。它们可作为未来 compiled rig unit 的算法种子，但不再构成 per-world admission、pose postprocess、atomic publish 或诊断链。

共享 Animation compiler 的新增也不能被误算成 Control Rig compiler。当前 `zircon_runtime::core::framework::animation::compiler` 只覆盖 Graph、State Machine 和 Sequence；Runtime asset enum、compiler product、Animation 插件 pipeline 与 Editor document 中没有 ControlRig source、hierarchy、unit graph、compiled rig program 或 rig instance。`AnimationRigRuntimeStatus` 名称虽然包含 Rig，但字段仅是 skeleton、bone count、avatar mask、GPU skinning 与 missing targets，且除构造测试和导出外没有生产填充者；它是骨架姿态状态 DTO，不是 Control Rig execution receipt。

本轮不新增父报告 P0。Editor14 的静态成功界面/无真实 toolkit 与 preview、Editor76/197 的 compiler/runtime authority、Editor63/184 的 transaction、Editor77/198 的 canonical Sequence source、Runtime08C 的 pose/skeleton/solver 热路径仍各自拥有通用阻断。本报告重判 Editor78 的 14 项 P1 为 **11 Open / 3 Partial / 0 Closed**，5 项 P2 为 **4 Open / 1 Partial / 0 Closed**；48 个资格门为 **46 Fail / 2 Partial / 0 Pass**。Partial 只承认纯函数求解先返回值再由调用者决定提交、基础 finite/weight/axis 校验和少量数学测试，不表示存在 Control Rig runtime。

本轮只做 current-source review 和文档建账，不修改 production Rust/ZUI。未运行 Cargo、Editor、GUI/GPU、save/reopen、cook、live preview、viewport manipulation、bake、fault/soak/profile 或跨引擎 benchmark，因此不能宣称当前功能完整、正确、性能达标，更不能宣称超过 Unreal。

## 2. 审查边界、owner 与冻结语料

### 2.1 本报告唯一纵向边界

本报告只拥有：`ControlRigSourceDocument -> typed hierarchy/control/space/constraint/unit graph -> ControlRigCompiler -> immutable CompiledRigProgram -> generation-qualified RigInstance/RigSolveTransaction -> Editor preview/direct manipulation/backwards solve/bake`。

- Editor14 继续拥有 Animation 默认 toolkit 可达性、静态成功 UI 与通用 preview/save/compile 真实性 P0。
- Editor32 继续拥有 Skeleton/Skin/import/reimport/retarget identity；Control Rig 只能引用稳定 Skeleton artifact。
- Editor63/184 继续拥有 document transaction/history/savepoint/async operation 总合同。
- Editor75/196 继续拥有 Timeline/Curve/Track/Key/transport 交互。
- Editor76/197 继续拥有共享 Graph compiler 基础和 Animation runtime authority 硬切；Rig Unit phase/hierarchy/VM 语义由本报告拥有。
- Editor77/198 继续拥有 Sequence/Clip/channel/event/prepared artifact/playback；Control Rig 只通过正式 bridge 写入其 canonical source。
- Runtime08C 继续拥有 pose/skeleton scheduling、joint limit、full-body/foot-lock/facial/cloth 与 solver 热路径；本报告不重复登记 solver 品类平台。

### 2.2 Currentness 与在途删除

- HEAD 锚点：`a721407083c5652619eb4b8743ae063fbb11fccf`；结论以 2026-08-28 当前磁盘内容为准，而不是只看 HEAD。审查期间从 `9501ef4afb1e61b2e97e5f4084375741f016a59e` 前进的提交未触及本报告冻结路径。
- 相关工作树处于 dirty 状态；Control Rig ZUI、feedback、Animation runtime/compiler/pipeline 等均有非本轮变化。本报告不回滚、不覆盖这些变化。
- `git diff --stat` 显示旧 IK command/postprocess 路径的 6 个文件合计删除 1,081 行；删除尚在工作树中，因此这是当前源码事实，但在合并前仍需实施者重检。
- 按用户要求，本轮没有查询、轮询、等待或实时跟踪协调器；没有使用协调器状态作为报告证据。

### 2.3 冻结语料与可复算 fingerprint

统计口径：路径转小写、`/` 规范化并排序；每个文件取 SHA-256，再按 `path + NUL + lowercase file hash + LF` 聚合集合 fingerprint。行数按文本换行计数，非空行按首个非空白字符计数。

| 范围 | 文件 / 行 / 非空行 / bytes | fingerprint |
|---|---:|---|
| Zircon Editor/product | **9 / 5,067 / 4,919 / 230,813** | `0cf90da14730cdac72deddd505d40f6217a7da2533bd93bb2239a97f3514eb64` |
| Zircon Runtime/IK boundary | **12 / 1,612 / 1,471 / 56,221** | `dc90374a690ea70e969541185c27794bba2a5aa46bc835bf859ecd8bca760286` |
| Zircon focused tests | **2 / 183 / 160 / 6,598** | `8adc650097216866de6665e4e6c9a66c81df7fcc0410da801e7ce69ca165a8a6` |
| Zircon deduplicated focused set | **23 / 6,862 / 6,550 / 293,632** | `a651f90802eb5bf3e4cda9ddc0214e84b812eb9ecb43ff2ff1db8327a92a60eb` |
| Unreal selected set | **17 / 26,280 / 22,693 / 992,887** | `b4b75d54d5c5b68ffdd31fa74fab77acedf875d0bc03a1f6e14422df9ca9ab05` |
| Godot selected set | **26 / 9,916 / 8,456 / 392,691** | `fbce9f6ae05849256e601d117fce814c637f234cc1bd710469195ce266cfd990` |
| Fyrox selected set | **3 / 770 / 692 / 30,487** | `294bd20690fde6a3168df3c029f57ebbd63ecc7db87942db7b4ba4b415df79fd` |
| Bevy selected set | **2 / 1,018 / 931 / 41,097** | `f0d6092cf979af8f4ab08466340cf4c2d18cb912c6c092e56983db99a38303e1` |
| Unity Graphics selected set | **4 / 462 / 414 / 21,159** | `f087df51937d0fb6a582d70655f297e60c15d36fecffda8b84d08132b7726bde` |
| Five-engine reference total | **52 / 38,446 / 33,186 / 1,478,321** | `7a209677ff73fef96d84f86ecc72a669193ca9c239213b0c934cddb2f7a03f5b` |

## 3. 当前真实实现与旧报告校正

### 3.1 Editor surface 仍是固定 projection

Control Rig ZUI 的三个标签、三个 hierarchy row、四个 solve row、两个 command 和三个 field 都是真实可渲染控件；template binding 与 navigation hash index 也是可保留的通用 UI 基础。但它们没有领域模型：route 只保存 workspace/tab/row/command control ID 和 `field_action: bool`，没有 asset/document/session/element/node/revision/generation。

`apply_workbench_extension_module_command_feedback` 只修改 `WorkbenchStatusReady.text`、`WorkbenchStatusMessages.text` 和 output row 的 `value_text`。Control Rig open/preview/validate 分支直接返回 `CR_Hero`、`Hand_IK_L`、`64 controls`、`1 warning` 固定文本。静态 action 白名单测试只证明 action 可路由，不证明 mutation、compile 或 preview 执行。

### 3.2 旧 IK production bridge 已从当前磁盘移除

| Editor78 旧事实 | 当前源码 | 重判 |
|---|---|---|
| `AnimationIkCommand` 与 per-world queue/replacement epoch | framework command/error 删除，manager queue/drain API 删除 | 不再是可保留 Runtime 底座 |
| tick 在 simulated pose 后、scene apply 前调用 IK | `apply_ik_commands` import/call 已删除 | Control Rig/IK 无生产调度点 |
| postprocess 复用 model-pose scratch 并写 pose | 650 行 `postprocess.rs` 删除 | 无 runtime pose consumer/publisher |
| execution error/diagnostic | 两个文件删除 | 只剩四值 `AnimationIkError`，无 source address/receipt |
| integration/atomicity tests | 235 行 `ik_postprocess` test 删除 | 只剩 3 个算法 happy-path test |
| TwoBone/LookAt 数学 | 两个纯值 Job 保留 | 可迁移算法种子，不能宣称系统能力 |

`TwoBoneIkJob` 验证 target/pole/root/mid/tip finite 和 `[0,1]` weight，检查两段长度，再返回 root/mid/tip 新值；`LookAtJob` 验证 direction/axis/current/clamp finite、非零轴和 weight，再返回归一化 quaternion。两者不加载 skeleton、不修改 pose，因此局部函数失败不会半写调用者状态。这只关闭了旧“函数内部先写 root 再失败”的具体实现形态，没有提供 rig/entry/batch transaction、writer conflict 或 sealed generation。

### 3.3 `AnimationRigRuntimeStatus` 不是 Control Rig

该 DTO 只有 world/entity/skeleton、bone/posed count、avatar mask、GPU readiness、missing targets 和字符串 diagnostics。生产搜索只命中定义、re-export 与构造测试，没有 manager/pipeline 填充者；它也没有 rig asset、entry、phase、control、compiled generation、input/output pose generation 或 terminal disposition。后续应避免因命名中的 `Rig` 把 skeletal pose coverage 误报为 Control Rig runtime。

### 3.4 Shared Animation compiler 与 Control Rig 仍完全断开

当前 shared compiler 已有 Graph/State Machine/Sequence typed IR 和诊断，这是 Editor76/197、Editor77/198 的真实进展。但 compiler product 无 `ControlRig` variant，asset schema无 Rig hierarchy/unit graph，Animation runtime pipeline 无 compiled rig install/evaluate，Editor Control Rig surface 也没有 document owner。应复用 compiler foundation，不得把现有 Animation Graph artifact 强转命名成 Rig program，也不得建立第三套临时图编译器。

## 4. P1：Control Rig 生产差距

### ED199-P1-01 · Open · 没有 canonical `ControlRigSourceDocument`、版本、revision 与 binding identity

产品唯一 rig 身份仍是模板字符串 `CR_Hero`。不存在 asset/document/instance ID、source schema version、migration、Skeleton dependency、object binding、source revision、compiled generation、last-good relation或 cook identity。目标至少建立 `ControlRigAssetId + ControlRigDocumentId + RigElementId + RigNodeId + SourceRevision + SkeletonArtifactId`，open/save/reload/preview/PIE/cook 必须按 qualified identity 与 CAS 工作。

### ED199-P1-02 · Open · 没有 typed hierarchy、稳定 element identity 与 initial/current transform 合同

Runtime 只有 Skeleton asset/target table，Editor 只有通用 row ID。缺 Bone/Control/Null/Curve/Connector、parent/multi-parent topology、local/global、initial/current、topology/pose version、metadata 和 cycle policy。目标为 `RigHierarchySource -> CompiledRigHierarchy`，引用 Editor32 的 Skeleton artifact，而不是复制第五份骨架 schema。

### ED199-P1-03 · Open · Control value/settings 仍是字符串，没有 typed value、limit、shape 与 value lifecycle

Control dropdown 和 `Weight: 1.00` 不携 Bool/Float/Integer/Vector/Rotation/Transform 类型、Euler policy、min/max、limit enable、initial/current、offset/shape、animatable/transient、visibility/selectability。目标分离 `RigControlDefinition` 与 `RigControlValue`，Editor field 由 schema 投影，invalid value fail-close，limit 对 authoring/preview/bake/runtime 一致。

### ED199-P1-04 · Open · Space、multi-parent、constraint 与 maintain-offset 图不存在

World/Local/Parent 只是静态 options；仓内没有 stable Space/Constraint identity、parent weight、offset、active interval、priority、evaluation order、cycle diagnostic 或 compensation key。space switch 必须在 transaction 中生成补偿计划，并通过 Editor77/198 canonical channel 写入，不能只改 dropdown 文本。

### ED199-P1-05 · Open · 没有 Rig Unit registry、typed pins、node/edge、external variable 与 upgrade schema

所谓 Solve Graph 是四个 table row。现有 shared Graph compiler不认识 hierarchy read/write、Control、Space、Constraint、solver unit 或 execution pin。目标在 Editor76/197 compiler foundation 上建立 versioned `RigUnitDescriptorRegistry`、typed pins/edges、external variable、function library、node upgrade 与 plugin generation/revoke；未知/失效 unit 必须 Unavailable/fail-close。

### ED199-P1-06 · Open · 没有 Construction/Forward/Backward/Interaction phase 与 read/write schedule

当前既没有 IK production stage，也没有 phase/entry/dependency/read-write set。目标定义 typed phase、entry DAG、barrier、writer ownership 与 deterministic schedule；无效跨 phase dependency、cycle 和 overlapping writer 必须 compile 拒绝或使用显式仲裁，不能依赖 Vec/HashMap 遍历顺序。

### ED199-P1-07 · Open · 没有 compiled Rig program、memory layout、debug map、artifact currentness 与 LKG install

缺 instruction/kernel plan、constant/work/external/debug memory、dense hierarchy/operand page、entry table、source map、artifact digest/ABI/capability 和 current/LKG install。Preview、PIE、cook 必须执行同一 immutable `CompiledRigProgram`；compile 失败可保留明确 stale 的 LKG，但不得把旧结果显示为 current。

### ED199-P1-08 · Open · 没有 generation-qualified Rig instance、input snapshot、object binding 与 lifecycle

旧 command ABI 已删除，当前两个 Job 只接收瞬时向量/四元数。缺 RigInstanceId、artifact generation、source/node address、space provider、dynamic parameter page、input/output pose generation、plugin/world/session retire 和 object binding。不要重新膨胀一个 command enum；动态目标应写入 typed input snapshot，由 compiled entry 消费。

### ED199-P1-09 · Partial · 纯值 kernel 避免函数内半写，但没有 phase/batch solve transaction 与 atomic publish

当前两个 Job 均先验证并返回新值，失败时不会直接改变外部 pose，这是比旧 postprocess 原地写更安全的局部形态。但仓内没有读取 sealed pose、scratch/output page、writer conflict validation、entry failure policy、batch commit 或 generation publish。目标仍是 `RigSolveTransaction`，明确 fail-entry/fail-rig/explicit-partial policy 并产生 terminal receipt。

### ED199-P1-10 · Open · transform、orientation、scale、mirror 与 constraint 数学合同不完整

TwoBone 只求 model-space 三点，root 原样返回；不输出 joint rotation、effector orientation、twist/stretch、preferred angle 或 local transform。LookAt 只有单 axis 和对旋转弧的总角 clamp，没有 primary/secondary axis limits。两者都没有 non-uniform/negative scale、shear、mirror、decomposition 和 orthonormalization policy。compiler/runtime 必须冻结允许域并有 golden oracle。

### ED199-P1-11 · Partial · 有粗错误枚举与 skeletal status DTO，但没有 source-qualified diagnostic/execution receipt

`AnimationIkError` 只有 NonFiniteInput、DegenerateChain、DegenerateAxis、InvalidWeight；`AnimationRigRuntimeStatus.diagnostics` 是字符串且无 production publisher。缺 document/instance/artifact generation/entry/phase/node/pin/element、severity/code、bounded context、duration、input/output generation 与 terminal disposition。Editor 必须按 generation 拒绝 stale Validate/Preview 结果。

### ED199-P1-12 · Partial · kernel 有 finite/weight/axis 校验，但没有三层 Rig validation

局部 Job 能拒绝 non-finite、invalid weight、零轴和零长骨段；没有 stable ID uniqueness、hierarchy/space/constraint cycle、pin/default/link type、entry reachability、external binding、phase crossing、multiple writer、control limit、unsupported transform 或 artifact capability validation。目标统一 source schema、semantic compile、runtime admission 三层 diagnostic code。

### ED199-P1-13 · Open · 没有 runtime-backed viewport shape、picking、selection、gizmo 与 direct manipulation

当前只能选择静态 row 或编辑字符串 field。缺 generation-qualified control shape/pick proxy、hover、多选、local/global gizmo、interaction begin/update/cancel/commit、temporary control 和 undo bracket。通用 Scene entity gizmo 不能直接以显示名充当 Rig control identity；必须有 `RigManipulationSession` adapter 和 scratch interaction phase。

### ED199-P1-14 · Open · 没有 Backwards Solve、Control channel 与 Sequence bake bridge

缺 inverse entry、control-to-bone round-trip、sample range/rate、mask、space/constraint/weight channel、key reduction、cancel/rollback 和 bake receipt。目标链为 `RigBakePlan -> RigBakeScratch -> AnimationEditTransaction -> RigBakeReceipt`，最终只写 Editor77/198 canonical Sequence source；任何旁路文件/channel writer 都应停止。

## 5. P2：模块化、调试、协作、规模与资格债务

### ED199-P2-01 · Open · Modular Rig、Connector、Function Library、Template 与 unit migration 未建立

大型工程需要可嵌套 rig module、connector/resolve rule、function library、template parameter、dependency package 与 versioned unit upgrade。它们必须建立在 P1 stable identity/compiler 上，禁止先用字符串 include、复制节点或不可迁移宏实现。

### ED199-P2-02 · Open · watch、breakpoint、single-step、influence、phase timing 与可回放 trace 缺失

目标为 compiled source map 提供 bounded watch/breakpoint/trace，按 entry/unit/phase 统计 CPU、scratch、cache、solver iteration，并可冻结输入重放；disabled 时应接近零成本。当前四值 error 不能支持任何一项。

### ED199-P2-03 · Open · source migration、semantic diff/merge、multi-user conflict 与 review artifact 缺失

需要 version migration、hierarchy/graph structural diff、rename/move/link/setting conflict、review annotation、lock/merge。文本行 diff 不能代替 stable element/node identity 上的 Rig 语义合并。

### ED199-P2-04 · Open · 大 hierarchy/graph 的 virtualization、incremental compile 与 batch solve 没有预算

固定 `64 controls` 不是规模证据。必须为 10K element、数千 unit、多选属性、function library 建立 paged projection、query index、visible virtualization、incremental validation/compile、cancel、memory budget；Runtime 需要 prepared batch、无 frame 内同步 I/O 和可复现 P50/P95/P99。

### ED199-P2-05 · Partial · 少量数学测试存在，但专项 fault/soak/profile/资格矩阵几乎为空

3 个测试覆盖 TwoBone 可达/不可达和 LookAt clamp；没有 invalid input、property/orientation、scale/mirror、hierarchy、space switch、constraint cycle、overlapping writer、atomic rollback、hot reload、direct manipulation、bake、plugin revoke、large rig 或 1 小时 soak。静态 action 白名单也不证明产品行为。超过 Unreal 只能由同骨数、同 unit/solver、同更新率、同精度、同线程/硬件/warmup 的 profile 与 correctness receipt 支撑。

## 6. 五套参考源码的工程裁决

### 6.1 Unreal：Control Rig 产品边界与 RigVM 主架构参考

`ControlRig.h` 把 dynamic hierarchy、VM lifecycle、construction、forward/backward support、control value/transform、parent switch、limit、selection 和 object binding 放在同一 host；`RigHierarchyElements.h` 明确 typed element、single/multi-parent，以及 current/initial、local/global transform/dirty state。Execution units 显式提供 Forwards、Backwards 与 Interaction 事件。

`RigVM.h` 同时拥有 bytecode、literal/work/debug memory、entry/instruction 与 instance initialization；compiler 明确分 SetupMemory/BuildInstructions，并跟踪 watched pins。ControlRig Editor 的 edit mode、Sequencer section 和 Bake helper又覆盖 shape/gizmo/selection、control/space/constraint/weight channels、transaction、backwards solve、range 和 key reduction。

裁决：Zircon 不复制 UObject/反射布局，但必须达到同等清晰的 source/compiler/artifact/runtime/editor/bake 分层。Rust typed source、immutable artifact、dense data、explicit generation、transactional output 与批处理性能可以成为超越点；两段纯数学函数不能替代这条纵向链。

### 6.2 Godot：modifier lifecycle、pose cache、solver setting 与 Editor transaction 下限

`SkeletonModifier3D` 有 active/influence/process hook；`Skeleton3D` 保存 rest/local/global pose、dirty cache 和 modifier pose backup。TwoBone 缓存 joint/rest/current rotation并提供更完整 solver setting，LookAt 支持主/次轴角度限制。Skeleton Editor 使用 UndoRedo action 对 transform/metadata 等变更执行 do/undo/commit。

裁决：即使 Zircon 暂不一次完成 RigVM，也至少要有真实 modifier/instance lifecycle、typed settings、pose cache、transaction 与 Editor 操纵。当前孤立 Job 加静态 ZUI 连 Godot 的轻量工程下限都未达到。

### 6.3 Fyrox：typed pose ownership 的受限参考

Fyrox `NodePose/AnimationPose` 提供 typed handle、可复用 clone/blend 和 root motion，但聚焦源码没有第一类 Control Rig authoring。可借鉴 pose ownership 与 command execute/revert 纪律；不得把普通 pose/blend tree 改名为 Control Rig，也不得因参考缺失而降低目标。

### 6.4 Bevy：serialized graph asset 与 prepared traversal 的受限参考

Bevy `AnimationGraph` 区分 serialized graph/path reference 与 runtime handle，并维护 prepared/threaded traversal。它证明 source asset 与 runtime traversal 应分离；聚焦版本没有 Rig hierarchy/control/constraint/IK authoring，因此 Rig Unit phase、VM 和 Editor 闭环仍以 Unreal/Godot 裁决。

### 6.5 Unity Graphics：仅作为 deformation consumer 边界

Unity Graphics 快照中的 Linear Blend Skinning/Compute Deform/VFX 节点消费 skin matrix/deformed vertex stream，不包含 Animation Rigging package 的 authoring/runtime。它只能约束最终 sealed pose/palette 如何被 Renderer/VFX 消费，不能证明 Control Rig source、solver 或 Editor 已实现。

## 7. 目标架构与唯一 authority

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
  Dense Layout -> Kernel Plan -> Debug Map -> Digest
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

| Owner | 必须拥有 | 禁止拥有 |
|---|---|---|
| Control Rig source | hierarchy/control/space/constraint/graph/version | runtime scratch、viewport node |
| ControlRigCompiler | validation/lowering/artifact/debug map | Editor widget 状态、World mutation |
| RigRuntimeService | instance/input/phase solve/atomic publish/receipt | source migration、文件 I/O |
| Animation Runtime | sealed pose/schedule/prepared artifact cache | Rig Editor document |
| Rig Editor | document/view/selection/manipulation/diagnostic projection | 自造 solver、绕过 artifact 写 pose |
| Bake Coordinator | sample plan/backward solve/atomic sequence edit | 直接改 runtime pose cache |
| Renderer/VFX | palette/deformation 消费 | hierarchy/graph 求解语义 |

必须硬切：

1. 在真实 capability 接线前，Control Rig Workbench 必须显示 Unavailable/Prototype，不能继续返回固定 Preview/Validate 成功形状。
2. 删除 `CR_Hero / Hand_IK_L / Weight: 1.00 / 64 controls / 1 warning` 充当 identity/value/result 的路径。
3. 不恢复旧巨型 `AnimationIkCommand` 队列；TwoBone/LookAt 迁入 compiled unit/kernel，动态输入进入 generation-qualified buffer。
4. Preview、PIE、cook、bake 只接受同一 `CompiledRigProgram` 格式和 generation。
5. Editor 不得直接写 Skeleton/Scene node 冒充 Rig output；只提交 solve transaction 并发布 sealed pose generation。
6. Control Rig source/compiler/runtime 不得再建一套 Skeleton、Sequence、Graph、transaction 或 selection authority。

## 8. 重构里程碑

### ED199-M0：product truth、owner、corpus 与 RED guards

- 将当前 Workbench 标为 Prototype/Unavailable，增加 test 证明 Preview/Validate 只有固定反馈。
- 冻结 source/compiler/runtime/editor/bake owner 与 hard-cut 列表。
- 增加 stale generation、invalid input、overlapping writer、atomic rollback、non-uniform scale RED tests。

### ED199-M1：stable source identity、Hierarchy 与 Control schema

- 建立 versioned `ControlRigSourceDocument`、migration envelope 和 Skeleton dependency。
- 建立 typed hierarchy/control/value/settings/initial-current/local-global 模型。
- 所有 ID 持久且跨 save/reopen/reimport 可验证。

### ED199-M2：Space、Constraint 与 transactional authoring

- 建立 space/multi-parent/constraint/maintain-offset/cycle 模型。
- 接入 Editor63/184 transaction、savepoint、undo/redo 与 revision CAS。
- space switch 产出 compensation plan，不直接写显示值。

### ED199-M3：Rig Unit graph 与唯一 compiler

- 在 Editor76/197 foundation 上建立 registry、typed pins/edges、external variable 和 unit upgrade。
- 定义 phase、entry DAG、read/write set 与 conflict validation。
- 禁止 Editor plugin 或 Runtime 再保留平行 compiler authority。

### ED199-M4：Compiled program、memory、LKG 与 cook

- 输出 immutable program、dense hierarchy/operand/memory、entry schedule、source map 与 capability。
- 建立 digest、dependency/currentness、atomic install 与 explicit stale LKG。
- preview/PIE/cook 使用同一 artifact。

### ED199-M5：Atomic Rig runtime 与基础 kernel 收敛

- 把 TwoBone/LookAt 变成 compiled units，补 orientation/space/transform policy。
- 建立 input snapshot、prepared pose/scratch、phase output page、writer conflict 与 atomic publish。
- 成功/失败均发布 generation-qualified receipt；frame 内零同步 asset I/O。

### ED199-M6：diagnostics、debug 与 runtime-backed preview

- 建立 source-qualified diagnostic/execution receipt 与 stale reject。
- Validate/Preview 执行真实 validator/artifact，删除固定 feedback。
- 建立最小 watch/trace/phase timing 与 frozen-input replay。

### ED199-M7：Viewport control 与 direct manipulation

- 发布 shape/pick proxy，接入 qualified selection/gizmo/input capture。
- interaction preview 与 commit/cancel transaction 闭环。
- 多 control、space、limit、hot reload 恢复通过动态测试。

### ED199-M8：Backwards Solve 与 Bake

- 建立 inverse entry、bake plan/range/rate/mask/tolerance/cancel。
- 通过 Editor77/198 canonical Sequence source 执行 atomic edit/key reduction。
- Control/space/constraint/weight channel 具有 stable ID 与 receipt。

### ED199-M9：规模、fault、soak 与跨引擎资格

- 10K element/数千 unit virtualization、incremental compile、batch solve、memory budget。
- plugin revoke、asset reload、compile failure、cancel/rollback、session close与 1 小时 soak。
- 同语义、同精度、同硬件 profile；无完整 correctness/receipt 不得宣称性能超过 Unreal。

## 9. 48 个资格门

| Gate | 资格 | 当前 |
|---|---|---|
| ED199-G01 | versioned ControlRig source roundtrip byte/semantic 稳定 | Fail |
| ED199-G02 | schema migration 与 unknown field/version fail-close | Fail |
| ED199-G03 | asset/document/element/node stable ID 跨 save/reopen 稳定 | Fail |
| ED199-G04 | Skeleton dependency 与 source revision CAS 明确 | Fail |
| ED199-G05 | Bone/Control/Null/Curve/Connector typed hierarchy | Fail |
| ED199-G06 | initial/current、local/global transform/value 语义完整 | Fail |
| ED199-G07 | parent/multi-parent topology 与 cycle diagnostic | Fail |
| ED199-G08 | typed Control value/settings/limits/shape schema | Fail |
| ED199-G09 | Space identity、maintain-offset 与补偿切换 | Fail |
| ED199-G10 | Constraint identity、weight、order 与 cycle policy | Fail |
| ED199-G11 | Rig Unit registry 支持 plugin generation/revoke | Fail |
| ED199-G12 | typed pin/default/link/external variable validation | Fail |
| ED199-G13 | node/function/unit version upgrade deterministic | Fail |
| ED199-G14 | Construction/Forward/Backward/Interaction phase 明确 | Fail |
| ED199-G15 | phase DAG/read-write conflict compile 期拒绝 | Fail |
| ED199-G16 | unique ControlRigCompiler，无第三 authority | Fail |
| ED199-G17 | immutable CompiledRigProgram 自包含且可 cook | Fail |
| ED199-G18 | dense hierarchy/operand/memory 布局有版本合同 | Fail |
| ED199-G19 | source debug map 与 artifact digest/currentness | Fail |
| ED199-G20 | compile 失败 LKG 明确标 stale | Fail |
| ED199-G21 | Preview/PIE/cook 执行同一 artifact | Fail |
| ED199-G22 | Rig instance/object binding 有 qualified lifecycle | Fail |
| ED199-G23 | dynamic input buffer 带 generation 与 space provider | Fail |
| ED199-G24 | TwoBone/LookAt 由 compiled unit 执行 | Fail |
| ED199-G25 | solve frame 内零同步 asset load | Fail |
| ED199-G26 | 每 rig 每 phase 复用 prepared pose/scratch | Fail |
| ED199-G27 | overlapping writer 结果确定或 compile 拒绝 | Fail |
| ED199-G28 | kernel/entry 失败不留下半写 pose | Partial |
| ED199-G29 | batch publish 为 atomic sealed generation | Fail |
| ED199-G30 | negative/non-uniform scale/shear policy 有 oracle | Fail |
| ED199-G31 | invalid chain/axis/space/constraint 不产生 NaN | Partial |
| ED199-G32 | diagnostic 可定位 document/entry/node/pin/element | Fail |
| ED199-G33 | diagnostic/receipt 携 source/artifact generation | Fail |
| ED199-G34 | stale preview/validate 结果被 Editor 拒绝 | Fail |
| ED199-G35 | Validate 运行真实三层 validator | Fail |
| ED199-G36 | Preview 产生真实 pose/frame/receipt | Fail |
| ED199-G37 | 静态 64 controls/18 constraints/1 warning 已删除 | Fail |
| ED199-G38 | control shape/pick proxy 与 frame generation 一致 | Fail |
| ED199-G39 | hover/selection/multi-selection 跨 Hierarchy/Viewport 同步 | Fail |
| ED199-G40 | gizmo begin/update/cancel/commit 无 dirty 泄漏 | Fail |
| ED199-G41 | interaction phase obeys space/limits/constraints | Fail |
| ED199-G42 | backwards solve 有 roundtrip tolerance oracle | Fail |
| ED199-G43 | bake range/rate/mask/cancel/rollback 正确 | Fail |
| ED199-G44 | bake 只写 canonical Sequence/Control channels | Fail |
| ED199-G45 | reload/plugin revoke/session close 无悬挂 instance | Fail |
| ED199-G46 | 10K element/数千 unit projection/compile 有预算 | Fail |
| ED199-G47 | 1 小时 preview/manipulate/reload/bake soak 无增长 | Fail |
| ED199-G48 | 同语义同质量同硬件 benchmark/profile 可复现 | Fail |

## 10. 验证边界、状态与禁止项

### 10.1 本轮验证

- 已静态阅读旧 Editor78 全文、frontmatter 所列当前 Editor/Runtime/test 文件，并沿 ZUI -> binding -> route -> feedback、IK export -> caller -> pipeline -> pose publish 调用链复核。
- 已静态对照 Unreal ControlRig/RigHierarchy/RigVM/compiler/editor/bake、Godot skeleton modifier/IK/editor、Fyrox pose、Bevy graph、Unity Graphics deformation consumer。
- 已通过 production 搜索确认 Control Rig 领域类型只存在于 Editor 静态 surface；两个 IK Job 除自身测试外没有调用方；`AnimationRigRuntimeStatus` 没有 production producer。
- 未运行 Cargo、Editor、GUI/GPU、save/reopen、cook、fault/soak/profile 或跨引擎 benchmark。源码测试只作为覆盖意图，不声明本轮通过。
- 本轮只修改 review/index/coverage；工作树原有 production 变化保持不动。

### 10.2 状态与禁止项

- review：`current_source_refresh_complete`；implementation：`pending`；canonical owner：Editor78；本轮不新增跨报告 canonical finding 总数。
- P1：14 项，11 Open / 3 Partial / 0 Closed；P2：5 项，4 Open / 1 Partial / 0 Closed；Gate：46 Fail / 2 Partial / 0 Pass。
- 禁止把通用 Animation compiler、`AnimationRigRuntimeStatus`、三个 IK 单元测试、静态 action 白名单或固定 `queued` 文本描述成 Control Rig 能力。
- 禁止恢复无 source/artifact generation 的 command queue；禁止让 Preview、PIE、cook、bake 执行不同 schema/kernel；禁止 Editor 直接写 Skeleton/Scene node；禁止用工具预览脚本或 Unity Graphics consumer 快照补足 production 证据。
- 实施前必须重算本报告 source/reference fingerprint，复核 dirty 工作树中旧 IK 删除的最终归属，并从 ED199-M0 truth/RED guards 开始。

## 11. 最终判断

当前 Zircon 的 Control Rig 不是“已有 Runtime IK、Editor 尚需接线”，而是“静态 Editor surface 加两个没有生产调用方的数学 Job”。旧 per-world command admission、postprocess、diagnostic 与 integration test 已从当前磁盘移除；这消除了旧半写 pose 的具体实现，却也撤掉了唯一的运行时桥。共享 Animation compiler、pose pipeline 与 skeletal status 是可复用基础设施，但没有任何一项拥有 Control Rig source、phase、program 或 instance 语义。

下一步应先让产品诚实显示 unavailable/prototype，并建立 stable source/hierarchy/control/space/constraint 与唯一 compiler；随后以 immutable program、input snapshot、atomic solve transaction 和 generation receipt 恢复 Runtime，而不是先恢复临时 command queue。只有同一 artifact 真正贯通 Editor preview、PIE、cook、direct manipulation、backwards solve 与 Sequence bake，再完成规模/fault/profile 资格，Control Rig 才能进入工程级实现阶段。
