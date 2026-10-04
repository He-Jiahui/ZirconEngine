---
title: Runtime Duplicate Control Detail Single Buffer
category: zircon_runtime
report_id: Runtime873-duplicate-control-detail-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime873 Duplicate Control Detail Single Buffer

## Finding

UI template compilation cloned every sorted duplicate control ID, collected the
clones into a vector, joined them, and formatted that joined child into the
final invalid-document detail. Large invalid component trees therefore incurred
one owned child string and one vector slot per distinct duplicate ID.

## Optimization

- Retain the existing `BTreeMap` as the ordering authority and borrow its keys.
- Sum key bytes and separators, allocate the final detail exactly once, and
  append the prefix and sorted IDs directly.
- Preserve duplicate detection/counting, deterministic lexical order, comma
  spelling, Unicode and empty IDs, asset attribution, and the success fast path.

## TDD and deterministic evidence

The combined Editor892/Runtime873 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. Lower regressions compare empty, singleton, out-of-order,
Unicode, empty-ID, and uneven-length maps against the retired
clone/collect/join/format implementation and require exact output capacity.

Across 4,096 details with 64 duplicate IDs, the deterministic model changes
cloned child strings/vector slots/join outputs from
`262144/262144/4096` to `0/0/0`; the required final error detail remains.
Ignored marker `RUNTIME873_DUPLICATE_CONTROL_DETAIL_SINGLE_BUFFER_BENCH_V1`
emits 101 alternating p50/p95/p99 sample pairs and requires single-buffer p95
to remain within 10% of clone/collect/join/format.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Runtime873/Editor892 plus adjacent settings-window and ZUI contracts pass
  `140/140`.
- Runtime873 received no per-task Cargo run and was submitted with Editor892 in
  asynchronous v19 (PID `32536`) at `2026-09-21T22:14:16.6877626+08:00`.
- No v13-v19 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/compiler/control_scope.rs` | `5A69499BC044CA772E0A9F6379D84CDF55987BAC5F223AB2F0E58FCF7D968777` |
| `zircon_runtime/src/ui/template/asset/compiler/control_scope/duplicate_detail_single_buffer_tests.rs` | `DC2E7DAC05D1E043A299D1E3B0797A5E3B5822F1B79986E1298AD14EB9539F7E` |
| `tools/tests/test_runtime873_duplicate_control_detail_single_buffer_performance_contract.py` | `ECDFD0A9BF1327819700C5E0A1FF45D5AFA89427C6BF7797EFF1AAA2EAD4B898` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Runtime, executes the lower
regression and ignored Release marker, and supplies allocator plus real invalid
UI-template compilation p50/p95/p99 evidence. The deterministic allocation
model is not product acceptance.
