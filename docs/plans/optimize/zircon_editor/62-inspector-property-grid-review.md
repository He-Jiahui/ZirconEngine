---
title: Editor Inspector、Property Grid、Reflection Schema、Multi-Selection、Edit Transaction、Undo、Prefab Override、Customization、Asset Reference、Virtualization 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor62
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_editor/src/core/extension/inspector.rs
  - zircon_editor/src/core/extension/field_editor.rs
  - zircon_editor/src/core/state/editor_state_selection.rs
  - zircon_editor/src/core/state/editor_state_snapshot_build.rs
  - zircon_editor/src/core/state/inspector.rs
  - zircon_editor/src/ui/retained_host/binding_dispatch/inspector
  - zircon_editor/src/ui/retained_host/app/inspector
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/runtime_component/adapter/inspector.rs
  - zircon_editor/src/ui/runtime_component/adapter/reflection.rs
  - zircon_editor/src/ui/scene_inspection_publication.rs
  - zircon_editor/assets/ui/editor/inspector
  - zircon_editor/assets/ui/editor/workbench/components/workbench_inspector_panel.zui
  - zircon_runtime/src/scene/reflection
  - zircon_runtime/src/scene/world
tests:
  - zircon_editor/src/tests/editing/inspector.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs
  - zircon_editor/src/ui/runtime_component/adapter/inspector.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/44-archetype-class-defaults-instance-override-property-propagation-reset-to-default-authoring-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
  - docs/plans/optimize/zircon_editor/60-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
  - docs/plans/mvp/05-f4-basic-authoring.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor
  - dev/godot/editor/inspector/editor_inspector.cpp
  - dev/godot/editor/inspector/editor_inspector.h
  - dev/godot/editor/inspector/multi_node_edit.cpp
  - dev/godot/editor/inspector/multi_node_edit.h
  - dev/Fyrox/fyrox-ui/src/inspector
  - dev/bevy/crates/bevy_reflect/src/path
  - dev/bevy/crates/bevy_reflect/src
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor
doc_type: current_source_review
canonical_owner: docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Inspector、Property Grid、Reflection Schema、Multi-Selection、Edit Transaction、Undo、Prefab Override、Customization、Asset Reference、Virtualization 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Inspector不是纯静态假面。Runtime已有schema、field slot、dynamic component反射读写和原子command capture；Editor已有selection snapshot、history transaction、customization ticket、field editor descriptor、retained projection和两套可见Inspector surface。这些基础应保留并收敛，而不是另造第四套Inspector。

但产品语义仍不合格。Editor05登记的两条P0在当前源码中都可稳定到达：任何一次Apply都会把未编辑的Translation/Scale按两位小数草稿重写；多选Apply会把primary的Name、Parent、Translation、Scale和dynamic字段完整覆盖给所有secondary。旧retained pane的显式Apply会触发它们，componentized workbench的单轴Position/Scale提交也会落到同一个full-form Apply，因此新界面没有绕开数据破坏路径。

本轮还确认产品内同时存在三种不等价的Inspector投影：legacy pane、componentized workbench和runtime component adapter。`FieldEditorInstance`在中途被丢弃，Boolean/Color/Enum/Asset/Curve最终退化为Number或Text；插件组件属性的Edit与Commit都只修改模板节点预览，从未生成World mutation；focused field delta已发布却没有被Inspector消费；`virtual_rows`按总属性数clone节点，不是viewport windowing；auto reflection又被“必须存在customization”错误地关成只读。

Editor05继续是唯一canonical owner。本报告新增canonical findings为 **0 P0 / 0 P1 / 0 P2**，只刷新其 **2项P0 Open、32项P1 Open、9项P2 Open与20个资格门Fail** 的当前源码证据。Prefab/default/instance override归Editor44，asset reference工作流归Editor57，document/world identity归Editor61，selection/multi-world归Editor60，reflection schema/address/artifact归Runtime111；跨报告汇总不得重复相加。

目标不是把现有表单补几个控件，而是建立`InspectorSessionRegistry + PropertyProjectionArtifact + TypedPropertyHandle + PropertyEditCoordinator + PropertyEditorRegistry + ComponentAuthoringCoordinator`。Runtime拥有稳定schema、typed address、访问计划、revision和mutation prepare；Editor拥有selection/session、mixed value、draft/interaction、transaction/undo、customization、source-of-value与可见行池。所有可见surface必须消费同一immutable row tree和delta，并通过同一精确changed-path transaction提交。

