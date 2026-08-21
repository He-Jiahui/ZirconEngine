---
related_code:
  - Cargo.toml
  - .github/workflows/profile-feature-contract.yml
  - tools/check-runtime-profile-features.ps1
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/macros.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/export.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/mod.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/session/extract_cache.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/scene/ecs/query/query_state/cache.rs
  - zircon_runtime/src/scene/ecs/query/query_state/mod.rs
  - zircon_runtime/src/scene/ecs/query/query_state/stats.rs
  - zircon_runtime/src/scene/ecs/change_detection/stats.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/world/performance_diagnostics.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/worker_pool/diagnostics.rs
  - zircon_runtime/src/core/framework/script/hot_path_metrics.rs
  - zircon_runtime/src/plugin/native_plugin_loader/benchmark_harness.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/atlas_renderer/mod.rs
  - zircon_runtime/src/text/font/fallback_cache.rs
  - zircon_plugins/animation/runtime/src/evaluation/pose_pool.rs
  - zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs
  - zircon_plugins/animation/runtime/tests/animation_pose_buffer_allocation_contract.rs
  - zircon_plugins/sound/runtime/src/tests/kira_graph_sync.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_plugins/ai/runtime/src/perception/scan.rs
  - zircon_plugins/physics/runtime/src/backend/builtin/step.rs
  - zircon_plugins/physics/runtime/src/manager/world_sync.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_performance_acceptance.rs
  - zircon_runtime/src/scene/tests/ecs_change_detection.rs
  - zircon_runtime/src/dynamic_api/session/tests/frame_diagnostics.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/hotspot_inventory/ecs_extract_counters.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/performance_hotspots/owner_budget/mirror_docs/performance_guard.rs
  - tests/acceptance/runtime-performance-filters-current-result.md
plan_sources:
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/runtime/07/2026-07-09-runtime-performance-hotpath-output-records.md
  - docs/plans/zircon_runtime/runtime/07/2026-07-11-runtime07-durable-performance-evidence-and-resource-gate.md
  - docs/zircon_runtime/performance/hotspot_inventory.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08e-network-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
  - docs/plans/optimize/zircon_tooling/07-performance-benchmark-profile-capture-symbol-crash-evidence-baseline-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/optimize/zircon_tooling/25-memory-allocation-domain-budget-oom-pressure-fragmentation-pooling-cache-residency-observability-review.md
  - docs/plans/optimize/zircon_tooling/29-rust-module-boundary-root-entry-large-file-declaration-behavior-folder-topology-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Containers/Array.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/MemStack.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/ParallelFor.h
  - dev/bevy/crates/bevy_ecs/src/storage/table/mod.rs
  - dev/godot/core/templates/local_vector.h
  - dev/godot/core/templates/rid_owner.h
  - dev/Fyrox/fyrox-core/src/pool/mod.rs
  - dev/Fyrox/fyrox-core/src/sparse.rs
  - dev/Fyrox/fyrox-core/src/task.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphObjectPool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/NativePassCompiler.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 32 · Hot Path Catalog、算法复杂度、数据搬运、Batching、Cache Locality 与性能治理审查

## 1. 结论

Zircon已经有真实而且值得保留的性能工程基础。Runtime07在2026-07-12完成了同一精确Vampire命令的两次正式样本：`30.894424483213513 FPS / 32.3683 ms`和`33.98320549984198 FPS / 29.4263 ms`，均为116 mesh draws，均值相对偏差`9.521868%`；direct runtime-frame trace、QueryState 128实体/8次重复查询缓存基线、change detection计数和unchanged extract复用也已闭环。当前源码又有frame/stage span、counter hotspot export、worker batch临时控制buffer字节计数、asset worker queue/copy/wall-time诊断、animation pose pool与sound graph allocation测试。这些不是“临时完全没有性能意识”的代码，后续不得删除或退化。

但Runtime07解决的是一组已知ECS/extract/render提交候选的取证与局部优化，不是全仓Hot Path工程治理。当前production-like Rust快照有11,993个文件、1,340,621行、46,909,799 bytes；保守词法候选中有12,790个`.clone()`、4,452个`.collect()`、578个sort、6,438个format、9,021个`vec!`/Vec构造、1,513个map构造、631个filesystem I/O、1,100个锁调用和231个blocking候选。它们不能直接判为缺陷，却说明仅靠46个固定Runtime07源文件和字符串锚点不可能覆盖引擎规模的成本演化。

当前动态结构审计已经证明这种脆弱性：`performance_hotpath_boundary`声明46个源文件却只读到40个，声明91个测试owner却实际枚举90个，219个固定锚点有35个缺失，owner-budget重新出现11个热点/3个migration debt，最终返回7条risk。六个缺失源文件都是已经从Runtime移出的旧`animation/scene_hook/*`；asset worker诊断其实仍在，只是下沉到`worker_pool/diagnostics.rs`；extract和schedule span也只是签名/宏名变化。也就是说，同一红结果混合了真实animation metric handoff缺口、正常模块迁移和过时字符串三类事实。历史镜像仍写`risks=[]`与`classified-and-clear`，说明“结构锚通过”没有形成持续、typed、source-bound的性能资格。

更关键的是，多个领域报告已经各自确认真实steady-state风险：`World::node_records()`每次分配owned `Vec<SceneNode>`、投影全部实体并按ID排序，而Navigation agent、AI perception、Physics builtin step/world sync等路径在tick内消费它；Navigation还解析动态JSON并重建多组map/set，Detour query会重建查询数据；Editor diagnostics presentation会clone store并重建投影；WebSocket读路径复制frame并投递多个队列。具体修复仍归Runtime08A/08D/08E/08F、Editor25及Tooling24/25，本篇不重复计为新领域缺陷。本篇新增的是缺失的全局控制面：每个frame/tick/audio callback/render submit/editor presentation/network packet热路径都必须有稳定身份、workload scale、复杂度/访问/分配/字节搬运/等待/I/O成本合同、增量失效策略、batch/scratch/pool规则、观测点和BuildSet绑定资格。

本报告登记 **0项P0、48项P1、12项P2和40个验收门**。Tooling07继续拥有benchmark runner、capture、统计、硬件与长期baseline；Tooling24/25继续拥有并发和内存安全；各Runtime/Editor/Plugin报告继续拥有具体算法。Tooling32只拥有跨仓`HotPathCatalog -> WorkloadScale -> CostContract -> Mutation/DirtySet -> Batch/Scratch/Cache -> Observation -> Complexity/Benchmark Evidence -> QualificationReceipt`控制面。没有current evidence时，不得宣称“性能优于当前Unreal”；也不得用关键词计数直接生成机械优化。

