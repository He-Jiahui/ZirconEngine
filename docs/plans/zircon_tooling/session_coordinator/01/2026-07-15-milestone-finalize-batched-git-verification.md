---
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize.py
plan_sources:
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
tests:
  - python -m unittest tools.session_coordinator.tests.test_git_finalize -v
  - python -m unittest tools.session_coordinator.tests.test_workflow_commit -v
  - python -m py_compile tools/session_coordinator/git_finalize.py tools/session_coordinator/tests/test_git_finalize.py
---

# Milestone finalize 批量 Git 校验收束

## 状态与产出记录

| 计划 | 里程碑 | 状态 | 完成日期 |
|---|---|---|---|
| Tooling Session Coordinator 01 | M4.4 failure correction：原子 finalize | `completed` | 2026-07-15 |

## 已完成

- staged blob 读取从逐路径 `rev-parse :path` 收敛为 NUL-safe、UTF-16 argv budget 感知的批量 `ls-files --stage -z`。
- ignored、HEAD tracked 与 dirty scan 同步批量化，Git 子进程数量按 pathspec chunk 数增长，不按 manifest 文件数增长。
- scoped stage 后以 attribution → Git-filter-aware index/worktree compare → attribution 的顺序关闭 stage 注入竞态；validation 后继续比较批准的 staged blob。worktree-only validation drift 不进入 commit tree，并留在下一 baseline diff。
- `git add` 与提交后 `git reset` 都按 UTF-16 argv budget 分块；maintenance finalize 从 HEAD 构造隔离 index，在 degraded baseline 下仍保留共享 index 的外部 staged 状态。
- `.codex/skills`、`.codex/hooks` 与 `.codex/hooks.json` 的受管 force-add 资格保持显式白名单；其他 ignored 路径仍拒绝。

## Fresh Testing Evidence

- `test_git_finalize`：39/39 通过；fresh 修复后复跑用时 277.108 秒，exit 0。
- `test_workflow_commit`：13/13 通过；fresh 修复后复跑用时 124.671 秒，exit 0。
- stage injection、end-to-end add/reset 分块、maintenance degraded+foreign-staged 隔离三项 focused regression：3/3 通过，42.882 秒。
- 320 路径 focused regression：1/1 通过，用时 7.885 秒，并断言 staged scan 的 `ls-files --stage` 调用不超过 2 次。
- `py_compile` 与定向 `git diff --check` 通过。
- 新协调器 daemon instance `dfda8581db4e4f7a8dd2fb6db9860def` 已重载当前实现并报告 healthy。

## Review 与回传

独立 fresh re-review 为 Critical=0 / Important=0；首轮发现的 stage-toggle 与未分块 add/reset 两项 Important 均已修复并独立复现通过。Failure return 在受管 coordinator maintenance commit 前完成；最终 commit SHA 由协调器记录持有。Tooling01 M4 已有历史 accepted manifest，本次 correction 不覆盖其历史 `Files`；该切片只修复提交基础设施，不宣称 Frameworks05 的 runtime text 物理 hard-cut 已完成。
