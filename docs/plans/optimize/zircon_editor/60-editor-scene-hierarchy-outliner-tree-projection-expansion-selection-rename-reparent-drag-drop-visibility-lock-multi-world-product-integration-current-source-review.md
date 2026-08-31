---
title: Editor Scene Hierarchy、Outliner Tree Projection、Expansion、Selection、Rename、Reparent、Drag Drop、Visibility、Lock 与 Multi-World Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor60
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/transaction.rs
  - zircon_editor/src/core/editor_message/message/scene_inspection
  - zircon_editor/src/core/editor_event/hierarchy_host_event.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/ui/host/scene_inspection_publication.rs
  - zircon_editor/src/ui/host/editor_event_execution/hierarchy_event.rs
  - zircon_editor/src/ui/workbench/snapshot/data/scene_entry
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer
  - zircon_editor/src/ui/retained_host/app/hierarchy_filter.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_rename.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/hierarchy
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/hierarchy
tests:
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/derived_state/hierarchy_behavior.rs
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/tests/editing/history.rs
  - zircon_editor/src/tests/editor_event/runtime/integration/project.rs
  - zircon_editor/src/tests/host/retained_asset_refresh/scene_reload.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/hierarchy.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/interaction.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_projection/scene_fragment.rs
  - zircon_editor/src/tests/host/retained_hierarchy_template_body.rs
  - zircon_editor/src/tests/host/retained_list_pointer/bridge_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/scene_and_object.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99k-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99l-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerTreeItem.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerHierarchy.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Public/ISceneOutlinerMode.h
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SSceneOutliner.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/SOutlinerTreeView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/SceneOutliner/Private/ActorMode.cpp
  - dev/godot/editor/scene/scene_tree_editor.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/Fyrox/editor/src/world/mod.rs
  - dev/Fyrox/editor/src/world/item.rs
  - dev/Fyrox/editor/src/world/graph.rs
  - dev/bevy/crates/bevy_ecs/src/hierarchy.rs
  - dev/bevy/crates/bevy_ecs/src/relationship/mod.rs
  - dev/bevy/crates/bevy_ecs/src/entity/mod.rs
  - dev/bevy/crates/bevy_ecs/macro_logic/src/component.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor-PrivateShared/Tools/Converter/Window/RenderPipelineConverterVisualElement.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor-PrivateShared/Tools/Converter/Window/RenderPipelineConverterVisualElementListFilter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor-PrivateShared/Tools/Converter/ConverterState.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Hierarchy、Outliner Tree Projection、Expansion、Selection、Rename、Reparent、Drag Drop、Visibility、Lock 与 Multi-World Product Integration 当前源码工程化差距

## 1. 结论

当前Hierarchy不是静态mock。Runtime已经有generation-scoped inspection artifact、稳定的parent/depth顺序、subtree hash、稀疏name patch和selection revision；Editor publication能拒绝旧fragment并在delta缺口后请求权威reflow；rename/reparent最终进入共享transaction，批量reparent会先折叠为top-level selection，任一command失败会rollback先前command和selection。native painter只绘制viewport可见行。这些基础应保留，不能为了做World Outliner而退回临时树控件或在UI回调里直接修改World。

但当前产品链存在两条可达P0。第一，Hierarchy在primary press时立即武装可变更Scene的drag状态，不等待移动阈值、不取得pointer capture，也不建立已验证drop session；primary release只要落在另一row或root就直接发出reparent。因此一次普通press-release序列就可能变成结构修改。第二，drag、rename和事件payload只保存裸`NodeId`，World/project替换不会退休active drag、active node IDs、double-click状态或inline rename focus。旧World A中武装的release/commit在World B复用相同数值ID时会修改B。

其余差距不是“再补几个字段”能够修复。最终pane把Runtime row的kind、active-in-hierarchy、focused和has-children压成`id/name/depth/selected`；`expanded`实际被写成`has_children`且所有row仍可见；所谓virtual rows只限制paint范围，retained controls、model conversion、filter和结构reflow仍按总量工作；drag/drop又没有before/after顺序、transform保持、owner/instance/lock校验、引用修复或拒绝反馈。它应收敛为Editor41定义的typed Outliner item/provider/mode/column/filter系统，并消费Runtime110/111的权威层级和inspection产品，而不是让Editor拥有第二套World authority。

