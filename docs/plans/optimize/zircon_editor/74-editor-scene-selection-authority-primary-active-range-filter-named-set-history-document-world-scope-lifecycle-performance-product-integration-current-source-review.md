---
title: Editor Scene Selection Authority、Primary-Active、Range、Filter、Named Set、History、Document-World Scope、Lifecycle、Performance 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor74
review_date: 2026-08-23
baseline_head: 7762880fd1d8db3d3872888ba8377910177574af
baseline_epoch: 342
related_code:
  - zircon_editor/src/scene/selection
  - zircon_editor/src/scene/modes
  - zircon_editor/src/scene/viewport/controller
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/editor_event
  - zircon_editor/src/core/editor_message/message/scene_inspection
  - zircon_editor/src/ui/binding/selection
  - zircon_editor/src/ui/binding_dispatch/selection
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/retained_host
  - zircon_editor/src/ui/workbench
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value.rs
tests:
  - zircon_editor/src/tests/editing/state/selection.rs
  - zircon_editor/src/tests/editing/state/viewport.rs
  - zircon_editor/src/ui/retained_host/app/tests/retained_host_automation.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/62-editor-inspector-property-grid-reflection-schema-multi-selection-edit-transaction-undo-prefab-override-customization-asset-reference-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/70-editor-scene-viewport-object-visibility-temporary-hide-isolate-local-view-selection-eligibility-hierarchy-feedback-persistence-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/73-editor-scene-viewport-region-selection-marquee-box-lasso-preview-query-performance-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/TypedElementRuntime/Public/Elements/Framework/TypedElementSelectionSet.h
  - dev/UnrealEngine/Engine/Source/Runtime/TypedElementRuntime/Public/Elements/Interfaces/TypedElementSelectionInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/TypedElementRuntime/Private/Elements/Framework/TypedElementSelectionSet.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Selection.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorActor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorSelectUtils.cpp
  - dev/godot/editor/editor_data.h
  - dev/godot/editor/editor_data.cpp
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/scene/commands/mod.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/Fyrox/editor/src/interaction/select_mode.rs
  - dev/Fyrox/editor/src/interaction/move_mode.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_picking/src/events.rs
  - dev/bevy/crates/bevy_picking/src/hover.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/ShaderGraph/Includes/SelectionPickingPass.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/ShaderLibrary/UniversalDOTSInstancing.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/Particles/ParticlesEditorPass.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/BRGPicking.shader
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Selection Authority、Primary-Active、Range、Filter、Named Set、History、Document-World Scope、Lifecycle、Performance 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon已经有一个可保留的最小多选模型：`SelectionModel`按Edit/Play保存两个`IndexSet<EntityId>`，每域都有有序items、primary和generation；Replace会去重并修复不在集合内的primary，Extend/Toggle/Clear会更新revision；Viewport highlight能够投影全部active items，Frame Selection也已经遍历全部选择。事务快照中的`SceneSelection`用`Arc<[NodeId]>`和`Arc<SelectionState>`共享payload，10,000项clone测试证明该局部路径不是无条件深拷贝。这些不是占位符，应作为重构输入保留。

但它还不是工程级Selection Authority。Scene、transaction、host publication和retained projection各自持有或重建不同形态的选择；production调用点可直接取得`&mut SelectionModel`，没有统一request、admission、policy、atomic commit和receipt。UI binding更把本来是`u64`的NodeId通过`as_u32()`解码，合法高位ID无法从Hierarchy/automation路径选择。重复选择同一节点时模型会正确返回unchanged，`apply_intent`却丢弃结果并固定返回`true`，随后仍发布scene inspection并触发Presentation/Reflection刷新。

产品能力同样停在最小点选：只有`SelectSceneNode`和Replace/Extend/Toggle，没有Subtract、Select All/None/Invert、按type/tag/component/property/material查询、range anchor消费、named set、previous/next selection、primary promotion、provider customization、subobject address、权限/插件owner、持久化profile或诊断receipt。Editor03/60已登记document/viewport/tool scope、range anchor、primary-only consumer和Outliner modifier丢失；Editor70拥有隐藏/锁定资格；Editor73拥有Region atomic mutation。因此本报告不重复这些finding，只定义Selection Authority必须提供的共享合同。

本轮没有发现新的P0。正式project/world replacement路径会显式清空`SelectionModel`，所以不能把“旧World同值ID会被选择操作命中新World”登记为当前可达错误；selection中仍可残留已删除ID，但现有snapshot/publication多会过滤它，当前证据更符合P1的一致性、生命周期和未来多文档风险，而不是已证明的数据破坏P0。