## 2. 审查边界、口径与限制

### 2.1 当前物理与语义证据

| Evidence | 本轮结果 | 可支持结论 |
|---|---:|---|
| E1 production-like Rust source | 11,993文件、1,340,621行、46,909,799 bytes | 全仓性能治理不能只维护少量手写source list |
| E2 allocation/data-movement lexical candidates | clone 12,790、to_owned 548、to_vec 443、collect 4,452、sort 578、format 6,438、Vec构造9,021、map构造1,513 | 只用于候选发现；不含调用频率、payload大小、优化器结果或路径冷热，不能直接变finding |
| E3 blocking/shared-state lexical candidates | filesystem I/O 631、lock 1,100、blocking 231 | 只提示需要owner分类；初始化、测试、错误路径和每帧路径必须语义区分 |
| E4 positive instrumentation | `profile_frame!` 19处/13文件、`profile_scope!` 368处/131文件、`profile_counter!` 375处/60文件；reserve/with_capacity 1,190处/640文件 | 已有局部可观测和容量意识，目标是登记与资格，不是重写宏或禁止标准容器 |
| E5 Runtime07 current audit | 46 expected/40 present source；91 expected/90 enumerated test owner；219 anchors/35 missing；11 large-file hotspots、3 debt；7 risks | 结构审计当前为红；红项混合owner迁移、锚漂移与真实能力丢失，不能按一个bool解释 |
| E6 Runtime07 accepted dynamic baseline | 双次FPS 30.8944/33.9832，116 draws，偏差9.521868%；trace与ECS/extract基线已接受 | 证明历史快照与固定场景；不证明当前dirty源码、其他产品、规模或新增插件路径仍达标 |
| E7 whole-world projection | `World::node_records()`为每次调用分配/全实体投影/排序；非tests路径词法命中30处/22文件，其中包含结构断言与非frame路径 | API本身有确定O(N + N log N)和owned-copy成本；调用是否是热路径必须逐owner判定 |
| E8 semantic hot-path reads | Navigation agent tick、AI perception、Physics builtin step/world sync确认在frame/tick链使用whole-world投影；具体证据已在领域报告 | 本篇建立跨域gate，具体数据结构与算法由既有领域finding修复 |
| E9 positive allocation contracts | animation pose pool steady-state、sound graph sync、native live host、UI atlas/image cache/text fallback已有allocation/copy count或零分配测试 | 可迁移为HotPathDescriptor关联的typed metric/test，不应被新的中央框架替换掉 |
| E10 reference spot reads | Unreal Array/MemStack/ParallelFor、Bevy ECS Table、Godot LocalVector/RID owner、Fyrox Pool/Sparse/TaskPool、Unity RenderGraph pool/compiler | 提取容量、临时内存、稠密存储、generation、free-list、batch和对象复用约束；不据此宣称整机性能 |

production-like口径排除了`dev/docs/vendor/target/node_modules`以及明确tests/fixtures/generated路径。词法扫描会把注释、宏、冷路径和小对象包含进来，也可能漏掉封装后的隐式分配、驱动内部成本和编译器消除，因此只用于建立candidate queue，不进入severity统计。

### 2.2 Runtime07当前性判读

当前审计器红结果必须拆解，而不是简单把7条risk都解释为运行时回退：

1. 六个missing source都是旧`zircon_runtime/src/animation/scene_hook/{diagnostics,events,node_pose,pending,scan,tick}.rs`。Animation执行已迁入`zircon_plugins/animation/runtime/src/evaluation/pipeline/*`，旧路径缺失是hard cutover后的inventory漂移；但旧`animation.scene.*`扫描/输出/事件/transition frame counters没有在新owner中等价恢复，后者才是真实可观测缺口。
2. 11个asset-worker anchor仍存在于`worker_pool/diagnostics.rs`，审计器只把父`worker_pool.rs`读入anchor source，因此误报能力缺失。正确修复是从module/source owner graph解析文件，不是把实现搬回大父文件。
3. `current_extract()`仍调用`record_frame_extract_stats`，但参数从`(&runtime, &extract, status)`演进成`(&runtime, diagnostics_summary, status)`；scheduler仍有stage profile，只是从动态宏/string变成`profile_scope!`加静态stage name。精确源码片段不是稳定能力ID。
4. `EXPECTED_TEST_FILE_COUNT=91`与当前tuple实际90项自相矛盾，且`missing_test_files=[]`。expected count必须由manifest派生，不能维护第二个手写整数。
5. 当前anchor集合已增长到query 34、test 30，而历史mirror仍显示32/29；required string presence没有可靠覆盖数值currentness。
6. large-file owner gate当前为`migration-debt-present`，11个hotspot/3个debt；这属于Tooling29结构owner，不应与算法热路径风险压成同一个`risks`数组。

Runtime07的动态验收仍是有效历史证据。本篇要求把它迁入versioned catalog并标注source/BuildSet/workload/currentness，不删除记录、不伪造当前复跑，也不因结构审计漂移否认2026-07-12已经发生的正式样本。

### 2.3 本轮没有做的事

- 没有修改Rust、Cargo、tests、workflow、profile工具、Runtime07计划或结构审计器。
- 没有在当前dirty并发工作树重跑ZrVM real backend、完整Runtime、Editor或GPU capture。
- 没有用clone/collect/sort计数判断某一实现必慢，也没有提出全仓替换`Vec`/`HashMap`。
- 没有把参考引擎某个container API等同于其完整帧性能，更没有形成“已优于Unreal”的结论。
- 本报告是review与重构设计；实施时必须在稳定BuildSet上重取source graph、workload和动态证据。

## 3. 必须保留的工程基础

### 3.1 Runtime07已经证明“先测量、再优化”可以执行

Runtime07明确规定没有named diagnostic、capture source和owner verdict就不得进入M2，并留下精确命令、ZrVM binding identity、双样本偏差和trace artifact测试。新HotPath Catalog应引用这些record，而不是重新发明一套互不兼容的完成状态。

### 3.2 ECS QueryState与extract已经有incremental seed

128实体、8次重复查询得到8 hit、1 initial miss、1 initial rebuild；unchanged extract从rebuild `[1,1]`收紧到`[1,0]`，并有cache hit/miss与scene mutation invalidation断言。目标是把key、dependency generation、steady-state zero-work和scale预算变成通用合同，不是移除当前cache。

### 3.3 Scheduler正在记录batch临时控制内存