本报告登记 **2项P0、24项P1、8项P2与40个资格门**。其中Editor41 P0-05及P1-41至P1-60、Editor03 P1-08至P1-10、Editor55 P1-51至P1-56、Runtime24/109/110/111已经拥有的通用差距只作current-source细化与实施路由，跨报告汇总时不得重复计算为新的canonical owner。本轮是review-only；没有运行Cargo、真实Editor、pointer/IME、多窗口、多World、100K/1M、fault/soak或同语义跨引擎benchmark，不能据此宣称性能或表现优于Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes / ignored / dirty | 证据等级 | fingerprint |
|---|---:|---|---|
| Runtime inspection与hierarchy authority | **17 / 6,087 / 5,519 / 217,938 / 48 / 1 / 2** | E3：artifact、row builder、override、subscription、hierarchy mutation与derived state | `f76ea2321bd5e821acf3ed62b3a97a03fa329302428712d184fe0dad80411899` |
| Editor publication、projection与paint | **22 / 4,218 / 3,887 / 146,796 / 8 / 0 / 0** | E3：message、publication、fragment、retained control、pane DTO与native row paint | `a2cc11b9c7eb701ddda7775794b88e7cb1cb207f58f7fb3a3550316c10bf2d7f` |
| Interaction、command与World replacement | **26 / 3,896 / 3,605 / 137,308 / 27 / 0 / 0** | E3：pointer、drag、rename、event、intent、transaction与workspace reload | `d3f45459f8b17f483dcde8efbb09d7bb4d000b53424e07f26cc18a47de2bb6de` |
| Focused tests | **12 / 3,365 / 3,051 / 114,950 / 74 / 0 / 1** | E3静态阅读；transaction行为较强，interaction lifecycle缺口明确 | `3be76134c15ebd9bd5fff13d52997676771238b0b64be95b7adbb79dfcf16eb1` |
| Zircon去重focused set | **77 / 17,566 / 16,062 / 616,992 / 157 / 1 / 3** | E3当前working tree静态证据；未执行测试 | `a51cddd5c7e1e17ccf12bd8d0408ff17f257c6aa835d781899c59e079b079366` |
| 五引擎18个显式参考文件 | **18 / 19,930 / 17,191 / 706,227 / 49 / 0 / 0** | E2/E3：Outliner contract、interaction、reparent semantics、identity与recycled tree cells | `b72dcf98e83fceff5eba95e6353cbea0ce031e0d831be64940508088cec78bb8` |

fingerprint算法沿用Editor58/59：按normalized lowercase relative path排序，把`path + NUL + lowercase per-file SHA-256 + LF`串联后再取SHA-256。它只标识本轮读取的working-tree集合，不是ABI、artifact、cache key或动态验收receipt。

冻结Git基线为`bee4c707b714738346b49bba15c59468b8bd9b39`，coordinator baseline epoch为339。Godot、Fyrox、Bevy、Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像随主仓基线和reference aggregate fingerprint冻结。

77份Zircon语料中有3份非本轮产生的dirty文件：`scene/world/derived_state.rs`、`scene/world/hierarchy.rs`与`scene/tests/derived_state/hierarchy_behavior.rs`。本报告读取并报告当前working tree，但不覆盖这些修改；实施前必须重算fingerprint并重新验证两条P0事件序列。

### 2.2 当前产品链

```text
World hierarchy / name / active mutation
  -> SceneInspectionArtifact generation + sparse overrides
  -> SceneInspectionPublication retained latest message
  -> full reflow or hierarchy/selection fragment
  -> SceneHierarchyProjectionState entity <-> control map
  -> retained control per projected row
  -> SceneNodeData { id, name, depth, selected }
  -> native viewport-range paint

primary press on hierarchy row
  -> immediately set active_scene_drag_payload
  -> separately retain Vec<NodeId>
  -> click dispatches replace-only selection
primary release on row/root
  -> consume active payload
  -> EditorHierarchyEvent::ReparentNodes { bare NodeId }
  -> EditorIntent::SetParents
  -> transaction apply / rollback / history

F2 or ad-hoc 500 ms second click
  -> HierarchyInlineRename { bare NodeId, draft }
  -> trim + nonempty check
  -> clear focus/draft
  -> EditorHierarchyEvent::RenameNode
  -> mutate whichever World is current at execution time
```

