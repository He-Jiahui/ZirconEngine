---
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/dirty_state.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/transform_validation.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/component_mutation_effects.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/scene/module/level_manager_contract.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/workbench/startup/editor_state_construction.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_runtime/runtime/07/failure-2026-07-22-world-derived-state-full-rebuild.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/SceneComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SceneComponent.h
tests:
  - zircon_runtime/src/scene/tests/derived_state.rs
  - zircon_runtime/src/scene/tests/derived_state
  - zircon_runtime/src/scene/tests/ecs_hierarchy_structure.rs
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - direct rustfmt check 20/20 passed
  - managed Windows zircon_runtime lib-test compile failed before focused tests ran
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene派生状态、层级与变换current-source结构性能复审（2026-08-14）

## 范围、快照与当前进展

本轮沿`Hierarchy/LocalTransform/ActiveSelf -> DerivedStateDirty -> internal scene systems ->
WorldMatrix/ActiveInHierarchy/NodeCache -> render/inspection/world-sync`完整复审 **11/11个生产Rust文件、
4,901行、4,518个非空行、8条内联测试**，并反查 **9/9个直接测试文件、2,072行、46条测试**。
生产与测试manifest指纹分别为
`5A540455B088AFF2B95D9746890A50EA894EFAC0CB34307A3E40A03D518FEA85`和
`9D9E1FB3333D07C5D4507D89009058880908F2DF36415062E08D4ED1DF959534`。相关生产文件和测试均被
Runtime08活跃Session纳入write scope；本轮只读，不覆盖其未提交实现。

相对2026-07-22的PERF-MVP-459报告，当前工作树已有两项真实改进：`HierarchyMutationIndex`持久维护稳定
root/children顺序，active与world transform不再各自重建临时children map；subtree、active和world传播已改为显式
DFS栈，100k深链不再依赖递归调用栈。domain dirty为false时internal system也能直接跳过。以上是必须保留的基线，
但它们只消除了重复建索引和递归风险，没有解决局部变更扩大为全世界工作的根因。

Runtime07的`.codex/outbox`存在dirty-frontier、topology和node-cache-delta候选补丁，但它们不是当前生产源，也没有
current-source Cargo或产品trace；本报告不把候选工件记为已修复。

## P0：局部变更仍扩大为串行全世界计算和写回

`DerivedStateDirty`只有hierarchy、active、transforms、node_cache、render_extract五个模块级bool，没有changed entity、
dirty root、subtree range或generation delta。一次局部`LocalTransform`写入会标记WorldTransform、NodeCache和
RenderExtract；一次`Hierarchy`写入会标记全部五个domain。随后当前实现从所有root遍历：active为每个entity调用
`replace_derived_component`，world matrix又为每个entity调用一次，NodeCache再遍历所有stable entity并复制宽
`SceneNode`。

`replace_derived_component`不比较旧值，直接使用当前mutation tick覆盖table/sparse component。因此未受影响实体不仅
重复矩阵乘和storage write，还会被伪标为`Changed<WorldMatrix>`或`Changed<ActiveInHierarchy>`，把一次局部变更放大到
下游change query。对N个entity、受影响子树K、平均深度D，当前源码的确定性工作量如下；这是调用结构计数，不是动态
耗时数据。

| mutation | 当前必做工作 | 目标规模 |
|---|---|---|
| stable frame | dirty system为0，正确跳过 | 保持0 |
| one transform | world traversal/write N；NodeCache visit N，且每个命名entity复制Name与最多17个optional component | topology rebuild 0；visit/write/clone近K，未变值tick write 0 |
| one active flag | active traversal/write N；NodeCache visit/clone N | visit/write/clone近K |
| one reparent | parent snapshot N + entity Vec N + 每entity ancestor walk，最坏O(N*D)；随后active N + world N + NodeCache N | changed edge验证近old/new affected roots；传播近affected union |
| Dynamic mobility preflight | `stable_entity_ids()`扫描N并逐项查parent，只为找direct static child | child visits近direct child count |

层级/active mutation还会在正式派生阶段前调用`mark_inspection_subtree_fields_dirty`，先分配一个subtree entity Vec并遍历K。
reparent为命中旧父链和新父链watcher，在insert前后各调用一次`record_world_fact`；这两次ancestor walk是正确语义，不能
简单删除，但当前每次都各自Arc clone、获取同一subscription Mutex、探测world tokens并执行fact index/merge。修复应让
一次reparent transaction在一个短锁域内输入before/after ancestry并只入队一个fact，而不是删除旧链失效。

## P0：RenderExtract错误依赖宽NodeCache投影

默认schedule在PostUpdate顺序执行HierarchyValidity、ActiveHierarchy、WorldTransform和NodeCache；RenderExtract阶段又运行
`RenderExtractPrepare`，后者显式补跑上述四项。可是实际render extract按MeshRenderer、lights、camera等ECS组件列直接
收集，并读取WorldMatrix/active等派生组件，不消费`World::nodes()`。因此只面向运行时渲染的transform变化也必须先重建
宽NodeCache：每个命名entity复制Name，并clone camera、mesh、2D render、五类light、physics和五类animation共最多17个
optional component，其中多个字段可继续深复制Vec/String/asset handle集合。

NodeCache仍有真实消费者：dynamic session启动选择默认节点、LevelSummary、editor viewport gizmo、selection/startup以及
多个authoring读路径。正确方向不是删除缓存，而是把它从render freshness依赖中拆出，作为editor/legacy consumer按
generation订阅的delta projection或显式snapshot。render、inspection和editor不得各自维护第二份hierarchy truth；它们
消费同一个derived change set，并只投影各自字段。

