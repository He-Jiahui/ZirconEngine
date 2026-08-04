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

Open state: `Coordinator01 implementation integrated / local topology gate GREEN / managed and Runtime11 origin replay pending`; no `fixed` return is claimed.

- `TopologyParser` 仅忽略“最近更高层级、同 milestone ID”的 `测试阶段` / `Testing stage` 子标题；同级或独立的 `Testing stage rollout` 仍保留为真实节点，重复节点校验未被放宽。
- 当前组合回归 `test_workflow_topology_testing_stages + test_workflow_topology` 为 18/18 通过；旧记录中的外部 content-version 断言失败已经前向收敛。
- 当前源码直接解析真实 Runtime11 计划得到 `M0`、`M1`、`M2`、`M3` 各一个节点。原始 Runtime11 Session 仍须通过 coordinator wakeup 重新执行 `milestone prepare ... --milestone M2`；本修复 Session 不代替 origin Session 创建 workflow run。

## 状态与完成项目

| 日期 | 切片 | 状态 | 完成项目与证据 |
|---|---|---|---|
| 2026-08-03 | Coordinator01 nested testing-stage hard cut | `implementation_integrated` | 生产 parser 与中英文嵌套测试阶段回归均已进入当前源码；完整 topology 组合门 18/18 通过，真实 Runtime11 计划只生成四个唯一 milestone。待 immutable managed validation 与 Runtime11 原始 `milestone prepare` 复放；未声明 fixed、Cargo pass 或 origin closeout。 |
