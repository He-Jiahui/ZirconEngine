---
record_kind: implementation_slice
status: accepted
created_at: 2026-07-15
plan_source: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
related_code:
  - tools/session_coordinator/client.py
  - tools/session_coordinator/tests/test_deferred_action_client.py
  - tools/session_coordinator/tests/test_milestone_cli.py
  - docs/cli-and-tooling/local-session-coordinator.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_deferred_action_client -v
  - python -m unittest tools.session_coordinator.tests.test_milestone_cli.MilestoneControlClientTests.test_control_action_polls_executing_confirmation_until_terminal -v
  - python -m unittest tools.session_coordinator.tests.test_milestone_cli -v
---

# Coordinator01：Deferred Controlled Action 终态轮询

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `ACCEPTED / UPWARD REPLAY GREEN` | 2026-07-15 | `CoordinatorClient.execute_control_action` 在一次 preview/confirm 后，以原 action ID 轮询 detail endpoint，直到 `previewed` / `executing` 转为终态；不再把 `executing + result=null` 当作完成，也不会重复创建 validation action。RED 回归先稳定得到 `expected succeeded, actual executing`，实现后定向 `1/1`、完整 `test_milestone_cli` `5/5` 与补充终态/timeout/malformed 契约 `3/3` 通过。Editor02 M1.3 真实 action `34316e6cd8ce4fb2a9cbc7f9f079f221` 等待约 58 秒后由 CLI 成功返回唯一 job `dabf5b4394d04cf18aa061bb0d7c090c` / run `38ecc821b332447d9512337ed70b796d`；受管验证随后 `24/24`、`exit 0`。本切片已验收，但不宣称 support-slice lifecycle 整体 fixed。 |

## 最低根因

服务端 action 协议允许长操作在 confirm 后保持 `executing`，并在后台 materialize validation copy 后才写入终态 result。Web 控制台已经按 action ID 跟踪该状态，但 Python client 在 confirm 响应后直接返回；CLI 随即要求 result 必须为对象，导致真实 job 已唯一创建并继续运行时，调用方先得到 `invalid_response`。

## 实现边界

- 只创建一次 preview 和一次 confirm；轮询只读取既有 action detail。
- `previewed` 与 `executing` 是非终态，其余枚举状态原样返回给调用方。
- 复用 client 的 command deadline；超时返回带 action ID 和 kind 的 `command_timeout`，不自动重试 mutation。
- 本切片不修改 `milestones.py`、Failure graph 或 Editor 业务文件；node-scoped Failure 过滤仍由同一 Coordinator01 failure lifecycle 处理。

## 验证状态

纯 Python 定向测试和 Editor02 M1.3 真实 upward replay 均已完成。CLI 在同一 action 上等待到 materialize 终态并返回唯一 validation-copy handle，验证 run 自然完成为 `24/24`、`exit 0`。`failure-2026-07-15-support-slice-exact-finalize-plan-output-conflict.md` 仍保持 open，因为 node-scoped Failure gate、共享 owner 原子提交和 lifecycle return 不属于本子切片。
