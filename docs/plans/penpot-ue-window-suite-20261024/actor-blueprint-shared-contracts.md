# Actor / Blueprint：共享消费者合同

## AB-PF-27：保留森林与原展开状态的单一可见行查询

[PrefabSourceReadSnapshot::visible_filtered_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:287)组合原filtered路径与原view展开predicate；它在创建迭代器前对每条native row匹配一次，消费时才以完整u64查询可见保留分支。父自身匹配但全部子项被裁掉时has_children=false，不查该叶展开；隐藏/裁掉的分支也不查询。全匹配与原visible_hierarchy_rows等价，两入口共用原域内私有遍历，无caller自填source_index/层次输入或第二状态。

对应UE[filtered child query](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2045)与[SetNodeExpansionState](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:3071)的数据/展开分离；[搜索回调](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2224)自动展开、选择端点及root fallback仍是宿主操作，不能据此方法声称实现，也不能套AB-PF-25仅收起规则。当前函数只应用已有展开状态。

即使筛选只显示一个重复name/ID记录，entity_by_id仍Ambiguous；positions只属本次read，不能变持久target。迭代器在来源移除/同revision重建后仍保留原数据，原expected key重验拒Stale；Ready-empty不调用matcher或展开。caller须在读前与消费发布前核原view incarnation/project/document/selection/request/source及原key，仅退休旧view，禁止fresh-key重绑旧回调/清替换后的新上下文。实体parent不是component attachment；匹配/展开/选择/焦点/滚动仍由原host持有，不增store/clock/history/trait/producer。

[两项回归及字节审阅](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-visibility-35/semantic-review.json)当前42定义（41普通+1原ignored）/0 verified Rust/actual null，原02 accepted19不覆盖；真实per-view materializer、typed TreeRow输入、正式导出及EXE验收仍pending。


## AB-PF-26：来源筛选、UE搜索与原context/key

入口[PrefabSourceReadSnapshot::filtered_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:258)只读原native来源。caller的matcher收到每条borrowed PrefabSourceEntityRow（含完整u64/原name/opaque components/父诊断）一次，原序匹配；输出既有hierarchy row，保留命中及有效祖先，重算留下的直接children，不保留父命中下的无关后代。missing/ambiguous/cyclic边不传播，duplicate entity仍Ambiguous；source_index只属原read，不是持久itemKey。

UE[FSubobjectEditorTreeNode::RefreshFilteredState](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectDataInterface/Private/SubobjectEditorTreeNode.cpp:262)和[ChildMatches传播](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectDataInterface/Private/SubobjectEditorTreeNode.cpp:187)支持这一数据投影；[搜索回调](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2224)另外展开可见祖先、选择靠前分支端点，无候选时fallback RootNodes[0]。因此不能把AB-PF-25的“仅收起后再展开不自动选回”推广成搜索规则。本方法不作这些副作用；Parent原选择/展开/焦点消费须以真实唯一native target为准，不按序号选择歧义记录。UE的root Actor特例与UActorComponent IsChildOf不等于通用entity root或provider字符串，仍需真实模型。

Ready-empty为零输入resident查询，过滤零命中也是零行，但均非NotResident。旧read保留原数据；remove/recreate即便同数值revision仍拒原key。消费前与发布/清旧Details前核原view incarnation/project/document/selection/request/source及原expected native key，不fresh-key重绑旧回调，只退休原view，不能清替换后的新上下文。caller继续持有匹配/展开/选择/滚动状态，不建第二store/clock/history/DTO、公共trait或fixture producer。实体parent不是component attachment。

[三项回归与完整静态定义](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-filter-34/semantic-review.json)当前40=39普通+1原ignored、Rust0/actual null；原02 accepted19不覆盖新源，正式运行/生产publisher/materializer/搜索输入和EXE证据仍pending。本轮只交可消费的方法和业务边界。


## AB-PF-25：隐藏选择与来源Details的真实边界

UE[STreeView](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h:161)默认AllowInvisibleItemSelection=false；[重建线性列表](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h:635)比较实际可见选择、移除隐藏项并发ESelectInfo::Direct，[PopulateLinearizedItems](../../../dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h:741)只把实际可见的选中项放回新集。SSubobjectEditor构造未覆盖该选项；[选择回调](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2149)转发TreeWidget实际选择集。因此收起隐藏的原选中项后，重新展开不会自动恢复旧选择。这里不是声称全部Toolkit的Inspector内容都清空；本族readonly来源视图用明确的“未选择实体”提示。

[原reader32](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:234)仍仅返回本次source的可见hierarchy_rows，不变更选择/焦点/滚动、不建状态。Parent原caller应在相同read/context中对实际可见且能唯一定位的目标重核选择；隐藏目标退出选择时一并退休其旧Details，展开本身不造选择，明确点击后才选中。重复entity ID仍Ambiguous，不用source_index/文案补target；entity parent不是component attachment。

读前与发布/清选前继续核原view incarnation/project/document/selection/request/source及原native key；context或key失效只处理原旧view，不能fresh-key收养旧回调或清替换后的新上下文。选择通知与source/tree/Details须由原publisher保持同源，具体typed materializer、取消/retention/键盘焦点仍归原host，不扩公共trait或第二store/clock/history。

[设计33](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-tree-visibility-33/result.json)以source32单元输入验证原生selected→collapsed-none→expanded-none→明确再选及unselected→collapse，三宽四边实际可达，12导航保存。窄窗Details开关可达但BODY焦点；disclosure当前14×14路径paint-only命中及旧relative-trigger继承字段需原public/codec owner接续。仅设计与原reader语义，不是生产producer、写权限或Rust/产品通过。


## AB-PF-24：原来源森林与caller展开状态的边界

入口为[PrefabSourceReadSnapshot::visible_hierarchy_rows](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:234)。caller须把predicate绑定到原source身份和原view展开状态；该函数只传完整u64实体ID，不提供跨source全局键。由私有resident source重新构造已有lossless forest，输出原PrefabSourceHierarchyRow，保持root/sibling原序和异常根；source_index只属本次read。隐藏后代与叶不查询展开，不改选择/焦点/滚动，选中行隐藏时的保持及实际控件事件仍由原host实现。

此为实体parent森林的可见性，不创建attachment/root/reparent或递归展开动作。重复ID不能作为无歧义target；切环根仍保留其唯一有效子节点。Ready-empty仍resident，NotResident为另一个读取结果；旧read在同ID/revision remove/recreate后仍保持原数据，必须用原key及原view incarnation/project/document/selection/request/source在读前/发布前核验，不fresh-key收养旧回调，仅清原view。无producer/materializer/第二store/clock/公共trait/事务写入。

UE数据与展开分离依据为[SSubobjectEditor child query](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:2045)和[SetNodeExpansionState](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:3071)，不把UE component tree与Zircon entity forest等同。三项未执行回归与精确源在[审阅32](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-visibility-32/semantic-review.json)；当前37静态定义、正式Rust/插件/EXE与export/import均open，原accepted19不覆盖。

## AB-PF-23：从原来源read投影真实实例行

入口是[PrefabSourceReadSnapshot::instance_authoring_snapshot](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:156)。原caller已从自己的project/document/view取得typed PrefabInstanceAsset和source read后，传入该实例及同生命周期registry/resources。方法解析实例自己的prefab引用，再以原self.version_key调用既有expected reader；UUID迁移按原registry权威处理，异源/异manager/旧key不采用fresh替换。返回既有PrefabInstanceAuthoringSnapshot，borrow生命周期同时受read与instance约束，没有新store、clock、public trait、producer或handler。

Available只说明本次resident检查成功；保留原序override行、所有重复path值及opaque JSON，诊断仍只含原空路径/重复path检查。PrefabPropertyOverrideAsset只有entity_path/property_path/value，未见已确认的路径→native component/property语法，本入口不拆Transform字符串、造component identity或把这些路径当绑定/Graph定位。解析/类型/旧key错误原样返回，不能转成Missing/Ready-empty；初次read的NotResident与后续expected失效Stale分别消费。

Parent仍在原view incarnation/project/document/selection/request/source与原native key读前/发布前同源核验；本方法不是publisher、lease或事务准入。旧回调拒绝只清对应旧view，不清已替换新context，不用fresh key重绑。原raw constructor供未知/离线状态继续保留，不扩通用history/CAS。源回执与三项未执行回归见[审阅31](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-instance-source-31/semantic-review.json)，正式Rust/实际caller/EXE均open。

历史29/30的line967按各自source SHA保留；当前同一component-conflict回归已移至[当前line1145](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:1145)，不能把旧静态定位或旧19项payload当当前源码验证。

## AB-PF-22：来源组件描述歧义的原位置

[PrefabSourceComponentConflict](../../../zircon_plugins/prefab_tools/editor/src/authoring/snapshot.rs:124)由 `PrefabSourceReadSnapshot::authoring_snapshot()` 返回的 `PrefabSourceAuthoringSnapshot.component_conflicts` 携带。`entity_source_index` 指向该snapshot原entity_rows/scene.entities，`component_source_indices` 指向同一entity原components，含首次与每次重复位置；完整原记录可从原borrow直接查看。组按序列化owner位置、首次component位置排序，不按显示名称、实体ID大小或descriptor字典序重排原树/记录。

分组复用 `PrefabSourceComponentDescriptor::from(record)` 的type_id/schema_id/schema_version/provider_id完整相等；不trim或从payload/reference/name猜实例，未知provider保持opaque。每个序列化owner独立建临时索引，因此同名、重复entity ID的记录不跨owner合并；原DuplicateEntityId诊断与entity_by_id拒绝仍保留。这是“无法仅凭描述选择唯一记录”的诊断，不宣布这些组件或Prefab非法，不删除/合并记录。

原Parent typed materializer可显示本地化提示“多个来源记录具有相同组件描述；无法仅用描述确定目标”，并用本次borrow位置标注所有相关行。位置与组都不是持久row identity、场景组件attachment或mutation target；模型未提供instance ID时不得给重复组派发Apply/Delete。显示前与发布/清选前继续验证原view incarnation/project/document/selection/request、source和expected native key，不能用fresh key重绑旧回调，不能清替换后的新上下文。

[三项回归定义](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:967)覆盖不同payload/reference全部保留、Z/A首次顺序、四字段精确匹配（u32MAX/中文/尾空白）、同ID独立owner、空来源、同数字revision重建后的旧key拒绝与旧/新组分离。当前31定义/0执行；格式/guard见[审阅29](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-component-conflicts-29/semantic-review.json)。无公共trait、producer、schema/store/clock或原31节点ZUI改动；实际per-view调用者及运行显示仍由原owner接续。

## AB-PF-21：opaque组件描述的原只读解析

原 `PrefabSourceReadSnapshot::component_by_descriptor(entity_id, descriptor)` 接收完整u64及 `PrefabSourceComponentDescriptor`，后者只含原 `SceneComponentAssetRecord` 的 `type_id/schema_id/schema_version/provider_id`。可从原记录的 `From<&SceneComponentAssetRecord>` 借用，不复制payload或references、不分配UUID、不新增schema/store/clock。

返回 `PrefabSourceComponentLookup::{Unique(&record), MissingEntity, AmbiguousEntity, MissingComponent, AmbiguousComponent}`。比较精确字符串及整数，不trim、大小写折叠、按名称/位置回退或只比type。重复实体即AmbiguousEntity；同实体内完整描述重复即AmbiguousComponent，所有原记录仍留在原来源。Unique只支持本次source read的只读Details；未知provider/字段仍opaque，不能据此声称字段可编辑或运行中的组件已解析。

Parent原caller先核view incarnation/project/document/selection/request与原expected native key，再由同次保留读解析实体和组件。延迟发布须重验同一上下文及原key；即使entity/描述/数字revision全相同，remove/recreate的native key仍不同，不能用fresh read授权旧回调。描述是定义字段组合，不是模型尚未提供的组件实例identity，也不能充场景Delete/Undo或Prefab写事务target。失效只退休原view旧展示，不清替换后的新上下文。

[实际实现](../../../zircon_plugins/prefab_tools/editor/src/authoring.rs:170)、[三回归定义](../../../zircon_plugins/prefab_tools/editor/src/tests/authoring_snapshot.rs:771)及[结果27](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-components-27/result.json)提供可消费API；原host/typed materializer/生产publisher仍由Parent接续，本域不改公共trait或注册fixture producer。28定义无Rust回执；原02 accepted19项和全部历史失败保持，不因本次修订自动重投或建第三身份。

## AB-PF-20：源实体森林到原树materializer的只读合同

[原snapshot](../../../zircon_plugins/prefab_tools/editor/src/authoring/snapshot.rs:92)新增 `hierarchy_rows: Vec<PrefabSourceHierarchyRow>`。每行的 `source_index` 只定位同次 `entity_rows`，`depth` 是无显示上限的usize层级，`has_children` 只统计唯一且未切断的Resolved父边。根/同级沿源序列排列；Missing/Ambiguous/Cyclic作为可诊断的根显示，其有效子项仍保留。空来源为零行，不造“默认根”；实体parent不授予组件Make Root/Reparent能力。