`SceneScheduleRunner::flush_worker_batch`会聚合worker-safe system，按conflict graph分batch，并记录callback、ready delay、batch elapsed、temporary control buffer count/bytes。这个粒度已经比单纯“开线程”更工程化；后续需要稳定HotPathId、grain/parallel threshold与1/100/10k workload资格。

### 3.4 Asset、Animation、Sound、UI与Native Host有局部成本断言

Asset worker记录queue peak、completion bytes、payload clone bytes、queue/cancel/drop wall time和thread budget；animation pose pool与sound graph有CountingAllocator测试；native live host记录allocation count；UI atlas/image cache和text fallback暴露allocation/cache统计。这些局部契约应通过catalog被发现和比较，而不是被词法扫描误判为“没有性能实现”。

### 3.5 Profiling export已经能排序counter candidate

`CounterHotspotReport`与`counter_hotspots.json`可按total/avg/p95/max/latest/count/frame count输出counter候选。它仍只是evidence routing，不能在缺少workload、unit、direction和budget时自动宣判优化优先级；但它是统一Observation pipeline的直接迁移基础。

## 4. 已确认热点与领域Owner路由

| 现有事实 | canonical具体owner | Tooling32只增加的横向约束 |
|---|---|---|
| `World::node_records()`每次owned projection并排序 | Runtime05 Scene/ECS | API登记cost class；frame caller必须声明N、visits、bytes与替代typed/incremental路径 |
| Physics builtin step/world sync消费whole-world projection并重建/clone状态 | Runtime08A | physics step workload和steady/change path进入catalog；修复仍归Physics |
| Navigation agent tick扫描node records、解析JSON、建多组map/set；query重建数据 | Runtime08D P1-6/P1-13/P1-18 | agent/navmesh规模轴、JSON decode/sort/rebuild counters和budget成为required evidence |
| AI perception收集receiver/source并做全体投影/配对 | Runtime08F | receiver/source/pair cardinality、broad-phase reject和dirty update预算 |
| WebSocket frame复制后进入多个共享队列 | Runtime08E + Tooling24 | copy multiplicity、queue bytes、admission/drop/backpressure与consumer lag cost |
| Animation scene owner迁移后旧frame counters缺失 | Plugins04 + Runtime08C | HotPath/Metric ID必须跨hard cutover handoff并保留alias/retirement receipt |
| Editor diagnostic presentation clone/rebuild | Editor25 + Editor01 | presentation cadence、snapshot delta、projection bytes、UI thread budget |
| allocation domain、pool/cache residency | Tooling25 | Tooling32只关联每个HotPath的allocation/bytes budget，不重定义OOM/pressure owner |
| lock/atomic/blocking/backpressure | Tooling24 | Tooling32只要求每个HotPath关联wait/RMW/block metrics，不重定义并发语义 |
| benchmark/capture/statistics/hardware/baseline | Tooling07 | Tooling32提供被测HotPath/Workload/CostContract身份，不接管runner和artifact store |

## 5. P1：Catalog、Owner、Currentness 与审计语义

### TOOL-HOTPATH-P1-001 · 没有全仓HotPath Catalog

frame、fixed tick、audio callback、render submit、editor presentation、network receive、asset worker和tool loop各自用注释/宏/测试表达冷热，没有stable `HotPathId`、owner、cadence、execution domain或required evidence。建立versioned catalog；任何新增或迁移的hot path必须通过owner review进入catalog或明确声明cold/maintenance path及证据。

### TOOL-HOTPATH-P1-002 · Runtime07固定46个源文件不能覆盖当前引擎表面

固定tuple只覆盖ECS/extract/asset worker/旧animation hook和少量render diagnostics，不会自动发现AI、Navigation、Physics、Editor、Network或新plugin路径。source inventory应由Cargo-resolved SourceSet、module owner graph和catalog registration生成；手写列表只允许作为历史migration input。

### TOOL-HOTPATH-P1-003 · 审计用精确源码字符串充当能力身份

宏名、参数形态或静态stage name变化会让功能仍存在却报missing。把anchor改为typed registration、metric descriptor、test receipt或AST/semantic query；源码snippet只可作为诊断定位，不得决定capability truth。

### TOOL-HOTPATH-P1-004 · expected count与实际inventory存在第二权威

当前`EXPECTED_TEST_FILE_COUNT=91`，tuple实际90项且全部存在。expected count必须从唯一manifest计算并校验manifest自身unique/path role；禁止手写count、list和mirror三份真相。

### TOOL-HOTPATH-P1-005 · 单一`risks`数组混合不同失败域

missing moved source、missing metric、large-file debt和stale docs都进入同一string list，consumer无法判断应由Plugins04、Tooling29还是Runtime07处理。输出versioned typed `FindingSet`，至少包含code、owner、subject ID、severity、evidence kind、currentness和blocking policy。

### TOOL-HOTPATH-P1-006 · 历史mirror可在当前审计变红后继续显示green

`hotspot_inventory.md`仍写46/91、`risks=[]`、large-file clear，而当前direct audit是40 source、35 missing anchors和7 risks。镜像文档必须绑定audit schema、input digest和run receipt；current状态只能由最新成功required run投影，历史文本必须标为historical snapshot。

### TOOL-HOTPATH-P1-007 · completed计划没有持续资格/失效状态

Runtime07的2026-07-12完成是有效里程碑，但source owner迁移后没有自动把相关qualification标为stale/recheck-required。`QualificationReceipt`必须携带BuildSetId、catalog revision、workload revision、metric set和expiry/invalidation reason；completed不等于永久current。

### TOOL-HOTPATH-P1-008 · metric owner迁移没有强制handoff

旧`animation.scene.*`计数随Runtime scene hook移除而消失，新plugin pipeline没有等价frame metrics；审计只看到missing path。hard cutover transaction必须声明moved/renamed/retired HotPathId与MetricId、new owner、compat alias期限和行为证据，缺handoff时迁移gate失败。

### TOOL-HOTPATH-P1-009 · metric path、unit与语义没有中央typed catalog

counter字符串可被profiling export聚合，但同名/改名、cumulative与delta、zero sample、frame scope、unit和direction主要靠局部代码。定义`MetricDescriptor`和stable typed ID；counter writer、exporter、benchmark comparator和Editor projection消费同一schema。

### TOOL-HOTPATH-P1-010 · 词法candidate没有owner verdict生命周期

当前可枚举clone/collect/sort/lock/I/O候选，却没有状态机区分cold path、bounded small、measured acceptable、domain finding、waived或fixed。建立candidate triage ledger；机器只能发现，领域owner必须给出workload与证据，waiver有到期和source digest。

