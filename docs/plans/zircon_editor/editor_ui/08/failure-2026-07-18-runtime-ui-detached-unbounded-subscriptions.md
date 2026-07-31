---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: runtime-ui-detached-unbounded-subscriptions
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor_ui/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/ui/event_ui/manager/invocation.rs
  - zircon_runtime/src/ui/event_ui/manager/subscription.rs
  - zircon_runtime/src/ui/event_ui/manager/reflection_store.rs
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
---

# Runtime UI失联订阅与无界diff fanout

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`zircon_runtime/src/ui/event_ui` 10/10及editor control/reflection产品调用图
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`
- 联动责任：Editor02负责remote messaging transport生命周期；既有`failure-2026-07-17-editor-event-full-reflection-rebuild.md`负责上游incremental reflection generation。
- 交接原因：subscription transport、reflection generation与workbench projection owner属于EditorUI08及Editor02协议边界。

## 失败现象与复现证据

PERF-MVP-252：`SubscribeDiffs`创建crossbeam unbounded channel后立即丢弃receiver，仅返回id；sender永久保留。每个远程subscribe都泄漏一个死订阅，之后每个diff/invocation仍逐死订阅clone并send。有效慢consumer也可让queue entries/bytes无界增长。`replace_tree`每次还清空并重建所有tree的node index。

## 最低共享层根因

control request只返回subscription id，协议没有把receiver/sink交给transport owner；event manager又把sender存在无生命周期、无容量和无generation语义的表中。reflection changed set也没有成为node index与delivery coalescing的共享增量边界。

## 架构修复验收

- Subscribe协议必须把id绑定到真实transport stream/sink；receiver不得在请求函数内丢弃。
- transport disconnect、explicit unsubscribe与send failure都在有界时间内移除sender；下一次broadcast不得继续clone给dead subscriber。
- queue同时有entry和byte hard cap，reflection diff按tree/generation coalesce；慢consumer不阻塞editor主线程，过载策略与最终generation明确。
- `replace_tree`只增量维护目标tree changed/removed node index，其他tree visited=0；与上游PERF-MVP-076/099共享changed set。
- 1/100/10k subscribe-disconnect、1/100/10k nodes与100k diff风暴记录live/dead、queue entries/bytes、notification clone bytes、send failure、coalesce/drop、age p95与CPU p95。
- request/transport/reconnect/unsubscribe、route invocation、tree query、current-source Cargo与editor产品trace通过。

## 禁止临时方案

- 不得只在显式Unsubscribe清理，忽略disconnect与send failure。
- 不得把unbounded channel换成超大固定容量但没有byte cap/coalesce/最终generation规则。
- 不得每次广播前全表探测receiver状态，或继续每snapshot重建全部tree index。

## 修复结果与回传

Open state: `等待EditorUI08联动Editor02回传transport-owned bounded subscription、incremental node index、风暴counter与current-source Cargo证据`。