原调用者先以原native expected key和真实view incarnation/project/document/selection/request核验来源，再在保留的 `PrefabSourceReadSnapshot` 上调用 `authoring_snapshot()`。按 `hierarchy_rows` 遍历，以 `entity_rows[row.source_index].entity` 取完整源ID/名称/组件；source index不持久化，也不作为操作target。稳定身份仍来自原document/source/完整u64实体ID及真实模型身份；重复ID必须维持原 `entity_by_id` 的Ambiguous结果，不能选第一个或按文案重绑定。

Parent保留原typed row materializer、组件实例、展开/选择/滚动状态、source→view投影与发布前同源核验；本域只交树语义，不另建Tree store/clock/producer。旧结果失效仅退休原view的旧展示，不清替换后的新上下文。本次新增两项源回归并扩展原深层/空树测试，25定义仅静态检查，未形成Rust、生产者或真实窗口通过证据。

## AB-EXPORT-01～03：原canonical模块的可逆导出边界

[逐节点源接入15](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-export-map-15.json)基于独立API Actor107/Shared5及原canonical31节点逐一对应。六板仅为语义设计，role/bridge-version/document缺失，全文件没有原模块sourcePath归属；有效Shared leaf sourceRef不是原模块node/control/instancePath。plugin.ts的selectedAssetBoard/导出前置与zui-source-export.ts的owner/node edit守卫必须保留，不能给设计板贴假metadata、重签或强行改component类型。

原7widget导入只有Button和PropertyEditorRow的精确canonical URI已对应Shared；另5项为WorkbenchDropdown、WorkbenchField、WorkbenchListRow、WorkbenchSectionTitle、WorkbenchTableRow。TreeRow/Search是不同source，不能按外观/同名映射UUID或作为Property required value替代。原public owner交真实缺件/slots/版本/依赖与qualified canonical导入上下文，本域在自己的文件执行导入并记录身份；Parent接有必要的源组件迁移，不复制公共main，也不授公共owner跨写Actor。源文件所有29controls/15events、7imports/slot和token表达式必须保留在合法导出链。

原源根是模块body Overlay+rail/right-reserve/compact drawer，设计根含document chrome且Details topology不同；Root拥有DocumentHost/Dock/chrome边界。首个可执行导出步骤是带原sourceHash/owner-map的canonical模块导入，先证明未改roundtrip，再投真实布局/样式delta。当前语义表不是export manifest，本域未运行SDK导出/Node测试或改公共codec。

## AB-PF-18：projection patch与真实可变行materializer

EditorUiTemplatePaneDataSnapshot仍是values/component_patches；UiComponentProjectionPatch在runtime_interface/ui/component/data_binding/projection_patch.rs仅有control_id、attributes、state_values，不是任意新增/删除行API。原Prefab模块source没有repeat，左3/中心4行是固定fixture。原有虚拟行类在实际物理路径ui/retained/dispatch/templates/virtual_rows.rs，pub(crate) TemplateBridgeVirtualRowSequence::from_surface_repeat要求已有repeat和authored_count=1，再由reconcile_with_keys协调运行时pool；component_property_rows实际物理文件位于dispatch/workbench，不在templates/workbench。它是可复用的Parent私有实现，不授予插件直接调用或广开pub。

Parent须将真实每view model与原source key/context接入现有row materialization/稳定key与事件定位。域内source_index仍仅为当前借用位置；重复entity ID/opaque同类型多条保留为各自诊断/记录，不选首项伪造持久编辑目标。需要源repeat/prototype/typed属性的具体改动先按Root所有权交接，不在本插件新增平行row schema/store或冻结公共源。

## AB-PF-19：TreeView摘要不能代替完整Prefab树

实际ui/retained/contract/paint/tree_view/content.rs将collection_items解析为字符串摘要后take(8)，depth夹0～3；TreeItem仅depth/label/selected/expanded，没有persistent item identity。它不是31节点原资产正在使用的ListRow子树，也不是Prefab实体树/组件attachment或typed operation backend。不能把更多源记录转换为带竖线的label来关闭首尾滚动、深层层级或稳定选择门。真实Scene hierarchy/Runtime虚拟行材料可由Parent复用，但World命令/模型不能因此混入Prefab来源事务。

原4213/r03 host正常截止退出及其Job/EOF只属于该旧发行物HTTP服务，当前source未构建/正式owner导出未交付；不借它作当前codec或产品通过，不恢复服务。以上只读合同核查不改4Rust/原ZUI/Shared或公共接口，16定义0执行，所有产品门继续pending。

## AB-SHARED-09：来源空态消费既有实例与publisher失效合同

[设计14](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/result.json)只在Actor原file/page新增两状态各三板，消费原Shared rev5的Button/Search及原本域SourceReferenceData；没有新公共组件、master或theme。18个新增Shared native heads和每个copy/component/main/source/shapeRef/touched见[原生身份](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-status-14/save-reopen-preservation.json)，当前58个Shared roots/653个总引用逐项有效，原802非根对象及两本域main精确保留。Shared data/4main/34+3未写，字体provider未盲替换。

通用Button unavailable应由真实host capability/合法props体现；本轮保留灰色不可用文案但没有制造未发布enabled字段/handler。空/不可用设计中不再提供旧Details overlay入口或静态Load/Retry。640说明断词仍open；source-owner导出仍按原AB-SHARED-07/08闭包保留事件、绑定及原资产稳定ID，browser截图不能替代exporter或normal EXE。

Parent原per-view producer接线缺口不变：当前no-context snapshot和template-keyed batch不能证明某个view/document的来源。本域提供的只读prefab_source_snapshot及PrefabSourceAuthoringSnapshot消费原registry/resources/key；Parent负责真实生命周期/context/selection/retention与原native key发布前重校验，失效时清空旧树与Details/选择，保留Reference/NotResident/wrong-kind/rejected诊断。禁止global active猜测、固定fixture producer或用source_index/label/creation Arc作持久target；不要求扩trait/store/时钟。Graph/VM/Definition/WG边界按现有AB-WG交接继续open，不借中立图表面当执行后端。

源08/review09、已选02和原#348不变，16定义0执行。本轮只推进可编辑状态内容/重开/逐图设计证据，不改公共Rust、原ZUI、共享主题/目录或服务/桌面；正式source-owner导出/运行/EXE/150%DPI/双端≤1DIP仍pending。


## AB-SHARED-08：消费module_details，保留focus/字体/export缺口

[设计12](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-compact-details-12/result.json)从原ZUI及现有responsive_layout/toolbar_layout/componentized_window/reference_menu_actions核对已有compact drawer，640/900消费它，不造公共Drawer/Tab。两新Shared Button native copies引用原component6ddc248b-bd35-80c5-8008-d6c8c01096f3/main6ddc248b-bd35-80c5-8008-d6c8bd9b057f/rev5，“详情”文字与toggle-overlay只覆盖自有copy。Shared主件/34+3样式/data未改，两本域main身份不变；普通内容flex，仅真实弹层按flow目标/源frame定位。

Parent消费现有WorkbenchModuleDetailsDrawerToggle / Workbench/ToggleModuleDetailsDrawer / workbench.module.details_drawer.toggle，在真实Actor视图/能力/生命周期证明控件、focus和per-view publisher。本域不扩公共trait/history或借graph/VM callback作backend。Penpot代理g无role/tabindex，click成功不关闭键盘/focus合同；小fixture完整可达不代表真实空/大数据/scroll/EXE通过。

公共owner沿原合同接source-owner导出、TreeRow灰底/叶disclosure/action和合法Property.value。Native SVG呈serif样字形，与画布不同；两context报Roboto400/700 loaded/500 unloaded仍不证明实际glyph/provider/CJK相同，不盲套Typography或改Shared时间戳。未发明UUID/重复主题、detach或绕过export guard，不计全局A/B/Engine验收。

本文件向 Parent、材质图表面 owner、动画时间轴 owner 交接实际需求。它是接入合同提案，不是新 Rust trait、公共组件实现或已发布的 Penpot library。沿用 [具体设计](actor-blueprint-design.md)、[开放差异](actor-blueprint-differences.md)；公共框架/主题/catalog、导出和正常捕获协议仍归 Parent 集成。

## 当前源码修订与Jenkins身份边界

[后继入口对账13](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-successor-route-reconcile-13.json)明确时间边界：原Jenkins owner22:27 a928的真实pure parse拒绝--manifest-path（非Cargo/授权失败）；本域随后实读5fc41dbe…/同driver-runtime，manifest与11成员匹配，client仍57153be4…，parse_cargo的options无该flag、437行落unsupported分支；runner b38a18a9…的4786/4805已有Prefab prefab_和manifest插入。只读源码能定位client/runner合同不一致，不能代替5fc新解析、服务生命周期或完整闭包准入。本域没有新parse/QueryOnly/server调用或公共入口写入。原管理owner须通过真实部署链闭合并资格化同一selected的bounded nested-manifest client/runner/source准备，保留manifest/lock/config/path边；不丢掉flag、不借材质filter、不绕selector。

09已对当前08四源完成语义审阅，原未提交-02及86b638a0…输入不变，旧04保留历史。本次3文档变化使12的775c9c84…有限元组亦只保留为原时点；当前有限11路径另登记，不是9-root membership、全writer、有界稳定窗、pool/cache writer、reservation或native EOF。原owner精确keys/marker未见收件仅限22:27有界读回，本域未扩大为全局不存在或自动补intake。

当前source-owner审阅已将原未提交`validate-prefab-source-adapter-02`关联到source-tree-08四源tuple `e9d70578…`及16项定义（15普通+1原ignored/0执行），见[审阅09](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-successor-review-09.json)。候选JSON仍为原1801B/SHA`86b638a0…`，无第三operation ID或受理payload改写。08最终23941B/SHA`92d445ce…`的guard/format已完成；原owner20:35读到SHA`8478f93d…`的pending中间状态保留，不据此否定最终结果，也不声称原owner已接受最终报告。旧review04仍只覆盖当时13项，当前审阅不取代正常server intake或资源/完整writer准入。

原#348单配方已complete/observed、FAILURE/preparation-conflict、prepare/compile=false。此前source-adapter-02修订为13项定义、12普通可运行+1原ignored/0执行，见[source-revision-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-revision-04.json)。source-tree-08又修改原四源并新增3项测试定义，当前16项/15普通+1原ignored/0执行，四SHA与前后修订见[source-tree-08](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-08.json)。旧input、source-adapter-02及source-revision-04不覆盖新字节，均未被覆盖；后继JSON本身不包含完整源码指纹，当前修订关联由审阅09明确登记。本域沿用既有后继ID及不变候选payload，原Jenkins正常收件/准入仍pending，不是同operation attempt转换或执行回执；历史命令只作配方描述。

当前authorization active/selected11pins匹配与源码guard/format不能替代完整源码准入、所有writer、稳定窗口、pool/cache writer、reservation或host生命周期。后继未提交，capture-ready=false；普通收件和资源门仍由原Jenkins owner接续，业务Rust/产品/清理继续open。无需用户手工grant，本域不改公共控制面或把未知资源门写成通过。

已实读原Jenkins owner的[独立Host准备合同](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/host-targeted-template-pane-20261024/contract.prepared.json)与[本域对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/host-preparation-reconcile-06.json)。三Host源91257173/7d4cb542/b68235c3与原交接相同；根manifest/zircon_editor/targeted_template_pane_的4项属于Parent独立验证，request/operation/attempt未建、执行0，不能借Prefab身份。host06时点Prefab四源匹配source-adapter-02的a033/1c21/f129/898f；此为本轮source-tree-08之前的观察，原合同三份变化仅相对旧#348，不归责于后继修订。

本域实读selected 0b594015…并核对5份capsule源码+6份pinned clients；197 driver与正式HTTP binding复用原owner证据。显式test profile可由local-cargo selector表达并交Jenkins，module脚本无Profile参数/recipe元数据dev；模板和完整Cargo feature图均未执行。capability/纯parse不是membership/writer、source-prepare/native EOF或物理E reservation/pool/cache-lock回执。继续保留现有Runtime feature声明、两独立测试scope与全部缺门；不启动capture/freeze、normal validate或清缓存来制造准备通过。

## AB-PF-15：域内实际驻留来源查询入口

`zircon_plugins/prefab_tools/editor/src/authoring.rs::prefab_source_snapshot` 已实现并由 lib.rs 重导出，实际输入为调用方持有的 `AssetReference`、`AssetRegistryIndex`、`ResourceManager`；输出为既有 native active key 和真实 typed Prefab snapshot。UUID 解析保留迁移身份，不按旧 locator 绑定其它 UUID；先核目录 kind，再用 `snapshot_expected::<PrefabMarker, PrefabAsset>` 校验 active key/载荷。它只消费已驻留数据，不调用 loader、不注册生产者、不建立全局 map 或新版本时钟。

