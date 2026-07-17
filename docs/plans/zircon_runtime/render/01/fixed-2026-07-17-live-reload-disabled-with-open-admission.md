---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: live-reload-disabled-with-open-admission
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/control_plane/actions/catalog.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/cli.py
tests:
  - tools/session_coordinator/tests/test_supervision_actions.py::SupervisionActionTests::test_rollover_preserves_admission_and_unstarted_work_for_successor
  - tools/session_coordinator/tests/test_supervision_actions.py::SupervisionActionTests::test_rollover_rejects_a_live_managed_cargo_tree_without_draining
  - tools/session_coordinator/tests/test_action_catalog.py::ActionCatalogTests::test_lifecycle_parameters_are_service_scoped_and_bounded
resolved_at: 2026-07-17
---


# Coordinator01：准入开启时无法加载已验证的调度修复

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行者：`render01-deferred-graph-mesh-pipeline-resources-20260716`
- 来源执行切片：directional parity 的单测试、保留 target 受管绑定
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：本地服务的 lifecycle 策略和单实例 successor 启动由 Coordinator01 独占；Render01 无权停止、替换或热加载协调器。

## 失败现象与复现证据

- Render01 many-point parity `28cb7f882ce941fabde99689ca44cad7` 已受管完成并释放，保留 target 为 `D:\cargo-targets\zircon-engine\pool\e6d4c9249106da748fd8ddc595a70ab1f3bbd574f309ae58d55a73cc07f88c21`。
- 新的 CPU reservation target 绑定能力已完成本地回归，但当前 daemon 仍是旧内存实例，无法接收 `target_dir`。
- 在没有 live Cargo PID、仅存在未启动 lease 时，官方 `zircon-session stop` 返回 `lifecycle_global_shutdown_disabled`：`Global stop, restart, and force-stop are disabled while task admission is open`。
- 因而无法在不关闭准入、不排空、不重写外部 lease 的前提下加载已验证修复。随后错误 payload 的 `ad9738792c97485a86c30dac361a536c` 自然终态 exit 101；它不属于 directional parity evidence。
- 随后启动的 `d57cb5f09ee24b9685c36e63fd445457` 虽然只包含 directional `--lib` filter，但绑定了错误的 `F:\cargo-targets\...\7ad7c2...` compatibility/target，而不是被要求保留的 `D:\cargo-targets\...\e6d4c924...`。它是正在运行的 foreign managed tree，必须自然终态，且不能作为 canonical parity evidence。

## 最低共享层根因

`CoordinatorClient.shutdown()` 只能走全局 `service.stop`，而 lifecycle policy 将所有 stop/restart 都绑定为“准入关闭”操作。控制面、SQLite action enum 与 lifecycle intent enum 也没有一个针对“无 live PID、保留未启动 job/reservation、单实例 successor”的受管 rollover 契约，因此 source 修复不能在正常开发准入下安全加载。

## 架构修复验收

- 提供明确的 Coordinator-owned rollover 路径：只有所有 managed Cargo 的 live PID 为空时才允许；存在真实 PID 时必须拒绝且不终止进程。
- rollover 必须保留未启动 lease、CPU reservation、兼容性 payload、target 和 FIFO 顺序；不得将其孤儿化、释放或重新创建。
- successor 必须恢复固定 6518 descriptor，并证明同一 repo 只有一个 serve 实例；正常准入不变，不进入 draining/maintenance hold。
- 原始向上门禁：Render01 只能在 successor 上创建带 e6d4 target 的 canonical `--lib` directional reservation，运行时原始输出必须显示 `running 1 test` 和该测试名称通过。

## 禁止临时方案

- 不得杀死、释放、接管或手工修改任何 foreign lease/job。
- 不得用 global drain、maintenance hold、端口切换、平行 daemon 或未记录 target 绕过 lifecycle。
- 不得把 generic `cargo acquire`、broad `shadow` 测试、错误 compatibility pool 或 `ad973…` 的结果当作 Render01 directional parity。

## 修复结果与回传

- 根因：CoordinatorClient previously exposed only global stop/restart paths, so a safe successor could not be started while preserving unstarted reservations and admission state.
- 架构修复：The controlled service.rollover action accepts only zero live managed Cargo PIDs, preserves unstarted leases, reservations, compatibility targets, and FIFO state through successor reconciliation, and refuses real PID trees without enabling draining or maintenance hold.
- 验证：Current exact regressions passed 3/3: rollover preserves admission and unstarted work for the successor; rollover rejects a live managed Cargo tree without termination; lifecycle parameters remain service-scoped and bounded. The source handoff retains its action and migration evidence.
- 回传：The current schema successor uses non-draining rollover on the fixed 6518 endpoint while preserving pending work; no global stop, drain, or foreign job mutation was used.