本报告新增 **0项P0、24项P1、10项P2与48个资格门**。目标不是继续给`SelectionModel`加方法，而是建立per-document/session的唯一`SceneSelectionAuthority`、qualified target address、typed request/plan/receipt、immutable snapshot/delta observer、selection query/named set/history和consumer projection，并把所有现有直写路径硬切到同一入口。当前状态为`review complete / implementation not started`；未运行Cargo、真实Editor、GUI/GPU、save/reopen、插件reload、fault/soak/profile或跨引擎benchmark，不能宣称性能与表现已经达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / test declarations | 本轮证据 | fingerprint |
|---|---:|---|---|
| Selection model/core | **13 / 1,282 / 1,123 / 40,227 / 7** | `SelectionModel`、domain/mutation、mode checkpoint、transaction selection、runtime ID与binding value | `e3c56ef8658b14af2e1ad2083f4bab900e7b20d27728e64085c5168542643f50` |
| Product routes/consumers | **21 / 3,657 / 3,378 / 137,431 / 17** | binding/event/dispatch、workbench、Play、viewport、render packet与publication invalidation | `8251ca8d2b656df7f596b96dacae83afbf82fc7708f4ca82e386611c8fbbc5a0` |
| Projection/focused tests | **11 / 2,889 / 2,635 / 98,145 / 40** | scene inspection delta、Hierarchy fragment/projection、selection/viewport/automation tests | `23f207d7826b3337cd6a7021e125869707b2bce81b09f92e3f98cc18b8d4bb97` |
| Zircon去重合计 | **45 / 7,828 / 7,136 / 275,803 / 64** | 三组按normalized path去重的current working tree | `998f3ae0683f6156ab56022f14e072b815f31a18fe9a24cb0eae030f70791991` |
| Unreal selected set | **9 / 15,817 / 13,431 / 602,175 / 0** | Typed Element selection、legacy USelection、select all/invert/matching与toolbar product | `af859cd262129493c8265ab6d631456213b8da6516087d3827698d5a958f180a` |
| Godot selected set | **4 / 11,814 / 10,049 / 433,342 / 0** | selection lifecycle/top roots、history、SceneTree与3D viewport policy | `c8945e80e0194168817a59a1fa56500a28888aaaa5141d2079fec22a209b4117` |
| Fyrox selected set | **5 / 3,031 / 2,722 / 101,588 / 0** | per-scene typed selection、command swap与interaction mode | `df17eaa1112d5b184a008f55dfc127fd634bf17620f88f958d086c9125707592` |
| Bevy selected set | **3 / 3,640 / 3,329 / 150,191 / 14** | generational entity identity、observer事件与picking state | `8ac1bd375ed2a35975c7468089e0aee65f53645c34bd38ad6f63b37c079ab2ac` |
| Unity Graphics selected set | **4 / 329 / 257 / 12,257 / 0** | renderer selection/picking pass、instance/submesh ID与alpha parity | `c71908a43ad928153c19027e82d9a279519cf932111b60a157b658cbb2b3c2c2` |
| 五引擎参考合计 | **25 / 34,631 / 29,788 / 1,299,553 / 14** | 五类本地参考按path去重 | `055c4ed34889b15576fbae3a119db43584d291afdb6d3ef34551def7ceb8ab11` |

fingerprint算法沿用Editor58-73：按normalized lowercase relative path排序，把`path + NUL + lowercase per-file SHA-256 + LF`串联后再取SHA-256。它只证明本轮读取的working-tree集合，不是ABI、cache key或验收receipt。冻结Git基线为`7762880fd1d8db3d3872888ba8377910177574af`，coordinator epoch为342；共享工作树存在其他Session改动，本报告不覆盖它们，实施前必须重算语料与父owner终态。

### 2.2 当前产品链

```text
Hierarchy click / automation binding
  -> SelectionCommand::SelectSceneNode { u64 }
  -> codec decode through as_u32()
  -> SelectionHostEvent::SelectSceneNode
  -> EditorIntent::SelectNode
  -> SelectionModel::select_only_active
  -> sync_selection_state + status
  -> event record changed=true + Presentation/Reflection effects

Viewport point/region
  -> controller select_nodes
  -> direct &mut SelectionModel
  -> Replace / Extend / per-item Toggle

Editor command transaction
  -> SelectionModel -> SceneSelection Arc snapshot
  -> CoreEditContext selection generation
  -> command apply/revert
  -> SceneSelection -> SelectionModel replace_active

Scene inspection
  -> filter current Scene IDs
  -> rebuild Arc<BTreeSet<EntityId>>
  -> BTreeSet difference + focused_entity side channel
  -> Hierarchy fragment/projection
```

### 2.3 已有基础必须保留

1. 保留`IndexSet`提供的去重、membership和稳定插入顺序；未来实现需显式定义顺序语义，而不是退回无序`HashSet`。
2. 保留per-domain items、primary、generation与全模型revision，但把domain提升为qualified session的一部分。
3. 保留Replace对重复项和无效primary的canonicalization逻辑，并将它上移为所有入口共享的不变量验证。
4. 保留transaction snapshot的`Arc`共享和selection-before/after能力，避免把hot path重新做成全量复制。
5. 保留scene inspection的delta/resync思路和unchanged 10K set复用测试，改为直接消费authority snapshot/delta。
6. 保留runtime highlight遍历全部active items的进展；renderer精确呈现继续由Editor59/Graphics owner实现。
7. 保留project replacement显式清Selection和Play enter/exit保存Edit选择的用户意图，但改成session lifecycle transition。
8. 保留selection mutation返回`bool changed`的底层能力，产品层必须忠实传播为typed disposition。

## 3. 父报告校正、唯一owner与不重复计数

