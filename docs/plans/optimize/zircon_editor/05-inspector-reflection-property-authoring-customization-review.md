---
related_code:
  - zircon_runtime_interface/src/reflect
  - zircon_runtime/src/scene/reflect
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_editor/src/core/extension/inspector.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/ui/binding_dispatch/inspector
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/ui/retained_host/app/inspector
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/inspector_fields.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
  - docs/plans/zircon_editor/editor/06/failure-2026-08-01-inspector-multi-selection-batch-mutation-missing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Public/PropertyHandle.h
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/PropertyHandleImpl.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/DetailLayoutBuilderImpl.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/DetailMultiTopLevelObjectRootNode.cpp
  - dev/godot/editor/inspector/editor_inspector.cpp
  - dev/godot/editor/inspector/multi_node_edit.cpp
  - dev/godot/editor/inspector/editor_properties_array_dict.cpp
  - dev/godot/editor/inspector/editor_resource_picker.cpp
  - dev/Fyrox/fyrox-ui/src/inspector
  - dev/bevy/crates/bevy_reflect/src/path
  - dev/bevy/crates/bevy_reflect/src/set.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/InspectorCurveEditor.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/RelativePropertiesDrawer.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 05 · Inspector、Reflection Property Authoring 与 Customization 工程化差距

## 1. 结论

当前 Inspector 已经具备一些值得保留的底层骨架：Runtime 有统一 `TypeRegistry`、built-in/dynamic component adapter、字段可编辑性与 `default_value/numeric_range/enum_options/editor_hint/documentation` schema；Editor 有稳定多选集合、命令捕获、单事务提交/回滚/undo、capability-scoped customization/field-editor contribution，以及 retained UI 的 draft event 与投影。它不是完全没有反射，也不是完全没有事务。

但真实产品 authoring 链没有把这些骨架组成一个工程级 property system。当前 Inspector 只有两条互相分裂的属性路径：Name、Parent、Local Translation、Scale由 `EditorState` 中五组字符串和硬编码component type path管理；插件dynamic component则从反射读出后再降成字符串。产品快照只枚举 `dynamic_components_for_entity`，因此Runtime已经注册的Camera、MeshRenderer、Mobility、Activation、RenderLayer、各类Light与RigidBody等built-in component没有进入通用Inspector。可见的`Add Component`只存在于 `.zui` route声明，没有production handler。

更严重的是提交语义破坏数据正确性。选择同步把Translation和Scale格式化为两位小数；任何Apply都会无条件把这些显示字符串重新解析并写回，即使用户只改Name或一个插件字段。多选时，系统又把primary的Name、Parent、绝对Local Translation、Scale以及Apply按钮重新打包的所有可编辑插件属性写到每一个selected node。现有测试明确把“所有选中对象变成同一个Name和同一个插件值”固化为成功行为。由此可以稳定推出：只需选中两个不同对象后点击Apply，即可覆盖secondary的未编辑属性；只需对带高精度transform的单个对象点击Apply，即可丢失小数精度。这两项符合全局P0的“持久化/authoring数据正确性被破坏”定义。

Customization同样只完成了注册外形。`InspectorCustomization::build`在production没有调用；surface的document/controller/template/data root/bindings进入snapshot后，controller、data root、bindings与field-editor kind/asset markers又在host projection中被丢弃。最终实际控件仍只是另一套按字符串`value_kind`判断的NumberField或InputField。Boolean、Color、Enum、Asset Reference与Curve editor并未按已解析的`FieldEditorKind`挂载；Curve本身仍明示为placeholder。插件customization目前实际作用更接近“允许generic string字段可编辑”的gate，而不是可执行的自定义属性面板。

本报告记录2个P0、32个P1、9个P2。没有运行Cargo、真实Editor、多窗口、10k属性、多选异构component、插件卸载、undo crash recovery或reference engine benchmark；性能结论只基于同步调用、复制/查找复杂度和缺失预算，不宣称已完成与Unreal/Fyrox/Godot的同机性能比较。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Runtime reflection clean production | 46 / 5,544 | E3：schema/value/address、registry、built-in/dynamic adapters、read/write；fingerprint `ded07e2a...bdce5` |
| Editor Inspector model clean production | 24 / 6,587 | E3：extension、snapshot、binding、state、command/transaction integration；fingerprint `611355dc...f423` |
| retained product projection clean production | 94 / 19,164 | E3：builtin surface、pane payload/projection、control dispatch、workbench bridge；fingerprint `974ffd03...ff35` |
| focused Inspector tests | 13 / 2,860 | E2：46个test attributes；fingerprint `78db2465...b7fcb`；未运行 |

