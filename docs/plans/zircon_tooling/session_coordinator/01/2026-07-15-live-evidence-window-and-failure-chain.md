---
record_kind: implementation_slice
status: implemented_pending_service_reload
created_at: 2026-07-15
plan_source: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
related_code:
  - tools/session_coordinator/codex_sync/evidence.py
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/failures.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py
tests:
  - tools/session_coordinator/tests/test_codex_evidence_projection.py
  - tools/session_coordinator/tests/test_failures.py
  - .codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/test_validate_plan_failure_handoffs.py
---

# Coordinator01：实时证据窗口与开放 Failure 链投影

## 目标

把 `C:\Users\HeJiahui\.codex\sessions\YYYY\MM\zircon-engine-evidence-live-YYYY-MM-DD.md`
收敛为实时协作视图。它只服务于当前会话调度，不承担历史审计、计划产出或完整日志保存职责。

## 已实现

- Codex rollout 会话仅保留最近四小时内仍位于 active source 的记录，最多 50 条。
- 协调器会话必须先满足同一四小时 heartbeat 窗口，之后才保留 `active`、
  `resolving_failure`、等待租约/验证的状态或窗口内 `registered` 会话；stale、
  abandoned-active、completed、cancelled、archived 不再污染实时列表。
- Cargo 继续只显示 `leased` 与 `running`。
- 受控动作仅显示正在执行或同一窗口内的终态动作；预览与过期预览不构成实时证据。
- 新增按 priority 排序的 open `failure-*` 链摘要，包含稳定 slug、来源/修复计划和相对记录路径。
- 隐私边界不变：不写入提示词、命令行、日志正文、CWD、绝对路径或 webhook。

## 原因与边界

旧投影无条件选择 200 条会话和 40 条动作，实际文件约 45KB，绝大多数是 stale 或历史
记录，且 failure 链没有单独入口。问题位于投影查询而非 Hook；Hook 和 worker 仍负责
触发、发现、数据库提交和原子替换。完整的 milestone、failure/fixed 文件和验证日志仍由其
编号子计划与协调器数据库保存，不能被这份实时视图取代。

## Failure 分类边界修复

`2026-07-15-live-evidence-window-and-failure-chain.md` 曾因旧的 date-first
候选规则“文件名包含 `failure`”被当作 handoff；它不是 handoff，而是本编号子计划中的
普通 `implementation_slice` 输出记录。该误分类会产生 17 条缺少 handoff frontmatter 和
交接段落的无关诊断，进而阻塞不相关的 Render18 `failure return`。

修复收紧两个必须一致的入口：协调器 immutable-action 的
`failure_artifact_snapshot` 与 skill validator 只把 canonical
`failure-{date}-{summary}.md` / `fixed-{date}-{summary}.md`，以及尾部明确为
`-failure-handoff.md` / `-fixed-handoff.md` 的 legacy date-first 名称视为 handoff。
普通日期产出记录即使标题包含 Failure 也不会进入图。回归先证明旧匹配错误收录该文件，随后
验证外部 handoff validator 17/17、协调器 Failure 图 12/12 通过，并在当前工作树实证该
文件不在 immutable snapshot 中。

## 验证

新增 RED/GREEN 回归：旧实现会错误输出 stale Codex/协调器会话，并缺少 Failure 章节；修改后
`CodexEvidenceProjectionTests` 3/3 通过，覆盖实时筛选（包括过期 heartbeat 但 status 仍为
`active` 的记录）、隐私脱敏、原子投影调用顺序与开放 Failure 链呈现。

全仓 `audit_plan_output_records.py` 仍报告 4 个既有的 Editor UI 计划 notice 违规
（`editor_ui/01`、`10`、`11` 和 `index.md`）；它们不涉及本记录、没有由本切片修改，也不以
放宽审计规则掩盖。

## 服务加载状态

写入时 Frameworks05、独立会话和 Shader06 M1 仍各有受管 Cargo/GPU 作业。根据共享调度规则，
本切片不会重启、释放或中断它们。待所有运行中的 Cargo/validation 自然结束后，才通过
`service.drain` → controlled `service.restart` 加载该投影逻辑；重新生成的 evidence 文件必须
确认不含 stale 标识并包含 `## 开放 Failure`。Failure 分类修复同样待该安全窗口加载；它不会
通过改写普通 evidence record 的内容来绕过 handoff 规则。
