---
related_code:
  - zircon_editor/assets/ui/editor/keymap/default.keymap.toml
  - zircon_editor/assets/ui/editor/components/showcase/showcase_collections_section.zui
  - zircon_editor/assets/ui/editor/components/showcase/showcase_selection_section.zui
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/drag_source.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/entity
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/value/remap.rs
  - zircon_runtime/src/ui/surface/input/keyboard_clipboard.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/clipboard.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
tests:
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/scene_and_object.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_clipboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/clipboard_newline.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorActor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorServer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditorActions.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/EditorEngine.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Tests/CopyPasteCrossActorRefTests.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/godot/editor/docks/scene_tree_dock.h
  - dev/Fyrox/editor/src/scene/clipboard.rs
  - dev/Fyrox/editor/src/ui_scene/clipboard.rs
  - dev/bevy/crates/bevy_ecs/src/entity/clone_entities.rs
  - dev/bevy/crates/bevy_ecs/src/component/clone.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/CopyPasteGraph.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Drawing/Views/MaterialGraphView.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 55 · Editor Structured Clipboard / Cut-Copy-Paste / Duplicate / Delete / Cross-Document Remap / Drag Payload 产品集成工程化差距

## 1. 结论

Zircon Editor当前没有结构化authoring clipboard产品。默认command registry、keymap、Edit/Selection菜单、`EditorIntent`与`EditorCommand`都没有Copy、Cut、Paste或Duplicate；产品只接通Delete Selection。组件showcase里可见的`Duplicate|Ctrl+D`只是静态展示资产，不是可执行命令。层级drag确实能把当前多选节点重挂到另一个parent，但它使用`scene://node/{id}`字符串和旁路`Vec<NodeId>`，不具备可跨文档、可验证或可持久的transfer payload。

这不是补四个快捷键就能关闭的差距。工程级复制必须同时定义根选择归一化、任意组件捕获、依赖闭包、内部与外部引用政策、stable identity重映射、目标上下文、资源/owner/name冲突、原子事务、撤销恢复、OS clipboard格式、拖拽同构协议、大小预算与不可信输入边界。Unreal、Godot、Fyrox、Bevy和Unity Shader Graph分别证明了上下文能力路由、single-action paste、图克隆映射、per-component clone policy以及结构化依赖闭包这些最低约束。

本轮还确认一项现存P0数据完整性缺陷：`DeleteNodeCommand`捕获固定字段`Vec<NodeRecord>`，执行删除时调用`remove_entity_recursive`取得无损`DetachedEntityBatch`后立即丢弃，撤销再用`insert_node_records`重建。`NodeRecord`只覆盖内建场景字段，因此Delete→Undo会静默丢失任意注册typed component、动态/插件组件、observer以及原始tick/storage状态。当前节点ID和层级能恢复，并不等价于实体无损恢复。

本报告登记 **1项P0、64项P1、12项P2与40个资格门**。Editor55唯一拥有通用structured authoring transfer的payload、capture/plan/apply orchestration、scene产品命令、跨文档transfer和drag convergence；Editor02继续拥有通用transaction/history/save/recovery，Editor03拥有scene hierarchy/prefab业务语义，Editor05/23/24拥有各自domain的copy/paste UI语义，Editor08拥有命令与keymap基础设施，Runtime11A/11B拥有通用host request与文本clipboard，Runtime24拥有通用qualified identity。

## 2. 审查边界、currentness与证据等级

### 2.1 冻结语料

| 子域 | 文件 / 行 / bytes | 测试 | 证据等级 | 本轮检查重点 |
|---|---:|---:|---|---|
| 聚焦Zircon源码与资产 | 78 / 17,182 / 602,452 | 93个`#[test]`、1个ignore | E3 | command/intent/delete/selection/history、层级drag、DynamicScene、无损detach、文本clipboard与host request合同 |
| Editor产品调用链 | command defaults、keymap、menu、intent、workbench state、retained hierarchy host | Delete已接通；Copy/Cut/Paste/Duplicate production入口为0 | E3 | UI入口到事务、选择与Scene mutation的真实路径 |
| Runtime transfer候选底座 | `DetachedEntityBatch`、`DynamicScene`、`EntityRemap`、reflected value remap | whole-world capture/spawn有测试；无selected subtree API | E3 | 本地无损恢复与portable transfer必须分域 |
| 参考源码 | 13 / 29,536 / 1,117,327 | Unreal/Godot/Fyrox/Bevy/Unity Graphics | E2/E3 | capability routing、root normalization、deep clone、dependency closure、identity remap、single transaction与selection result |

78份聚焦文件按normalized relative path的ordinal顺序写入`path + NUL + raw bytes + NUL`后取SHA-256，working-tree fingerprint为`f31924c2c742d6ae368be14ece7edbd786b5f5f647b8a9751b70bb90d569f7bd`。13份参考源码按同一算法计算，fingerprint为`7cdd90cad2ec13b711d7ab5e6cc6ea657ec86ff6cec5abea355ce30b8ab7dfa9`。冻结Git基线为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。

