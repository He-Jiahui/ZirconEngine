---
handoff_kind: failure
status: open
created_at: 2026-08-30
summary_slug: frameworks01-zr-resource-journal-intent-validation-materialization
origin_plan: docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
fixing_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
origin_child_dir: docs/plans/optimize/zircon_runtime/02
fixing_child_dir: docs/plans/zircon_runtime/frameworks/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
  - zircon_runtime/crates/zr_resource/src/io/transaction/journal/intent.rs
tests:
  - validation ticket 78ef39a572e1422e83b9c048832034e8
  - validation copy job 7b1be72ab9404e6aa1f16c4fe5450e4b
---

# Frameworks01: `zr_resource` journal intent is outside validation materialization

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md`
- 来源执行切片：batched Runtime/Editor Release validation for optimization batches 501-514
- 修复责任计划：`docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md`
- 交接原因：the missing source is the Frameworks01 physical owner path for the Runtime25
  resource hard cut, below Runtime02 optimization ownership.

## 失败现象与复现证据

After Runtime02 corrected `crash_windows.rs` from the removed monolithic journal path to the
canonical `zircon_runtime/crates/zr_resource/src/io/transaction/journal/intent.rs`, ticket
`78ef39a572e1422e83b9c048832034e8` reached closure planning with the new path. Job
`7b1be72ab9404e6aa1f16c4fe5450e4b` then failed
`validation_copy_compile_time_resource_missing` because the canonical intent file is still
untracked and owned by Session
`frameworks01-shader-invocation-hard-cut-r12-1b2684b4-20260825`.

Current intent SHA-256 is
`7781ec21ea073e8290e7e885b6637a614b4bcbc2b30f6222c9f74301b9ade8a5`. Runtime02 did not edit,
attribute, transfer, or include that foreign source in a commit candidate.

## 最低共享层根因

The hard-cut consumer now names the correct crate-owned implementation, but the implementation has
not entered a validation-copy materializable source closure. Repeating Runtime02 tickets cannot pass
closure planning while the file exists only as an untracked foreign-owner change.

## 架构修复验收

- Frameworks01 integrates the exact `zr_resource` journal intent source and its required module
  closure, or performs a legal exact-path ownership transfer with current hashes.
- The canonical path remains the only implementation; no monolithic Runtime compatibility file is
  restored.
- A managed Runtime/Editor validation copy advances beyond closure planning and compiles the
  batches against the same intent implementation.
- Runtime02 then reruns one aggregate validation rather than one ticket per optimization batch.

## 禁止临时方案

- Do not recreate `zircon_runtime/src/core/resource/io/transaction/journal/intent.rs`.
- Do not copy the intent implementation into Runtime02-owned tests or add an alternate include
  fallback.
- Do not use maintenance ownership override or claim Frameworks01 source without owner rotation.

## 修复结果与回传

Return the integrated/transfer request ID, exact intent hash, and managed validation evidence that
the source is materializable. Runtime02 will resume the aggregate 501-514 validation without
polling the fixing Session.

## 2026-09-07 Current Source Reconciliation

The preceding failure evidence describes the original 2026-08-30 source state.
The exact intent implementation is now tracked at Git blob
`ad79c25ec3a16fb0b82eefbecd6da536f845a081`, introduced by commit
`5798051603e7f7f565538125c9aba96d5beabae2` at `2026-09-01T02:29:39+08:00`.
Its current SHA-256 remains
`7781ec21ea073e8290e7e885b6637a614b4bcbc2b30f6222c9f74301b9ade8a5`, matching the
original missing-resource evidence. The current `crash_windows.rs` consumer
continues to include only the canonical crate-owned journal path.

The record was incorrectly stored in the origin Runtime02 directory despite
declaring Frameworks01 as its fixing owner. It is now in the declared fixing
child directory, with the required source-executor fields and section headings.
Its created date, summary slug, origin/fixing plans, validation ticket, copy job,
original reproduction, acceptance conditions, and lifecycle key are unchanged.
Pre-normalization snapshot `2937`, request `df91398cb43848e1aee23d029d51a310`,
retains the original file and the previously absent canonical destination.

Open state: `source integrated / managed materialization and upward validation pending`.
Tracked-source evidence resolves the historical untracked-source premise but
does not prove that ticket `78ef39a572e1422e83b9c048832034e8` passed. Its exact
successor must still establish materialization and the Runtime02 aggregate
501-514 compilation gate. No Cargo admission was retried while the user-requested
`zr_vm` exclusion remains in effect; no fixed return, closeout, or notification
is claimed.

The normalization passed independent review with Critical 0, Important 0,
Moderate 0, bound to pre-snapshot `2937` and post-snapshot `2938` (request
`591200cd67f348ccae1b1d144dd1e0fc`). The standard validator reports no finding
for this lifecycle; the full repository remains at 63 findings over 787 artifacts.
Import request `87d2276e35a0415ebafab49d7e30a151` was accepted, outlived its client
response timeout, and completed at `2026-09-07T12:32:32.869149+00:00`. Its exact
request status was reconciled without resubmission. The resulting index points
to this canonical fixing path with the unchanged lifecycle key and `open` status.

Original-ticket reception (`35c46bde97484399a2d76d729c56815b`) still reports
ticket `78ef39a572e1422e83b9c048832034e8` as `failed`, `cargoMs: 0`, with the
original intent-path materialization error. Its command was the Runtime/Editor
release filter `optimization_batch_20260830db_`; no pass is inferred from the
subsequent source integration or this document review.