fingerprint按相对路径排序，将 `path + NUL + per-file SHA-256 + LF` 串联后再计算SHA-256。它只标识本轮clean阅读集合，不是schema/version ID，也不能代替构建、运行或产品验收。

### 2.2 在途文件隔离

成文时以下文件有其他Session或用户修改，本轮没有用它们支撑稳定算法结论：

- `zircon_editor/src/ui/host/scene_inspection_publication.rs`；
- `zircon_editor/src/ui/template_runtime/component_adapter/inspector.rs`；
- `zircon_editor/src/ui/workbench/reflection/inspector_route.rs`及同目录animation/asset/docking/draft/root/registration/viewport route文件。

P0结论来自clean的`retained_host/app/inspector/surface_controls/apply_arguments.rs`、`binding_dispatch/inspector/apply.rs`、`workbench/state/editor_state_selection.rs`和`tests/editing/reflected_command.rs`，不依赖上述在途route。实施前仍必须重读active owner，确认route payload是否已经引入generation或changed mask；若已引入，必须硬切下游旧合同，不能并存两种提交语义。

### 2.3 本轮追踪的产品链

1. Selection mutation -> `sync_selection_state` -> primary Name/Parent/Translation/Scale字符串draft。
2. Editor snapshot -> primary scene artifact + `dynamic_components_for_entity` -> reflected fields -> plugin component/property snapshot。
3. snapshot -> pane payload -> retained host view data ->手写Inspector节点/控件。
4. ValueChanged -> component adapter/draft binding -> `EditorState`字符串字段或`inspector_dynamic_fields`。
5. Apply click ->重新读取完整snapshot ->打包base字段和所有editable插件属性 -> `EditorInspectorEvent` -> binding batch。
6. `ApplyInspectorChanges` ->枚举全部active selection ->为每个对象生成同一份base updates与draft dynamic updates ->捕获`SetReflectedSceneFieldCommand` ->一次Global transaction。
7. extension ticket/capability -> customization chain和field-editor container -> snapshot metadata -> pane projection；本轮继续追踪metadata在哪一层丢失。

## 3. 已有工程基础，重构时必须保留

### 3.1 Runtime reflection与字段约束

- `ReflectTypeRegistration`已经区分component/resource、serialization、plugin ownership、editor/remote/script visibility，并保留type/module/plugin identity。
- `ReflectFieldInfo`已有display name、editable、serializable、editor visibility、default value、numeric range、enum options、editor hint与documentation；问题是Editor不消费，不应另建Editor-only schema。
- Runtime同时支持按field name与field slot读写，component/resource adapter会校验可编辑性与value类型，`ZrReflectValue<f32>`及vector转换会拒绝非有限值。
- built-in registry已经注册Name、Hierarchy、LocalTransform、Activation、RenderLayerMask、Mobility、Camera、MeshRenderer、各类Light与RigidBody；目标是让Editor消费现有registry，不是复制第三套component descriptor。

### 3.2 Selection与事务原子性

- `SelectionModel`已有Edit/Play domain、稳定active set、primary与generation，多选本身不是缺失。
- `SetReflectedSceneFieldCommand::capture`在变更前读取typed `before`，相同值不产生命令，apply/revert都经过Runtime reflection，并把before/after写入journal payload。
- Inspector把所有命令一次交给`execute_scene_commands`，现有invalid parent与missing component测试覆盖了事务前失败或批次回滚，不应退回逐对象独立history。
- undo/redo会保留active multi-selection；应在此基础上增加per-property/per-object value，而不是复制SelectionModel或新建Inspector undo stack。

### 3.3 Extension ticket与fail-closed基础

- Customization和field editor进入ContributionStore前会校验ID、target type、`.zui` document、controller、template/data root与operation binding。
- capability snapshot会过滤当前启用贡献；qualified field type miss不会错误回退到同名built-in editor，插件撤销后的ownership边界是正确方向。
- dynamic component在schema或customization缺失时保持serialized data并切成只读诊断，没有直接把未知JSON写坏；这个fail-closed原则必须保留。

### 3.4 Retained投影与command入口

- UI ValueChanged只修改draft，真正world mutation集中在Apply command路径；基础方向符合“view不直接写world”。
- subject path至少会校验node存在，Apply前还会检查当前transaction selection context，并在错误时恢复Inspector UI checkpoint。
- retained pane已有滚动、稳定control identity、disabled/diagnostic projection与响应式字段宽度；本报告不要求推翻UI host，只要求替换property数据合同与控件解析。

## 4. P0：当前Apply会破坏未编辑数据

### E-INSP-P0-01 · 无关字段Apply会把Transform永久量化到两位小数

证据链：