### 2.3 已有基础必须保留

1. 保留Runtime inspection artifact的immutable generation、Arc row/index、sparse name override与ancestor hash patch。
2. 保留Latest消息缺口后的authoritative reflow和generation/selection revision拒旧，不把旧fragment勉强套到新树。
3. 保留selection独立于filtered projection、drag source使用authoritative selection以及top-level root折叠。
4. 保留Editor command transaction的apply/revert、失败rollback、selection snapshot与单history record。
5. 保留Runtime hierarchy的typed mutation入口和derived-state owner；Editor只请求命令，不直接维护parent真相。
6. 保留native visible-row paint bound，但必须明确它只解决paint工作量，不等于model/control virtualization。
7. 保留非递归深树构建与filter，避免在5K/100K深层数据上引入调用栈风险。
8. 保留source-shape守卫作为补充，但不能再用`include_str!`断言替代pointer、World replacement和规模行为测试。

## 3. 旧报告current-source校正与唯一owner

| 旧条目 / owner | 当前源码事实 | 本报告裁决 |
|---|---|---|
| Editor03 P1-08 | parent-only hierarchy、无sibling/folder/layer/visibility/lock仍成立 | Editor03拥有Scene authoring业务语义；Editor41/60拥有Outliner产品投影与交互适配 |
| Editor03 P1-09/10 | selection仍只有Edit/Play域；hierarchy click没有modifier/range anchor | per-document selection identity和range政策继续归Editor03；本报告负责Hierarchy adapter不丢输入语义 |
| Editor41 P0-05 | pane最终仍只有`id/name/depth/selected`，10K只证明visible paint bound | 保持Open；本报告提供current-source细化，不新增第二个World Outliner owner |
| Editor41 P1-41至50 | typed item/provider/mode/column/filter/expansion/context能力仍未实现 | 全部保持Open；当前row kind/active字段没有穿过最终pane，不能算Partial |
| Editor41 P1-51至60 | control总量、full reflow、name-only filter和10K paint测试仍不足 | 全部保持Open；现有pointer O(1)和visible paint只能作为局部preserve evidence |
| Editor55 P1-51/52 | `scene://node/{id}`与旁路`Vec<NodeId>`双authority仍存在 | payload统一与transfer session归Editor55；本报告不另造Hierarchy私有payload协议 |
| Editor55 P1-54/55 | press立即武装drag，World替换不退休session仍存在 | 当前release会实际提交reparent，且ID可在新World重用；提升为本报告两条可达P0 |
| Runtime24 | stable identity、generation、exhaustion与stale reference为通用owner | `u64 -> i64::MAX`投影碰撞和裸NodeId跨World问题必须由qualified identity合同统一修复 |
| Runtime109/110 | World生命周期、hierarchy/transform/participation为authority | Editor不得把folder/lock/visibility临时塞进runtime parent，也不得建立第二套derived hierarchy |
| Runtime111 | reflection/schema/inspection publication为owner | Editor消费typed inspection DTO；row字段与delta扩展应在跨模块合同中完成，不用字符串property旁路 |

当前transaction测试证明多节点cycle失败会撤销已经应用的command，因此本报告不声称“批量reparent会部分写坏Scene”。问题发生在事务之前：没有drag资格、没有qualified identity、没有drop preflight、没有完整reparent plan。修复必须保留现有原子rollback，而不是替换成UI逐节点写入。

## 4. P0：必须先关闭的当前可达错误

### ED60-P0-01 · Primary press立即武装可变更Scene的reparent，release无需形成有效drag

native hierarchy primary press先调用`hierarchy_pointer_event(kind=0, button=1)`，随后才进入click callback。press分支立即写入`active_scene_drag_payload`和独立`active_hierarchy_drag_node_ids`；没有distance/time threshold、pointer capture、window/pointer identity、source generation或drag-detected transition。primary release的`kind=2`直接消费该状态，并根据release位置向row或root派发reparent。

