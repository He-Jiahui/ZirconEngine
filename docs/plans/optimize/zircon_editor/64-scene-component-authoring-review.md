---
title: Editor Scene Component Authoring、Type Catalog、Add/Remove、Enable/Disable、Dependency、Multiplicity、Ordering、Default、Reflection、Transaction、Plugin Lifecycle 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor64
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/core/framework/scene/component_type_descriptor
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/plugin/extension_registry
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_editor/src/core/editing
  - zircon_editor/src/ui/workbench/snapshot/data
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/inspector.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui
  - zircon_plugins
tests:
  - zircon_editor/src/tests/editing/editor_projection.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_editor/src/tests/host/binding_dispatch/inspector.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration/plugin_contributions.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/surface_contract/chrome_routes.rs
  - zircon_runtime/src/tests/plugin_extensions/dynamic_components.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_components.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/dynamic_components.rs
  - zircon_runtime/src/scene/tests/dynamic_scene/scene_patch_document.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/mvp/05-f4-basic-authoring.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/ComponentTypeRegistry.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/ComponentTypeRegistry.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/SComponentClassCombo.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/SComponentClassCombo.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SubobjectDataInterface/Public/SubobjectDataSubsystem.h
  - dev/UnrealEngine/Engine/Source/Editor/SubobjectDataInterface/Private/SubobjectDataSubsystem.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/Kismet2/ComponentEditorUtils.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Kismet2/ComponentEditorUtils.cpp
  - dev/godot/editor/gui/create_dialog.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/godot/scene/main/missing_node.cpp
  - dev/godot/scene/main/node.cpp
  - dev/bevy/crates/bevy_ecs/src/component/required.rs
  - dev/bevy/crates/bevy_ecs/src/component/info.rs
  - dev/bevy/crates/bevy_ecs/src/bundle/info.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/component.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/bundle.rs
  - dev/Fyrox/editor/src/menu/create.rs
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/Fyrox/editor/src/plugins/inspector/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentListEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeProfile.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/64-editor-scene-component-authoring-type-catalog-add-remove-enable-disable-dependency-multiplicity-ordering-default-reflection-transaction-plugin-lifecycle-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Component Authoring、Type Catalog、Add/Remove、Enable/Disable、Dependency、Multiplicity、Ordering、Default、Reflection、Transaction、Plugin Lifecycle 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon已经有一组应保留的组件基础：Runtime `World`能注册动态组件描述符、为实体设置或移除JSON组件、维护schema/world generation、建立ECS presence、使inspection artifact失效，并通过DynamicScene保存可反射组件；`RuntimeExtensionRegistry`能校验组件的plugin owner、冻结列表并按owner撤销；Editor能读取已存在的动态组件、解析reflection schema与customization，并用统一事务撤销字段值修改。14个第一方provider文件已经声明18种组件和137个property，说明组件不是只有测试中的Cloud Layer样例。

但当前产品没有工程级Scene Component Authoring。可见的`Add Component`只在ZUI中声明`workbench.inspector.add_component`，全仓production没有handler；只要存在selection，retained bridge就无条件显示按钮。`EditorIntent`和`EditorCommand`没有add/remove/enable/disable/reorder component操作。Inspector snapshot只枚举实体上已经存在的dynamic component，漏掉15种builtin reflected component；componentized retained surface又只显示`plugin_components.first()`。因此“有组件注册API”和“编辑器能改组件字段”不能证明用户能发现、兼容性检查、创建、移除、排序、禁用、撤销和保存组件结构。

更深层的问题是Runtime描述符只有`type_id/plugin_id/display_name/properties(name,value_type,editable)`。它没有stable schema identity/version、provider generation、category/search metadata、default factory、instance/multiplicity policy、requires/excludes、ordering、enable policy、lifecycle hook、clone/serialize/remap/migrate adapter或unknown retention policy。`World::set_dynamic_component`以`entity -> HashMap<type_id, JSON>`直接覆盖，隐式规定每type单实例且按type id展示；这可以作为局部运行时存储，却不能直接充当Unreal级authoring contract。

Editor05已经唯一登记可见假按钮和通用Inspector component topology缺口；Editor31、44、50、55、63与Runtime99i至99l分别拥有脚本组件规则、default/override、provider lifecycle、复制保存、transaction identity与ECS/reflection父合同。本报告不重复抬高这些问题，新增 **0项P0、15项P1、5项P2**。现存父P0仍阻断实施，尤其Runtime99l登记的component/type registration半提交风险，以及Runtime99k登记的protected/derived component mutation authority。

目标不是再加一个UI私有列表，而是建立Runtime唯一权威的`ComponentAuthoringDescriptor + ComponentTypeKey + ComponentInstanceKey + ComponentDependencyGraph + ComponentDefaultFactory + ComponentStructuralMutationPlanner + ComponentLifecycleAdapter + ComponentStructuralMutationReceipt`，再由Editor建立generation-qualified的`SceneComponentAuthoringCatalogSnapshot + ComponentAuthoringSession + ComponentTopologySnapshot + ComponentStructuralCommand + AddComponentDialogModel`。Editor负责selection/document/transaction和产品交互；Runtime负责类型事实、World preflight、结构提交、系统生命周期与最终复验。