| 结果 | 调用方必须保留的语义 | 当前接线边界 |
| --- | --- | --- |
| 成功 `(key, source)` | 来源资产 UUID、实际载荷和 native key；延迟使用前复核 key 与 owning project/document/view/selection/World/schema/request | 同项目生命周期是调用方前提；当前没有实际 host publisher 调用，不能当生产绑定完成 |
| `Reference(AssetRegistryError)` | 真实目录解析诊断；原 UUID/locator 保留，不能用旧地址猜另一个资源 | 源文件删除/改址、加载错误与导航回执仍由原 asset/host authority 处理 |
| `WrongAssetKind { actual }` | 目录实际 kind 不是 Prefab | 禁止消费，不能把错误降级为默认数据 |
| `NotResident { asset_id }` | 引用已解析，但尚无 active resident version；不等于 Missing | 等原 loader/请求，不能在本函数新建 fixture 或同步盘上加载 |
| `SnapshotRejected { asset_id, error }` | 原 native `Stale` / `KindConflict` 诊断；`NotResident` 是前一行的本域独立结果 | 退休旧结果并沿原请求重新核对；key 包括 remove/recreate 身份，但不是 residency lease 或持久 mutation permit |

source-adapter-02阶段五项新增回归覆盖 UUID 迁移、旧地址被不同 UUID 复用、目录存在但未驻留、目录/载荷类型错误和同 ID/同 revision 删除重建后的旧 key 拒绝。该阶段13项定义/12普通可运行+1原默认ignored/0执行及当时四源见[source-adapter-02](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-adapter-02.json)。后继source-tree-08的当前16项与新字节另记，不以此历史receipt代验。Parent host guard 与原 ZUI 未变；AB-PF-07～14 的稳定 target、per-view publication/retention 和写事务缺口保持。

## AB-PF-16：真实树、Details与operation的消费边界

以下是现有字段/API的消费合同，不增加Rust类型、序列化字段或公共写入口。源码与最终回执字节复核见 [target-contract-03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-target-contract-03.json)。通用scene Delete、gateway/history/journal/inverse/retention归Parent核心层；本域提供真实业务目标、可用性和显示/结果映射。

| 原字段/API | 本域消费与失效规则 |
| --- | --- |
| `ActiveSceneDocumentIdentity` 的document/project_root/scene_uri/activation_revision/source_revision；实际scene entity与完整component type | 在原document authority内定位entity/type及模型实际提供的组件身份。跨关闭重开先由原document owner重新解析资产/实体，再恢复可证明的行；不把当前DocumentId或Arc打印成永久UUID。`same_activation`不包含source_revision，延迟结果须另外复核来源修订。 |
| `project_inspector_components` 的component.id/property.id | 实际路径 `zircon_editor/src/ui/ui/data/visual/product_presentation/inspector_components.rs`：native id取component_type_path，plugin id取component_id，property id取field_id；title/label是显示内容。该对象自身不含document/entity authority，不能把component.id单独当全局itemKey。native属性明确readonly；plugin属性仍受property.editable和customization_available共同限制。 |
| `resolve_subject_path` 的 `entity://selected` / `node://…` | 实际路径 `ui/binding_dispatch/inspector/subject_path.rs`：前者读取当前active_primary，后者按当前state/World检查node。两者都不能独立表达跨document/asset的持久目标；异步或tab切换后，必须先核验原document/view/selection/World，再由原owner重解析，不能对当前全局选择派发旧行。 |
| `inspector_component_add_catalog/template/preflight` → `inspector.component.add` | 实际host在Edit模式获取committed scene history、UI/core selection generations、完整gateway stamp、world_generation、targets/primary；翻页/取模板也校验原scope，提交再次复核。类型使用真实descriptor.type_id，显示名不作目标。该crate内Add合同只用于absence-only场景实例；本域不复制它或广开pub，更不借它写类/Prefab资产。 |
| `validate_created_dynamic_components` → `remove_created_dynamic_components` → Add命令revert | 已有Add undo：全批value与creation receipt预检，且只移除该命令创建且未变的组件。receipt的Arc::ptr_eq独立核验运行时创建实例；不参与持久行序列化，也不授予任意既存组件Delete/restore能力。 |
| Add执行/撤销的 `Completed` / `AppliedThenGatewayFailed` | 消费核心实际回执，不把所有Err都当零变更，不在插件再次补偿/重投。预检拒绝保持零变更；执行后错误须沿原history/core结果刷新真实变化与错误。当前未做产品操作或验证这些分支。 |
| `prefab_source_snapshot` 返回的原native key与snapshot | 延迟发布核验原key：对同项目ResourceManager调用 `snapshot_expected` 并保留该次错误；不能仅获取最新key再给旧payload/诊断重新盖章。来源UUID/owning document与完整发布上下文也必须一致。`NotResident`不做Missing负缓存，`Stale/KindConflict`退休旧结果，重读属于原请求的新结果而非旧结果通过。 |
| Make Root/Reparent/类继承/child-actor | 现有entity.parent和component payload不提供通用attachment/root/继承身份合同；保留UE目标设计与具体差异，真实capability缺失时保持不可用。不给未注册的Delete/attachment操作编造route/action ID。 |

target-contract-03当轮四份Prefab源、Parent三份targeted host及原ZUI均无新改动。原#348只关联旧8项源码且compileStarted=false；source-adapter-02的13项/0执行、当前source-tree-08的16项/0执行和独立4项host测试分别记录，不交叉代验。Widget类/图/Definition合同及公共导出/normal EXE gate保持原owner，无新桌面或服务操作。

## AB-PF-17：已实现的来源实体只读投影

`PrefabSourceAuthoringSnapshot::new(&PrefabAsset)`消费实际模型，`source`及每个`PrefabSourceEntityRow.entity`直接借用原资产/实体，全部native字段、opaque组件type/schema/provider/payload和引用表仍由原模型持有，不重建组件schema或复制`SceneEntityOverview`。`entity_rows`保留原序和全部冲突项；诊断的`source_indices`只指本次来源实体vector，与实例override位置分开。

| 实际结果 | 消费约束 |
| --- | --- |
| `Root` | 原实体parent确为None；不意味着UE SCS根、继承根或可Make Root |
| `Resolved { source_index }` | parent u64在同一来源vector恰有一个匹配；索引只用于当前借用中的树遍历，不得持久化或用来提交写操作 |
| `Missing` | 原parent不存在，保留原parent和本行诊断，不降级为正常根 |
| `Ambiguous` | 原parent ID匹配重复实体；不挑首项，DuplicateEntityId诊断集中保留全部候选位置 |
| `Cyclic` | 迭代检测得到的循环成员；不发布可遍历父边，原parent和所有行仍保留，后代不丢弃 |

来源UUID及native key不由`PrefabAsset.uri`、名称或row推导。真实caller先用AB-PF-15返回的resident payload构造投影，保留原key及owning document/project/view/selection/request；延迟发布前按AB-PF-16复核原key和全部上下文并处理取消/清旧值。本轮没有实际每view publisher，不称runtime UI已经读到这些行；component attachment/SCS、写target、journal/history/undo仍由原owner接续。

三项新增回归定义验证lossless resident借用/u64::MAX/重复组件引用、重复ID/缺失/歧义、8192层链循环/self-parent及空来源。当前16定义/15普通+1原ignored/0执行；[source-tree-08](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-source-tree-08.json)登记当前四源、静态结果和未受理的后继验证边界。此项是本域可审阅实现，不是完整组件树、插件/Rust或正常EXE通过。

## Shared rev2：当前单leaf局部颜色消费

原body与颜色leaf分开消费：当前body `9fffda54-1c42-5c4f-8e69-bb03997f533d` 属本域SourceReferenceData main，没有Typography引用；原custom-495ff285…/normal-400与Shared body的gfont-roboto/regular不是同一provider身份。同family、weight、size或FontFace loaded均不证明字体URL/字节、variant、CJK/Engine呈现一致。18:11 saved/暂存状态保持未确认，新context只核对当前源，不补签历史或重套Typography。当前两图、精确source字段和全240对象解析见[源对照07](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-source-compare-07/result.json)；120 Shared+120本域main位置分列，13仅是Shared根数。

按Parent第六次合同，仅在原受控板的Button文字leaf `89eb3427-27ba-801b-8008-d6d09c3492ba`从Shared Assets选用`editor.text.primary` / `e022fcac-39df-51eb-8515-4723f9932060`。fillColorRefFile为原Shared `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`；paragraph/run/positionData三处引用仍属于同一leaf。真实token-source指回`zircon_editor/assets/ui/editor/theme/editor_tokens.zui`的palette.text_primary，当前SHA `dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743`匹配发布元数据；未新建主题或改token。

保存rev46、Undo rev47全322恢复、Redo/fresh reopen rev48均独立API验证。新增text-content-attribute覆盖组，中文、原custom字体、source/shapeRef、几何、其余321对象和原13copy/全文件240linked身份不变。此为local override，原色/样式同#c0c0c0；没有main继承、全局A/B、正式导出或产品传播证明。新rev45读回未见旧body引用，18:11保存仍未确认；不能据此证明从未暂存，原失败保持，未重选Typography。

Shared仍rev2且对象/components/34+3样式/library metadata全部与本轮基线相同。重开出现真实共享库更新横幅，与新使用颜色modifiedAt17:37:45.684Z大于原syncedAt13:50:51.913694Z相符；未点Update、未改时间戳或ignore-sync。真实main发布仍由原public owner交付，再核定受影响组件，保留全部覆盖后原生消费。新context的Roboto400/700 loaded只记录该时刻浏览器字体面，不当CJK/Engine ready。完整引用/4逐图/退出在[leaf-style-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-leaf-style-05/result.json)。

给public pipeline owner的具体导出边界：其当前`tools/penpot/apps/zircon-zui-plugin/src/penpot-capture-validation.ts::singleSolidFill`明确拒绝`fillColorRefFile/Id`，`penpot-text-capture.ts::captureNodeTexts`经该检查读取editable text。此处只是源码核对，尚未观察本leaf经SDK读取的fills或实际export错误。请在原owner范围内确认SDK样式投影，再给Shared color ID→受验证token-source→原palette/style字段的合法回写合同，保留sourceHash、stable ID、shapeRef及局部覆盖；未知引用应给明确诊断，不能把引用抹成literal、detach或关闭guard当解决。本域不改插件、Shared、catalog或token来绕过。

## Shared rev2：上一轮消费与参数边界

Shared现已实际发布34 colors/3 typographies，四main保持原UUID；本域独立读回rev2及原13个external native copies基线。原生库更新页却显示无需更新：组件modifiedAt仍13:21:31.125Z，小于本域syncedAt13:50:51.913694Z；新文件modifiedAt17:37:45.683436Z未使已使用组件通过Penpot的component变更过滤。旧copy的typographyRef空字段不算真实style绑定。请原public owner接续合法组件发布，再由本域原生同步并核对覆盖；不通过改Shared、unlink/relink、detach或清覆盖绕过。

本域业务main field0已通过原生素材选择body，随后服务拒绝连接，保存与重开未确认。Roboto400/700 status=error，body/CJK检查false；publication不等于字体ready。真实UUID、基线/截图/失败和两次Parent入队回执见 [shared-style-04](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-style-04/result.json) 与 [设计本轮记录](actor-blueprint-design.md)。尚无全局style A/B、外部copy同步或产品传播通过。

| 公开契约 | Actor/Prefab的真实输入与可用性 |
| --- | --- |
| NumberField的value_text/placeholder/value | `PrefabPropertyOverrideAsset.value`实际为serde_json::Value；当前snapshot保留原值，只读显示与原文档/实体/属性目标关联。模板42不是业务值，不能把整数经f32转回后写入。value的实际控件数值表示、schema与精度未核实前，不开放数值写入。 |
| min/max/step/large_step/precision | 由真实字段schema/单位/限制提供；当前Prefab override本身只有entity_path/property_path/value，不自带这些元数据。不能用0..100/1/10/precision0替代未知schema，也不能为接控件发明schema。 |
| editable_text/layout_stepper | 原控件呈现参数；启用写入仍须有真实可编辑字段、原document/generation、注册operation与结果/错误回执。现有只读Prefab adapter不因此取得写authority。 |
| PropertyEditorRow.value | required=true、单个合法Field/NumberField等。Shared仍只有原四主件，无已发布的合法value件UUID；保留必需槽缺口，不塞SearchInput或推导UUID。 |
| description/source | 仅在真实schema描述、来源引用/状态存在时填充；source保留真实UUID/URI及当前查询诊断，不把NotResident写成Missing或把native key序列化为持久身份。缺内容时省略optional槽。 |
| reset | 需真实“本实例override→来源值”的目标与事务/保存/undo回执；目前无对应Prefab写命令。新增槽不构成handler，原生Penpot文字reset试验也不构成业务reset。 |

新catalog SHA `4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5`，独立核对NumberField的10个参数名与Property的4个槽、原required value与可选槽标记；未重跑Parent的25项完整静态检查。新contract候选不由旧plugin/Rust receipt覆盖；本域未改catalog、tokens、Starship、公共master或导出guard。