因此“按下A，释放到B”本身就是结构修改协议，即使用户从未越过拖拽阈值、UI从未展示合法drop decorator，也没有一个已验证的drop plan。Inspector object-field drag还会复用同一press payload，使交互owner更加含混。必须先将press变为`PressedCandidate`，只有越过平台drag metrics并成功取得capture后才能创建qualified transfer session；每次drag-over先返回typed validation，再允许release提交。click、double click、rename和drag必须是互斥且有明确terminal disposition的状态机。

### ED60-P0-02 · 旧World的drag/rename状态可在World替换后修改新World的同值NodeId

`active_scene_drag_payload`、`active_hierarchy_drag_node_ids`、`last_hierarchy_rename_click`与`HierarchyInlineRename`只持裸`NodeId`或字符串URI。`reload_default_scene`会在同一个`RetainedEditorHost`中调用`runtime.replace_world`，本轮搜索没有找到替换时退休上述状态的逻辑。event和intent同样只携`NodeId`，执行时直接作用于当前`EditorState.world`。

可复现序列为：在World A press武装drag或进入rename；切换project/scene使World B成为current；B复用同一数值NodeId；随后旧release或rename commit通过存在性检查并修改B。selection/history被World replacement重置不能封闭这条链，因为active retained-host交互状态在外层继续存活。必须让所有Hierarchy handle绑定`DocumentKey + WorldSessionId + WorldGeneration + typed object id + interaction generation`；World close/replacement在commit前先广播OwnerLost并等待terminal retirement。任何旧事件到达新World必须fail closed并生成stale receipt。

## 5. P1：工程级Outliner与authoring工作流差距

### 5.1 Identity、publication与row contract

1. **ED60-P1-01** Scene inspection message、projection state、hierarchy event、selection与active interaction没有共同的document/world/session identity；generation只是artifact generation，不能证明来自同一个World owner。
2. **ED60-P1-02** `scene_node_id`把`u64`大于`i64::MAX`的值全部clamp为同一整数，control identity会发生确定性碰撞；必须使用无损opaque key或checked admission，绝不能饱和转换。
3. **ED60-P1-03** Runtime canonical artifact调用row builder时不提供focused entity，因此`WorldInspectionHierarchyRow.focused`固定为false；Editor又在message旁路携focus。这是双authority和死字段，应由qualified selection/focus product统一。
4. **ED60-P1-04** Runtime row已有`kind`、`active_in_hierarchy`、`has_children`，最终`SceneNodeData`却只保留`id/name/depth/selected`；类型、参与状态、展开能力和focused状态在产品边界丢失。
5. **ED60-P1-05** invalid/cyclic/unreachable node会作为额外depth-0 root输出，却保留原parent字段；Editor没有invalid topology item/diagnostic，用户看到的视觉树与row关系合同矛盾。
6. **ED60-P1-06** subtree hash只覆盖display name、child count和ordered child identity/hash，不覆盖kind、active/focus等pane语义；新增字段后必须定义field revision或完整hash/currentness规则，避免错误复用旧row。

### 5.2 Typed Outliner model、projection与规模

7. **ED60-P1-07** 没有typed `OutlinerItemId`、item/provider/mode/column/filter registry；World、root、folder、entity、component、descriptor、layer、instance无法以不同能力和ownership参与同一树。
8. **ED60-P1-08** `expanded`被直接设为`row.has_children`，所有row仍保持visible。它既不是用户展开状态，也没有collapse、reveal、breadcrumb、filter前恢复或workspace persistence。
9. **ED60-P1-09** template bridge在reflow时为每个row clone并同步retained control；native paint虽只遍历可见范围，model、control tree、entity map和pane conversion仍为`O(N)`内存与工作量，不能称为真正virtualized tree。
10. **ED60-P1-10** name-only patch可稀疏更新row与祖先hash，但insert/remove/move/reorder等结构变化仍要求full reflow；没有topology operation queue、range delta、per-frame processing budget或last-known-good projection。
11. **ED60-P1-11** filter每次对完整row集合做case-insensitive name scan、clone matches与ancestors；ancestor通过depth/order反推而非parent identity，缺type/tag/component/layer/state grammar、index、cancel和query generation。
12. **ED60-P1-12** active filter强制authoritative full reflow，consumer请求完整rows时还会materialize sparse view；连续输入、远端inspection或大World下没有CPU、allocation、latency与stale-result预算。
13. **ED60-P1-13** row renderer只有text、selection与hover；没有type icon、visibility/lock、active/loaded、warning/error、dirty/source-control、instance provenance、pin或扩展列，也没有recycled cell callback解绑合同。