## 6. P1：Workload 与可执行Cost Contract

### TOOL-HOTPATH-P1-011 · 没有统一WorkloadScale身份

性能测试各自写128 entities、116 draws或1M jobs，不能比较同一逻辑在不同规模下的增长。定义typed axes，例如entities、archetypes、components、systems、agents、sources、receivers、draws、passes、widgets、glyphs、packets、payload bytes、assets和workers，并给每个scenario固定组合与上限。

### TOOL-HOTPATH-P1-012 · 没有声明算法复杂度预算

代码review能看出`node_records()`为O(N + N log N)，Navigation邻接构建存在O(P²)候选，但仓库没有可被gate消费的expected complexity。`CostContract`应记录dominant axes、expected bound、amortized/average/worst-case语义和允许退化条件；scale test用斜率/ratio而非单点时间验证。

### TOOL-HOTPATH-P1-013 · 实体/组件访问次数没有预算

“扫描了多少实体”只在少数ECS/animation计数出现，其他system无法说明一帧重复访问世界几次。每个world-facing hot path至少记录candidate visits、matched items、full-scan count、random lookup count和projection count；稳定场景必须能定位重复消费者。

### TOOL-HOTPATH-P1-014 · 比较、排序、hash与树操作成本不可见

578个sort候选里既有正确的deterministic cold output，也有可能每帧重复排序；BTree/Hash选择同样缺规模证据。为hot path记录sort count/elements、comparison estimate、hash probes/rehash、tree lookup和deterministic-order requirement，禁止只凭容器偏好重构。

### TOOL-HOTPATH-P1-015 · owned clone只计次数不计payload bytes

小型Arc/handle clone与深clone `SceneNode`/render extract/diagnostic snapshot成本完全不同。建立`CopyClass`与bytes moved/copy multiplicity指标；clone lint必须结合type/layout/known payload或runtime byte counter，不能把所有`.clone()`等价处理。

### TOOL-HOTPATH-P1-016 · 临时allocation没有按hot path归因

全局allocator测试只覆盖少数路径，普通frame/tick没有allocation count/bytes/peak/retained分类。将allocation domain、scratch growth和allocator sampling关联到HotPathId；steady-state budget可为0或明确上限，cold/change path另设预算。

### TOOL-HOTPATH-P1-017 · decode、format和字符串规范化成本没有cadence合同

动态JSON、format、path/name normalization在authoring/import冷路径合理，在每agent/perception/frame路径则会放大。每个decode/encode/format hot candidate必须声明输入bytes、calls、schema缓存、失败比例和是否可预编译成typed runtime projection。

### TOOL-HOTPATH-P1-018 · 锁等待与atomic RMW没有进入相同成本模型

Tooling24已发现共享锁、global atomic和backpressure问题，但性能top list无法把wait duration、contended acquisitions、atomic RMW、cache-line sharing与CPU span关联。Observation需关联execution domain/thread/core和HotPathId，并区分diagnostics on/off。

### TOOL-HOTPATH-P1-019 · filesystem/network/blocking工作没有frame deadline归属

631个I/O和231个blocking词法候选不能说明是否进入frame/audio/editor UI线程。catalog必须声明`may_block`、allowed execution domain、deadline、timeout、cache/mmap/async策略和worst-case fallback；违规调用以typed finding阻断，而非靠命名约定。

### TOOL-HOTPATH-P1-020 · CPU到GPU数据搬运与submission成本没有统一字节合同

Runtime07观察到draw数并将render提交移交领域owner，但全仓没有per-frame upload bytes、buffer copies、descriptor writes、draw/dispatch/pass和readback wait的统一contract。Graphics owner提供typed metrics，Tooling32只要求它们与workload、frame和BuildSet可关联。

### TOOL-HOTPATH-P1-021 · 预算只有绝对时间，没有frame占比和critical-path关系

单个span 1ms是否危险取决于30/60/120Hz目标、并发重叠和critical path。BudgetRef应包含target cadence、CPU main/worker/GPU/audio/editor thread slice、absolute与frame-share上限、p50/p95/p99及miss policy。

### TOOL-HOTPATH-P1-022 · cold、warm、steady、dirty与recovery路径混在同一结论

extract cache、shader compile、asset import、first glyph与device recovery的合理成本不同。每个workload必须明确phase：cold start、warm-up、steady unchanged、bounded dirty update、burst、shutdown/recovery；不同phase不能共享一个平均数或相互掩盖。

## 7. P1：Incremental Projection、Cache Locality 与数据生命周期

### TOOL-HOTPATH-P1-023 · `World::node_records()`把兼容投影视为通用查询API

该API分配容量为entity count的Vec，遍历stable IDs，构造owned SceneNode并排序。保留它用于serialization/inspection cold path，但frame systems应使用typed ECS query、borrowed view或domain-maintained projection；新增tick调用必须由structure/performance gate拒绝。

### TOOL-HOTPATH-P1-024 · 没有全仓steady-state zero-work合同

unchanged extract已证明第二帧可不重建，但Navigation、AI、Physics、Editor presentation和多类derived view没有统一要求“无变化时访问/分配/上传/发布应为零或有界”。每个derived projection声明steady-state counters及阈值，并用连续多帧测试锁定。

### TOOL-HOTPATH-P1-025 · DirtySet没有公共generation/provenance模型

系统各自用revision、epoch、boolean dirty、hash或全量比较决定重建，难以解释为何某缓存失效。定义只共享schema而不共享全局锁的`MutationStamp/DependencyGeneration/DirtyReason`，每个cache记录input generations和invalidation provenance。

### TOOL-HOTPATH-P1-026 · cache key完整性主要靠局部人工维护

Runtime extract key包含change tick/query revision/camera/viewport是良好种子，但新依赖字段加入时没有通用contract检查。每个cache登记dependency set、key encoder、hit/miss/rebuild和mutation matrix；mutation test必须覆盖每一dependency以及无关变化不失效。

### TOOL-HOTPATH-P1-027 · derived projection缺少single-writer/multi-reader发布合同

Navigation/AI/Physics/Editor各自从world重扫，而不是消费同代typed projection，造成重复工作和代次不一致。领域owner维护generation-bound snapshot/delta；consumer只能借用同代只读view，不能在读取时重建第二份authority。

### TOOL-HOTPATH-P1-028 · 容器布局没有按访问模式登记

当前代码同时使用Vec、HashMap、BTreeMap、Arc和owned DTO，但没有说明dense iteration、stable order、random lookup、mutation churn或SIMD要求。为hot state记录layout rationale、key density、iteration order和measured locality；结构变更必须带behavior与scale evidence。

