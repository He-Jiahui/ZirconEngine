---
title: Editor125 Tool Scheduler Revoke Capacity
category: zircon_editor
report_id: Editor125-tool-scheduler-revoke-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor828 · Tool scheduler revoke capacity

## Scope

Owner-generation and resource-kind revocation first collect matching active
leases and queued requests before detaching them. The former iterator
`collect` paths grew temporary vectors geometrically even when the match set
was sparse, while the output vectors also lacked an exact bound. This slice
keeps the existing owner/kind predicates, BTree order, request positions,
capture release events, and promotion semantics while making the scratch
capacity proportional to the first actual match.

## Implementation

- `revoke_owner_generation` now builds `lease_ids` and `request_positions`
  with lazy `reserve` using the respective map lengths; a no-match revoke
  remains zero-capacity.
- Released leases and withdrawn requests use the exact matched-vector lengths
  as output bounds before detachment, avoiding a second geometric growth path.
- The lower regression covers owner-generation and resource-kind filtering,
  preserves active/queued results, and keeps the scheduler empty after revoke.
  The ignored benchmark emits
  `EDITOR828_TOOL_SCHEDULER_REVOKE_CAPACITY_BENCH_V1`.
- Tooling production remains deferred for the later Rust migration.

## Local evidence

- TDD source/model contract is green (`3/3`).
- The lower Rust module is wired under `scheduler.rs` and Rustfmt-clean.
- The merged non-tooling Runtime/Editor source-contract batch loads `624`
  modules and passes `2227/2227` tests with zero failures, errors, or skips in
  one process (`22.343s`); the six-slice focused loader passes `21/21`; no
  Cargo process is started by this local slice.
- Managed Windows Cargo/Release and product scheduler p50/p95/p99 evidence
  remain pending behind the external dirty `E:\Git\zr_vm` gate.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/tools/scheduler.rs` | `2A57A3CDEC261C654C257B77579A342DD0A6E90E7D0CF8DEDCC6F1DECA72FDDC` (shared current-worktree hash after Editor876) |
| `zircon_editor/src/core/tools/optimization_batch_editor828_tool_scheduler_revoke_capacity_tests.rs` | `563A3AD6E2D0506C7D2E104D7D676EE8353CCDE91398C97804CF835AD3038393` |
| `tools/tests/test_editor_tool_scheduler_revoke_capacity_performance_contract.py` | `D9DDFE7651B78ABA11AF241988324B4E8C0E21561B94ACA2AC2E3CDA594E8681` |