| 既有owner | 当前源码重验 | 本报告裁决 |
|---|---|---|
| Editor03 P1-09 | Selection仍只有Edit/Play，不按document/viewport/tool分域 | 不重复计数；Editor74 authority消费Editor61/58/53提供的qualified session identity |
| Editor03 P1-10、Editor60 P1-14 | Shift仍只映射Extend，Hierarchy click不携modifier/range anchor | Outliner输入归Editor60；Editor74提供range-anchor state和range mutation API，不重复旧finding |
| Editor03 P1-11 | Highlight与Frame已改为多项，但gizmo、render anchor和多个consumer仍primary-only或各自解释 | 旧描述部分过时；具体consumer归03/59/62/66，Editor74拥有统一consumer projection合同 |
| Editor03 P2-02 | selection generation/revision仍`wrapping_add`且无epoch | 保持Editor03唯一计数；Editor74新schema必须绑定session epoch |
| Editor59 P1-03 / Editor70 | `select_nodes`仍只检查`scene.find_node`，没有effective visibility/lock/editability | eligibility继续由Editor70唯一实现；Selection Authority只执行其revision-qualified policy |
| Editor60 P1-15 | 裸NodeId、无per-document selection和stale prune仍成立 | 保持Editor60登记；Editor74承担central lifecycle/reconcile实现，不增加同义finding |
| Editor61 | project/world/document replacement和多文档生命周期已有总账 | `replace_world`当前会清Selection，因此本轮不虚构跨World P0；authority绑定其document session |
| Editor62 | Inspector多选mixed value、批量写入与virtualization已有owner | Editor74只提供ordered/full/primary projection，不复制Inspector editing finding |
| Editor63 | transaction namespace、qualified object/history与journal总账已有owner | Selection mutation是否进入Undo由产品政策决定；若进入则消费Editor63，不建第二history stack |
| Editor73 P1-06/P1-15/P1-17 | Region缺Subtract，mass Toggle逐项revision，commit无receipt | Region手势继续由Editor73计数；Editor74提供通用atomic mutation primitive和receipt |

## 4. P0：本轮没有新增

### 4.1 为什么不登记“旧World选择新World同值ID”

`EditorState::replace_world`先清history/context，替换world后显式把`SelectionModel`设为default；`clear_project`也清selection。虽然`NodeId`是world-local raw `u64`，`reset_from_scene`本身不prune selection，测试也证明两个不同world可以有相同`world_generation`，但正式replacement路径已消除当前selection。没有证据证明selection mutation可在该正式路径后保留旧ID并删除或修改新World对象，因此不能提升为P0。

### 4.2 仍需P1处理的风险

删除单个node、直接测试/插件 mutation或未来多document adapter仍可留下stale ID；scene inspection会过滤不存在节点，transaction binding却直接复制active items，consumer看到的集合可能不同。它是必须修复的authority/lifecycle问题，但当前证据没有达到“可达数据破坏或错误提交”的P0门槛。

## 5. P1：本轮新增的工程差距

### ED74-P1-01 · `SelectionModel`与`CoreEditContext::SceneSelection`形成双authority

Workbench在命令前把Viewport模型复制到transaction context，命令后再把snapshot写回Viewport；两边有独立generation和表示，sync还是fallible。再加上`selection_mut()`公开给construction、project、Play、viewport和workbench直接赋值，任何调用点都可绕过统一admission、observer和journal。必须只有一个per-session authority拥有状态，transaction只引用其immutable before/after snapshot或commit token。

### ED74-P1-02 · 没有typed request、plan、admission、terminal disposition与receipt

当前公共意图只有`SelectNode(NodeId)`，底层只有Replace/Extend/Toggle和bool；没有request/operation identity、source、policy revision、expected generation、accepted/rejected/no-op/stale disposition、before/after digest或correlation。调用者无法证明自己修改了哪一份选择、为何被拒绝、observer看到了哪个commit，也无法做可靠自动化与故障重放。

### ED74-P1-03 · UI binding错误收窄`u64` NodeId并拒绝合法高位值

`SelectionCommand`字段和`UiBindingValue::Unsigned`都是`u64`，codec decode却调用`as_u32()`再cast回`u64`。大于`u32::MAX`的NodeId会在binding admission失败；Runtime allocator又持续递增raw `u64`，没有任何全局32-bit上限。必须端到端无损解码opaque target address，并增加边界、最大值和跨codec round-trip测试。

### ED74-P1-04 · 模型的no-op被产品层伪报为changed并触发全链刷新

`select_only_active`已能对相同单选返回false，但`apply_intent(EditorIntent::SelectNode)`忽略返回值、固定设置status并返回`Ok(true)`；selection event又固定附带Presentation/Reflection effects。Dispatch因`changed=true`发布scene inspection，reflection层执行patch/invalidation。重复点击、重复automation或同步回放都会制造假operation、假日志和无效重建。typed receipt必须把Changed/Unchanged/Refocused/Rejected分开，只有真实diff或独立focus变化才发布对应effect。

### ED74-P1-05 · 两套selection表示没有共同canonical invariant

`DomainSelection::replace`会去重并保证primary属于items，`SceneSelection::new`却接受重复items、任意primary和任意顺序。当前内部调用通常来自canonical模型，但transaction、journal演进或插件入口一旦产生非法snapshot，写回模型时会静默变成另一状态。应由唯一constructor/validator生成canonical immutable snapshot，反序列化和provider输入返回typed validation error，不能依赖下一次sync修复。

### ED74-P1-06 · Primary、active item与selection order没有完整操作和规范

Replace默认取最后一项，Extend只在插入新项时更新primary，重复Extend现有项无法提升active；Toggle移除primary后取当前最后一项。没有`PromotePrimary`、`SetActiveWithoutMembershipChange`、stable anchor、top/bottom、focus-only revision或consumer-visible order contract。不同手势和provider将依赖遍历顺序决定gizmo/Inspector焦点，必须显式定义并测试。

### ED74-P1-07 · Consumer各自重算“可操作选择”，没有canonical projection

Delete在命令路径计算top-level roots；Inspector对全集合写同一primary表单值；gizmo和render anchor取primary；highlight取全集合；Frame取所有节点位置；Hierarchy丢掉order只保留set。Unreal Typed Element集中提供normalized selection处理group、parent-child、copy/delete/gizmo。Zircon需要由authority/provider给出`FullOrdered`、`Primary`、`TopLevelRoots`、`TransformRoots`、`CopyDeleteRoots`、`RenderableTargets`等typed projection及其source revision。