本轮为review-only。未修改production Rust，未运行Cargo、真实Editor、save/reopen、plugin reload、pointer/IME、10K字段/目标、fault/soak/profile或同语义跨引擎benchmark，不能据此声称性能或表现优于Unreal。tooling按用户要求排除。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes | 本轮证据 | fingerprint |
|---|---:|---|---|
| Zircon production focused set | **52 / 9,342 / 334,986** | core Inspector、state/snapshot、binding dispatch、legacy pane、componentized workbench、adapter与publication | `48fce3c2d4d74750db117b00a8b17c1270ba9eb8307ada7da079e1fe20e32b48` |
| Focused tests | **11 / 2,742 / 95,728** | selection/Apply、reflected batch、adapter与component row preview | `5cd5662b12b6cb7911f7cead9a2cd6bba0f49434accdd6959a82befe2076e614` |
| Inspector UI assets | **7 / 811 / 57,491** | legacy body/surface与componentized workbench panel | `d47465c6124944030746c0e46be561b8d76e88c17e63d5a164c423a68ade3ba7` |
| Unreal PropertyEditor selected set | **4 / 9,238 / 323,605** | property handle、per-object values、interactive transaction、details roots | `2fbbfd5c43ebbbb013d9f44a26d49c5515fed849a134f1900cdbc2588a1a6b84` |
| Godot Inspector selected set | **4 / 10,340 / 362,775** | multi-node common properties、per-target undo、revert/pin/key/resource | `c15df248a6c03bb3233240280dfc6899dc32722d60282a27a0273bd35e3fbf66` |
| Fyrox Inspector subtree | **28 / 8,315 / 309,683** | real editor definitions、recursive paths、collections、curve与sync | `de3ccaaf59793ba957f6ae9ce9f5910617e2d960c3c9d784d2c4f211ff2c6d09` |
| Bevy Reflect selected set | **6 / 2,679 / 93,749** | parsed structured path与reflect container kinds | `a718f9b237c933a1a9ff3e9bdb51735aa8f383e5c83631ba66d8854ac7da23c0` |
| Unity Graphics selected set | **2 / 1,046 / 39,813** | relative SerializedProperty与真实curve consumer | `1820bba19d25a50861e35941ace524d63460eb566dd2e3acd3f8f69c30b8d1da` |

fingerprint按当前working-tree文件内容冻结，只证明本轮读取集合，不是ABI、artifact、cache或动态验收receipt。主仓基线为`bee4c707b714738346b49bba15c59468b8bd9b39`，coordinator baseline epoch为339。Godot、Fyrox、Bevy和Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像随主仓基线冻结。

### 2.2 在途文件隔离

focused corpus中只有`zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/component_property_rows.rs`存在非本Session修改，差异仅为两个binding reference补`component_event: None`。它不改变component property Edit/Commit只更新row preview的结论。本报告没有覆盖或回退该改动；实施前仍必须重取所有focused文件。

### 2.3 canonical owner与去重规则

| 主题 | 唯一owner | Editor62职责 |
|---|---|---|
| Inspector/property authoring finding与20门 | Editor05 | current-source刷新、产品路径补证、实施切片排序 |
| Reflection schema/address/artifact/subscription | Runtime63；当前刷新Runtime111 | 只定义Editor消费合同，不复制Runtime finding |
| Transaction/history/save/autosave | Editor02 | 复用history，不建立Inspector私有undo |
| Prefab/archetype/default/instance override | Editor44 | source-of-value/reset UI消费其authority |
| Selection、Hierarchy、multi-world与stale NodeId | Editor60及Editor03 | InspectorSession绑定其qualified context |
| Scene document/world identity | Editor61 | PropertyAddress携Document/World generation |
| Asset browser/reference picker | Editor57 | Inspector只拥有typed reference editor与pick session |
| Plugin lifecycle/customization ownership | Editor50 | customization ticket与surface卸载复用其lifecycle |
| UI message/event runtime | Editor48/49 | row delta与edit intent使用其typed event合同 |

Editor62不新增finding编号，也不把父报告P0重新相加。后续若源码改变，先回填canonical owner状态，再刷新本报告的currentness。