### TOOL-HOTPATH-P1-029 · stable identity与dense storage没有明确分层

对外stable EntityId/handle不应迫使每帧遍历owned scene records。参考Bevy Table、Godot/Fyrox generation owner，建立stable handle -> generation validation -> dense domain slot/column的分层；具体ECS/physics/nav实现归各owner。

### TOOL-HOTPATH-P1-030 · repeated scratch container每次重新分配/rehash

schedule batch、animation evaluation、navigation/AI collect和render preparation存在许多临时Vec/map/set；有些已复用，有些每次新建。catalog关联`ScratchClass`、capacity high-water、clear/reuse策略和retention budget；禁止无界保留，也禁止steady path反复grow。

### TOOL-HOTPATH-P1-031 · scratch/pool lease没有generation与return证明

局部pose pool/object cache能复用，但全仓没有统一lease receipt、owner thread、reset contract、poison/discard和shutdown drain规则。Tooling25拥有内存安全，本篇要求任何宣称pool优化的hot path同时证明reuse hit、capacity、return和steady allocation。

### TOOL-HOTPATH-P1-032 · cache/pool指标没有区分容量、有效驻留和命中价值

只记录entry count或allocation zero不足以证明缓存有效，可能保留大量冷数据或高昂validation。至少记录resident bytes、logical items、hits/misses/rebuilds/evictions、lookup/validation cost和age；与memory pressure策略连接。

### TOOL-HOTPATH-P1-033 · data movement跨stage重复但没有lineage

world -> extract -> submission -> backend、runtime -> editor snapshot -> presentation、network frame -> multiple queue会复制或投影同一逻辑数据。为大payload赋`DataProductId`和generation，记录producer、consumer、borrow/share/copy/serialize边及bytes，capture才能识别重复搬运。

### TOOL-HOTPATH-P1-034 · deterministic order与排序成本没有解耦

许多路径为了稳定输出在读取时全量sort，这是正确性需求，但不必每个consumer重复执行。由owner在mutation/commit时维护stable index或生成ordered snapshot；读取热路径借用已排序generation。任何改为unordered都必须保持determinism/replay tests。

## 8. P1：Batching、Parallelism 与执行计划

### TOOL-HOTPATH-P1-035 · batch只是局部容器，没有统一BatchPlan合同

Scheduler、renderer、asset worker、event/network和Editor各自聚合工作，但batch ID、admission、max items/bytes、flush reason和partial failure不统一。定义轻量`BatchPlanDescriptor`，由领域owner实现；性能证据能比较calls saved与latency增加。

### TOOL-HOTPATH-P1-036 · parallel threshold与grain size没有规模证据

发现少量Rayon-like并行和scheduler recursive join，但多数路径没有1/100/10k输入下串行/并行切换点。每个parallel hot path声明minimum grain、worker budget、oversubscription domain和fallback；benchmark验证speedup、tail和temporary bytes。

### TOOL-HOTPATH-P1-037 · worker batch每flush仍materialize多个临时Vec

`flush_worker_batch`构造system IDs、taken systems、timings和command buffer refs，并已记录临时control bytes，这是可观测基础。下一步用scale evidence决定复用scratch、small-vector或直接iterator，而不是先验改写；gate锁定temporary bytes、batch count和callback ratio。

### TOOL-HOTPATH-P1-038 · fan-out/fan-in成本没有端到端预算

一个producer向多个subscriber/queue复制、一个frame等待多个jobs merge时，局部queue/worker指标不能说明总工作。Workload记录fan-out/fan-in轴，Observation关联copies、wakeups、wait、merge probes和slowest consumer。

### TOOL-HOTPATH-P1-039 · per-item dynamic dispatch/JSON/reflection没有预编译计划

plugin、scene dynamic component、editor reflection在扩展边界合理，但每entity/tick反复字符串查找与decode不可接受。建立compile/install/change-time `ExecutionPlan`或typed projection，runtime只执行stable IDs/slots；保留unknown/future错误与plugin generation验证。

### TOOL-HOTPATH-P1-040 · diagnostics自身没有采样/成本预算

大量profile counter与span是必要基础，但Instant、atomic、string path、store publication也可能改变微任务与audio/frame行为。每个instrumentation family声明off/on/sampled成本、allocation和atomic RMW；高频路径使用registered ID与thread-local/sharded aggregation，语义结果保持一致。

### TOOL-HOTPATH-P1-041 · CPU/GPU/IO pipeline没有统一overlap与stall解释

单独统计CPU span、GPU marker或worker elapsed会把等待误判成计算。Capture应输出queue submit、fence/readback、I/O wait、worker readiness和main-thread dependency edges，构建critical-path view；具体graphics/backend tracing仍归Runtime09与Tooling07。

## 9. P1：Evidence、Regression 与产品资格

### TOOL-HOTPATH-P1-042 · static conformance与dynamic cost qualification没有分层

当前Runtime07 audit把source/anchor/large-file检查包装成一个boundary，文档容易把`risks=[]`理解为性能通过。required pipeline必须输出独立的CatalogConformance、InstrumentationConformance、ComplexityEvidence、BenchmarkEvidence和ProductQualification，任何一层不得替代另一层。

### TOOL-HOTPATH-P1-043 · span/counter没有绑定stable HotPathId与source owner

字符串path便于展示，却不能可靠处理rename、plugin移出或同名多实例。Registration返回typed handle，export同时携带stable ID、display path、owner package、source revision、world/session/frame和metric schema version。

### TOOL-HOTPATH-P1-044 · scale regression主要是单点阈值

单一128实体或116 draws能防特定回退，不能识别O(N)退化到O(N log N)/O(N²)。为关键axis运行至少small/medium/large三点，比较normalized operations和time slope；高噪声wall time只作辅证，确定性operation counters先阻断结构退化。

### TOOL-HOTPATH-P1-045 · representative product workload没有catalog closure

Vampire是重要场景，但Editor、WOC client/server、headless、UI/text、asset burst、network burst、physics/nav/AI大场景和不同graphics profile没有统一required矩阵。Tooling07拥有runner，本篇要求每个P1 hot path至少映射一个micro、one subsystem和one product workload或记录有期限豁免。

### TOOL-HOTPATH-P1-046 · performance gate没有进入统一required build/test计划

profile feature workflow主要验证build组合，许多性能测试仍ignored/manual。按resource class拆PR static/operation-count gate、nightly CPU scale、managed Windows/GPU capture和release product qualification；typed skip/block/inconclusive不能记passed。

