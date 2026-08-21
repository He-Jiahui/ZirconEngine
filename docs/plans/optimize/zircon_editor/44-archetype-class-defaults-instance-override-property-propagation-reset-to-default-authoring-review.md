---
related_code:
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/scene/world/project_io/scene_asset.rs
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime_interface/src/reflect
  - zircon_editor/src/core/extension/inspector.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/workbench/snapshot/data/inspector_snapshot.rs
  - zircon_editor/src/ui/material_editor/projection.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_prefab_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_plugins/prefab_tools
  - zircon_plugins/editor_support/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/31-script-source-code-editor-build-compiler-hot-reload-debugger-visual-script-class-component-authoring-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/UObjectArchetype.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Internal/UObject/UObjectArchetypeHelper.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Tests/ClassDefaultObjectTest.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/InheritableComponentHandler.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/InheritableComponentHandler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/ComponentInstanceDataCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ComponentInstanceDataCache.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/PropertyEditorArchetypePolicy.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/PropertyEditorArchetypePolicy.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SResetToDefaultMenu.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/UserInterface/PropertyEditor/SResetToDefaultPropertyEditor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstancePropertyOverrideAsset.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelInstance/LevelInstancePropertyOverrideAsset.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LevelInstance/LevelInstancePropertyOverridePolicy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LevelInstance/LevelInstancePropertyOverridePolicy.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WorldPartition/WorldPartitionPropertyOverride.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BlueprintGraph/Private/K2Node_GetClassDefaults.cpp
  - dev/godot/scene/property_utils.h
  - dev/godot/scene/property_utils.cpp
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/editor/inspector/editor_inspector.h
  - dev/godot/editor/inspector/editor_inspector.cpp
  - dev/godot/editor/scene/packed_scene_editor_plugin.h
  - dev/godot/editor/scene/packed_scene_editor_plugin.cpp
  - dev/godot/tests/scene/test_packed_scene.cpp
  - dev/Fyrox/fyrox-core/src/variable.rs
  - dev/Fyrox/fyrox-core/src/reflect/inherit.rs
  - dev/Fyrox/fyrox-ui/src/inspector/editors/inherit.rs
  - dev/Fyrox/editor/src/scene/property.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/mod.rs
  - dev/Fyrox/fyrox-impl/src/resource/model/mod.rs
  - dev/bevy/crates/bevy_scene/src/scene_patch.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/bevy/crates/bevy_reflect/src/std_traits.rs
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeProfileEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeComponent.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeParameter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Volume/VolumeManager.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 44 · Archetype / Class Defaults / Instance Override / Property Propagation / Reset-to-Default Authoring 工程化差距

## 1. 结论

Zircon当前没有统一、工程级的对象默认值、class default、Prefab原型、实例覆盖、来源传播或Reset-to-Default产品。仓内存在三个名字相近但责任完全不同的碎片：reflection字段可携带一个可选`default_value`；Prefab DTO可保存字符串路径加无类型JSON的override列表；ECS的`archetype`目录按component signature组织table和row。最后一项是运行时存储布局，不是Unreal式对象archetype、Class Default Object或实例继承，不能作为本专题的完成证据。

现有Prefab链不是“功能偏少”，而是尚未形成可安全启用的产品。`PrefabAsset`内嵌`SceneAsset`并以`Vec<String>`表示exposed properties；`PrefabInstanceAsset`只记录source reference、local transform及`entity_path/property_path/serde_json::Value`三元组。它没有stable source object/component/property identity、source revision、typed before/base value、schema migration、topology override、orphan/conflict状态或传播回执。source entity重命名、reparent、component替换或字段迁移后，override可能静默失配、误命中或被丢弃。

更严重的是Scene活动持久化链已存在明确的数据损失：asset DTO和cache payload都携带`prefab_instance`，但`World::from_scene_asset`不读取它，`World::to_scene_asset`则固定写入`prefab_instance: None`。该问题已经由Editor 41拥有canonical修复责任，本报告把它登记为默认值/覆盖系统的硬前置阻断，不复制第二套修复owner。

`prefab_tools`的五个Editor operation只有descriptor/menu/toolkit/template/customization注册，没有operation factory或executor；三个被注册的`.zui`/`.toml`资源不存在；runtime importer明确返回“backend is not installed”。`apply_prefab_overrides`只做校验和去重，然后清空instance override；`revert_prefab_overrides`只清空vector；`break_prefab_instance`只返回transform和override DTO。这些函数没有写source Prefab、恢复live effective value或物化完整普通Scene subtree。若按名称直接接入UI，将从“不完整”升级为可触发的数据破坏。

主Editor另有一套230行静态Prefab Workbench，固定展示`PF_Chest`、`Chest_04`、18 children、6 overrides和2 warnings；Apply/Validate回调只写queued反馈。它与`prefab_tools`包没有数据provider或事务连接，是需要fail-closed的虚假成功面。两个override/revert SVG图标和Material Editor局部`default_value/override_value/is_overridden`投影也不能证明通用实例覆盖产品存在。

