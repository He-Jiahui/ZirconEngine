---
related_code:
  - zircon_runtime/src/scene/ecs/bundle.rs
  - zircon_runtime/src/scene/ecs/bundle_transaction_diagnostics.rs
  - zircon_runtime/src/scene/ecs/commands
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/scene/world/deferred_structural_segment.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_entry.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction
  - zircon_runtime/src/scene/ecs/archetype/signature.rs
  - zircon_runtime/src/scene/components/scene/node.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-bundle-single-archetype-transaction.md
  - docs/plans/zircon_runtime/runtime/08/failure-2026-07-22-ecs-deferred-command-dense-buffer.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassCommandBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassCommandBuffer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassCommands.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Public/MassArchetypeGroupCommands.h
  - dev/UnrealEngine/Engine/Source/Runtime/MassEntity/Private/MassEntityManager.cpp
tests:
  - current deferred structural production slice 22/22 files and 7 inline tests statically reviewed
  - related command and bundle tests 11/11 files and 75 tests statically reviewed
  - direct rustfmt 33/33 passed
  - managed Windows zircon_runtime lib-test compile failed; focused tests and profiles did not run
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime ECS deferred structural commands current-source结构性能复审（2026-08-14）

## 范围、快照与当前有效修复

本轮完整复审deferred command payload、producer/worker buffer、queue/apply、spawn token、结构命令分段和
bundle transaction主链共 **22/22个生产Rust文件、4,954行、4,492个非空行、7条内联测试**；另复审
相关 **11/11个测试文件、3,162行、75条测试**。生产和测试快照指纹分别为
`30054A77F4E88979DB43026E0D2B25EFDD551C5D2E5886C85ECEFA873D6E8F4D`和
`7F4ACED0F1290EB332CAC3F364621C616072D2A3D25DBDBDC8B7271CC179EFAE`；直接rustfmt
33/33通过。另重读直接依赖的`NodeRecord`与`ArchetypeSignature`算法；22个生产文件均为其它Session的
修改或新增，本轮没有覆盖其源码。

当前源码已经修复旧报告的两个核心误判：不再为每条小命令单独Box；小于等于192字节、对齐不超过64的
payload进入64 KiB packed block，producer-local arena上限4 MiB，apply后队列entry与block可复用。其次，
worker buffer按compiled order确定性合并，同一target的连续insert/remove/spawn/despawn命令会合成一个最终
row transaction，整批target先preflight后才publish，insert后又remove的组件不会暴露中间生命周期事件。
这些是有效修复，但仅解决了“单target内合并”，没有把结构操作提升为跨target的archetype batch。

## P0：target分段先形成确定的平方成本

`DeferredStructuralBatch::segment_mut`对每条结构命令调用`segments.iter().position(...)`。当N条命令各自
命中唯一target时，target比较次数精确为`N(N-1)/2`，尚未进入bundle staging、archetype move或observer
发布就已支付平方工作：

| unique targets | target comparisons |
|---:|---:|
| 1 | 0 |
| 1,000 | 499,500 |
| 100,000 | 4,999,950,000 |

给`segments`旁加一个HashMap只能降低这一个探测点，仍保留后续逐target事务和publish，因此不能作为结构
验收。Runtime08应先把command表示提升为typed operation batch：按operation、component set/payload schema
积累target与平行payload，再按source/target archetype编译entity collection/range。输入稳定顺序应作为batch
内sequence或显式barrier合同保存，不以全局逐实体事务维持。

## P0：每个target仍建立宽固定事务并逐实体发布

每个segment都建立`BundleInsertionTransaction`，随后把`DeferredBundleTransactionArtifact`在借用World的
transaction与detached artifact之间反复拆装。单个artifact固定携带8个preflight component槽、8个
`Box<dyn PendingBundleComponentValue>`值槽、23个default值Box槽、31个unregistered type槽、31个deferred
removal槽；实际只有一个组件或空spawn也承担同一控制结构。显式值和default值又分别逐项Box。

默认spawn同时保留完整`NodeRecord`，`stage_default_node_record_components`又clone `name`及camera、mesh、
sprite、physics、animation和light等可选owned component后逐项装箱。`final_archetype_signature`对每个增删组件
调用immutable `with_component_added/removed`；该方法每次clone table/sparse两组Vec，再binary-search并
insert/remove，故宽bundle在每个entity重复复制逐步增长的签名，而非一次归一化最终signature。

`DeferredStructuralBatch::finish`虽先对全部segment完成preflight和materialize，publish仍逐segment重新附着
transaction。commit按entity推进world generation、bundle diagnostics和lifecycle dispatch，没有按
archetype range执行一次row move或批量发布。当前设计因此更接近`O(targets * transaction width)`份控制
工件与逐实体publication，而非`O(command schemas + archetype ranges + payload bytes)`。静态数组实际字节、
Box/allocator数量和generation发布成本必须由恢复编译后的layout/counter测试测量，本报告不把布局推算冒充
内存实测。

## P0：queue有payload上限，但没有工作量与驻留上限

inline arena的4 MiB只约束单producer active inline payload；queue entry Vec、oversize/overalign/overflow
fallback Box、等待时长和整批apply时长均没有count/total-byte/age/deadline admission。`apply`在单个World-owned
barrier内drain全部已排队工作，命令风暴仍可形成主线程长尾。

`with_capacity(nonzero)`会预分配至少一个64 KiB block，即使只为一条命令预热；peak后`clear`保留全部block
capacity，单buffer可长期保留到4 MiB。worker merge先按含`Arc<str>`的key做`O(W log W)`排序，再为每个新
worker arena用`.position`线性查找，unique W形成`W(W-1)/2`比较，reclaim再做一次线性查找。compiled
schedule已经拥有稳定顺序，热路径不应重新用字符串identity排序和寻址。