本轮是review-only：未修改production Rust，未运行Cargo、真实Editor、add/remove/save/reopen、plugin unload/reload、multi-selection、fault/soak/profile或跨引擎同语义benchmark；tooling按用户要求排除。因此当前不能声称功能、表现或性能达到或超过Unreal。

## 2. 审查边界、currentness与冻结语料

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes / test attributes | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Zircon Runtime catalog/mutation | **18 / 4,400 / 162,431 / 11** | descriptor、schema generation、dynamic storage、reflection、plugin owner/revoke、DynamicScene | `867bc332a6f033a89a6d3cf697d3cdf88d1d36a3728a23f7687eb38050c36238` |
| Zircon Editor product | **9 / 2,627 / 106,022 / 3** | intent/command、snapshot、retained projection、pane payload、Add Component ZUI | `1918847ec85cca61fa6c70cc7c374fde1f3a8f31a78c3c4b81edfb5b009d972f` |
| First-party component providers | **14 / 1,130 / 40,699 / 3** | AI、Navigation、Particles、Sound、Prefab、Decal、VFX、Terrain与Tilemap的18 descriptor/137 property | `36acbfe270cafe524490eada5ad3b226564ffaa70508d0fc509e258f161dcdb9` |
| Zircon focused tests | **9 / 4,520 / 167,082 / 80** | reflection field、registry、serialization、unload guard及静态surface contract | `76a6070b3b9185d4ac6b586777b3c6acbf2f27ce33766955496f1bb89b63226a` |
| Unreal selected set | **8 / 6,935 / 262,509 / 0** | discoverable component catalog、subobject admission、transaction、ownership与register | `81ed6cb2d103e2b2ddf61d6d6d2707e2f536b0bee880dfcd06f3c42a792c717e` |
| Godot selected set | **4 / 10,818 / 361,525 / 0** | searchable type creation、undo/owner/name、missing type preservation与tree mutation guards | `06a1658a5ab08244c8a6767485930f452fe78a84d24e005f64217d0990e3a0cf` |
| Bevy selected set | **5 / 3,624 / 138,745 / 30** | required graph/constructor、hooks、bundle insert/take与reflect adapter | `f30bb45e946a9469a447144af6860d7b8e46610bbcd71e64fc6ec1026bff3038` |
| Fyrox selected set | **3 / 1,361 / 47,362 / 0** | constructor-driven Create menu、reversible AddNode command与Inspector editor registry | `2da6ed0074334de339fce5f37c337bb9f4214e9d3a7a2d433dafea8c71e20d7f` |
| Unity Graphics selected set | **4 / 2,880 / 119,676 / 0** | Volume component add/remove/order/active/reset/copy/paste/default与Undo | `6bcb6ba79f053fabe598009560bf30044411278a7d4a8763c65c1e6e651e348a` |

fingerprint按规范化相对路径和逐文件SHA-256基于本轮working-tree内容计算，只证明这组源码被读取；它不是ABI、artifact、动态测试或性能receipt。主仓与Unreal镜像基线为`bee4c707b714738346b49bba15c59468b8bd9b39`；Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。

### 2.2 在途修改隔离

共享checkout存在大量其他Session修改。本轮focused production set中，`component_property_rows.rs`处于非本轮dirty状态；索引文件也已有连续审查写入。该文件只用于证明现有property row处理字段Edit/Commit，没有用来证明结构变更算法。报告没有覆盖、回退或归因任何其他Session改动；实施前必须重取fingerprint、active owner和route终态。

coordinator Session为`optimize-editor64-scene-component-authoring-review-r1-20260822`，baseline epoch为339。本轮只租用本报告及三个共享索引。MVP F4仍被F0基线恢复阻断，所以没有用Cargo结果包装静态审查。

### 2.3 范围与非范围

本报告覆盖Scene实体上的builtin、dynamic、script/plugin组件如何进入统一catalog并完成结构authoring。UI template中的“component registry”属于Retained UI/template domain，不是Scene Component，不得合并命名或复用同一无类型ID。

以下父域只引用、不重复建账：Inspector字段编辑由Editor05拥有；script class/component schema由Editor31拥有；Prefab/default/override传播由Editor44拥有；provider mount/revoke/quiesce由Editor50拥有；copy/paste/delete无损性由Editor55拥有；history/object generation由Editor63拥有；Runtime ECS/World/hierarchy/reflection由Runtime99i至99l拥有。

## 3. 当前实现拓扑与可保留基础

### 3.1 Runtime已有三种不同层次，不能互相冒充

1. `ComponentRegistry`把Rust type或DynamicPlugin source映射到local `ComponentId`、storage type与table layout，服务ECS存储，不是用户可搜索的authoring catalog。
2. `ComponentTypeRegistry`按字符串type id保存`ComponentTypeDescriptor`并维护per-type schema generation，可为动态/VM字段writer失效提供基础，但描述符不足以决定能否添加实例。
3. `TypeRegistry`保存reflection field/schema/default与adapter，是属性访问权威的一部分，但没有component structural rules。

未来必须用稳定identity把三者关联并原子发布，而不是让Editor复制第四份注册表。Runtime99l已经拥有当前多权威和注册原子性问题，本报告只规定Editor消费的结构authoring快照。