目标不是照搬UObject层级，而是建立清晰的authority：`DefaultValueAuthority`统一解析Native Schema、Script/Class Default、Prefab/Archetype Source、Derived Variant、Instance Override与Session/Runtime Transient各层；versioned Prefab source和stable object/property identity承载差量；propagation service对clean、overridden、conflict、orphan、missing和type-incompatible状态分类；Editor通过事务完成reset、apply-to-source、revert和break；cook只发布已解析runtime artifact及紧凑provenance，而不是每帧遍历authoring继承链。

## 2. 本轮证据边界

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 证据强度与结论 |
|---|---:|---|
| Zircon asset/persistence | 9 / 3,267 / 126,377 | E3逐Prefab DTO、Scene DTO/cache/World IO与focused asset tests；18个test attributes |
| Zircon reflect/default schema | 41 / 4,415 / 144,266 | E3逐Runtime与Interface reflection/default metadata；10个test attributes |
| Zircon Editor inspector/surfaces | 14 / 4,106 / 163,735 | E3逐Inspector DTO/command/snapshot、Material局部override、Prefab Workbench与SVG；15个test attributes |
| Zircon prefab plugin/support | 17 / 1,250 / 47,192 | E3逐完整package、registration/helper/importer/resources与support batch；12个test attributes |
| Zircon ECS archetype | 12 / 1,945 / 61,882 | E3逐signature/index/table/row/change tick，确认仅为ECS存储语义；10个test attributes |
| Unreal reference | 17 / 3,984 / 146,572 | E2/E3按CDO/archetype、component override/cache、reset policy及Level Instance diff职责路由 |
| Godot reference | 9 / 11,110 / 385,757 | E2/E3按native/script/scene default precedence、SceneState与Inspector revert路由 |
| Fyrox reference | 6 / 5,818 / 210,757 | E2/E3按inheritable modified bit、revert command与resource inheritance路由；37个test attributes |
| Bevy reference | 6 / 4,162 / 162,115 | E2/E3按ReflectDefault与resolved ScenePatch layered spawn路由；4个test attributes |
| Unity Graphics reference | 5 / 4,233 / 174,591 | E2/E3按Volume override state、default stack reset与Editor undo路由 |
| selected combined scope | 136 / 44,290 / 1,623,244 | 当前工作树fingerprint `075f20379dee50d7f366a8949e750ff9592889108ef8ccba1813d79078699614`；106个test attributes、0 ignored、0个在途文件 |

指纹按136个selected path排序，对每个文件取lowercase SHA-256，再以`forward/slash/path|hash`和LF连接、无末尾LF后取总SHA-256。它只证明本轮静态证据集合，不证明产品行为、性能或跨平台资格；实施前必须重算。

### 2.2 Prefab source与override schema

`zircon_runtime/src/asset/assets/authoring.rs`定义的结构揭示了当前authority上限：

1. `PrefabAsset`只有URI、name、内嵌Scene和字符串exposed property列表。
2. `PrefabInstanceAsset`只有asset reference、local transform和override vector。
3. override地址由`entity_path`与`property_path`两个字符串组成。
4. override值为任意`serde_json::Value`，没有反射type ID、codec、unit、constraint或schema generation。
5. 记录中没有source revision/digest、base value/hash、instance revision或expected-before。
6. 没有component/child add-remove、reparent/reorder、resource ownership或reference remap操作。
7. 没有override来源层、优先级、local/inherited状态、orphan原因或冲突分类。
8. `exposed_properties`没有参数ID、类型、默认值、display metadata、rename alias或migration。

这种结构可以作为早期序列化草图，不能直接成为长期资产格式。尤其字符串entity path同时编码identity和当前层级位置，任何rename/reparent都改变地址；JSON又使loader无法在应用前证明值与目标field兼容。

### 2.3 Scene roundtrip断裂

`SceneEntityAsset`与cache payload的双向转换都保留`prefab_instance`，说明DTO层原本希望无损携带实例信息。但活动World IO出现了断层：

1. `from_scene_asset`创建entity、transform、hierarchy和components时完全不消费`entity.prefab_instance`。
2. `to_scene_asset`构造每个entity DTO时固定写`prefab_instance: None`。
3. Scene World tests里的fixture均为`None`，没有一项覆盖non-None load-save-reopen。
4. asset tests只覆盖直接引用计数和overview，不会经过World roundtrip。
5. 即使未来cache payload继续无损，任何经过World的正常编辑保存仍会擦除source link、local transform和override。

这不是“尚未传播”的普通P1，而是已建模数据经过合法操作静默消失的P0。Editor 41负责Scene/Level Instance存储修复，本专题在该gate通过前不得开放任何默认值或Prefab编辑动作。

### 2.4 Prefab Tools的真实行为

Editor helper共92行，行为边界很窄：

1. `effective_prefab_overrides`用`BTreeMap<(entity_path, property_path), _>`去重，同键last-wins并按字符串排序。
2. validation只检查source available布尔值及两个路径非空，不解析source object、component或field。
3. `apply_prefab_overrides`返回去重结果后清空`instance.overrides`，没有加载、修改、保存或原子发布source Prefab。
4. `revert_prefab_overrides`只清空vector，没有解析当前parent effective value，也没有更新live scene。
5. `break_prefab_instance`只返回local transform和去重override，没有解析source subtree、复制components、修复references或移除link。
6. tests只断言这些DTO/vector语义；测试名中的apply、revert或bake不等同产品行为。