## P1：dirty期间重复读取按深度分配和计算

WorldTransform尚未flush时，每次`world_transform/world_matrix`读取都新建`Vec` lineage和`HashSet`，沿ancestor走到root后
再从root乘到leaf，单次O(D)且至少两个可增长容器；active projected read每次也新建HashSet并走ancestor。现有性能测试只在
两层层级上重复128次并打印`elapsed_us`，没有allocation、ancestor visit、cache hit或阈值断言。高频gizmo、script、physics
或多camera读取同一dirty entity时，单帧成本可退化为reads*D。

MVP应由dirty frontier一次计算并发布affected derived values；同一generation重复读取命中已发布值。若必须保留事务内
projected read，则至少复用transaction scratch/memoized lineage并以generation失效，不能让每个scalar getter拥有临时
图算法。

## 测试false-green与主线程边界

现有derived-state源码形状测试验证BTree/HashMap字段、显式DFS和reserve/clear，因而会认可一个仍然全世界遍历和全世界
写tick的实现。`ecs_hierarchy_structure.rs`还要求已经不存在的`self.entities.iter().copied()` mobility扫描，当前源码改为
`stable_entity_ids()`后静态即不满足；它既没有保护行为，也没有量化复杂度。应以确定性counter测试替换容器token。

`SceneScheduleRunner`只把声明worker-safe的native systems组成worker batch；每个internal derived system都会先flush worker，
再在`LevelSystem::with_world_mut`独占闭包内串行调用。这里不应直接把现有全量遍历丢进线程池：先收敛dirty roots和单一
topology，达到work-efficient后，再按affected roots数量/实体阈值把独立子树计算为worker-local delta，owner thread按稳定
拓扑顺序一次发布。小K应保持串行，避免job调度成本超过计算。

## Unreal主依据与结构方向

UE `SceneComponent.cpp:107-108`为UpdateComponentToWorld和UpdateChildTransforms分别声明cycle stat；
`760-826`按需先更新未初始化parent，在`810`比较新旧transform，只在变化时写`ComponentToWorld`，随后把changed标志传入
传播；`968-983`在`FScopedMovementUpdate`期间延迟传播；`984-1026`只对当前component更新bounds/render dirty并沿
attached children继续；`2909-2953`遍历direct children，跳过不需要继承的absolute child。

UE依据支持“持久attached topology + changed-value gate + affected subtree + scoped mutation合并 + 独立cycle counters”，不支持
Zircon每次局部变更重扫全World，也不意味着必须复制UE对象模型。Zircon应保留ECS column和确定性schedule，但让一次mutation
发布唯一`DerivedChangeSet`：changed edges、root-minimized transform/active frontiers、changed value ranges和consumer demand。

## 统一实施计划与责任计划回填

本轮不新建重复编号，补强既有PERF-MVP-459和open Runtime07 failure。实施必须按以下顺序，不允许先做线程池包装：

1. Runtime07+Runtime08把当前`HierarchyMutationIndex`收敛为唯一generation-owned topology，mutation transaction原子更新
   parent/direct-child/root/topological metadata，并维护去祖先覆盖的dirty-root frontier。
2. Runtime08让HierarchyValidity只验证changed edges；active/world按affected subtree迭代计算，比较旧值后才写component/tick，
   同一pass发布changed ranges。mobility直接消费topology child range。
3. Runtime07+Editor05将NodeCache从RenderExtractPrepare依赖中拆出，render/inspection/editor按同一change set和consumer demand
   增量投影；LevelSummary等只需计数的消费者不得强迫宽SceneNode刷新。
4. Runtime03+Runtime11在上述工作量门通过后，才为多个独立大dirty roots启用worker-local计算与稳定owner-thread publish；
   reparent before/after watcher失效在一个mutation publication内完成。
5. Runtime07把源码形状守卫替换为行为、visit/write/clone/tick/allocator和schedule counter；Runtime08/Editor05共同跑
   F2/F4矩阵并回传open failure。

## 动态验收矩阵与当前工具结果

entities为1/1k/100k，depth为1/64/100k，affected subtree为1/1k/100k，分别覆盖stable、rename、leaf/root transform、
leaf/root active、reparent、spawn/despawn和mobility。记录topology rebuild、parent/direct-child/entity visits、ancestor scratch
alloc、matrix/active computations、component/tick writes、NodeCache field/clone bytes、render/inspection/editor projection count、
worker jobs、main/worker wall、p50/p95/p99、RSS、cache miss、CSwitch、ReadyThread和energy。验收硬门为stable全部0；single
change工作近affected subtree；未变derived值write/tick=0；render-only NodeCache build/clone=0；100k深链无递归栈风险；并行
只在超过实测阈值后启用且总work不增加。

20/20个相关文件的direct rustfmt check通过。沿用本会话managed Windows证据：`zircon_runtime` focused lib-test在843.4秒后
因foreign current-source 361个编译错误失败，0条本切片测试执行；当前没有可运行Zircon二进制。WPR 10.0和xperf可用，
RenderDoc 1.44位于`D:/Tools/RenderDoc/renderdoccmd.exe`，Tracy profiler不在PATH。没有binary时不运行空WPR/RenderDoc
capture；RenderDoc后续只证明draw/dispatch/readback未回归，CPU hierarchy/transform瓶颈必须由counter、WPR/xperf、Tracy和
allocator共同证明。

本切片继续留在`pending.md`，不进入`review.md`；没有前后性能数据、current-source Cargo和F2/F4产品证据，因此不提交
性能里程碑，也不发送企微完成消息。