### 3.2 Dynamic component mutation是底层primitive，不是authoring transaction

`World::set_dynamic_component`验证实体/type/value后，写入`HashMap<component_id, JSON>`、插入ECS presence、使inspection fields失效并推进component/world generation；`remove_dynamic_component`执行对应移除和失效。这些primitive可保留。

但接口只有“对一个实体设置/删除一个type字符串”。它不携document/world generation、instance key、expected topology revision、default constructor、dependency closure、conflict decision、provider generation、multi-target result、undo payload或terminal receipt。`HashMap`还把multiplicity固定为one-per-type，把presentation order固定为type id排序。Editor不能直接循环调用它们并宣称原子结构编辑。

### 3.3 Runtime descriptor没有authoring metadata

当前`ComponentTypeDescriptor`只有四个字段；property也只有name、value type和editable。第一方Navigation Agent等描述符虽能列出大量字段，却无法回答以下产品问题：

- 类型属于哪个category，如何搜索、显示icon/help/deprecation，在哪种World/NodeKind/capability中可用；
- 添加时如何创建完整默认值，是否允许多个实例，是否必须排序在某个组件之前；
- 依赖哪些required sibling，和哪些组件冲突，删除时cascade、block还是detach；
- 是否支持enable/disable，禁用后system/render/physics/replication如何处理；
- provider reload时如何迁移、clone、serialize、remap或保留unknown payload；
- lifecycle hook失败时如何rollback，哪些derived cache/system需要重建。

### 3.4 Editor当前只投影“已有动态组件的属性”

`inspector_plugin_components()`只调用`scene.dynamic_components_for_entity(node_id)`。Builtin reflection实际注册Name、Hierarchy、LocalTransform、ActiveSelf、ActiveInHierarchy、RenderLayerMask、Mobility、Camera、MeshRenderer、五类Light与RigidBody共15类，但没有进入统一component list。snapshot的每个plugin component只有type/display/plugin/customization/diagnostic/property，没有instance identity、enabled、order、remove capability、schema/provider generation或default provenance。

legacy pane payload会保留全部dynamic component；componentized retained surface却取`plugin_components.first()`。这意味着同一snapshot存在两个表现能力不同的consumer。Editor05拥有这项Inspector分裂，本报告要求未来ComponentTopologySnapshot成为唯一结构产品projection。

### 3.5 Add Component是可见死入口

ZUI声明`Inspector/AddComponent -> workbench.inspector.add_component`，production Rust没有该route字符串。retained bridge在无selection时隐藏，有任意selection时无条件显示；它不查询document writable、world mode、selection compatibility、plugin readiness或可添加类型数量。静态surface test只证明按钮style存在，不证明click可执行。

`EditorIntent`只有Create/Delete/Select/Rename/Parent/Transform/Apply/Undo/Redo，`EditorCommand`只有CreateNode/DeleteNode/UpdateNode/SetReflectedSceneField。80个focused test覆盖字段reflection、undo、registry、roundtrip和unload guard，但没有Scene component add/remove/enable/disable/reorder的行为测试。

### 3.6 Plugin owner能力是基础，但未闭合World和Editor产品

`RuntimeExtensionRegistry`能把component登记到owner slot，校验plugin id前缀，freeze并在`revoke_owner_registrations()`中按owner移除catalog条目。`World::ensure_plugin_components_can_unload()`则在存在live dynamic instance时拒绝卸载。这比无owner全局Vec更可靠，应保留。

然而撤销registry descriptor、World中已安装schema、live instance、Editor旧snapshot和customization callback没有一个共同provider generation/receipt。当前策略只有“阻止卸载”或“schema缺失时把JSON只读显示”，没有用户可操作的定位实例、导出原始数据、安装provider、迁移到替代类型、安全移除或重试reload工作流。跨family quiesce继续由Editor50/Runtime plugin报告拥有。

## 4. 父账current-source刷新：不重复计数

| Canonical owner | 本轮确认仍成立的父问题 | Editor64处理方式 |
|---|---|---|
| Editor05 `E-INSP-P1-27` | 可见Add Component无handler；remove/reorder/enable/disable不存在；builtin/dynamic Inspector分裂 | 不重复计数；Editor64定义catalog/plan/receipt及资格门 |
| Editor31 `P1-37..44` | Script Component stable identity、default/override、lifecycle、requires/excludes/multiplicity/ordering缺失 | script schema由Editor31实现；统一Runtime planner消费其规则 |
| Editor44 | component stable source identity、add/remove topology override、default/reset/propagation | 不把add-time factory替代Prefab/default authority |
| Editor50 | owner generation、cross-registry commit、quiesce、revoke与旧callback retirement | Editor64只定义catalog reader和orphan authoring recovery |
| Editor55 | component clone/serialize/remap、unknown type、Delete undo与structured transfer | ComponentContextAction调用其typed transfer contract |
| Editor63 | document/world/object generation、transaction scope、journal与async side effect | ComponentStructuralCommand必须绑定该session合同 |
| Runtime99i/99j/99k/99l | ECS storage、World identity/persistence、protected hierarchy/derived state、reflection/schema/inspection/registration | Runtime结构planner建立在这些修复之后，不另建并行World |