聚焦范围只有`zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs`存在非本轮产生的在途修改。本轮按working tree内容审查并设置`source_recheck_required: true`；实施前必须重算78文件fingerprint、重查全仓Copy/Cut/Paste/Duplicate caller并重新验证Delete无损恢复，不能回退共享工作树。

### 2.2 产品能力矩阵

| 能力 | 当前入口 | 当前执行事实 | 工程结论 |
|---|---|---|---|
| Delete Selection | command、Delete键、Selection菜单、inspector/template route | 顶层根归一化、camera预检、单transaction、选择同步 | 产品可达，但撤销存在P0数据丢失 |
| Copy / Cut / Paste | 无command、无keymap、无menu、无intent | 0个Editor structured clipboard service | 不存在产品能力 |
| Duplicate | showcase静态文案含`Ctrl+D` | 无command/intent/scene clone调用 | 展示fixture不能作为功能证据 |
| Hierarchy reparent drag | pointer down构造`SceneInstance` payload | authoritative IDs另存`active_hierarchy_drag_node_ids`，drop转`SetParents` | 可保留交互，payload不是transfer authority |
| Runtime text clipboard | `UiClipboardRequest::{ReadText, WriteText}` | Runtime UI可产生host request；Editor retained host没有消费链 | 只覆盖文本，且Editor产品桥断开 |
| `DynamicScene` | `from_world`与spawn API | 捕获整World和serializable reflected state，目标分配新ID并remap | 可复用底层，但不是selected-object clipboard |
| `DetachedEntityBatch` | recursive removal返回move-only batch | 保存完整erased component/observer/tick/storage state | 适合进程内撤销，不适合直接序列化/跨进程 |

### 2.3 Delete→Undo数据损失证明链

| 阶段 | 当前代码事实 | 丢失面 |
|---|---|---|
| Capture | `DeleteNodeCommand::capture`只保存每个实体的`NodeRecord` | 固定记录之外的component/observer从未进入command state |
| Apply | `remove_entity_recursive`返回`DetachedEntityBatch`，command只判断空值后丢弃 | Runtime已经产生的无损恢复载荷被主动放弃 |
| Revert | `insert_node_records(&self.records)`重建节点 | 只恢复NodeRecord枚举的内建字段 |
| Identity | 原NodeId可恢复 | “同ID”掩盖了实体组成、tick和observer已改变 |
| Multi-delete | parent+child会折叠顶层根并在单事务执行 | 根归一化正确，但每个root都复用同一有损command |
| Recovery/journal | journal只序列化NodeRecord payload | durable replay也无法恢复捕获时不存在的数据 |

## 3. 必须保留的工程基础

1. 保留Delete前的去重、`top_level_node_ids`父子折叠、聚合camera preflight和单`MergeMode::Disable`事务；问题在恢复载荷，不在这些选择/准入规则。
2. 保留`SceneSelection`的generation、ordered items、primary与事务后同步；paste/duplicate必须返回明确selection result而不是从World重新猜测。
3. 保留`DetachedEntityBatch`的move-only exact storage语义，把它升级为local undo/rollback owner，不把它错误序列化为公共clipboard格式。
4. 保留`DynamicScene`的target-generation preflight、typed error、transactional spawn和`EntityRemap`，但新增subset capture、显式external-reference policy和component clone/serialize registry。
5. 保留hierarchy reparent的多选节点集合、drop target计算和`SetParents`事务，把drag的authority迁移到统一transfer handle/envelope。
6. 保留Runtime11B的文本编辑copy/cut/paste行为；structured authoring clipboard使用自定义MIME并提供文本fallback，不与文本控件争夺快捷键。
7. 保留Editor08的command descriptor、when predicate、keymap和menu model；Editor55只贡献业务command与capability，不再建立平行快捷键系统。
8. 保留domain owner的业务规则。Scene、UI Asset、Shader Graph、Data Table和Property各自实现provider，通用service只拥有传输协议与原子编排。

## 4. 参考源码给出的结构约束

| 参考 | 本轮源码事实 | 对Zircon的约束 | 不照搬的部分 |
|---|---|---|---|
| Unreal `LevelEditorActions` | Cut/Copy/Paste先委派Typed Element common actions，再回退Component、Actor和Scene Outliner能力 | command必须按focus/selection/document capability路由，不能把Ctrl+C固定绑定Scene | 不复制Slate command宏或legacy多路实现 |
| Unreal `EditorActor` / `EditorServer` | copy过滤不可复制对象，导出selected context；paste导入当前level并替换selection；cut在transaction内copy+delete | capture、destination、selection和transaction必须有显式合同；cut不能先删后赌clipboard成功 | 不采用文本Actor导出作为唯一格式 |
| Unreal cross-actor reference test | default/instance/nested subobject及交叉Actor引用在paste后指向新peer，不依赖map顺序 | remap必须覆盖深层对象图并有顺序无关测试 | 不复制UObject反射模型 |
| Godot `SceneTreeDock` | copy只取top selected nodes并排序；paste支持child/sibling/replacement、cycle检查、local resource remap、owner恢复和单UndoRedo action | 根归一化、目标语义、resource/owner policy、selection result必须是paste plan的一部分 | 不复制Node owner/property系统 |
| Fyrox scene/UI clipboard | 独立clipboard持有内部Graph/UI；先deep clone到clipboard，再deep clone到destination并返回old→new mapping/root handles | Rust实现应使用隔离snapshot和确定映射；scene/UI domain不能互相猜类型 | 其进程内clipboard不足以直接作为跨应用格式 |
| Bevy `EntityCloner` | per-component clone behavior、opt-in/out、move/clone、required component、linked relationship递归、EntityMapper和insert mode均显式 | Runtime clone registry必须拒绝未声明策略，不能假定所有ECS component可Clone/Serialize | 不要求Zircon改成Bevy ECS API |
| Unity Shader Graph | `CopyPasteGraph`包含node/group/edge/input/category/note；剔除不可复制node和orphan edge；GraphView补依赖闭包、目标资格、undo和selection | payload必须domain-typed、versioned，并由provider决定依赖闭包与失效关系 | Unity Graphics只证明package graph authoring，不推断Unity Scene内部实现 |

