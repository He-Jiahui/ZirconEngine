---
title: Editor Scene Component Authoring、Type Catalog、Add/Remove、Enable/Disable、Dependency、Multiplicity、Ordering、Default、Reflection、Transaction、Plugin Lifecycle 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor185
review_date: 2026-08-27
baseline_head: 9a7c88349d22fb92c99e91f31d629a1644007ab7
related_code:
  - zircon_runtime/src/core/framework/scene/component_type_descriptor
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/world/dynamic_components/registration_tests.rs
  - zircon_runtime/src/scene/world/transaction.rs
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/plugin/extension_registry
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/workbench/snapshot/data
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_window_template_bindings.rs
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
  - zircon_runtime/src/scene/world/dynamic_components/registration_tests.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/64-editor-scene-component-authoring-type-catalog-add-remove-enable-disable-dependency-multiplicity-ordering-default-reflection-transaction-plugin-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/183-editor-inspector-property-grid-reflection-schema-multi-selection-edit-transaction-undo-prefab-override-customization-asset-reference-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99i-runtime-scene-ecs-entity-component-storage-archetype-query-access-change-detection-command-schedule-parallel-event-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/06/failure-2026-08-01-inspector-multi-selection-batch-mutation-missing.md
  - docs/plans/zircon_editor/editor/06/failure-2026-07-28-plugin-contribution-ticket-revoke-contract.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/64-editor-scene-component-authoring-type-catalog-add-remove-enable-disable-dependency-multiplicity-ordering-default-reflection-transaction-plugin-lifecycle-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/64-editor-scene-component-authoring-type-catalog-add-remove-enable-disable-dependency-multiplicity-ordering-default-reflection-transaction-plugin-lifecycle-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Scene Component Authoring 当前源码复核（Editor185）

## 1. 结论

Zircon当前已经有真实而且应保留的组件基础：Runtime能够注册动态组件类型、校验plugin owner与property schema、按owner冻结和撤销extension、为每个type维护schema generation、把动态JSON组件接入ECS presence与inspection invalidation，并通过DynamicScene的preflight/publication事务复制或恢复组件。14个第一方production provider文件仍声明18个组件类型和137个property；15个builtin reflection registration也确实存在。组件能力不是只有测试样例。

但当前产品仍没有工程级Scene Component Authoring。最直接的证据不是“按钮尚未完成”，而是按钮形成了误导性的成功链：ZUI声明`workbench.inspector.add_component`，生成绑定却发送`workbench.inspector.component.add`；后者被列入`WORKBENCH_PREVIEW_ACTION_IDS`，dispatch后记录`operation_id=None`、`transaction_id=None`、空`effects`、revision不变的成功事件，retained bridge再静默忽略。按钮只要存在selection就显示，不检查document、mode、target capability、provider generation或可添加候选。现有相关测试只验证外观、颜色和surface contract，没有验证增加组件、history、dirty、save或reopen。

Inspector也没有组件拓扑。snapshot只调用`dynamic_components_for_entity(primary)`，没有统一列出builtin/native/script/plugin组件；DTO没有instance key、multiplicity、storage/execution/display order、enabled state、admission、schema/provider generation或dependency状态。componentized retained surface继续只消费`plugin_components.first()`。`EditorCommand`只有Create/Delete/Update Node、PlayTransform与SetReflectedSceneField，没有Add/Remove/Enable/Disable/Reorder Component，也没有结构journal payload。

Runtime描述符仍只有`type_id/plugin_id/display_name/properties(name,value_type,editable)`；字符串`type_id`同时承担schema、storage和UI identity。`World`以`entity -> HashMap<type_id, JSON>`保存动态实例，隐式限定每type单实例，枚举时按type id排序；`set_dynamic_component`和`remove_dynamic_component`是单次直接primitive，而不是依赖闭包、默认构造、生命周期、rollback、receipt和generation复验构成的结构事务。这些实现适合作为底层局部机制，不能被包装成Unreal级authoring产品。

