---
title: Editor25 pipeline counter summary single buffer
category: zircon_editor
report_id: Editor825-pipeline-counter-summary-single-buffer-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260919
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Editor825 · pipeline counter summary single buffer

## Scope

`pipeline_counter_summary` represented the fixed ten-counter descriptor as a
temporary filtered `Vec<String>` and then joined it. Every active counter paid
an intermediate formatted string even though the output is already a single
diagnostic line.

## Implementation

- Keep the fixed counter names and source order in the existing descriptor.
- Walk the descriptor once, skip zero values, and append commas and
  `name=count` fields directly to one returned `String` through `fmt::Write`.
- Preserve the `none` result for an all-zero counter bag.
- Add lower byte-parity/order/source regressions and the ignored
  `EDITOR825_SINGLE_BUFFER_PIPELINE_COUNTER_SUMMARY_BENCH_V1` Release marker.

No pipeline counter ownership, field selection, ordering, wording, or Runtime
ABI changes are made.

## TDD and deterministic model

The focused source/model contract was intentionally RED against the prior
`format! → collect::<Vec<_>>() → join` chain and GREEN after direct writes
were introduced. With all ten counters active, the old shape creates ten
intermediate strings and one temporary vector; the new shape creates neither.
The final output `String` remains required. This is allocation-shape evidence,
not allocator, CPU, or product p50/p95/p99 evidence.

## Local evidence

- Focused source/model contract:
  `tools/tests/test_editor_pipeline_counter_summary_performance_contract.py`
  (`4/4`).
- Lower Rust regression compares the retired and direct output for mixed and
  all-zero counter bags. The ignored Release marker carries an alternating P95
  gate of `optimized <= 80% of retired`.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_editor/src/ui/workbench/debug_reflector/schedule_sections.rs` | `EE098A19C11BCD722D97A476C93F35EF9A4C4329FF2EF434CA5965BA9F6B6B30` |
| `tools/tests/test_editor_pipeline_counter_summary_performance_contract.py` | `D0BC8DC7AE33EB1E4DF5A1FF660E80FF10F1B59DE19E29518D7AD3FC83EE3A88` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending`
until the owner-attributed Windows Release batch compiles the current Editor
tree, runs the lower regression and ignored marker, and supplies allocator
plus Debug Reflector/product p50/p95/p99 evidence. Tooling production remains
deferred for the later Rust migration; this session does not poll the
coordinator.