## 5. P0：必须先封闭的数据完整性缺陷

### ED55-P0-01 · Delete撤销丢失NodeRecord之外的实体状态

`DeleteNodeCommand`在capture阶段只保存`Vec<NodeRecord>`。apply阶段调用Runtime的`remove_entity_recursive`，该API已经返回包含table/sparse/dynamic component、observer、tick、kind、stable order和signature的`DetachedEntityBatch`，但Editor立即丢弃；revert阶段再以`insert_node_records`重建固定场景字段。只要节点上存在registered typed component、插件/动态component、observer或依赖原tick/storage状态的系统，Delete→Undo就会在UI看似恢复节点时静默损坏数据。多选删除、事务回滚和journal recovery不能弥补capture时未保存的数据。

**必须重构：** 将local `DeleteNodeCommand`改为拥有每个top-level root的exact detached batch及必要的parent/order/selection恢复信息；apply成功后batch进入command state，revert原子reattach exact batch，再次redo时重新detach并更新state。capture/preflight不能提前mutation。若transaction中途失败，已detach批次必须逆序无损恢复。durable journal另用versioned portable component snapshot，并对不可持久component fail-closed或声明recovery policy；绝不能把move-only内存batch伪装成跨进程序列化格式。

## 6. P1：工程级结构化传输能力差距

### 6.1 产品入口、能力路由与真实状态（ED55-P1-01～08）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED55-P1-01 | command registry没有Copy/Cut/Paste/Duplicate descriptor | 由Editor55注册typed authoring commands，能力由当前focus、document、selection和provider snapshot决定 |
| ED55-P1-02 | 默认keymap没有Ctrl/Cmd+C/X/V/D | 通过Editor08 keymap贡献跨平台primary modifier chord，并处理文本控件优先级 |
| ED55-P1-03 | Edit菜单只有Undo/Redo，Selection菜单只有Create/Delete | 由同一command state生成Cut/Copy/Paste/Duplicate/Delete的enabled/visible/reason |
| ED55-P1-04 | `EditorIntent`与`EditorCommand`没有structured transfer operation | 引入capture/prepare/apply请求与terminal receipt，不把大payload塞进UI intent enum |
| ED55-P1-05 | showcase静态`Duplicate|Ctrl+D`可被误认为已实现 | fixture必须绑定真实command或明确标记demo-only，产品审查不再把文案当能力 |
| ED55-P1-06 | 没有focus/domain capability resolver | 建立`TransferDomainProvider` registry，按focused surface、document kind、selection kind与operation解析唯一owner |
| ED55-P1-07 | command无法解释disabled原因 | 暴露no selection、read-only、unsupported type、invalid destination、clipboard incompatible等typed reason |
| ED55-P1-08 | keyboard/menu/context-menu/remote automation没有同一执行authority | 全部路由同一operation service和receipt，禁止各入口手写copy/delete顺序 |

### 6.2 Capture、根归一化、依赖闭包与payload schema（ED55-P1-09～20）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED55-P1-09 | 没有通用capture request或source snapshot | 定义绑定project/document/generation/provider/selection/operation的immutable `CaptureRequest` |
| ED55-P1-10 | 只有Delete临时实现scene top-level root归一化 | 将root normalization变成provider合同，稳定排序并返回被折叠项诊断 |
| ED55-P1-11 | Scene没有selected entities/subtree的`DynamicScene` capture API | 增加subset capture plan，显式包含descendants、components、resources和reference closure |
| ED55-P1-12 | `DynamicScene::from_world`只能捕获整World | 把whole-world snapshot与portable subset artifact分开，避免复制整个场景来实现几个节点的paste |
| ED55-P1-13 | reflected capture只接受registered且`serializable`字段 | clone/clipboard policy registry必须区分cloneable、portable、editor-only、runtime-only、transient与forbidden |
| ED55-P1-14 | unreflected typed component和observer没有portable policy | 未声明策略时fail-closed并列出具体type/owner；允许provider注册clone/serialize/remap adapter |
| ED55-P1-15 | 没有domain dependency collector | provider构建对象、component、subobject、edge、resource、owner和external reference闭包 |
| ED55-P1-16 | 没有orphan/invalid relation清理阶段 | capture后运行typed graph validation，剔除或拒绝失去endpoint的edge/constraint/binding |
| ED55-P1-17 | 没有versioned payload envelope | 定义format ID、schema/compiler version、engine build、domain、source identity、limits、digest和capability manifest |
| ED55-P1-18 | payload没有逐entry稳定身份和类型描述 | 使用provider-local portable object key、stable type/schema ID及显式parent/ownership关系 |
| ED55-P1-19 | 没有依赖manifest或缺失依赖诊断 | 记录asset/package/plugin/schema依赖、可内嵌/可解析策略与paste前link report |
| ED55-P1-20 | capture结果没有coverage/partial语义 | 默认要求complete；允许partial时逐项记录omission code，UI不能把降级结果显示为完整复制 |