## 3. 当前产品链与事实边界

### 3.1 共享state与mutation链

```text
selection/world changes
  -> sync inspector drafts from primary entity
     -> Translation/Scale format to two decimal strings
     -> dynamic values stringify into InspectorField

legacy Apply or componentized field submit
  -> InspectorFieldBatch updates one or more draft strings
  -> ApplyInspectorChanges always reads the whole draft form
  -> for every selected NodeId, generate:
       Name
       Hierarchy.parent
       LocalTransform.translation
       LocalTransform.scale
       every dynamic draft field
  -> one EditorTransaction, MergeMode::Disable
  -> capture before/after and commit atomically
```

事务引擎能对command batch做原子提交和undo，这是可保留基础；危险发生在事务之前：request并不表示“用户改了哪个property”，而是“把primary来源的整个表单重新写给当前selection”。因此原子性只能保证错误一起发生，不能保证语义正确。

### 3.2 legacy retained pane

```text
InspectorSnapshot
  -> inspector_projection
     -> drops FieldEditorInstance and asset markers
  -> inspector_fields
     -> guesses NumberField or InputField from value_kind string
  -> explicit Apply button
     -> Name / Parent / Position / plugin fields packed
     -> shared full-form Apply also rewrites existing Scale drafts
```

该pane没有Rotation或Scale控件，却仍可能在任意Apply时重写Scale。Boolean、Color、Enum、AssetReference和Curve不会挂载真实editor；numeric parse失败会以`0.0`继续投影，形成显示/草稿双重事实。

### 3.3 componentized workbench

```text
workbench_inspector_panel.zui
  -> Position and Scale axis submit routes
  -> Rotation controls are visible but have no edit routes
  -> plugin component property Edit / Commit routes
       -> edit_inspector_component_property
       -> mutate template node value/value_text
       -> refresh surface + request paint
       -> no Editor binding, no World command, no history
```

Position/Scale axis route看似typed提交，但dispatch最终仍执行shared full-form Apply，所以两项P0同样可达。插件component property的Edit与Commit实现完全相同，测试也只断言row preview和repaint；任何“commit成功”文案都不能当作World mutation receipt。

data sync只消费`inspector.plugin_components.first()`，而legacy pane遍历全部component。`WorkbenchAddComponent`可见并已路由，但当前production没有handler；Rotation可见而无route。三个事实共同证明componentized surface尚不是功能完整的替代路径。

### 3.4 runtime component adapter

runtime component Inspector adapter把`ValueChanged`和`Commit`都解释为更新draft字符串，并返回带`transaction_id`字段的label；它没有提交authoring transaction。adapter只覆盖Name、Parent、Translation和dynamic fields，Scale又缺失。reflection adapter采用相同draft模式，因此命名上的transaction不能替代World/history terminal receipt。

### 3.5 inspection publication旁路

Runtime/Editor publication链已经发布immutable `WorldInspectionArtifact`、selection revision、focused fields与带generation的field delta；retained consumer当前只使用hierarchy fragment触发reflow，没有把focused field delta接入Inspector。Inspector仍在shell lock内同步重建完整字符串snapshot，并重复clone dynamic component JSON/descriptor/fields。

这同时确认Editor05 P1-29/P1-32和Runtime111的artifact/subscription缺口：增量事实已经存在一部分，但产品Inspector仍旁路它。修复必须扩展统一artifact消费，不应再添加第四条snapshot通道。

## 4. P0 current-source裁决

### 4.1 E-INSP-P0-01：未编辑Transform仍被量化

当前状态：**Open**。

1. selection同步把Translation和Scale格式化为两位小数字符串。
2. 任意Apply都重新解析这些字符串并生成Transform command，哪怕用户只改Name或一个plugin property。
3. componentized单轴Position/Scale submit也会继续触发全表单Apply。
4. focused测试明确期望`12.50`等量化字符串，旧测试正在固化错误合同。

关闭条件不是把显示精度提高到更多位，而是让display/draft与authoritative typed value分离，commit只携精确changed path；no-op或编辑其他字段时不得生成Transform command。

### 4.2 E-INSP-P0-02：primary完整快照覆盖所有secondary

当前状态：**Open**。

