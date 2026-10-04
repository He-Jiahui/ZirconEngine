---
title: Runtime77 Input Effect Result Capacity Reservation
category: zircon_runtime
report_id: Runtime77-effect-result-capacity-2026-09-14
date: 2026-09-14
session_id: root-runtime77-effect-result-capacity-20260914
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Input Effect Result Capacity Reservation

## Scope

`apply_dispatch_reply_core` already knows the exact number of effects in the reply, but every
result projection started with a zero-capacity `Vec`. This slice reserves that bounded upper
bound for applied, rejected, host-request, and component-event projections before the effect
transaction runs. Effect ordering, rejection/rollback behavior, diagnostics, and public DTOs are
unchanged.

## Implementation

- Added `reserve_effect_result_capacity` in
  `zircon_runtime/src/ui/surface/input/effect.rs`.
- The helper is called once after `UiInputDispatchResult` construction and reserves
  `effect_count` entries for each projection that can be populated by the effect loop.
- Added a lower Rust capacity regression and an ignored paired release marker
  `RUNTIME77_EFFECT_RESULT_CAPACITY_BENCH_V1`.

## Deterministic work model

For `E` effects, the common path no longer performs geometric growth for the four bounded result
projections: the known upper bound is reserved once (`4` growth paths -> `0` growth paths in the
model). The reservation is only an allocation-shape optimization; it does not claim a particular
allocator count, RSS value, or product frame latency until managed Release evidence is collected.

## Validation

- RED/GREEN Python source contract:
  `tools/tests/test_runtime_ui_effect_result_capacity_performance_contract.py` (`4/4`).
- Lower Rust capacity regression and the ignored release marker are present; Cargo execution is
  intentionally deferred to the shared managed batch.
- Scoped Rustfmt and diff checks pass for the production and regression files.
- The combined Runtime/Editor performance-contract loader passes `1832/1832` tests in one process;
  both Runtime77 capacity contracts are included in that receipt.
- The subsequent full non-tooling Runtime/Editor discovery passes `3581/3581` tests in one process
  (`441.896s`), providing batched regression coverage beyond the focused contracts.
- Managed Windows Cargo/Release and product p50/p95/p99 allocation/latency evidence remain
  pending because the shared admission gate is blocked by the external dirty `E:\Git\zr_vm`
  checkout. No coordinator status was polled.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/effect.rs` | `84E90214C1E51129E49E0A67E1CA271E26F2FD01AF66E0F32F2FF60D79AD2E25` |
| `zircon_runtime/src/ui/surface/input/effect/capacity_tests.rs` | `D665C10D18D280B26CD62967CB0055AB46CF7D51A3DC06215C8CAC2C3C53C49F` |
| `tools/tests/test_runtime_ui_effect_result_capacity_performance_contract.py` | `F8352CA06A38913105AC9E37CE79CE6BA81E7003B3A4DC67038902E08D34A0EC` |

## Remaining parent work

Runtime77 still owns the broader atomic effect transaction, qualified input identity, host
acknowledgement, queue budgets, and product input-to-present evidence. This capacity slice does
not close those architectural or managed-performance gates.