Editor185不新增canonical finding，Editor64继续唯一拥有15项P1与5项P2。本轮按当前源码重新分级为：**P1 11 Open / 4 Partial / 0 Closed；P2 5 Open / 0 Partial / 0 Closed；40项Gate为34 Fail / 6 Partial / 0 Pass**。Partial只表示可复用基础存在，不表示产品可用。

一项重要旧结论需要校正。Runtime111在2026-08-22登记的`RSR-P0-001`所述“descriptor先写live registry、反射失败后半注册”路径已不符合当前源码：`World::register_component_type`现在先完成descriptor、reflection和ECS descriptor preflight，再执行无recoverable step的publish；`component_type_registration_failure_is_atomic_and_retryable`逐项比较三套registry和两套schema generation，并验证同type可立即重试。Editor185将其记为**当前源码已闭合、动态验证待补**；Runtime111 owner文档仍需单独刷新，本文不替代其canonical ledger，也不把未运行的测试写成通过。

本轮只做静态review和重构计划，没有修改production、tests、Cargo、ABI或参考源码；没有运行Cargo、真实Editor、add/remove/save/reopen、plugin unload/reload、multi-selection、fault/soak/profile或同硬件跨引擎benchmark。Tooling按用户要求排除。本轮也没有查询、轮询、等待或实时跟踪协调器。当前证据不支持“表现或性能达到或超过Unreal”的声明。

## 2. 审查边界与currentness

### 2.1 冻结语料

fingerprint算法为：相对路径转小写并统一为`/`，追加NUL、文件原始bytes、NUL，再对排序后的文件集合计算SHA-256。它只冻结本轮读取的当前磁盘内容，不是ABI、artifact、动态验收或性能receipt。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 | fingerprint |
|---|---:|---|---|
| Zircon Runtime focused set | **15 / 3,957 / 3,663 / 147,529 / 6 / 1** | descriptor、registry、World mutation、DynamicScene transaction与owner revoke | `90a5bae60d6d0d1383e263e1a86a5725902aecab29e35f1447cb587e3d5f6765` |
| Zircon Editor product set | **11 / 4,892 / 4,573 / 194,992 / 17 / 0** | command、snapshot、pane、retained route、preview action与ZUI | `d44dfb0332622d7f84ace2b0ec1435d8b80cf33e3cbcdb8e2c5c83709d860011` |
| First-party component providers | **14 / 1,130 / 1,008 / 40,699 / 3 / 0** | 18个production descriptor、137个property | `f60bb3677d0a0b200cf9d39b59b2aac13634ba8af2609e647ba91b05ef41cba8` |
| Zircon focused tests | **10 / 4,948 / 4,613 / 183,421 / 85 / 1** | reflection、registry、serialization、unload、surface与registration atomicity | `9e0e8f9c4cac66c24dd2f502931e84a3129e0a9cecf6c8d6f888e2580385431a` |
| Unreal selected set | **8 / 6,935 / 5,922 / 262,509 / 0 / 0** | catalog、subobject admission、transaction、ownership与register | `ee84a78d4b36d93c37496d37fc7f90da7efdb60ce0a66ce723cbbdcd8a70634b` |
| Godot selected set | **4 / 10,818 / 8,968 / 361,525 / 0 / 0** | searchable creation、Undo、owner/name与missing type | `091e77f821e8c0c9877e7d0da3598fd1eb90522a5f98bc6cf62a7182ba6579e7` |
| Bevy selected set | **5 / 3,624 / 3,229 / 138,745 / 30 / 0** | required graph、constructor、hooks、bundle与reflect adapter | `c56b8ef6087607815f1e9c613205a1615f5cb4d3f00db707c849ffab06c3e2a0` |
| Fyrox selected set | **3 / 1,361 / 1,221 / 47,362 / 0 / 0** | constructor menu、reversible AddNode与Inspector registry | `97061ab670e40a8271368439ad0de61b6caecdaa059166d1e4847ead4ac8499e` |
| Unity Graphics selected set | **4 / 2,880 / 2,466 / 119,676 / 0 / 0** | Volume add/remove/order/active/reset/copy/paste/default与Undo | `6fa22f4c6fac7e043acfee8d87f82540e87dcb9cad5d9563cbafb82171faffa4` |