1. snapshot和draft只读取primary，没有mixed value或per-target before/value状态。
2. Apply遍历全部selected NodeId，对每个目标写相同Name、Parent、Translation、Scale与dynamic draft。
3. `reflected_inspector_batch_mutates_all_selected_nodes_in_one_history_record`明确断言多个目标获得相同Name和plugin值，测试把数据覆盖误认成正确多选语义。
4. parent、relative object path、Transform和override default本应按目标解析，复制primary字符串尤其危险。

关闭条件是共同schema投影、`Uniform/Mixed/MissingOnSome/ReadOnly`值状态，以及Absolute/Relative/PerTarget/Reset的显式语义。每个目标必须独立计算before/after，不能从primary完整快照推导。

## 5. Editor05 finding状态复核

### 5.1 P1总账

| canonical范围 | 数量 | 当前状态 | 本轮关键补证 |
|---|---:|---|---|
| E-INSP-P1-01..08 权威、寻址、多对象与container | 8 | **8 Open** | built-in与dynamic仍分裂；batch仍只有string field/value；subject在dispatch时重解；List/Map/Json不可编辑 |
| E-INSP-P1-09..16 schema、default、typed reference与Transform | 8 | **8 Open** | range/enum/default/doc未进入row；asset/entity仍是裸字符串/NodeId；Rotation可见无route |
| E-INSP-P1-17..23 field editor与customization | 7 | **7 Open** | descriptor在projection丢弃；`build()`无production caller；无customization即只读；surface未真实mount |
| E-INSP-P1-24..28 interaction、validation、revision、component与daily actions | 5 | **5 Open** | Commit只改row preview；无begin/update/end；无expected revision；Add Component无handler；无reset/override/source |
| E-INSP-P1-29..32 artifact、规模、diagnostic与单一presentation | 4 | **4 Open** | focused delta未消费；shell lock全量snapshot；`virtual_rows`物化全部；三条presentation功能不等价 |
| 合计 | **32** | **32 Open** | 无Partial或Closed证据 |

### 5.2 P2总账

| canonical范围 | 数量 | 当前状态 | 本轮关键补证 |
|---|---:|---|---|
| E-INSP-P2-01..04 validation/path/projection/presentation metadata | 4 | **4 Open** | Scalar仍接受NaN/inf草稿；path反复字符串化；parse失败伪装0；label/order/precision多处手写 |
| E-INSP-P2-05 plugin fault与budget | 1 | **Open** | customization build尚未进入production，更无panic/deadline/item/byte边界 |
| E-INSP-P2-06..09 tests、plugin fixture、scale和文档一致性 | 4 | **4 Open** | 旧测试固化覆盖；property Commit只测preview；无large-scale/gesture/currentness矩阵 |
| 合计 | **9** | **9 Open** | 无Partial或Closed证据 |

### 5.3 本轮current-source elaboration到canonical ID的映射

| 当前事实 | canonical映射 | 裁决 |
|---|---|---|
| 单轴typed submit仍触发full Apply | P1-03、P1-24、P1-32；两项P0 | 不新增编号；证明新surface未关闭旧破坏路径 |
| runtime component返回`transaction_id`但只写draft | P1-24、P1-32 | receipt命名不构成transaction事实 |
| component property Commit只改模板preview | P1-03、P1-17、P1-24、P1-32 | 必须接PropertyEditCoordinator，不允许继续假提交 |
| focused field delta已发布却未消费 | P1-29、P1-32；Runtime111 | Editor消费缺口，不复制Runtime artifact finding |
| `FieldEditorInstance`在pane projection丢弃 | P1-17..21 | resolved editor identity必须端到端保留 |
| 无customization时auto reflection只读 | P1-20、P1-22、P1-23 | customization只能覆盖fallback，不能作为primitive edit admission |
| `virtual_rows`按总行数clone | P1-30、P2-08 | 不是viewport range virtualization |
| workbench只同步第一个plugin component | P1-01、P1-32 | 统一component tree后删除特殊投影 |
| Rotation可见但无route | P1-16、P1-19、P1-32 | 不以可见控件声明功能完成 |
| Add Component有route无handler | P1-27 | 与remove/reorder/enable一起进入component transaction |
| 无default/instance/prefab source-of-value/reset | P1-09、P1-28；Editor44 | UI消费Editor44 authority，不在Inspector重造override真值 |
| Asset/Entity仍无typed picker | P1-14、P1-15；Editor57 | 复用asset workspace/reference service |

