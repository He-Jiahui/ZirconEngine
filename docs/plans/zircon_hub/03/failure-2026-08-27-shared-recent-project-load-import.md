---
handoff_kind: failure
status: open
created_at: 2026-08-27
summary_slug: shared-recent-project-load-import
origin_plan: docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
fixing_plan: docs/plans/zircon_hub/03-project-lifecycle-robustness.md
origin_child_dir: docs/plans/optimize/zircon_tooling/06
fixing_child_dir: docs/plans/zircon_hub/03
plan_link_mode: child_record_only
related_code:
  - zircon_hub/src/tauri_app/runtime_state/tests.rs
tests:
  - cargo test -p zircon_hub --lib focus_refresh_reconciles_pending_hub_recents_with_editor_registry --locked
---

# Hub03: shared recent-project loader test import and identity drift

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md`
- 来源执行切片：Tooling06 managed Cargo process-tree lifecycle current-source Hub library replay
- 修复责任计划：`docs/plans/zircon_hub/03-project-lifecycle-robustness.md`
- 交接原因：受管 Hub library gate 已越过 Coordinator process-tree lifecycle 与 lockfile admission，最低错误位于 Hub03 runtime-state inline test 的 projects API import 边界。

## 失败现象与复现证据

2026-08-27 受管命令 `cargo test -p zircon_hub --locked --verbose --lib` 真实启动 Cargo，并在编译 `zircon_hub` lib tests 时以 exit 101 终止。`zircon_hub/src/tauri_app/runtime_state/tests.rs:158` 调用 `load_shared_recent_projects`，但文件顶部的 `crate::projects` use 组只导入 `project_metadata_key`、`reconcile_shared_recent_projects` 与 `RecentProject`，因此 rustc 报 E0425。补齐 import 后，受管 focused test 已实际启动并暴露第二层 RED：测试用完整 `RecentProject` 相等性检查成员，而 reconcile 会按共享 registry 顺序 authority 推进 Hub 新记录的 `last_opened_unix_ms`。panic fixture 中 `hub.toml` 与 `recent_projects.json` 均同时含 HubGame 与 EditorGame，失败不是条目丢失。

## 最低共享层根因

Hub03 的 focus-refresh regression 使用公开 projects re-export 回读共享 recent-project registry，但测试模块未把该函数导入局部作用域；导入修复后，测试又把可由 registry 单调推进的时间戳误当成项目身份。生产身份 authority 已是 `project_metadata_key(path)`，测试应按同一 authority 验证 Hub/Editor 两个路径都存在，而不是锁定完整 DTO 的瞬时时间戳。生产 API、reconcile 行为和文件格式均不需要改变。

## 架构修复验收

- 测试模块从 `crate::projects` 的既有公开 re-export 导入 `load_shared_recent_projects`，不新增旁路 helper 或重复文件解析。
- 内存 config 与磁盘 shared registry 均使用既有 `project_metadata_key` 断言 HubGame/EditorGame 路径身份存在；不得删除任一侧断言，也不得锁死 registry 可推进的 recent timestamp。
- 受管 focused lib test 实际执行并通过，随后 Hub library gate 至少越过该 E0425；若出现新的外部错误，单独路由且不吸收到本节点。
- scoped rustfmt 与 `git diff --check` 通过。

## 禁止临时方案

- 不得删除或跳过 shared registry 回读断言。
- 不得在测试内重复实现 JSON loader、扩大生产可见性或改变 recent-project 持久化格式。
- 不得把 Tooling06 进程树修复或其他 Hub failure 合并进本单文件修复。

## 修复结果与回传

Open state: `managed compile RED and focused assertion RED captured; import plus identity-authority repair implemented; focused managed validation and fixed return pending`.

### 2026-09-19 rolling successor source reconciliation

- Successor Session `failure-roll-01a084c8-hub03-shared-recent-r1` reclaimed the
  current Hub runtime-state test and this failure record at baseline epoch
  `611`. Ownership transfer fingerprint:
  `c813c3ebd5c146faf8e8009fade4070ccc37f8338a7daa572f2bcb0921d8f668`.
- The current test keeps the required `load_shared_recent_projects` import and
  validates both in-memory and shared-registry recents by the existing
  `project_metadata_key(normalize_project_root(...))` authority. Later
  unrelated publication-epoch tests are present in the working file and are
  preserved; this slice does not revert or absorb them.
- The original focused Hub Cargo gate remains pending a fresh managed run. The
  external `E:\\Git\\zr_vm` clean-worktree prerequisite, independent review,
  canonical return, and closeout also remain pending.

### 2026-09-19 current-source static validation

- Static request `failure-roll-01a084c8-hub03-shared-recent-20260919-r1`
  admitted ticket `b0f5b877f9364d029ba9052c4b29d20b` with sealed manifest
  `071c77b80193217830403a66da45294158153fd7caf1a2ca7223e533b5e54651`.
  The checker verifies the existing public loader import, identity-key-based
  assertions for both Hub and Editor entries, and absence of timestamp locking.
  Status is `queued` pending terminal evidence.
- This is static-only. Fresh managed Hub Cargo, review binding, canonical return,
  closeout, and clean `E:\\Git\\zr_vm` remain pending.

### 2026-09-19 static terminal result

- Ticket `b0f5b877f9364d029ba9052c4b29d20b` passed at
  `2026-09-19T07:30:00.748602Z` in job
  `15f0db27a49548d5adc7cfda982c6cbb`, exit code `0`, with marker
  `HUB03_SHARED_RECENT_PROJECT_IDENTITY_CURRENT_SOURCE_CONTRACT_PASS`.
  Cleanup event `10932` completed.
- This confirms the current source-contract region only. Fresh managed Hub
  Cargo, independent review binding, fixed return, closeout, and clean
  `E:\\Git\\zr_vm` remain pending; the failure stays open.

### 2026-09-20 independent source review r1

The independent reviewer inspected the focused refresh regression and its direct
shared-registry helpers.  The test imports the existing public
`load_shared_recent_projects` function, uses `normalize_project_root` plus
`project_metadata_key` for both HubGame and EditorGame identity assertions, and
retains the in-memory and on-disk checks.  It does not compare mutable
`last_opened_unix_ms` values, rewrite the shared registry during refresh, add a
parallel parser, or alter production reconciliation behavior.

Read-only checks:

- `rustfmt +1.94.1 --edition 2021 --config skip_children=true --check` on
  `zircon_hub/src/tauri_app/runtime_state/tests.rs`:
  `HUB03_RUSTFMT_PASS`.
- `git diff --check` on the owned test path: pass.

Current source hash inspected:

```text
zircon_hub/src/tauri_app/runtime_state/tests.rs fcb6afc1ef18c04d9a804e1e761294c525b24cefc7610bbe5e1a02d43ceea200
```

Independent review result: `Critical=0 Important=0 Moderate=0`.  No managed
Hub Cargo, UI/product result, canonical fixed return, or closeout is inferred.