### TOOL-HOTPATH-P1-047 · comparison不能证明“优于当前Unreal”

Zircon与参考引擎缺同语义场景、资产、画质、分辨率、硬件、driver、warm state、statistical protocol和功能完整度映射。建立FeatureParityReceipt与ComparisonProtocol；在所有必要语义对齐前，只能报告Zircon自身趋势和局部机制差异。

### TOOL-HOTPATH-P1-048 · evidence失效、promotion与回滚没有原子receipt

source、Cargo lock、feature、asset、workload或metric schema变化后，旧baseline仍可被文档引用。comparison/promotion receipt绑定BuildSet、input artifact、catalog/workload/contract revision、raw result hashes和acceptor；新结果失败不覆盖最后accepted baseline，rollback可追溯。

## 10. P2：可解释性与长期优化能力

| ID | 改进项 | 退出条件 |
|---|---|---|
| TOOL-HOTPATH-P2-001 | HotPath Explorer | 按product/frame/thread/owner展示catalog、budget、latest qualification和stale reason，不从自由文本猜状态 |
| TOOL-HOTPATH-P2-002 | Query/Execution Plan Explain | ECS/nav/AI/render/editor projection可导出只读plan：inputs、indices、full scans、sorts、batches和expected bound |
| TOOL-HOTPATH-P2-003 | Data Movement Graph | 以DataProductId显示producer/consumer、generation、copy/share/serialize边和bytes，支持定位重复全帧复制 |
| TOOL-HOTPATH-P2-004 | Dirty/Invalidation Explain | 对一次rebuild给出触发dependency、old/new generation、affected pages/entities和无关变化过滤 |
| TOOL-HOTPATH-P2-005 | Automatic scale candidate suggestion | 静态扫描提出whole scan/sort/decode/clone候选并给confidence；必须等待owner verdict，不自动改代码 |
| TOOL-HOTPATH-P2-006 | Source-diff cost review | PR显示新增HotPath注册、budget变化、new full-scan/copy/lock/I/O边及qualification影响 |
| TOOL-HOTPATH-P2-007 | Hardware/profile dashboard | 同一workload按CPU/GPU/RAM/driver/power profile分层，不把异构机器raw time直接排序 |
| TOOL-HOTPATH-P2-008 | Expiring performance waiver | waiver含owner、reason、scope、budget delta、deadline、source digest和replacement milestone |
| TOOL-HOTPATH-P2-009 | Cache locality optional counters | 在受控平台采集cache miss/branch/IPC等硬件计数；不可用时typed skip，不以0填充 |
| TOOL-HOTPATH-P2-010 | Code size/instruction-cache budget | 高频泛型/inline/monomorphization路径可关联symbol/code-size证据，防止只看runtime allocation |
| TOOL-HOTPATH-P2-011 | Energy/thermal qualification | mobile/laptop profile记录power mode、temperature、throttling和energy/frame，结果与desktop分域 |
| TOOL-HOTPATH-P2-012 | Optimization decision archive | 保存candidate、measurement、owner verdict、rejected alternatives和rollback trigger，避免重复无证据重构 |

## 11. 参考实现对照

| 参考 | 本轮读到的机制 | Zircon应吸收的约束 | 不应机械复制 |
|---|---|---|---|
| Unreal `TArray` | allocator计算grow/shrink/reserve slack，提供reserve、uninitialized add与swap removal等明确成本操作 | 容量策略、是否收缩、ordered/unordered removal必须在hot state中显式；bytes/relocation可观测 | C++ allocator API、所有unsafe/uninitialized技巧或默认增长常量 |
| Unreal `FMemStack/FMemMark` | 栈式临时内存与scope mark/rewind | frame/task scratch有scope、owner、high-water和回收证明 | 把所有Rust allocation改成单一全局arena |
| Unreal `ParallelFor` | flags/context/min-batch等控制并行执行 | 并行入口必须有grain、worker budget、single-thread fallback和context identity | 认为parallel必然更快或忽略oversubscription |
| Bevy ECS Table | component列稠密存储、capacity/reserve、row move/swap-remove与change tick相邻维护 | stable identity与dense iteration解耦，结构变化维护索引，查询读取不投影owned scene DTO | 整套ECS API/unsafe布局或假设其任何版本都更快 |
| Godot LocalVector/RID owner | local storage/reserve/unordered removal；RID owner/free list/validator区分identity与slot reuse | small/local capacity需workload证明；generation/validator保护slot reuse | 用固定inline capacity替代所有Vec，或复制Godot宏风格 |
| Fyrox Pool/SparseBuffer | generation handle、free stack、with_capacity、slot reuse；TaskPool区分异步工作 | pool声明capacity、generation、reuse与task owner；稳定handle不等于dense payload | 以通用pool隐藏domain生命周期或忽略backpressure |
| Unity RenderGraphObjectPool/NativePassCompiler | render graph临时对象、列表/数组在compile/execution边界集中复用和清理 | render/compiler scratch由明确owner集中管理，compile-time plan与frame execution分离 | 托管GC/object pool实现、RenderGraph内部布局或固定capacity |

参考源码证明成熟引擎会把容器增长、临时内存、dense storage、generation、batch、compile plan和对象复用做成显式机制；它们没有证明任意一个容器选择就能让Zircon整体超过Unreal。最终比较仍必须经过Tooling07定义的同语义、同硬件、同BuildSet统计协议。

## 12. 目标架构

### 12.1 控制面与运行面分离

```text
Cargo-resolved SourceSet + Product Composition
                    |
                    v
       HotPathCatalog / MetricCatalog
       - HotPathId, owner, cadence, domain
       - workload axes, phase, budget refs
       - source/metric/test registration
                    |
          +---------+----------+
          |                    |
          v                    v
 WorkloadRegistry         CostContract
 scenario/input/build     visits/comparisons/alloc/bytes
 scale axes/phases        wait/io/cpu/gpu/complexity
          |                    |
          +---------+----------+
                    v
 MutationStamp -> DirtySet -> Domain Projection/ExecutionPlan
                    |
       BatchPlan + ScratchLease + CacheState
                    |
                    v
 Registered Observation -> Capture/Raw Artifact
                    |
                    v
 ComplexityEvidence + BenchmarkEvidence
                    |
                    v
 ProductQualificationReceipt / Promotion / Stale / Rollback
```

Catalog是build/tooling控制面，不得让每次runtime lookup经过中央字符串map。产品compose时生成compact descriptor table和numeric handles；hot loop只写thread-local/sharded counter或已注册span，export阶段再恢复display path与owner元数据。

### 12.2 `HotPathDescriptor`

