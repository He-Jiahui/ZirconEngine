---
title: Runtime77 Input Timer Drain Capacity
category: zircon_runtime
report_id: Runtime77-input-timer-drain-capacity-2026-09-19
date: 2026-09-19
session_id: root-runtime-editor-async-optimization-20260915
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime77 · Input timer drain capacity

## Scope

The existing single-pass `BTreeMap::retain` timer drains removed the expensive
clone-and-remove second pass, but each drain still grew its returned vector from
zero capacity when many timers expired. This follow-up adds a lazy map-size bound
for typeahead, submenu, tooltip, and toast drains. A no-expiry tick remains
zero-capacity and all map order, payload move, and pending-timer semantics stay
unchanged.

## Implementation

- Capture each expiration map's current length before `retain`.
- On the first expired item only, reserve that bound; subsequent items append
  without geometric growth.
- Keep `std::mem::take` payload moves and the existing deterministic BTreeMap
  iteration order.
- The lower regression covers all four drain kinds, preserves a future timer,
  checks returned capacity, and proves the no-expiry path remains allocation-free.
  The ignored benchmark emits `RUNTIME824_INPUT_TIMER_DRAIN_CAPACITY_BENCH_V1`.

## Local evidence

- TDD source contract is green (`3/3`).
- Existing lower retain-drain module contains the Runtime824 parity and Release
  marker and is Rustfmt-clean.
- The current merged non-tooling Runtime/Editor batch loads `623` modules and
  passes `2224/2224` tests with zero failures, errors, or skips; its receipt is
  recorded in the async admission log and Runtime UI aggregate.
- A later shared batch including Editor828 loads `624` modules and passes
  `2227/2227` tests with zero failures, errors, or skips; this remains local
  source/model evidence; the six-slice focused loader passes `21/21`. Managed
  Cargo/Release and product percentile gates remain open.
- Managed Windows Cargo/Release and input-product p50/p95/p99 evidence remain
  pending behind external dirty `E:\Git\zr_vm`; tooling production remains
  deferred for the later Rust migration.

## Source fingerprints

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/dispatch/input_manager/timers.rs` | `2A5A0B8D4A7F5A8ABA09472C23969364D40F89F86357501944FCAE2F7982B721` |
| `zircon_runtime/src/ui/dispatch/input_manager/timers/retain_drain_tests.rs` | `8803733CE2220396B718DCEF97C0748326952A52EA945BF10B5046722928093A` |
| `tools/tests/test_runtime_input_timer_drain_capacity_performance_contract.py` | `AF0FA7AE8BA9A502D5216F4BFA5AEC73A3D1C1BC826D12FD92C23008E1E931EA` |