- `editor_state_selection.rs:169-195`从authoritative scene读Translation/Scale后使用`format!("{:.2}")`写入Inspector draft；
- `apply_inspector_changes:48-56`在每次Apply都解析Translation与Scale，不检查这两个字段是否被编辑；
- `prepare_reflected_node_updates:94-115`始终生成Name、Parent、Translation、Scale四个update；
- `apply_arguments.rs:13-45`即使只点击Apply，也重新发送Name、Parent、Translation和全部插件属性；Scale虽未进入此payload，仍由state层无条件写回；
- `SetReflectedSceneFieldCommand`会把量化后的值当作正式`after`提交并允许保存。

因此原值`12.3456`会在一次无关Name/插件字段提交后变成`12.35`，原Scale同样被量化。UI显示精度可以是两位，但authoring draft不能用显示字符串替代typed source value。最低修复不是“多显示几位”，而是changed property handle只提交实际edited typed value，未编辑字段从命令集合中物理缺席。

### E-INSP-P0-02 · 多选Apply会把Primary完整快照覆盖到所有Secondary

证据链：

- snapshot只投影`active_primary`，Name/Parent/Translation/Scale及插件properties都是primary值；
- Apply按钮把所有base字段和所有`property.editable`插件字段重新打包，没有dirty mask；
- `apply_inspector_changes:37-71`枚举全部`active_items`，但每个对象都复用同一份`self.name_field/self.parent_field/self.transform_fields/self.scale_fields/self.inspector_dynamic_fields`；
- `reflected_inspector_batch_mutates_all_selected_nodes_in_one_history_record`明确断言Cube和Camera都被重命名为`Selected Batch`，不同coverage都被改成`0.8`；
- 原Editor06 failure record只要求“枚举active set”，没有定义mixed value、changed field或absolute/relative policy，修复因此把primary-only错误扩大成整对象覆盖。

复现不需要编辑字段：选择两个Name、Parent或Transform不同的对象后直接点击Apply，secondary会收到primary值；若primary的Parent本身在selection内，还可能因自父/循环校验使整个无关提交失败。正确模型必须先计算所有目标的共同schema与`Uniform/Mixed/Unavailable`值状态；只有用户明确编辑的property path才能产生per-target command。Transform还必须明确Absolute、Relative/Delta、Local/World和pivot语义，不能从primary显示快照推断。

## 5. P1：工程级property system缺口

### 5.1 权威、寻址与多对象语义

| 编号 | 缺口与源码证据 | 必须收敛到的合同 |
|---|---|---|
| E-INSP-P1-01 | `inspector_plugin_components`只调用`dynamic_components_for_entity`；Runtime已注册的Camera/Mesh/Light/Physics/Mobility/Activation/RenderLayer均不在产品Inspector通用枚举中。 | 从Runtime registry + entity component presence生成统一component/property tree；built-in与dynamic不得分两套发现机制。 |
| E-INSP-P1-02 | Name/Hierarchy/LocalTransform靠Editor硬编码type path和专用parser，dynamic字段走reflection；同一对象有两套property authority。 | base字段也通过统一property address与typed accessor，仅Transform customization决定呈现，不另存authoring真值。 |
| E-INSP-P1-03 | `InspectorFieldChange`只有`field_id/value`，Apply重发所有字段，`EditorState`没有dirty/provenance集合。 | request必须携精确changed path、typed value、edit mode、source generation与target set；no-op Apply不得产生命令。 |
| E-INSP-P1-04 | snapshot只读取primary，没有schema intersection、mixed value、per-object value或missing component状态。 | 多对象投影定义`Uniform(T) / Mixed / MissingOnSome / ReadOnly(reason)`，只显示所有目标兼容的共同属性并允许显式per-object策略。 |
| E-INSP-P1-05 | `entity://selected`在dispatch时重新解释current primary，`node://u64`只校验当前world存在；没有document/world/schema generation或selection revision。 | `PropertyEditContext`绑定DocumentId、WorldGeneration、SelectionRevision、schema generation与稳定object handles；stale event返回Conflict，不重定向到复用NodeId。 |
| E-INSP-P1-06 | Runtime read/write request仅有顶层`field_name: String`；Editor dynamic id靠最后一个`.`拆type与field。没有嵌套path、variant/index/key segment或持久stable field ID。 | 引入结构化`PropertyPathSegment`和可缓存slot plan；字符串只作显示/序列化格式，不能作为内部执行协议。 |
| E-INSP-P1-07 | `ReflectedValue`虽有List/Map/Json，Editor把它们标为不可编辑，Runtime request也没有insert/remove/move/resize等container mutation。 | property API提供struct/tuple/enum/list/map/set/option的递归读取与原子container op，并保留元素stable identity或明确index invalidation。 |
| E-INSP-P1-08 | Runtime reflection支持Resource address，但`SetReflectedSceneFieldCommand`和Inspector只写scene component。 | Asset、resource、project setting与scene component共用property frontend，后端通过domain-specific transaction adapter提交。 |