本轮没有发现需要从P1升级为新增P0且未被父账覆盖的独立路径。假按钮是明确的产品断路，但Editor05已按P1唯一登记；重复登记或改号只会破坏总账。实施M0必须先复现并引用父账P0/P1，不得用Editor64编号掩盖它们。

## 5. 本轮新增P1

### 5.1 Catalog、identity与admission

| ID | 当前差距 | 必须重构 |
|---|---|---|
| ED64-P1-01 | 没有统一`SceneComponentAuthoringCatalogSnapshot`；Runtime descriptor iterator、builtin reflection、plugin registry和Editor customization各自提供局部事实 | Runtime发布generation-qualified catalog artifact，统一type/schema/provider identity；Editor只做不可变projection |
| ED64-P1-02 | 没有target-specific admission；按钮只看是否有selection | `ComponentAdmissionQuery`输入document/world/selection/type/capability/mode，返回Allowed/Disabled/Hidden与typed reason、dependency preview和revision |
| ED64-P1-03 | `type_id`字符串同时承担schema、storage与UI identity，没有稳定version/fingerprint/redirect | 建立`ComponentTypeKey(package,type_guid)`、schema version/digest、provider generation与redirect/migration window；display name不参与identity |
| ED64-P1-04 | dynamic `HashMap<type_id, JSON>`隐式one-per-type且无instance identity | 每个可author组件有stable `ComponentInstanceKey`；descriptor显式声明single/multiple/keyed/subobject policy，Runtime复验 |
| ED64-P1-05 | dynamic组件按type id排序，builtin没有共同order，无法表达用户顺序或system-required order | 分离storage order、execution order与authoring display order；若顺序无语义则明确禁止拖动，若有语义则保存stable rank/anchor并可撤销 |

ED64-P1-01不是要求把所有ECS internals暴露给UI。catalog entry应是经过Runtime验证、面向authoring的不可变描述符，引用而非复制reflection schema，并带source generation、capability和availability evaluator。catalog publication必须all-or-nothing；Editor旧generation只能读，不能发起新mutation。

ED64-P1-02必须对多selection返回每个target的结果汇总，而不是用primary target冒充全体。对“部分可加”必须由产品policy明确选择Add to Missing、Only Compatible、Cancel或进入decision dialog，默认不能静默部分成功。

### 5.2 Structural plan、default、dependency与lifecycle

| ID | 当前差距 | 必须重构 |
|---|---|---|
| ED64-P1-06 | 没有`ComponentStructuralMutationPlan/Receipt`；Editor若直接循环World primitive会失去整体preflight与rollback证据 | planner冻结targets、expected revisions、add/remove/enable/order ops、dependency closure、defaults、cost和inverse，commit一次发布terminal receipt |
| ED64-P1-07 | descriptor没有default factory；`set_dynamic_component`要求caller提供JSON，第一方137个property没有add-time value authority | `ComponentDefaultFactory`按schema/provider generation构造typed value，记录default provenance；失败时零mutation |
| ED64-P1-08 | 通用Runtime没有required/excludes/conflicts/capability graph；Editor31规则没有产品consumer | `ComponentDependencyGraph`验证cycle、required-by、exclude、multiplicity和remove cascade；plan显式列出auto-added/blocked/decision节点，Runtime再次复验 |
| ED64-P1-09 | 没有统一per-component enable/disable；`ActiveSelf`是entity activation，`playing/active/enabled`只是各插件普通字段 | descriptor声明enable support与disable effect；Runtime以结构化state驱动schedule/render/physics/replication lifecycle，Editor显示mixed state并可撤销 |
| ED64-P1-10 | add/remove只触发通用generation/invalidation，没有provider lifecycle prepare/commit/rollback receipt | `ComponentLifecycleAdapter`提供validate/on_add/on_remove/on_enable/on_disable、derived invalidation和compensation，禁止callback在World锁内无界执行 |

ED64-P1-07只拥有“添加实例时如何得到合法初值”。Class Default、Prefab source、instance override、reset-to-default和传播仍由Editor44拥有。factory产物必须通过同一reflection/serialization validation，且factory code受provider generation、deadline、panic/fault boundary和output budget约束。

ED64-P1-08刷新Editor31 P1-44但新增的是全组件产品消费合同；Script、native、dynamic与future managed component都必须进入同一图。删除required component时policy必须是Block、Cascade或Replace，不能依赖调用顺序碰巧成功。Bevy证明required constructor、inherited graph、cycle/duplicate检查和required-by反向关系可成为Runtime结构合同，但Zircon无需复制Bevy API。

ED64-P1-09不能把所有组件强制成一个bool。descriptor可以声明Unsupported、RuntimeActive、EditorOnly、ProviderDefined等策略；关键是能力和后果显式、可查询、可保存和可复验，而不是Inspector猜某个名为`enabled`的property。

### 5.3 Editor session、product workflows与provider lifecycle