### 6.3 Identity、reference remap与跨文档政策（ED55-P1-21～32）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED55-P1-21 | 现有scene ID是单World数值，payload没有source document/generation | 每个capture绑定qualified source identity，paste拒绝stale generation或重新resolve source |
| ED55-P1-22 | `EntityRemap`只表达旧ID到新ID | 扩展paste plan中的portable key→qualified destination identity，并保留Runtime内部快速map |
| ED55-P1-23 | fixed `NodeRecord`只remap parent和少量Joint引用 | 建立字段级reference descriptor/visitor，覆盖所有内建component与nested container |
| ED55-P1-24 | reflected value remap依赖约定JSON/entity形状 | reflection metadata必须声明entity/resource/subobject reference，禁止形状猜测 |
| ED55-P1-25 | source ID不在map时可能保留同数值并命中target World | external reference必须按Keep/Resolve/Null/Reject/Import policy显式决定，跨文档默认不能碰巧命中 |
| ED55-P1-26 | 没有internal/external reference分类报告 | prepare阶段生成reference table、resolution status、target与policy provenance |
| ED55-P1-27 | 没有跨Actor/节点互相引用顺序无关保证 | 先分配全部destination identity，再执行component/subobject materialization和reference patch |
| ED55-P1-28 | nested subobject/component identity没有映射层 | payload为每个可引用subobject分配portable key，provider维护object graph而非仅entity list |
| ED55-P1-29 | resource identity/ownership只在DynamicScene whole-world路径局部处理 | 建立reuse/duplicate/embed/import/resolve政策，local-to-scene资源必须按目标文档重建owner |
| ED55-P1-30 | plugin type缺失或版本不兼容时没有paste contract | link阶段按schema support window迁移或拒绝，并返回缺失provider/plugin列表 |
| ED55-P1-31 | duplicate与paste可能走不同identity语义 | Duplicate必须复用同一capture→plan→apply管线，只选择in-process source和placement policy |
| ED55-P1-32 | 跨project transfer没有trust boundary | source project/build/plugin身份只作声明，destination必须独立验证schema、资源路径与权限 |

### 6.4 Destination、placement、名称、owner、selection与冲突（ED55-P1-33～42）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED55-P1-33 | 没有typed destination context | `PasteDestination`包含document、world generation、parent/sibling/replacement target、viewport/graph position和provider mode |
| ED55-P1-34 | Scene paste没有child/sibling/root/replacement语义 | 明确Paste、Paste as Child/Sibling/Replacement与invalid target规则，支持cycle preflight |
| ED55-P1-35 | 没有placement policy | Scene支持same transform、cursor/pivot/viewport offset；Graph支持paste cursor与bounds clamp，结果可预测且可撤销 |
| ED55-P1-36 | 名称冲突没有deterministic policy | provider按目标namespace生成唯一名并把rename映射写入receipt |
| ED55-P1-37 | sibling order/stable order没有插入合同 | paste plan预分配deterministic insertion slots，redo不因并发排序漂移 |
| ED55-P1-38 | owner关系只存在于各domain临时代码设想 | provider在prepare中验证合法owner，apply后做完整owner/parent closure validation |
| ED55-P1-39 | paste后选择没有统一结果 | operation返回ordered destination roots/primary/focus target，单次替换selection并支持undo restore |
| ED55-P1-40 | 多文档Editor没有目标document authority | 目标必须来自focused document lease而非全局EditorState；失焦/关闭会使plan失效 |
| ED55-P1-41 | 当前scene edit固定使用`HistoryContextId::Global` | Editor02建立真实Document history context后，transfer必须绑定目标document transaction |
| ED55-P1-42 | 无冲突预览或typed decision | 对名称、external ref、resource、schema、owner与read-only冲突生成可审计decision set，不能在apply中弹临时问题 |