### 5.3 Selection、rename与input semantics

14. **ED60-P1-14** hierarchy click固定派发replace-only selection，modifier snapshot没有进入callback；SelectionModel虽能extend/toggle，产品Hierarchy无法使用Ctrl/Shift语义，也没有range anchor和filtered range policy。
15. **ED60-P1-15** selection key只含裸NodeId；没有per-document primary/ordered selection、stale prune、hidden selected count、active row或跨provider/owner限制提示。
16. **ED60-P1-16** double click只判断同NodeId且间隔不超过500 ms，不绑定window/pointer/button、空间距离或OS click count；跨World同值ID和慢帧会产生错误rename admission。
17. **ED60-P1-17** rename只执行trim和nonempty检查；没有owner/editability、保留名、同级冲突、长度/Unicode规范、source/instance政策或provider validation。
18. **ED60-P1-18** rename commit在命令成功前清除focus/draft，失败会丢用户输入；也没有pending、async validation、IME composition、error inline state、cancel reason或World replacement retirement。

### 5.4 Drag/drop、reparent plan与authoring integrity

19. **ED60-P1-19** drag以`scene://node/{id}`字符串和旁路`Vec<NodeId>`维持双authority；payload没有schema、source document/generation、operation、digest、limits或single-use session handle。
20. **ED60-P1-20** drag-over没有`ValidateDrop`/typed rejection/decorator；cycle、self、no-op、locked、foreign owner、instance boundary和unsupported target只能等事务执行后失败或根本没有检查。
21. **ED60-P1-21** drop只能设置parent或root，没有on/before/after anchor、stable sibling order、folder placement、sort-independent insertion或selection order政策。
22. **ED60-P1-22** reparent plan没有world-transform preservation选项，也没有non-uniform/negative scale、singular parent、pivot、internal node或multi-root规则；只改parent不足以定义专业编辑器语义。
23. **ED60-P1-23** plan没有source owner、inherited/instanced/editable state、lock/visibility、level/data-layer/folder约束，也不处理NodePath/对象引用、animation track、selection/context和debug live-edit修复。
24. **ED60-P1-24** 只有Runtime artifact局部counter，缺Outliner operation ID、input-to-paint latency、reflow原因、row/control峰值、filter cancel、drop rejection、stale event、rollback和World replacement retirement指标；现有测试也没有覆盖pointer状态机、跨World复用ID、rename failure draft、100K/1M churn与真实窗口反馈。

## 6. P2：主线完成后的诊断与扩展缺口

1. **ED60-P2-01** `expanded`命名掩盖`has_children`事实；hard cut后禁止保留同名compat字段或把旧布尔解释成真实展开状态。
2. **ED60-P2-02** hierarchy callback仍依赖数字event kind、control ID与URI literal；迁移到typed input/drop contract后删除字符串桥，不保留双写。
3. **ED60-P2-03** 缺远端PIE/server World并排比较、cross-world selection bridge和read-only runtime hierarchy模式；依赖qualified identity完成后再做。
4. **ED60-P2-04** 缺saved collections、smart folders、bookmark sets、自定义grouping与团队共享filter preset；不能先用runtime parent模拟。
5. **ED60-P2-05** 缺multi-user selection/presence、hierarchy edit lease、conflict marker与change review；必须建立document/operation identity后再接入。
6. **ED60-P2-06** 缺million-item background indexed query、paged unloaded descriptor和分布式World Partition provider；Editor41/16共同拥有规模边界。
7. **ED60-P2-07** 缺可导出的、默认脱敏的Outliner interaction receipt与replay corpus；payload、名称、路径和用户信息必须受redaction policy约束。
8. **ED60-P2-08** 缺headless Outliner query/validation API与automation command；它只能消费同一provider/plan，不能另写一套脚本树遍历authority。

