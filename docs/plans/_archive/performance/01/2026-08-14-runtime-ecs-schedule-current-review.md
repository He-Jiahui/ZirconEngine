---
related_code:
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/scene/ecs/commands
  - zircon_runtime/src/scene/ecs/system
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/scene_system_registry.rs
  - zircon_runtime/src/scene/world/schedule.rs
  - zircon_runtime/src/scene/module/world_driver.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/03/failure-2026-07-22-production-schedule-remains-serial.md
  - docs/plans/zircon_runtime/runtime/03/failure-2026-07-17-schedule-executor-frame-allocations.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassProcessorDependencySolver.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassExecutor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/TickTaskManager.cpp
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/mod.rs
  - dev/bevy/crates/bevy_ecs/src/schedule/executor/multi_threaded.rs
tests:
  - current control slice 52/52 files and 38 inline tests statically reviewed
  - related external schedule tests 13/13 files and 81 tests statically reviewed
  - direct rustfmt 65/65 passed
  - managed Windows focused zircon_runtime lib-test compile failed after 843.4 s; 0 tests ran
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime ECS schedule current-source结构性能复审（2026-08-14）

## 范围、快照与旧结论修正

本轮完整复审任务池、schedule plan/runner/conflict graph、native system与全部SystemParam、deferred
command、帧诊断、World schedule和WorldDriver共 **52/52个Rust文件、12,273行、10,994个非空行、
38条内联测试**；另复审`scene/tests`对应 **13/13个文件、3,853行、81条测试**。两组快照指纹分别为
`4E9CE72D9B22781871A2C79810911C76A19944F4D4B683D439BADD9395D60521`和
`CCF2D2C8141BF5BF5EFD92249696F4100EF27AA121620AD334DEC61FDE83DFED`；直接rustfmt 65/65通过。
52个控制面文件中38个为其它Session的修改/新增，本轮不覆盖生产源码。

7月“产品schedule完全串行”的结论已不再准确：当前runner会把标记worker-safe且互不冲突的native
system从World取出，经`JobScheduler::join`并行运行，再按稳定key合并worker command buffer；Internal、
Runtime、main-thread native与ApplyDeferred仍是显式barrier，panic时丢弃未发布命令并恢复system。这是有
价值的source repair，但还没有解决真实ECS数据并行与稳定帧控制面成本。

## P0：worker lane目前不承载ECS数据

`SceneSystem::supports_worker_dispatch`同时要求WorkerSafe、worldless、无ordering constraint且无保守
World访问。更关键的是，sealed `WorldlessSystemParam`当前只有`()`、`LocalParam`、`CommandsParam`及其
tuple实现；`Query`、`Res/ResMut`、event/message、removed-component等数据参数均不在此集合。普通
`FunctionSceneSystem`也始终不支持worldless执行。

因此当前并行lane只能运行局部状态/纯回调/deferred-command producer，真正读取或写入ECS component、
resource、event/message的系统仍回到主线程。现有overlap测试使用独立原子量/命令生产者，不能证明F2场景
系统已获得data-parallel执行。Runtime03的旧failure保持open；其“source repaired”必须进一步细化为
“worldless子集接入，data-bearing ECS worker view仍缺失”。

## P0：stable frame仍重建字符串身份并搬移registry

- runner每个worker system、每帧调用`DeferredSystemKey::compiled(..., id)`；`&str -> Arc<str>`产生新的
  shared-string allocation。主线程路径又在全部stage/step中线性查找ID并重建同一key。
- `Schedule::take_native_system`线性查找后为taken guard执行`to_string()`；registry用`Vec::remove`取出
  system。恢复时先线性删除taken String，再binary-search并`Vec::insert`。一批N个system按顺序取出和
  插回可产生O(N^2)指针搬移/字符串比较，以及每帧N个String和N个Arc身份分配。
- `ecs_scheduled_native_systems.rs`中的源码形状测试明确要求`Vec<String>`、`to_string`、线性while、
  `remove(index)`等token；这些测试保护了实现缺陷，而非“mutation generation内身份稳定”的行为合同。

## P1：conflict build和逐帧batch均退化为平方规模

`ScheduleConflictGraph::from_node_vec`先计算同stage非barrier节点的`N(N-1)/2`上界，并据此
`Vec::with_capacity`，随后才判断实际access conflict。按x64下两个`String`、一个`Vec`和stage/padding
推算，`ScheduleConflictEdge`约80字节；这是静态布局估算，不是allocator/working-set实测：

| same-stage systems | candidate pairs | runner HashMap lookups/frame | requested edge capacity estimate |
|---:|---:|---:|---:|
| 16 | 120 | 240 | 9.4 KiB |
| 256 | 32,640 | 65,280 | 2.49 MiB |
| 1,000 | 499,500 | 999,000 | 38.1 MiB |
| 10,000 | 49,995,000 | 99,990,000 | 3.72 GiB |

零冲突的10k系统也会请求3.72 GiB edge capacity并执行约5千万pair access比较。稳定帧runner又忽略graph
已有的`conservative_parallel_batches`，逐system与当前batch所有成员调用`systems_conflict`；每次先做
两次String HashMap lookup，再做adjacency binary search。零冲突10k系统因此每帧约1亿次HashMap lookup。
连续greedy batch还不能表达dependency-ready executor状态，某些冲突排列会无谓缩小并行宽度。

## P1：barrier控制面仍逐批分配、排序和线性找lane