### 6.5 Transaction、Cut/Delete、Undo/Redo、journal与恢复（ED55-P1-43～50）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED55-P1-43 | Cut没有“clipboard成功后才删除”的原子顺序 | capture并验证payload，完成clipboard write receipt后，才在document transaction中删除source |
| ED55-P1-44 | OS clipboard和document mutation无法做真正单资源原子提交 | 定义可恢复saga：clipboard generation/write receipt、source precondition、delete commit和失败补偿/用户可见状态 |
| ED55-P1-45 | Delete command的local undo和durable journal混用NodeRecord | local exact batch与portable journal snapshot分开，分别测试redo、rollback、restart recovery |
| ED55-P1-46 | 多root apply中途失败的exact rollback未覆盖 | 预检全部root；mutation失败时逆序reattach batch并恢复selection/order，返回unchanged或typed partial-failure invariant breach |
| ED55-P1-47 | paste没有单一transaction scope | prepare零mutation，apply在一个document transaction中创建、link、patch、validate、select并提交 |
| ED55-P1-48 | redo可能依赖当时外部clipboard内容 | command state拥有已验证的immutable payload/plan或durable artifact引用，redo绝不重读系统clipboard |
| ED55-P1-49 | journal payload没有schema/provider/source/destination provenance | 记录format/provider/version、operation ID、identity map摘要、dependency decision和terminal receipt |
| ED55-P1-50 | crash恢复无法区分clipboard已写但cut未删、已删未记账等阶段 | durable operation phase与idempotency key进入Editor02 recovery，重启后给出完成/回滚/人工决策 |

### 6.6 Drag convergence、OS bridge、安全、性能与可观测性（ED55-P1-51～64）

| ID | 当前差距 | 需要重构的内容 |
|---|---|---|
| ED55-P1-51 | `UiDragPayload`只有kind、字符串reference和source metadata | authoring drag携带versioned transfer handle/envelope、source document/generation、domain和digest |
| ED55-P1-52 | hierarchy authoritative NodeIds存放在payload之外 | 删除`active_hierarchy_drag_node_ids`双权威，drop只解析同一transfer session中的authoritative item set |
| ED55-P1-53 | `scene://node/{id}`没有owner/generation资格 | URI仅作显示/debug reference；实际resolve必须经过qualified source lease与stale check |
| ED55-P1-54 | hierarchy pointer down立即武装drag，未消费共享drag metrics阈值 | 用统一drag state machine处理distance/time threshold、capture、cancel、focus loss与terminal disposition |
| ED55-P1-55 | project/world替换时没有明确清理active scene drag | source document close、world generation变更、window focus loss和plugin unload必须退休transfer session |
| ED55-P1-56 | reparent drag与copy/move drag没有operation语义 | payload声明Move/Reparent/Copy/Link候选，destination capability决定可接受操作并显示一致feedback |
| ED55-P1-57 | Runtime clipboard host request只支持text | Runtime11A扩展typed multi-format read/write contract，Editor55定义custom MIME与文本fallback内容 |
| ED55-P1-58 | Editor retained host没有消费Runtime clipboard host requests | 作为Runtime11A既有P0的跨owner前置条件接线；Editor55不重复拥有通用host request bridge |
| ED55-P1-59 | 没有OS clipboard ownership/change generation | 记录write generation、format inventory与external replacement，Paste每次重新读取并验证系统事实 |
| ED55-P1-60 | 不可信payload没有大小、深度、数量或解压预算 | decode前做byte/object/edge/depth/string/resource预算，使用bounded parser并拒绝zip bomb/递归图 |
| ED55-P1-61 | 自定义payload没有完整性与来源诊断 | digest只用于完整性，不作信任；记录source app/build，所有path/URI/type仍独立验证 |
| ED55-P1-62 | 大payload可能在UI线程重复serialize/deep clone | capture采用snapshot lease与budgeted worker，apply分阶段prepare但只在短事务内publish；禁止无界clone |
| ED55-P1-63 | 没有transfer metrics和failure taxonomy | 记录capture/serialize/read/decode/link/plan/apply耗时、对象/byte数、remap/omission/rollback与provider |
| ED55-P1-64 | 没有可复现receipt或diagnostic export | 每次operation生成stable ID、payload digest、source/destination generation、decision与terminal state，可导出但默认脱敏 |

## 7. P2：完成正确性之后的产品增强

| ID | 后续增强 | 前置条件 |
|---|---|---|
| ED55-P2-01 | Paste Special展示可选component/resource/reference policy | P1 typed paste plan与decision set完成 |
| ED55-P2-02 | 可选的Editor内clipboard history | 加密/隐私、byte budget、retention与project隔离完成 |
| ED55-P2-03 | 粘贴前ghost preview和placement gizmo | plan immutable且preview与commit使用同一generation |
| ED55-P2-04 | 多目标批量paste | 单目标原子transaction和identity map资格先通过 |
| ED55-P2-05 | 跨Editor实例的signed local transfer channel | OS clipboard格式、安全预算和version negotiation完成 |
| ED55-P2-06 | 结构化payload的人类可读diagnostic view | redaction与bounded rendering完成，不直接显示敏感字段 |
| ED55-P2-07 | 可撤销的Paste as Replacement diff preview | Scene diff/merge owner与transfer plan联合接线 |
| ED55-P2-08 | reference conflict可视化与交互重连 | stable subobject identity和typed reference table完成 |
| ED55-P2-09 | provider级copy/paste telemetry dashboard | P1 metrics schema稳定且默认本地、可关闭 |
| ED55-P2-10 | automation API支持payload artifact引用 | auth/capability/idempotency与artifact retention完成 |
| ED55-P2-11 | 内容浏览器与层级视图间的typed drag-copy/link组合 | Scene/Asset provider和resource policy完成 |
| ED55-P2-12 | 跨版本clipboard migration preview | schema support window、migration chain与golden corpus完成 |