Editor plugin注册create/open/apply/revert/break五个descriptor以及menu、toolkit、creation template和Inspector customization，但`EditorAuthoringContributionBatch`没有operation factory/executor字段。三个注册资源`editor/authoring.zui`、`templates/default_prefab.toml`和`editor/prefab_instance.zui`物理不存在。Runtime plugin把`.prefab.toml`交给`DiagnosticOnlyAssetImporter`，稳定返回backend未安装；`overrides`反射property还标记为不可序列化。manifest的beta/Partial标签相对诚实，UI注册则没有fail-closed。

### 2.5 Inspector与默认值投影

reflection不是零基础，但还没有形成default resolution contract：

1. `ReflectFieldInfo`只有一个可选`default_value`，`TypeRegistry`只验证其类型。
2. metadata不记录default来自native、script、class、Prefab、variant还是instance parent。
3. generic `InspectorField`只保存ID、label、type name、字符串value和editable。
4. plugin property snapshot同样只保存label/value/value kind/editability/editor。
5. snapshot builder不投影`ReflectFieldInfo.default_value`、origin、local override、mixed state或reset capability。
6. `InspectorFieldChange`和`InspectorBindingBatch`没有target revision、expected-before、default source或override intent。
7. `SetReflectedSceneFieldCommand`保存before/after以支持本地undo，但apply不验证current仍等于before。
8. Material Editor的shader/material局部override是可借鉴的domain实现，不是通用object/class default service。

Editor 05拥有generic Inspector row、default显示和reset affordance；Editor 31拥有Script Class schema与typed instance override；本报告拥有跨domain的default layer resolver、Prefab/class source propagation、apply/revert/break及conflict语义。

### 2.6 静态Prefab Workbench与无消费者图标

Prefab Workbench ZUI包含27个node、19条route和固定的hierarchy/override/validation数据。feedback callback对Open、Apply、Validate及selection都返回固定`PF_Chest`、`Chest_04`、6 overrides、18 children和2 warnings。没有document/provider/backend、asset revision、transaction、job或receipt进入该链。

`override-property.svg`与`revert-override.svg`存在于Inspector icon目录，但production没有文件名引用。图标可以保留为未来资源，不能被计入功能完成度；静态Workbench则必须在真实provider存在前隐藏、标成明确fixture或返回unsupported，不能输出queued/success-like反馈。

### 2.7 ECS archetype命名隔离

`zircon_runtime/src/scene/ecs/archetype`的职责是把component signature映射到`ArchetypeId`，维护row-aligned tables、locator和change ticks。它解决query/storage locality，生命周期跟随World结构变化；它不拥有class default、source asset、继承链、实例override、reset或propagation。

保留该命名没有问题，但公共文档和新API必须明确区分：

- `EcsArchetype`：component layout/storage bucket。
- `DefaultSource`或`ObjectPrototype`：authoring default authority。
- `PrefabSource`：可版本化、可实例化的scene subtree source。
- `ClassDefault`：script/native class构造与字段默认层。

禁止让同名术语诱导实现者把ECS table当作对象原型数据库，或让default propagation依赖实体当前所在的ECS archetype ID。

## 3. 参考引擎差异

### 3.1 Unreal：authority、实例缓存与property policy分层

Unreal选集展示的关键不是API数量，而是职责分离：

1. CoreUObject区分CDO、普通object archetype和subobject template，并沿class/outer archetype关系解析。
2. `InheritableComponentHandler`单独拥有组件template override、移除记录、验证和fixup，而不是把所有差量塞进一个property JSON数组。
3. `ComponentInstanceDataCache`在construction/reinstancing前捕获实例数据，之后按明确阶段重新应用，避免重建时覆盖用户状态。
4. Property Editor通过archetype policy判断每个对象的default、差异与reset能力，并把reset menu/widget作为真实命令入口。
5. Level Instance property override使用diff serialization、actor GUID容器、instance到archetype object/subobject map、reset path、dirty notification和Editor policy。
6. Class Defaults节点读取class default，不把可变实例伪装成默认authority。

Zircon不需要复制UObject反射或Blueprint层级，但必须复制“默认值来源、实例暂存、diff policy、Editor reset、source publication”分属明确owner的工程原则。

### 3.2 Godot：明确的default precedence与SceneState

`property_utils.cpp`明确按native class default、topmost script exported default、scene instantiation/inheritance stack override解析属性来源；`get_node_states_stack`沿nested instance/inheritance和owner关系构建状态链。PackedScene/SceneState保存实例层级、owner及local-to-scene resource边界，Inspector再基于这些来源提供revert/default/pin行为。

这证明Reset-to-Default不是“取reflection里一个值”，而是对当前对象、当前scene inheritance和当前property解析immediate parent。Zircon应复制precedence和owner可观察性，不复制NodePath作为唯一长期identity。

### 3.3 Fyrox：modified bit与typed revert

Fyrox的`ReflectInheritableVariable`只有在当前值未被modified时才继承parent，并显式支持mark/reset modified。Inspector仅在对象有parent且字段已修改时展示“Revert To Parent”，再发出typed `InheritableAction::Revert`；resource resolve阶段会恢复original handles、继承property并重映射引用。