### 5.2 Schema与typed editor被丢弃

| 编号 | 缺口与源码证据 | 必须收敛到的合同 |
|---|---|---|
| E-INSP-P1-09 | `ReflectFieldInfo::default_value`没有Editor production consumer，也没有Reset/Revert入口。 | 每行提供可查询的default/override source、CanReset理由与transactional Reset；多选按目标各自default回退。 |
| E-INSP-P1-10 | `numeric_range(min/max/step/precision)`未进入snapshot；base Position反而硬编码±100000、step 0.1/1.0。 | 数值控件完全由schema/unit策略驱动，soft/hard range、slider、step、precision和clamp policy分离。 |
| E-INSP-P1-11 | `enum_options`与`editor_hint`未进入snapshot；Enum最终是任意字符串InputField。 | Enum使用受限option model、display/doc与unknown-value恢复；hint必须决定editor而不是type-name猜测。 |
| E-INSP-P1-12 | field/type documentation在Runtime存在但没有tooltip/help/search projection。 | schema documentation、deprecation、experimental、read-only reason与validation message进入稳定row model。 |
| E-INSP-P1-13 | `ReflectedValue`只有i64/u64/f32与少量向量；缺f64、128位、char/bytes、matrix/transform、typed option/result/set及精确asset/entity handle。 | 反射value应覆盖引擎公开authoring类型，或以typed dynamic reflect node避免把所有类型压扁进单一枚举。 |
| E-INSP-P1-14 | Resource是裸`String`；FieldEditor用21个marker猜asset类型，markers进入payload后在host view-data层被丢弃。 | 使用AssetTypeId/allowed base types/nullable policy；提供picker、搜索、drag/drop、clear、locate、preview与broken-reference状态。 |
| E-INSP-P1-15 | Entity是裸`Option<u64>`文本输入，没有scene/document identity、对象picker、过滤、cycle/ownership预览。 | 使用typed object reference和pick session，在提交前显示missing/cross-world/cycle/forbidden原因。 |
| E-INSP-P1-16 | Vec/Quaternion被格式化成逗号字符串；base Inspector没有Rotation，亦无Euler/Quaternion、local/world、unit或linked-axis策略。 | Vector/Rotation/Transform由compound property editor处理各子字段、坐标空间、单位、锁轴与连续drag transaction。 |

### 5.3 Field editor与Customization只停留在描述层

| 编号 | 缺口与源码证据 | 必须收敛到的合同 |
|---|---|---|
| E-INSP-P1-17 | `FieldEditorInstance`只保存kind与asset marker，不含真正widget factory、typed binding、validation或lifecycle；factory只是返回descriptor。 | field editor实例拥有build/update/validate/begin-edit/commit/cancel接口，并绑定一个typed property handle。 |
| E-INSP-P1-18 | snapshot/payload能解析`field_editor_kind`，但`InspectorPluginComponentPropertyViewData`丢弃kind与asset markers；实际节点重新按字符串`value_kind`只分NumberField/InputField。 | 已解析editor必须贯穿immutable projection到host，不允许consumer再次猜type。 |
| E-INSP-P1-19 | Boolean、Color、Enum、AssetReference从未进入production控件选择；Curve明确为`CurvePlaceholder`。 | 实装各类editor；Curve复用Editor07共享curve/timeline基础，而不是永久placeholder。 |
| E-INSP-P1-20 | `InspectorCustomizationChain::build`只在定义/测试出现，production只调用`matching`读取surface metadata。 | production必须执行受控layout customization，或删除虚假的build API并以真正surface composition替代。 |
| E-INSP-P1-21 | surface的controller/data root/bindings进入snapshot后在pane projection丢失；document/template只被写到header的value/validation文字，没有挂载插件UI。 | winning surface必须由ticket-owned host实例化、绑定controller/data root与operation allowlist，并在ticket撤销时确定性卸载。 |
| E-INSP-P1-22 | `customization_available = schema && customization`同时作为字段enabled gate；即使schema足以自动编辑primitive，缺class customization仍全部只读。 | 自动反射editor是安全fallback；customization只覆盖/扩展指定行。插件卸载时按ownership和schema availability决定保留只读或fallback。 |
| E-INSP-P1-23 | chain采用注册顺序first-match whole-target截获，没有priority、property-level customization、category extension、conflict report或多贡献组合。 | 分离class layout、property type editor、row extension与validation provider；排序/冲突是确定性合同并可诊断。 |

### 5.4 事务、错误、交互与component topology

