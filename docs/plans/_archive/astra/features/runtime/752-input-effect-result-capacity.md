---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77/2026-09-14-effect-result-capacity.md
related_code:
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/effect/capacity_tests.rs
tests:
  - zircon_runtime/src/ui/surface/input/effect/capacity_tests.rs
  - tools/tests/test_runtime_ui_effect_result_capacity_performance_contract.py
---

# Runtime77 Input Effect Result Capacity

`apply_dispatch_reply_core` now reserves the exact reply effect count before applying the effect
transaction. Applied, rejected, host-request, and component-event projections keep their existing
ordering and failure semantics while avoiding zero-capacity geometric growth in the common path.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
| --- | --- | --- | --- |
| Runtime77 / effect result projection | Reserve the known effect upper bound for all four result vectors | implemented_pending_validation | RED/GREEN source contract `4/4`; lower Rust capacity regression and ignored `RUNTIME77_EFFECT_RESULT_CAPACITY_BENCH_V1` marker added; scoped Rustfmt/diff checks pass. Managed Cargo and product allocation/latency evidence remain pending. |

## Complexity boundary

The effect loop, transaction ordering, rejection handling, and rollback path are unchanged. The
helper only changes initial vector capacity; it does not alter the public result shape or claim
product CPU/RSS/p50/p95/p99 until the managed release batch runs.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/surface/input/effect.rs` | `84E90214C1E51129E49E0A67E1CA271E26F2FD01AF66E0F32F2FF60D79AD2E25` |
| `zircon_runtime/src/ui/surface/input/effect/capacity_tests.rs` | `D665C10D18D280B26CD62967CB0055AB46CF7D51A3DC06215C8CAC2C3C53C49F` |
| `tools/tests/test_runtime_ui_effect_result_capacity_performance_contract.py` | `F8352CA06A38913105AC9E37CE79CE6BA81E7003B3A4DC67038902E08D34A0EC` |

## Managed gate

Cargo was not started locally. The shared managed admission remains blocked by the external dirty
`E:\Git\zr_vm` checkout; this slice is queued for the next batched Runtime/Editor validation.
Coordinator status was not polled, and no product performance claim is made here.
