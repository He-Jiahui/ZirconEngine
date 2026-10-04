---
title: Editor02 Autosave Retired Diagnostic Capacity
category: zircon_editor
report_id: Editor800-autosave-retired-diagnostic-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor800 · autosave retired-diagnostic capacity

## Scope

`EditorAutosaveState::pump_retired` aggregates at most one persistence issue per retired
project per poll: `RetiredAutosaveProject::persist_fallback_diagnostics` stops after the first
failed append and returns an empty vector on a clean append. The aggregate issue vector therefore
has a safe upper bound equal to the retired-project count, but the normal no-error path should not
allocate merely because retired projects are draining.

## Implementation

- Keep the aggregate vector lazy for the normal no-error path.
- Once a retired project produces an issue, reserve the remaining retired-project bound before
  extending the aggregate vector.
- Preserve retired-project rotation, first-error retry, issue order, and all persistence/error
  semantics.
- Do not change the active-project fallback collector, whose error cardinality is not known
  without a pre-scan.

## Deterministic performance model

For 4,096 retired projects that each emit one issue, the legacy aggregate vector performs 11
geometric growth events from an empty start; the lazy bounded path reserves the remaining bound on
the first issue and performs zero growth events (`11 -> 0`). For a clean poll with no persistence
issues, both the logical output and the optimized vector capacity remain zero. This is allocation-
shape evidence, not allocator, CPU, RSS, or product p50/p95/p99 evidence.

## Local evidence

- TDD source contract `3/3` is green.
- Lower regression `optimization_batch_20260919_editor_autosave_diagnostic_capacity_preserves_issue_bound`
  covers the clean zero-capacity path and the one-issue-per-retired-project upper bound.
- Ignored Release marker `EDITOR_AUTOSAVE_DIAGNOSTIC_CAPACITY_BENCH_V1` is wired for the shared
  managed batch.
- The combined non-tooling Runtime/Editor source-contract loader passes `2208/2208` across `599`
  modules in `8.264s`, with zero failures, errors, or skips. This is a batched local receipt, not
  Cargo, Release, allocator, or product percentile acceptance.
- The focused cross-surface contract invocation for Editor800 and adjacent Runtime/Editor slices
  passes `23/23` in `0.026s`.
- Exact-file Rustfmt and Python compilation are required before the batched Runtime/Editor
  contract receipt; managed Cargo/Release and product autosave latency/allocation evidence remain
  pending.

### Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/core/recovery/autosave_service.rs` | `E641513A6C5B0108EC5C4004E48FBBF3AE6FFC3ADC8C9DD0CCC2831406845D30` |
| `zircon_editor/src/core/recovery/autosave_service/diagnostic_capacity_tests.rs` | `F3B4E2E002D94C94AA12335755DCCD049949BFAADD1380A13614C7450FBDE57B` |
| `tools/tests/test_editor_autosave_diagnostic_capacity_performance_contract.py` | `AFA6483C8831BE8BD4578084C75863F7DF9CB71EEC3D5A8AB0FF48109604E621` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until the owner-
attributed Windows Release batch compiles the Editor recovery module, runs the lower regression and
ignored marker, and supplies autosave diagnostic allocation and p50/p95/p99 evidence. Tooling
production work remains deferred for the later Rust migration.