这是比Zircon字符串JSON更小却更完整的局部语义基线：值、是否本地修改、parent availability和revert action彼此一致。不过它仍不是Zircon大规模Prefab propagation、跨资产事务和partition产品的性能上限。

### 3.4 Bevy：typed layered spawn，不是Editor override产品

Bevy `ReflectDefault`为类型级构造/default reflection提供基础；`ScenePatch`先解析依赖，再spawn/apply，`ResolvedScene`缓存scene并允许local template按层覆盖，复制时使用copy-on-write语义。这适合参考typed template、dependency resolution和缓存边界。

选集没有通用Editor property provenance、apply-to-source、reset menu或Prefab conflict产品，因此不能把ScenePatch存在推断为本专题完成度。Zircon应借鉴其resolved artifact和缓存，不把runtime spawn patch直接作为authoring source格式。

### 3.5 Unity Graphics：domain override与增量reset性能基线

开源Graphics选集只能证明Volume domain：`VolumeParameter.overrideState`显式记录覆盖状态，Editor为每个property渲染override checkbox并通过Undo/SerializedObject批量修改；`VolumeManager`维护default component/profile baseline，baseline变化时标记stack reset，并只重置需要的parameter。

它说明override状态不能仅由“当前值是否等于default”临时猜测，也说明reset应有增量dirty策略。该选集不包含Unity闭源Prefab实现，本报告不据此推断其class/default或Prefab能力。

## 4. 差距清单

### 4.1 P0：立即封口

1. **E-DEFAULT-P0-01** 在Editor 41修复Scene World roundtrip前，任何含`prefab_instance`的Scene不得经过当前load-save链静默写成`None`；必须fail closed或提供逐字段无损preservation path，并以load-save-reopen验证。
2. **E-DEFAULT-P0-02** 在真实Prefab document/provider、revision、executor、transaction和receipt接通前，禁用或明确标记静态Prefab Workbench的19条route及固定Apply/Validate反馈，禁止继续呈现`PF_Chest`假产品状态。
3. **E-DEFAULT-P0-03** Prefab Tools不得在factory/backend/资源缺失时注册可执行operation、toolkit、template或customization；admission必须把缺失资源和diagnostic-only importer投影为Unavailable，而不是只发布descriptor。
4. **E-DEFAULT-P0-04** 隔离或重命名当前apply/revert/break helpers；在source原子写入、effective parent恢复、完整subtree物化、reference remap和rollback实现前，禁止让这些函数清空override或移除link后向用户报告成功。
5. **E-DEFAULT-P0-05** 禁止把`(entity_path, property_path, JSON)`确立为长期override authority；production admission前必须引入stable typed identity、schema/source revision、base evidence、conflict/orphan分类和migration，否则source演进会静默误用或损坏实例数据。

### 4.2 P1：工程化主线

#### 4.2.1 Default authority与解析层

1. **E-DEFAULT-P1-01** 建立独立`DefaultValueAuthority`服务，禁止Inspector、Prefab plugin、script和material各自猜测默认值。
2. **E-DEFAULT-P1-02** 定义有序default layer：Native Schema、Script/Class Default、Prefab Source、Derived/Variant、Instance Override、Session/Runtime Transient。
3. **E-DEFAULT-P1-03** 为每个default source定义稳定`DefaultSourceIdentity`，包含project/package/asset/type和generation资格。
4. **E-DEFAULT-P1-04** 每次source publication必须生成revision/digest，instance解析和写操作携带expected revision。
5. **E-DEFAULT-P1-05** 建立跨rename/reparent稳定的source object ID，path只作为display和diagnostic。
6. **E-DEFAULT-P1-06** component/subobject使用稳定instance-in-source ID，禁止仅靠type name或当前数组位置定位。
7. **E-DEFAULT-P1-07** property address由reflect type ID、field ID和必要container key组成，并支持rename alias。
8. **E-DEFAULT-P1-08** default/override schema显式版本化，catalog fingerprint参与load、reimport和cook admission。
9. **E-DEFAULT-P1-09** override value使用typed reflected payload和codec，load前验证类型、range、unit及resource kind。
10. **E-DEFAULT-P1-10** 提供`resolve_effective_value`与批量snapshot API，返回value、origin、parent、local state和diagnostic，并按generation缓存。

#### 4.2.2 Override数据模型

11. **E-DEFAULT-P1-11** 将override建模为版本化operation union，而不是只有property-value tuple。
12. **E-DEFAULT-P1-12** 区分ValueSet、ValueClear与RemoveLocalOverride，避免用null或复制default模拟reset。
13. **E-DEFAULT-P1-13** 支持ComponentAdd、ComponentRemove与component-template override，并保留stable identity。
14. **E-DEFAULT-P1-14** 支持ChildAdd、ChildRemove及其local ownership/provenance。
15. **E-DEFAULT-P1-15** 支持Reparent与Reorder，并定义source更新后的确定性合并规则。
16. **E-DEFAULT-P1-16** reference/resource override记录qualified target和ownership，应用后执行完整remap与validity检查。
17. **E-DEFAULT-P1-17** 把`exposed_properties`升级为typed parameter schema，包含ID、default、constraint、display、alias和migration。
18. **E-DEFAULT-P1-18** 持久化Inherited、LocalOverride、Conflict、Orphan、MissingSource、TypeIncompatible和Suppressed状态。
19. **E-DEFAULT-P1-19** 每项override记录origin layer、source revision、base hash及last resolution receipt。
20. **E-DEFAULT-P1-20** 建立schema/object/property migration registry和可审计migration receipt，禁止load时静默丢项。