主仓与Unreal镜像冻结HEAD为`9a7c88349d22fb92c99e91f31d629a1644007ab7`。Godot、Fyrox、Bevy与Unity Graphics revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。

### 2.2 共享工作树规则

冻结时共享checkout有大量其他Session或用户的tracked/untracked修改，且本轮focused范围包含正在拆分的Runtime与Editor文件。本文以fingerprint对应的磁盘内容为事实，不归因、不覆盖、不回退这些修改。后续实现必须重取HEAD、active owner、route、descriptor和测试终态；本报告中的line number只能用于本轮证据定位，不能替代实施时重读。

静态验收期间HEAD从冻结值前进到`681588f7a1cbfaae3147e8b93e1be6705d810f21`（`docs(editor51): record managed caret blocker`）；四组Zircon focused fingerprint复算后与表中值逐字一致，所以本报告继续以`9a7c88349d22fb92c99e91f31d629a1644007ab7`和对应磁盘语料作为冻结点，不把无关文档提交误记为组件实现变化。

### 2.3 范围与owner去重

| 主题 | 唯一owner | Editor185职责 |
|---|---|---|
| Scene Component Authoring的15/5 findings与40门 | Editor64 | 当前源码状态、差异与重构顺序 |
| Inspector property/multi-selection/customization | Editor05 / Editor183 | 只引用字段编辑和primary-copy P0，不复制finding |
| Transaction/history/savepoint/journal | Editor63/02 / Editor184 | 复用统一history，不建立组件私有undo |
| Script class/component schema | Editor31 | 要求进入统一catalog，不复制脚本编译finding |
| Prefab/default/override/rebase | Editor44 | 消费default/override authority，不在dialog猜默认值 |
| Provider contribution/reload/quiesce | Editor50 | 消费ticket、lease和revoke，不复制plugin lifecycle finding |
| Copy/paste/duplicate/remap | Editor55 | 消费typed clone/remap，不以JSON字符串复制代替 |
| ECS/storage/query/lifecycle event | Runtime108 | 定义structural planner所需内核，不复制ECS finding |
| World/persistence/scene transaction | Runtime109 | 复用DynamicScene原子发布，不混同component mutation |
| protected/derived mutation | Runtime110 | structural planner必须服从domain authority |
| reflection/schema/address/inspection | Runtime111 | 本轮校正旧半注册源码结论，其余仍由Runtime owner负责 |

MVP F4仍被上游阶段阻断；review-only允许继续取证，但不授权提前实施高级组件产品。

## 3. 当前实现拓扑

### 3.1 类型与provider链

`ComponentTypeDescriptor`只有四类事实：字符串`type_id`、字符串`plugin_id`、`display_name`和property列表；property只有`name/value_type/editable`。`RuntimeExtensionRegistry::register_component_for_owner`能验证module owner与plugin id一致，`TypedExtensionPoint`能freeze，`revoke_owner_registrations`能按owner删除component贡献并先通知listener。这些是可靠的provider ownership基础。

但component描述符和extension table没有形成authoring catalog artifact：没有stable type key、schema version/digest、provider generation、redirect、category/search keyword、target constraints、default constructor、multiplicity、required/excluded graph、enable policy、order policy、lifecycle adapter、clone/remap/migrate能力或deprecation/recovery metadata。`ComponentTypeRegistry`只用`BTreeMap<String, Descriptor>`和per-type generation；`component_type_descriptors()`返回借用列表，没有immutable generation-qualified snapshot、delta、lease、old-generation admission关闭或batch receipt。

第一方production provider的18个descriptor证明内容规模已经超过样例，但它们仍只能声明137个属性，不能声明一个组件如何安全创建、依赖什么、是否可重复、如何启停、怎样复制或provider失效后如何恢复。

### 3.2 World结构mutation

`World::set_dynamic_component`先检查实体和schema，再直接写入每实体`HashMap<String, Value>`、插入ECS presence并推进generation；`remove_dynamic_component`直接删除map项和presence。返回值只有`bool`，没有plan、target set、precondition、dependency closure、factory output、provider lease、lifecycle phase、inverse、partial-decision或receipt。map key决定one-per-type；`dynamic_components_for_entity`clone JSON与descriptor并按component id排序，存储顺序被误当成展示顺序。

