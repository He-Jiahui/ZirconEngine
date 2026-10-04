---
related_code:
  - zircon_runtime/src/scene/event_mirror
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/scene/tests/ecs_event_mirror.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
reference_sources:
  - dev/bevy/crates/bevy_ecs/src/message/message_cursor.rs
  - dev/bevy/crates/bevy_ecs/src/message/messages.rs
tests:
  - zircon_runtime/src/scene/tests/ecs_event_mirror.rs
  - current-source Windows zircon_runtime event-mirror tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime scene event mirror逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/scene/event_mirror/**`当前源 **4/4** 个Rust文件、**354** 行已逐文件阅读；并完整阅读132行、4 tests的`scene/tests/ecs_event_mirror.rs`，继续核对`scene/world/event_mirror.rs`与`dynamic_api/session/event_mirror.rs`真实产品调用链。范围覆盖typed event factory、descriptor/registry/reader count、subscription connect/disconnect/drain及JSON erased boundary。

## 已直接修复

`RuntimeEventMirrorSubscription::drain`原先在检查connected与执行typed drain前无条件clone `event_id`；成功批次从不消费这个String，因为event id只属于Disconnected/Serialize错误。现先执行connected gate与typed drain，仅在两种错误构造时clone id，稳定成功drain的scene层event-id clone降为0。源码守卫先观察RED、实现后GREEN，scoped rustfmt/diff check通过；登记PERF-MVP-454。current-source受管Cargo取测试lane时被`runtime13-plugin08-script-call-table-atomic-hardcut-20260722`精确预约，本轮未启动Rust测试。

## 既有无界delivery根因

typed subscription对全部未读事件逐个`serde_json::to_value`并一次性collect `Vec<Value>`；dynamic session随后clone完整descriptor、为每delivery再clone event/schema Strings、构造第二个Vec并整体JSON encode。空批也继续到ABI编码，且dynamic session mutex覆盖该链。该根因已由PERF-MVP-432与Runtime10 open `plugin-event-bounded-delivery` handoff准确覆盖，本轮补入scene producer文件，不建立重复计划。

## 生命周期性能泄漏

公开`RuntimeEventMirrorSubscription`持`connected: bool`但没有Drop cleanup，也没有World-owned subscription token。直接consumer在connected状态drop时，World registry的reader count与typed ECS reader连接不会回减，reader-count callback也不收到N→N-1/0边沿。Navigation现用该callback控制`NavigationDebugCapture`；因此泄漏subscription可让本应按需的capture/producer工作永久保持启用，并使event retention/reader状态在长会话积累。

产品dynamic session显式unsubscribe与失败重试降低了主链发生率，但公开Scene API仍允许该状态，不能以“当前consumer守规矩”代替生命周期契约。问题登记PERF-MVP-455并交接Plugins12：World应拥有generational subscription record，公开token drop只提交有界disconnect/reclaim意图；World销毁/插件reload/session destroy统一quiesce，reader count/callback恰一次收敛。不能让Drop持裸`&mut World`或静默忽略失败。

## 参考引擎对照

Bevy `Messages`用明确cursor跟踪读取位置并由双generation buffer维护消息生命周期；reader cursor是消费状态而不是在payload层复制descriptor与JSON。Zircon跨动态ABI仍需要序列化，但应把typed cursor、descriptor generation、bounded batch与consumer lifetime留在owner层，只在预算内的ABI边界编码一次。

## 动态验收

1. current-source scene event-mirror Cargo：schema/duplicate、current-only drain、multi-reader count、unsubscribe rollback与新增success no-id-clone guard。
2. 0/1/1k/10k/1M events、payload 0/1KiB/1MiB、1/16 consumers：记录typed reads、Value/descriptor/String clone、JSON bytes、batch count/time/bytes、queue peak/age/drop/coalesce、session lock hold与p95；PERF-MVP-432完成后全部预算有界、empty encode=0、stable descriptor clone=0。
3. 1/100/10k subscribe→drop/unsubscribe、World/session destroy、plugin reload及callback failure：记录live readers/callback count、retained events、debug capture frame count与RSS；PERF-MVP-455完成后drop/destroy最终reader=0、callback边沿恰一次、producer idle work=0且无双disconnect。

动态验收未完成，因此该目录继续保留在`pending.md`，不进入`review.md`。