## Shared rev1消费回执：历史阶段

Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13`在该阶段为rev1/isShared=true/41对象/4主件；本段与下文带时间的rev0/空库段均保留历史，当前发布以本轮rev2记录为准。Actor在原file/page新增验证board `89eb3427-27ba-801b-8008-d6cfcabb7a3c`，原生管理库连接后由Assets真实拖动产生四个linked copies，初次消费阶段最终rev21/151对象、原2业务主件/12copies保留。完整component/main/copy UUID及逐图表见[设计AB-D08](actor-blueprint-design.md)，独立API与33个shapeRef/元数据核对见[consumer-01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-shared-consumer-01.json)。没有复制main或detach，Shared及公共源码未写。


前一轮Actor rev30/241对象：新增三尺寸AB-CONSUMER-02板/9个native copies/87个shapeRef；原150个非根对象、业务mains/12copies及旧四件消费者保留。Shared仍rev1，库对象/component哈希不变。中文、fill和两轮fresh-context重开已核实，但内部宽度问题未解决，见[该轮回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-component-content-02/result.json)。本轮另增AB-D09本域状态设计到rev45/322对象，保留上述241对象中的全部非根内容和公共主件；不以业务状态板替代公共内容宽度修复。

给Parent/原public pipeline owner的最小接续提案（未实施，不扩公共写权）：

1. Button根592/852/1232时文字仍220宽、left锚定；使分配随根宽变化并保留既有text_align=center/padding/font/token，不由各族坐标补偿。
2. Search根fill而surface仍240、text204；给出native消费宽度合同，surface/text随根分配，搜索图标保留语义inset。
3. Tree caption仍128×16、中文两行，eye/lock在源宽位置；需caption单行ellipsis及末尾动作分配。Actor组件树的eye/lock可用性须由真实action/state合同表达，不把无handler动作当完成。
4. 9个copy的native-layout-allocation.parent均指向canonical父节点。只读定位`tools/penpot/apps/zircon-zui-plugin/src/penpot-control-content.ts:288` treeRowPainterSize、`:554` refreshTreeRowPainterContent及`penpot-layout-allocation.ts:37` allocateNativeLayoutSize；原spec保留foreign-parent拒绝。需owner明确native linked copy的source/shapeRef/generated-state/allocation映射并保持guard；本域未调用refresh/export、不声称已抛运行错误，也不改metadata绕过。
5. 原Property required value/合法Field或NumberField发布请求保持；三probe板未放空Property伪装可编辑Details。原资产投影/codec/slot/override回执仍缺，probe不是原ZUI导出root。

canonical-source是URI#component字符串，canonical-record是component.pluginData.devZirconZui里的JSON；root shape另外有sourcePath/sourceNodeId/instancePath/controlId。四个asset board的document先解开typedValues包装，再读penpot_original_source；本域逐字节比较当前原ZUI一致：

| 主件 | 当前原源SHA-256；源路径见canonical-record |
| --- | --- |
| WorkbenchButton | `05c1245c915f75b1f3b1be025942f2d05bc985e88d8a75db1f168c554563f3e5` |
| WorkbenchSearchInput | `9be3d699fff401efe819a3444d3ce47919cbb7270642d0f5a426a91390830c7e` |
| WorkbenchTreeRow | `c6687ef563179bfbe395917cb4805914cb9e2b3f7f0bc49091f00135be144cf2` |
| WorkbenchPropertyEditorRow | `bf29d5740c317b486ec5e6f2ef3faf06f7e800f6ad1dfcd773f0b403e87ef6ce` |

这是消费者的独立源检查，不替代原public owner的正式codec/导出回执。metadata record version1不是component revision；library rev1、真实modifiedAt和各copy/ref分别记录。Button中文文字覆盖在rev19保存/关闭/fresh context保留，原生“重置覆盖项”得到rev20/Button/touched清空，再设中文保存rev21/第二次fresh context保留；这里只验证Penpot文字，未验证类/实例属性reset或业务Add。

最小公共依赖：PropertyEditorRow的value槽required=true、multiple=false，目前4主件都不在允许的Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle集合。copy value_slot `89eb3427-27ba-801b-8008-d6d0b6203907`引用原`6ddc248b-bd35-80c5-8008-d6c8c88eaacb`、children为空。本域保留缺口，请原owner交其中一个合法最小value主件的真实UUID及slot/覆盖导出合同；不推导未发布UUID或塞入SearchInput。Menu/Dock、native共享colors/typographies（当前均0）、两族全局style A/B、官方ZUI/PNG导出及EXE仍独立pending。原有效业务源映射未替换。

验证board没有原产品asset owner/document投影元数据，不作为原ZUI导出root。正式接入仍需由原pipeline将原`WorkbenchExtensionPrefabEditorWorkspace`及既有control/binding/event身份投影到同一真实资产；不能从验证板名、文字覆盖或Shared URI推造业务绑定和事件执行。

## 已确认的归属与真实文件

已读取 [Parent 交接](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)，本域 chat `01a16d53-8965-7472-931c-73cba748fe8e` 已登记。材质 chat `01a16d50-6c7f-7370-853d-4581184b66a1` 牵头共享节点图表面；动画 chat `01a16d51-741b-73b3-a61a-2189cd38f81e` 负责共享 timeline/curve/transport；Parent `01a16d4e-0491-7353-84fa-038bd7706546` 负责跨域合同、公共树/属性/菜单/Dock/主题和集成。专用业务命令与资产 authority 仍归各域，分工不等于共享源码写入许可。

本域真实文件为 [09 Actor and Blueprint](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2)，file `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf1`、page `b4a1e3ce-9e6a-81fe-8008-d69d60f73bf2`、project `b4a1e3ce-9e6a-81fe-8008-d69d606f6160`。2026-10-24T10:33:10Z 本域独立认证 API `POST /api/rpc/command/get-file` 实际返回 HTTP 200、Content-Type `application/json`、`revn=0/vern=0`、页面 1 个根对象、组件/颜色/typographies 均为 0、`isShared=false`；响应 SHA-256 为 `951718e76798611538cb06f833607b5599dddde5d7dc831e34037def6dadbee7`。

同轮只读确认公共文件 `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`、page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13`：HTTP 200、`revn=0/vern=0`、根对象 1、组件/颜色/typographies 均为 0、`isShared=false`；响应 SHA-256 `fe0d8e5c3ff7850da27eade99425f4192fa7f8665047b43a506167ff4afcc734`。`get-file` 未返回 libraries 列表，不能将缺字段当作已验证所有链接库为空；已知本次公共文件不是已发布组件库。

上述初次只读请求复用原 P21 认证 storage 的 cookie，仅在内存中使用，没有打印 secret、另存认证、创建账户或修改 Penpot。首次检查器按旧 kebab-case `pages-index` 查找而断言失败；实际响应字段为 `pagesIndex/isShared`，核实后修正检查器，两文件均真实读回。失败输出保留，未改服务使读取通过。文件分配/读取不计为设计保存、重开或验收。

后续按 Parent 允许的独立设计范围，仅修改本域已分配文件：真实保存 5 个区域布局草稿板，首次 get-file 为 revn=1/58 对象/41 flow frame，浏览器后续 revn=2；首板已无头浏览器关闭/全新 context 重开，五板都有完整设计图与复核。详见具体设计与自有回执；组件实例仍为 0、无已验证共享样式引用。所有文本为不可导出的设计标注，未复制公共主件或创建平行主题。公共库/实例、正式导出和产品能力合同仍需接续。

## 合同表与写入边界

| 提案 | 已指定牵头/集成 owner | 本域实际消费者与需要的结果 |
| --- | --- | --- |
| AB-SHARED-01 树 | Parent | scene-instance Components、类/Prefab Components、My Blueprint；沿用 TreeRow 状态/主题，提供稳定行身份、继承/root/override 可用性、drag verdict 和 selection/focus 投影。 |
| AB-SHARED-02 节点图 | 材质牵头；Parent 跨域协议/集成 | Actor 事件/构造/函数/接口图、图查找与 diagnostic 定位；复用 GraphCanvasState/GraphModel/GraphDeltaCommand，Actor schema/执行/编译仍由本域与脚本 owner 负责。 |
| AB-SHARED-03 文档与焦点 | Parent | 两个资产与 scene-instance 的 history context 隔离；真实 asset URI/tab/dirty/关闭/恢复；菜单/弹层及无效 trigger 的返回合同。 |
| AB-SHARED-04 属性与诊断 | Parent | 类默认、实例 override/reset、反射 schema/provider、编译 source location；复用现有 row/slots，不能把 GraphPin.name 或显示文案当持久字段 ID。 |
| AB-SHARED-05 预览 | Parent 协调实际 renderer/input/resource owner | Components 预览 world、camera/selection、构造/编译后实例刷新；现有 WorkbenchPreviewViewport 为静态 Image/SVG 默认内容，不满足该消费者。 |
| AB-SHARED-06 内嵌 Timeline/Curve/Transport | 动画；Parent 跨域协议/集成 | Blueprint Timeline 子文档：同一 owning asset/history、秒时钟和 track/key identity、事件/标量/向量/颜色轨、真实播放状态与可逆编辑。它不是独立动画资产或 Sequencer 的复制品。 |

本域当前写入为 `docs/plans/penpot-ue-window-suite-20261024/actor-blueprint*.md`、自有证据与已分配的 Actor Penpot 文件。产品目标是原 `zircon_editor/assets/ui/editor/world/extension_prefab_editor_workspace.zui` 和 Prefab 域业务候选；`zircon_editor/src/ui/{graph,timeline,curve}`、公共 .zui/catalog/theme、插件和核心事务路径不由本域直接修改。需要变更时给具体文件/symbol/行为和受影响消费者，先由 Parent 完成重叠交接。

## 给材质 owner：AB-SHARED-02

已核对其 [MAT-SHARED-001](material-family.md) 以及实际 `zircon_editor/src/ui/graph/{model,canvas,node_widget,commands,routing}.rs`。renderer-neutral 类型不表示存在 retained 交互 surface，也不能证明某域 GraphModel 适配已接通。本域不另造画布、选择模型或整图快照历史。

| 边界 | 可复用的当前事实 | Actor/Blueprint 的具体合同需求 |
| --- | --- | --- |
| 目标/修订 | GraphModel 为域 authority，GraphDeltaCommand 通过 GraphEditContext 按 target 获取模型，retention proof 与 revision 绑定 | target 必须能区分 asset/document/graph/subdocument，旧 revision/已关闭文档拒绝 mutation。GraphModel 适配与当前 history context 对齐；pan/zoom/选择只属 view state。 |
| 节点与 pin identity | GraphNodeView 有 node id、bounds、inputs/outputs；GraphPortRef 当前以 node_id/direction/pin_name 定位，pin 没有独立稳定 ID | Actor pin/member 的重命名、动态引脚、split/recombine 与编译 source map 不能只按显示名称恢复。先明确 pin_name 是不可变 schema key 还是改名时由域 remap；若需稳定 pin ID，由 Parent/图 owner 共同改合同，不在本域生成第二套端点。 |
| 执行/数据语义 | GraphPinView 只有 value_type/required/direction；default_connection_verdict 为严格同型，支持 FreeGraph/Tree/Dag | Actor event execution 与数据 pin、object/interface/class/容器类型、wildcard 解析与转换由域 schema 决定。不能把材质 Dag/单输入默认值硬套事件图，不能仅以颜色判定类型。先传递明确 domain verdict/拒绝原因及需要的原子替换/转换事务。 |
| 接线反馈 | 现有 ConnectVerdict 仅 Allowed/Rejected；GraphConnectRejection 为封闭枚举 | UE 参考 `UE/Editor/BlueprintGraph/Private/EdGraphSchema_K2.cpp:2176` 还区分断开旧连线、转换节点等响应。本域需要提案如何在现有模型上表达这些域结果；尚缺 schema 时保留待定，不画出“成功”后丢数据。 |
| 布局/输入 | GraphCanvasState 处理 pane-local 逻辑变换、选取/节点拖动，GraphNodePresentation 是中立投影 | 复用一次 DPI 转换及实际测量 pin anchors；普通节点内部 title/字段/pin rows 用 flow。拖动预览不写资产，mouseup 一次可逆 delta，Esc 不留 mutation。 |
| 菜单/剪贴板 | GraphNodePaletteDescriptor、GraphClipboardModel 已定义域创建/序列化边界 | Palette/context/pin-drag 创建走同一业务 command，来自同一 schema。复制/粘贴保留类型/外部引用并做稳定 ID remap；无模型时不提供虚假节点动作。返回原 trigger 或有效 graph surface。 |
| 诊断与调试 | 当前 GraphDiagnostic 带 node_id/port 和静态 message；没有证明具备完整异步 compiler source map/VM debugger | 共享呈现消费本域 source map 和真实 request/source revision；支持高亮实际 node/pin、跳转 focus、过期结果。断点/执行高亮与 selection 分开，由真实 VM feedback 提供，不从运行按钮推断成功。 |
| 证据 | 源 node/pin、实例与行框需要保留到 native capture | 每个节点/引脚/连线/文本要有同一 case 可回溯的 sourcePath/sourceNodeId/instancePath；viewport transform、hit bounds 与输入回执对应，不能只有一张整体图。 |

UE K2 schema 在此仅作行为依据，不要求 Zircon 采用 K2/UObject 类型系统。Actor 类/事件图资产与 compiler adapter 尚未确认，合同保留该事实。可先支持真实模型具备的子集；其余仍列 AB-004/005/006，不因共享图 surface 到位就关闭。

## 给动画 owner：AB-SHARED-06