unknown reflection type和missing descriptor payload可保留在map中，Inspector会回退到JSON字段并设为不可编辑。这防止部分数据被直接丢弃，但没有original provider/version/digest、raw serialized bytes、迁移候选、定位实例、reinstall/replace/remove/export动作或隔离状态。`ensure_plugin_components_can_unload`和VM catalog unused检查会把所有active实例拼成一个无界字符串，不能作为10K/100K实例下的分页locate与quiesce产品。

### 3.3 已出现的原子基础

当前`register_component_type`先构造reflection registration、验证两套registry并preflight ECS descriptor import，然后publish；失败前不再修改live state。对应回归测试比较`ComponentTypeRegistry`、`TypeRegistry`、ECS `ComponentRegistry`、两套schema generation与dynamic component id，并在失败后立即重试。这足以纠正Runtime111旧的静态路径结论，但本轮没有执行测试或managed benchmark。

DynamicScene spawn也已建立`preflight_dynamic_scene_publication -> PreflightedDynamicScenePublication -> publish_preflighted_dynamic_scene`：descriptor、entity identity、component row、resource row和allocator先校验；publication阶段暂存lifecycle event，再发布registry、实体、dynamic JSON、resource与storage row。它是ED64-P1-06/P1-10的重要Partial基础，但适用对象是完整scene导入/复制，不包含对现有实体执行依赖感知的Add/Remove/Enable/Reorder，也没有Editor结构命令receipt。

### 3.4 Editor snapshot与命令

`inspector_plugin_components`只枚举primary node的dynamic component。reflection schema、customization匹配、field editor和missing-schema diagnostic是真实属性投影基础；但15个builtin reflection registration不会进入同一component topology，native/derived/protected状态也没有统一DTO。`InspectorPluginComponentSnapshot`只携component id、display/plugin/customization/diagnostic/properties，没有实例和结构能力。

`EditorCommand`的五类variant均为节点或字段值操作。字段命令能撤销一个已存在组件的字段，不等于能够撤销组件结构。全仓没有`SceneComponentAuthoringCatalogSnapshot`、`ComponentStructuralMutationPlanner`、`ComponentStructuralCommand`、`ComponentInstanceKey`、`ComponentDependencyGraph`、`ComponentDefaultFactory`、`ComponentLifecycleAdapter`或`AddComponentDialogModel`。

### 3.5 产品route的假成功

当前链如下：

```text
WorkbenchAddComponent visible for any selection
  -> ZUI route: workbench.inspector.add_component
  -> generated binding: workbench.inspector.component.add
  -> preview-action allowlist
  -> transient PressNode event
  -> operation_id = None
  -> transaction_id = None
  -> effects = []
  -> before_revision == after_revision
  -> success result
  -> retained bridge silently ignores action
```

这是必须优先硬切的临时实现：在真实catalog/session/command存在前，产品应明确Unavailable并给出typed reason，不能继续产生成功形状的无副作用记录。

## 4. Editor64 P1状态刷新