最低字段：

- `HotPathId { namespace, owner, name, version }`，rename/move有alias与retirement receipt。
- `ExecutionDomain`：main/render/worker/audio/network/editor UI/tool process/GPU queue等。
- `Cadence`：frame、fixed tick、audio quantum、packet、asset event、presentation、batch或maintenance。
- `PhaseSet`：cold、warm-up、steady、dirty、burst、recovery/shutdown。
- `WorkloadAxisRef[]`与required scenario；明确small/medium/large和product scale。
- `CostContractRef`：operation、allocation、bytes、wait/I/O、CPU/GPU与complexity预算。
- `MutationDependency[]`、DirtySet、cache/scratch/batch descriptor及steady-state合同。
- `MetricId[]`、test/capture owner、waiver、last accepted qualification与stale reason。

### 12.3 `CostObservation`

Observation必须绑定BuildSetId、product/session/world/frame或operation、HotPathId、workload scale和phase。计数优先使用确定性整数：visits、matched、sort elements、decode bytes、alloc count/bytes、copy bytes、queue items/bytes、draw/dispatch/pass、wait nanoseconds等；wall time和hardware counters是同一observation的可选维度，不以缺失值填0。

### 12.4 Incremental与数据搬运合同

Domain owner从world mutation/asset revision/input generation生成DirtySet，维护typed projection或execution plan。consumer借用同代数据；跨线程/进程需要immutable snapshot或bounded delta。若必须copy/serialize，DataProductId、generation、bytes和consumer count进入observation。兼容inspection/serialization API可以保留owned stable snapshot，但不得被frame system无证据复用。

### 12.5 资格层级

1. Catalog conformance：ID、owner、source、metric、test和budget引用完整。
2. Static cost conformance：禁止的frame I/O、whole-world owned projection、unbounded decode/copy等规则无新违规。
3. Deterministic operation scale：访问/比较/分配/bytes随scale满足合同。
4. Micro/subsystem benchmark：统计协议、noise和environment由Tooling07执行。
5. Product capture：Vampire/WOC/Editor/headless等真实composition在目标硬件达预算。
6. Comparison qualification：只有feature/workload parity完整时才能比较Unreal等外部引擎。

## 13. 实施里程碑

### M0 · 冻结当前HotPath与Runtime07漂移

- 保存当前Runtime07 direct audit typed snapshot：40/46 source、90/91 test、35 missing anchors、11 hotspots/3 debt、7 risks。
- 将35项逐条分类为moved、renamed、retired-with-replacement、true missing或unrelated structure debt。
- 冻结production source candidate inventory与领域owner路由，不生成自动fix。
- 为Runtime07历史accepted evidence加BuildSet/workload/currentness标注，不改写历史结果。

### M1 · Catalog、Metric Schema 与审计器hard cutover

- 定义HotPathId、MetricId、WorkloadAxis、CostContract和typed FindingSet schema。
- 从Cargo SourceSet/module registration生成inventory，移除手写expected counts。
- asset worker submodule、new animation plugin owner、scheduler/extract新API成为迁移样本。
- mirror docs从latest required receipt生成；旧字符串审计降为迁移兼容后删除。

### M2 · Operation/Allocation/Data Movement基线

- 接入visits、sort/decode、allocation、copy bytes、lock wait、I/O和CPU/GPU typed metrics。
- 迁移现有QueryState、extract、asset worker、pose/sound/native/UI局部计数。
- 建立diagnostics off/on/sampled成本测试和registered numeric handle fast path。
- Tooling25/24提供allocation domain与concurrency metrics，不复制owner。

### M3 · Incremental Projection、DirtySet 与Cache Contract

- 先以`node_records()` frame consumers为样本建立typed projection/dirty generation规范。
- 各Runtime08 owner按其报告移除per-frame full world projection/JSON decode/rebuild。
- cache dependency mutation matrix和steady-state zero-work测试成为required。
- DataProduct lineage连接world/extract/submission/editor/network大payload。

### M4 · Batch、Scratch、Pool 与Parallel Grain

- 迁移scheduler worker batch、render submission、event/network fanout和asset worker。
- 登记scratch lease、capacity high-water、return/poison/shutdown和memory pressure行为。
- 为1/100/10k workload确定serial/parallel threshold和oversubscription策略。
- 验证calls saved、latency、temporary bytes与tail，禁止只看throughput。

### M5 · Scale/Complexity Required Gates

- PR lane运行catalog/static/operation-count与小规模slope；nightly运行CPU scale。
- managed Windows/GPU lane运行render/editor产品capture；resource blocker为typed inconclusive。
- 每个P1 HotPath至少有small/medium/large和steady/dirty两个phase。
- Complexity regression能在wall-time噪声前阻断O(N²)或重复full scan。

### M6 · Product Workload Closure

- Vampire、WOC client/server/headless、Editor workbench、UI/text、physics/nav/AI和asset burst组成required matrix。
- 每个产品BuildSet发布QualificationReceipt并链接raw evidence。
- budget miss有owner、candidate、rollback/waiver，不修改baseline掩盖失败。
- currentness因source/catalog/workload/driver变化自动失效。

### M7 · External Comparison 与持续治理

- 定义与Unreal/Fyrox/Bevy/Godot/Unity Graphics可执行的FeatureParityReceipt和同语义场景。
- 固定硬件、OS/driver、quality、resolution、warm state和统计协议。
- 只发布完整比较矩阵；缺功能或证据项显式unknown/incomparable。
- HotPath Explorer、decision archive和自动candidate建议进入日常review。

## 14. 验收门