UE `UE/Editor/Kismet/Private/BlueprintEditorTabFactories.cpp:119` 的 FTimelineEditorSummoner 是 `UTimelineTemplate` 子文档 factory，保存时记入 owning Blueprint 的 LastEditedDocuments；实际 body 是 `STimelineEditor`。`UE/Editor/Kismet/Private/STimelineEditor.cpp` 按 Event/Float/Vector/LinearColor tracks 分流。共享 timeline 不应另复制成无 owner 的独立 Sequence。

| 传递给共享表面的上下文 | 本域/共享职责及实效要求 |
| --- | --- |
| owning asset/document + timeline identity + source revision/history | Actor 域持有实际模板/轨/变量关系和 mutation，动画表面只投影；关闭/删除 timeline 或切资产时释放相关 session/selection，不能继续编辑旧 owner。 |
| authoritative seconds、duration、view/play range、display FPS、playhead | 与动画 AN-P01/02 对齐。FPS 是显示/吸附设定，不能改变源时间语义；start/end 与 step forward/backward 分别有能力和事件，不能把硬编码 previous/next 当逐帧。 |
| track/key/curve/component identity、value kind、event semantics | 复用 TimelineModel/TimelineElementRef 和 CurveModel/CurveSelection；事件轨与数值曲线分开，vector/color 的分量关系保留。本域未证实有对应资产时保持 capability 缺失。 |
| key/value/tangent/interpolation 的预览/完成/取消 delta | 共享 canvas 返回意图，域在同一文档事务提交及恢复 inverse；不在 renderer 复制 timeline 资产或另存 undo history。数值输入与拖动走同一 mutation path。 |
| 实际 playback / scrub / event feedback | 来自同一个 preview/runtime session。时间推进、暂停/循环/重放、scrub 是否触发事件必须由本域运行语义定义；不能用 timer 或动画图片当执行结果。 |
| 关联图节点/轨创建、重命名/删除与 diagnostics | Timeline track 输出 pin 与图节点绑定要保持稳定身份，引用变化与图修改原子或有明确失败恢复。没有真实 timeline→graph/compiler mapping 时不创建可点击的无效果跳转。 |

复用现有 `zircon_editor/src/ui/timeline/model.rs` 和 `zircon_editor/src/ui/curve/model.rs`；当前 foundation 的 `apply` 返回 inverse 不证明已有 Actor Timeline runtime/文档适配。时间单位、输出类型、trigger 和 callback schema 由本域与脚本 owner 提供；动画 owner 无需承担 Actor VM。

## 给 Parent：树、属性、宿主、资源与证据

1. 树投影要同时传递稳定 target identity、层级、selection/focus、展开、继承/root/override、命令可用性/原因和 drag verdict。当前动态组件 creation token 是 World 内临时证明，不能做持久 itemKey。旧固定 Mesh/LootSocket route 与 ListRow→TreeRow 引用迁移需要明确映射，保留原 control/event/binding 身份，不按文案派发。
2. 属性复用 WorkbenchPropertyEditorRow 的必需 value slot；当前没有证明其 description/source/reset 足以表达类与实例全部状态，优先与设置域同一扩展对齐。reset 明确“当前实例 override”或“类字段”及写 scope，不能清掉错误默认源。诊断 row 的 severity/message 之外，节点跳转 action/键盘 focus 需明确公共合同。
3. DocumentHost/UiHostWindow 是原 shell view 中原生 region_panel 节点，不能作为独立 component import。Parent 提供资产 URI/tab/context、body mount、dirty/保存/关闭、弹层 focus 栈、panel 重开及窄窗可达性；Actor 只供给真实内容和命令，不建立平行 Dock。
4. PreviewViewport 默认静态 Image/SVG、输入关闭的事实已确认。所需 renderer/input/resource 合同包含 owning document/revision、实际 preview world/subject、camera/selection/gizmo、资源 readiness、构造/compile 后 generation，以及正常 EXE 的 GPU present/source 关联；不以 primitive 注册计为真实能力。
5. Shared rev1四主件已可消费，本域已链接并保留component/main/source身份；当前仍缺上方列出的合法最小value组件、Menu/Dock、native共享style及正式slot/override导出合同。由Parent/原pipeline owner按既有范围接续，本域不复制按钮/字段/树/主题或替换原有效源映射。
6. 桌面时段未授予。待 EXE/资源和 case 就绪后，按 Parent 协议提交 file/page/board、真实状态/尺寸/实际 DPI、source/EXE hashes、预计时长、nonce/generation，再等明确交接。队列成功不等于时段或 capture 通过。

本轮新增只读 API 观察及合同不产生 passed/accepted。资源 ready、真实 editable component/flow 保存重开、导出原资产、产品命令、normal-window 捕获、双端逐图审核均分开保留；旧失效服务/P21/Rust 验证失败证据不删除。

## 本域业务数据实例与真实 Prefab 适配补充

先前 revn=2、0 实例是第一阶段观察。当前本域 file revn=4/vern=0，两个 SourceReferenceData/OverrideRecordData 业务主件和 12 个原生 copies 已保存、关闭并在全新 context 重开；实际 UUID/modifiedAt、来源夹具和逐图复核见 [设计 AB-D08](actor-blueprint-design.md)。公共控件实例、样式引用和导出仍未完成。这两个纯数据叶主件没有 Engine component ID、绑定/事件或查询/写命令，不提交公共 catalog，不复制 AB-SHARED-01/04 的 Tree/Property 主件。

Prefab 资源应复用已有 `zircon.builtin.toml.prefab` typed importer 与 `AssetManager::load_prefab_asset`，不能因插件的 DiagnosticOnly 注册而另造解析器。registry 首先比较 capability availability，既有 available-vs-diagnostic 测试有实际源码；本轮没有运行测试或项目 registry，仍需真实加载回执。场景的 `zircon.prefab.instance` 元数据 roundtrip 不是 Prefab 实例展开、作者 Apply 或继承默认值。

具体域/公共适配依赖：保留 `PrefabAsset.scene/exposed_properties`、`PrefabInstanceAsset.prefab/local_transform/overrides` 和资源/文档修订；明确 scene project IO 的 `zircon.prefab.instance` 与 plugin descriptor `prefab_tools.Component.PrefabInstance` 是否/如何对应，不按相似名称合并。override 无持久 UUID 且重复路径有效，mutation target 不能仅用 entity_path/property_path 或行序号。该稳定身份与 reset/source authority 需求归 AB-SHARED-01/04 消费，新增资产 schema/事务仍先交 Parent/core owner，不在本域新建 store/history。

已读取 Parent 新增的公共 pipeline 委派和 [45 项覆盖对账](parent-window-coverage.md)：原 pipeline owner 接续真实共享主件/样式/导出及 Actor normal recipe；尚无交付回执。Editor Utility 的脚本/图合同归 Actor、UI 内容与 Widget 协作、VM/宿主归 Parent 接续。服务 ready、消息入队和归属均不构成组件发布或功能完成。

当前 [资源接入审计](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/source-integration-audit-01.json) 补充公共 pipeline 需返回的准确材料：实际 discovered Prefab package manifest/root、两个插件 URI 的真实文件及编译回执；Inspector template/data-root/allowed bindings 与本域操作工厂；canonical `world/extension_prefab_editor_workspace.zui` 与 native pane 旧 suffix 的当前 bundle/source/instance 对应；Actor 正常打开/资源/review-host/case-producer identity。通用 OpenView 已注册，仍缺带资产上下文的 open。反射 snapshot/descriptor、文件存在或旧 EXE 均不能代替同一资源加载闭包。

本域不改共享 `plugin_documents.rs`、Inspector host、native pane source identity 或 pipeline 来迁就候选路径。所查通用 component_drawer host 会真实调用 operation，不能拿 helper 的 changed 结果推断实际写入成功；Prefab 自己没有声明 bindings/写工厂，需按原命令事务接通。canonical 插件文件缺失和 suffix 不一致均为待运行核实的源码事实，不发布为已经复现的产品 bug。

生产数据还有一个明确注入点：`core/editor_extension/view_descriptor.rs::EditorUiTemplatePaneDataSource` 当前只有 `snapshot(&self)`，host 的 `extension_access.rs::ui_template_pane_data_snapshots` 在 enabled capability 过滤后调用它。本次限定搜索未发现 Prefab 或其他插件注册该 producer；不能注册一个固定 fixture snapshot 就称查询接通。Parent/host owner 需交可注入的真实 project/document/selection/world/schema/asset-revision 读取 handle 与生命周期/失效约束；本域实现具体 Prefab query/字段投影，沿原注册及 component patches 接入，不新建平行数据 store、history 或公共 trait。

本域已接续 `PrefabInstanceAuthoringSnapshot::new(&PrefabInstanceAsset, PrefabSourceState)`：来源引用/local transform 仍借用原资产，覆盖行保留原序、原值和 `source_index`，typed diagnostics 关联全部冲突位置或各条非法路径。`source_index` 只对当前借用有效，不能充当持久 ID 或写命令目标；宿主后续必须一并交 document/revision。`Unchecked` 不冒充 `Missing`，`Available` 只表示调用方来源查询结果，不证明预览 readiness。旧 `validate_prefab_instance` 复用同一诊断并保持排序去重的消息合同，已有 effective query 算法未改。三个新增回归测试覆盖这些边界；目前仅静态检查通过，生产 producer、写入/undo 与窗口未接通，详见 [adapter 证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/authoring-adapter-01.json)。

## Widget Graph：实际可复用边界与未接通项

接续 Parent「Widget/UI 第二次接续」及 [Widget 设计](widget-ui-design.md)。本节只交源码合同：Widget chat `01a16d52-d54d-7ac0-b0d5-21864f383a1e` 持有 UI node/property/binding 和 UiAssetPreviewHost；Actor 持有本域类/函数/变量/事件图的业务模型及脚本适配盘点；材质只提供中立 GraphModel/Canvas；Parent 集成公共 host、命令目录、SourceLocator/Goto Definition、跨模式焦点并对接脚本 VM 实现 owner。没有把这些责任分配写成已有后端。读取范围、指纹及三项限定搜索见 [Widget Graph 源码回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/widget-graph-contract-01.json)。

