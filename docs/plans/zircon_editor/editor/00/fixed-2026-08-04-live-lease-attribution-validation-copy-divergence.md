---
handoff_kind: fixed
status: fixed
created_at: 2026-07-26
summary_slug: live-lease-attribution-validation-copy-divergence
origin_plan: docs/plans/zircon_editor/editor/00-editor-architecture-overview.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/00
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/tests/test_server.py
  - tools/session_coordinator/tests/test_live_lease_validation_copy.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_live_lease_validation_copy
  - exact untracked Session source file lease -> baseline attribute -> validation-copy materialize-cargo
  - lease claim durability and heartbeat regression for untracked Rust sources
  - validation-copy source manifest hash equals current attributed overlay hash
resolved_at: 2026-08-04
---


# Coordinator01: live lease attribution validation-copy divergence

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/00-editor-architecture-overview.md`
- 来源执行切片：root facade atomic integration validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：lease、baseline attribution 和 validation-copy 都是 Coordinator01 的服务端控制面。Editor00 可以领取精确路径并冻结 snapshot，但不能手写 SQLite attribution、复制共享工作树或修改 coordinator 的 ownership 判定来绕过不一致。

## 失败现象与复现证据

Editor00 session `editor00-core-facade-integration-r1-20260726` 已冻结 snapshot `1089`，其中含 `core/mod.rs`、Context wiring 与五个新增 core 子树的 29 个 Rust 文件，当前 SHA 复核为 `0` drift。

1. 对这 29 个未跟踪 Rust 文件执行 `lease claim` 返回 `acquired: true`、`conflicts: []` 和完整路径列表。
2. 使用固定外部 `zr_vm` descriptor（commit `503fb72163cd20ddf32a38f8a330083712f5d648`，binding 与 sys roots）执行 `validation-copy materialize-cargo`，job `d1c2fb6f7b2f4829a0a9728cad49d48d` 在 `baseline_archive` 以 `validation_copy_unowned_path` 终止，首个路径为 `zircon_editor/src/core/notifications/decision/center.rs`。
3. 随后 `baseline attribute --session-id editor00-core-facade-integration-r1-20260726 <same 29 paths>` 返回 `baseline_lease_missing`，列出同一批 29 个路径。
4. 对协调器 `leases` 表的只读查询显示该 session 的 `zircon_editor/src/core/%` live rows 为 `0`；同一会话稍后单独领取的 failure 文档 lease 存在且可 heartbeat。因此不是 session 不存在、snapshot 漂移、外部依赖未固定或普通失效超时。

`workspace_copy.py` 当前只从 `attributions` 读取 overlay 所有权；`baselines.py::attribute` 又要求 live lease。因此 lease API 的成功响应若未持久化源码路径，就会令 untracked source 永远无法进入 immutable validation copy。

## 最低共享层根因

Coordinator01 的 lease claim 成功语义与后续 baseline attribution 所读取的 live lease ledger 发生分歧。该分歧使 exact untracked source path 无法从 lease 转为 attributed overlay，进而在 validation-copy baseline archive 阶段被误判为未拥有。

## 架构修复验收

- 对已接受的精确 untracked 文件 `lease claim` 必须在同一受管事务中持久化可查询的 live lease；success response、`lease list`、baseline attribute 和 heartbeat 必须观察到同一组路径。
- 在 lease 有效期内，`baseline attribute` 必须为该 session 写入当前 hash；只在真实缺少、冲突或过期时返回 `baseline_lease_missing`。
- `validation-copy materialize-cargo` 必须能使用已归因的 untracked Rust overlays 创建 immutable copy，并返回 source manifest/hash；副本中的 hash 必须等于 attribution/snapshot 的 hash。
- 增加 focused regression：一个 session 领取未跟踪 Rust 文件、attribute、materialize-cargo、heartbeat/expiry；覆盖文件级与目录-derived claim，且拒绝其他 session 的路径。
- 修复后重跑 Editor00 snapshot `1089` 等价 current-source manifest 的 validation-copy，再让 Editor00 继续受管 `cargo check -p zircon_editor --lib --locked --jobs 1`。

## 禁止临时方案

- 不得直接写 SQLite `leases`/`attributions`、手工复制 source root、或用共享工作树 Cargo 替代 immutable validation copy。
- 不得把目录 scope、过期 lease 或其他 session attribution 当作本 session 的 source ownership。
- 不得放宽 `validation_copy_unowned_path` 检查或用 tracked 空壳替换实际 untracked core 子树。

## 修复结果与回传

- 根因：The lease claim success response diverged from the persisted live lease ledger consumed by baseline attribution. Accepted untracked source paths could disappear before attribution, so immutable validation-copy materialization rejected the same paths as unowned.
- 架构修复：The current coordinator implementation persists exact and directory-derived lease claims transactionally, baseline attribution reads that same live ledger and records the current content hash, and validation copy overlays only attributed untracked sources. A dedicated end-to-end regression now locks exact-file and directory-derived ownership, foreign-session rejection, heartbeat expiry extension, attribution, and copied-byte identity.
- 验证：Local focused unittest passed 2/2 in 3.269s and Python compilation plus diff checks passed. Managed ticket 2c7893cc9fbe43f6a4c139d4c4527719, source manifest bc507259b0e471134c50212910c42284e8a4680edb76d12ce09ad7b891de6e02, copy job 2f6bfa7b723646ab890e3e42d9516e10 passed 2/2 in 3.432s with exit code 0. Handoff graph before return validated 561 artifacts with 0 errors.
- 回传：The coordinator-owned lease-to-attribution-to-validation-copy contract is accepted on an immutable managed copy. The original Editor00 session is archived and snapshot 1089 now has widespread current-hash drift across its historical business paths, so no historical session or Cargo replay was fabricated; Editor00 must use a new current-source session for any further business validation.
