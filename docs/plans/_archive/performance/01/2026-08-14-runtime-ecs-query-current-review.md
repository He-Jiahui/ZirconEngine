---
related_code:
  - zircon_runtime/src/scene/ecs/query
  - zircon_runtime/src/scene/ecs/archetype/index.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/query_order.rs
  - zircon_runtime/src/scene/ecs/system/query.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-world-fixed-component-storage-and-stable-query-index.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityQuery.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassExecutionContext.h
tests:
  - current production slice 29/29 files and 2 inline tests statically reviewed
  - related query tests 21/21 files and 61 tests statically reviewed
  - direct rustfmt 50/50 passed
  - managed Windows zircon_runtime lib-test compile failed; focused tests and profiles did not run
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime ECS query current-source结构性能复审（2026-08-14）

## 范围、快照与已生效修复

本轮完整复审`ecs/query/**`、archetype query index、World query/order桥接共 **29/29个生产
Rust文件、5,756行、5,213个非空行、2条内联测试**，并复审相关 **21/21个测试文件、
2,524行、61条测试**。生产与测试快照指纹分别为
`33A7C754CC5B03396FACBD17573F23D3105EF2E3526D910128304988C5B1575C`和
`D0163C3C126390F0D0280160DE56EAE2ACA54E6B3E5B84AB35DBFFC386A7D001`；直接rustfmt
50/50通过。29个生产文件中22个为其它Session修改/新增，本轮不覆盖其源码。

7月PERF-MVP-466的“cache miss命中archetype后仍全扫World”结论已被当前源码修复：
`QueryState`按archetype generation增量编译`CachedArchetypePlan`，dense component绑定table column
slot，sparse component绑定storage种类；新增archetype只编译新增plan，不再缓存每entity的完整location
projection。dense正文也已由archetype table单一持有。这些是有效的架构进展，但当前默认产品查询仍未成为
chunk-linear热路径。

## P0：默认system query为全局稳定顺序支付树索引和heap merge

`StableQueryOrderIndex`为每个entity同时维护HashMap entry、全局`BTreeMap<order, entity>`和每
archetype一张`BTreeMap<order, StableEntityLocation>`。spawn写两类关联索引；每次archetype move执行旧树
remove、新树insert；swap-remove row修正还要再次树查找。默认`Query::iter()`随后为每个matching
archetype分配一个BTree iterator，再分配`BinaryHeap`做k-way global-order merge：每个entity至少一次
heap pop/push，之后又按archetype ID binary-search plan。

这不是旧的O(World entities) miss扫描，但把成本搬成了持续写放大与稳定帧`O(E log A)`：`E`为命中entity，
`A`为命中archetype。`QueryState::update_cache`即使archetype generation完全不变，也会调用
`refresh_plan_memberships`线性扫描全部A个plan，只为刷新generation与diagnostic entity count；该
membership generation不参与迭代器失效判断。因此Q个稳定query每帧先额外支付`O(Q*A)`，而非O(1)版本检查。

全局stable world order已经被跨move/clone/serde测试写成公共合同，不能由性能修复暗改。Runtime08必须先做
显式契约裁决：simulation/system热路径使用archetype/chunk order和连续column view；只有editor投影、
serialization或明确请求确定顺序的API进入`Stable/OrderedQuery`边界。若最终仍要求所有system全局order，
则必须以产品数据证明其收益大于树写放大和heap merge，不能把当前实现直接标为完成。

## P0：compiled slot之后仍逐entity重建location并做平方TypeId查找

plan虽然已知道table column slot，但每行仍把全部C个component binding写进
`Vec<ComponentStorageLocation>`；tuple内每个data/filter fetch又用`TypeId`线性扫描这C个location。
因此C-component tuple的定位比较接近`O(C^2)`/entity，且没有把table column暴露成连续slice/chunk view。
默认cached iterator还维护两个A-sized控制容器；direct iterator虽跳过heap merge，仍逐row经
`internal_entity_location`恢复stable location，而不是一次绑定chunk columns后线性推进。

可变与组合路径进一步放大临时工作：

| path | 当前call-local临时工作 | 规模问题 |
|---|---|---|
| `iter_mut` | 先复制全部E个`StableEntityLocation` | 每次全量query为O(E)分配/复制 |
| `single_mut` | 先collect全部candidate，找到第二项前也不停止 | 单项判定仍按全部E分配 |
| `for_each_mut` | 先collect E个candidate，随后每entity新建location Vec | 至少E个短命heap buffer |
| read/mut combinations | 先collect E个candidate，每个输出组的K个item各新建location Vec | `C(E,K)*K`个短命buffer；E=1,000、K=2即999,000个 |

