---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: ecs-event-message-bounded-lifecycle
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/events
  - zircon_runtime/src/scene/ecs/messages
  - zircon_runtime/src/scene/ecs/system/events.rs
  - zircon_runtime/src/scene/ecs/system/messages.rs
  - zircon_runtime/src/scene/world/events.rs
  - zircon_runtime/src/scene/world/messages.rs
tests:
  - cargo test -p zircon_runtime --lib ecs_events_messages --locked --jobs 1 -- --nocapture --test-threads=1
  - 10k idle frames and 1M retained-message counters
---

# Runtime08：ECS event/message bounded lifecycle交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：scene ECS events/messages 12/12逐Rust文件审查，PERF-MVP-485
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：Runtime08 M3明确拥有events/messages分工、清理策略与cursor语义。
- 生命周期键：`ecs-event-message-bounded-lifecycle`

## 失败现象与复现证据

`Messages<T>`把全部payload保存在单个Vec，只有调用方显式`clear_messages<T>`才释放；仓内产品代码没有清理调用，连续writer会让entries和RSS随会话增长。`Events<T>`虽用current/next双缓冲和debounced capacity shrink保持两代有界，但`EventStore::update_all`每帧遍历所有注册channel并虚调用update，reader_count与本帧dirty状态不参与推进集合。

## 最低共享层根因

events/messages没有统一World/schedule lifecycle authority：messages把retention责任下放给未知consumer，events则以全registry扫描换取推进；channel缺少dirty membership、consumer cursor watermark、entry/byte/age预算和drop/backpressure diagnostics。

## 架构修复验收

- 明确定稿events与messages的持续时间和消费语义；messages采用cursor-aware generation/ring或等价结构，并同时有entry、byte、age硬预算与drop/lag指标。
- World/schedule拥有唯一推进/回收点；调用方无需记住每帧clear，slow reader策略显式且可观测。
- EventStore维护dirty/retirement channel集合；只有send、仍有current payload或容量回收倒计时的channel参与update，stable idle不全扫registered types。
- dormant event subscription保持late connect不回放历史；connected reader顺序不丢不重，clear/generation reset语义不变。
- types 1/1k/100k、writes 0/1/1M、idle 10k frames记录channel visits、retained entries/bytes、drop/lag与p95：idle visits近0，RSS严格有界。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅在编辑器或某一个system手动clear，继续让其他producer无界。
- 禁止无指标地静默丢弃oldest消息或按reader_count为0跳过必要generation推进。
- 禁止为dirty set复制另一份channel payload/queue truth。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