#### 4.2.3 Source propagation与rebase

21. **E-DEFAULT-P1-21** 将Prefab/class default source接入Asset dependency graph，形成source-to-instance反向依赖。
22. **E-DEFAULT-P1-22** dependency index同时覆盖loaded、unloaded、partitioned和cooked consumer，不能只扫描当前World。
23. **E-DEFAULT-P1-23** source保存产生typed change set，包含object/component/property/topology变更和old/new revision。
24. **E-DEFAULT-P1-24** 对未本地修改的实例自动传播，并验证解析结果与source generation一致。
25. **E-DEFAULT-P1-25** 对已有local override的字段保留override，仅更新其parent/base evidence和状态。
26. **E-DEFAULT-P1-26** source与instance同时修改时执行base/source/instance三方rebase，输出可持久化conflict artifact。
27. **E-DEFAULT-P1-27** 大规模传播通过Editor Background Jobs运行，使用不可变input snapshot和generation-qualified publication。
28. **E-DEFAULT-P1-28** propagation job支持cancel、pause、retry、progress、partial failure清单和安全重入。
29. **E-DEFAULT-P1-29** object与operation应用顺序必须确定，跨线程、跨机器和重复运行得到同一artifact digest。
30. **E-DEFAULT-P1-30** 每次传播输出affected/updated/preserved/conflict/orphan/missing统计、durable receipt和Diagnostic Journal事件。

#### 4.2.4 Reset、Apply、Revert与Break事务

31. **E-DEFAULT-P1-31** Create Prefab From Selection先验证selection ownership、external reference、nested source cycle和save target，再原子创建source与instance replacement。
32. **E-DEFAULT-P1-32** Apply-to-Source必须预览typed change set、affected instances、source revision和潜在conflict。
33. **E-DEFAULT-P1-33** Apply-to-Source使用跨document单一事务，CAS source revision、原子保存、传播和receipt，任何阶段失败均回滚。
34. **E-DEFAULT-P1-34** 普通Reset移除当前local override并重新解析immediate parent，禁止把当前default复制成新的local value。
35. **E-DEFAULT-P1-35** reset menu可显式选择Class/Script、Prefab、Variant等目标层，并清楚展示将移除哪些中间override。
36. **E-DEFAULT-P1-36** Break Instance先解析并物化完整effective subtree、components、topology和resources，验证成功后才移除source link。
37. **E-DEFAULT-P1-37** break/create/apply统一使用stable object map修复内部、外部、soft与asset references。
38. **E-DEFAULT-P1-38** 所有动作具备preflight、staging、commit、rollback和不可变receipt，禁止部分source写入后清空instance metadata。
39. **E-DEFAULT-P1-39** 所有default/override动作进入Editor transaction/history/journal，undo/redo后dirty、selection和resolver cache一致。
40. **E-DEFAULT-P1-40** 多资产写操作接入save admission、source control/changelist、autosave/recovery和外部修改检测。

#### 4.2.5 Inspector与Prefab Editor产品

41. **E-DEFAULT-P1-41** 扩展Inspector snapshot，逐target投影effective value、origin、parent value、override state、source revision和reset capability。
42. **E-DEFAULT-P1-42** property row显示稳定的来源badge/breadcrumb，允许定位并打开source document。
43. **E-DEFAULT-P1-43** reset affordance只在真实可reset时出现，tooltip/menu说明immediate parent与预期结果。
44. **E-DEFAULT-P1-44** 提供per-property、per-component和selected-subtree的Apply/Revert/Reset命令，不以字符串route直接改数据。
45. **E-DEFAULT-P1-45** multi-selection按每个target解析default layer，禁止以第一个对象的default覆盖全部对象。
46. **E-DEFAULT-P1-46** mixed value、mixed origin、mixed override和mixed editability分别投影，不合并成一个含糊状态。
47. **E-DEFAULT-P1-47** Hierarchy/Outliner显示instance root、nested source、local addition/removal、conflict和orphan状态。
48. **E-DEFAULT-P1-48** Prefab Editor提供source/instance/effective三栏或等价diff视图，可定位每个change及其identity。
49. **E-DEFAULT-P1-49** Apply和propagation前展示affected instance数量、unloaded assets、conflict预测、cost与save/changelist计划。
50. **E-DEFAULT-P1-50** origin、override、reset和conflict UI具备keyboard、screen reader、high contrast、localization和非颜色唯一编码。

#### 4.2.6 Persistence、runtime与cook