## 6. 现有基础必须保留

1. 保留Runtime reflection registry、schema generation方向、field slot和dynamic component adapter，升级为stable ID与nested typed path。
2. 保留command batch的capture-before、atomic commit、rollback与undo，修正输入语义为exact changed path和per-target plan。
3. 保留immutable inspection artifact、selection revision、focused field delta和generation过滤，扩展为Inspector唯一projection source。
4. 保留extension registration ticket、customization/field editor容器与卸载ownership方向，替换descriptor-only实例。
5. 保留retained模板、componentized panel和native visible-row paint基础，但合并成一个row model consumer。
6. 保留当前typed `UiBindingValue`入口，停止在draft/mutation边界把它再次stringify。
7. 保留动态component反射失败时不写World的fail-closed行为，但错误必须成为typed row diagnostic，不能被只读JSON静默掩盖。
8. 保留focused单元测试中的atomic rollback覆盖；把固化full overwrite的断言改成RED regression，不删除失败场景。

## 7. 五引擎对照与适用边界

### 7.1 Unreal PropertyEditor

`IPropertyHandle`提供child/array handle、multiple-values、per-object values、reset-to-default和pre-change入口。`PropertyHandleImpl.cpp`在interactive gesture开始时开启一条transaction，向top-level object发布pre/post change及per-object array index context，在gesture结束时关闭；只有实际值变化才标dirty。Details体系还区分class layout与property type customization，并支持多个top-level root。

Zircon不复制UObject、Slate或Package，但必须达到相同原则：handle拥有typed address、对象集合、transaction和通知；multi-object不是primary复制；interactive edit不是每帧一条history；array/map/set/reset是property model，而非各控件私有逻辑。

### 7.2 Godot EditorInspector与MultiNodeEdit

`MultiNodeEdit::_get_property_list`只暴露所有选中节点都存在且name/type/class/hint/hint_string/usage兼容的property。`_set_impl`只修改指定property/field，对每个target重新计算NodePath并记录独立旧值，再以一条`MERGE_ENDS` undo action提交。EditorInspector同时具有changing/commit区分、single-property update与tree rebuild、revert/default、pin、key、copy/paste、array和真实resource picker。

Godot证明多选的最低正确性是“共同schema + 指定字段 + per-target转换”，而非“复制primary表单”。它不是Zircon类型安全或规模性能上限。

### 7.3 Fyrox Inspector

Fyrox的`PropertyEditorDefinition`真正创建widget、生成sync message并把widget message翻译为`PropertyChanged`；type-keyed container记录plugin source以便卸载。默认editor覆盖bool/string/numeric/vector/matrix/range/quaternion/path/color/collection/curve等；nested object与collection使用递归base path和index path；`FieldRef` metadata进入control。

这直接反驳“Rust反射只能做字符串表单”的假设。Zircon可借鉴definition/container/message分层，但仍需补齐更强的revision、multi-document、plugin fault、large-scale和typed transaction合同。

### 7.4 Bevy Reflect

`ParsedPath`把field/name/index、tuple、list和enum variant解析成可缓存structured access，避免每次事件重新切字符串。Reflect kind覆盖struct/tuple/enum/list/array/map/set，set支持insert/remove。

Bevy参考树提供反射底座，不提供完整Editor Inspector/undo UX。本报告只采用structured path、container kind和预解析计划，不把它作为产品基线。

### 7.5 Unity Graphics

Graphics仓内`RelativePropertiesDrawer`通过`SerializedProperty.FindPropertyRelative`查找嵌套字段并交给`PropertyField`；`InspectorCurveEditor`持有真实`SerializedProperty`并调用`ApplyModifiedProperties`。这证明渲染资产authoring也依赖typed serialized property和真实curve control，而非字符串placeholder。

该仓是闭源UnityEditor核心的consumer，无法独立证明Unity完整multi-object、undo或custom inspector内部质量；本轮不从缺失源码推导结论。

## 8. 目标架构与唯一权威

### 8.1 Runtime-owned reflection contract