## 8. 目标架构

### 8.1 分层与owner

| 层 | 目标合同 | Owner |
|---|---|---|
| Product command | Copy/Cut/Paste/Duplicate/Delete descriptor、when state、menu/keymap/context route | Editor55业务贡献，Editor08基础设施 |
| Domain provider | normalize roots、capture closure、validate、plan destination、apply与selection result | Scene归Editor55/03协作；UI/Graph/Table/Property归各domain报告 |
| Transfer service | provider resolution、snapshot lease、payload envelope、prepare/apply、receipt和transfer session | Editor55 |
| Document transaction | history context、dirty、undo/redo、journal、crash recovery | Editor02 |
| Runtime clone primitives | exact detach/reattach、component clone/serialize/remap policy、subset DynamicScene | Runtime scene owner与Runtime24 |
| UI/OS bridge | multi-format clipboard host request、platform clipboard、focus/text priority | Runtime11A/11B与App host；Editor55只定义authoring MIME |
| Drag protocol | typed payload/handle、metrics、capture/cancel/drop disposition | Runtime11A通用协议，Editor55 authoring transfer adapter |

### 8.2 两种载荷必须严格分离

`DetachedEntityBatch`是进程内、move-only、exact的ECS storage ownership。它用于Delete undo、事务rollback和同一World中的短期structural move，目标是保留实际component value、observer、tick和storage细节。它不要求稳定serialization，也不能跨版本、跨进程或跨文档长期保存。

`PortableAuthoringPayload`是versioned、bounded、provider-defined的逻辑对象图。它用于Copy/Paste、Duplicate、drag-copy、journal recovery与跨文档传输，必须通过stable type/schema、portable object key、dependency manifest和reference policy重建目标对象。它不能承诺恢复源World的内存布局或tick，而要保证声明覆盖的authoring语义完整。

把两者合并会得到最坏结果：若用`NodeRecord`式portable DTO做undo会丢内存状态；若把exact batch直接当clipboard则无法序列化、验证、迁移或跨进程。实现必须在API和类型层阻止误用。

### 8.3 建议核心数据模型

```text
CaptureRequest
  operation, source_project, source_document, source_generation
  focused_domain, ordered_selection, provider_generation, budgets

PortableAuthoringPayload
  format_id, schema_version, producer_build, domain
  source_identity, root_keys, entries, relations
  dependency_manifest, reference_table, omissions
  limits, content_digest

PastePlan
  destination_document, destination_generation, destination_mode
  provider_generation, allocated_identity_map
  reference_decisions, resource_decisions, name_order_decisions
  mutations, selection_result, diagnostics, plan_digest

TransferReceipt
  operation_id, payload_digest, plan_digest
  source/destination generation, phase timings
  created/updated/deleted roots, omissions, rollback, terminal_state
```

### 8.4 操作时序

Copy：resolve provider → freeze source snapshot → normalize roots → capture/validate closure → encode bounded payload → write custom MIME + text fallback → verify host receipt → publish clipboard generation/receipt。

Cut：先完整执行Copy并取得write receipt → revalidate source generation/preconditions → 在单document transaction中exact delete → commit journal/selection → 若delete失败保留clipboard并明确报告“copied, not cut”，不伪装全部失败或静默重试删除。

Paste：读取clipboard format inventory → bounded decode/migrate/link → resolve focused destination lease → provider prepare生成零mutation plan → 预分配全部identity并解析reference → 单transaction apply/validate/select/commit → terminal receipt。Undo/Redo只消费command-owned immutable plan/artifact，不再次读取外部clipboard。

Duplicate：使用in-process capture直接进入同一Paste plan，destination和placement来自当前document；不写OS clipboard，不另造clone算法。

Delete：preflight roots → transaction内exact detach并把batch移交command → commit selection/journal；Undo exact reattach，Redo exact detach。durable recovery同时记录portable snapshot或明确不可恢复类型并阻止提交。

Drag：pointer threshold后创建短期transfer session；payload只携带qualified handle/envelope。drop target prepare同一Paste/Move plan；cancel、focus loss、source generation变化或provider unload都会terminal retire session。

## 9. 分阶段重构路线

### R0 · 先关闭现存Delete数据损失

1. 为Delete command设计exact batch state machine，覆盖首次apply、undo、redo、partial failure和drop cleanup。
2. 增加含table/sparse/dynamic/plugin component、observer、parent/child和tick的roundtrip测试；证明值、presence、observer、order和identity都恢复。
3. 将journal recovery与local undo分开；在portable snapshot未支持某类型时fail-closed，不允许继续产生不可恢复的durable transaction。

### R1 · Runtime subset clone与portable policy