Runtime03/08/11应共同冻结numeric compiled lane/slot，worker arena按slot直接取得、合并和归还；command
storage采用小首page/渐进增长、可测retention/shrink规则以及count+bytes+age/deadline硬预算。拒绝、延期到下一
barrier或拆批语义必须显式，不能用共享World mutex、无界fallback或静默丢命令解决背压。

## P1：spawn token在apply前后重复经过有序树与clone

queue先扫描全部command，把spawn token插入`BTreeSet`，再生成`BTreeMap` resolution；World安装时
`clone_from`复制整张map。publish结果随后又收集并投影为report的`BTreeMap`。S个spawn在真正结构提交之外
重复支付多轮`O(S log S)`树插入、key比较和clone。

Runtime08应让token由queue-local dense ordinal/range承载，reservation和publish使用同一线性resolution
buffer；只有公共report确实要求稳定map时才在冷边界投影一次。必须保留duplicate/stale token、apply失败、
panic discard、嵌套enqueue进入下一窗口和确定性entity ID合同。

## 测试与诊断缺口

现有packed arena测试可证明100,000条小命令enqueue不走fallback，但不执行apply也不计时。结构batch只覆盖
小target数；worker行为覆盖1/8/64 lane；bundle width覆盖1,000次同步spawn，100,000 fixture被ignored。
detached entity batch另有1/1k/100k fixture，但不是当前deferred structural apply主链。当前没有以下门：

- deferred targets 1/1k/100k、commands/target 1/8/64和payload 0/64/192/193字节；
- target/lane probe、command-type batch、archetype range、transaction artifact/Box/allocator字节；
- queue count/bytes/age、peak/retained arena、fallback、generation/lifecycle publish和World锁持有时间；
- apply/main-thread p50/p95/p99、RSS、CPU cache miss、CSwitch/ReadyThread与energy。

`bundle_transactions.rs`还有源码形状守卫从root文件切取`pub(crate) fn finish`并期待commit正文；当前实现已迁到
`deferred_bundle_commit.rs`，该测试保护的源码形状已失效。应改为行为、原子性、复杂度和allocator counter
门，不得为满足旧字符串切片把实现搬回大文件。

本轮managed Windows focused `zircon_runtime` lib-test沿用同一D盘coordinator target，在843.4秒后因其它
foreign dirty模块累计361个编译错误、1,520条warning失败，0条相关测试执行。没有current-source可运行
binary，因此WPR/xperf、Tracy、allocator benchmark和F2产品trace未运行；本切片是CPU结构路径，RenderDoc
只能在产品恢复后验证draw/dispatch/readback未回归，不能证明command apply瓶颈消失。

## Unreal主依据与统一结构计划

UE Mass不是逐entity保存一条opaque transaction。`MassCommandBuffer.h:391-410`按command type index复用唯一
command instance；`MassCommandBuffer.cpp:109-225`按operation type flush整个batch后reset并保留容量。
`MassCommands.h:618-789`的add/remove先聚合全部entity，再按archetype建立collection；带值插入在
`1429-1535`维护target与typed parallel payload，交给一次batch API。build在`1749-1821`按target archetype
和range提交；destroy由`MassCommands.h:321-358`与`MassEntityManager.cpp:945-985`先生成archetype range再
批量销毁chunk。opaque lambda command在`MassCommands.h:1942-2003`被明确保留为last-resort fallback。

`MassEntityManager.cpp:2550-2586`双缓冲deferred command并给observer产生的nested flush设置5次上限，说明
结构batch还需要可观测的嵌套工作预算。`MassCommandBuffer.cpp:240`也把MoveAppend标为待优化；这里借鉴的是
“typed reusable command + archetype collection/range + parallel payload + bounded flush”的边界，不宣称UE每个
细节或阈值都是Zircon答案。

| task / owner | 结构目标 | 必须证明的验收 |
|---|---|---|
| PERF-MVP-607 / Runtime08 | 以typed reusable command batch替代逐target固定事务；一次构造最终signature，按source/target archetype collection/range preflight、move和publish | target grouping近O(N)，probe不再为N(N-1)/2；每target固定31槽artifact=0、payload内存按实际值增长；archetype move/generation/lifecycle按batch计数且原子性/order等价 |
| PERF-MVP-608 / Runtime03 + Runtime08 + Runtime11 | numeric compiled lane直接拥有arena；small-page渐进增长和retention规则；count/bytes/age/deadline admission与有界apply | warm merge String compare/sort/linear arena lookup=0；空/小producer不预留64 KiB；peak后retained bytes受预算；风暴下主线程p99有界且无静默丢失 |
| PERF-MVP-609 / Runtime08 | spawn token用queue-local dense ordinal/range一次reservation/publish；stable public map只在report冷边界投影 | hot path tree build/clone=0；工作与S线性；duplicate/stale/failure/panic/nested-window/entity-ID行为等价 |

动态矩阵必须覆盖targets 1/1k/100k、lanes 1/8/64/1k、component width 0/1/8/31、payload
0/64/192/193字节、spawn/insert/remove/despawn和observer nested depth；记录上述counter、p50/p95/p99、RSS、
CPU/lock/调度/energy，并在同一硬件同一场景前后比较。WPR/xperf/Tracy是CPU/调度/功耗authority；RenderDoc
只做GPU无回归辅助。取得这些证据前，不宣称瓶颈消失、功耗接近UE经验值或算法达到最优规模。

本切片继续留在`pending.md`，不进入`review.md`；编译门、规模counter、产品trace和独立复核完成前，不提交
性能里程碑，也不发送企微完成消息。
