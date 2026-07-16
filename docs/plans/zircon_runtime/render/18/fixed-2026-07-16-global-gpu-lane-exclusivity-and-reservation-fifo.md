---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: global-gpu-lane-exclusivity-and-reservation-fifo
plan_link_mode: child_record_only
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/server.py
tests:
  - python -m unittest -v tools.session_coordinator.tests.test_cargo_jobs.CargoJobTests.test_gpu_lane_is_global_across_distinct_targets tools.session_coordinator.tests.test_cargo_jobs.CargoJobTests.test_gpu_reservation_keeps_fifo_until_nominated_job_reaches_terminal_state tools.session_coordinator.tests.test_cargo_jobs.CargoJobTests.test_gpu_startup_audit_reports_existing_leases tools.session_coordinator.tests.test_server.ServerTests.test_startup_audits_gpu_lease_that_predates_the_latest_reservation
  - python -m unittest -v tools.session_coordinator.tests.test_supervision_service.SupervisionServiceTests.test_explicit_stop_blocks_new_mutations_even_if_timeout_restores_healthy_state tools.session_coordinator.tests.test_supervision_actions.SupervisionActionTests.test_maintenance_hold_requires_explicit_resume_release
resolved_at: 2026-07-16
---


# Coordinator01：全局 GPU lane 互斥与预约 FIFO 被破坏

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行者：`render18-af-m2-rebase-20260715`
- 来源执行切片：AF-M3 DX12 product gate 的受管 GPU lane 预约
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：跨 target 的 GPU lane 仲裁、预约消费和启动审计均由协调器的 Cargo job 服务拥有；Render18 不得以手工抢占、非受管 Cargo 或重复 acquire 绕过 FIFO。

## 失败现象与复现证据

schema 34 在两个不同 target 上同时启动了 Frameworks05 GPU job：

- `5d667e0f37974564a02b4a7f4d2b4aa1`：2026-07-15 23:26:50 +08 开始；
- `e5d7eef9a3a24434bf88743211504f1a`：2026-07-15 23:26:59 +08 开始，随后已成为 orphaned。

两个 job 都是 `lane_kind=gpu`。这违反“单一受管 GPU lane”的服务级约束，并会让 Render18 已获批准的一次性 GPU 预约失去 FIFO 意义。来源执行者没有终止其他会话进程，也没有启动非受管 Cargo。

## 最低共享层根因

`CargoJobService.acquire()` 仅检查 target/reuse key 重叠，没有在同一 SQLite `BEGIN IMMEDIATE` 事务中拒绝现有 `lane_kind='gpu' AND status IN ('leased','running')`。同时旧的预约逻辑以 nominated job 已 `started_at` 为预约已消费条件，因此允许另一 GPU acquire 在 nominated job 仍在执行时进入。

## 架构修复验收

- GPU acquire 在插入 lease 前必须在同一写事务中检查全局活跃 GPU lane；不同 target、不同 reuse key 也只能有一个 `leased` 或 `running` job。
- 最新带 `gpuReservationSessionId` 的成功 resume 必须阻止非 nominated Session，直到 nominated GPU job 已启动并进入终态/released；启动前 release 保持该 Session 的可重试 FIFO 优先级。
- 服务启动时必须审计任何在最新预约前已存在的 GPU lease/running job，写入可查询的审计事件；不得静默忽略。
- 在 maintenance hold 生效时，`service.resume` 必须携带建立当前 hold 的 drain action ID；没有该 proof 或携带过期 ID 的 resume 不得取消排空。这样运行中的外部 Session 无法以泛化 resume 反复打开新 acquire 窗口。
- 在受控排空并安全重载后，为 `render18-af-m2-rebase-20260715` 创建一个且仅一个 `leased`、未启动的 GPU job，目标为 `E:\cargo-targets\zircon-engine\render18-af-m3-plugin`，兼容 key 为 `fd9ba63b67ac508ce0aa8dc153fe9d8d84f03017da2a72f900891be5c0f7fd00`；其原执行者负责 start/finish/release。

## 禁止临时方案

- 不得终止、release 或接管 Frameworks05、Runtime10 或其他外部 Session 的受管 job。
- 不得通过 target 目录隔离来替代全局 GPU lane 互斥，不得因已 `started_at` 就允许第二个 GPU job。
- 不得使用 raw Cargo、手工删除 target 或伪造 Render18 完成证据。
- 不得削弱 GPU lane 或预约 FIFO 回归测试。

## 修复结果与回传

- 根因：CargoJobService.acquire 只检查 target/reuse 重叠，未在同一写事务中检查全局 GPU lane；旧预约又在 nominated job 启动时过早消费，因而可让第二个 GPU acquire 插入。
- 架构修复：GPU acquire 在插入前以同一 SQLite 写事务执行跨 target 的全局 GPU lane 检查；成功 resume 的 gpuReservationSessionId 保持 FIFO，直到 nominated job 进入终态；启动时审计早于预约的 GPU lease，maintenance hold 的恢复继续要求精确 drain proof。
- 验证：四项聚焦回归均通过：test_gpu_reservation_keeps_fifo_until_nominated_job_reaches_terminal_state、test_gpu_lane_is_global_across_distinct_targets、test_gpu_startup_audit_reports_existing_leases、test_startup_audits_gpu_lease_that_predates_the_latest_reservation。schema36 实例 e3a97c6e45114976a5175fd4329fc11a 固定在 127.0.0.1:6518；Render18 exact handle da6f0c1f7eea49bc8b9707e48124145a 使用指定 target/compatibility，已由 owner 正常 start 为 running。
- 回传：Render18 继续仅通过受管 job da6f0c1f7eea49bc8b9707e48124145a 执行当前 RenderDoc capture，并负责 heartbeat/finish/release；不得通用 GPU acquire 或 raw Cargo。
