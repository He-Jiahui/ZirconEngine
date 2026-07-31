---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: plugin-event-bounded-delivery
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/event_mirror/subscription.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime_interface/src/plugin_events.rs
  - zircon_runtime/src/plugin
tests:
  - bounded mixed plugin-event storm
  - unsubscribe/destroy/reload parity
---

# Runtime10：plugin event有界delivery边界

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime dynamic API 64/64与scene event mirror 4/4逐Rust文件性能审查，PERF-MVP-432
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：Runtime10拥有dynamic session mutex、plugin-event ABI与batch encode；Runtime06/Plugins01共同冻结descriptor generation与reload lifecycle，Scene只拥有typed cursor producer。
- 生命周期键：`plugin-event-bounded-delivery`

## 失败现象与复现证据

delivery当前无上限全drain，逐payload复制subscription event/schema Strings并序列化owned JSON；空批也编码，且session mutex覆盖drain与encode。burst producer可同时放大主线程停顿、锁等待和内存。

2026-07-22 scene producer复核补充：typed mirror本身已对全部未读事件逐个`serde_json::to_value`并一次性collect `Vec<Value>`；dynamic session再构造delivery Vec和整批JSON，因此预算必须从typed cursor/read开始，而不能只在ABI encode后截断。scene层成功drain的无用event-id clone已直接删除；descriptor/per-delivery clone、empty encode与无界batch仍属于本交接。

## 最低共享层根因

typed ECS cursor、dynamic delivery DTO与ABI byte buffer是三段独立owned projection；没有跨三段共享的descriptor generation、count/time/bytes budget或可续游标。session action owner又把World drain与JSON encode都包在同一session mutex内。

## 架构修复验收

- Runtime10冻结typed descriptor-id batch ABI；Runtime06/Plugins01提供稳定descriptor generation与生命周期。
- event按lossless/latest/bounded分类，drain有count/time/bytes配额、backpressure及queue age/peak/drop/coalesce指标。
- empty encode=0、stable descriptor clone=0、序列化/大复制锁外；0/1/1k/10k/1M事件下queue/RSS有界，顺序和reload语义等价；回传PERF-MVP-432。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止先无界drain/JSON后在ABI尾端truncate；禁止为每consumer维护无精确generation失效的descriptor cache。
- 禁止在session/World/plugin全局锁内执行序列化、foreign callback或等待背压。
- lossless边沿不得静默丢弃；latest/bounded策略必须由event contract显式声明。

## 修复结果与回传

Open state: `待 Runtime10 联动Runtime06/Plugins01建立typed bounded plugin delivery`。

2026-07-22 Editor consumer复核提供反证：Editor02已有256 events/4ms和64/consumer预算，但Runtime10 gateway仍先产出完整Vec，之后才在consumer侧延期；因此RSS/ABI encode+copy不受预算。本failure的acceptance必须包含producer cursor、`max_events/max_bytes/deadline`、remaining/oldest age与slow-consumer硬内存上限，不能用Editor pump report替代。
