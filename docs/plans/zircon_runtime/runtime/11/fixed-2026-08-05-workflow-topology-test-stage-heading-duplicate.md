---
handoff_kind: fixed
status: fixed
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
resolved_at: 2026-08-05
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

- 根因：Fallback workflow topology parsing treated every level-2 through level-6 M<n> heading as a milestone, so a nested 测试阶段 / Testing stage heading duplicated the enclosing milestone ID before graph validation.
- 架构修复：Ignore a testing-stage heading only when its nearest higher-level milestone candidate has the same ID. Preserve standalone testing-stage rollout headings and the existing rejection of genuine duplicate milestone IDs.
- 验证：Local combined topology regression: 18/18 passed in 19.943s. Current-source Runtime11 parse: headings source with exactly M0, M1, M2, M3. Coordinator-managed ticket 88f0d976a8724b578a487a3bb97a462d, request 38672d72e802479ca359f822d2fa0ad5, copy 11accda2934d432a997039d7851ba539: 18/18 passed in 15.360s, exit 0, copy removed. Runtime11 replay action 42a3c5db474d48ea972d839cef954054 reached milestone_manifest_record_ambiguous for M2, proving the duplicate-node parser gate passed. Handoff validator: 561 artifacts, 0 errors.
- 回传：The original Runtime11 native-plugin session is archived. The current-source replay passed topology parsing and stopped at the legitimate child-record manifest selection gate; Runtime11 must resume through a current-source session, and no Cargo result or synthetic M2 output record is claimed.