51. **E-DEFAULT-P1-51** Scene World IO逐字段无损保存Prefab instance source、revision、mapping、local transform和typed operations。
52. **E-DEFAULT-P1-52** cache payload、project document、autosave、recovery和archive使用同一versioned instance codec并验证roundtrip。
53. **E-DEFAULT-P1-53** plugin component/property override必须可序列化或明确由adapter拥有，禁止`serializable: false`状态在保存中消失。
54. **E-DEFAULT-P1-54** 实现真实Prefab importer/loader/validator，diagnostic-only importer只可保留为Unavailable fallback。
55. **E-DEFAULT-P1-55** cook解析完整default链并发布immutable resolved prefab/scene artifact，附source/schema/catalog fingerprint。
56. **E-DEFAULT-P1-56** runtime通过artifact generation实例化，不在每帧解释字符串path、JSON或Editor authoring chain。
57. **E-DEFAULT-P1-57** nested Prefab/Class source建立cycle detection、maximum depth budget和完整diagnostic chain。
58. **E-DEFAULT-P1-58** source hot reload/reimport采用generation swap，旧实例安全迁移或保持旧generation并报告原因。
59. **E-DEFAULT-P1-59** World Partition/Level Instance/unloaded asset传播通过manifest和dependency receipt工作，不强制全量载入World。
60. **E-DEFAULT-P1-60** resolved artifact canonicalization、ordering和hash跨Windows/Linux及重复cook稳定。

#### 4.2.7 扩展、兼容与安全

61. **E-DEFAULT-P1-61** 建立`DefaultSourceProvider` registry，让native、script、Prefab和插件类型通过统一合同提供parent/default。
62. **E-DEFAULT-P1-62** 建立typed override codec/migration registry，未知codec必须保留opaque payload并fail closed。
63. **E-DEFAULT-P1-63** render、physics、animation、script等domain可注册propagation participant处理derived state和runtime reinstall。
64. **E-DEFAULT-P1-64** plugin default/override操作受capability、trust、resource budget和document scope限制，不能任意写其他资产。
65. **E-DEFAULT-P1-65** Editor/runtime/plugin/cook协商schema与codec version，不兼容组合在admission阶段给出完整诊断。
66. **E-DEFAULT-P1-66** legacy字符串JSON记录进入只读quarantine/migration workflow，禁止静默best-effort应用。

#### 4.2.8 性能与可观测性

67. **E-DEFAULT-P1-67** source-to-instance反向索引支持增量更新、partition和package粒度失效，避免项目全扫描。
68. **E-DEFAULT-P1-68** resolver按source/object/property generation缓存，并只重算change set影响的subtree和field。
69. **E-DEFAULT-P1-69** runtime artifact使用紧凑typed layout和预解析reference，frame hot path不得分配JSON、解析path或加全局锁。
70. **E-DEFAULT-P1-70** 为resolve、propagate、rebase、break、cook设定CPU、内存、I/O、队列和取消延迟预算。
71. **E-DEFAULT-P1-71** telemetry记录cache hit、fan-out、conflict/orphan率、stale rejection、rollback、artifact size和最长source chain。
72. **E-DEFAULT-P1-72** 建立10万实例、深层nested source、大override集合、source storm和unloaded partition benchmark，并与参考引擎同场景测量。

### 4.3 P2：产品增强

1. **E-DEFAULT-P2-01** 提供可视化inheritance/source graph和循环、fan-out、hotspot叠加。
2. **E-DEFAULT-P2-02** 提供override preset、批量选择性apply及可复用parameter set。
3. **E-DEFAULT-P2-03** 提供source revision时间线、历史diff和受控回退。
4. **E-DEFAULT-P2-04** 提供override搜索、过滤、按origin/conflict/type分组和跨资产查询。
5. **E-DEFAULT-P2-05** 提供propagation dry-run导出及CI可消费的machine-readable report。
6. **E-DEFAULT-P2-06** 提供可插拔custom conflict resolver及domain-specific merge preview。
7. **E-DEFAULT-P2-07** 提供instance promotion、variant generation和source extraction向导。
8. **E-DEFAULT-P2-08** 提供unused/orphan override自动修复建议，但所有删除必须显式确认并可撤销。
9. **E-DEFAULT-P2-09** 提供跨项目Prefab package导入时的identity remap和dependency reconciliation。
10. **E-DEFAULT-P2-10** 提供runtime provenance debug overlay，按需定位effective value来自哪个artifact/layer。
11. **E-DEFAULT-P2-11** 提供team policy控制可apply层、protected source和required review gate。
12. **E-DEFAULT-P2-12** 在基准证明收益后支持大规模instance override的压缩、dedup和shared immutable pages。

## 5. 目标架构

### 5.1 Authority与value resolution

建议核心合同：

```text
DefaultResolutionRequest
  = target identity
  + property address
  + expected source/catalog generation
  + resolution policy

EffectivePropertyValue
  = typed effective value
  + immediate parent value/source
  + origin layer/source revision
  + local override state
  + conflict/orphan diagnostic
```

解析顺序必须是数据合同而非UI约定。每一层只能覆盖下层，不得改变更低层source；Reset默认删除当前层的local operation后重新解析。显式“Reset to Class Default”可以移除多个上层operation，但必须先展示change set并进入事务。

### 5.2 Prefab source与instance record

