---
handoff_kind: fixed
status: fixed
created_at: 2026-07-16
summary_slug: native-slice-closeout-checker-staged-index-contract-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_workflow_node: M1.3
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - .codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.ps1
  - .codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.Tests.ps1
  - .codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/read-closeout-evidence.py
tests:
  - Invoke-Pester -Script .codex/skills/zircon-project-skills/close-session-goal-milestones/scripts/check-closeout.Tests.ps1 -PassThru
  - .\.codex\skills\zircon-project-skills\close-session-goal-milestones\scripts\check-closeout.ps1 -RepoRoot . -Mode Milestone -SessionId editor02-m1-3-inspection-hardening-20260715 -CommitMessage "fix(editor): hard-cut inspection hierarchy snapshot contracts" -ManifestPath .\.codex\state\session-coordinator\editor02-m1-3-inspection-hardening-closeout-manifest.json
resolved_at: 2026-08-04
---


# Session Coordinator 01：原生 slice closeout checker 仍依赖共享暂存区

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / CHECKER CONTRACT DRIFT` | 2026-07-16 | Editor02 `M1.3` 六文件 current-hash manifest 已由 schema 36 coordinator prepare 绑定，`commit_manifest`、`failure_audit`、`plan_output` 均 accepted，共享 Git index 为 0。按 closeout skill 对同一六文件分类 manifest 执行只读 checker，仍返回 `milestone_id_invalid`、`plan_evidence_missing`、`staged_scope_mismatch`、`empty_commit_scope` 等错误：checker 只接受 `M<number>`，并要求调用前把计划记录和业务文件放入共享 index；这与原生 `M1.3` node、禁止手工 staging、服务在 Git mutex 内原子暂存的当前合同冲突。未手工 `git add`、未修改其他会话暂存区，Editor02 acceptance 保持未完成。 |
| `IMPLEMENTED / LOCAL GREEN / MANAGED ACCEPTANCE PENDING` | 2026-08-03 | 已 hard-cut 删除调用方预暂存兼容路径：checker 先拒绝任何非空共享 index，计划证据、tracked deletion、untracked 分类和敏感内容均读取当前工作树；evidence reader 只暴露 coordinator attribution SHA256 与 current worktree SHA256，删除 `staged_hashes` 旧合同。TDD RED 为 24/30、六个预期失败；初次实现 30/30 GREEN 后，独立复审发现 `ACMRD` filter 会遗漏 type-change/unmerged index，已移除 filter 并增加真实 mode-120000 type-change 与 merge-conflict stage 1/2/3 回归。最终完整 Pester 32/32 通过（339.81s），复审 C0/I0/M0，Python 编译、PowerShell parser 与 `git diff --check` 通过。当前待 managed validation receipt 与原 Editor02 六文件复放；未宣称 fixed/accepted。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：`M1.3`
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：业务代码、测试、文档与 Failure return 已形成合法六文件 node manifest；最低共享失败位于 Coordinator01 的 closeout preflight checker，而非 Editor02 inspection 实现。

## 失败现象与复现证据

schema 36 instance `e3a97c6e45114976a5175fd4329fc11a` 下，Editor02 run
`ac800d3e33174e38a77ba5da7a8250f2` 对 `M1.3` prepare 后三项前置门均为
accepted。精确分类 manifest 包含 `snapshot.rs`、`tests.rs`、inspection 模块文档、
M1.3 产出记录和两份 fixed handoff，共 6 个路径；Git index 为 0。

对该 manifest 执行本文件 frontmatter 中的 checker 命令，得到：

- `milestone_id_invalid`：只允许 `M2` 一类完整 milestone，拒绝合法 native slice `M1.3`；
- `plan_evidence_missing` / `untracked_category_mismatch`：要求新产出记录已经进入共享 Git index；
- `staged_scope_mismatch` / `empty_commit_scope`：把协调器要求的 pre-action staged=0 误判为失败；
- `manifest_path_not_owned` / `staged_content_not_attributed`：以 staged blob 而非 current-hash attribution 判断 tracked source。

这些错误发生在 managed validation 已启动后，未修改六文件业务内容；当前验证结果不能替代
checker `status: ok`，也不能通过手工 staging 绕过。

## 最低共享层根因

`check-closeout.ps1` 仍实现旧的“调用者预先暂存完整 milestone”合同：milestone ID 解析只接受
`M<number>`，完成证据和 ownership 均从 Git index 推导。当前 coordinator 已把 node identity、
current-hash attribution、validation/review gates 和 scoped staging 固定在服务端；调用前 index 必须为空，
并允许 `M<number>.<positive-number>` 原生 slice。checker 与 authoritative finalizer 已发生双合同漂移。

## 架构修复验收

- checker 接受机器拓扑中存在的 native slice ID `M<number>.<positive-number>`，并校验精确 node identity；不得把 slice 冒充父 milestone accepted。
- preflight 从 coordinator 当前 Session attribution、node manifest、validation/review/failure gates 读取证据；调用前要求 staged path 为 0，而不是要求业务路径已暂存。
- `untracked` 分类按 worktree/HEAD 判断，不以 staged-add 判断；所有分类路径必须精确等于 node manifest，且 current hash 与 attribution 一致。
- checker 不自行实现第二套 milestone completion 聚合；父 milestone/sibling slice pending 时，合法 child slice 可返回 `status: ok`，父节点仍保持 pending。
- 以 Editor02 当前六文件 `M1.3` 复放，checker 返回 `status: ok`、staged paths 仍为 0；随后 canonical managed validation、独立 review、milestone commit 可原子完成。
- 增加 Pester 回归覆盖 native slice、完整 milestone、空 index、foreign staged path、stale attribution 和 untracked 分类；不得放宽 foreign staged isolation。

## 禁止临时方案

- 不得手工 `git add`、伪造 index、修改 manifest 为父 `M1`，或跳过 closeout checker。
- 不得在 Editor02 特判 `M1.3`、删除三份计划记录，或把 checker 错误标为业务 Failure。
- 不得恢复 legacy `finalize --milestone`、hook bypass 或客户端 staging。

## 修复结果与回传

- 根因：The closeout checker still enforced the retired caller-prestaged milestone contract: it rejected native slice IDs, derived evidence from the shared Git index, and disagreed with coordinator-owned current-hash attribution and atomic finalization.
- 架构修复：Hard-cut closeout preflight to accept native M<number>.<positive> slices, require an empty shared index, read plan evidence and ownership from current worktree/current-hash attribution, classify deletions and untracked paths against worktree/HEAD, preserve exact node manifests, and reject type-change or unmerged index state.
- 验证：Local Pester passed 32/32 in 252.11s; managed validation ticket e5eff23a152b495fbc885e005704924a passed against source manifest d58fb43ca9c9e83c9d9de1dacf3ee9fac52c6e5e038280a7d81e52559c016b44; Python compile, PowerShell parser, git diff check, and handoff validation (561 artifacts, 0 errors) passed. The archived Editor02 source session has no current attributions and all six preserved manifest paths are clean at HEAD, so no live replay was fabricated; managed fixtures replay the native M1.3 empty-index and exact-manifest contracts.
- 回传：Coordinator01 now validates native slice closeout from current worktree evidence with an empty shared index; the original Editor02 six-path manifest remains historical evidence, and the failure is returned as fixed after source-bound managed acceptance.