| ID | 状态 | 当前源码证据 | 必需重构 |
|---|---|---|---|
| ED64-P1-01 | **Open** | builtin reflection、Runtime extension与dynamic/VM registry仍是分离事实，Editor无统一snapshot | Runtime发布唯一`SceneComponentAuthoringCatalogSnapshot`，Editor只消费artifact |
| ED64-P1-02 | **Open** | Add按钮只按selection可见；没有target/mode/document/provider admission | 建立逐target `ComponentAdmissionDecision`与typed拒绝原因 |
| ED64-P1-03 | **Open** | 字符串`type_id`同时承担schema/storage/UI identity | 引入`ComponentTypeKey + schema version/digest + provider generation + redirect` |
| ED64-P1-04 | **Open** | `HashMap<type_id, Value>`强制one-per-type且没有instance identity | 引入Single/Multiple/Keyed/Subobject policy及stable `ComponentInstanceKey` |
| ED64-P1-05 | **Open** | dynamic按type id排序；builtin storage/execution/display/attachment次序未统一 | 分离四类order并定义稳定anchor/rank与持久化 |
| ED64-P1-06 | **Partial** | DynamicScene已有完整preflight/publication原子基础；现有实体component primitive仍直接写 | 抽出结构planner、final-state复验、all-or-none commit、inverse与receipt |
| ED64-P1-07 | **Open** | descriptor无default factory；Editor无法构造合法初值 | provider-qualified、可取消、可预算、可失败的`ComponentDefaultFactory` |
| ED64-P1-08 | **Open** | 无requires/excludes/conflict/required-by图 | Runtime拥有统一依赖图、cycle检测、auto-add与remove policy |
| ED64-P1-09 | **Open** | `ActiveSelf`是entity activation；没有component enable policy | descriptor声明Unsupported/RuntimeActive/EditorOnly/ProviderDefined及持久化语义 |
| ED64-P1-10 | **Partial** | ECS lifecycle event staging和owner revoke listener存在；无authoring lifecycle adapter/compensation receipt | `prepare/commit/rollback` adapter、lease、deadline、panic隔离与terminal receipt |
| ED64-P1-11 | **Open** | Inspector DTO只有component/property展示字段 | 发布instance/order/enabled/capability/schema/provider generation/topology revision |
| ED64-P1-12 | **Open** | snapshot只取primary；无common/missing/incompatible target集合 | 建立multi-target topology intersection/union与单事务策略 |
| ED64-P1-13 | **Open** | 无Remove/Reset/Copy/Paste/Duplicate/Move/Locate结构action registry | 建立typed context action、availability和per-action plan |
| ED64-P1-14 | **Partial** | missing schema的JSON仍被保留并只读显示；无恢复产品 | 增加provenance、raw export、reinstall/migrate/replace/remove与quarantine |
| ED64-P1-15 | **Partial** | per-type schema generation、inspection dirty与有界property rows存在；无catalog/topology delta和规模预算 | 增量catalog/topology publication、10K type/target预算、遥测与取消 |

统计：**11 Open / 4 Partial / 0 Closed**。P1-06、10、14、15只能证明底层可复用片段，不得计为Add Component产品完成。

## 5. Editor64 P2状态刷新

| ID | 状态 | 缺失内容 |
|---|---|---|
| ED64-P2-01 | **Open** | category、keyword、favorite、recent、deprecated、provider filter与search ranking |
| ED64-P2-02 | **Open** | localization、accessibility name/state、keyboard/focus、高对比与10K结果导航 |
| ED64-P2-03 | **Open** | catalog/admission/plan/commit/recovery诊断和support bundle |
| ED64-P2-04 | **Open** | 第三方provider conformance kit、fault fixture、compatibility matrix与certification receipt |
| ED64-P2-05 | **Open** | component set/preset、bulk template与可版本化批量应用 |

统计：**5 Open / 0 Partial / 0 Closed**。

## 6. 参考引擎裁决

### 6.1 Unreal

`FComponentTypeRegistry`同时维护loaded/unloaded Blueprint component class、common/basic/custom group，订阅Asset Registry变化并广播component list更新；`SComponentClassCombo`消费可搜索和可过滤列表。`USubobjectDataSubsystem`用typed handle和FailReason执行Add/Delete/Rename/Reparent/Copy/Paste/Duplicate，并在操作前验证context、class、ownership和inherited hierarchy；实现使用`FScopedTransaction`。`ComponentEditorUtils`负责把instance component接入owner Actor并register lifecycle。

Zircon需要吸收catalog subscription、target admission、stable instance identity、ownership/lifecycle与transaction，不照搬UObject/SCS/Blueprint继承模型。尤其不能用Editor私有字符串列表模拟Runtime catalog。

### 6.2 Godot

`CreateDialog`统一ClassDB与global script type，过滤virtual、不可实例化与未暴露类型，并提供search、favorite、recent和category。SceneTree mutation通过`EditorUndoRedoManager`登记do/undo、owner和selection；`Node`结构操作验证main thread、busy、cycle、name和owner。`MissingNode`保留original class/scene/property/signal信息。

