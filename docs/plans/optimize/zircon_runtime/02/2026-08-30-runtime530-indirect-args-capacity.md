---
title: Runtime Indirect Args Capacity 530
category: zircon_runtime
report_id: Runtime530-indirect-args-capacity-2026-08-30
date: 2026-08-30
session_id: root-runtime-editor-optimize-20260829-r5
implementation_status: implementation_complete
validation_status: managed_validation_snapshot_stale
---

# Runtime Indirect Args Capacity 530

GPU indirect draw batching previously grew its CPU indirect-argument vector incrementally while
walking a command list. Once indirect submission is supported, the command count is an exact upper
bound for generated argument rows, so the builder now reserves that capacity before the hot loop.
The larger batch-record vector remains demand-grown to avoid reserving one batch per command when
adjacent commands coalesce.

The ignored Release evidence `RUNTIME530_INDIRECT_ARGS_CAPACITY_BENCH_V1` models 32,768 frames with
256 commands per frame, or 8,388,608 argument pushes. The optimized model records zero capacity
growth events versus a positive legacy count, eliminating 100% of modeled argument-vector growth
events. This is an allocation-growth model, not an end-to-end render-time or GPU-time claim.

## Static evidence

- TDD RED: the supported-indirect path constructed `args_cpu` through `Self::default()`.
- TDD GREEN: `args_cpu` uses `Vec::with_capacity(commands.len())` before the command loop.
- A source contract rejects command-count preallocation for `batches`.
- Existing grouping, fallback, and per-draw indirect behavior tests remain the functional gate.
- `rustfmt 1.94.1 --edition 2024 --check` passes.
- `git diff --check` passes (PowerShell reports the repository LF/CRLF notice).
- Source SHA-256:
  `09d2306a4b479442e76a8c73896f86f94cce590010d4f7c871e549c9a06ab19b`.

## Acceptance gates

1. Managed Windows native Release compilation and focused Runtime tests pass.
2. Existing indirect grouping and fallback regressions remain green.
3. The ignored evidence emits the Runtime530 marker with zero optimized growth events.
4. Commit and push remain coordinator-owned; WeCom publication follows accepted validation only.

No direct Cargo validation, compile, commit, push, or WeCom success is claimed by this record.

## Managed validation result (2026-08-30)

The Runtime530/531 batch ticket `5d35546c649e4e16aea4544e2b7a782e` stopped before Cargo in
`materialization / owned_overlay`. Job `376b92b4427c4058abc45a89c18b7da7` reported
`validation_copy_attribution_stale` for this session's earlier Runtime506 dependency
`zircon_runtime/src/text/font/database/system_fonts.rs`. The Runtime530 source hash remained exact;
no compile, test, performance, commit, push, or WeCom success evidence was produced. Recovery
validation is pending after exact Runtime506 attribution repair.

The first recovery ticket `b61165051e4a43ba8448974c85716a6c` then reached the same
`materialization / owned_overlay` stage and exposed the next historical attribution dependency,
Runtime508 `zircon_runtime/src/ui/surface/surface/default_interactions/radio.rs`. Job
`99ab1ce5fa27435896e247762e438105` stopped before Cargo. The current Runtime508 blob differs from
its original manifest only by rustfmt import ordering and is now exactly re-attributed; aggregate
ownership scans report no remaining Runtime/Editor attribution blocker.