### ED74-P1-08 · Selection observer delta丢失scope、primary、order与原因

`SceneInspectionSelectionDelta`只有previous revision、new revision、added和removed；document/world/domain、primary before/after、ordered move、source、reason、operation、policy、rejection和target kind均不存在。`focused_entity`作为message旁路字段补primary，不能表达primary-only变化或证明与delta同一commit。需要单一`SelectionChangeEvent`和gap-resync协议，observer按capability订阅而不是猜测两个字段。

### ED74-P1-09 · Viewport只比较primary，secondary-only变化可能跳过workbench同步

`handle_viewport_input`在调用controller前后只保存并比较`active_primary()`；若Replace/Extend/Toggle改变secondary集合但primary保持不变，就不会执行`sync_selection_state()`。Highlight revision可能变化，Inspector/Hierarchy/status等workbench projection却可能停留旧集合。必须依据authority commit receipt/revision驱动consumer，而不是采样单一primary。

### ED74-P1-10 · Stale target被不同consumer以不同方式隐藏，形成split-brain selection

测试明确允许向SelectionModel写入不存在的`999999`，snapshot/scene inspection通过当前Scene过滤它但不prune source；transaction binding又直接复制全部active items。UI可能显示“未选择”，history仍携带stale target，revision也没有reconcile事件。实现上应在node removal/replacement/provider revoke时由authority原子prune/remap并发布原因；本条只计consumer一致性，裸ID与stale-prune父问题仍由Editor60登记。

### ED74-P1-11 · Select All、None、Invert和Select Matching没有产品入口

Zircon source对`SelectAll`、`DeselectAll`、`InvertSelection`、`SelectBy`均为零命中；菜单只有Delete Selected和Frame Selection。Unreal本地源码提供SelectNone、Select Invert、All of Same Class及matching static mesh/skeletal mesh/material。Zircon需要canonical operation和可解释eligible universe，不能让各pane自行全扫World。

### ED74-P1-12 · 没有selection query/filter compiler与currentness

按type、component、tag、name、property、material、layer、owner、prefab/instance state选择均不存在，也没有query AST、schema/provider revision、indexed plan、cancel、result cap或stale receipt。Outliner name filter不能冒充Selection Query；它归Editor60且只改变row visibility。Selection Query应消费共享scene index和Editor70 eligible snapshot，并以atomic result提交。

### ED74-P1-13 · 没有named selection set与静态/动态集合语义

仓内没有Selection Set、保存当前选择、Recall/Add/Remove Set或动态query-backed set。工程级产品需要区分静态target snapshot和动态query set，定义missing target、rename/remap、document复制、project/team/user scope、未知provider保留、schema migration和冲突。Named set不能只是把裸NodeId Vec写进settings。

### ED74-P1-14 · 没有selection navigation history

没有Previous/Next Selection、history ring、branch truncation、pin、reason过滤或失效清理。Godot的`EditorSelectionHistory`会裁剪forward branch、维护object/property path并清除失效对象，证明导航history与Undo history是不同产品。Zircon应保留轻量immutable snapshot/digest并按document session维护，是否把selection mutation纳入Undo仍由Editor63政策决定。

### ED74-P1-15 · 没有typed target provider、selection customization与alias resolution

当前所有目标都被当作Scene NodeId，无法让actor/component/subobject、editor-only helper、asset、descriptor或插件对象定义CanSelect/CanDeselect、实际selection element、parent/group normalization、display/presentation和replacement。Unreal通过typed element interface/customization registry集中这些策略。Zircon需要owner-qualified provider lease和stable target kind，插件卸载时可revoke并reconcile。

### ED74-P1-16 · Subobject、component、instance与render element地址不可表达

`EntityId/NodeId`不能区分组件、mesh instance、submesh/material slot、bone/socket、spline point、gizmo subhandle或provider-defined subobject。Unity Graphics本地pass按entity/submesh索引读取selection ID，Godot viewport也有gizmo/subgizmo路径。Selection Authority必须使用qualified target address，并为owner projection和subobject promotion定义政策；具体component authoring仍归Editor64。

### ED74-P1-17 · Mutation没有provenance、reason与policy snapshot

点击、框选、Hierarchy、automation、history restore、Play transition和未来插件都只留下结果集合。无法回答“谁改变了选择”“使用了哪组modifier/eligibility规则”“是user gesture、target removal还是document transition”。没有这些字段就无法做auditing、multi-user presence、telemetry、replay、accessibility feedback或防止stale policy commit。

### ED74-P1-18 · 插件、mode与automation没有capability/owner lease或mutation权限

`SceneModeCtx`直接暴露`&mut SelectionModel`，host automation最终也进入同一直接路径；没有owner identity、allowed target kinds、max result、document scope、read-only/Play policy、revocation或fault isolation。插件/mode可以写任意ID并绕过产品检查。应改成scoped `SelectionMutationCapability`，owner unload或session retirement后所有旧token fail-close。

### ED74-P1-19 · 产品command surface仍只有single-node Replace

Selection binding/event/retained callback/automation只识别`SelectSceneNode`，Hierarchy click不携modifier；没有command registry中的Clear/Invert/Promote/History/Named Set/Query，也没有disabled reason、count/status、keyboard和screen reader反馈。Editor08拥有通用command routing，Editor60拥有Outliner手势；Editor74必须提供可由menu、palette、keymap、automation和pane共同调用的canonical intent。

### ED74-P1-20 · Mode input隔离对每个mode深拷贝整份SelectionModel

