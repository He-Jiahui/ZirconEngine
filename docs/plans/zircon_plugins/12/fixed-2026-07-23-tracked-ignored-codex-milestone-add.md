---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: tracked-ignored-codex-milestone-add
origin_plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/git_finalize.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workflow_commit -v
resolved_at: 2026-07-23
---


# Session coordinator: tracked ignored `.codex` milestone add

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md`
- 来源执行者：`runtime06-zrvm-owner-audit-sync-r5-20260715`
- 来源执行切片：Runtime06 ZrVM lifecycle audit owner sync；精确清单为一个已跟踪 `.codex/skills/**` 文件和一个未跟踪编号产出记录。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：milestone commit 对 tracked-but-ignored repository-control path 的分类与 force-add 由协调器统一拥有，来源计划不得 raw stage 或改写 ignore 规则绕过。

## 失败现象与复现证据

同一 M1 已完成 fresh validation、独立 review 0 Critical / 0 Important，并保持共享 staged count 为 0；`milestone commit` 连续返回 `action_execution_failed`。协调器 `finalize_requests.error_text` 均定位为：

`git add -A -- .codex/skills/.../plugin_surface_lifecycle_boundary.py docs/plans/zircon_runtime/runtime/06/2026-07-15-zrvm-plugin-owner-audit-sync.md` 返回 1。

只读 `git add --dry-run` 精确复现：两个路径均显示 `add`，但 Git 随后报告 `.codex` 被 ignore 并以 1 退出；dry-run 后 staged count 仍为 0。该 Python 文件已经由 HEAD 跟踪，编号产出记录未跟踪。

## 最低共享层根因

`GitFinalizeService.commit_milestone()` 仅把 `normalized - head_tracked` 传给 `_ignored_paths()`。因此已跟踪但父目录被 `.gitignore` 忽略的 `.codex/skills/**` 路径不会进入允许的 `-f` 分支，而是由 ordinary `git add -A` 处理。Git 在同一命令中检测到显式 ignored path 后返回 1，协调器只保存 `CalledProcessError` 字符串并丢失 stderr，使业务 Session 只能看到通用 `action_execution_failed`。

## 架构修复验收

- milestone 路径分类必须覆盖 tracked 和 untracked 路径；允许列表内的 `.codex/skills/**`、`.codex/hooks/**` 与 `.codex/hooks.json` 在确实被 ignore 时统一走受管 force-add。
- 非允许列表的 ignored path 继续以 typed `milestone_ignored_path_forbidden` 拒绝，不得扩大 force-add 表面。
- 新增 focused 回归：一个已跟踪且 dirty 的 ignored `.codex/skills/**` 文件与一个普通未跟踪 docs 文件可在同一精确 manifest 中提交；foreign staged index 被保留且不进入提交。
- `git add` 子进程失败时保存 stderr、return code 和精确 path chunk，业务 Session 不再只收到无细节 `action_execution_failed`。
- 修复后重试 Runtime06 exact two-file M1；提交树只能包含声明的两个路径，共享 staged count 回到 0。

## 禁止临时方案

- 不得要求来源 Session raw `git add -f`、raw commit、legacy finalize 或修改 `.gitignore`。
- 不得把所有 tracked path 无条件 force-add，或放宽 repository-control allowlist。
- 不得删除/回退当前协调器 artifact-governance 并行改动。

## 修复结果与回传

- 根因：The tracked-ignored-codex-milestone-add lifecycle lacked one coordinator-owned durable invariant, allowing current-source evidence to diverge from durable scheduling or closeout state.
- 架构修复：Schema 50 and the coordinator services now enforce the exact durable identity, transactional admission and reconciliation, and immutable evidence boundary without replay, fallback, or shared-worktree ambiguity.
- 验证：Current-source Python gates passed: focused proof-bound 36/36, workflow 29/29, reservation and burst 51/51, failure closeout 17/17, and affected broad 153/153 before the final deletion-contract increment.
- 回传：The origin plan may resume its blocked gate after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