```text
PrefabSourceDocument
  source identity + revision + schema/catalog fingerprint
  stable object/component graph
  typed properties + exposed parameter schema

PrefabInstanceRecord
  source identity + expected revision
  source-object -> instance-object map
  ordered typed override operations
  resolution/conflict/orphan receipts
```

authoring source可以保留完整Scene表达，runtime则接收已解析artifact。稳定ID必须在rename/reparent后保持，复制或跨项目导入时通过显式remap生成新namespace；path只用于展示。

### 5.3 Propagation pipeline

```text
source commit
  -> typed source change set
  -> reverse dependency index
  -> loaded/unloaded affected-instance plan
  -> clean/overridden/conflict/orphan classification
  -> preview + admission
  -> transactional publication
  -> cook/runtime generation invalidation
  -> durable receipt and diagnostics
```

clean字段自动更新，local override保留并更新base，三方冲突进入artifact等待用户或policy解决。任何缺失source、identity ambiguity、schema incompatibility或stale revision都必须阻止写入，不能以last-wins继续。

### 5.4 Command semantics

| 命令 | 必须完成 | 禁止替代品 |
|---|---|---|
| Reset | 删除指定层local op并重新解析effective value | 复制当前default为本地值 |
| Revert | 恢复immediate parent或指定source revision并更新live document | 只清空override vector |
| Apply to Source | CAS、跨document change set、原子保存、传播、rollback、receipt | 返回DTO后清空instance |
| Break | 物化完整effective subtree、remap reference/resource、验证后移除link | 只返回transform与override |
| Create from Selection | preflight ownership/reference/cycle、创建source、原子替换selection | 只注册menu descriptor |

### 5.5 Runtime与性能

默认值解析和冲突处理属于authoring/cook控制面。运行时加载generation-qualified resolved artifact，按紧凑object/component layout实例化；只有debug build或按需inspection保留provenance map。source编辑使dependency generation失效，由后台编译和原子swap更新，而不是让每个实例在frame loop中追踪source asset。

## 6. 责任边界

| Owner | 拥有 | 不拥有 |
|---|---|---|
| Runtime Asset/Scene | versioned source/instance codec、stable identity、resolved artifact、runtime instantiate | Editor菜单、conflict UX、save/changelist policy |
| Runtime Reflection | typed field identity、codec、native default provider、schema migration hooks | Prefab source保存和Editor transaction |
| Editor Default/Prefab domain | resolution projection、apply/revert/reset/break、propagation plan、toolkit | 私建第二套reflection或runtime ECS storage |
| Editor Inspector（Editor 05） | generic row、mixed state、origin/reset affordance与customization | 决定跨资产propagation算法 |
| Script/Class（Editor 31） | class schema compilation、script default publication、instance schema migration | Prefab topology与World persistence |
| Level Instance/World（Editor 41） | `prefab_instance`无损Scene roundtrip、Level Instance load/edit/rebase | 通用property row和class defaults |
| Snapshot/Diff（Editor 42） | 可复用semantic diff/three-way artifact primitive | default layer precedence与source authority |
| Asset/Save/Jobs/Diagnostics | dependency index、atomic save、background execution、receipts | domain override semantics |
| ECS archetype | component signature、table、row和query locality | CDO、Prefab、default/override或reset |

## 7. 里程碑

| 里程碑 | 交付与退出条件 |
|---|---|
| M0 | 真实性封口：Scene数据损失fail-closed，静态Workbench和descriptor-only Prefab入口Unavailable，危险helper不可被产品调用 |
| M1 | 身份与schema：source/object/component/property stable ID、revision、typed codec、legacy quarantine合同冻结 |
| M2 | Default authority：六层precedence、provider registry、effective value/origin/state API和缓存通过 |
| M3 | Prefab source/instance：versioned document、typed topology/property operations、nested cycle validation通过 |
| M4 | 无损持久化：Scene/cache/autosave/recovery/archive/cook roundtrip与migration通过 |
| M5 | Transactional commands：create/apply/revert/reset/break的preflight、CAS、rollback、undo/redo和receipt通过 |
| M6 | Propagation/rebase：dependency index、loaded/unloaded fan-out、三方分类、conflict/orphan artifact通过 |
| M7 | Inspector/Prefab UX：origin、mixed state、reset menu、diff、affected-instance preview和accessibility通过 |
| M8 | Runtime/cook：resolved artifact、generation install/hot reload、nested instantiate和debug provenance通过 |
| M9 | Plugin/domain extension：provider/codec/migration/participant、安全能力和兼容协商通过 |
| M10 | Scale/robustness：10万实例、partition、source storm、fault injection、跨平台确定性和性能预算通过 |
| M11 | 硬切与资格：删除legacy JSON/path authority和静态fixture入口，默认产品装配、文档、CI与release gates闭合 |

依赖顺序为M0 -> M1 -> M2/M3 -> M4 -> M5/M6 -> M7/M8 -> M9 -> M10 -> M11。M1前不得并行实现可写UI；M4前不得开放普通保存；M5前不得把helper接到menu；M6前不得声称source修改会安全传播。

## 8. 产品资格门

