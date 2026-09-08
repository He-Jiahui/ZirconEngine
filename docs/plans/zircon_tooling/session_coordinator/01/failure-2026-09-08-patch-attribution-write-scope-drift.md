---
handoff_kind: failure
status: open
created_at: 2026-09-08
summary_slug: patch-attribution-write-scope-drift
origin_plan: docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/patches.py
  - tools/session_coordinator/tests/test_patches.py
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/ownership_transfers.py
  - tools/session_coordinator/tests/test_ownership_transfers.py
  - tools/session_coordinator/snapshots.py
  - tools/session_coordinator/tests/test_snapshots.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_patches
  - python -m unittest tools.session_coordinator.tests.test_ownership_transfers
  - python -m unittest tools.session_coordinator.tests.test_snapshots
---

# Tooling01: applied patch attribution must retain its write scope

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md`
- 来源执行切片：项目 manifest fixture 输入闭包修复。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：PatchService 与显式 ownership transfer 的持久归属合同属于协调器。
- 上层：[manifest fixture failure](../../../optimize/zircon_runtime_interface/02/failure-2026-09-08-project-manifest-fixture-input-closure.md)。

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
  patch、伪造新 Session 或直接 Git 操作；该恢复仍需实现/验收。
- 正式 fixing-Session 验证绑定、failure return、closeout 完成。

## 禁止临时方案

- 不放宽重复 session register 的 immutable scope gate，不接管活动文件以清除错误。
- 不把外层 operational 作业冒充正式 failure closeout 票据。
- 不直接改 SQLite，不将历史 patch 重复应用，也不自动重发通知。

## 修复结果与回传

Open state: `future-patch-source-repaired_historical-recovery-pending`。
稳定 Session `failure-roll-01a07160-coordinator01`，baseline 601。完整阅读两文件及其
HEAD diff 后，transfer `706521579331451f80e1888009acc604` 保存前置快照 3169。
patches.py 原有 ObjectStore 事务改动被保留，test_patches.py 匹配已完成 owner 的原 hash。
源码快照 3171：patches.py
`aff46edaa31ab51a3ea2bbadefc83b68fed2e6c6a91fc751033fa5e9b1e04452`；
test_patches.py `3bd8ab4d06016e48ef75119fc42dc2a57f53926a31f674d640cd95f9e19c913b`。
Windows 隔离临时仓库中 `python -m unittest tools.session_coordinator.tests.test_patches`
6/6 通过，新增检查 exact scope、audit、排队不获取与 base 漂移不获取；静态 diff check 通过。
运行中的协调器尚未声明载入该源码；历史恢复、受管验证、独立审查与回传继续 pending。
未 failure return、未 commit、未发送企微。

### 2026-09-08 historical recovery candidate

Current open state: `source-repaired_managed-validation-and-runtime-reload-pending`.
Snapshot 3176 freezes the six implementation/support/test files. The new
ownership_transfers.py hash is
`c448f3cb1ea551eae1d334b70ca8116ad64fe98cc8ba4ab1485ad58987b3bb01`;
test_ownership_transfers.py is
`12586c11a106fa98206731466fb78ce7b7a7d998b50ceb9dcf57b2b83bcc0a38`.
Transfer `372b46a136384a07b0470b1ec7a8f384` and preimage 3175 retain the
previous owner's source; no prior implementation was discarded.

The existing preview/confirm/apply flow now admits an exact same-owner path only
when the current file matches its attribution and the Session scope omits it.
It restores scope through the normal audited transaction without changing source
bytes. Covered paths remain ineligible; stale hashes, foreign leases, or scope
changes after preview reject the operation. Structured path containment preserves
the coordinator's case-insensitive scope semantics and component boundaries.
The clean-baseline recovery test first failed with `path_matches_baseline` and
`path_already_owned_by_target`; the final patch plus ownership transfer suites
passed 24/24, including replay, stale preview, directory scope, adjacent directory,
root scope, and uppercase scope cases.

The pinned Session base does not contain the ObjectStore transaction support
already required by the inherited PatchService. Its current implementation and
tests were read and adopted through transfer fingerprint
`f74f9fa54a822b1833da7b9bc58e47a398430e68c5577a8ee3cbd90050583797`,
without further source edits. Snapshot 3176 includes snapshots.py
`8339988bccca9e6cb24d5cbb9aa89a11a075253a432248137425ab8f528777e9`
and test_snapshots.py
`3460aa4d0300d18eadb059a8f09fc4069a95e0d3edf5433d278511472bbe8834`.
The ObjectStore suite passed 5/5 on Windows.

These local regression results do not replace a fixing-Session managed ticket.
Independent review, complete managed input closure, supported runtime reload, and
the real patch-155 recovery remain pending. No live recovery request was applied.

### 2026-09-08 managed pass and review repair

Formal fixing-Session ticket `33d1855d99e64fc48a6dcaf61861a071` passed 29/29 on
snapshot 3176 plus record 3177 against pinned base
`585b031793088c28feef357488211f290927ec50`. The preceding ticket
`32e50de97942402cb7a5233012ea29bb` ran 28/29 successfully but lacked the dynamically
loaded failure validator. Adding its baseline dependency root corrected the input.
That older pass does not validate the subsequent crash-recovery changes below.

Independent review in the existing coordinator-efficiency task returned C0/I1/M0:
Git could change the worktree before an after-snapshot or database error, leaving
`applying` permanently outside queue recovery. Review evidence:
`.codex/tmp/coordinator01-patch-scope-3176-review-20260908-result.txt`.
Three actual fault-injection regressions reproduced that state; log:
`.codex/tmp/coordinator01-patch-crash-red-20260908.log`.

Patch application now holds an OS file lock for its lifetime, persists its start
audit before Git, and commits applied status together with exact scope and
attribution. An unexpected exception records current objects and a needs-rebase
audit without reverting the worktree. After a hard process exit, reconstruction or
queue processing can recover the same record once its lock is available and no
Git process can still be writing. Active locks, failed process enumeration and
live Git defer recovery. Historical records without lifetime-lock proof are not
guessed to be abandoned; the live read-only snapshot contained no applying records
before this repair. Lock files are retained so releasing one cannot race a new
owner onto a different inode.

Windows regression batch `test_patches`, `test_ownership_transfers` and
`test_snapshots` passed 36/36 in 71.840 seconds; log:
`.codex/tmp/coordinator01-patch-crash-green-r3-20260908.log`. It includes real child
`os._exit` after Git and immediately after the attribution transaction, rollback,
unchanged recovered bytes, exactly-once recovery audit, live application and
unknown/live Git guards. The source and canonical record must be frozen in a new
formal ticket and independently re-reviewed before integration. Current state:
`review-finding-source-repaired_managed-validation-and-review-pending`.

### 2026-09-08 interrupted lease lifecycle repair

Formal ticket `ed8d4f413eed45709c74f677fe3478cc` passed snapshot 3192 with
36/36 tests. Its independent review returned C0/I1/M0: a real hard exit after Git
or immediately after the applied transaction retained the original patch lease,
blocking another Session until lease expiry. Review evidence:
`.codex/tmp/coordinator01-patch-crash-3192-review-20260908-result.txt`.
The new regression log `.codex/tmp/coordinator01-patch-lease-red-20260908.log`
reproduced both failures, while renewed and replacement lease guards passed.

The application-start transaction now records the exact lease identity, including
its owner, path, base hash, acquisition, heartbeat and expiry. Every terminal
application transition conditionally releases that original lease in the same
transaction as its status, including APPLIED with scope/attribution and recovered
NEEDS_REBASE with current objects. No unconditional finally-release remains.
A lease renewed or acquired by a later owner is preserved. Recovery still requires
the lifetime lock and absence of a possible Git writer; no source is replayed.
Legacy records without lease proof do not authorize deleting a current lease.

Windows local regression batch passed 38/38 in 64.106 seconds; log:
`.codex/tmp/coordinator01-patch-lease-green-20260908.log`. Real child hard exits now
assert that another Session can acquire the path, including the crash immediately
after committing APPLIED. Active application locks, live or unknown Git, renewed
leases and replacement owners remain protected. This new source needs a new
formal ticket and C0/I0/M0 review. Integration, runtime reload, actual patch-155
scope recovery and failure closeout remain pending.

### 2026-09-08 application admission transaction

Formal ticket `4463a1ca14b54890a829c91e1aa650ba` passed snapshot 3197 with
38/38 tests. Independent review confirmed the hard-exit I1 repaired and reported
C0/I0/M1 for the earlier submission prelude: ObjectStore failure could leave a
lease with no patch row, and queued hash I/O failure could similarly leak a lease.
Review: `.codex/tmp/coordinator01-patch-lease-3197-review-20260908-result.txt`.

Submission now persists the queued patch and objects before acquiring any new
lease. Immediate and queued application share one admission path: hold the
application lock, acquire exact leases in the start transaction, and commit
APPLYING with lease proofs together. An admission failure rolls back lease
acquisition; the queued record remains retryable. The redundant pre-application
hash read in process_queue is removed; the existing guarded application path
checks the base and records NEEDS_REBASE with current bytes on I/O failure.

Four actual red regressions in
`.codex/tmp/coordinator01-patch-prelude-red-20260908.log` cover ObjectStore failure,
start-audit failure, lock failure and queued hash-read failure. The complete
Windows patch/ownership/ObjectStore batch passed 42/42 in 78.443 seconds;
`.codex/tmp/coordinator01-patch-prelude-green-20260908.log`. A new formal snapshot
and independent review are required; the preceding 38-test ticket does not
validate this source. Historical patch155 recovery and closeout remain pending.