| ID | 当前差距 | 必须重构 |
|---|---|---|
| ED64-P1-11 | Inspector component DTO没有instance/order/enabled/capability/schema/provider generation，无法安全承载结构action | 建立`ComponentTopologySnapshot`，每entry携qualified address、generation、state、provenance、actions和diagnostic；旧snapshot提交被拒绝 |
| ED64-P1-12 | 没有multi-selection结构语义；现有批量测试只覆盖字段值 | session计算common/present-on-some/missing/incompatible/locked，Add to Missing、Remove Common、Enable/Disable和order均生成单一transaction plan |
| ED64-P1-13 | 没有per-component context workflow：remove/reset/copy/paste/duplicate/move/locate provider/docs/source均不可发现 | `ComponentContextActionRegistry`从Runtime capability和Editor owner组合typed action；所有mutation进同一command/decision/receipt，不允许UI直接改World |
| ED64-P1-14 | provider卸载只有World block或只读JSON diagnostic，没有orphan恢复产品 | orphan entry保留raw payload/schema provenance，提供Locate Instances、Export、Install/Reload Provider、Migrate、Replace、Remove with confirmation；旧provider generation不可执行 |
| ED64-P1-15 | 没有结构authoring性能预算、增量catalog或规模遥测；snapshot路径克隆JSON，catalog变化也无产品订阅 | 定义1/100/10K types、1/100/10K targets与大dependency graph预算；catalog/availability/topology使用generation delta、virtualized list、cancel和top-offender telemetry |

ED64-P1-11必须解决当前legacy pane保留全部组件而retained surface只显示first component的分裂。未来所有Inspector表现层读取同一snapshot；presentation可以不同，但component identity、action capability和generation不能不同。

ED64-P1-13中的Copy/Paste/Reset不在本报告重新实现。它们分别委派Editor55与Editor44，并返回同一`ComponentStructuralMutationPlan`可组合操作。Context menu不是字符串route集合，而是针对当前qualified component/selection计算出的typed capability snapshot。

ED64-P1-14依赖Editor50的provider quiesce和Runtime99l的schema preservation。目标不是允许缺失native代码继续执行，而是在严格不可执行状态下保护用户数据并提供可审计恢复选择。

## 6. 本轮新增P2

| ID | 增强项 | 进入条件 |
|---|---|---|
| ED64-P2-01 | Catalog支持category、keywords、favorites、recent、deprecated replacement、provider/source筛选和搜索评分 | P1 catalog identity/admission稳定，搜索结果不改变安全policy |
| ED64-P2-02 | 组件类型、禁用原因、dependency preview和context action完成本地化、键盘、screen reader与高对比资格 | typed label/help/diagnostic key存在，不从type id临时生成人类文本 |
| ED64-P2-03 | Component authoring diagnostics面板与support bundle导出catalog generation、plan/receipt、provider、fault、budget，不导出敏感property value | observation schema、redaction与retention policy通过 |
| ED64-P2-04 | 第三方provider conformance kit覆盖descriptor、default、dependency、lifecycle、clone/serialize/migrate、reload和fault injection | 核心adapter ABI/version/support window冻结 |
| ED64-P2-05 | 常用component set/preset与批量模板，可预览dependency/default diff后一键应用 | 单操作原子transaction、冲突decision和rollback已经通过P1资格 |

P2不能用来绕过P1。例如favorites不能缓存裸type字符串，preset不能绕过admission，support bundle不能序列化任意组件私有数据，conformance test通过也不能替代真实产品端到端测试。

## 7. 五类参考引擎裁决

### 7.1 Unreal：catalog、subobject identity、admission与事务产品

`FComponentTypeRegistry`维护loaded/unloaded Blueprint component class、basic shape/common/custom group，监听Asset Registry add/remove/rename并在catalog变化时广播；`SComponentClassCombo`消费可筛选列表。`USubobjectDataSubsystem::AddNewSubobject`返回typed handle与FailReason，并提供Delete/Rename/Reparent/Copy/Paste/Duplicate及CanCopy/CanPaste；实现会验证context/class/ownership/inherited hierarchy，再以`FScopedTransaction`提交。`ComponentEditorUtils`把instance component加入owner Actor并RegisterComponent。

裁决：Zircon至少需要可订阅catalog、stable instance handle、target admission reason、ownership/lifecycle和transaction。不能照搬UObject/SCS/Blueprint继承结构，也不能假定Unreal所有路径都无锁或无legacy；Zircon应把这些能力映射到Rust World和generation receipt。

### 7.2 Godot：可搜索创建、owner/name约束、missing type保护

Godot `CreateDialog`统一ClassDB与global script type，过滤virtual/不可实例化/未暴露类型，支持search、favorite、recent与category；SceneTree创建通过`EditorUndoRedoManager`登记add/remove/move/owner/selection，`Node::add_child/remove_child/reparent`验证main thread、busy、cycle/name/owner与keep-global-transform。`MissingNode`保留original class/scene/property/signal信息。

裁决：Godot是Node/object authoring参考，不是ECS component模型。Zircon应吸收可发现catalog、创建前过滤、single transaction和missing-provider数据保护，不应把组件重命名成Node或复制scene tree ownership语义。

### 7.3 Bevy：required graph、constructor、hooks与reflect structural adapter

Bevy `RequiredComponents`区分direct/inherited关系，按depth-first顺序保存constructor并检查cycle/duplicate/archetype状态；`ComponentInfo`保存hooks、required/required-by和clone behavior；BundleInfo区分explicit与required components及InsertMode。`ReflectComponent/ReflectBundle`提供apply-or-insert/remove/take和entity remap，bundle take在缺任一component时不移除任何项。

