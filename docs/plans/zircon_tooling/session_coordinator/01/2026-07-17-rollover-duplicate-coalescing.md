---
record_kind: implementation_slice
status: accepted
created_at: 2026-07-17
plan_source: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
related_code:
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/supervision/lifecycle.py
  - tools/session_coordinator/tests/test_supervision_actions.py
  - docs/tools/session_coordinator/control-plane.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_supervision_actions
  - python -m unittest tools.session_coordinator.tests.test_control_snapshot
  - git diff --check -- tools/session_coordinator/supervision/service.py tools/session_coordinator/supervision/lifecycle.py tools/session_coordinator/tests/test_supervision_actions.py docs/tools/session_coordinator/control-plane.md docs/superpowers/plans/2026-07-17-coordinator-adaptive-cpu-burst-lanes.md
---

# Coordinator01：Rollover 重复请求合并

## 验收结论

一次受控 rollover 的后继在 60 秒稳定窗口内会把第二个 rollover
转成有审计结果的成功无操作，而不会安排第二次监听器关闭。这仅影响
服务更新的重复请求：Session 准入、租约、Cargo/FIFO 状态以及稳定窗口外
的正常更新路径保持不变。

## 运行证据

- 现场曾出现两个 rollover 在 16 秒内连续成功，导致后继刚健康就又被替换。
- 回归：`test_supervision_actions` 31/31、`test_control_snapshot` 11/11，通过范围格式检查。
- 生产：在自然的空 Cargo 窗口中，受控动作 `3a172…` 创建后继 `658bb4…`；后继为 `read_write` 且 `maintenanceHold=false`。
- 在该后继稳定窗口内，受控动作 `c9cf4…` 返回 `succeeded` 与 `coalesced=true`，引用 `3a172…` 和同一后继实例；没有第二次关停或 Cargo 进程。

## 边界

- 有实时受管 Cargo PID 树时 rollover 仍以 `lifecycle_rollover_live_cargo` 拒绝，不会排空、停止或释放该作业。
- 已完成的前一实例动作不能由当前实例读取，仍受 daemon-instance 绑定保护；合并动作的结果保留前一动作 ID 作为审计关联。
