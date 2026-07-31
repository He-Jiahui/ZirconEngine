---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: workflow-topology-test-stage-heading-duplicate
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/runtime/11
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workflows/topology.py
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
tests:
  - .\tools\zircon-session.ps1 milestone prepare --session-id runtime11-native-plugin-refresh-contract-r1-20260729 --milestone M2
---

# Session Coordinator: Workflow Topology Test-Stage Heading Duplicate

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 来源执行切片：Runtime11 M2 testing stage, native-plugin discovery bounded refresh publication.
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：所有受管 `milestone prepare` 都先经过 Session Coordinator 的 workflow topology parser；该 parser 错误地把测试阶段标题识别为第二个 milestone，属于工具层公共根因。

## 失败现象与复现证据

2026-07-29 Windows PowerShell 执行：

```powershell
.\tools\zircon-session.ps1 milestone prepare --session-id runtime11-native-plugin-refresh-contract-r1-20260729 --milestone M2
```

命令未启动 Cargo，终端结果为 `Duplicate workflow node ID: M0`。Runtime11 计划的人类可读结构同时包含 `### M0 ...` 与 `#### M0 测试阶段（milestone-first）`；`tools/session_coordinator/workflows/topology.py` 的 `_PLAIN_NUMBERED_MILESTONE` 以任意二至六级 `M<n>` 标题作为 milestone，因此将测试阶段也加入拓扑并在 `_validate_graph` 中拒绝重复 ID。

这不是 Rust contract RED，也不是有效的验证失败。它阻止当前 Runtime11 native refresh contract、Runtime11 asset worker，以及采用相同测试阶段标题模式的计划创建冻结验证快照。

## 最低共享层根因

Coordinator topology parsing does not distinguish an implementation milestone heading from its `测试阶段` / `testing stage` child heading. The duplicate-ID check correctly rejects duplicated nodes after the parser has constructed the wrong graph; suppressing that check or changing individual plan headings would hide the shared parsing defect.

## 架构修复验收

- `tools/session_coordinator/workflows/topology.py` parses a plan containing `### M0 ...` and `#### M0 测试阶段（milestone-first）` as exactly one `M0` milestone.
- Coordinator-focused tests cover Chinese and English testing-stage headings without weakening detection of genuinely duplicated milestone IDs.
- The recorded Runtime11 `milestone prepare ... --milestone M2` reproduction creates a source-bound run instead of returning `Duplicate workflow node ID: M0`.
- Runtime11 then reruns its managed focused validation through the returned run ID; no current native refresh result is claimed before that gate completes.

## 禁止临时方案

- Do not rewrite existing plan definitions solely to satisfy the parser.
- Do not suppress duplicate-node validation globally or accept malformed duplicate milestones.
- Do not generate a run ID manually, bypass `milestone prepare`, or run unmanaged Cargo.
- Do not change Runtime11 native-plugin source or tests as a workaround for this coordinator failure.

## 修复结果与回传

源级修复已完成（2026-07-29）：

- `TopologyParser` 仅忽略“最近更高层级、同 milestone ID”的 `测试阶段` / `Testing stage` 子标题；同级或独立的 `Testing stage rollout` 仍保留为真实节点，重复节点校验未被放宽。
- 定向编号标题回归：3 passed。覆盖中文和英文嵌套测试阶段、过滤后的依赖区间保留、独立 `Testing stage rollout` 节点，以及真实重复 ID 拒绝。
- 本地解析真实 Runtime11 计划得到 `M0`、`M1`、`M2`、`M3` 各一个节点；独立复审为 Critical 0 / Important 0 / Minor 0。
- 独立 `tools.session_coordinator.tests.test_workflow_topology_testing_stages` 为 3 passed。既有 `tools.session_coordinator.tests.test_workflow_topology` 为 14 passed / 1 failed；唯一失败是他会话已有的 `test_content_only_change_updates_metadata_without_splitting_topology_identity` 版本语义断言，本次未修改其实现或期望。
- 修复执行会话已由错误地将三个路径拼接为单一范围的 `runtime11-workflow-topology-fix-r2-20260729` 迁移到 `runtime11-workflow-topology-fix-r3-20260729`；r3 对 parser、隔离回归测试和本记录持有三个精确租约，r2 在无租约、无源码变更时取消。

尚未 fixed：生产 Coordinator daemon 在本修复前已加载旧解析器，因此原始 `milestone prepare` 仍返回 `Duplicate workflow node ID: M0`。该服务只可在所有受管 Cargo 进程树结束后由 maintainer 发起受控 `service.rollover`；不得为此重启或中断活跃任务。滚动完成后，Runtime11 必须重新执行本文记录的 `milestone prepare ... --milestone M2`，取得 source-bound run 后才可开始 native refresh 的受管 Cargo 验证并将本 artifact 移回 origin 为 `fixed-*`。