裁决：Bevy没有本地Editor产品，不能作为Add Component UX完成证据；但它证明required closure、constructor、hook、clone policy和all-or-none structural operation应属于Runtime，而不是由Editor按钮临时拼装。

### 7.4 Fyrox：constructor catalog与可逆对象命令

Fyrox Create menu来自`serialization_context.node_constructors`，按variant/group构建并排序；创建提交`CommandGroup(AddNodeCommand + MoveNodeCommand)`，非edit mode会禁用。`AddNodeCommand`用generational Handle/Ticket在execute/revert/finalize间保留或最终释放对象，Inspector使用共享`PropertyEditorDefinitionContainer`。

裁决：Fyrox同样是Node-based参考。Zircon应吸收constructor作为catalog真实创建来源、模式能力和exact reversible command state，不把Fyrox Node menu直接等同组件依赖图。

### 7.5 Unity Graphics：专用component consumer的完整工作流

Unity Graphics `VolumeComponentListEditor`在同一专用产品中提供Add、Remove、Move Top/Up/Down/Bottom、Reset、Copy/Paste、Toggle All/None，使用SerializedObject Update/Apply与Undo created/destroyed object；`VolumeProfile`保存ordered list并禁止重复type；`VolumeComponentEditor`编辑`active`和override；`VolumeManager`按render pipeline过滤supported type并计算default-constructed + global + quality + custom profile的default state。

裁决：本地Graphics只证明Volume domain，不是Unity通用GameObject component核心。它仍说明一个成熟组件consumer不能只有property rows：order、active、default、support filter、context actions、Undo和persistent ownership必须闭合。Zircon不应把Volume的one-per-type或default profile规则强加给所有组件。

## 8. 目标架构与唯一权威

```text
Runtime extension/type activation
  -> ComponentAuthoringDescriptor candidate
  -> schema/default/dependency/lifecycle/adapter validation
  -> ComponentAuthoringCatalogGeneration (immutable, all-or-nothing)
  -> World-bound ComponentAdmissionQuery

Editor DocumentTransactionSession + SelectionSnapshot
  -> SceneComponentAuthoringCatalogSnapshot
  -> AddComponentDialogModel / ComponentTopologySnapshot
  -> ComponentAuthoringIntent
  -> ComponentStructuralMutationPlanner (Runtime preflight)
  -> ComponentStructuralMutationPlan + inverse + decision set
  -> ComponentStructuralCommand (Editor transaction)
  -> World atomic structural commit
  -> ComponentStructuralMutationReceipt
  -> topology/inspection/dirty/savepoint/event publication
```

### 8.1 Runtime owner

Runtime唯一拥有：

- `ComponentTypeKey`、schema version/digest、provider generation与Runtime storage binding；
- `ComponentInstanceKey`和multiplicity/ordering/enable semantics；
- dependency/exclusion/required-by graph及cycle validation；
- default factory、reflection、clone/serialize/remap/migrate与lifecycle adapter；
- target admission复验、atomic World structural commit、inverse delta和receipt；
- component/system/cache invalidation、plugin unload guard与unknown payload安全状态。

### 8.2 Editor owner

Editor唯一拥有：

- document/world/selection-qualified `ComponentAuthoringSession`；
- catalog搜索、筛选、favorites/recent和Add dialog presentation；
- `ComponentTopologySnapshot`、multi-selection mixed state与decision UI；
- component structural intent、command/history/journal/dirty/savepoint路由；
- context action组合与provider recovery产品；
- stale snapshot、read-only document、PIE/runtime mode和permission policy。

Editor不得保存可独立变更的component schema registry，不得用UI order当execution order，不得从display name推导identity，也不得在route handler内逐实体直接调用`set_dynamic_component`。

### 8.3 关键数据合同

`ComponentAuthoringDescriptor`至少包含：type key、schema version/digest、provider owner/generation、display metadata、availability evaluator、instance policy、ordering policy、enable policy、dependency edges、default factory handle、reflection registration handle、lifecycle/clone/serialize/remap/migrate adapter handles与support window。

`ComponentStructuralMutationPlan`至少包含：document/world generation、selection revision、target object generations、expected topology revisions、catalog generation、provider generations、normalized operation DAG、constructed defaults、decision set、estimated cost、inverse strategy、affected systems/resources与expiry。plan只读且有deadline；commit若任一precondition变化必须整体Rejected/Stale。

`ComponentStructuralMutationReceipt`至少包含：operation ID、transaction/history ID、before/after world generation、per-target exact changes、auto-added/removed dependencies、lifecycle results、dirty/savepoint impact、diagnostics、timing/alloc budget与terminal status。Partial只能作为显式用户policy的per-target子receipt，不能表示内部半提交。

## 9. 依赖有序重构路线

