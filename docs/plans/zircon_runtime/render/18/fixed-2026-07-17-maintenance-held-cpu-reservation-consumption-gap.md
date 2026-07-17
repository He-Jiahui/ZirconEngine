---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: maintenance-held-cpu-reservation-consumption-gap
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
origin_workflow_node: M3
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cli.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/tests/test_supervision_service.py
  - tools/session_coordinator/tests/test_maintenance_cpu_reservation_consume.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_supervision_service tools.session_coordinator.tests.test_server tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_cargo_reservations
resolved_at: 2026-07-17
---


# Coordinator01: maintenance hold could not safely consume an exact lane reservation

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：Render18 AF-M3 volumetric media retest 的受管 CPU FIFO validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Render18 已有 active Session、26 项精确文件租约和 canonical CPU reservation；将其转为唯一未启动 job 的控制面边界由 Coordinator01 所有。

## 失败现象与复现证据

当前 schema 41 daemon 处于 `draining + maintenanceHold=true`，无 active Cargo；
Render18 Session `render18-af-m3-volumetric-retest-20260716` 是 active，且持有
27 项精确租约与 pending CPU reservation `b681198b51354f90bfe2c21395aa675c`。

Shader03 的下游前置已经由受管 exact commit
`46435d575713a249ca95a709cd261d2e95632897` 完成，包含 M2 acceptance、fixed return 和
contract test 三项路径。此时不应通过 generic resume 放开其它 Session；但当前维护保持也会在
`CargoJobService.acquire()` 之前拒绝 `cargo.acquire`，使该 exact reservation 无法转为 lease。

不得以 `--dry-run` 作为探针：现有 acquire 即使 dry-run 仍会创建 job 并消费 FIFO reservation，
因此在缺少 narrowly-scoped control action 时不能进行生产复现。

## 最低共享层根因

`SupervisionService.require_mutation_allowed()` 只对 configured maintenance Session 的
`_MAINTENANCE_SESSION_OPERATIONS` 放行 metadata 操作；`cargo.acquire` 不在该集合。
与此同时，`CargoJobService.acquire()` 是唯一在同一事务中验证 CPU 队首、Session、
compatibility key、target reuse，并把 pending reservation 绑定为 `leased` job 的路径。

另有已验证的加载根因：`CoordinatorApplication._maintenance_session_ids_for_startup()`
此前按 `service_supervision_events` 的最新 drain/resume 事件选取 scope。同状态、同 reason
的后续 union drain 会被事件去重，因此新 action 的 scope 不会在重启后恢复。正确来源是最后一条
成功的 `service.drain` action（每次 action 输入显式携带完整 union），再并入本地启动环境 scope。

因此系统只有两个错误选择：保持 hold 时拒绝 Render18，或 generic resume 后允许任何 Session
抢占下一 CPU/GPU lane。CPU 原先缺少“仅消费本 Session 已存在 canonical reservation”的类型化操作；
GPU 还只能依赖 resume action 的临时 Session 指示，不能把指定 target 与 command 一起持久化，因而
无法在 hold 中创建可审计的单一 DX12/RenderDoc lease。

## 架构修复验收

- 增加一个受管、类型化的 reservation-consume 操作：只接受指定 pending CPU reservation，
  并在单一事务中验证 owner Session、队首、canonical compatibility payload、expiry 与
  executable Session 状态后创建一个 `leased`、未启动的 CPU job。
- 维护保持下只能对 daemon 明确配置的 Session 放行该 consume 操作；普通
  `cargo.acquire`、任意新 reservation、任意 generic resume 和任何其他 Session 仍必须拒绝。
- consume 不接受客户端覆盖 target、compatibility 或命令；target 必须由存储的 canonical
  compatibility pool 决定，后续 `cargo.start` 继续用 reservation 的 command fingerprint 校验。
- exact job 启动同样不得退回 generic `cargo.run`：在 controlled action 恢复 owner Session
  `active` 后，只能通过 reservation-bound run 路径复核 job、command fingerprint 与由 canonical
  payload 导出的 allowlisted environment。
- 对 GPU 使用等价的 typed `reserve-gpu` / `consume-gpu-reservation`：reservation 必须持久化
  `lane_scope=gpu`、canonical compatibility、精确 target 和 command fingerprint；consume 只能
  生成唯一未启动 GPU job，并禁止 client target/compatibility/environment 覆盖。
- GPU reservation 与 CPU FIFO 语义独立，但全局 GPU lane 在 `pending`、`leased`、`running` 或
  `finished` 时只能存在一项；generic `cargo.acquire gpu` 不得绕过该 durable claim。
- 同一 reservation 的重复 consume 必须幂等地返回同一 job 或给出已消费状态，不能生成第二个 job；
  foreign、expired、released、compatibility 不匹配或非队首请求都必须拒绝且不改变 FIFO。
- 增加 focused service/server/cargo regressions，并在持久 maintenance hold 中验证：仅该
  Render18 reservation 变为 one leased-unstarted job、无 Cargo PID、无 generic admission。
- 修复加载后由 Render18 使用该实际 job 执行既有 exact command；不得 raw Cargo、手工 SQL、
  重新创建/复用历史 reservation，或清除 hold 作为替代方案。

## 禁止临时方案

- 禁止把 `cargo.acquire` 整体加入 maintenance allowlist，或在 hold 中开放普通 CPU/GPU acquisition。
- 禁止为 Render18 临时 generic resume、手工创建 Cargo job、直接改写 reservation/job SQLite 行。
- 禁止把 `--dry-run` 当作无副作用的 FIFO 探针。
- 禁止改变 Render18 的命令、兼容键、target 复用策略或 27 项已租约的源码范围来绕过此问题。

## 修复结果与回传

- 根因：A typed reservation consume path was missing under maintenance hold, and the record used AF-M3, a prose slice label absent from the Render18 machine workflow topology, so graph import could not bind the origin node.
- 架构修复：Coordinator reservation consumption validates owner, FIFO head, canonical compatibility, target, and command fingerprint before creating the sole leased-unstarted job under hold; the failure graph now binds the origin to canonical Render18 M3.
- 验证：Dedicated maintenance reservation regressions passed 13/13. Failure import audit is zero diagnostics after origin_workflow_node=M3; the record retains prior focused implementation evidence.
- 回传：Render18 owns only the generated fixed child-record lease. No Cargo job, generic admission, drain, or foreign reservation changed while correcting the graph and returning this failure.
