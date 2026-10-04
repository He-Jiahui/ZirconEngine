---
related_code:
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_runtime/src/scene/ecs/component
  - zircon_runtime/src/scene/ecs/entity
  - zircon_runtime/src/scene/ecs/storage
  - zircon_runtime/src/scene/world/identity.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/typed_api/component_row.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-archetype-columnar-storage.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-bundle-single-archetype-transaction.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Internal/MassArchetypeData.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassArchetypeData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp
  - dev/bevy/crates/bevy_ecs/src/archetype.rs
  - dev/bevy/crates/bevy_ecs/src/bundle/info.rs
  - dev/bevy/crates/bevy_ecs/src/bundle/insert.rs
tests:
  - current archetype storage kernel and direct World bridges 37/37 files and 18 inline tests statically reviewed
  - related identity storage performance and structure tests 11/11 files and 69 tests statically reviewed
  - direct rustfmt 48/48 passed
  - managed Windows zircon_runtime lib-test compile failed; focused tests and profiles did not run
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime ECS archetype/storage current-source结构性能复审（2026-08-14）

## 范围、快照与有效进展

本轮完整复审`ecs/{archetype,component,entity,storage}/**`及直接World row transition桥接，共
**37/37个生产Rust文件、5,199行、4,676个非空行、18条内联测试**；另复审相关
**11/11个测试文件、2,761行、69条测试**。生产和测试快照指纹分别为
`83652D094A8ADF6704C9794A1626ACA0B48EF29DD1E5CB8DC1C5B35F2757E6F2`和
`BD06D6CA8125642FE9B2C8986598CFFB9EA4BD7A0F8BC079F639FB56FE4947B5`；直接rustfmt
48/48通过。相关bundle/deferred command与query路径已在同日相邻复审中逐文件覆盖。

当前源码已完成重要的单一owner修复：dense table component不再同时存于`ComponentStorage`，而由
`ArchetypeTable`的row-aligned raw columns唯一持有；query可用compiled column slot直接读取，despawn用已知
archetype row做swap-remove并只修复被换入entity的位置。`SparseComponentStorage`也不是entity HashMap，
而是dense entity/value rows加generation-aware sparse index。这些都优于旧的双写、全storage扫描和逐row
component HashMap，必须保留。

但当前“单一owner”只解决静态驻留，没有解决结构迁移：dense value一旦跨archetype，又会退回
`Box<dyn Any> + BTreeMap`逐entity搬运，且sparse-only membership变化也触发这条路径。

## P0：full archetype identity与dense table identity错误耦合

`ArchetypeSignature`同时包含table与sparse-set component；`ArchetypeIndex::id_or_insert`却为每个完整signature
新建独占`ArchetypeRecord { table: ArchetypeTable }`。因此只新增/删除一个sparse component，也会切换到另一
archetype table。`World::insert/remove`和dynamic component presence都明确在sparse value变化后调用
`transition_entity_archetype_row(..., BTreeMap::new())`，把全部dense values搬到列布局相同的新表。

对同一dense schema有S个独立sparse component时，已使用的membership组合都会建立不同archetype和重复dense
column directory；理论上限为`2^S`：

| sparse component kinds | possible full signatures per dense schema |
|---:|---:|
| 8 | 256 |
| 16 | 65,536 |
| 20 | 1,048,576 |

实际数量取决于产品使用的组合，表中不是当前实测archetype数；它说明复杂度上界和为什么不能用“目前场景小”
作为架构验收。组合膨胀同时增加by-signature key、by-component posting、query plan、空table/column directory和
结构写入缓存失效。

Runtime08必须分离full membership identity与dense table schema identity。可采用“多个full archetype指向同一
TableSchema/TableId，sparse-only变化只迁移archetype metadata row”或经语义审查后的UE式chunk sparse presence；
不能让sparse-only变化搬运dense body。query仍须能按full component membership过滤，并保持generation、
stable entity handle、lifecycle和deterministic order合同。

## P0：dense row move把连续列重新拆成每component Box与树节点

`ArchetypeColumn`以登记的layout在连续raw allocation中持有component body，这是正确的热读取布局。但
`ArchetypeColumn::take`通过`TableColumnLayout::take_box`把每个非ZST value重新分配成Box；
`ArchetypeTable::take_row`把C个value插入`BTreeMap`。目标append再按每个column从BTreeMap remove并写回列。
preflight另建`BTreeSet`计算最终component set并做多轮binary search/contains。