这些测试多为源码形状守卫，明确要求candidate Vec、stable iterator和当前projection token；没有allocator、
heap-operation、TypeId comparison或chunk-contiguity门。实现前应把它们改为行为、复杂度和分配计数测试，
避免继续保护成本本身。

## P1：combination count overflow可转成近似不终止执行

`combination_count`在中间`checked_mul`溢出时返回`usize::MAX`。iterator以该值作为唯一remaining终止条件；
当indices到达最后一组时，`advance_indices`不能再推进但也不返回失败，后续会反复产出最后一组，直到
`usize::MAX`次递减结束。中间乘法还可能在最终组合数可表示前过早溢出。这是算法正确性与frame hang风险，
不是可接受的saturating size hint。

Runtime08应让cursor是否成功推进成为唯一终止authority；组合数只负责可选的exact upper bound，溢出时返回
`None`/非exact hint。RED门必须覆盖接近溢出的N/K、末组只出现一次、K=0/1/N/N+1和mutable alias安全；不得
通过减小输入上限或在热循环加入任意超时掩盖。

## 诊断与编译门

当前`last_matched_entity_count`在cache rebuild/refresh时直接赋成candidate count，Added/Changed等运行时
filter拒绝多少entity并未计入，所以现有candidate/matched指标不能证明选择率或瓶颈消失。另有两个同名
`cached_archetype_plans()` inherent method分别位于`query_state/mod.rs`与`cache.rs`，当前源码静态上形成重复
定义；`ecs_query_combinations.rs`源码守卫还期待已不存在的`entities.iter().copied()`形状。应先恢复行为测试
可编译，再增加真实counter，不能用这些shape tests宣称优化完成。

本轮managed Windows命令在D盘coordinator target执行843.4秒，以 **361个编译错误、1,520条warning**
失败；0条focused test执行。错误跨多个foreign dirty模块，最后可见错误包括text rich cache的
`String -> Arc<str>`不匹配。由于不存在可运行current-source binary，allocator benchmark、WPR/xperf、
Tracy和F2产品trace均未运行；RenderDoc也不能用于CPU query结论。

## Unreal主依据与统一结构计划

UE Mass `MassEntityQuery.cpp:138-228`以archetype data version做O(1)稳定检查，仅对新增archetype追加
matching handle与requirement mapping；`258-396`直接遍历matching archetype/chunk，不做跨archetype的全局
spawn-order heap merge。`MassArchetypeData.cpp:830-870`在chunk/subrange边界一次绑定requirement mapping后执行
函数；`MassExecutionContext.h:640-658`把fragment暴露为连续array view。`MassEntityQuery.cpp:573-690`又把同一
chunk job描述交给ParallelFor。这里借鉴的是“versioned query plan + chunk-bound column views +显式并行
边界”，不是复制UE类型、阈值或放弃Zircon既有确定性需求。

| task / owner | 结构目标 | 必须证明的验收 |
|---|---|---|
| PERF-MVP-604 / Runtime08 + Runtime03 | 公共order契约裁决；默认system query改为archetype/chunk cursor，stable/ordered只在显式边界维护 | unchanged generation plan work=O(1)；默认每query BTree iterator/heap allocation=0、heap op=0；ordered API跨move/clone/serde等价 |
| PERF-MVP-605 / Runtime08 + Runtime11 | QueryData/Filter编译typed binding index并在chunk绑定连续column/tick view；mutable cursor在禁止structural mutation的borrow内流式推进；scratch/executor按generation复用 | warm table query每entity location materialization、TypeId scan、HashMap location lookup和allocation=0；sparse/optional/tick、alias和panic合同通过 |
| PERF-MVP-606 / Runtime08 | combination cursor以advance成功终止，safe count仅作size hint；candidate与K-item scratch复用 | overflow不重复末组/不假终止；每输出组allocator call=0；small-N exact size与全部组合顺序/alias等价 |

动态矩阵覆盖entities 1/1k/100k、archetypes 1/8/256/4k、components/query 0/1/4/8、match
0.1%/1%/100%、sparse 0/50/100%、query calls/frame 1/16/1k、ordered/fast、read/mut/filter和
combination N/K。记录plan refresh/compile、index writes、BTree/heap ops、candidate/location bytes、TypeId
comparisons、column spans、allocator calls/bytes、cache miss、CPU cache miss、main/worker time、p50/p95/p99、
CSwitch/ReadyThread与energy。WPR/xperf/Tracy是CPU/调度/功耗authority；RenderDoc只验证改造未增加
draw/dispatch/readback。取得同一硬件同一场景前，不宣称达到UE经验值、功耗接近或算法最优。

本切片继续留在`pending.md`，不进入`review.md`；编译门、规模counter、产品trace和独立复核完成前，
不提交性能里程碑，也不发送企微完成消息。