Godot是Node authoring参考，不是ECS component内核；可吸收的是发现、创建前过滤、单事务和missing-provider数据保护。

### 6.3 Bevy

`RequiredComponents`区分direct/inherited关系，持有constructor并检查cycle/duplicate；`ComponentInfo`保存hooks、required/required-by与clone behavior；BundleInfo区分explicit/required components和InsertMode。`ReflectComponent/ReflectBundle`提供apply-or-insert/remove/take/entity remap，bundle take在缺少任一项时保持all-or-none。

Bevy没有本地Editor产品，不能证明Add Component UX；它证明依赖闭包、constructor、hook、clone policy和原子结构操作应由Runtime拥有。

### 6.4 Fyrox

Create menu来自`serialization_context.node_constructors`，按variant/group构建和排序，并受Edit mode控制；`AddNodeCommand`用generational Handle/Ticket在execute/revert/finalize间保留对象，Inspector消费共享`PropertyEditorDefinitionContainer`。

Fyrox同样是Node参考；Zircon应吸收constructor作为真实创建来源、mode capability与exact reversible state，而不是复制Node menu形状。

### 6.5 Unity Graphics

`VolumeComponentListEditor`在一个真实consumer里闭合Add/Remove、Move Top/Up/Down/Bottom、Reset、Copy/Paste和Toggle；`VolumeProfile`保存ordered list并限制重复type；`VolumeComponentEditor`编辑active与override并接Undo；`VolumeManager`按render pipeline过滤supported type并计算default state。

本地Graphics只覆盖Volume domain，不是Unity通用GameObject组件。它仍证明成熟组件consumer不能只有property rows；order、active、default、support filter、context action、Undo和persistent ownership必须同时闭合。

## 7. 目标架构与唯一权威

Runtime唯一权威建议收敛为：

```text
ComponentTypeKey + ComponentInstanceKey
  -> ComponentAuthoringDescriptor
  -> ComponentDependencyGraph
  -> ComponentDefaultFactory
  -> ComponentStructuralMutationPlanner
  -> ComponentLifecycleAdapter
  -> ComponentStructuralMutationReceipt
```

Editor消费层建议收敛为：

```text
SceneComponentAuthoringCatalogSnapshot
  + ComponentTopologySnapshot
  + qualified Selection/Document/World session
  -> AddComponentDialogModel / typed context actions
  -> ComponentStructuralCommand
  -> shared history + dirty + journal
  -> receipt-driven projection refresh
```

核心约束如下：

1. Runtime拥有类型事实、依赖、默认值、multiplicity、结构plan、lifecycle和最终复验；Editor不能复制这些规则。
2. Editor拥有document/selection/mode、用户决策、command/history和产品呈现；Runtime不能假定当前primary selection。
3. catalog、topology、command与receipt都绑定world/document/object/topology/catalog/provider generation。
4. 所有结构操作先plan再commit；partial apply必须由用户显式选择并返回per-target receipt。
5. builtin、native、dynamic、script和plugin组件进入同一catalog/topology，但storage/execution adapter可以不同。
6. unknown/missing provider数据必须保真、可定位、可导出和可恢复，不能fail-open为可编辑普通JSON。

## 8. 分层重构顺序

### M0：先撤销假产品语义

- 为preview no-op建立产品RED：点击不能返回成功且revision不变。
- 在真实session存在前把Add Component标为Unavailable并显示typed reason。
- 建立ZUI route、generated binding、operation registry的一致性测试。
- 保留Editor05/183的Inspector P0和Runtime110 protected mutation为上游阻断，不在本里绕过。

### M1：类型identity与Descriptor V2

- 定义`ComponentTypeKey`、schema version/digest、provider key/generation和redirect。
- 描述multiplicity、enable/order policy、target constraints、dependencies、default/lifecycle/clone/migrate能力。
- 为18个第一方provider和15个builtin registration提供显式adapter，不允许Editor硬编码补metadata。

### M2：原子catalog与provider lifecycle

