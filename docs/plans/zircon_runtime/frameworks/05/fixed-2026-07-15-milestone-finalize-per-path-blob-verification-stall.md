---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: milestone-finalize-per-path-blob-verification-stall
origin_plan: docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/05
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_git_finalize.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_git_finalize -v
  - python -m unittest tools.session_coordinator.tests.test_workflow_commit -v
resolved_at: 2026-07-15
---


# Tooling01：milestone finalize 逐路径 blob 校验阻塞大清单提交

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md`
- 来源执行切片：M3 runtime text implementation owner hard-cut
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：原子提交、shared index 隔离与 staged attribution 的唯一 owner 属于 Session Coordinator；Frameworks05 不得绕过协调器或以普通 Git commit 吞入共享工作区内容。

## 失败现象与复现证据

2026-07-15 Render18 的受管 M2 commit action `a551fc74f8a045629b38bfb426a1f2e5` 对约 90 个 manifest 路径执行成功，但从 `2026-07-15T01:53:16.289643Z` 到 `2026-07-15T02:07:08.657054Z` 共耗时 **832.37 秒**。期间 `git_mutex.lock_name=index` 始终由合法 owner 持有，没有外部 Git 子进程死锁；主要成本来自 `GitFinalizeService` 在已经批量完成 dirty scan 后，仍对每个普通文件单独执行一次 `git hash-object --path`，并在 commit 前后对每个路径各执行一次 `git rev-parse --verify :path`。

Frameworks05 M3 必须物理移动约 163 个文本实现文件、删除旧 owner、更新约 84 个 consumer 文件并同步模块文档，预计精确 manifest 超过 250 个路径。继续使用三组 O(N) Git 进程会把 Git mutex 与 validation fingerprint 的暴露窗口扩大到数十分钟，重复此前 `milestone_gate_stale_evidence` / shared-index contention 风险，实际阻断用户要求的“尽快同步到协调器”。

## 最低共享层根因

先前的 line-ending 修复只把 worktree-vs-HEAD dirty 判定收敛为批量 `ls-tree` + `diff --name-only -z`，但 staged attribution 仍保留逐路径 Git subprocess。正确不变量应在一次或少量批量命令中证明：

1. worktree 内容仍等于 Session attribution；
2. scoped `git add` 后 index 与同一 worktree 内容一致，包括 clean/smudged EOL、删除、untracked 与允许 force-add 的路径；
3. validation 后再次验证已批准的 index blob 未漂移，再创建 commit tree；validation 只改 worktree 而未改 index 时，commit 仍使用批准的 staged snapshot，该 worktree-only drift 留在下一次 baseline diff；
4. 任意 mismatch 继续返回 typed finalize error，不能弱化为 best effort。

## 架构修复验收

- 大清单 staged/worktree 验证不再按路径调用 `git hash-object` 或 `git rev-parse :path`，Git 子进程数量按 pathspec chunk 数增长而不是按文件数增长。
- 校验继续正确覆盖 modified、deleted、untracked、ignored force-add、`core.autocrlf=true`、前导空格路径与 UTF-16 pathspec budget。
- attribution 到 scoped stage 期间的 worktree/index 不一致必须 typed reject；validation 期间的 index/staged blob 漂移也必须 typed reject。validation 只改 worktree 而未改 index 时不得污染已批准 commit tree，并必须留在下一次 baseline diff；不得因性能修复放宽 checker-to-commit race。
- 完整 `test_git_finalize` 与 `test_workflow_commit` 通过，独立 review 为 Critical=0 / Important=0；加载新 daemon 后用一个多路径 milestone commit 记录实测时间。

## 禁止临时方案

- 不得绕过协调器、直接操作 shared index、放宽 live lease/attribution/failure/review gate，或用普通 Git commit 提交 Frameworks05。
- 不得假设 LF/CRLF worktree bytes 必须等于 HEAD blob；必须保持 Git filter-aware 语义。
- 不得以仅适用于普通文件名的 newline-delimited `--stdin-paths` 取代 NUL-safe/pathspec-safe 批量路径处理。

## 修复结果与回传

- 根因：GitFinalizeService retained per-path staged blob subprocesses and trusted the post-stage index as its own expected value, leaving both O(N) Windows process cost and a stage-toggle attribution race.
- 架构修复：Batch NUL-safe staged/ignored/HEAD scans by UTF-16 pathspec budget; prove attribution then Git-filter-aware index/worktree equality then attribution again; chunk add/reset; isolate maintenance finalize from degraded baselines and foreign staged index.
- 验证：Fresh test_git_finalize 39/39 in 277.108s; workflow_commit 13/13 in 124.671s; race/chunk/maintenance focused 3/3; py_compile and diff-check pass; independent review Critical=0 Important=0.
- 回传：Frameworks05 may resume large runtime-text hard-cut commits through the coordinator; no compatibility shim or shared-index manual cleanup was introduced.