## 7. 五引擎参考约束

| 参考 | 当前源码证据 | 对Zircon的硬约束 |
|---|---|---|
| Unreal Scene Outliner | `ISceneOutlinerTreeItem`有stable ID、expanded/filtered/interactive/sort flags、visibility/pin/rename；Hierarchy provider负责create/children/parent和change event；Mode负责parse drag、ValidateDrop、OnDrop、filter/selection/rename/context政策；`SSceneOutliner`按pending add/move/remove和frame budget填充，按ID恢复expansion；TreeView用`DetectDrag`后才创建operation | press不能直接成为drag；item/provider/mode/column/filter必须是typed extension boundary；drop必须先验证并展示reason；增量op与展开状态必须跨filter/sort稳定 |
| Godot SceneTree | scene root ObjectID变化会reset cache/current scene；visibility drag有viewport threshold、Escape/right-click cancel和UndoRedo；tree显示icon、warning、lock/group/visibility/processing；dock reparent验证foreign/inherited/instanced节点、before/after顺序、transform、owner/name/reference/animation并在一个undo action中提交 | World replacement必须退休旧interaction；reparent是带owner/order/transform/reference政策的plan，不是单字段parent写入 |
| Fyrox World Viewer | provider抽象root/children/parent/name/icon/instance/selection/validation/mutation；UI增量remove/add/reorder并保留handle map；filter保留祖先；per-scene expansion、breadcrumb、locate selection；DropAnchor区分OnTop与Side | Zircon应复用provider + stable handle + anchor，不把所有对象压成Entity row；展开和落点是per-scene产品状态 |
| Bevy ECS hierarchy | `ChildOf`是关系source，`Children`为自动维护target且collection保持private；Entity由index+generation组成，stale generation读取被拒；宏层阻止公开可直接改写的relationship target | Editor handle必须携generation并拒绝stale；Runtime parent/derived children不能被任意generic mutation或Editor副本改写 |
| Unity Graphics本地corpus | 本地镜像不含Unity proprietary Scene Hierarchy源码；Converter工具仍展示`MultiColumnTreeView`、root item refresh、recycled cell bind/unbind、composable status filter和serialized expansion/selection/filter state | 只能把这些作为通用tree UI证据，不能虚构Unity Scene实现；recycled cell必须清理旧callback，view state应显式持久化 |

Unreal并不自动证明其每个选择都适合Zircon，Godot/Fyrox也各有历史包袱。采用它们的原因是这些源码共同证明了相同工程边界：qualified item identity、provider-owned hierarchy、drag detection、drop preflight、stable expansion、transactional reparent和World lifecycle retirement。目标不是复制类名，而是补齐这些不可省略的责任。

## 8. 目标架构与hard cut

```text
zircon_runtime::scene
  SceneGraphAuthority
    WorldSessionId / WorldGeneration
    typed relationship mutation transaction
    hierarchy + participation + inspection delta
    invalid-topology diagnostic

zircon_editor::scene
  SceneDocumentSessionRegistry
    DocumentKey -> authoring World lease / history / selection / view state

  WorldOutlinerRegistry
    OutlinerProvider
    OutlinerItemId { provider, owner scope, stable local id, generation }
    OutlinerMode / Column / Filter / ContextAction
    OutlinerProjectionGeneration

  OutlinerViewState
    expanded IDs / active row / range anchor / scroll anchor
    filter + sort + columns + saved preset

  HierarchyInteractionController
    Idle
    PressedCandidate(pointer, source, metrics)
    Dragging(TransferSessionLease, capture)
    RenamePending(RenameSessionLease)
    Terminal(receipt)

  ReparentPlanner
    validate owner/instance/lock/cycle/no-op
    resolve on/before/after and sibling order
    transform/reference/selection policy
    immutable plan -> one Editor transaction -> receipt
```

Hard cut规则：