| Milestone | Owner与工作 | 退出条件 |
|---|---|---|
| ED64-M0 | Editor05/Runtime99k/99l父账RED测试；假按钮止血 | 无handler时按钮不可达；注册半提交、protected component remove均有RED；不新增临时handler |
| ED64-M1 | Runtime identity/descriptor v2 | type/schema/provider/instance identity、version/digest/support window冻结；legacy descriptor只进显式migration reader |
| ED64-M2 | Runtime catalog原子发布与adapter | builtin/dynamic/script/plugin统一catalog generation；default/dependency/lifecycle/clone/serialize/migrate adapter验证all-or-nothing |
| ED64-M3 | Runtime structural planner/commit | add/remove/enable/order/multi-target preflight、inverse、dependency closure、expected revision和receipt通过fault injection |
| ED64-M4 | Editor catalog/session/projection | Add dialog、topology snapshot、builtin/dynamic统一列表、stale generation、read-only/mode/capability与virtualization闭合 |
| ED64-M5 | Editor command与产品workflow | add/remove/enable/order/context action/multi-select进入DocumentTransactionSession；Undo/Redo/dirty/savepoint统一 |
| ED64-M6 | Persistence、Prefab、plugin recovery | save/reopen、default/override、copy/paste、missing/reload/migrate/orphan数据保护按父owner闭合 |
| ED64-M7 | 规模、故障与竞争资格 | 10K types/targets、large graph、reload/soak/fault/profile与同硬件同语义跨引擎benchmark通过 |

M0不能提交一个只会`set_dynamic_component(entity, type, {})`的route作为“先工作起来”。M1/M2必须先关闭Runtime catalog identity和原子性，M3才允许结构commit；M4可以在Runtime能力未完成时显示明确Unavailable，但不能伪造成功。M6通过后才允许删除legacy reader，且删除必须是硬切迁移而非永久shim。

## 10. 产品级资格门

所有门当前均为 **Fail**；静态源码存在不等于通过。

### 10.1 Catalog与identity

1. **ED64-G01 Fail**：builtin、native Rust、dynamic、script/VM与plugin component全部进入同一generation-qualified catalog，无第二Editor registry。
2. **ED64-G02 Fail**：type key、schema version/digest、provider generation和display metadata分离，rename不改变identity。
3. **ED64-G03 Fail**：catalog批次任一descriptor/adapter失败时零项可见，旧generation仍可安全读取但不可新建实例。
4. **ED64-G04 Fail**：1/100/10K type下search/filter/category结果确定，列表virtualized且满足CPU/alloc/latency预算。
5. **ED64-G05 Fail**：catalog delta点名changed type/provider/generation，Editor不为无关变化全量克隆所有schema。

### 10.2 Admission、default与dependency

6. **ED64-G06 Fail**：无selection、read-only、closed/stale document、PIE/runtime-only mode、missing capability和quiescing provider均返回typed不可用原因。
7. **ED64-G07 Fail**：single与multi-target admission不使用primary target冒充全体，present/missing/incompatible/locked逐项可解释。
8. **ED64-G08 Fail**：default factory按schema/provider generation生成合法完整值，panic/timeout/invalid output为零mutation。
9. **ED64-G09 Fail**：direct/inherited required graph、required-by、exclude、cycle、duplicate和conflict deterministically验证。
10. **ED64-G10 Fail**：auto-add dependency、remove cascade/block/replace和用户decision均在plan预览中列出，Runtime commit再次复验。

### 10.3 Structural mutation与transaction

11. **ED64-G11 Fail**：add/remove/enable/disable/reorder均经同一planner/command/receipt，不存在route直接改World。
12. **ED64-G12 Fail**：任一target、dependency、lifecycle或storage步骤失败时World/component presence/system state/selection/dirty/history全部回滚。
13. **ED64-G13 Fail**：plan绑定document/world/object/topology/catalog/provider generation；任一变化都整体Stale并可重建。
14. **ED64-G14 Fail**：Undo/Redo恢复exact component value、instance key、order、enabled state、dependency side effect、system registration和selection。
15. **ED64-G15 Fail**：journal/recovery codec版本化；不可durable component默认拒绝并点名type/provider，不静默遗漏。

### 10.4 Multiplicity、ordering与enable

16. **ED64-G16 Fail**：single/multiple/keyed/subobject instance policy均有正反测试，duplicate single返回typed conflict。
17. **ED64-G17 Fail**：multiple instance拥有stable key，删除/排序/undo/reopen后identity不依赖数组index或display name。
18. **ED64-G18 Fail**：storage、execution、attachment与display order边界明确；无语义order不显示误导性拖拽。
19. **ED64-G19 Fail**：有语义order用stable anchor/rank，multi-user或stale reorder不产生静默错位。
20. **ED64-G20 Fail**：Unsupported/RuntimeActive/EditorOnly/ProviderDefined enable policy逐类验证；entity ActiveSelf不冒充component enable。

### 10.5 Inspector与产品交互

21. **ED64-G21 Fail**：15种builtin reflected component与全部dynamic/plugin实例在同一TopologySnapshot中可见，derived/protected项正确标为不可移除。
22. **ED64-G22 Fail**：componentized与legacy/alternate presentation消费同一identity/action capability，不再只显示first plugin component。
23. **ED64-G23 Fail**：Add Component按钮只在真实route、session、catalog和可用candidate存在时enabled；触发后打开可搜索dialog而非直接造空JSON。
24. **ED64-G24 Fail**：context action按component/selection capability生成；Remove/Reset/Copy/Paste/Duplicate/Move/Locate均有typed禁用原因。
25. **ED64-G25 Fail**：键盘、screen reader、focus return、localization、high contrast和10K result navigation通过真实Editor测试。

### 10.6 Multi-selection与父域组合