| 编号 | 缺口与源码证据 | 必须收敛到的合同 |
|---|---|---|
| E-INSP-P1-24 | 所有编辑依赖显式Apply，最终固定`MergeMode::Disable`；没有interactive begin/update/end、preview、Esc cancel或scrub merge。 | 定义`Idle -> InteractivePreview -> Validating -> Commit/Cancel`状态机；slider/drag/color/curve形成一条可撤销事务。 |
| E-INSP-P1-25 | 错误统一降成String/whole-batch failure和status line，没有per-field validation、cross-field dependency、warning/repair action。 | typed validation result包含property path、target、severity、code、message、repair；批次原子失败仍能逐字段呈现原因。 |
| E-INSP-P1-26 | command capture只检查当前schema editable并记录before/after，没有expected object/schema revision；长寿命UI事件无法做compare-and-set。 | command携expected revision或在authoritative lock内重新验证snapshot token；stale edit必须明确冲突或重基，不能静默覆盖。 |
| E-INSP-P1-27 | `.zui`显示`WorkbenchAddComponent`并路由`workbench.inspector.add_component`，全仓没有production handler；remove/reorder/enable/disable component也不存在。 | component catalog、compatibility/dependency检查、add/remove/reorder/enable操作全部进入同一transaction与undo，未知plugin data保持可恢复。 |
| E-INSP-P1-28 | 没有copy/paste property、reset、pin/override、keyframe、context menu或source-of-value；这些不是装饰，而是大型authoring的日常操作。 | property row capability模型声明可用命令，统一走command registry与transaction，不在每个控件手写。 |

### 5.5 投影规模、性能与可观测性

| 编号 | 缺口与源码证据 | 必须收敛到的合同 |
|---|---|---|
| E-INSP-P1-29 | `editor_snapshot()`持有shell lock同步读取scene；`dynamic_components_for_entity`克隆完整JSON/descriptor，随后`reflect_fields`再次克隆字段值。 | generation-owned immutable property snapshot/delta；只有visible/expanded paths materialize value，重型值按需或摘要化。 |
| E-INSP-P1-30 | 每个schema field都线性`find`返回fields，形成O(F²)配对；pane又一次性为全部component/property创建节点，没有item/time/byte budget或虚拟化。 | field slot/hash join降到O(F)，tree按expanded range虚拟化；每帧build/visit/allocation有硬预算和延迟统计。 |
| E-INSP-P1-31 | reflect failure通过`unwrap_or_else`悄悄退到只读JSON；diagnostic只区分“无schema/无customization”，不保留真实read/type错误。 | snapshot携typed per-component/per-field error与retry/currentness；不得用fallback抹去adapter/schema corruption。 |
| E-INSP-P1-32 | 仓内并存builtin Inspector surface、pane动态节点和componentized workbench bridge：一条只含Name/Parent/Position，一条显示Scale；workbench data sync只取第一个plugin component，pane路径遍历全部。 | 单一Inspector presentation model和单一host adapter；所有surface消费同一row tree/delta，不保留功能不等价的三套投影。 |

## 6. P2：诊断、测试与维护债

| 编号 | 缺口 | 收敛要求 |
|---|---|---|
| E-INSP-P2-01 | dynamic Scalar文本parser接受`NaN/inf`，到Runtime adapter才失败；向量则在Editor提前拒绝，验证层级不一致。 | 一个typed validator同时服务draft与commit，非法值留在编辑态但不能进入command。 |
| E-INSP-P2-02 | field ID、type path、subject path和值在UI/event/state间反复字符串化，`rsplit_once('.')`隐含字段名/path语法约束。 | path codec集中版本化、round-trip/fuzz测试；内部使用结构化ID。 |
| E-INSP-P2-03 | number projection对parse失败使用`unwrap_or(0.0)`，可能让无效文本同时显示为数值0与原value_text。 | 明确`Valid/Invalid/Incomplete` draft状态，绝不把parse失败伪装成0。 |
| E-INSP-P2-04 | 显示精度、坐标、单位、label生成和numeric kind在多处手写；排序固定按字段名而非schema order/category。 | schema-ownedpresentation metadata与locale/unit formatter，保留声明顺序和稳定category order。 |
| E-INSP-P2-05 | customization若未来进入production，`can_handle/build`没有panic隔离、deadline、row/byte budget；active contribution重建用`expect`。 | plugin callback经fault boundary、预算与ticket quarantine，错误进入诊断而非Editor panic。 |
| E-INSP-P2-06 | 现有测试验证“多选全部写同一值”，却没有no-op Apply、无关字段、mixed values、精度保持或异构schema用例。 | M0先写失败回归并修正旧断言，防止错误合同继续充当完成证据。 |
| E-INSP-P2-07 | customization测试只检查metadata传递，未验证插件document/controller/binding真正实例化、事件回流与ticket revoke。 | 加真实plugin fixture端到端mount/edit/unload/reload测试。 |
| E-INSP-P2-08 | focused fixture规模小，没有1/100/10k fields、1/100/10k targets、125/500/1000Hz scrub、巨大List/Map或慢plugin基准。 | 建立build/visit/clone/allocation/p95/p99/frame budget与历史内存门。 |
| E-INSP-P2-09 | Editor06计划把descriptor、placeholder与“单事务多选覆盖”记为contract complete，failure record仍open，文档状态与产品语义不一致。 | 本报告成为canonical gap owner；实现后回填旧记录为superseded/fixed，不继续从旧完成声明推导质量。 |

