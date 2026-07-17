---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: post-commit-baseline-archive-latency
origin_plan: docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/render/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/tests/test_baselines.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_baselines.BaselineTests.test_accept_commit_updates_from_changed_git_paths_without_full_archive
resolved_at: 2026-07-17
---


# Coordinator01: small managed commits rebuild the entire baseline archive

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md`
- 来源执行切片：Render01 current-source managed coordinator finalization
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Render01 managed commits were accepted correctly, but post-commit
  baseline capture held the shared Git mutex for minutes after a small exact
  manifest had already committed.

## 失败现象与复现证据

The committed Render01 manifest was small and exact, yet the finalizer client
waited while `BaselineService.accept_commit` streamed `git archive` for the
whole HEAD and hashed every tracked file. The coordinator was not waiting on
Cargo or source validation; it was rebuilding a baseline after the commit while
the shared Git mutex remained owned.

## 最低共享层根因

`accept_commit` requires a complete commit-derived manifest so unrelated shared
files committed at the same HEAD never become false dirty-worktree changes. It
implemented that invariant by unconditionally calling `_commit_manifest`, which
archives and hashes every tracked file even when a prior complete baseline and
the exact Git tree delta are available.

## 架构修复验收

- Transform the prior commit manifest from the pinned old-to-new Git tree delta:
  retain unchanged tracked hashes, remove deleted paths, and hash only added or
  modified new-commit blobs with the same Git filter semantics.
- Preserve prior accepted untracked entries and retain the existing full archive
  fallback when the prior baseline is incomplete or cannot safely be advanced.
- Keep the resulting baseline complete for the new HEAD; a managed slice must
  never hide concurrent committed paths outside its manifest.
- Add a focused regression proving `accept_commit` advances a changed, added,
  and deleted tracked path without invoking the full archive builder.

## 禁止临时方案

- Do not derive the baseline from live checkout bytes.
- Do not omit foreign committed paths or weaken content-hash comparison.
- Do not defer the update asynchronously while claiming the new HEAD baseline
  is healthy.

## 修复结果与回传

- 根因：BaselineService.accept_commit rebuilt the entire pinned HEAD archive after every managed commit, holding the shared Git mutex although the prior complete baseline and exact Git tree delta were available.
- 架构修复：New HEAD baselines retain unchanged tracked hashes, apply the old-to-new Git delta, hash only changed commit blobs through Git filters, and preserve accepted untracked entries. Same-HEAD repair, incomplete historical baselines, and any .gitattributes change retain the strict full-archive fallback.
- 验证：BaselineTests passed 14/14, including changed/added/deleted incremental advance without archive and the .gitattributes full-rebuild guard. git diff --check is clean for the owned implementation paths.
- 回传：The Render01 origin holds only its generated fixed child-record lease. No live workspace byte is accepted as baseline content, and no Cargo/reservation/foreign index state was changed.