1. 建立component clone/serialize/remap registry，默认拒绝未知类型并记录owner/schema。
2. 为`DynamicScene`增加subset capture/prepare API和external reference policy，先分配全部目标identity再materialize。
3. 建立nested subobject、resource、owner与reference visitor，补顺序无关、跨World ID碰撞和schema缺失测试。

### R2 · Editor transfer service与Scene provider

1. 落地`TransferDomainProvider`、`CaptureRequest`、`PortableAuthoringPayload`、`PastePlan`和`TransferReceipt`。
2. Scene provider复用top-level selection、DynamicScene/clone registry和Editor02 transaction，支持Paste child/sibling/root及Duplicate。
3. command/menu/keymap/context/automation全部路由同一service，并实现focus与文本编辑优先级。

### R3 · Cut、OS clipboard与host bridge

1. Runtime11A/App host提供multi-format clipboard read/write、format inventory、size和generation receipt。
2. Editor写入自定义MIME及可读文本fallback，完成bounded decode、version negotiation和不可信输入防护。
3. Cut实现copy receipt→source precondition→delete transaction的可恢复saga。

### R4 · Drag convergence与跨文档

1. hierarchy drag改用transfer session，删除字符串+旁路IDs双权威。
2. 建立document lease/generation、destination context、resource/external reference政策和跨document history。
3. UI Asset、Graph、Data Table和Property provider按各自owner逐步接入，不在通用service硬编码domain。

### R5 · 规模、故障与产品资格

1. 建立10K对象/100K关系/大payload预算、异步capture、短commit窗口与内存峰值基线。
2. 注入clipboard replacement、provider unload、document close、schema mismatch、OOM/budget、apply中途失败与crash recovery。
3. 完成Windows/Linux/macOS系统clipboard、跨Editor实例、跨版本golden、辅助功能和自动化receipt验收。

## 10. 资格门

### 10.1 产品真实性（G01～G08）

1. G01：Copy/Cut/Paste/Duplicate/Delete在command、menu、keymap和context route中来自同一descriptor/capability snapshot。
2. G02：文本输入获得Ctrl/Cmd+C/X/V优先权；焦点离开文本控件后authoring provider才接管。
3. G03：无selection、read-only、unsupported type、invalid destination和incompatible payload均显示typed disabled reason。
4. G04：showcase文案不能成为能力证据；产品测试必须从真实入口观察World/document变化。
5. G05：Scene Duplicate不读写OS clipboard，且与Paste共享同一capture/remap/apply实现。
6. G06：Paste child/sibling/root/replacement的目标、order和selection结果确定且可撤销。
7. G07：所有入口返回同一operation/receipt ID，禁止keyboard与menu产生不同语义。
8. G08：provider缺失/unload时命令立即Unavailable，活动operation按明确policy完成或回滚。

### 10.2 数据完整性与remap（G09～G18）

9. G09：Delete→Undo→Redo对table、sparse、dynamic/plugin component、observer、tick、order和identity逐项无损。
10. G10：多root删除第N项失败时，前N-1项exact rollback，World和selection与开始前等价。
11. G11：subset capture只包含normalized roots及声明closure，不扫描/序列化整个World作为实现捷径。
12. G12：未知component clone/serialize policy默认拒绝并点名type/owner，不静默丢字段。
13. G13：全部目标identity先分配，交叉引用和nested subobject引用与输入顺序无关。
14. G14：source数值ID与destination现有ID碰撞时，external reference不会因数值相等误绑定。
15. G15：internal、external、resource和owner reference各有明确policy与resolution receipt。
16. G16：不可复制node/component与orphan relation被拒绝或以typed omission报告，不能伪装完整成功。
17. G17：跨文档local resource、owner和name/order在Undo/Redo后保持目标文档语义。
18. G18：payload encode→decode→migrate→encode有golden roundtrip和support-window测试。

### 10.3 Transaction与生命周期（G19～G26）

19. G19：Paste prepare阶段零mutation，apply阶段只在一个目标document transaction中publish。
20. G20：Paste中途失败不留下实体、component、resource、selection、dirty或history残留。
21. G21：Undo/Redo不重读系统clipboard，也不依赖source document仍然打开。
22. G22：Cut只有在clipboard write receipt成功且source precondition仍成立时删除。
23. G23：clipboard成功但delete失败时明确终态为CopiedNotCut，且source保持不变。
24. G24：document close/focus change/world generation变化会使旧plan/drag session fail-closed。
25. G25：crash发生在Cut/Paste每个durable phase后都能恢复、回滚或给出明确人工决策，不重复mutation。
26. G26：selection before/after/undo/redo包含ordered roots和primary，且与transaction同代提交。

### 10.4 平台、安全、性能与观测（G27～G34）

27. G27：Windows/Linux/macOS custom MIME和text fallback读写均有真实platform roundtrip测试。
28. G28：外部clipboard replacement有generation/change检测，Paste读取当前系统事实而非陈旧cache。
29. G29：byte/object/edge/depth/string/resource/decompression预算在allocation前执行，恶意payload被bounded拒绝。
30. G30：path、URI、plugin/type声明和source build均不被信任，destination独立validation无越权读取。
31. G31：10K roots/100K relations capture与plan不阻塞UI超预算，commit窗口有P50/P95/P99证据。
32. G32：内存峰值、重复clone次数和payload bytes有基线；实现不得通过whole-world复制掩盖局部操作。
33. G33：metrics按阶段、provider和terminal state聚合，不记录未脱敏authoring payload。
34. G34：receipt可导出并复现identity/reference/decision摘要，同时遵守project/privacy redaction。