```text
StableTypeSchemaCatalog
  TypeSchemaId + SchemaGeneration
  StableFieldId / StableVariantId
  type kind + metadata + default/revert contract
  nested/container topology

TypedPropertyAddress
  domain: Scene | Resource | Asset | ProjectSetting
  qualified object identity
  component/type schema identity
  path: Field | Variant | TupleIndex | ListItem | MapKey | SetValue

CompiledPropertyPlan
  address + schema generation -> bounded accessor plan
  typed read / validate / prepare mutation

ReflectionMutationTransaction
  expected object/schema revisions
  per-target before/after
  atomic commit / rollback / precise changed-path receipt
```

Runtime不拥有Inspector panel、selection UX或widget。Editor不得用type-path/field-name字符串替代上述合同，也不得在dispatch时重新解释`entity://selected`。

### 8.2 Editor-owned Inspector session

```text
InspectorSessionKey
  DocumentId
  WorldGeneration
  SelectionRevision
  stable target handles[]
  SchemaGeneration

PropertyRow
  stable row/category identity
  TypedPropertyAddress
  value: Uniform | Mixed | MissingOnSome | Unavailable
  editor descriptor + presentation metadata
  read/write/reset/copy/paste/key/override capabilities
  validation + source-of-value + diagnostic

InspectorInteraction
  Idle
  -> Drafting
  -> InteractivePreview
  -> Validating
  -> Committing | Conflict | Cancelled
```

draft只能属于session中的changed path，不能再把Name/Parent/Transform字符串保存在全局EditorState作为第二真值。selection/world/schema变化必须退休旧session；旧事件返回Conflict，不能命中新World复用的NodeId。

### 8.3 Property edit coordinator

```text
PropertyEditIntent
  session token
  exact changed address
  value mode: Absolute | Relative | PerTarget | Reset
  typed draft/value
  phase: Begin | Update | Commit | Cancel
  expected revisions

PropertyEditCoordinator
  resolve all targets
  compute per-target plan
  validate permissions/dependencies/cycles
  open or reuse history transaction lease
  preview or atomically commit
  publish typed receipt + changed-path delta
```

no-op产生零command和零dirty generation。任一目标失败则整批零World mutation，同时返回per-field/per-target typed diagnostic。连续drag/color/curve gesture只保留一条history；Esc、focus loss、selection change、plugin unload和document close都必须显式commit或cancel。

### 8.4 Editor registry与presentation

| 层 | owner | 职责 |
|---|---|---|
| Property type editor | Editor core/plugin ticket | build/update/validate/begin/commit/cancel一个typed value control |
| Property row extension | Editor core/plugin ticket | label、unit、actions、visibility、read-only dependency、validation UI |
| Component/class layout | Editor core/plugin ticket | category/order/group/custom row组合，不绕过property handle |
| Complex surface/toolkit | Editor extension host | ticket-owned document/controller/data root/binding allowlist与确定性卸载 |
| Row tree/virtualization | Inspector presentation owner | immutable generation、expanded/visible range、bounded cell pool、delta patch |

auto reflection是安全fallback；customization只覆盖或扩展指定type/property/layout。插件卸载、panic或超时后，session取消interaction并回退auto/read-only，不能让整个component因为没有customization而不可编辑。

### 8.5 Prefab override与asset reference边界

Inspector row只消费Editor44提供的`SourceOfValue`、default、instance override、inherited/locked和revert plan，不自己比较序列化字符串推断override。Reset对多选目标分别回到各自default/source，并通过同一transaction提交。

Asset/Entity editor只持typed reference constraint和pick-session token。asset查询、folder/filter/drag/drop/locate/preview/broken state复用Editor57；entity picker复用qualified scene/document/world identity。裸URI、裸u64和marker substring不能作为兼容性判断。

## 9. 硬切重构范围

以下旧合同不保留compat shim：

1. 删除`ApplyInspectorChanges`读取全局完整表单并写全部selection的行为；所有提交必须携exact changed address。
2. 删除Name/Parent/Transform与dynamic property两套mutation authority；base component也进入统一schema/address。
3. 删除`entity://selected`的late binding和`node://u64`的无generation寻址；旧subject path不得与新session token并存。
4. 删除mutation边界的typed-value-to-string-to-typed round trip，以及`field_id.rsplit_once('.')`执行协议。
5. 删除descriptor-only `FieldEditorInstance`和host按`value_kind`二次猜控件；resolved editor identity端到端保留。
6. 删除component property“Commit只更新preview”的route；未接真实transaction前不得继续暴露为Commit。
7. 删除功能不等价的legacy pane、componentized和runtime adapter三套presentation model；surface只能消费同一row tree/delta。
8. 删除按总行数clone的`virtual_rows`语义，迁移为viewport/expanded range和bounded reusable row pool。
9. 删除无handler的Add Component和无route的Rotation假完成面；在同一里程碑接真实能力或移除入口。
10. 删除“customization存在才editable”的admission；schema和property permissions决定auto editor，customization只改变呈现。