## 7. 与参考引擎的差异及适用边界

### 7.1 Unreal PropertyEditor

- `PropertyHandle.h`把property handle定义为负责Pre/PostEditChange、transaction、package modification的统一读写入口；支持typed get/set、`MultipleValues`、per-object values、child handles、array handle、reset-to-default与possible values。
- `InteractiveChange`与`PropertyHandleImpl.cpp`中的begin/end transaction把spin/slider连续更新收敛成一条事务，并向每个top-level object发布change event与array index context。
- Details系统同时有class layout、property type customization、multi-top-level root、permission/restriction、copy/paste、asset picker、array/map/set和reset UI。

Zircon不需要复制UObject/Slate/Package架构，但必须复制三条原则：property handle拥有完整事务与通知语义；multi-object值不是primary快照；嵌套/container/reset/customization是property model的一部分，而不是控件特例。

### 7.2 Godot EditorInspector与MultiNodeEdit

- `multi_node_edit.cpp`只把所有目标都具有且PropertyInfo type/class/hint/usage兼容的属性放入共同property list；每次set只修改指定property或指定field，并在一个UndoRedo action中记录每个node的旧值。
- NodePath会按每个target重新计算相对路径，而不是把primary的字符串路径复制给其他对象；array/dictionary/object引用还更新引用计数关系。
- EditorInspector支持changing/commit区分、merge-ends、revert/default、property check/pin/key、array resize/clear和resource picker。

Godot当前实现也不是性能或类型安全上限，但它直接证明“多选=共同schema + 指定字段 + per-target转换”，不是“把primary完整Inspector复制N次”。

### 7.3 Fyrox Inspector

- `PropertyEditorDefinitionContainer`有numeric/bool/char/string/path/color/enum/vector/quaternion/matrix/range/collection/curve/inspectable等实际editor定义。
- nested inspectable通过`base_path`递归构建，collection editor产生Add/Remove/ItemChanged并为`field[index]`创建子editor，curve editor使用真实CurveEditor并同步消息。
- `FieldRef`把read-only、immutable collection、min/max/step/precision/tag/doc传给editor，InspectorContext支持filter与增量sync。

Fyrox证明Rust反射环境下不必把字段降成字符串；其实现规模小于Unreal，不能单独作为插件隔离或超大工程性能标准。

### 7.4 Bevy Reflect

- `ReflectPath`/`ParsedPath`结构化支持struct field/name/index、tuple、enum variant与list index，并能缓存预解析path避免重复字符串解析。
- Reflect体系覆盖Struct/Tuple/Enum/List/Array/Map/Set；Set提供insert/remove与dynamic representation。

Bevy参考树没有完整Editor Inspector/transaction UX，因此本报告只借用其nested path和container reflection能力，不把它当Editor产品基线。

### 7.5 Unity Graphics

- Graphics仓内`RelativePropertiesDrawer`使用`SerializedProperty.FindPropertyRelative`处理嵌套字段；`InspectorCurveEditor`持有`SerializedProperty`并通过`ApplyModifiedProperties`提交真实curve编辑。
- 该仓是UnityEditor consumer，不包含闭源SerializedObject/Inspector核心，不能据此推断完整multi-object、undo或custom editor内部实现。它只证明Graphics级material/render authoring依赖typed serialized property与真实curve control，而不是字符串placeholder。

## 8. 目标架构与唯一权威

### 8.1 Property地址

```text
PropertyAddress
  document_id
  world_generation
  target_object_ids[]
  component_type_id
  schema_generation
  path: [Field(stable_id), Variant(id), Index(i), MapKey(key), ...]
```

- `target_object_ids`是稳定目标集合，不在dispatch时重新解释`selected`；selection只用于创建session。
- `stable_id`由reflection registration/derive提供，rename通过schema migration映射；slot可作为同generation内加速缓存，不能持久化。
- resource/asset/settings使用同一path结构但不同domain transaction backend。

### 8.2 Inspector session