`SceneModeCtx::checkpoint`包含完整`SelectionModel::clone()`；`SceneModeStack`在每个overlay和base mode处理每次input前都checkpoint，PassThrough再restore。`IndexSet` clone是O(selection size)复制，因此一次pointer move成本为O(active mode count * selected count)，与transaction `Arc`快照的局部优化相矛盾。checkpoint应保存authority revision/transaction token或copy-on-write handle，不复制集合。

### ED74-P1-21 · Scene inspection在每次变化时重建、排序和diff整份集合

Publication把current Scene selection收集为`BTreeSet`和`Arc<BTreeSet>`，变化时执行两个BTreeSet difference；full snapshot再materialize Vec。10K unchanged测试只证明相同revision复用Arc，不能覆盖频繁add/remove、mass select或million-item observer。Authority应直接发布persistent ordered snapshot和commit-time added/removed/primary delta，consumer不应重复O(N log N)归一化。

### ED74-P1-22 · Play transition按值clone/restore整个双域模型

进入Play先clone完整SelectionModel，再把Edit集合复制进Play，退出时整份赋回；它无法表达多个Play instance、remote world、simulated read-only selection或并行document，且大选择集造成同步复制峰值。Edit/Play分域缺document/viewport/tool是Editor03父finding；本条只登记transition的copy/lifecycle实现，目标是session registry引用immutable snapshots并显式retire。

### ED74-P1-23 · 没有规模预算、benchmark或过载策略

除10K Arc clone和unchanged publication复用外，没有100K/1M selection、high-churn Toggle/Replace、mode stack输入、observer fanout、query、named set recall、history、Play transition或memory benchmark。也没有max targets、deadline、allocation budget、chunking、cancel、truncate/defer/reject语义。工程目标必须用同负载receipt比较，不可凭数据结构名称宣称优于Unreal。

### ED74-P1-24 · 没有selection diagnostics、telemetry与可重放receipt

现有profile没有request latency、selected count、diff size、no-op rate、rejection reason、stale prune/remap、observer lag/gap、snapshot bytes、mode checkpoint copy、query cost或provider fault指标。日志也没有document/world/session/operation/target digest。必须输出默认脱敏、bounded且schema-versioned的receipt/trace，并允许从snapshot + operations重放authority状态。

## 6. P2：测试、维护与产品完整性缺口

- **ED74-P2-01**：没有`u32::MAX + 1`、`i64::MAX + 1`、`u64::MAX` NodeId的binding/event/automation无损round-trip测试，现有测试没有暴露`as_u32()`对合法高位值的拒绝。
- **ED74-P2-02**：没有“重复选择同一节点不得changed、不得发布inspection、不得Presentation/Reflection invalidation”的negative regression。
- **ED74-P2-03**：没有duplicate items、primary不在items、primary-only change、order-only change和canonical snapshot反序列化性质测试。
- **ED74-P2-04**：没有node removal/replacement、world transition、provider revoke、Play enter/exit和stale snapshot晚到的lifecycle矩阵。
- **ED74-P2-05**：没有observer gap/resync、primary/order delta、slow subscriber、reentrant subscriber和partial delivery fault测试。
- **ED74-P2-06**：没有All/None/Invert/Select Matching、query、range、named set、history back/forward与persistence migration产品测试。
- **ED74-P2-07**：没有component/subobject/instance/provider alias、mixed target、plugin reload/panic/oversize和unknown provider preservation测试。
- **ED74-P2-08**：`SelectionDomain { Scene, Asset }`与`WorldDomain { Edit, Play }`都使用domain一词却代表target family和world mode两种维度，schema演进前必须改成不歧义的typed scope。
- **ED74-P2-09**：没有100K/1M规模、high-frequency input、mass diff、history/named set retention、Play transition和observer fanout benchmark/soak。
- **ED74-P2-10**：没有真实Editor/native input、keyboard、screen reader、save/reopen、多document/multi-view或同语义跨引擎资格receipt。

## 7. 五引擎参考结论

| 参考 | 可采用的工程事实 | 不应照搬/证据限制 |
|---|---|---|
| Unreal | `UTypedElementSelectionSet`强制mutation经selection interface；CanSelect/CanDeselect、batch Set/Clear、modifier policy、selection element resolution、normalization、top/bottom、pre/change notification、undo和element replacement/update；legacy产品还有Select None/Invert/All Same Class/Matching Material等操作 | UObject/global editor singleton、legacy actor/object/component多selection facade和历史API不适合直接移植；应采用typed contract与产品能力，不复制全局状态 |
| Godot | `EditorSelection`在node tree exit时prune，缓存top roots，deferred/coalesced通知；`EditorSelectionHistory`清理失效ObjectID并支持back/forward；3D viewport考虑scene ownership、group/lock与append | 仍是Node特化、中心Editor单例和大量调用点policy；不能作为多document/provider/性能终态 |
| Fyrox | 每个`EditorSceneEntry`拥有独立typed selection和command stack；Graph/UI/Navmesh等selection可type erase；`ChangeSelectionCommand`通过swap统一execute/revert，Select mode一次提交 | selection container clone和局部线性逻辑只证明Rust可以形成per-scene闭环，不证明million-item或插件安全 |
| Bevy | Entity index+generation防止普通recycle alias；picking event携target、pointer/location/event并通过Observer传播；hover状态复用allocation并保留previous state | 没有完整Editor Selection产品；只采用generational identity、observer和input provenance思想，不能宣称selection feature parity |
| Unity Graphics | SceneSelection/ScenePicking pass输出对象/selection ID；DOTS picking按entity/submesh索引；alpha-tested particle/material在selection/picking pass沿用轮廓规则；BRG fallback覆盖缺专用pass的shader | 本地`dev/Graphics`不包含UnityEditor Selection authority、named set/history/query UI，不能从渲染仓推断这些产品；只约束renderer target address和visual parity |