1. 删除`scene://node/{id}`加旁路IDs的双authority；旧payload不保留compat decoder。
2. 删除`expanded = has_children`假合同；真实expansion由`OutlinerViewState`拥有，Runtime artifact不保存per-user展开状态。
3. 删除`u64 -> i64::MAX`饱和投影；所有control/item identity必须无损或在进入产品前checked拒绝。
4. 删除World无关的active drag/rename状态；所有interaction都必须持owner lease并在World transition前terminal retire。
5. 删除按总item数clone retained node的“virtual rows”路径；row/cell pool以viewport + overscan为稳定上限。
6. 不新增`pub use`旧Hierarchy DTO、shim event或第二套folder/visibility authority；调用点一次迁移到typed contract。

## 9. 分层实施顺序

| Milestone | 交付内容 | 退出条件 |
|---|---|---|
| ED60-M0 | P0交互止血 | press不再武装mutation-capable drag；World transition能同步retire drag/rename；旧event fail closed |
| ED60-M1 | Qualified identity与document owner | Document/World/session/generation/item key贯穿message、selection、event、intent、receipt；clamp路径删除 |
| ED60-M2 | Typed Outliner contract | item/provider/mode/column/filter/context contract接入Entity provider；最终pane不再丢kind/active/children/diagnostic |
| ED60-M3 | View state与input state machine | expansion、active/range/scroll anchor持久化；drag threshold/capture/cancel；rename validation与draft recovery |
| ED60-M4 | Reparent planner与transaction | drop preflight/decorator、on/before/after、order、transform、owner/instance/lock/reference policy及单transaction receipt |
| ED60-M5 | Incremental projection与bounded row pool | topology range delta、frame budget、viewport+overscan pool、indexed/cancelable filter、last-known-good projection |
| ED60-M6 | Observability、fault与规模资格 | 100K/1M、World churn、filter/sort、multi-window、fault、soak、memory/latency/receipt全部通过 |

依赖顺序不能颠倒。M0先阻止误操作和跨World写入；M1建立身份后，M2/M3才能安全持久化view与interaction；M4需要Runtime110/Editor03的authoring政策；M5不得用whole-tree clone掩盖模型缺口；M6通过前不能宣称World Outliner production ready。

## 10. 资格门

### 10.1 M0/M1：安全、身份与生命周期

1. **G01** primary press + 未越过threshold + release到另一row不会产生reparent command。
2. **G02** drag只在平台metrics满足并取得qualified pointer capture后进入Dragging。
3. **G03** Escape、right-click cancel、focus loss、capture loss、window close均产生一个terminal cancel receipt。
4. **G04** source document close、World replace、project switch、plugin unload会退休active drag与rename。
5. **G05** World A事件在World B复用同值NodeId时被stale rejection，B authority零变化。
6. **G06** message、selection、event、intent、plan和receipt都携同一document/world/session generation。
7. **G07** 全`u64`范围item identity无碰撞；越界adapter checked失败，不允许saturation。
8. **G08** interaction terminalization幂等；重复release/cancel不会提交第二个command。

### 10.2 M2/M3：模型、视图与交互

9. **G09** Entity、root、folder、component、unloaded descriptor可由不同provider产生stable typed item。
10. **G10** provider change event可表示add/remove/move/reorder/update并带projection generation。
11. **G11** row kind、active、has-children、warning和owner scope无损穿过Runtime/Editor/pane合同。
12. **G12** invalid/cyclic/unreachable topology显示typed diagnostic，不作为无说明的普通root。
13. **G13** expand/collapse、expand all、reveal、breadcrumb和workspace restore通过。
14. **G14** filter/sort前后按item ID恢复expansion、active row、range anchor和scroll anchor。
15. **G15** Ctrl toggle、Shift range、filtered hidden selection和跨owner限制行为有明确测试。
16. **G16** double click按window/pointer/button/position/OS count判定，不跨World或跨control误触发。
17. **G17** rename验证read-only/lock/instance/name conflict/length/normalization并显示typed reason。
18. **G18** rename失败保留draft/focus；IME composition不被commit/cancel错误截断。
19. **G19** visibility、lock、active/loaded列调用真实provider transaction并支持undo/redo。
20. **G20** recycled row/cell重新bind时注销旧callback，不向已回收item发送操作。

### 10.3 M4：drop与authoring integrity