因此一次N-entity、C-dense-component迁移至少构造N*C个非ZST Box，并执行每entity的`O(C log C)`tree工作；
还未计入command staging中的新增值Box：

| entities | dense components | row-transfer Box constructions |
|---:|---:|---:|
| 1,000 | 8 | 8,000 |
| 100,000 | 8 | 800,000 |
| 100,000 | 31 | 3,100,000 |

这不是allocator实测次数：ZST不一定分配，allocator也可能复用；恢复编译后必须用allocator/counter验证。但源码
已足以证明连续列的结构迁移不是range-linear，也不能靠把BTreeMap改成小Vec就达到目标。

Runtime08应编译source/target table schema之间的move plan：共享column、added column、removed column和tick
column各自绑定slot与type-erased range move/init/drop回调；命令层PERF-MVP-607提供archetype entity range与平行
payload，storage层一次预留目标chunk/range后按range迁移。UE的range `Memcpy/InitializeStruct/DestroyStruct`
证明了批处理边界，但Rust非`Copy` component不得无条件memcpy；必须用登记layout的move/drop语义或显式
relocatable合同，且panic/rollback不double-drop、不泄漏半初始化row。

## P0：单体列同步扩容制造结构写入尖峰，也不能形成worker chunk

每个archetype的每个dense component是一整块从0开始按1.5倍增长的allocation。row-aligned columns长度相同且
使用同一阈值，所以一个新row跨capacity边界时，全部C个body column和C个tick Vec同时reserve；到100,000
rows源码增长序列发生30次，最近一次从92,170扩到138,255。`realloc`可能原地扩展，也可能移动整列，实际
copy bytes由allocator决定，但同步尖峰边界是确定的。

整列allocation也没有固定chunk/range authority，query只能把“archetype全列+row区间”临时解释为工作块，
结构操作无法只锁定/预留受影响chunk。Runtime08/11应按目标bytes与component layout编译chunk capacity，
chunk内保持SoA连续column与ticks；batch reserve/move、query range和worker partition共享同一chunk identity。
chunk大小、growth与empty-chunk retention必须由同一硬件场景数据决定，不复制UE默认值。

## P1：sparse value与component metadata仍有重复owner和分配放大

`ComponentStorage`为同一dense `ComponentId`维护`storage_types`、`component_types`、`sparse_components`三张
HashMap；前两项已存在于`ComponentRegistry` descriptor/layout authority。`ComponentRegistry`本身又用
`table_column_layouts: HashMap<ComponentId, ...>`保存dense ID可直接寻址的layout。每次sparse访问先HashMap
定位component storage，再进generation-aware sparse row。

每个sparse value仍是独立`Box<dyn Any>`；`entries`连续的只是Box指针与ticks。每个component的
`sparse_rows`还会resize到该component见过的最大InternalEntity slot，低密度、高slot或大量sparse类型时，
索引内存按各component高水位之和增长，而不是实际value数。

Runtime08应让descriptor dense slot直接持有storage type、Rust type/layout与可选sparse store，删除重复
HashMap metadata。sparse store复用type-erased contiguous value/tick column和dense entity rows；索引采用
direct、paged或density-adaptive策略，选择必须由`live values / high-water slots`矩阵证明。目标不是把所有
sparse component强制改成table，而是让低密度收益不被每value Box和每type world-sized index反噬。

## 诊断与测试false-green

当前`ecs_performance_acceptance/columnar.rs`虽遍历1/1k/100k entities，但先执行`spawn_empty_at + insert`构建
全部row，计时只包围后续query；结构写入、sparse-only变化和row move完全不计时。query所谓p95只有4个样本，
断言仅为不等于`Duration::MAX`。100k archetype fixture也只计20次单posting-list lookup，build time只在计时
窗口外，retained bytes只断言大于0。

现有源码形状测试还明确要求`take_row`逐column收集`ArchetypeTakenRow`、`ComponentStorage`三HashMap/entry
lookup以及sparse source tokens；这些测试保护owner边界时也锁住了成本实现。应保留“dense单一owner、sparse
generation安全、swap-remove row修复、atomic preflight、tick/lifecycle/order”等行为门，删除对BTreeMap、
HashMap数量和函数正文字符串的依赖。

