---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/169/2026-09-20-navigation-projection-capacity.md
related_records:
  - docs/plans/astra/features/runtime/680-20260911-runtime-ui-optimization-batch-completion.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_runtime/src/navigation/runtime/world_scan.rs
  - zircon_runtime/src/navigation/runtime/world_scan/capacity_tests.rs
tests:
  - tools/tests/test_runtime_navigation_projection_capacity_performance_contract.py
---

# Runtime846 · navigation projection capacity

## 计划完成列表

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime navigation world projection | Reserve the known agent and obstacle component-row bounds for projection vectors before their drains, preserving filtering, ordering, transform fallback, and avoidance-index semantics. | TDD source/model contract `4/4`; lower reservation/empty regression and ignored `RUNTIME846_NAVIGATION_PROJECTION_CAPACITY_BENCH_V1` marker are wired; dense 4,096-row model changes `11→0` growth events per collector. The focused six-contract Runtime/Editor batch passes `24/24` in `0.013s`; the one-process broad non-tooling Runtime/Editor loader passes `2382/2382` across `651` modules in `5.334s`, with zero load errors/failures/errors/skips. Managed Cargo/Release, allocator, and navigation product p50/p95/p99 evidence remain pending. | implemented_pending_validation |

## Complexity boundary

This slice changes only temporary vector allocation shape. It does not alter
navigation authority, component decoding, row ordering, avoidance limits, or
tooling production.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/navigation/runtime/world_scan.rs` | `7D2C378679079250DC651502D48A9F0D62A4A2C3F791DBA2B690798C227D3984` |
| `zircon_runtime/src/navigation/runtime/world_scan/capacity_tests.rs` | `9670173E0C4EADDB4167B1FFBBCC388B86A7CD1E2D3A210332FD548EAFE445EE` |
| `tools/tests/test_runtime_navigation_projection_capacity_performance_contract.py` | `6BD373A0049B8AEEBBDB270E13FCC242C6145F9C0E8CB3214EDAFCEA5F00D296` |

## Managed gate

No standalone Cargo process is started locally and coordinator status is not
polled. Keep this entry `implemented_pending_validation` until the combined
owner-attributed Windows Release lane proves current-source compilation,
lower-test reachability, allocator behavior, and navigation projection
product p50/p95/p99 evidence.

## 2026-09-25 structure-guard repair

The new folder-backed `runtime/world_scan/capacity_tests.rs` regression guard
made the Runtime 14 module-family inventory grow from 16 to 17 Rust files. The
audit, Rust mirror contract, navigation module documentation, and Runtime 14
closeout now share that current count; the 16-file runtime-owner count remains
explicit so the test-only owner cannot be mistaken for a production split.
Focused repair validation passes locally; the combined Cargo/Release and
product percentile gates remain pending.

The post-repair grouped Runtime static discovery passes `2255/2255` in
`239.875s`, with zero failures, errors, load errors, or skips. This does not
replace the pending managed Windows Cargo/Release or navigation percentile
receipts.
