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
  - zircon_runtime/src/scene/event_mirror/registration.rs
  - zircon_runtime/src/scene/event_mirror/subscription.rs
  - zircon_runtime/src/scene/world/event_mirror.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/dynamic_api/session/event_mirror.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/status.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_slot.rs
  - zircon_runtime/src/dynamic_api/session/registry/session_store.rs
  - zircon_runtime/src/dynamic_api/tests/session_lifecycle.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/sink/worker.rs
  - zircon_runtime/src/diagnostic_log/sink/tests/lifecycle.rs
  - zircon_runtime/src/scene/tests/ecs_event_mirror.rs
  - zircon_plugins/navigation/runtime
tests:
  - cargo test -p zircon_runtime --lib ecs_event_mirror --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib event_mirror --locked --jobs 1 -- --nocapture --test-threads=1
  - cargo +1.94.1 test -p zircon_runtime --lib --locked --jobs 1 final_dynamic_session_release_retains_retry_authority_after_shutdown_timeout -- --nocapture --test-threads=1
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

Open state: `resolving_failure`; no fixed return is claimed.

当前源码已把 observer、typed queue 和 reader record 收归 World-owned generational slot；公开
`RuntimeEventMirrorSubscription` 只保留 descriptor、handle 与 bounded reclaim channel。token Drop
只提交去重回收意图，不持有 World 指针、不执行 callback。显式 unsubscribe 与 Drop reclaim
共享 take/restore/retire 状态机；reader-count callback 失败时恢复原 reader count、observer 与
generation，并把 Drop 路径重新排队。`WorldDriver` 在首个 schedule stage 前批量处理回收，
dynamic Session shutdown 先丢弃全部 token，再在 World lane 收敛回收。
session unload 成功条件现包含 reclaim retry 状态；World teardown 会把仍 live 的 generational
handles 统一送入同一回收路径。reclaim queue 以 live-handle 集合作为 hard budget 与 stale
generation 权威，因此 World quiesce 后延迟 token Drop 不会污染已复用 slot。
dynamic ABI registry 仅在该 quiescence 成功后移除 Session；失败返回 teardown-incomplete 并保留
closing Session 供显式 destroy 重试，禁用的 action/wake 入口不会重新开放。`World::drop` 只是
普通所有权销毁的最终兜底，动态库卸载的可重试权威由 registry slot 持有。
最终 process-log lease 同样服从该重试协议：controller 只在 worker 已确认停止后把最后一个
dynamic session count 归零；超时会恢复 active sink generation，并由 closing Session 保留可变
lease。后续显式 destroy 重试成功后才从 Session 清空 lease，避免 worker 仍可执行动态库代码时
registry 已移除 owner。

尚未宣称 fixed：当前 immutable snapshot 的受管 Rust 1.94.1 focused tests、独立二次审查、
managed atomic commit、Navigation 产品级 producer-idle trace 以及来源 Performance01 要求的
RSS/p95 证据仍待完成。源码变更按 correctness leak 处理，不声明为已测量性能优化。

## 完成项目与证据

| 日期 | 状态 | 完成项目与验证证据 |
|---|---|---|
| 2026-07-22 | `RED / reproduced` | public token Drop 无法通知 World，observer、reader count 与 Navigation callback 保持 live。 |
| 2026-08-10 | `TDD/source implemented / validation pending` | 新增 1/100/10k bounded reclaim、callback failure retry、WorldDriver ordering 与 Session Drop 回归；实现 World-owned generational record、bounded deduplicated reclaim queue 和 N->0 单 callback edge。二次源码自审已消除 drain 后逐句柄全队列扫描造成的 O(N^2) 回收路径；Rustfmt/diff/结构静态门禁通过，immutable 独立二审与受管 Cargo 尚待完成；无 Cargo GREEN 或性能数值声明。 |
| 2026-08-10 | `second-review repair implemented / validation pending` | exact34 二审指出 session 忽略 reclaim report 且 World teardown 无 quiescence owner。现将 retry_pending 并入 unload 成功条件，World Drop 主动收敛所有 live handles，并新增 foreign World、显式/Drop 混合、slot generation 复用、World destroy 与 callback 持续失败重试回归。旧 `f1412406...` 快照已失效，fresh immutable 二审与受管 Cargo 尚待完成。 |
| 2026-08-10 | `teardown retry owner implemented / validation pending` | snapshot1549 二审确认 ABI registry 在 teardown-incomplete 后仍提前 drop/remove Session。现由 registry slot 保留 failed teardown 的 closing Session，后续显式 destroy 可重试；新增第一次失败、第二次成功且 reader count 最终归零的直接 registry 回归。快照与受管 Cargo 证据待重新生成。 |
| 2026-08-10 | `stale lifecycle guard repaired / validation pending` | snapshot1552 二审发现 dynamic API lifecycle 静态守卫仍读取 registry façade 并断言已删除的 take/drop 流程。现改为读取 `session_store.rs`，按源码顺序锁定 failure preserve-retry 在成功 take/remove 之前，并把测试名纳入 `event_mirror` focused filter。snapshot1552 与对应票据仅保留为 stale 诊断证据。 |
| 2026-08-10 | `process-log retry owner repaired / validation pending` | 后续二审发现 process-log shutdown 超时前已消费 lease 并把 dynamic session count 减为 `0`。现改为借用式 shutdown：最后 worker 未停止时 count 保持 `1`、active generation 恢复、Session lease 不清空；新增 blocked-output 首次超时、释放后第二次成功且 count/active state 收敛的回归。fresh immutable snapshot、受管 Rust 1.94.1 与独立二审仍待完成。 |