| 合同 | 已存在的入口及准确语义 | 当前缺口与接续 owner |
| --- | --- | --- |
| AB-WG-01 UI 源定位 | `UiV2AssetDocument.nodes` 的源 node key 与 `events: Vec<UiBindingRef>`；binding 含 `id/event/component_event/mode/route/action/targets`。`UiTemplateNodeMetadata` 保留 `source_path/source_node_id/instance_path`，与运行时 UiNodeId 分开；字段可能缺失，不从 node path 猜来源。[来源元数据](../../../zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs:15)、[v2 arena](../../../zircon_runtime_interface/src/ui/v2/arena.rs:58)。 | Widget 输出真实资产/文档修订、源三元组、binding ID/event、所选 property/target；重复组件实例须保留 instance_path，control_id 或当前行号不能单独定位。Actor 不改 UI 源归属。 |
| AB-WG-02 编译绑定追溯 | `UiCompiledBindingHandle` 是 generation + dense binding_id；program 提供 `binding_asset_id/binding_name/route_name/action_name`，binding 保留 node_id/source_binding_index/event/owner_asset_id。编译器逐绑定保留来源资产；它不是脚本字节码或类成员索引。[绑定程序](../../../zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs:79)、[编译来源](../../../zircon_runtime/src/ui/template/asset/compiler/binding_program.rs:77)。 | Widget/Runtime 沿现有编译来源恢复原 binding，必须检查当前 generation；dense ID/source_binding_index 不是持久 graph member/node/pin ID。尚无该 handle → 脚本 class/variable/event 节点的已证实映射。 |
| AB-WG-03 合法命令与动作目录 | Editor 的实际目录是 `EditorCommandRegistry.commands/command/generation`，descriptor 有 `when`、required capabilities、remote callable、payload schema、execution contract 和 action/factory。`EditorOperationControlRequest::ListOperations` 按当前 command context 的 enabled 与 remote 条件过滤，返回 operation_id/label/menu_path/callable_from_remote/undoable/undo_display_name/required_capabilities。[命令目录](../../../zircon_editor/src/core/commands/registry.rs:227)、[公开查询](../../../zircon_editor/src/ui/host/editor_operation_dispatch.rs:567)。 | Parent 从当前项目/文档的真实 registry 发布目录，Widget 只消费。ListOperations 没有完整 payload schema、registry generation 或所有 handler readiness 字段；有 descriptor 也可能在 invoke 时 MissingFactory，不能把目录项或 suggestions 显示为执行成功。需要扩展共享投影时先按原 scope 交接。 |
| AB-WG-04 Route/Action 的实际执行边界 | retained 通用 `dispatch_template_action_invocation` 已存在：Action 分支要求空 payload，转 EditorCommand；Route 分支解析 EditorOperationPath，把具名 payload 转 JSON object 后 `invoke_operation(UiBinding, ...)`。remote UiBinding arguments 则转换 JSON array，两者不可混用。host 检查 descriptor、权限、asset authority、enabled、native/event/factory，回传 record 或错误。[通用消费者](../../../zircon_editor/src/ui/retained/dispatch/common/dispatch.rs:86)、[host 执行](../../../zircon_editor/src/ui/host/editor_operation_dispatch.rs:109)。 | Parent 接 preview 发出的正确种类与参数 schema，不从字符串相似性选 handler；UI 的 route/action 两种目标和 Editor operation identity 分开。`UiActionHostPolicy` 只是副作用准入，不注册 handler。Widget preview 对外动作接到此消费者的闭环尚未证明，保留 gap。 |
| AB-WG-05 结果与错误回执 | `UiInputDispatchResult` 保留 applied/rejected effects、host requests、widget/component events、binding_reports；component event 可带 `template_action`，只是待执行意图。真实调用复用 `UiInvocationResult{route_id,binding,value,error}`、`UiInvocationError` 与 `EditorOperationControlResponse`/EditorEventRecord。`UiEventManager.register_route` 才持有实际同步 handler；`register_binding_route` 仅登记给宿主消费，直接 invoke 会报 no execution handler。[输入结果](../../../zircon_runtime_interface/src/ui/dispatch/input/result.rs:254)、[路由登记](../../../zircon_runtime/src/ui/event_ui/manager/registration.rs:21)、[调用结果](../../../zircon_runtime_interface/src/ui/event_ui/control.rs:126)。 | Parent 保留同次输入、源绑定、文档/Surface generation 到实际 operation/result/error 的关联，消费返回值和失败分支；早退 UnknownBinding/UnknownRoute/无 handler 不保证广播，不能只等通知。Widget 的 Preview updated、delivered、生成 template_action 都不是外部 handler 完成回执，不另建平行“成功”摘要。 |
| AB-WG-06 脚本与 VM backend | `zr_vm_language::load_project_package` 实际调用 ProjectWorkspace.open/compile/start_session；VmPluginManager 可 resolve_package_callback/invoke_callback，句柄带 slot/module/function/generation。`script.bindings` 消费实体生命周期 onStart/onUpdate/onFixedUpdate，不是 UI event dispatcher。VM 有能力受限的 register_editor_operation 和 active registry 查询。[真实编译入口](../../../zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs:13)、[callback](../../../zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs:351)、[注册贡献](../../../zircon_runtime/src/script/vm/host_interface/descriptor.rs:87)。 | 现存 `CR-VM-HOST-EDITOR-CONSUMER-0001` 明确编辑器执行消费者待定位；本轮 `zircon_editor/src` 对三种注册/查询符号限定搜索 0 命中。没有找到 Widget binding → callback 的实际 adapter，也没有 Actor/Widget class/variable/function/interface/event graph 的资产/代码生成/诊断 source map。由 Parent 指派 script/VM owner 接执行适配，本域提供 Actor graph 语义；不能借材质 compiler 或场景 callback 冒充后端。 |
| AB-WG-07 Source 与 Graph 导航 | Widget 已有按当前 source revision 缓存的源 outline、节点选择、source line/byte offset 及 cursor 恢复；binding diagnostics 已保留 target_node_id/target_binding_id。[源导航](../../../zircon_editor/src/ui/asset_editor/session/navigation_state.rs:111)、[诊断映射](../../../zircon_editor/src/ui/asset_editor/diagnostics/binding.rs:5)。本轮在 Editor、UI interface、script 与 VM 插件限定搜索 SourceLocator/GotoDefinition 等入口名均为 0 命中。 | 这些入口只证明 UI 源文本定位，不能切成 Graph 或解析类定义。Parent 持共享 SourceLocator/Goto Definition/focus；Actor/backend 还需真实 graph/document/class/member/event/node/pin 与 callable 源 URI/span 的映射。名字搜索未命中是限定证据，不宣称整个 VM 没有调试信息。 |

另有一项必须先对齐的实际标识边界：VM `register_editor_operation` 当前只校验原串无首尾空白且恰好三个非空段；EditorCommandId 要求至少三段、字符仅小写 ASCII/数字/下划线。`Game.Asset.Open` 这类现有 VM 测试注册名不能直接作为合法 EditorOperationPath。Parent/VM owner 需明确同一 canonical ID 与能力/代际合同，不能由 Widget 自动转小写或重写 source binding。此项是源码合同差异，尚无已观察的产品调用失败。[VM 校验](../../../zircon_runtime/src/script/vm/host_interface/registry.rs:573)、[Editor 校验](../../../zircon_runtime_interface/src/editor_command_id.rs:17)。

Runtime/UI 的动作目录须与 Editor commands 分开读取：`UiControlRequest::QueryTree/QueryNode` 返回当前树的 `UiNodeDescriptor.actions`；每个 `UiActionDescriptor` 含 action_id、event_kind、binding_symbol、parameter_schema、callable_from_remote 和可选 route_id。普通 Surface 的 `node_actions` 只从源 binding 建描述，没有填实际 handler 或完整参数 schema；不能据此把所有声明列为合法可执行目标。`CallAction` 会分别返回 UnknownNode、UnknownAction、ActionNotCallable、ActionMissingRoute，再进入真实 route handler。Parent 持当前树/host 投影和注册状态，Widget 保留该状态及错误，不把反射 action ID 当脚本 event ID。[控制查询](../../../zircon_runtime_interface/src/ui/event_ui/control.rs:176)、[动作描述](../../../zircon_runtime_interface/src/ui/event_ui/reflection.rs:476)、[Surface 投影](../../../zircon_runtime/src/ui/surface/reflection_snapshot.rs:213)、[调用检查](../../../zircon_runtime/src/ui/event_ui/manager/invocation.rs:50)。另外 `handle_request(SubscribeDiffs)` 当前丢弃 receiver，现存 `CR-UI-COMP-0002` 已登记；只有 subscription ID 不能证明接收循环存在。沿真实同步调用返回值交付结果，通知流修复归共享 host/runtime owner，不在本次改写。[订阅边界](../../../zircon_runtime/src/ui/event_ui/manager/invocation.rs:137)。

后续 SourceLocator 的最小交接要求是：Widget 侧提供真实 UI 资产/document/source revision、source_path/source_node_id/instance_path、binding.id/event 和 property；Actor/后端返回同一 owning graph/document 的真实 class/member/event/node/pin 身份、对应 callable 源位置、编译/VM generation，以及找不到/过期/能力不可用的原因。这里只有字段约束，**没有已实现的 SourceLocator DTO/API**，不向 `.zui v2` 添加 graph 字段，不把 binding.id 或 route 拼成虚构 event node。类/变量/接口 schema、graph 持久化/undo、UI 事件参数到 callable 的 lowering、graph 编译诊断与运行 source map、执行完成结果关联目前均未交付；有真实后端前 Graph 入口保留不可用原因，不建立静态假图。

本轮只改本域文档和回执，没有修改 Widget、VM、材质、公共 host/GotoDefinition/focus 或 schema 源码；未运行插件/Rust/EXE/新截图。既有 Prefab adapter 及其 pending 验证原样保留；没有把本轮盘点计作实现或 passed/accepted。

Parent 最新仅将 `zircon_editor/src/ui/graph/mod.rs` 中现有 `GraphDeltaProofV1` / `PreparedGraphDeltaV1` 的重导出委派材质 owner；Actor/Widget 作为消费者，不并行改该文件、不绕过 retention/prepare/一次 permit，不把导出可见性当事件图编译/执行后端。插件测试入口按当前正式 wrapper selector 核查：本轮 installed cdd14500 的 parser 明确仅接纳材质单包 `material_graph_` recipe，正确 Prefab 参数的 QueryOnly 仍被拒绝；公共 owner 交付本域单包支持后才能接续原 Prefab identity。详 [当前入口证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/plugin-route-02.json)，不借材质 filter 或原能力修复 identity。

## 本轮设计消费者参数与实际导出缺口

Actor专属file/page沿原身份，最终revn12仍117对象/55 flex/2业务主件/12 copies；业务component UUID、main、modifiedAt不变，非公共控件且无Engine component ID。本域source/override实例保留稳定shape-ref和原夹具值，未接生产读取或写命令。[flow-05](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-domain-flow-05.json)给出最终独立身份、原生重排、8项Penpot源码SHA及逐图结果；不将同文件业务实例当已链接公共库。

给AB-SHARED-01/02的具体参数提案：640内容区初始Left94.8、Viewport355.2、Details182，两separator合计8；182来自当前真实字体Roboto13.333333下来源URI advance181.8943取整。URI保持原值并完整可见，普通属性/警示自然换行，左栏标注自动两行32高。Parent仍需确定正式UE字体/组件padding、窄树可达/停靠策略、属性行完整复制与tooltip/焦点/滚动合同；字体切换后须重测，不把182注册为全局常量或强制所有窗口采用该分配。本域未改公共Dock/Tree/Property、tokens/catalog/Starship。

给Parent/原pipeline owner的具体缺口：实际中文“导出1个元素”请求`http://127.0.0.1:19001/api/export`返回404、页面内部错误、0下载，见[receipt-11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-browser-01/receipt-11.json)。PNG1×预设已保存，后续fresh context文件正常；需核对当前路由/导出服务接续，不需重建文件或清库。此原生PNG边界与官方ZUI插件export/原asset owner写回是两个尚未完成的环节，本域没有用截图/静态SVG绕过。

12:57Z独立只读公共file仍revn0、1根对象、0 components/colors/typographies、isShared=false。下一项可消费输入仍是实际发布库/component/style/version及正式codec映射；收到后复用，不复制主件。现行Jenkins的Prefab单包范围、真实Prefab文档handle/producer、Actor normal host/case/EXE资源闭包及桌面时段仍需对应owner交付。AB-WG-01～07继续是实际能力/缺口合同，未因设计重排而升级为脚本图执行。

## Widget 第三次接续：Definition 定位合同

接续 Parent 的 WG-C07。以下 AB-DEF-01～08 是现有来源与必要适配边界，不是已实现的 SourceLocator DTO、handler registry 或图后端。48 项输入指纹、三项限定搜索、固定版本比对和候选路径见 [Definition 源码回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/widget-definition-locator-01.json)。本轮只写本域文档/报告，不取得 Widget、公共 host、Runtime 或外部 VM 仓库写权。

### 现有 API 与可提供的定位精度

| 入口 | 已核对的实际能力 | 当前不能据此承诺的能力 |
| --- | --- | --- |
| [UI 源索引](../../../zircon_editor/src/ui/asset_editor/source/source_sync.rs:16)、[session 导航](../../../zircon_editor/src/ui/asset_editor/session/navigation_state.rs:25) | `UiAssetSourceOutlineEntry` 含 node_id、block_label、line/end_line/excerpt；`entry_for_node` 查源节点块。`select_source_line` / `select_source_byte_offset` 可移动源选择。 | 未发现 binding/event 的独立精确声明 span；节点块首行不能冒充 binding 行框。Source 模式仍为 UI 源文本。 |
| [VM 注册表](../../../zircon_runtime/src/script/vm/host_interface/registry.rs:228)、[操作贡献](../../../zircon_runtime/src/script/vm/host_interface/descriptor.rs:87) | 实际 `VmEditorOperationRegistration` 关联 operation 与 callback；`resolve_callback` 还原已登记的 module/export 名，并用调用者提供的 active generation 更新句柄。 | callback 数值索引不含源码位置；该方法本身不证明 active generation 是当前值。`resolve_package_callback` 会 intern callback，不应当作只读定义探测。尚缺 Widget handler → 此注册的实际消费者。 |
| [C 定义查询](E:/Git/zr_vm/zr_vm_language_server/include/zr_vm_language_server/lsp_interface.h:386)、[实现](E:/Git/zr_vm/zr_vm_language_server/src/zr_vm_language_server/interface/lsp_interface.c:2205) | `ZrLanguageServer_Lsp_GetDefinition(state, context, uri, position, result)` 接真实源 URI/游标，返回 location 列表；可指向物理源或 native 虚拟声明文档。语义查询还提供 `ResolveAtPosition` / `AppendDefinitions`。 | 它不是 module/export 字符串到位置的查询，也不是 Graph API。没有源 URI/语义引用位置时，不能猜文件路径或取同名函数的第一处文本。 |
| [语义 symbol](E:/Git/zr_vm/zr_vm_parser/include/zr_vm_parser/semantic_query.h:105)、[快照](E:/Git/zr_vm/zr_vm_language_server/include/zr_vm_language_server/lsp_semantic_snapshot.h:16) | 当前快照可提供 symbol/owner、reference/declaration/definition range；外部元数据有 owner/provider generation/token/signature hash；快照有 document/project/provider/semantic generation 与 dependency fingerprint。 | symbol ID、AST、字符串及 analyzer 均有当前快照/owner 生命周期。没有已确认的跨编辑持久 class/member/event/graph UUID 映射；provider generation=0 表示不可用。 |
| [Rust safe binding](E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding/src/lib.rs:317)、[sys binding](E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding_sys/src/lib.rs)、[sys 构建声明](E:/Git/zr_vm/zr_vm_rust_binding/rust/zr_vm_rust_binding_sys/build.rs:201) | 当前 ProjectWorkspace 暴露编译、artifacts、manifest、run/call/session；manifest 含 module/hash/imports/artifact 路径。 | safe/sys 源没有上述 LSP 定义查询出口，sys 构建设置 `BUILD_LANGUAGE_SERVER=OFF`；manifest 不含成员声明文件/span。不能将 C 源码存在写成 Zircon 已有可调用服务。 |

