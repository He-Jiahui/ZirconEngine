---
record_kind: implementation_slice
status: implemented_pending_service_reload
created_at: 2026-07-15
plan_source: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
related_code:
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/workflows/plan_import.py
tests:
  - tools/session_coordinator/tests/test_workflow_commit.py
---

# Coordinator01：当前源码 Milestone Manifest 选择

## 触发与最低根因

Shader06 当前源码 M1 attestation run
`7a5653d606a64662b2618a1662968e3a` 在 coordinator validation 阶段被
`milestone_manifest_record_ambiguous` 拒绝。编号子计划目录合法地同时保留：

- 历史已验收记录 `2026-07-14-current-source-hdri-pbr-acceptance.md`；
- 新的单文件当前源码证明 `2026-07-15-m1-current-source-attestation.md`。

旧 `_derive_milestone_paths` 在读取任何 Session 归属信息前，要求同一
`Plan + Milestone` 的 child record 全局唯一。因此它把不可变历史证据当作新 attestation 的
冲突，错误地要求业务会话重写、移动或伪造历史记录。

## 架构修复

当目录中有多份同一 milestone record 时，协调器现在只选择同时满足以下条件的唯一记录：

1. 路径相对 `HEAD` 仍为 dirty；
2. 路径已被执行该 manifest bind 的 Session attribution 覆盖。

选择后，其 `Files` 声明仍必须全部在同一 Session 的当前 attributed changes 内。零份或多份
当前归属候选继续以 `milestone_manifest_record_ambiguous` 拒绝，并返回所有候选及
`attributedRecords`，不允许任意默认挑选历史记录。

该规则只解决 coordinator 的 current-source record 选择；不导入历史 M1、不会修改 Shader06
文档、不会绕过独立 review/validation/commit gate，也不会放宽全局 plan-output 审计。

## 验证与加载条件

新增最小 RED/GREEN 回归先复现“历史 M1 record + 当前 attributed attestation”被误判歧义，随后
确认只返回当前 source 文件和新 attestation record。核心定向组 `3/3` 通过，覆盖该新选择场景、
既有的 controlled review/gate refresh，以及受管 manifest 最终提交路径。完整
`test_workflow_commit` 组在共享主机负载下超过 60 秒工具上限，未获得失败结论，保留为无受管
Cargo 的后续窗口复验。

Shader06 还报告 handoff validator 为 151 artifacts / 0 errors；全局
`audit_plan_output_records.py` 的 4 项失败全部是既有 Editor UI `01`、`10`、`11` 与 `index.md`
archive-notice 问题，不属于本次选择规则且不会被屏蔽。当前 Frameworks05 仍有受管 Cargo，故此
修复与先前 Failure 分类修复都必须等其自然结束、且 Shader06 M1 受管操作完成后，才通过一次
controlled drain/restart 加载。