21. **G21** drag payload只有一个qualified transfer authority，无字符串URI加旁路ID双写。
22. **G22** drag-over对每个target/anchor返回Accept或typed rejection并更新decorator。
23. **G23** self、cycle、no-op、locked、foreign owner、read-only instance在commit前被拒绝。
24. **G24** on/before/after落点与stable sibling order在filter/sort视图下仍按source authority解释。
25. **G25** keep-world/keep-local transform政策覆盖non-uniform、negative与singular parent。
26. **G26** 多选含父子时只移动normalized roots，selection order和relative sibling order确定。
27. **G27** owner、instance provenance、NodePath/object reference、animation track和selection修复有显式receipt。
28. **G28** 任一plan/apply失败回滚全部node、order、transform、reference和selection。
29. **G29** undo/redo不重做drop hit-test或读取当前selection，而使用冻结的immutable plan/command state。
30. **G30** large reparent有item/time/byte预算、进度和cancel policy，不在UI callback中无界执行。

### 10.4 M5/M6：规模、故障与产品资格

31. **G31** retained row/cell对象数上限为viewport + overscan，与总item数无关。
32. **G32** insert/remove/move/reorder只触及affected range/ancestors，不做无理由full reflow。
33. **G33** filter使用indexed fields、query generation和cancel；过期结果不能覆盖last-known-good projection。
34. **G34** 100K loaded item的memory、first paint、scroll、selection、filter和1% churn满足冻结预算。
35. **G35** 1M loaded/unloaded mixed item用paged/lazy provider完成检索与reveal，不materialize全部retained rows。
36. **G36** multi-window/multi-document并行Outliner的selection、expansion、drag、rename和history互不串扰。
37. **G37** subscription gap、provider panic/error、World teardown和OOM admission有degraded state与恢复测试。
38. **G38** operation metrics包含stage、generation、latency、item count、reflow reason、rejection和terminal state且默认脱敏。
39. **G39** 真实Editor窗口完成click/range/rename/drag/drop/visibility/lock/filter/undo/World switch像素与反馈验收。
40. **G40** 与冻结Unreal/Godot/Fyrox同语义场景做可复现benchmark；只报告同hardware/workload结果，不作口号式优越声明。

## 11. 验证矩阵

| 层级 | 必须新增或修正的验证 | 当前状态 |
|---|---|---|
| Unit | qualified item identity、state transitions、drop anchor、rename validation、top-level normalization、transform policy | 缺失；现有pointer `drag.rs`没有独立状态机测试 |
| Property / fuzz | random tree move/cycle/order、stale generation、payload decode limits、Unicode rename、event reorder/duplication | 缺失 |
| Transaction | multi-node move + order + transform + reference + selection atomic rollback、undo/redo | parent-only cycle rollback已有强基础；完整plan缺失 |
| Integration | publication gap、World replace、project switch、provider reload、multi-document、multi-window | fragment/reflow局部存在；interaction lifecycle缺失 |
| UI | threshold/capture/decorator、invalid reason、inline rename draft/IME、expansion/filter restore、recycled row | 缺失；大量测试是source-shape或静态pane投影 |
| Scale | 100K/1M memory、filter/sort/scroll/churn、row pool、latency percentile | 只有5K深filter和10K可见paint局部证据 |
| Fault / soak | provider failure、subscription gap、World churn、focus/capture loss、long edit session | 缺失 |
| Comparative | 同场景Unreal/Godot/Fyrox workload、硬件、版本、采样和结果归档 | 未建立 |

## 12. 审查限制与完成定义

本轮没有修改production Rust、ZUI或测试，只新增review文档与索引记录。没有运行Cargo是有意的：当前Goal仍处于MVP baseline `00` review阶段，本报告没有实现候选，静态测试数量不能转写为动态通过。也没有启动Editor、注入pointer/capture、执行World replacement、检查真实像素、运行100K/1M或跨引擎benchmark。

`review_status: complete`仅表示本轮77份Zircon focused set、18份参考文件、父报告owner与当前工作树事实已经形成可执行差距记录。它不表示Editor41完成，不表示两项P0关闭，也不表示World Outliner production ready。只有40项资格门全部有可复现evidence artifact，相关parent owner状态同步关闭，并重新冻结无冲突current-source fingerprint后，才能把`implementation_status`改为complete。