### 7.1 综合判断

成熟引擎的共同点不是一个更大的`HashSet`，而是：选择对象具有稳定类型与身份；mutation经过统一policy和batch；primary/order/normalized projection有明确语义；删除、替换、undo和session lifecycle会reconcile；产品提供选择全部、反选、匹配、历史与扩展入口；observer只消费同一commit的snapshot/delta。Zircon已有集合底座，但上述工程闭环多数缺失。

## 8. 目标架构

### 8.1 Qualified Session与Target Address

```rust
struct SelectionSessionKey {
    document: DocumentSessionId,
    world: WorldGeneration,
    view_scope: SelectionViewScope,
    tool_scope: SelectionToolScope,
    epoch: SelectionEpoch,
}

struct SelectionTargetAddress {
    provider: SelectionProviderId,
    owner: StableObjectAddress,
    kind: SelectionTargetKind,
    subobject: Option<StableSubobjectAddress>,
    generation: TargetGeneration,
}
```

`view_scope/tool_scope`是否共享由document policy决定，不能继续隐含为一个全局active set。Address必须跨binding/journal/observer无损，provider负责resolve/remap/display/normalization，authority负责session和membership。Runtime实体generation可作为地址的一部分，但不能把跨document身份下沉成一个裸EntityId。

### 8.2 唯一Selection Authority

每个active document/session只有一个`SceneSelectionAuthority`：内部保存immutable ordered snapshot、membership index、primary、range anchor、revision/epoch和bounded navigation history。外界只能提交`SelectionRequest`，authority在expected revision和policy snapshot上plan/admit，一次commit产生`SelectionMutationReceipt`。`SelectionModel`、`SceneSelection`和publication BTreeSet不得继续作为可写第二authority。

```rust
enum SelectionOperation {
    Replace(BoundedTargets),
    Add(BoundedTargets),
    Subtract(BoundedTargets),
    Toggle(BoundedTargets),
    Clear,
    PromotePrimary(SelectionTargetAddress),
    SelectUniverse(EligibleUniverseId),
    Invert(EligibleUniverseId),
    ApplyQuery(SelectionQueryId),
    RecallNamedSet(NamedSelectionSetId),
    Navigate(SelectionHistoryDirection),
}

struct SelectionMutationReceipt {
    operation: SelectionOperationId,
    session: SelectionSessionKey,
    disposition: SelectionDisposition,
    before: SelectionRevision,
    after: SelectionRevision,
    added: Arc<[SelectionTargetAddress]>,
    removed: Arc<[SelectionTargetAddress]>,
    primary_before: Option<SelectionTargetAddress>,
    primary_after: Option<SelectionTargetAddress>,
    reason: SelectionChangeReason,
}
```

### 8.3 Policy、Provider与Consumer Projection

Provider registry按owner lease注册target kinds、resolve/remap、CanSelect/CanDeselect、alias/group/parent-child normalization和presentation。Eligibility只消费Editor70 compiled snapshot；document/session currentness来自Editor61/58；命令权限来自Editor08/53。Authority提供revision-qualified projections：full ordered、primary、range anchor、top-level roots、transform roots、copy/delete roots、Inspector targets、render highlight targets。Consumer禁止自己重新解释裸NodeId集合。

### 8.4 Query、Named Set、Range与History

Selection Query使用typed AST和compiled plan，引用scene index/schema/provider/eligibility revisions，支持预算、取消和typed stale/degraded receipt。Named Set分静态address snapshot与动态query-backed set，具有user/project/team scope、schema migration、unknown provider preservation和missing target报告。Range anchor由authority保存，Outliner/viewport只提交ordered universe与endpoint。Navigation history保存COW snapshot/digest并在target失效时reconcile，不与Editor63 Undo stack混为一谈。

### 8.5 Observer、Lifecycle与Performance

Authority在commit时直接产生ordered delta；observer按session和capability订阅，gap后用immutable full snapshot resync。Node removal、replacement、provider revoke、world retire、Play transition和document close都提交typed lifecycle operation，原子prune/remap并说明reason。Mode checkpoint只保留revision/token；scene inspection直接复用snapshot/delta，不构造BTreeSet；large set使用COW chunk/structural sharing与独立membership index。所有query/mutation/publication有target、time、allocation和observer fanout预算。

## 9. 分层里程碑

### ED74-M0：Currentness、RED Guards与Hard-Cut Inventory

- 固化高位NodeId codec失败、no-op假changed、secondary-only sync遗漏和mode deep clone的RED证据。
- 枚举全部`selection_mut()`、SelectionModel/SceneSelection转换和consumer set重建点。
- 冻结父owner schema，禁止在旧模型上继续增加旁路命令。

### ED74-M1：Session、Address与Canonical Snapshot

- 建立SelectionSessionKey、SelectionTargetAddress、epoch/revision和canonical immutable snapshot。
- 定义primary/order/range-anchor与invalid target invariants。
- 端到端无损替换u64/u32 binding桥。

### ED74-M2：Authority、Request与Atomic Receipt

- 建立唯一authority、typed operation、plan/admission/disposition/receipt。
- Replace/Add/Subtract/Toggle/Clear/Promote均一次revision、一次observer event。
- 忠实传播NoOp/Refocused/Rejected/Stale，不产生假刷新。

### ED74-M3：Lifecycle、Provider与Policy

- 建立provider registry、owner lease、target resolve/remap/customization。
- 接node removal/replacement、world/document retire、Play transition和plugin revoke。
- 消费Editor70 eligibility和Editor61/58 currentness，不复制父authority。