- 建立batch `ComponentCatalogTransaction`和immutable snapshot/delta。
- registration/reload/revoke先preflight，旧generation只读且禁止新建实例。
- ticket/lease/quiesce/drain/revoke返回terminal receipt；10K type更新可取消且有预算。

### M3：Runtime结构planner

- 计算target admission、required closure、conflict、auto-add和remove policy。
- factory产出合法默认值，planner绑定所有generation和target topology。
- commit一次性修改storage、systems、lifecycle和invalidation；失败恢复全部状态。
- receipt包含exact inverse、per-target结果、changed topology和diagnostic。

### M4：Editor catalog/session/projection

- 建立qualified `ComponentAuthoringSession`和增量`ComponentTopologySnapshot`。
- builtin/dynamic/script/plugin组件统一展示，多实例有stable key。
- 10K type search、category/filter和大target selection使用有界增量模型。

### M5：命令与真实产品工作流

- 增加Add/Remove/Enable/Disable/Reorder结构命令和versioned journal payload。
- Add dialog、context action、multi-selection、partial decision、keyboard/accessibility全部消费同一plan。
- history/dirty/save/reopen只接受成功receipt；不从UI字符串推断世界事实。

### M6：Prefab、复制与provider恢复

- 接入Editor44 default/override/rebase和Editor55 clone/remap。
- provider失效后保留provenance/raw payload，支持reinstall/migrate/replace/remove/export。
- unload/reload使用分页locate、lease drain和old-generation拒绝矩阵。

### M7：资格与性能

- 覆盖10K types、10K targets、大dependency graph、cancel/deadline/fault与8小时soak。
- 记录catalog/topology delta、plan/commit latency、allocation、lock hold和memory growth。
- 只有同资产、同硬件、同操作、同质量和同统计方法的动态对比通过后，才允许声称达到或超过Unreal。

## 9. 40项资格门刷新

