---
handoff_kind: fixed
status: fixed
created_at: 2026-09-08
summary_slug: patch-attribution-write-scope-drift
origin_plan: docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
tests:
  - python -m unittest tools.session_coordinator.tests.test_patches
  - python -m unittest tools.session_coordinator.tests.test_ownership_transfers
  - python -m unittest tools.session_coordinator.tests.test_snapshots
resolved_at: 2026-09-08
---

# Tooling01: applied patch attribution must retain its write scope

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
- 来源执行切片：项目 manifest fixture 输入闭包修复。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：PatchService 与显式 ownership transfer 的持久归属合同属于协调器。
- 上层：[manifest fixture failure](failure-2026-09-08-project-manifest-fixture-input-closure.md)。

## 失败现象与复现证据

请求 `05aa04987d8c491a9a189f98b852372a` 的 patch 155 已应用三行 Cargo metadata，
将 `zircon_runtime_interface/Cargo.toml` attribution 转给
`failure-roll-01a07160-interface02`，hash 为
`b5e30acb07e3e73019945a9bd855b59b4d7a4ca70d9c952971a1063af5669fa4`。
但该 Session write_scope 不含这个文件，后续重注册拒绝
`session_write_scope_immutable`（请求 `d1e0b5c3dfcd476cae9881af82b1cf2f`），
显式 self-transfer 也拒绝 `path_already_owned_by_target`（preview
`dffb7b3b098c48f5b62fba80f6dbfc28`）。证据来自命令终态与只读 SQLite；没有直接修改状态。

精确隔离回归 `test_applied_patch_adds_exact_target_to_existing_scope_and_audits_it`
首先失败：期望 `('docs/owned', 'README.md')`，实际只有 `('docs/owned',)`。

## 最低共享层根因

PatchService 在成功应用后写 attribution，却未调用 SessionService 现有
`extend_write_scope_in_connection`。新归属与不可变 scope 因此不一致，造成已经批准
并应用的 patch 无法按原 Session 完成精确提交。

## 架构修复验收

- 成功的即时或排队 patch，在同一数据库事务内追加精确 target scope 与 attribution，
  保留原 scope，记录 patch object hash 为审计身份。
- 仍在排队、base 漂移需重基或失败的 patch 不获取 target scope。
- 现有 six-test patch suite 和 ownership transfer 回归通过，独立审查 C0/I0/M0。
- 历史 patch 155 的范围缺失通过明确受审的协调器操作恢复，不靠改数据库、重放旧
  patch、伪造新 Session 或直接 Git 操作；该恢复已按下文证据验收。
- 正式 fixing-Session 验证绑定、failure return、closeout 完成。

## 禁止临时方案

- 不放宽重复 session register 的 immutable scope gate，不接管活动文件以清除错误。
- 不把外层 operational 作业冒充正式 failure closeout 票据。
- 不直接改 SQLite，不将历史 patch 重复应用，也不自动重发通知。

## 修复结果与回传

- 根因：Applied patch attribution omitted immutable Session scope extension; interrupted admission and Git completion also needed atomic lease lifecycle recovery.
- 架构修复：Persist queued objects before lease admission, bind exact scope and attribution to APPLIED, recover interrupted applications with OS lifetime proof, conditionally release the original lease, and support audited same-owner historical scope recovery.
- 验证：Snapshot 3204 fixing ticket fd748a69a6404bb9945da3328e0d9f69 passed 42/42; independent task 01a07063-6f03-7803-a12d-13ea015ca645 C0/I0/M0; integration 695e4e58987a1ac82111ac700f4e23b1ba52d74e WeCom succeeded; rollover 2c9f58ac9df84b74a72681530556b61e succeeded; apply c3b2d8a27ec04657842535908f2b0b53 restored actual patch155 scope without changing Cargo.toml hash.
- 回传：Interface02 can resume exact manifest-fixture validation and scoped delivery with its existing Session; its separate product acceptance remains open.

### Exact acceptance evidence

The formal command was
`python -B -m unittest tools.session_coordinator.tests.test_patches tools.session_coordinator.tests.test_ownership_transfers tools.session_coordinator.tests.test_snapshots -v`.
Ticket `fd748a69a6404bb9945da3328e0d9f69`, copy job
`697e1a0ba8614b91815cd504d628a133`, passed 42/42, exit 0, in 85.276 seconds.
The six source/test hashes match snapshot 3204 and the independent C0/I0/M0
review `.codex/tmp/coordinator01-patch-admission-3204-review-20260908-result.txt`.
`sessions.py` provides the existing scope-extension API; this repair did not
modify it and the exact closeout manifest excludes it.

Scoped candidate `4bfbc4c1de3a4d1d90f87442b501fa5e` integrated the reviewed
source as `695e4e58987a1ac82111ac700f4e23b1ba52d74e`, subject
`fix(failure): patch-attribution-write-scope-drift`, at
`2026-09-08T20:28:51+08:00` (7 files, 925 insertions, 67 deletions).
WeCom attempt `7ed783589dd04f62a0a34d7b8e81b028` succeeded with provider errcode 0.
The original canonical failure and its full RED/review-repair history are
preserved in that commit at
`docs/plans/zircon_tooling/session_coordinator/01/failure-2026-09-08-patch-attribution-write-scope-drift.md`.

Rollover action `2c9f58ac9df84b74a72681530556b61e`, intent
`af4f1653ce52484c997ab80d57405924`, succeeded with successor
`f3dc0f18de5c4123a7b96429ebbb4d78`. After the tray exhausted automatic startup
attempts, the supported `zircon-session.ps1 start` entry continued the accepted
rollover. No lifecycle request or patch was replayed.

Actual patch155 recovery used preview `ea981fa50f0b4ad695da43ec21b08e5b`,
fingerprint `f8fa65bf8edac8fa11a352bc3425ccc760426d7dba0fae30c348f35c3f7b705f`,
and apply `c3b2d8a27ec04657842535908f2b0b53`. Interface02's write scope now
includes `zircon_runtime_interface/Cargo.toml`; its hash remains
`b5e30acb07e3e73019945a9bd855b59b4d7a4ca70d9c952971a1063af5669fa4`.
The before/after observation is recorded in
`.codex/tmp/coordinator01-patch-admission-3204-rollover-recovery-result-20260908.json`.
Canonical return request `8a788f34881941b3b932fe68853a87ba` completed at
`2026-09-08T12:40:15.457273+00:00`. Final closeout review and commit bind this
record, its return receipt, the original deletion and the unchanged source hashes.
