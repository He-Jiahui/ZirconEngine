---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/931-editor09-reservation-preflight-borrowed-requests.md
  - docs/plans/astra/features/editor/936-editor09-submission-preflight-borrowed-batch.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/system/admission_ledger.rs
tests:
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract/reservation.rs
---

# Editor940 Editor09 Release-All Reservations Capacity


| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Shutdown reservation release | 关停时直接 `mem::take` reservations map，并消费分组条目调用既有索引删除路径；移除仅用于释放的 reservation ID 临时 `Vec`，保持 bytes、category 和 reservation 索引清理语义。 | source contract 确认 `mem::take` 消费 map、保留 `self.remove`，并禁止 `reservation_ids`/递归单项释放；单组和多组 shutdown reservation 行为测试覆盖 accounting 清零。 |
| 性能门禁 | 4,096 个 reservation group 的关停释放从一次 ID 缓冲分配降为零额外 ID 缓冲分配。 | ignored marker `EDITOR09_RELEASE_ALL_RESERVATIONS_CAPACITY_BENCH_V1` 要求 managed Editor09 Release 记录分配证据；本地 Editor09 Python 合同批量 `68/68`，托管回执仍待定。 |


- 本地 `rustfmt --edition 2021 --check` 通过。
- 本地 Editor09 Python 合同批量保持 `68/68`。
- 未修改 tooling；托管 Rust 编译与 Release 性能回执通过批量协调器验证后再更新状态。


### Fresh four-lane submission after Editor940 (2026-09-25)

Editor940 removes the shutdown-only reservation ID vector after the Runtime886
wave. The current Runtime/Editor source was submitted together:

- Runtime development: PTY `6829`
- Editor development: PTY `54360`
- Runtime02 Release ignored performance: PTY `97136`
- Editor09 Release ignored performance: PTY `24665`

The wrappers were intentionally left unpolled. Submission provides no Cargo
compiler result, test count, shutdown result, or Release performance threshold
receipt; tooling remains deferred. The first Release launch was rejected before
Cargo because `-TestFilter` requires `-LibTests`; the corrected two Release
wrappers above were then submitted with `-LibTests`. Local pre-submission
evidence remains Editor09 `68/68`, Rustfmt, source contract, and scoped diff
checks.

### Fresh four-lane submission after the multi-group shutdown regression (2026-09-25)

The additional behavior test now covers multiple reservation groups and verifies
that shutdown clears both pending-entry and pending-byte accounting before each
reservation commit is rejected. The current source was submitted together:

- Runtime development: PTY `27946`
- Editor development: PTY `98284`
- Runtime02 Release ignored performance: PTY `88515`
- Editor09 Release ignored performance: PTY `99265`

All wrappers remain intentionally unpolled. No Cargo, test-count, shutdown, or
Release timing result is inferred from launch; the managed performance gate is
still pending.