```text
Unbound
  -> Projected { document/world/schema/selection revisions, rows }
  -> Drafting { changed paths, typed draft, validation }
  -> InteractivePreview { transaction lease, per-target before values }
  -> Validating
  -> Committing -> Projected(new generation)
  -> Conflict { stale targets/schema/value }
  -> Cancelled -> Projected(authoritative values)
```

每个row必须保存`Uniform/Mixed/MissingOnSome/Unavailable`值状态、read/write/reset能力、validation与source/override信息。UI snapshot只是immutable session projection，不再由`EditorState`的Name/Parent/Transform字符串充当第二真值。

### 8.3 Mutation合同

```text
PropertyEditRequest
  context token
  exact changed PropertyAddress
  mode: Absolute | Relative | PerTarget | Reset
  typed value or per-target values
  interaction: Begin | Update | Commit | Cancel
  expected revisions
```

Authoring service先对所有目标解析path、检查共同schema/权限/依赖、生成per-target before/after，再打开一个transaction。任意目标失败则不发布world mutation；continuous interaction只保留一条history并支持Esc恢复。事件完成后发布精确changed paths/generation，Inspector、viewport、asset dirty与save系统消费同一结果。

### 8.4 Customization与field editor分层

| 层 | 职责 | 禁止事项 |
|---|---|---|
| Property type editor | 一个type/hint的typed value控件与validation | 不直接持world、selection或history |
| Property row extension | label、units、actions、visibility/read-only dependency | 不替换整个Inspector |
| Class/component layout | category/order/group/custom rows | 不绕过property handle写数据 |
| Surface/toolkit extension | 复杂专用asset/component UI | 必须使用ticket-owned lifecycle、operation allowlist与transaction adapter |

所有插件调用通过panic boundary、time/item/byte budget和ticket quarantine。自动反射layout始终可用；customization失效时回退或只读由schema ownership明确决定。

## 9. 硬切重构范围

以下旧合同不保留compat shim：

1. 删除`ApplyInspectorChanges`“读取整个EditorState表单并写全部selection”的语义；旧事件不能与changed-path request并存。
2. 删除`name_field/parent_field/transform_fields/scale_fields/inspector_dynamic_fields`作为authoring authority；若UI仍需draft，由`InspectorSession`按property address拥有。
3. 删除base component硬编码update生成器与`field_id.rsplit_once('.')`执行协议；Name/Hierarchy/Transform进入统一reflection/property path。
4. 删除descriptor-only `FieldEditorInstance`和production未调用的layout `build`假实现；一次迁移到可执行typed editor/customization contract。
5. 删除host层按`value_kind`二次猜editor；resolved editor identity和schema metadata必须端到端保留。
6. 删除只显示但无法执行的`Add Component` route，或在同一里程碑接入真实component transaction；不保留永久假按钮。
7. 合并三套Inspector projection；旧surface只能在同一hard cut中迁移成新row model consumer，不能继续功能分叉。

## 10. 测试先行的依赖序里程碑

### M0 · P0数据正确性封口

先增加失败测试：

- high-precision Translation/Scale在no-op Apply、Name edit、dynamic field edit后bitwise或policy-defined等价；
- 两个Name/Parent/Transform/plugin value不同的对象多选后，no-op Apply零命令；
- 只改一个field只影响该field，secondary未编辑字段不变；
- mixed Transform分别覆盖Absolute、Relative和PerTarget策略；
- selection含primary parent时，无关字段提交不触发reparent/cycle错误。

然后hard cut到changed-path/per-target command request，修正并重命名当前固化错误语义的测试。M0不得通过增加显示精度、隐藏Apply或回退primary-only来关闭。

### M1 · Runtime reflection path与schema generation

- 在`zircon_runtime_interface`定义结构化property path、stable field/variant IDs、schema generation、typed validation/error与container operation。
- derive/registry提供nested struct/tuple/enum/list/map/set/option信息与rename migration；保留现有field slot fast path。
- fields/schema query增加cursor/range、item/byte/time budget和immutable generation snapshot。
- component/resource adapter实现统一read/prepare-write/commit或等价transaction hook。

### M2 · InspectorSession与全component投影

- 从active selection创建versioned session，枚举built-in和dynamic component presence，计算共同schema与mixed values。
- base Name/Hierarchy/Transform迁入统一row tree；Transform customization只负责compound UI与坐标策略。
- 以generation delta更新expanded/visible row，删除EditorState字符串authority和O(F²)join。
- 将read error、missing component、plugin unloaded、read-only reason投影成typed diagnostics。

### M3 · Property transaction与交互编辑

- 引入exact changed-path request、per-target prepare、expected revision、atomic commit与precise invalidation。
- 支持Begin/Update/Commit/Cancel、merge-ends、Esc、focus loss与plugin unload中断；历史只记录typed before/after。
- 接入Reset/Copy/Paste、override/source indicator、context action；keyframe/pin通过capability扩展，不污染核心类型。
- Scene、asset、resource、project setting分别实现backend adapter，共用frontend/session。

