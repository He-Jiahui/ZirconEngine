---
related_code:
  - zircon_runtime/src/core/runtime/events
reference_code:
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - six production Rust files reviewed
  - one source-level RED to GREEN topic-key guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, event throughput counters and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core EventBus逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/events/**` 6/6个生产Rust文件，当前798行/1个inline源码守卫。范围覆盖topic/subscriber COW snapshot、publish delivery、bounded/latest/lossless queue、subscribe/unsubscribe、timeout与diagnostics。

## PERF-MVP-323：默认诊断与topic级串行化放大每次publish

`EventBus::default`启用完整诊断。一次有订阅者的publish会获取全局topics Mutex并clone topic Arc、分配Arc event、获取topic delivery Mutex、获取subscriber snapshot Mutex；每subscriber再锁queue，读取Instant，并执行queued/delivered/peak等共享原子，最后publish duration再执行samples/total/max原子。相同topic多publisher被delivery锁完全串行化；unsubscribe也持delivery锁逐项排空queue并更新诊断，lossless积压可形成长停顿。

已有topic新增订阅原先还为`HashMap::entry`无条件clone topic String。新增RED→GREEN守卫把owned String直接交给Entry，Occupied路径不再复制lookup key；Vacant仅为EventTopic authority复制一次名称，订阅/保留语义不变。

## 参考引擎与目标分层

Bevy typed `Messages<M>`使用双Vec generation buffer与reader cursor，batch write不为每reader建立队列，也不在每message执行topic字符串查找；它不覆盖Zircon跨线程动态JSON、Latest和lossless backpressure，不能直接替换。Runtime07应把MVP帧内typed事件分流到generation buffer/cursor，跨线程/动态topic才使用现有bus；bus提供publisher/topic token、batch publish、diagnostics off/sampled/sharded模式和有界lossless预算。

## 验收要求

对topics 1/100/10k、publishers/subscribers各1/10/100、events 1/100/1M、payload 0/1 KiB/1 MiB及diagnostics off/on记录topic/subscriber/queue locks、Instant calls、atomic RMW、Arc/String alloc、queue depth/age/drop和throughput/p95：existing subscribe key clone=0；frame-local typed lane无topic/queue lock和per-reader event clone；diagnostics off bookkeeping近0，on摊销/采样有界；同topic publisher无异常串行积压，lossless有byte/event budget。顺序、Latest/drop-oldest、disconnect/timeout、Cargo/F2 WPR通过前，6文件留在`pending.md`。