## 10. 依赖有序里程碑

### ED62-M0：数据破坏止血

先写RED tests并hard cut提交协议：

- no-op、Name、plugin field编辑后，未编辑Translation/Scale保持typed value不变；
- 多选不同Name/Parent/Transform/plugin值时只改指定property，其他per-target值保持不变；
- legacy Apply和componentized axis route都不能触发full-form mutation；
- empty/no-op batch产生零command、零history、零dirty；
- parent/relative reference按每个target解析，任一失败整批rollback。

修正当前把两位小数与多选全覆盖当作正确结果的测试。M0不得用隐藏Apply、提高格式精度或禁止多选规避。

### ED62-M1：Runtime schema、address与mutation prepare

依赖Runtime63/111：建立stable type/field/variant identity、nested/container address、compiled access plan、typed value/validation、expected revision和per-target prepare。built-in与dynamic component进入同一catalog；asset/resource/scene通过domain adapter共享frontend，不共享错误的存储实现。

### ED62-M2：InspectorSession与immutable projection

从Editor60/61的qualified selection/document/world context创建session，计算共同schema与mixed values。消费Runtime inspection artifact和focused field delta，只为expanded/visible path materialize value；删除shell lock下完整JSON/field clone、O(F²)join和EditorState字符串authority。

### ED62-M3：交互事务、undo与冲突

实现Begin/Update/Commit/Cancel、transaction lease、precise invalidation、per-target before/after、validation与Conflict。接入Reset/Copy/Paste、source-of-value和override；history复用Editor02 owner，不建立Inspector专属undo stack。

### ED62-M4：typed editors、customization与component topology

实装Boolean、Number、Enum、Color、Vector、Rotation、Transform、Asset、Entity、Curve、Struct、Collection editor。production真正mount plugin document/controller/data root/bindings，支持ticket revoke/fault budget。Add/Remove/Reorder/Enable/Disable component进入catalog/dependency validation和同一transaction。

### ED62-M5：单一presentation与虚拟化

将legacy pane、componentized workbench和runtime component adapter迁移到一个row tree/delta consumer，随后物理删除旧projection。建立expanded/visible range、bounded cell pool、focus/IME/drag capture和accessibility virtualization；大值只投影摘要或按需编辑器。

### ED62-M6：产品与规模资格

完成真实Windows Editor、save/reopen、undo/redo、multi-document/multi-world、plugin unload/reload、schema migration、prefab override、asset/entity picker、10K fields/targets、1000Hz scrub、fault/soak/profile和同硬件对照。只有新鲜receipt可关闭Editor05门禁。

M0受MVP F4直接需要，但当前MVP `00`仍in_progress且F4受F3阻塞；本轮不越过门禁实施production代码。

## 11. 产品资格门current-source裁决