[插件 Cargo 声明](../../../zircon_plugins/zr_vm_language/runtime/Cargo.toml)固定 `zr_vm` 为 `3c1352ecb6ddfab02752d46682cc376558d5f481`，可选 feature 为 `backend-zr-vm`，default 为空；外部只读 checkout HEAD 是 `447abb1e8efb44203b353671e9a0417173e16d58`。本次用只读 `git show <pin>:<path>` 在内存核对 11 项：上述查询/快照/语义声明、LSP 入口实现与 safe/sys lib 在 pin 已存在；safe/sys 无定义出口、pin 的 build.rs 仍关闭 LSP。`lsp_semantic_query.c` 与当前 HEAD 不同，已另外核对 pin 的 43～63、3020～3069 行，确有文档版本门槛和真实声明 URI 要求。没有 fetch、checkout、源码副本、构建或运行查询，不能据此确定当前 EXE 链接版本/能力。

### 身份、目标与失效合同

| 合同 | Widget / Actor 提供的事实与 Parent 接续要求 |
| --- | --- |
| AB-DEF-01 源身份 | Widget 交 owning asset URI/source_path、真实 owning component/root 归属、source_node_id、原 binding ID/event/component_event、原 target/payload/property 映射、invocation ancestry，以及已有 repeat stable item key。owning component 必须由实际 [asset root/components](../../../zircon_runtime_interface/src/ui/v2/asset.rs:40) 与当前实例上下文解析；`UiTemplateNodeMetadata.component` 是控件类型，不等于声明所属组件。缺失或多义时拒绝猜测。 |
| AB-DEF-02 源声明位置 | 精确 binding 声明位置需要 Widget 在现有 source index 上补同一解析文本/修订的 span 与 owner 映射。当前仅可明确标记“节点文本块”的降级位置；不是精确 binding 定义或外部 callable。保持原 binding ID，不把 route/action 当成员 ID，不新增隐藏 ID 表。 |
| AB-DEF-03 实际 callable 准入 | Parent 从当前 handler/command registry 解析真实目标，并保留 registry generation。只有实际 VM 贡献才交 package/source 身份、slot/active generation 与登记 callback 的 module/export；manifest name/version 为辅助，不能在包重名时选第一项。slot/数字 module/function 只用于当前注册，不充当持久脚本成员。纯本地 targets 赋值可合法没有外部 callable。 |
| AB-DEF-04 定义结果 | 由真实 provider 交返回 URI、文档种类、range、目标文本修订/hash 与语义快照身份；物理源、只读 native 虚拟声明、真正 graph 目标分开。图目标必须已有资产/graph/member/node/pin 和 revision；当前类/接口/构造/事件图没有该映射，继续 open。C `GetDefinition` 的多个 reaching-definition 候选全部保留，不能取首个当函数定义。 |
| AB-DEF-05 修订与生命周期 | 请求关联 document/session incarnation、`source_buffer.revision`、原文本 hash、last_valid_source_generation、compiled binding generation，以及适用的 preview incarnation/request。VM 结果再关联 active slot generation 和 document/project/provider/semantic generation/dependency fingerprint；响应发布及跳转前重新验证。当前脏源有诊断时，不把 last-valid 编译 binding 的位置直接套到当前文本。禁止靠 offset clamp 接受过期结果。 |
| AB-DEF-06 文本与所有权 | UI 源行号为 1-based，buffer offset 为字节；LSP line/character 为 0-based UTF-16。使用同一权威文本转换 CRLF、中文和代理对；内部 [position codec](E:/Git/zr_vm/zr_vm_language_server/src/zr_vm_language_server/interface/lsp_position_codec.h)不是已暴露 Rust API。结果复制为拥有所有权的 URI/range/身份后释放查询/快照，不能保留 borrowed AST/string/analyzer 指针。 |
| AB-DEF-07 完成与导航 | Widget 仍交原输入/binding 结果；Parent 分别关联实际外部操作完成/error/cancel 与 Definition 查询回执。导航本身不调用、编译或实例化脚本。Parent 在当前目标仍有效时打开真实 document、选择范围、恢复 trigger/focus；UI 文档可复用现有 open/source 接口，脚本文本/虚拟声明/图的目标 editor 注册仍需实际 owner。UI Source 打开不算 Graph 导航或外部 handler 完成。 |
| AB-DEF-08 无定义与诊断 | 明确区分来源缺失/多义、binding 不存在或重复、当前源无效、代际过期/取消、实际 handler 缺失、provider 未接、纯本地赋值无外部定义、仅虚拟声明、多个目标、memory package 无源 URI、class/graph backend 缺失。上述是必要语义，不是声称已有的新 enum。C `GetDefinition=false` 及 stdio 空数组合并了无目标/查询失败；无独立证据时只能报告这两者未区分，不能编造“符号不存在”的原因。 |

`ZrLanguageServer_LspSemanticSnapshot_Acquire/TrackDependency/Validate/Release` 是可复用的生命周期依据，但 header 已有 TODO：`Lsp_FindAnalyzer` 忽略 TrackDependency 失败，实际实现仍有丢弃返回值。新桥接必须处理失败；本轮没有修复它或证明全部传递依赖已可靠 fenced。快照只独立保活文本，AST/analyzer/context 仍借用；外部 token/hash 的值复制也不延长 provider 生命周期。

### 精确候选路径与归属

| owner | 现有路径及必要变化提案；本轮均未实施跨域修改 |
| --- | --- |
| Widget | `zircon_editor/src/ui/asset_editor/source/{source_sync,source_buffer}.rs`、`session/navigation_state.rs`、`session/lifecycle/source_outline_cache.rs`：沿现有缓存补精确 binding 声明范围和真实 owner；current/last-valid 文本与 generation 分开，不另建 source store。 |
| Parent | [host open](../../../zircon_editor/src/ui/host/asset_editor_sessions/open.rs:31)、[navigation](../../../zircon_editor/src/ui/host/asset_editor_sessions/editing/navigation.rs:79)、[source](../../../zircon_editor/src/ui/host/asset_editor_sessions/editing/source.rs:5)、既有 `editor_operation_dispatch.rs` / `core/commands/registry.rs`：准入当前 handler/provider，验证目标文档，再调用已有 UI 选择接口；其它文本/图目标须有真实 editor 注册。公共 history/focus/完成回执仍由 Parent 接，不在 Actor 另建目录或入口。 |
| Parent 指定 script/VM owner | 既有 `zircon_runtime/src/script/vm/host_interface/registry.rs` / `runtime/vm_plugin_manager.rs` 交一致 active-owner/注册符号；`zircon_plugins/zr_vm_language/runtime/src/real_backend/package.rs` 当前只编译/启动，不能冒充定义 provider。需要公开安全只读查询时先交精确 diff，不让 Editor 直接依赖 pub(crate) 生命周期函数。 |
| 外部 zr_vm language/binding owner | 上表 safe/sys lib、sys build.rs 与已有 C LSP/snapshot API：核实复用既有语言上下文/transport或安全 binding 的最小出口、依赖与结果寿命。当前未确认任何已安装/运行 transport；不强开新服务、不 cast runtime 私有指针、不为导航新建 VM。外部仓库本轮只读。 |
| Actor / 材质 | Actor 提供本表并在真实 class/member/graph backend 就绪后做域适配；当前没有可交付的 graph 源定位。材质继续只供中立 Graph 表面，不改其 compiler 或共享 `graph/mod.rs`。 |

UE 依据为 [FBlueprintEditor::JumpToHyperlink](../../../dev/UnrealEngine/Engine/Source/Editor/Kismet/Private/BlueprintEditor.cpp:4601) 与 [BringKismetToFocusAttentionOnObject](../../../dev/UnrealEngine/Engine/Source/Editor/BlueprintGraph/Private/Kismet2/KismetEditorUtilities.cpp:2438)：它们持真实 Graph/Node/Function/Blueprint 对象进入目标文档。Widget route 字符串没有这些对象身份，不能复制“切 Graph 模式”的表象。

已存在的 VM 源码回归覆盖快照/直接及传递依赖/provider 变化、reaching writes/多分支/无快照源拒绝；仅盘点，未运行。后续 owner 应验证同名包/导出、重复 UI 实例、无效当前源、热重载/关闭/取消、中文/CRLF/代理对和多个/虚拟/缺失目标。文档结构检查不替代这些运行测试；本阶段插件/Rust、正常 EXE、DPI 和双端验收仍 pending，passed=0/accepted=0。

## Prefab：原宿主读取口与调用者的精确候选

接续 Parent“Actor 第二次接续”的真实 query/绑定/事务要求。原交付 [三文件未应用 diff](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-host-injection-01.patch) 和 [50项输入源码回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-host-injection-01.json)。patch SHA256 `eadd42e718a3cd1db94e9779dead5e00f0bff2fba93cadd53127f5b65b626066` 保留为历史审阅输入，不整体重放。**Parent 第三次接续已仅实施定向 view/template/capability 校验及 callback 后的 contributions generation/能力复核；场景 reader/stamp 未应用。**本域未改共享宿主或原四份 Rust，没有注册数据源、固定 fixture、公共 trait 或第二套 history/store。下表保留原候选内容，当前接续见本节末尾 AB-PF-07～14。

### 当前源码与候选分别提供什么

原候选记录时，[定向 pane 收集器](../../../zircon_editor/src/ui/retained/app/host_lifecycle/pane_payloads.rs:82) 丢弃 target_instance_id。Parent 现已将真实 ViewInstanceId 传入 [定向 host 查询](../../../zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs:192)，并在锁外 callback 前后核验。`EditorUiTemplatePaneDataSource::snapshot()` 仍无参，source 仍由 template ID 选择；批量路径与 [pane 消费](../../../zircon_editor/src/ui/layouts/windows/window/pane_projection.rs:474) 仍按 template ID 共享结果。已有 guard 不提供逐窗口的 Prefab document/query 语义，不能只注册一个全局 source 就声称多资产隔离。

| 原候选路径；owner 为 Parent | diff 内容及当前实施边界 |
| --- | --- |
| `zircon_editor/src/ui/host/editor_event_runtime_access/extension_access.rs` | 添加私有 `template_pane_view_is_current`，用现有 ViewRegistry 的 instance/descriptor 定向查询核对活跃 view、能力与 pane body template；定向 `ui_template_pane_data_snapshot` 增加 instance_id 并先核验。无全量 view 枚举，无公共 DataSource trait 变更。 |
| `zircon_editor/src/ui/retained/app/host_lifecycle/pane_payloads.rs` | 在原 target 分支保留 `(instance_id, template)`，将真实 ViewInstanceId 传给上述查询。批量路径和 template-keyed map 不改，不能把此 ID 传递当成逐窗口 producer 或缓存隔离已经完成。 |
| `zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs` | 新增私有 `read_template_pane_scene_component` 及只读结果/stamp/error。仅当前已提交场景、Edit domain、单一真实选中实体；经原 `EditorAuthoringWorld::with_world` 拷出该实体一个动态组件的原 Value、数据/schema generation 与 schema 是否声明。返回 scene/project/source identity、完整 gateway identity、view/template、selection generation/entity；不克隆全世界、不借出 World/row、不读盘/编译、不产生写 permit。 |

候选 source 字段来自现有 [ActiveSceneDocumentIdentity](../../../zircon_editor/src/core/document/lifecycle.rs:110)、[权威 World 借用](../../../zircon_editor/src/core/editing/authoring_world.rs:170)、[组件值/schema 查询](../../../zircon_runtime/src/scene/world/dynamic_components.rs:196) 和 [组件数据代际](../../../zircon_runtime/src/scene/world/generation.rs:132)。`same_activation` 不比较 source revision，因此完整结果比较必须包含 scene identity 的 source revision，不能只调用该方法。网关失败保留 typed error，不变成正常空数据。

### 业务读取、绑定与旧结果失效