### 10.5 参考与端到端验收（G35～G40）

35. G35：具备Unreal cross-actor reference同等级的default/instance/nested/交叉引用顺序无关测试。
36. G36：具备Godot同等级的top-root copy、child/sibling/replacement、cycle、owner/resource和single-action undo测试。
37. G37：具备Fyrox同等级的scene/UI隔离图、double clone与old→new root mapping测试。
38. G38：具备Bevy同等级的per-component opt-in/out、required/linked relationship、move/clone与insert policy测试。
39. G39：具备Unity Shader Graph同等级的依赖闭包、不可复制项、orphan edge、target eligibility和selection测试。
40. G40：真实Editor窗口完成Copy/Cut/Paste/Duplicate/Delete、跨文档、drag-copy/reparent、Undo/Redo、重启恢复与像素/反馈验收。

## 11. 测试缺口与建议矩阵

| 当前测试 | 已证明 | 尚未证明 |
|---|---|---|
| `editing/node_ops.rs` | Delete可撤销、最后相机拒绝、多删单transaction、selection恢复、parent-child折叠 | 任意component/observer/tick无损、partial failure、durable recovery |
| hierarchy drag source tests | 多选节点集与`scene://node`payload构造 | threshold、stale generation、focus loss、project close、payload单一authority |
| DynamicScene spawn transaction tests | whole-world capture/spawn、target remap与多种failure preflight | subset roots、external ref policy、unreflected component、cross-document collision |
| Runtime text clipboard tests | editable text copy/cut/paste/newline行为 | Editor host bridge、structured MIME、platform clipboard、format/version/budget |
| command/keymap/menu tests | 现有Delete和Undo/Redo可注册 | Copy/Cut/Paste/Duplicate capability/focus/provider/disabled reason |

新增测试必须以行为和数据完整性为主，不以“源码中出现Copy字符串”作为资格。尤其P0修复必须定义一个NodeRecord完全不知道的typed component、一个dynamic/plugin component和observer，证明Delete→Undo后实际query/callback/tick行为仍存在；只比较hierarchy snapshot会继续漏掉同一问题。

## 12. 跨报告责任边界

| Owner | 保留责任 | Editor55消费/补充 |
|---|---|---|
| Editor02 | document transaction、history context、dirty/save、journal和crash recovery | 提供transfer command state、portable recovery payload与operation phase |
| Editor03 | Scene hierarchy、selection、reparent、prefab/instance业务规则 | Editor55拥有通用scene copy/paste/duplicate编排与payload，调用Editor03 policy |
| Editor05 | Inspector property authoring/customization | property provider定义字段级copy/paste语义，复用transfer envelope |
| Editor08 | command registry、keymap、menu、palette、context routing | Editor55贡献业务command和capability resolver，不复制基础设施 |
| Editor23/24 | UI Asset、Data Table等domain工具 | 各自provider拥有依赖闭包与目标规则，Editor55负责共同transport/receipt |
| Editor42 | Scene snapshot/diff/merge/restore | 可复用semantic diff预览，但clipboard不能退化成whole-world snapshot |
| Runtime11A | UI input/dispatch/host request/drag通用协议 | 补multi-format host bridge和qualified drag contract，Editor55实现authoring adapter |
| Runtime11B | 文本editing、selection、IME和text clipboard | 保持focused text priority；不承担scene/graph structured payload |
| Runtime24 | stable identity、generation、owner epoch和stale reference | 为source/destination/transfer handle提供qualified identity基元 |
| Runtime scene | exact detach/reattach、DynamicScene、clone/serialize/remap primitives | Editor55不在UI层复制ECS internals，只编排provider和document transaction |

Editor03的P1-06、Editor23的P1-12及其他domain报告已经提到各自Copy/Paste缺口；本报告把共享payload、identity/remap、transaction choreography、OS clipboard和drag convergence收敛为canonical foundation owner，不撤销domain报告对业务语义和产品toolkit的责任。Runtime11A已登记host request被丢弃的通用产品桥问题，本报告只声明其为structured clipboard前置依赖，不重复登记新的P0。

## 13. 本轮边界与实施前重检

本轮只完成静态review、参考源码对照和重构计划；未修改production/tests，未运行Cargo、Editor、GUI、OS clipboard、跨文档、crash、soak或性能测试。MVP 00当前仍阻断功能实现，因此本报告不能作为任何实现milestone、性能资格或产品完成证据。

实施前必须重新读取当前command/intent/workbench、Delete command、Runtime detach/DynamicScene、UI host request与drag代码；重算聚焦fingerprint；核对Editor02/03/08、Runtime11A/11B/24最新owner状态；再从R0数据完整性修复开始。若source drift已经改变Delete存储或clipboard/drag产品链，应更新本报告证据和severity，而不是机械执行旧文件路径。
