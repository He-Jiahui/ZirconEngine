---
title: Runtime58 Exact Plugin Dispatch Call Capacity
category: zircon_runtime
report_id: Runtime58-exact-plugin-dispatch-call-capacity-2026-08-27
date: 2026-08-27
session_id: root-runtime58-exact-plugin-dispatch-call-capacity-20260827
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime58 Exact Plugin Dispatch Call Capacity

## Scope

`dispatch_runtime_plugin_command_result` first materializes a complete `snapshots` vector, then
invokes every snapshot exactly once and appends exactly one `NativePluginRuntimeBehaviorCall` per
snapshot. The call-result vector previously started empty and repeatedly grew while moving already
completed plugin reports.

The dispatch path now initializes `calls` with `snapshots.len()`. This is an exact capacity, not an
upper-bound guess: snapshot acquisition aborts before dispatch on error, while every successfully
materialized tuple reaches the unconditional call push. Plugin ordering, callback invocation,
diagnostic collection, status and payload ownership, error propagation, and report shape are
unchanged. The diagnostics vector remains adaptive because each callback can contribute zero or
multiple diagnostics.

The same internal dispatch path is used by public command broadcast and the Play enter/exit
commands, so the allocation reduction applies to all three existing consumers.

## Performance Evidence

The isolated release model mirrors 65,536 plugin snapshots and the current 64-bit field layout as
an 80-byte call row. It runs 31 alternating sample pairs and 16 rounds per sample. The model was
compiled with `rustc -O` on Windows.

| Metric | Growing `Vec` | Exact capacity | Change |
|---|---:|---:|---:|
| Allocator calls per dispatch | 15 | 1 | -93.333% |
| Cumulative requested bytes per dispatch | 10,485,440 | 5,242,880 | -50.000% |
| P50 for 16 rounds | 84,664,200 ns | 46,434,800 ns | -45.154% |
| P95 for 16 rounds | 157,131,300 ns | 83,171,300 ns | -47.069% |

Model source:
`.codex/state/session-coordinator/runtime58-exact-plugin-dispatch-call-capacity-model.rs`.

## Contracts And Validation

- `tools/tests/test_runtime58_exact_plugin_dispatch_call_capacity_performance_contract.py` locks
  the exact `snapshots.len()` capacity, the one-call-per-snapshot push, diagnostic preservation,
  and the shared public/Play dispatch path.
- Existing Rust behavior coverage includes sorted multi-plugin broadcast, aborted snapshot
  acquisition without partial callback execution, 1/8/32-plugin managed benchmarks, and Play
  enter/exit state restore.
- Local source-contract result: 3 tests passed.
- Local `rustfmt --edition 2021 --check` passed for the production file.
- The release model passed allocator-call, requested-byte, P50, and P95 gates.
- Cargo compilation and focused Rust behavior tests remain pending in a managed asynchronous
  coordinator batch; no direct Cargo command was run.

## Remaining Parent-Plan Work

Runtime58 still owns bridge call leases, quiescence and retirement, generation-qualified native/VM
bindings, product registration replay, reload-safe World replacement, bounded diagnostics, fault
handling, soak, and same-hardware qualification. This slice only removes repeated heap growth from
the existing broadcast result projection.