| # | Editor05门禁摘要 | 当前状态 | 失败证据 |
|---:|---|---|---|
| 1 | 未编辑property永不被其他commit改变 | **Fail** | full-form Apply重写全部base/dynamic drafts |
| 2 | 合法浮点显示/其他编辑/save不损失精度 | **Fail** | Translation/Scale同步量化到两位小数 |
| 3 | 多选共同schema、mixed、指定字段提交 | **Fail** | primary-only snapshot与完整覆盖 |
| 4 | Transform Absolute/Relative/PerTarget语义 | **Fail** | 无模式；Rotation route缺失 |
| 5 | built-in组件统一可查看/编辑 | **Fail** | 只枚举dynamic component |
| 6 | built-in/dynamic同address/transaction | **Fail** | 两套硬编码/反射路径 |
| 7 | nested/container完整编辑与undo | **Fail** | List/Map/Json只读，无container op |
| 8 | range/enum/doc/default驱动UI | **Fail** | metadata在snapshot/projection丢失 |
| 9 | typed Asset/Entity picker与约束 | **Fail** | 裸String/NodeId与marker猜测 |
| 10 | typed controls不退化，Curve真实 | **Fail** | pane只选Number/Text，Curve placeholder |
| 11 | gesture一条history且可cancel | **Fail** | Apply + MergeMode Disable，无interaction state |
| 12 | validation失败零部分提交且逐项诊断 | **Fail** | batch string error/status line |
| 13 | stale document/world/schema/selection冲突 | **Fail** | dispatch时重解selection，NodeId无generation |
| 14 | Reset/Copy/Paste/component topology可撤销 | **Fail** | 无actions；Add Component无handler |
| 15 | plugin surface真实mount且fault-isolated | **Fail** | production只读取matching metadata |
| 16 | 无customization仍有auto fallback | **Fail** | `schema && customization`作为enabled gate |
| 17 | 10K tree只构建visible/expanded range | **Fail** | `virtual_rows`按总量clone；全量snapshot |
| 18 | 1000Hz输入有界且history合并 | **Fail** | 无scrub state、budget、queue/merge证明 |
| 19 | 三套projection与字符串authority删除 | **Fail** | 三条路径仍并存且功能分叉 |
| 20 | focused/package/workspace/产品/save矩阵新鲜 | **Fail** | 本轮仅静态审查，旧测试覆盖不足 |

门禁合计：**0 Pass / 0 Partial / 20 Fail**。任何控件可见、preview变化、metadata存在或单元测试返回transaction label，都不能替代World mutation、history、save/reopen与currentness receipt。

## 12. 测试缺口与首批RED矩阵

当前11个focused测试共2,742行，能证明部分transaction原子性、draft同步和template preview，但缺少以下关键矩阵：

1. no-op Apply、编辑无关property、非法draft恢复和零dirty/零history。
2. 多选Uniform/Mixed/MissingOnSome/ReadOnly、异构schema和per-target default/reference转换。
3. 高精度f32/f64、NaN/inf、incomplete numeric draft、locale/unit与save/reopen。
4. stale document/world/schema/selection generation、NodeId复用和selection在事件队列中变化。
5. Begin/Update/Commit/Cancel、Esc、focus loss、pointer capture、IME、1000Hz scrub与history merge。
6. Boolean/Enum/Color/Rotation/Asset/Entity/Curve/collection真实widget到World再到undo/redo。
7. plugin customization mount、event return、panic/timeout、ticket revoke、unload/reload和fallback。
8. component Add/Remove/Reorder/Enable/Disable、dependency/cycle、unknown plugin data保存恢复。
9. prefab/archetype/default/instance override、Reset、source-of-value和multi-select不同来源。
10. 1/100/10K fields与targets、深path、巨大List/Map、visible row allocation、p95/p99与history memory。

M0先建立前四类数据正确性RED tests；M4/M6再补真实widget、plugin和规模矩阵。不要继续用`include_str!`或row preview断言代替产品资格。

## 13. 实施边界、验收receipt与后续复核

每个里程碑至少保存以下新鲜receipt：

| receipt | 必须包含 |
|---|---|
| Schema receipt | catalog/schema generation、stable IDs、migration、nested/container coverage、fingerprint |
| Projection receipt | document/world/selection/schema revisions、row generation、visible range、build/clone/allocation预算 |
| Mutation receipt | exact changed addresses、target set、per-target revisions、validation、transaction/history ID、terminal disposition |
| Interaction receipt | begin/update/commit/cancel、merge count、before restore、focus/plugin/document interruption结果 |
| Product receipt |真实surface、pointer/keyboard/IME、asset/entity picker、prefab override、component topology、save/reopen |
| Scale receipt | dataset、hardware/build、fields/targets/input rate、p50/p95/p99、allocation/history memory、degrade/fail policy |

实现时按ED62-M0至M6推进，并同步回填Editor05的finding和20门状态；Runtime合同变化回填Runtime111，override/reference/document/selection变化分别回填Editor44/57/61/60。不得因本报告是current-source refresh而另建平行owner。

本切片只完成静态review、五引擎对照和重构路线，没有修改production代码，也没有声明Editor或Engine整体review完成。下一轮继续逐域深审；进入代码修正阶段后，Inspector必须先关闭两项P0，再扩展typed editor和高级工作流。
