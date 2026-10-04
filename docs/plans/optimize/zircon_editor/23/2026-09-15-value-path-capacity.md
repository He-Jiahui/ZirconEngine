---
title: Editor23 Value Path First-segment Capacity
category: zircon_editor
report_id: Editor23-value-path-first-segment-capacity-2026-09-15
date: 2026-09-15
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 · Value-path first-segment capacity

## Scope

The byte-slice value-path parser already avoids whole-input and index-substring temporary
allocations, but it still created its result vector with zero capacity. Deep binding paths therefore
paid geometric vector growth even though ASCII `.` and `[` delimiters provide a safe upper bound for
the number of segments. This slice removes that growth while preserving parser permissiveness,
owned key output, malformed-input rejection, and mutation/lookup contracts.

## Implementation

- Keep the result vector empty until the first valid key or index is decoded.
- On that first push, count ASCII `.`/`[` delimiters in the borrowed source and reserve the
  resulting segment upper bound once.
- Do not reserve for empty, all-delimiter, or immediately malformed paths, so rejected input does
  not acquire a large scratch allocation.
- Keep the existing byte scanner and segment order unchanged.

## Regression and performance contract

The lower module
`zircon_editor/src/ui/asset_editor/value_path/capacity_tests.rs` compares the optimized parser
with a byte-slice baseline across Unicode, whitespace, permissive-dot, index, and malformed paths.
It also verifies that a delimiter-only input remains allocation-free at the result-vector level.
The ignored release benchmark emits `EDITOR23_VALUE_PATH_CAPACITY_BENCH_V1`, records paired P50/P95
samples, and models the geometric-growth events removed from a 4,096-segment path.

The Python source contract is
`tools/tests/test_editor_value_path_capacity_performance_contract.py`.

## Local receipt

The source contract passes `3/3`. The final current-source non-tooling Runtime/Editor
performance-contract invocation covers `550` modules and passes `1966/1966` in `24.601s`
with zero failures, errors, or skips, including this Editor slice. Scoped Rustfmt, wiki
navigation (`272/272` pages, one pre-existing metadata warning), path-reference, trailing-space,
and `git diff --check` guards also pass.

## Validation boundary

The focused source contract and Rustfmt checks are local evidence. The managed Windows Cargo/Release
run, allocator counts, and product p50/p95/p99 remain coordinator-owned and pending; this record
does not claim those gates or the broader Editor23 schema/transaction milestones complete. Tooling
production changes remain deferred for the later Rust migration.