### M4 · Typed editor、Customization与component topology

- 实装Boolean/Number/Enum/Color/Vector/Rotation/Transform/Asset/Entity/Curve/Collection editor并消费全部schema metadata。
- production mount插件document/controller/data root/bindings；class/property/row customization可组合、有序、可撤销。
- component add/remove/reorder/enable进入catalog、dependency validation和同一事务；未知plugin data可保留与恢复。
- ticket revoke/panic/timeout后确定性取消interaction、卸载surface并回退到auto/read-only projection。

### M5 · 规模、产品与故障验收

- 1/100/10k fields与targets、深层path、巨大container、500/1000Hz scrub下记录build/visit/clone/allocation和p95/p99。
- 多窗口/多document、scene replace、NodeId reuse、plugin unload/reload、schema migration、undo/redo/save/reopen与crash recovery端到端测试。
- Windows产品Editor真实输入、asset picker、drag/drop、IME数字/文本、curve、color、multi-select与截图/交互验收。
- 与reference同硬件benchmark时报告数据集、版本、build profile与预算；不以“控件可见”替代authoring正确性。

## 11. 产品级验收门

1. 对任意未编辑property，任何其他字段commit、no-op Apply、selection refresh后值保持不变。
2. 任意f32/f64合法值经过显示、编辑其他字段、save/reopen不因Inspector格式化损失精度。
3. 多选只显示兼容共同schema；mixed状态不借用primary值，指定字段commit不修改其他字段。
4. Absolute/Relative/PerTarget Transform语义有独立测试，父子层级下Local/World结果可预测。
5. Camera、MeshRenderer、全部Light、Physics、Mobility、Activation与RenderLayer可由同一property tree查看/编辑。
6. built-in与dynamic component使用同一address、validation、transaction和undo路径。
7. nested struct/enum/list/map/set能读取、编辑、insert/remove/move/reset并完整undo/redo。
8. numeric range/step/precision、enum options、documentation、read-only reason与default真实驱动UI和validation。
9. Asset/Entity editor提供typed filter、picker、drag/drop、clear、locate与broken-reference状态，不能接受不兼容裸字符串。
10. Boolean/Color/Enum/Vector/Rotation/Curve不退化为generic text；Curve不是placeholder。
11. continuous scrub/color/curve drag每个gesture只有一条history，Esc/focus loss/plugin unload恢复before值。
12. 任意target validation失败时批次零部分提交，UI逐field/target显示typed原因。
13. scene/document/schema/selection generation变化时旧事件返回Conflict，NodeId复用不能命中新对象。
14. Reset/Copy/Paste/component add/remove/reorder全部可撤销并参与dirty/save generation。
15. 插件custom surface实际mount、执行allowlisted binding、撤销时unmount；panic/timeout不会终止Editor。
16. auto reflection fallback在无customization时仍能安全编辑受支持字段；unknown/unloaded plugin data保持只读可恢复。
17. 10k field tree只构建visible/expanded range，没有每帧完整JSON/field clone或O(F²)join。
18. 1000Hz输入不会产生1000条history或无界队列；frame/property budget有可观测超限与降级策略。
19. 三套旧Inspector projection和字符串authoring authority物理删除，stale symbol/route扫描为零。
20. focused、package、workspace、Windows产品与save/reopen矩阵全部产生新鲜receipt；没有运行的门明确保持open。

## 12. 依赖、owner与后续复核

- 本报告是Inspector/property authoring差距的canonical owner；Editor03旧报告继续拥有scene/selection/gizmo/prefab，Editor02继续拥有document dirty/save/recovery，不在这些报告复制P0。
- M1修改`zircon_runtime_interface`与Runtime reflection公共合同，必须先于Editor M2-M4；后续Interface DTO全量审查仍负责ABI/serialization/version兼容，不重复拥有Inspector行为。
- M3复用现有transaction/history engine；若需要扩展interactive lease或history memory budget，回填Editor02/旧Editor03 command plan的共享合同，但不新建第二套undo。
- Curve editor依赖Editor07共享curve/timeline基础；Asset picker依赖Editor04 authoritative catalog/reference；component topology依赖Editor03 scene command owner。
- 实施前必须重读本轮隔离的dirty route/publication/adapter文件，并核对其他Session是否已改变payload或invalidation合同。

本切片只完成静态review与重构设计，没有修改production代码，也没有声明Engine/Editor整体review完成。下一Editor切片继续审plugin UX/authoring与Play viewport/runtime bridge；Inspector实现应在用户后续授权进入代码修正阶段后，按M0-M5自底向上执行。