1. **G01** 含non-None Prefab instance的Scene经过load-save-reopen后source、revision、mapping、transform和所有typed operation逐字段相等。
2. **G02** 旧World IO遇到无法保留的instance记录时明确拒绝保存，不产生写成`None`的文件。
3. **G03** Prefab plugin缺factory、backend或任一声明资源时admission为Unavailable，UI不出现可执行动作。
4. **G04** 静态Prefab Workbench不能在production profile输出固定Apply/Validate成功或queued反馈。
5. **G05** rename/reparent source object后所有override仍由stable ID命中，display path随之更新。
6. **G06** field rename/type migration有alias/codec receipt；不兼容值成为TypeIncompatible而非静默丢弃。
7. **G07** Native、Script/Class、Prefab、Variant、Instance、Transient六层precedence有完整golden matrix。
8. **G08** 普通Reset只删除当前local override并恢复immediate parent，undo/redo完全可逆。
9. **G09** 显式reset到较低层会展示并准确删除跨越层的operations，不影响其他field。
10. **G10** Apply-to-Source遇到stale revision时零写入失败，source和所有instance保持原状。
11. **G11** Apply-to-Source在save、propagation或receipt任一fault点失败均完成跨document rollback。
12. **G12** Revert更新live document effective value、dirty、selection和Inspector snapshot，不只清空metadata。
13. **G13** Break后subtree visual/component/topology/reference语义等价，且不再依赖source asset。
14. **G14** Break内部、外部、soft和resource reference remap通过，失败时link和原实例不变。
15. **G15** nested source cycle在import、load、propagate和cook阶段都返回完整cycle chain。
16. **G16** local ComponentAdd/Remove与ChildAdd/Remove在source更新后按policy保留或产生可解释冲突。
17. **G17** source clean change自动更新所有loaded instance，已有local override保持effective value和provenance。
18. **G18** base/source/instance同时变化输出稳定三方conflict artifact，重复运行digest相同。
19. **G19** orphan、missing source、missing object和type mismatch在Inspector、Hierarchy和Diagnostics一致显示。
20. **G20** unloaded/partitioned consumer在下次load前已由manifest/receipt标记并完成安全重编译或阻断。
21. **G21** multi-selection分别解析每个target default，不因选择顺序改变结果。
22. **G22** mixed value、origin、override和editability状态可区分，keyboard与screen reader可完成reset/apply流程。
23. **G23** plugin unknown codec数据无损保留为opaque并阻止编辑，安装兼容provider后可恢复解析。
24. **G24** autosave、recovery、Session Archive和source-control external change不会丢失或重复应用override。
25. **G25** cook artifact包含source/schema/catalog fingerprint，stale或不兼容artifact拒绝安装。
26. **G26** runtime frame hot path不解析JSON/path、不遍历authoring inheritance chain且无新增全局锁。
27. **G27** 10万实例单字段source变更只访问受影响索引项，满足既定CPU、I/O和内存预算。
28. **G28** 深层nested source、万项override和source storm下job可取消，取消延迟和partial-state均满足预算。
29. **G29** Windows/Linux重复resolve、propagate和cook得到相同canonical artifact digest。
30. **G30** source/provider/plugin crash或进程终止后原子文件、transaction journal和receipt可恢复，无半发布generation。
31. **G31** 真实Prefab产品端到端覆盖create、edit source、override、reset、apply、revert、break、save、reopen、cook、runtime instantiate。
32. **G32** 与Unreal/Fyrox/Godot/Bevy/Unity Graphics可比场景公开测量correctness、fan-out、memory、artifact size和latency；只有实测优于目标基线才可声称性能领先。

## 9. 验证说明

本轮是review-only，没有修改production Runtime、Editor、Interface、Plugin、App代码或tests，也没有运行新的动态测试。此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断；本轮没有重复同一未变化且无法抵达Prefab/default产品行为的lane，也不能据静态测试数量宣称行为通过。

本报告静态验证要求：136个selected path存在且无重复；fingerprint匹配；P0/P1/P2分别为5/72/12；M0-M11连续；资格门为32；frontmatter、Editor索引、根索引与coverage链接无断链；Markdown为LF、无trailing whitespace或占位标记。动态实施阶段必须补Scene roundtrip、migration golden、default precedence matrix、transaction fault injection、source propagation/rebase、nested topology/reference、plugin compatibility、安全、跨平台确定性、10万实例benchmark及真实Editor端到端资格。

## 10. 审查决策

1. 保留ECS archetype实现与命名，但在文档/API中明确其仅为存储布局。
2. 保留reflection `default_value`作为native/schema provider输入，不把它升级为唯一default authority。
3. 保留Material Editor局部override作为domain adapter参考，不复制成第二套generic resolver。
4. 保留Prefab DTO只作为待迁移legacy输入；新写入在M1 schema冻结后走typed versioned codec。
5. 保留Prefab plugin的beta/Partial诚实状态，移除或fail-close无后端可执行入口。
6. Scene数据损失由Editor 41统一修复；本报告以依赖gate消费，不建立竞争owner。
7. Inspector通用行由Editor 05实施，Script Class schema由Editor 31实施，semantic three-way primitive可复用Editor 42。
8. 性能目标以resolved artifact、反向依赖索引、generation cache和实测资格实现，不以省略provenance、校验或事务换取表面速度。