| Gate | 状态 | 当前裁决 |
|---|---|---|
| ED64-G01 | **Fail** | builtin/native/dynamic/script/plugin仍无统一generation-qualified catalog |
| ED64-G02 | **Fail** | type key、schema digest、provider generation与display metadata未分离 |
| ED64-G03 | **Partial** | 单type registration已原子且可重试；统一provider batch、old-generation admission仍无 |
| ED64-G04 | **Fail** | 无1/100/10K type搜索、过滤、分类、虚拟化与预算产品 |
| ED64-G05 | **Partial** | 有per-type schema generation；无catalog delta/artifact订阅 |
| ED64-G06 | **Fail** | selection/read-only/stale/PIE/capability/quiesce没有typed admission reason |
| ED64-G07 | **Fail** | 无multi-target present/missing/incompatible/locked解释 |
| ED64-G08 | **Fail** | 无schema/provider-qualified default factory和zero-mutation failure |
| ED64-G09 | **Fail** | 无required/exclude/conflict/cycle/duplicate/required-by图 |
| ED64-G10 | **Fail** | 无auto-add/remove policy、用户决策和commit复验 |
| ED64-G11 | **Fail** | Add/Remove/Enable/Disable/Reorder未进入planner/command/receipt |
| ED64-G12 | **Fail** | component结构操作不存在，无法证明任一失败回滚World/system/selection/dirty/history |
| ED64-G13 | **Fail** | 无绑定document/world/object/topology/catalog/provider generation的plan |
| ED64-G14 | **Fail** | 无结构Undo/Redo恢复instance/order/enabled/dependency/system/selection |
| ED64-G15 | **Fail** | 无component结构versioned journal和recovery拒绝语义 |
| ED64-G16 | **Fail** | 无Single/Multiple/Keyed/Subobject instance policy |
| ED64-G17 | **Fail** | 无跨删除、排序、Undo、reopen稳定的多实例key |
| ED64-G18 | **Fail** | 未区分storage/execution/attachment/display order |
| ED64-G19 | **Fail** | 无stable anchor/rank与stale/multi-user reorder处理 |
| ED64-G20 | **Fail** | 无component enable policy；`ActiveSelf`只是entity activation |
| ED64-G21 | **Fail** | 15个builtin与dynamic/plugin不在同一topology，protected/derived remove无产品能力 |
| ED64-G22 | **Fail** | componentized/legacy surface未共享同一identity/action，前者仍只取first |
| ED64-G23 | **Fail** | Add按钮进入success-shaped preview no-op，没有catalog/candidate/dialog |
| ED64-G24 | **Fail** | 无Remove/Reset/Copy/Paste/Duplicate/Move/Locate typed context action |
| ED64-G25 | **Fail** | 无component catalog的accessibility/keyboard/focus/localization/10K导航 |
| ED64-G26 | **Fail** | 无multi-selection add-missing/remove-common/mixed-enable/compatible transaction |
| ED64-G27 | **Fail** | 无explicit partial decision与per-target receipt |
| ED64-G28 | **Fail** | prefab/class default/override/rebase topology未接入 |
| ED64-G29 | **Partial** | DynamicScene有clone/remap与unknown payload基础；无component structural clone adapter/product |
| ED64-G30 | **Partial** | DynamicScene与detached entity路径可保留dynamic JSON；无instance/order/enabled结构roundtrip |
| ED64-G31 | **Fail** | owner revoke基础存在，但无component admission close、lease drain和terminal generation产品 |
| ED64-G32 | **Fail** | unload guard拼接全部实例字符串，无分页locate和有界诊断 |
| ED64-G33 | **Partial** | missing schema payload可保留只读；无provenance/export/reinstall/migrate/replace/remove |
| ED64-G34 | **Fail** | 无old catalog/customization/adapter generation执行拒绝 |
| ED64-G35 | **Fail** | 无provider failure下component结构commit、callback和DLL pin回滚证明 |
| ED64-G36 | **Fail** | 无10K type search/update/open动态预算receipt |
| ED64-G37 | **Fail** | 无10K targets/大dependency graph cancel/deadline/progress/lock预算 |
| ED64-G38 | **Partial** | schema generation、inspection dirty和virtual property rows存在；无topology delta和结构增量publication |
| ED64-G39 | **Fail** | 无reload/world replacement/Undo/save/fault/8小时soak的stale/memory/data-loss证据 |
| ED64-G40 | **Fail** | 无同资产/硬件/操作/质量/统计方法的跨引擎动态比较 |

统计：**34 Fail / 6 Partial / 0 Pass**。

## 10. 禁止的临时修补

1. 不在Editor新增第二份`Vec<ComponentTypeDescriptor>`或字符串allowlist。
2. 不把`set_dynamic_component`循环包装成Add Component事务。
3. 不用JSON空对象、property默认值猜测或`Default::default()`反射兜底代替provider factory。
4. 不把`ActiveSelf`、名为`enabled`的property或UI折叠状态冒充component enable。
5. 不用type id排序冒充display/execution/attachment order。
6. 不用primary selection结果复制到全部target冒充multi-selection。
7. 不把preview action、toast、成功event或按钮截图作为mutation完成证据。
8. 不在provider unload时丢弃unknown payload，也不让old adapter继续执行。
9. 不把DynamicScene完整spawn transaction直接宣称为现有实体component structural planner。
10. 不在没有动态同语义benchmark前宣称性能优于Unreal。

## 11. 当前裁决

- review状态：current-source refresh complete；implementation状态：pending。
- canonical owner仍为Editor64；本轮新增canonical finding为0。
- P1为11 Open/4 Partial/0 Closed；P2为5 Open/0 Partial/0 Closed；40门为34 Fail/6 Partial/0 Pass。
- Runtime111 `RSR-P0-001`的旧半注册源码路径已被当前preflight/publish实现和精确回归测试覆盖，记为源码闭合候选；owner ledger和动态验证仍待独立刷新。
- 首个实现切片固定为M0：撤销Add Component假成功并建立route/receipt RED；随后必须先完成Runtime identity/catalog/planner，不能先堆dialog样式或Editor私有registry。
- 本轮没有运行Cargo或动态产品验证，也没有追踪协调器；共享工作树中的其他改动均未被覆盖或回退。
