---
title: Runtime Required Missing Summary Single Buffer
category: zircon_runtime
report_id: Runtime869-required-missing-summary-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime869 Required Missing Summary Single Buffer

## Finding

Runtime module-composition rejection formatted every missing required plugin
into a separate owned `String`, collected those strings into a temporary
vector, and joined them. Dense failure reports therefore paid one child
allocation and one vector slot per missing plugin before creating the required
summary output.

## Optimization

- Write every missing-plugin label, reason, and separator directly into one
  output `String` through `fmt::Write`.
- Preserve the borrowed availability order, exact label/reason spelling,
  `"; "` delimiter, trailing empty reason, and empty-report result.
- Leave availability ownership, fatal diagnostics, rejection display, module
  graph construction, and composition authority unchanged.

## TDD and deterministic evidence

The Runtime869 source/model contract was observed RED at `1/5` and GREEN at
`5/5`. Lower regressions lock empty, ordered, empty-reason, and Unicode-reason
output against the retired collect/join implementation.

For 4,096 missing plugins, the deterministic model changes temporary child
strings from `4096` to `0` and temporary vector slots from `4096` to `0`; the
required summary remains. Ignored marker
`RUNTIME869_REQUIRED_MISSING_SUMMARY_SINGLE_BUFFER_BENCH_V1` emits 101
alternating p50/p95/p99 sample pairs over 128 summaries of 64 entries and
requires single-buffer p95 to remain within 10% of collect/join.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Runtime869 and Editor888 contracts pass `10/10`; adjacent Editor behavior
  contracts add `18/18` passing tests.
- The attempted Runtime module-family audit reports only a stale structural
  count (`expected 16`, current `17`) caused by the foreign untracked
  `navigation/runtime/world_scan/capacity_tests.rs`; this task does not alter
  Navigation and Python tooling remains deferred.
- Runtime869 received no per-task Cargo run and was submitted with Editor888 in
  asynchronous v15 (PID `15424`).

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/builtin/runtime_modules/composition/outcome.rs` | `6D8446BD0D49767318A72AB4832C3969C02F09F33869509747EAD0424CE604B4` |
| `zircon_runtime/src/builtin/runtime_modules/composition/outcome/required_missing_summary_tests.rs` | `9402FA9AB01E31B2F159EAA0AD245C597DC723ECCFE0EF0D52B39927459EFC25` |
| `tools/tests/test_runtime869_required_missing_summary_single_buffer_performance_contract.py` | `3EE2A387B08FBE2FA32AE3D6C840EA1776AE8503413368F8A6F133957271EF3F` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Runtime, executes the lower
regression and ignored Release marker, and supplies allocator plus real module
composition-failure p50/p95/p99 evidence. The deterministic allocation model
is not product acceptance.