每个worker batch新建`system_ids`、taken `systems`、`timings`和可选command-buffer引用Vec；stage还新建
`worker_batch`。现有temporary-control指标只按capacity估算部分容器字节，不是全局allocator调用数。
worker buffer即使已按compiled plan顺序到达，merge仍按包含String的key做O(W log W)排序；world command
queue对每个worker arena用`.position`查找，apply后reclaim再次线性查找，W个lane可形成O(W^2) key比较。
inline command arena已把<=192字节payload聚合进64 KiB block，并有4 MiB预算与fallback计数，因此旧的
“每条command必定Box”结论已过时；真正待处理的是plan identity、lane索引、merge排序/查找与warm容量复用。

任务系统新增scheduled/queue/execution/wait总量与耗时，且disabled路径用tracked门避免`Instant::now`；
但报告仍只有aggregate sample+total ms，没有pool kind、queue peak、steal/yield、p50/p95/p99分布。native
schedule已有callback p95、ready delay、worker utilization与临时容量代理，应复用同一诊断链补齐，不建第二套
profiler。

## Unreal主依据与补充依据

Unreal Mass的`MassProcessorDependencySolver.cpp:200-290`不是先枚举所有processor pair，而是为fragment、
chunk/shared fragment、subsystem和sparse element建立read/write user表；新reader只依赖当前writer，新writer
只连接当前相关readers/writers，并用archetype overlap缩小真实冲突。`960-1060`缓存匹配archetype，并以
archetype data version作为dependency result失效边界。这个模型直接支持“按access倒排生成实际依赖”，
比Zircon的全pair预留更接近本问题。

`MassExecutor.cpp:165-238`把processor dispatch到AnyThread task，completion后由done task统一flush deferred
commands；它支持“数据执行与确定性barrier分离”，不意味着可复制UE的具体类型或阈值。通用tick侧，
`TickTaskManager.cpp:280-376`以稳定`FTickFunction*`作为task identity，batch `Reset`保留数组容量；
`1098-1130`在frame边界清空状态但保留batched allocation。Bevy executor的dense system index、预编译
metadata和可复用FixedBitSet仅作补充；Bevy自身也有pairwise conflict初始化，不能当10k规模结论的依据。

## 统一结构优化计划

| owner | 结构改造 | 必须保留/证明的合同 |
|---|---|---|
| Runtime03 + Runtime08 | `SceneScheduleStagePlan`冻结dense `SystemSlotId + generation`、direct slot、预建`DeferredSystemKey`、affinity/access/barrier/dependency；registry mutation只在generation改变时重编译 | stable frame String/hash/key allocation=0、registry remove/insert/move=0；重复/删除/重注册与generation失效正确 |
| Runtime08 | 为Query/Res/event/message建立经`SystemParamAccess`验证的scoped storage/resource view或等价sound分区；不能让workers共享`&mut World` | 非冲突data-bearing systems真实overlap；冲突/非Send/exclusive仍进正确lane；Miri/aliasing设计审查与状态等价门 |
| Runtime03 | 参照UE Mass按component/resource/event/message的read/write倒排表生成实际依赖；world writer为全局barrier，必要时用archetype overlap收窄 | build复杂度接近O(access declarations + emitted conflicts)，零冲突不预留N(N-1)/2；显式order与cycle诊断不回退 |
| Runtime11 | executor持有可复用ready/running/completed/deferred bitset/queue、timing与控制缓冲；按dense slot dispatch | warm stable frame控制容器growth=0；panic/cancel/wait不泄露system或部分authority；1/2/8/64 worker宽度均可观测 |
| Runtime08 commands | worker command lane改为compiled numeric slot，已按plan order的batch不重复String排序；arena按slot直接归还 | merge顺序、spawn generation、duplicate-key、panic discard与exactly-once apply行为不变；lane lookup/sort/string compare=0 |

先写行为与复杂度RED门，再实施结构迁移；不得以全串行、全exclusive、共享World mutex或未证明的unsafe
绕开问题。现有source-shape测试应改为slot generation、state/order、panic recovery和allocation counter行为门。

## 动态验收矩阵

运行systems 1/16/256/1k/10k、access 0/1/8/64、conflict density 0/1/10/100%、worker width
1/2/8/64，以及query/resource/event/command混合负载；分开记录plan build与stable frame：pair/access probes、
graph bytes、String/Arc/allocator calls、registry moves、lane sort/search、control-buffer growth、World lock
wait/hold/acquires、batch/overlap/utilization、queue peak、p50/p95/p99、CPU、CSwitch/ReadyThread和energy。

验收下限是：data-bearing非冲突系统在产品F2 trace中overlap>0；stable generation graph rebuild、ID/hash/key
allocation、registry搬移和warm控制缓冲growth均为0；零冲突build内存随N+access declarations增长；串/并行
state、event、deferred order与panic结果一致。WPR/xperf/Tracy为CPU/调度/功耗authority；RenderDoc只证明
schedule改造没有增加draw/dispatch/present或隐藏readback。没有同一硬件同一场景数据前，不宣称已接近UE
经验耗时、功耗或“算法最优”。

当前managed `zircon_app` build先在其它foreign dirty `zircon_runtime`源码上以6个编译错误失败；随后
focused `zircon_runtime` lib-test在D盘managed target运行843.4秒，又以361个编译错误和1,520条warning
失败，0条schedule test执行。allocator benchmark与F2产品trace均未执行。因此本切片只形成结构计划，继续留在
`pending.md`，不进入`review.md`，不提交性能里程碑或发送企微完成消息。
