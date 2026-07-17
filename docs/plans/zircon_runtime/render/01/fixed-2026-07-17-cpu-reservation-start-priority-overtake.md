---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: cpu-reservation-start-priority-overtake
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cli.py
tests:
  - tools/session_coordinator/tests/test_cargo_reservations.py::CargoReservationTests::test_unreserved_cpu_lease_cannot_start_ahead_of_consumed_priority_reservation
  - tools/session_coordinator/tests/test_cargo_reservations.py::CargoReservationTests::test_cpu_reservation_preserves_explicit_approved_target_when_consumed
resolved_at: 2026-07-17
---


# Coordinator01：CPU 预约绑定后旧通用租约可抢先启动

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行者：`render01-deferred-graph-mesh-pipeline-resources-20260716`
- 来源执行切片：deferred mesh-pipeline resources focused/parity/compile gate
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：CPU lane 的 acquire/reservation/start 状态机由 Coordinator01 集中实现；Render01 无权修改外部 Frameworks05 作业或绕过调度器。

## 失败现象与复现证据

- `Frameworks05` 作业 `765a0c8a69d745debea22813a503b801` 在 `2026-07-17 00:25:05 +08:00` 获得了无预约 CPU lease。
- `Render01` 的优先预约 `41fdb628b0cd4e118229475d3827a162` 在 `00:25:12 +08:00` 建立，绑定作业 `c753b96194c54b32b4bd8d3ae15b0f6b` 于 `00:25:18.735 +08:00` 创建。
- 外部无预约作业仍在 `00:25:18.750 +08:00` 获准启动，形成 15ms 的倒灌窗口；它随后以 exit 101 释放。Render01 的真实编译树继续运行，未被触碰。

## 最低共享层根因

`CargoJobService.start()` 只验证“本作业若绑定预约则匹配预约命令”，没有在“本作业没有预约”分支检查 CPU FIFO 中已经 `leased/running` 的优先预约。`acquire()` 已经阻止预约建立后的新通用 acquire，但无法阻止先前取得 lease 的作业随后启动。

## 架构修复验收

- 已绑定的 CPU priority reservation 存在时，无预约 CPU job 的 `start()` 必须以 `cargo_cpu_lane_reserved` 拒绝。
- 预约绑定 job 仍只能执行其指纹完全匹配的命令；正常无预约任务在没有活跃 CPU 预约时仍可启动。
- CPU reservation 可在创建时记录一个受批准根目录约束的 target；consume 只能复用该已记录 target，不能用错误兼容性或启动时参数伪造 warm-pool 复用。
- Render01 → Render05 → Shader06 的已声明队列不再被预先获取、后启动的通用 lease 插队。

## 禁止临时方案

- 不得直接释放、杀死、改写或接管 Frameworks05 的历史作业。
- 不得关闭普通任务准入、恢复全局排空，或用人工排序替代原子启动校验。
- 不得弱化 FIFO、命令指纹、兼容性或目标目录校验。

## 修复结果与回传

- 根因：CargoJobService.start previously allowed an already leased generic CPU job to start after a priority CPU reservation had been bound, because only acquire-time reservation ordering was enforced.
- 架构修复：CPU start admission now rejects an unreserved job while a bound priority CPU reservation is active; reservation consumption preserves the approved target and command fingerprint instead of relying on caller ordering.
- 验证：Current exact regressions passed 2/2: unreserved CPU start is rejected behind a bound reservation, and consumed reservations preserve their approved target. The failure record also retains the source suite evidence for tools.session_coordinator.tests.test_cargo_reservations 20/20.
- 回传：The FIFO start guard is loaded in the current coordinator and the Render01 origin session holds the returned child-record destination lease; no foreign job was released or rewritten.
