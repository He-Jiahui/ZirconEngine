---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: runtime-event-mirror-drop-lifecycle
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/12
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/event_mirror/subscription.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_plugins/navigation/runtime
tests:
  - cargo test -p zircon_runtime --lib ecs_event_mirror --locked --jobs 1 -- --nocapture --test-threads=1
  - subscribe-drop/destroy/reload reader-count and producer-idle stress
---

# Plugins12：runtime event mirror drop生命周期交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene event mirror 4/4逐Rust文件性能审查，PERF-MVP-455
- 修复责任计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 交接原因：Plugins12拥有typed event mirror公开consumer合同与editor/runtime mirror wiring；最低根因是subscription token、World reader record和plugin/session生命周期没有统一owner。
- 生命周期键：`runtime-event-mirror-drop-lifecycle`

## 失败现象与复现证据

`RuntimeEventMirrorSubscription`公开持有typed ECS subscription与`connected`状态，但Drop不能通知World。direct consumer在connected状态drop会遗留World mirror reader count、typed reader连接与reader-count callback enabled状态。Navigation以该callback控制debug capture，泄漏token可让按需producer在长会话永久逐帧工作；World/plugin/session销毁也缺统一的恰一次disconnect证明。

Dynamic session当前显式unsubscribe并在失败时保留local ownership供重试，因此产品主链概率较低，但这不是公开Scene contract的生命周期保证。PERF-MVP-432的bounded delivery只限制批量/JSON，不解决reader owner泄漏。

## 最低共享层根因

subscription executable state被token直接拥有，World registry只保存registration/count，没有generational subscription record或reclaim queue；Drop既无安全World handle，也没有可提交disconnect意图的owner lane。plugin reload/session destroy与direct token drop因此不是同一个quiescence协议。

## 架构修复验收

- World拥有generational subscription record与typed cursor；公开token只持稳定handle/owner channel，Drop提交有界reclaim意图，不持裸World引用、不执行foreign callback。
- 显式unsubscribe、token drop、session destroy、World destroy与plugin reload统一恰一次disconnect状态机；失败可重试且不会双减reader count。
- reader count与callback最终收敛到真实live token数；N→0后Navigation等按需producer下一帧idle work=0，旧generation in-flight delivery安全完成。
- 1/100/10k subscribe/drop/unsubscribe、callback failure、destroy/reload交错记录live readers、reclaim queue、callback edges、retained events、producer frames、RSS与p95；无泄漏、无double disconnect、队列有hard budget。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止给subscription保存裸`*mut World`或在Drop里同步取得World/plugin全局锁执行callback。
- 禁止仅给当前dynamic session consumer补`defer unsubscribe`而保持公开direct API泄漏。
- 禁止用定时全表清扫掩盖缺失的generation/lifetime owner。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