26. **ED64-G26 Fail**：Add to Missing、Remove Common、Enable/Disable mixed和Only Compatible均以明确policy生成一个transaction。
27. **ED64-G27 Fail**：部分兼容默认不静默部分成功；用户选择partial后receipt列出每target terminal state。
28. **ED64-G28 Fail**：Prefab/class default/instance override正确记录component add/remove/enable/order override并通过source rebase冲突测试。
29. **ED64-G29 Fail**：Copy/Paste/Duplicate通过Editor55 clone/serialize/remap policy，unknown component默认fail-closed或显式quarantine。
30. **ED64-G30 Fail**：Delete/Undo、save/reopen和DynamicScene roundtrip保留所有可author component topology、value、identity、order和enabled state。

### 10.7 Plugin lifecycle、unknown data与fault

31. **ED64-G31 Fail**：provider disable/reload先关闭new admission，再等待catalog/action/lifecycle callback lease归零，最后撤销并发布terminal generation。
32. **ED64-G32 Fail**：live instance阻断unload时Editor能定位全部实例且报告有界分页，不通过全World字符串拼接阻塞产品线程。
33. **ED64-G33 Fail**：missing provider payload保留type/schema/provider provenance和raw bytes/value，可导出、重装、迁移、替换或确认移除。
34. **ED64-G34 Fail**：old snapshot、old customization、old default/lifecycle adapter在reload后均不能执行；new generation可probe并独立fault quarantine。
35. **ED64-G35 Fail**：provider panic、timeout、invalid default、partial lifecycle和migration failure均无World半提交、callback泄漏或DLL pin遗留。

### 10.8 性能、规模与竞争资格

36. **ED64-G36 Fail**：10K types冷/热search、catalog update和Add dialog open达到声明的P50/P95/P99 CPU、wall、alloc、RSS预算。
37. **ED64-G37 Fail**：10K targets与large dependency graph plan支持cancel/deadline/progress且不持有World/UI长锁。
38. **ED64-G38 Fail**：component topology增量更新与viewport/render/physics/replication invalidation按changed set执行，无每帧全量JSON clone。
39. **ED64-G39 Fail**：plugin reload、document/world replacement、Undo/Redo、save/reopen、fault与8小时soak无stale command、内存增长或data loss。
40. **ED64-G40 Fail**：与Unreal及适用的Godot/Fyrox/Bevy/Unity Graphics路径在同资产、同硬件、同操作、同质量和统计方法下比较后，才允许宣称达到或超过。

## 11. 实施所有权与禁止事项

| Owner | 必须交付 | 不属于它的权威 |
|---|---|---|
| Runtime99i/99j/99k/99l实施链 | identity/storage/schema/catalog、World structural planner/commit、protected state、persistence primitive | Editor selection、dialog、history UX |
| Editor03/05实施链 | ComponentAuthoringSession、TopologySnapshot、intent/command、Inspector/Add dialog、multi-selection | Runtime schema/storage真相 |
| Editor06/12与Editor50 | provider desired/mounted generation、capability、quiesce/revoke/reload | component value/default或World commit |
| Editor44 | class/Prefab default、override、reset、propagation/rebase | add-time factory安全与Runtime dependency graph |
| Editor55 | clone/serialize/remap、copy/paste/delete conservation | Add catalog或provider lifecycle |
| Editor63 | DocumentTransactionSession、precondition、history/journal/savepoint | component dependency或default policy |

禁止以下临时修补：

1. 给`workbench.inspector.add_component`加一个直接写`json!({})`的handler。
2. 在Editor维护另一份`Vec<ComponentTypeDescriptor>`并定时和Runtime同步。
3. 用display name、当前数组index或裸type字符串作为持久instance identity。
4. 把任意名为`enabled/active/playing`的property自动解释为统一enable policy。
5. 逐selection循环mutation，失败后只弹toast而保留前半结果。
6. 遇到dependency时按临时顺序递归添加而不先验证cycle/conflict/cost。
7. plugin缺失时删除unknown JSON，或允许旧native callback继续编辑。
8. 用永久`pub use`、compat module、shim descriptor同时维持旧新authoring API。
9. 用源码字符串测试、按钮截图或单个Cloud Layer字段undo宣称产品完成。
10. 在没有同语义动态benchmark前宣称性能优于Unreal。

## 12. 当前裁决

当前Zircon Scene component系统具备真实的Runtime动态存取、reflection和plugin owner基础，也有第一方组件生产者；它不是空壳。但工程级authoring产品仍未开始闭环：没有可消费的统一catalog、没有目标admission、没有结构plan/receipt、没有instance/multiplicity/order/enable/default/dependency/lifecycle合同，也没有Add/Remove产品route与E2E conservation测试。

因此当前`Add Component`必须视为不可交付入口，而不是只缺一个callback的小功能。实施应先关闭Runtime父P0并建立descriptor/identity/catalog原子发布，再做Runtime结构planner，最后接Editor session/command/UI；顺序反过来只会把临时JSON mutation固化成新的长期债务。

本报告完成的是current-source差距建账，不是重构完成。15项P1、5项P2和40项资格门全部保持Open/Fail；下一轮实现前必须重取源码指纹、coordinator owner、父计划failure与F4基线状态。