### ED74-M4：Observer与Consumer Projection

- 建立immutable snapshot、ordered delta、primary/order/reason和gap-resync。
- 提供top roots、transform、copy/delete、Inspector、highlight等canonical projection。
- 删除scene inspection BTreeSet重建和primary-only采样同步。

### ED74-M5：Core Product Operations

- 接Select All/None/Invert、Promote Primary和Select Matching canonical commands。
- 接menu、palette、keymap、Hierarchy、viewport、automation与a11y反馈。
- 所有入口共享operation ID、admission和receipt。

### ED74-M6：Range、Query与Named Set

- 提供range anchor/order universe contract，Editor60负责Outliner交互。
- 建立query AST/compiler/index/currentness/budget与typed result。
- 建立static/dynamic named sets、scope、migration、missing target和unknown provider政策。

### ED74-M7：Selection Navigation与Transaction Integration

- 建立per-document bounded previous/next history、branch truncation和invalid cleanup。
- 按产品政策选择性接Editor63 Undo/journal，不建立私有第二stack。
- save/reopen、crash recovery和document duplicate使用schema-versioned state。

### ED74-M8：Scale、Backpressure与Diagnostics

- hard cut mode checkpoint deep clone和publication全量BTreeSet diff。
- 建立COW chunk/membership index、query/mutation cap、cancel/defer/reject和observer backpressure。
- 输出脱敏receipt、metrics、trace与deterministic replay corpus。

### ED74-M9：Product、Fault与跨引擎资格

- 完成native input、GUI、keyboard、screen reader、multi-document/view、Play和plugin reload矩阵。
- 完成100K/1M、high-churn、observer fanout、soak、save/reopen和fault injection。
- 与Unreal/Godot/Fyrox做同语义功能与性能比较；Unity Graphics只比较selection/picking render parity。

## 10. 资格门

| Gate | 要求 | 当前 |
|---|---|---|
| ED74-G01 | 每个active document/session只有一个可写Selection Authority | Fail |
| ED74-G02 | SelectionModel/SceneSelection不再形成双authority | Fail |
| ED74-G03 | production调用点不能取得裸`&mut SelectionModel` | Fail |
| ED74-G04 | session绑定document/world/view/tool/epoch identity | Fail |
| ED74-G05 | target address端到端无损且支持provider/kind/subobject/generation | Fail |
| ED74-G06 | u64边界值binding/event/journal round-trip通过 | Fail |
| ED74-G07 | snapshot对duplicate/primary/order执行统一canonical invariant | Fail |
| ED74-G08 | primary、active、order与range anchor语义文档化并稳定 | Fail |
| ED74-G09 | request携operation/source/policy/expected revision | Fail |
| ED74-G10 | admission返回typed accept/reject/stale/unsupported reason | Fail |
| ED74-G11 | Replace/Add/Subtract/Toggle/Clear/Promote全部typed | Fail |
| ED74-G12 | 每个batch只增加一次revision并只发布一次commit | Fail |
| ED74-G13 | no-op不产生changed、inspection或presentation/reflection刷新 | Fail |
| ED74-G14 | receipt包含before/after/diff/primary/reason/correlation | Fail |
| ED74-G15 | stale/late request按session epoch fail-close | Fail |
| ED74-G16 | provider registry拥有stable owner/type/generation/lease | Fail |
| ED74-G17 | provider定义resolve/remap/alias/group/customization | Fail |
| ED74-G18 | plugin revoke/panic/timeout/oversize被隔离和终态化 | Fail |
| ED74-G19 | eligibility只消费Editor70 revision-qualified snapshot | Fail |
| ED74-G20 | node removal/replacement会原子prune/remap并发布reason | Fail |
| ED74-G21 | world/document/Play transition显式retire或迁移selection session | Fail |
| ED74-G22 | observer event包含scope/primary/order/source/reason | Fail |
| ED74-G23 | observer gap可由同authority immutable snapshot resync | Fail |
| ED74-G24 | slow/reentrant/failing observer不会阻塞或破坏commit | Fail |
| ED74-G25 | secondary-only与primary-only变化都驱动正确consumer更新 | Fail |
| ED74-G26 | scene inspection不再过滤出不同于authority的隐式集合 | Fail |
| ED74-G27 | top roots/transform/copy-delete/Inspector/highlight projection统一 | Fail |
| ED74-G28 | parent-child/group/alias normalization只有一个owner | Fail |
| ED74-G29 | Select All/None/Invert使用明确eligible universe | Fail |
| ED74-G30 | Select Matching支持type/component/tag/property/material等typed query | Fail |
| ED74-G31 | query绑定schema/provider/index/eligibility revisions | Fail |
| ED74-G32 | query有deadline/cancel/result/memory cap和typed overload结果 | Fail |
| ED74-G33 | range mutation消费stable ordered universe和anchor | Fail |
| ED74-G34 | named set区分static targets与dynamic query | Fail |
| ED74-G35 | named set有scope/schema/migration/missing/unknown-provider政策 | Fail |
| ED74-G36 | previous/next history per-document、bounded且清理invalid target | Fail |
| ED74-G37 | history与Undo职责明确并只通过Editor63集成 | Fail |
| ED74-G38 | menu/palette/keymap/Hierarchy/viewport/automation共享canonical intent | Fail |
| ED74-G39 | command availability有disabled/rejection reason与count反馈 | Fail |
| ED74-G40 | keyboard和screen reader获得primary/count/change/terminal反馈 | Fail |
| ED74-G41 | mode checkpoint不随selection size深拷贝 | Fail |
| ED74-G42 | publication直接消费commit delta，不做全量BTreeSet重建 | Fail |
| ED74-G43 | snapshot/history/named set有COW与bounded retention预算 | Fail |
| ED74-G44 | telemetry覆盖latency/count/diff/no-op/reject/prune/observer/allocation | Fail |
| ED74-G45 | receipt可脱敏导出并确定性重放 | Fail |
| ED74-G46 | 100K/1M、high-churn、fanout、Play与multi-view benchmark通过 | Fail |
| ED74-G47 | removal/replacement/reload/fault/soak/save-reopen矩阵通过 | Fail |
| ED74-G48 | 同语义跨引擎功能、性能与renderer parity receipt可复现 | Fail |