1. `HotPathCatalog` schema有stable ID、owner、cadence、execution domain、phase、workload、budget、metric和test引用。
2. Catalog输入来自Cargo-resolved product SourceSet；没有手写文件count第二权威。
3. 当前所有catalog source/test路径存在、唯一且FileRole正确。
4. Runtime07 35个missing anchors全部有typed分类与owner，不剩自由文本unknown。
5. Asset worker moved submodule不再误报metric missing。
6. Scheduler/extract等API rename不再依赖精确source snippet判定capability。
7. Animation旧metric每一项有new owner replacement或显式retirement receipt。
8. large-file structure debt作为Tooling29 typed finding输出，不混入performance capability bool。
9. `EXPECTED_*_COUNT`由manifest派生；list/count/mirror不能漂移。
10. mirror current state绑定latest required run digest；历史snapshot明确日期和BuildSet。
11. 每个P1 hot path至少映射一个WorkloadScale与small/medium/large值。
12. 每个CostContract声明dominant axes和expected asymptotic/amortized bound。
13. scale gate能通过fixture证明检测O(N)到O(N²)的受控回退。
14. world-facing hot paths记录candidate/matched/full-scan/projection counts。
15. hot sort记录calls/elements/comparison或等价确定性operation count。
16. 大payload copy记录DataProductId、generation、bytes和consumer multiplicity。
17. steady-state allocation count/bytes有明确预算；missing observation不作为0。
18. decode/format/normalize hot path记录calls/input bytes并有compile/change-time替代计划。
19. lock wait、contended acquisitions与atomic RMW可关联HotPathId和execution domain。
20. main/render/audio/editor UI hot path无未声明filesystem/network/blocking调用。
21. graphics hot paths输出upload/copy/descriptor/draw/dispatch/pass/readback typed metrics。
22. budget同时定义target cadence、critical domain、absolute与frame-share、p95/p99政策。
23. cold/warm/steady/dirty/burst/recovery结果分开，不合并成一个平均值。
24. `node_records()`新增frame/tick consumer的static gate会失败。
25. 现有frame consumers按Runtime05/08 owner迁移到typed borrowed/incremental projection。
26. unchanged steady tests证明projection/rebuild/upload/publish为0或在显式上限内。
27. 每个cache登记完整dependency generations和invalidation reason。
28. mutation matrix证明每个依赖会失效、无关变化不会失效。
29. stable identity与dense storage分层保留generation/stale-handle验证。
30. scratch/pool报告capacity high-water、reuse、return、resident bytes和pressure行为。
31. BatchPlan报告items/bytes、flush reason、calls saved、partial failure和latency。
32. parallel plan在1/100/10k规模验证grain、speedup、tail、workers和temporary bytes。
33. diagnostics off/on/sampled分别有overhead、allocation和atomic预算。
34. CPU/GPU/worker/I/O observation能生成critical dependency edges而非只列孤立span。
35. Catalog、Static、Complexity、Benchmark、Product五层receipt分别发布且状态不可互相替代。
36. PR required lane运行catalog/static/operation gate；typed skip/inconclusive不记passed。
37. nightly CPU与managed Windows/GPU lane绑定BuildSet、hardware、driver、workload和raw artifact hash。
38. baseline promotion原子、可回滚；失败运行不覆盖last accepted receipt。
39. source/lock/feature/asset/workload/metric schema变化会使受影响qualification自动stale。
40. 任何“优于Unreal”结论必须有FeatureParityReceipt、同硬件/画质/场景/统计协议和完整raw evidence。

## 15. Owner边界

| Owner | 本篇要求 | 本篇不接管 |
|---|---|---|
| Runtime core/ECS（O08/O16） | HotPath注册、world access/dirty generation/query operation metrics | Scene/ECS具体storage/query/scheduler实现 |
| Runtime graphics（O09） | upload/copy/draw/dispatch/pass/stall metrics与workload映射 | RHI/render graph/renderer算法和visual correctness |
| Runtime subsystem plugins（O08） | Physics/Nav/AI/Animation/Audio各自typed projection、execution plan和scale gate | 各领域算法、asset、debug/authoring语义 |
| Runtime Interface/diagnostics（O03/O11） | stable descriptor/wire/unit/schema和低成本registration/export | product workload、runtime state或tool artifact store |
| Editor（O10/O13） | UI thread/presentation/authoring workload和projection cost | Runtime业务authority与benchmark runner |
| Tooling07（O01/O11/O14） | runner、capture、statistics、hardware、artifact、baseline promotion | HotPath/metric/domain cost语义 |
| Tooling24/25（O07） | lock/backpressure和allocation/OOM/pressure合同 | 本篇只引用到HotPath cost，不复制并发/内存policy |
| Tooling29/30 | source/module/large-file finding与refactor transaction | 不把文件大小等同运行成本 |
| Product App/Hub | composition、scenario、target cadence和release qualification | 不复制domain metrics或自行声明baseline passed |

本报告映射全局owner `O01 O07 O08 O09 O10 O11 O14 O16`。Catalog可以统一schema与资格，但不能成为运行时全局锁、全知world registry或每次counter写入的字符串服务。

## 16. 验证与Currentness

本轮运行了当前`audit_runtime_structure.py --json`并单独调用`performance_hotpath_boundary_audit`；结果一致为40/46 source、90实际test owner、35 missing anchors、11 large-file hotspots、3 migration debt和7 risks。语义spot-read确认asset worker为路径误报、scheduler/extract为snippet漂移、animation为owner迁移加真实metric handoff缺口。没有修改审计器去制造green。

本轮还对production-like Rust做保守词法inventory，并逐读`node_records()`、scheduler batch、extract、asset worker、animation pipeline、AI/Nav/Physics代表路径和既有领域报告。词法count只冻结candidate universe，report finding来自源码语义、动态审计输出或既有领域owner证据。

当前branch为`main`，source revision为`ae2be3d865a937b9ed368bf965592045346c64e3`，worktree包含其他Session正在进行的Runtime/Editor/Tooling改动。77个frontmatter输入路径均存在且唯一；按canonical path ordinal排序，以`path + LF + normalized UTF-8 content + LF`编码的SHA-256为`6364601ea43c9b2474253979add8c10e022c5141d02cf1f3b1e48dc84479d4e7`，输入正文共34,266个normalized LF、2,597,537 content bytes。实施前必须重取SourceSet、Runtime07 typed audit、catalog、workload和BuildSet。已知Editor/Hub/WOC/plugin构建阻断及dirty外部ZrVM不在本轮重复执行。

## 17. Review交接

首个实施切片不是优化`node_records()`，而是M0/M1：保存当前红审计、把35项分型、建立HotPath/Metric/Workload/CostContract最小schema，并让asset-worker submodule与animation hard cutover成为迁移测试。只有catalog能正确解释“能力仍在”“能力已迁移”“能力确实丢失”之后，才进入领域算法修复。

随后按风险和复用收益排序：Runtime05的world projection API与Runtime08D/08F/08A的frame consumers；animation metric handoff；scheduler scratch/batch scale；Editor25 presentation delta；Network fanout/copy/backpressure；graphics CPU-GPU data movement。每个具体改动回到其领域报告，Tooling32只验收identity、cost、scale和qualification闭环。

禁止的捷径：把所有clone/collect/sort改掉、全仓换arena、提高large-file阈值、把红risk从required改warning、把实现搬回父文件满足字符串、用旧FPS冒充当前BuildSet、用单点microbenchmark宣称产品更快，或在缺FeatureParityReceipt时宣称优于Unreal。