现有diagnostics只有component-index probe、signature membership check和row append；没有row take/move、
same-table sparse transition、Box/tree、column realloc/growth bytes、chunk/range或sparse density指标。并且这些
计数通过`AtomicU64::fetch_add`常驻执行，100k row append本身产生100k次atomic RMW。Runtime08/11应使用
disabled-fast-path或executor-local counter聚合，不让性能诊断成为结构写入authority的一部分。

本轮managed Windows focused `zircon_runtime` lib-test仍沿用D盘coordinator结果：843.4秒后因其它foreign
dirty模块累计361个编译错误、1,520条warning失败，0条本切片测试执行。没有current-source binary，故
allocator benchmark、WPR/xperf、Tracy和F2产品trace未运行；RenderDoc只能在运行恢复后验证draw/dispatch/
readback无回归，不能证明ECS storage CPU瓶颈消失。

## Unreal主依据、补充依据与统一计划

UE Mass `MassArchetypeData.h:360-460`以固定memory chunk、fragment config offset和NumEntitiesPerChunk持有
archetype body；`MassArchetypeData.cpp:1672-1785`按entity collection/range预留目标span，再成段迁移并倒序
批量移除source ranges。`1911-1971`对共享fragment执行range copy、对新增/删除fragment执行range init/drop，
而不是逐entity转成opaque heap object。Zircon只借鉴range move plan，Rust move语义必须更严格。

同一UE当前源的`2028-2110`让sparse element在现有archetype entity range内批量add/remove，通过chunk presence
和独立SparseElementsStorage发布，不触发full archetype move。Bevy补充证据更直接：`archetype.rs:7-14`
明确多个仅sparse component不同的archetype共享同一Table；`bundle/info.rs:383-395`区分
`NewArchetypeSameTable`与`NewArchetypeNewTable`，`bundle/insert.rs:620-680`在没有新增table component时复用
当前TableId。这里以UE的chunk/range与sparse batch为主，Bevy只补足Rust ECS table/archetype身份分离实现证据。

| task / owner | 结构目标 | 必须证明的验收 |
|---|---|---|
| PERF-MVP-610 / Runtime08 | 分离full membership ArchetypeId与dense TableSchema/TableId；sparse-only变化复用table/chunk，不搬dense body | same-dense sparse组合的table count=1；sparse add/remove dense value move/Box/realloc=0；query full-membership、tick/lifecycle/order、clone/serde等价 |
| PERF-MVP-611 / Runtime08 + Runtime11 | chunked SoA table；编译source/target schema range move/init/drop plan，与PERF-MVP-607 typed command range共用 | N*C row-transfer Box=0、tree op/entity=0；range reserve/move随affected rows+columns增长；non-Copy/drop/panic/rollback/Miri正确，query chunk worker可并行 |
| PERF-MVP-612 / Runtime08 | descriptor dense slot统一type/layout/storage authority；sparse contiguous body/ticks + dense entities + direct/paged/adaptive index | component metadata hash probe=0；non-ZST sparse Box/value=0；index bytes与live/high-water密度受预算；stale generation、swap-remove、dynamic plugin type等价 |
| PERF-MVP-613 / Runtime08 + Runtime11 | 以行为/复杂度/allocator门替换容器源码形状测试；diagnostics disabled快路或local aggregation | disabled atomic RMW=0；记录same-table transition、range、Box/tree/realloc/chunk/sparse density；真实20+样本p50/p95/p99而非4样本非MAX断言 |

动态矩阵覆盖entities 1/1k/100k、dense C 0/1/8/31、sparse S 0/1/8/20、membership组合
1/8/256/实际上限、archetype/chunk 1/8/256/4k、add/remove/replace/despawn/bundle/deferred、component size
0/4/64/4KiB和non-Copy/drop/panic。记录table/archetype/chunk数、range长度、row move、Box/tree/allocator、
realloc/growth bytes、sparse high-water/live ratio、cache miss、CPU/lock、p50/p95/p99、RSS、CSwitch/ReadyThread与
energy。只有同一硬件同一场景前后数据才可声称瓶颈消失或接近UE经验值。

本切片继续留在`pending.md`，不进入`review.md`；动态门和独立复核完成前，不提交性能里程碑，也不发送企微
完成消息。