## 11. 测试与动态证据矩阵

| 层级 | 必须新增的证据 |
|---|---|
| Model | canonical snapshot、order/primary/anchor、所有mutation、no-op、invalid input、property/fuzz |
| Identity | high u64、document/world/view/tool epoch、target generation、provider alias/remap、stale rejection |
| Lifecycle | node delete/replacement、world/project close/open、Play enter/exit、provider revoke、plugin reload |
| Observer | ordered delta、primary-only/order-only、gap/resync、slow/reentrant/failing subscriber、fanout |
| Projection | top roots、transform/copy-delete/Inspector/highlight、parent-child/group/subobject/mixed target |
| Product | All/None/Invert/Matching、range、query、named set、history、menu/palette/keymap/automation |
| Persistence | user/project/team scope、schema migration、unknown provider、missing target、save/reopen/recovery |
| Performance | 100K/1M set、mass diff、mode input、publication、query、history retention、Play transition |
| Fault | stale request、invalid provider、oversize result、deadline/cancel、partial delivery、OOM policy |
| UI/A11y | real Hierarchy/viewport/native input、keyboard、screen reader、status/count/rejection、multi-view |
| Comparative | Unreal typed/batch/normalized operations、Godot lifecycle/history、Fyrox per-scene command、Unity render ID parity |

当前没有执行上述动态矩阵。既有小集合unit test、10K Arc clone或unchanged publication reuse不能把任何Gate改成Pass。

## 12. Owner路由与禁止重复实现

| 责任 | Canonical owner | Editor74边界 |
|---|---|---|
| document/world lifecycle与multi-document | Editor61 | 绑定session/epoch并消费transition，不复制document registry |
| viewport/view/tool session与input capture | Editor58/53/59 | 接qualified source/capture，不拥有pointer state machine |
| Hierarchy range/modifier/filtered order | Editor60 | 提供anchor/range mutation contract，不实现Outliner UI |
| Inspector mixed/batch property editing | Editor62 | 提供target projection，不实现property transaction |
| transaction/Undo/journal | Editor63 | 选择性集成before/after receipt，不建第二history authority |
| visibility/hidden/lock/eligibility | Editor70 | 消费compiled eligibility snapshot，不复制predicate |
| Region gesture/query preview | Editor73 | 提供atomic mutation endpoint，不复制marquee/lasso |
| component/subobject authoring | Editor64及Runtime Scene owner | 表达target address/provider，不拥有component schema |
| renderer picking/highlight parity | Editor59与Runtime/Graphics owner | 输出render projection/selection ID，不拥有GPU pipeline |

禁止用以下临时方案关闭本报告：继续公开`selection_mut()`；给SelectionModel再加一组菜单专用方法；保留SelectionModel和SceneSelection双写；用`Vec<u64>`或URI字符串冒充qualified address；把high NodeId clamp、收窄或拒绝；由每个consumer过滤stale target；用primary比较判断整个selection是否变化；Toggle循环逐项发布；用Outliner filter冒充Selection Query；把named set保存成无scope裸ID列表；把navigation history塞入Undo stack；plugin/mode直接写集合；每个observer重建BTreeSet；用小cube和10K clone测试替代100K/1M、fault与真实产品资格。

## 13. 状态与产出记录

- 审查状态：`complete`，仅表示Editor74 current-source差距建账完成。
- 实现状态：`not_started`。
- 新增finding：`0 P0 / 24 P1 / 10 P2`。
- 资格门：`0 Pass / 48 Fail`。
- 建议首个实施点：ED74-M0，先用RED tests固定高位u64被错误拒绝、no-op假changed、secondary-only sync遗漏和mode checkpoint深拷贝，再进入M1 identity/canonical snapshot；不得先做named set UI。
- 实施前置：重取45份Zircon聚焦文件、Editor03/58/59/60/61/62/63/64/70/73和Runtime Scene/Graphics父owner终态，重算working-tree fingerprint。
- 验证声明：本轮未运行Cargo或任何动态产品验证，不能宣称功能、性能、表现、可访问性、插件安全或跨平台已经达到目标。

## 14. 最终判断

当前Zircon Selection是“多个调用点共享一个可变有序集合”，不是一个selection product。`IndexSet + primary + generation`和Arc transaction snapshot是值得保留的底座，但无损身份、单一authority、atomic receipt、lifecycle reconcile、normalized projection、advanced operations、query/named set/history、插件边界和规模预算全部缺位；UI错误拒绝高位u64、no-op假changed、primary-only同步和per-mode深拷贝还证明最小路径已有具体正确性与性能债务。

正确路线是先封死身份与双authority，再建立typed atomic mutation和observer；随后接provider/lifecycle/consumer projection，最后实现All/Invert/Matching、range/query/named set/history及规模资格。只有48个资格门全部通过，Selection才从“能点中和保存几个NodeId”提升为接近Unreal Typed Element/Godot lifecycle成熟度、并具备继续追求超越目标的工程级基础。
