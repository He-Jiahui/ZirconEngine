---
related_code:
  - zircon_runtime/src/core/manager
  - zircon_runtime/src/core/resource
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - core manager five of five Rust files reviewed
  - core resource eighteen of eighteen Rust files reviewed
  - source-level RED to GREEN performance guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, concurrency stress and product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core manager/resource逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/manager/**`当前Rust文件5/5与`zircon_runtime/src/core/resource/**` 18/18，覆盖manager weak resolver/service handles、resource locator/id/record registry、typed payload/snapshot/lease、runtime slot/ref-count、ready/reload/rename/remove、subscriber broadcast与测试。Manager resolver以weak core handle避免保活环，服务名解析的String/handle clone主要位于获取manager边界，未在此建立重复根因。

## PERF-MVP-327：resource访问和注册的冗余owner

typed `get`原先调用`snapshot`，为只返回payload也深clone完整ResourceRecord；`acquire`随后又clone一次payload Arc。`register_ready`clone previous record、对dependency/locator/hash等执行两次revision比较，再clone new record进registry；普通register也clone全record，reload则clone→upsert并无必要重建locator index。ready export排序在每次ID tie compare中把UUID双方格式化成String。

本轮以RED→GREEN守卫让typed get直接在同一registry/payload读窗口完成kind检查和一次Arc clone，acquire复用它；registry upsert借用旧record删除locator，register把record move进authority，ready revision只比较一次；reload/error在原record上修改，ready export直接比较`ResourceId: Ord`。locator、revision、event、last-good payload、typed downcast及返回owner语义保持不变。

## 剩余并发与调度根因

当前ResourceManager把registry/payload/runtime/subscribers拆为四把锁，但`acquire`先读取payload、释放读锁后才增加runtime ref-count；最后一个release可在两步之间把manager payload移除，形成“新lease存活但authority无payload”的竞态。每次acquire还分配一个`Arc<dyn Fn>`闭包并clone整个manager；drop同步进入runtime写锁，ref-count归零再进payload写锁。broadcast持subscriber Mutex对无界channel逐个send并为每个subscriber clone含locator String的event，慢消费者没有event/byte/age预算；ready export仍全量clone record并排序。

对照`dev/bevy/crates/bevy_asset`：Bevy strong handle共享`Arc<StrongHandle>`，最后一个Arc drop只发送含dense index/type的轻量DropEvent，asset events先写本地队列再批量进入frame messages；这避免每个handle实例各自拥有释放闭包。Fyrox也把资源生命周期集中在共享manager state并由task pool处理加载，但其全局Mutex警告说明Zircon不应简单收敛成一把大锁。

Runtime04/07应建立per-resource `Arc<ResourceEntry>` authority：record generation与payload在同一entry事务发布，strong handle clone只做Arc RMW，last-drop只发送小ID/generation到有界回收lane，由预算化manager drain验证generation后卸载；registry locator映射和entry arena分离。事件使用ID/kind/revision的Copy header和按需locator查询，frame批量发布、cursor消费并限制events/bytes/age；snapshot返回Arc record/generation视图，完整owned export按需执行。

## 验收要求

对resources 1/100/10k、record dependency/diagnostic bytes 0/1 KiB/1 MiB、threads 1/8/64、acquire/release 1/100/1M、subscribers 0/1/100与reload burst 1/100/10k记录record/locator clone bytes、Arc/closure alloc、four-lock wait/hold、event clone/queue bytes/age/drop、sort allocations、race outcomes和p95/RSS：typed get record clone=0、acquire payload Arc clone=1且closure alloc=0、register deep compare≤1、ID comparator alloc=0；并发acquire/last-release不丢authority，drop不在任意线程同步拿多把manager锁，队列有界公平；reload/rename/last-good/order/poison parity、Cargo/F0/F2/F4 trace通过前，两目录留在`pending.md`。
