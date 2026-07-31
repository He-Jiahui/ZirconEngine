---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: typed-asset-event-shared-bounded-dispatch
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/core/resource/manager/resource_manager.rs
  - zircon_runtime/src/core/resource/manager/events.rs
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
tests:
  - cargo test -p zircon_runtime --lib typed_asset_receiver --locked --jobs 1 -- --nocapture --test-threads=1
  - asset event storm, stalled consumer, rename/remove ordering and bounded RSS fixtures
---

# Runtime04：typed asset event共享有界分发缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset root/load/facade逐Rust文件性能审查，PERF-MVP-492
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：类型化facade的局部线程可直接删除，但resource/asset generation事件的容量、合并、顺序和consumer cursor属于Runtime04共享资产生命周期合同。
- 生命周期键：`typed-asset-event-shared-bounded-dispatch`

## 失败现象与复现证据

本轮已让`AssetEventReceiver<T>`直接消费底层resource receiver并原地过滤，消除每订阅一个`asset-event-filter-*` OS线程、shutdown通道和第二个无界队列。剩余`ResourceManager::subscribe`仍为每subscriber创建无界channel；`broadcast`持subscriber mutex逐项clone+send，慢或暂停的动态场景consumer会持续积压事件与RSS。

## 最低共享层根因

资源管理器没有generation-owned有界事件日志、consumer cursor、同resource revision合并规则或slow-consumer诊断；每个receiver私有队列成为无限历史truth。

## 架构修复验收

- 发布共享有界event log/ring，consumer以cursor读取；容量同时受entry/bytes/age预算约束，并暴露depth/age/coalesce/drop/lag诊断。
- 同`(resource kind,id)`可覆盖的Added/Updated按revision合并；rename/remove/reload-failed与生命周期边沿保持确定顺序，slow consumer可检测generation gap并重取snapshot。
- producer不在subscriber全局锁内执行N次channel send；订阅/退订和广播并发不阻塞资产发布热路径。
- typed receiver保持现有`recv/recv_timeout/try_recv`语义且不创建线程/二级队列；dynamic scene reload复用PERF-MVP-471预算与cancel。
- assets/subscribers 1/100/10k、events 1/1k/1M、stall 0/1/60s下RSS硬有界，producer p95不随历史积压增长，latest revision与边沿事件可验证。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止只把第二级typed channel改为bounded而保留每订阅专用线程和底层无界队列。
- 禁止静默drop rename/remove/failure，或让consumer无法识别cursor gap并恢复一致snapshot。

## 修复结果与回传

Open state: typed facade局部止损已完成；共享有界分发与产品storm验收仍`待修复`，no pass is claimed.