| 合同 | 当前可复用事实与必要接续 |
| --- | --- |
| AB-PF-01 真实实例来源 | Runtime [prefab_instance_for_record](../../../zircon_runtime/src/scene/world/project_io/prefab.rs:7) 从 `zircon.prefab.instance` 保留元数据反序列化为 PrefabInstanceAsset。插件声明的 `prefab_tools.Component.PrefabInstance` 则是另一完整类型路径，不能把两者的值或 schema 偷换。只读保留元数据可存在于未声明 schema 的场景；schema_declared=false/generation=0 不开启编辑。 |
| AB-PF-02 域内调用 | Parent 在原宿主组合点得到有效读结果后，使用既有 Runtime 类型的真实解码结果调用 `PrefabInstanceAuthoringSnapshot::new(&instance, source_state)`。本域保留原序、原值、所有重复诊断位置；source_index 只在该次 snapshot 有效。域 crate 的 serde_json 目前仅 dev-dependency，不擅自加生产解析器/manifest；原 Runtime 场景加载已提供 typed 资产，不另造模型。 |
| AB-PF-03 来源查询与 asset revision | 场景 identity 的 source_revision 只指 owning scene，不是被引用 Prefab 的 revision。真实 Prefab 查询由当前 Project/AssetManager 按原引用解析/加载；[现有 catalog_snapshot](../../../zircon_editor/src/ui/host/assets/api.rs:39) 的 project_root、catalog_revision/publish_epoch 与 record.source_hash/mtime 可关联观察身份，但目录中存在不等于 load 成功/preview ready。源查询在 shell/World 借用外执行，实际结果再关联请求与当前资产版本；I/O/解析失败不得伪装 Missing，未核实时保留 Unchecked。 |
| AB-PF-04 旧结果与关闭 | 返回 stamp 是观察值，非 retention/mutation permit。延迟发布前重读并比较 view/template、完整 scene/gateway、selection/entity、组件数据/schema generation，再核对真实来源查询的 asset revision/request。选择、项目、场景、组件或 schema 变化均退休旧结果，清理失效绑定/行选择，不留旧成功值。当前 ViewInstance 没有已证实的 incarnation；关闭后同 ID 重开即使 stamp 相等，也须由 Parent 原 close/reload cancellation/retention 排除，不能凭字符串再次存在接受旧结果。 |
| AB-PF-05 原绑定与生产者缺口 | 保留原 ZUI 的31 nodes/29 control IDs/15 events/7 imports。真实绑定须沿既有 values/component_patches，携带同一 view/template/request 和原源身份。现有无参 source 及 template-keyed map 尚未解决该合同；Parent 需先交私有宿主消费/发布组合点，不能注册不可变 fixture source 或以共享全局选择代替 owning document。候选本轮未提供 producer，也未让按钮获得假成功。 |
| AB-PF-06 事务与类/资产区别 | 上述仅是场景所选实例读取，不能推断 `editor.prefab_editor_window` 的资产文档归属，也不能当类默认值/Blueprint backend。资产 document/revision、当前字段 authority、真实事务/inverse/save 尚待宿主交付；apply/revert/break/create-from-selection 继续不可用。后续写目标至少保留当前文档/实体/完整组件路径与修订及原 entity_path/property_path/value；重复路径保持歧义诊断，不凭 source_index 写入。沿原 scene/asset history，不扩公共 engine/history/VM/schema。 |

Parent 后续公共发布范围的精确依赖是原 `retained/app/committed_shell_state.rs`、`host_lifecycle/pane_payloads.rs`、`layouts/windows/window/pane_projection.rs` 和 `ui/ui/data/visual/product_presentation/product_state.rs`。这些现有生产/消费路径共同依赖 template-keyed 结果；本域不并发改 key、增加隐藏 map 或新 registry。应先明确逐 view 的真实查询/结果退休方式，再让本域四文件适配消费 typed instance/source result。原 `EditorUiTemplatePaneDataSource` 公共 trait 保持不动。

diff 初版和格式化版的 hash 留在原回执；原阶段格式化只在 stdin 中运行已安装 rustfmt，三个原文件本已合规，未重排他人代码。原 `git apply --check` 仅是当时的适用性检查；Parent 已改动两个目标文件，旧 patch 不代表当前可直接应用。原检查与本轮 source/API 核查均不证明 Rust 类型检查或执行。本域本轮未运行 Cargo/插件/Jenkins、启动 EXE/桌面、点击 PNG 导出或重启服务；正常 Actor case、所有画面/事务和类图仍 pending，passed=0/accepted=0。

### Parent 定向 guard 后的只读数据合同：AB-PF-07～14

以 [readonly-identity-02 实际源码回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/prefab-readonly-identity-02.json) 接续。这里定义原 adapter 的输入与结果解释，不新增 Rust 类型、资产 schema、publisher 或公共接口。Parent 的四项 `targeted_template_pane_` controller 回归属于 Editor scope；原 Prefab 请求仍为 #348 preparation-conflict、compileStarted=false、八项源测试执行零。两组不能合并计数或借用结果。

| 合同 | 真实 API / 字段与消费者规则 |
| --- | --- |
| AB-PF-07 文档、视图与实体身份 | [ActiveSceneDocumentIdentity](../../../zircon_editor/src/core/document/lifecycle.rs:110) 提供 DocumentId、project_root、scene_uri、activation_revision、source_revision；它不是 Prefab 源资产身份，`same_activation` 也不比较 source_revision。场景实体是 [SceneEntityAsset.entity/parent](../../../zircon_runtime/src/asset/assets/scene/entity.rs:25) 的 `u64` / `Option<u64>`，World 加载/保存保留该 ID。行选择至少限定到实际文档激活与 World/gateway，再用 entity ID，不能用数组位置或 Mesh/LootSocket 标题。ViewInstanceId 的关闭重开同 ID 问题仍由 Parent 生命周期解决，现有 guard 未证明 incarnation。 |
| AB-PF-08 来源树与实例树 | [PrefabAsset.scene](../../../zircon_runtime/src/asset/assets/authoring.rs:228) 内嵌 SceneAsset；来源树直接保留其 entities 的 entity/name/parent，不把源 entity ID 当当前场景实例的 ID。实例树读取当前 owning scene 的实体与组件。现有 PrefabInstanceAsset 只有 prefab/local_transform/overrides，未找到 source-entity → instantiated-entity 映射、继承/SCS 成员 GUID 或组件 attachment 父字段；不能把两个同值 u64 合并，也不能把 entity.parent 解释成 UE 组件挂接。源码 DTO 存在不等于已实例化/验证 World。 |
| AB-PF-09 组件行身份与权限 | [World.dynamic_component](../../../zircon_runtime/src/scene/world/dynamic_components.rs:256) 按实体和完整 component_id 唯一取值；`dynamic_components_for_entity` 返回按完整类型路径排序的独立副本，descriptor 可为空。持久 [SceneComponentAssetRecord](../../../zircon_runtime/src/asset/assets/scene/component.rs:11) 有 type_id/schema_id/schema_version/provider_id/payload/references；加载器拒绝同实体重复类型、重复 schema/provider 或不匹配提供者。它没有每行实例 UUID，也没有组件 parent。内建可选字段按真实字段/codec 区分，不用本地化标题当类型。不声明 schema、已撤销能力或没有操作 factory 时只读；复制出的 Value 改动不写 World。component data generation 是整个类型的修订，不能宣称是某实体的版本。 |
| AB-PF-10 引用、真实来源与确切载荷版本 | [AssetReference](../../../zircon_runtime_interface/src/resource/asset_reference.rs:25) 保存 UUID 与 locator；[resolve_asset_id_for_reference](../../../zircon_runtime/src/asset/registry/query.rs:80) 以 UUID 为 authority，locator 仅提示，不回退绑定占用旧地址的新资产。`stale_path_for_uuid` 可报告迁移地址，读取不自动改写引用。[load_prefab_asset](../../../zircon_runtime/src/asset/pipeline/manager/assets/loading/load_asset.rs:128) 能加载真实 PrefabAsset，但返回独立 clone，没有确切载荷版本；目录 source_hash/mtime、catalog_revision/publish_epoch 也不能与该 clone 拼成原子版本。现有 ResourceManager `projection_snapshot().active().key(id)` 与 [snapshot_expected::<PrefabMarker, PrefabAsset>](../../../zircon_runtime/crates/zr_resource/src/manager/payload_ops.rs:88) 可在同一 authority 锁下核对 active key、kind、实际载荷与 record；key 含进程内 remove/recreate identity，revision 取同一 snapshot。它不主动加载或建立 residency lease；未驻留、过期或类型不符不能写成 Missing。继续消费现有加载任务/错误，不造 Prefab load trait 或版本时钟。 |
| AB-PF-11 override 行与排序/过滤 | 原 [PrefabOverrideRow](../../../zircon_plugins/prefab_tools/editor/src/authoring/snapshot.rs:31) 只借用本次 instance，source_index 对原 overrides 向量有效。排序/过滤只能携带同次 snapshot 的原行引用/位置，不重排源向量。相同路径甚至相同 value 可重复，路径对、内容 hash、当前视觉行号均不能成为持久 target。输入、拥有文档或来源版本变动后清空旧行选择与诊断索引，重新投影；没有经过原事务 owner 的目标解析与修订核验时，不把点击变成写命令。异步工作不持有该借用，结果回到原 host 后再核验并建立借用，不另造 authoring store。 |
| AB-PF-12 entity_path 的解析边界 | [EntityPath::parse](../../../zircon_runtime/src/core/framework/scene/entity_path.rs:57) 会裁剪段并略去空段，而原 override 诊断按完整原文路径配对，不能提前 normalize/合并。现有 [World.entity_path/get_entity_by_path](../../../zircon_runtime/src/scene/world/property_access/path_resolution.rs:8) 会按同级重名附 `#entity`、空名使用 `Entity<id>`，应复用其真实 authority，不按树显示文字重新拼接。Prefab override 的路径相对根、源/实例命名空间与映射尚无已确认执行合同；在合同到达前只显示原文与已有诊断，不选“第一个同名节点”。 |
| AB-PF-13 property_path 与解析目标 | [ComponentPropertyPath::new/parse](../../../zircon_runtime/src/core/framework/scene/entity_path.rs:95) 已有 `CR-FRAMEWORK-SCENE-0002`：parse 把第一个点作为组件/字段边界，不能恢复 `prefab_tools.Component.PrefabInstance` 这样的完整类型。后续从已核验的 component_id 和 property segments 调用结构化 new；不能从 override 字符串猜最长前缀，不能改写存量原文。已有 [compile_scene_property_target](../../../zircon_runtime/src/scene/world/compiled_binding/property_path/compile.rs:28) 要求可变 World 并驻留路径/字段身份，不是无副作用的只读定位函数；其 [is_current_for](../../../zircon_runtime/src/scene/world/compiled_binding/property_path/model.rs:248) 不核对跨 World 身份或字段 schema，更不授予 Prefab 写入、Definition 或 Graph 执行能力。 |
| AB-PF-14 原 adapter 的实际消费次序 | Parent 先提供真实 per-view owning document/生命周期与输入读取点 → 原加载 owner 在宿主/World 借用外解析 UUID、加载并取得载荷/版本与 typed error → Parent 原生命周期核对 view、document/activation/source、gateway/selection/component/schema、source active key/request → 本域在有效 typed instance 上调用 `PrefabInstanceAuthoringSnapshot::new` → 原 values/component_patches 按保留的 control/binding/event ID 投影。仅来源确实不可解析/缺失时 Missing；未查询/待加载保持 Unchecked，IO/解码/类型/权限错误单独保留，不误报空数据。任何过期结果先退休并清理旧展示，不能用当前全局 selection 替代 asset document。现有无参 callback 与 template-keyed batch/payload 不满足前置条件，未注册 producer，也未补 scene reader/stamp。 |

UE 对照并不支持把普通实体树涂成类组件树：[FSubobjectDataHandle](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectDataInterface/Public/SubobjectDataHandle.h:25) 的相等/哈希基于当前数据指针，属于活跃数据句柄；[USCS_Node](../../../dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/SCS_Node.h:30) 则保有 ComponentTemplate、ParentComponentOwnerClassName、ChildNodes 和 VariableGuid。[SSubobjectEditor](../../../dev/UnrealEngine/Engine/Source/Editor/SubobjectEditor/Private/SSubobjectEditor.cpp:727) 使用 CanReparent、继承与实例状态决定操作。Zircon 当前缺对应作者映射时，继承标识、类默认值、构造图和挂接写命令仍是能力缺口；不能由 name/parent 或 VM callback 推导。

给 Parent 的最小下一项是现有 per-view document/producer 生命周期组合点及真实实例/来源输入，而不是扩公开 DataSource trait。该输入到达后，本域四文件继续只读投影与诊断；写 factory、保存/undo、Graph/Definition、正常 EXE case 与完整画面验收分别保留原 owner 和开放门槛。原 patch、#348、原31节点/29control/15events/7个 widgets imports 均保留；imports 表有 widgets/styles 两组，计数应取 widgets 的七项，不能把分组数二当导入数。

## AB-SHARED-07：Prefab来源内容的640可达合同

[内容11](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/actor-blueprint/penpot-prefab-workspace-11/result.json)仅在本域文件消费现有native linked instances，未改Shared或另建公共控件。1280/900只读fixture已保存重开；640实际center18宽、render高1806/viewport520，不能据同文件发布关闭窄窗。

| 实际owner | 所需最小合同 | 本域边界 |
| --- | --- | --- |
| Parent公共Dock/Tab/host | 复用pane view identity、visibility/activation和focus返回；640可进入/退出Hierarchy、records、Details/Source，保持原document/key/selection，不按标签解析target。若采用scroll，交原scroll容器及输入/writeback/首尾合同。 | 只负责业务数据与稳定target；不新建共享Tab/Scroll/theme，不扩公共trait。 |
| 原public source-owner | TreeRow灰色action-slot与leaf disclosure的合法公开参数/source projection，保留copy/source/slot/shapeRef及内部flow；Button disabled源prop已存在但catalog未发布，需明确合法公开/导出面。 | 只做本域覆盖与绑定；不可用caption不等于disabled handler或校验已完成。 |
| Parent真实per-view producer | 同项目registry/resources、原来源key/snapshot、document/view/activation/selection/world/schema上下文；发布前重校验原key及上下文并退休旧结果。 | source08实体parent不是attachment；重复record的显示ordinal不是写target，无持久组件identity时保持只读。 |

正式ZUI导出、正常EXE和双端测量仍缺，Penpot数据与浏览器可见不授予上述能力或验收。

