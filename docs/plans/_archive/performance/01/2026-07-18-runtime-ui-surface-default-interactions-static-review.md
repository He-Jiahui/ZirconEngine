---
related_code:
  - zircon_runtime/src/ui/surface/surface/default_interactions.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions
  - zircon_runtime/src/ui/surface/surface/event_routing.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/IItemsSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/STreeView.h
  - dev/slint/internal/core/model.rs
  - dev/slint/internal/core/model/adapters.rs
tests:
  - unused pre-route hover clone source-level RED to GREEN guard passed
  - common scalar table sort borrowed-text source-level RED to GREEN guard passed
  - rustfmt check and scoped diff check passed
  - current-source Windows UI tests pending behind shared Cargo FIFO
  - route/control/table/tree/virtual-window scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI surface默认交互逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`surface/default_interactions.rs`、`surface/event_routing.rs`以及`surface/default_interactions/**`全部16个生产文件，共18/18。连同前批，`surface/surface`生产文件21/21静态读完，`zircon_runtime/src/ui/surface`当前批累计39/128，整个`ui`累计226/783。产品路径覆盖pointer dispatch、popup、radio/range/scrollbar、table、tree、timer与virtual scroll。

## PERF-MVP-283：输入事件重复拥有route并串行探测行为

pointer dispatch构造的route包含stacked/bubbled/entered/left等多组owned Vec；默认交互随后按range、scrollbar、table、tree、generic顺序逐项探测，binding/component-event识别还对descriptor id、route与action字符串做重复substring匹配。本轮源码RED→GREEN删除了routing前一份完全未使用的hover path clone，但route共享与预编译behavior dispatch仍未解决。

EditorUI01应让一次事件只拥有一份generation/event-lifetime route，并按命中owner的compiled behavior mask直接分发；popup外部点击应通过popup stack与indexed hit path判断，不能每次release逆序扫描全部arranged nodes。

## PERF-MVP-284：基础控件每次pointer move重做结构发现与解析

scrollbar drag每次move都按字符串target全扫`tree.nodes`；radio select递归扫描整组descendants再逐项mutation；range drag重复从metadata/component state读取和解析min/max/step/current。高频pointer move把静态控件关系、字符串解析和多次属性事务堆到UI主线程。

EditorUI06应在tree/component generation编译typed behavior context：owner/target/group成员、range scalar与canonical property identity直接索引；EditorUI01只提交pointer intent。结构或schema变化才重建context，move事件不得扫描全树或重parse静态TOML。

## PERF-MVP-285：表格交互全量投影与主线程排序

column resize用字符串drag token保存typed状态，每次move解析float、线性找column，并把完整`column_widths` map和`columns` array转换后多次mutation。client sort在UI线程解析并排序完整rows；常见String/Color/AssetRef/InstanceRef/Enum比较原先每次调用`display_text()`分配，O(N log N)放大。本轮已把该产品default-interaction comparator改为借用`&str`比较，语义不变；full sort、schema索引和atomic patch仍open。

EditorUI06应复用PERF-MVP-268的generation-owned typed schema/comparator与worker-budgeted row permutation，让resize/sort/selection一次提交canonical patch，不在UI state中维护多份alias authority。

## PERF-MVP-286：虚拟滚动每步写18个兼容别名

table与tree virtual scroll每次窗口变化都串行调用18次`mutate_property`，重复写`total_count/item_count/itemCount/row_count/rowCount`、visible/requested/overscan与scroll aliases。每项都可能产生binding report、style/dirty判断与String key分配；一格滚动被放大为18个同步事务。

EditorUI02拥有canonical visible-range/layout generation，EditorUI06拥有typed component patch与边界alias projection。二者应以一份`VirtualWindowPatch`原子提交canonical字段，只在外部绑定/序列化边界投影兼容名；unchanged字段和alias不得再次触发layout/render invalidation。

## PERF-MVP-287：TreeView重排与选择重复全树递归

tree support每次事件递归构建owned id Vec，`push_unique`线性去重最坏O(N^2)。reparent先flatten整树、flatten descendants、递归remove，再递归insert；insert在每个失败分支clone source subtree，最后又flatten结果。selection、rename与reorder分别串行写5至9个属性，drag token也反复String编码/解析。

EditorUI06需让PERF-MVP-267的linearized tree、id-to-row/parent索引真正成为default interaction authority；reparent先用索引定位source/target并对affected ranges做一次结构patch，selection/edit/reorder一次事务提交。UE ItemsSource/TreeView保留items source与linearized state，Slint ModelNotify/adapter按row changed/added/removed维护mapping，均说明事件层不应临时重建整棵owned模型。

## 责任计划与验收

EditorUI01收到route/default dispatch与popup全扫failure，EditorUI02收到virtual-window alias fanout failure，EditorUI06收到基础控件及table/tree重投影failure。以route depth 1/16/64、1k/10k/100k rows/nodes、10k pointer moves/scrolls/reorders记录route clone bytes、behavior probes、tree/metadata visits、property transactions、binding reports、alias writes、main/worker CPU p95与stale result age。current-source Cargo、MVP workbench table/tree/popup产品trace及layout/hit/像素对拍完成前，本批仍留`pending.md`。
