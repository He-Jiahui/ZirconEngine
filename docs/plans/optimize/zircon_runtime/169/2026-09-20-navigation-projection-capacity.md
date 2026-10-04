---
title: Runtime navigation world projection capacity
category: zircon_runtime
report_id: Runtime846-navigation-projection-capacity-2026-09-20
date: 2026-09-20
session_id: root-runtime-editor-async-optimization-20260920
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime846 · navigation world projection capacity

## Scope

`collect_navigation_world_projection` receives bounded dynamic-component rows
for agents and obstacles. The projection previously built the `agents`,
`agent_positions`, and `obstacles` vectors from zero capacity even though each
source row count is known before its corresponding drain.

## Optimization

- Keep the existing sorted `component_rows` scratch and row accounting.
- Reserve the agent-row count for both agent output vectors before the first
  drain, and reserve the obstacle-row count before the second drain.
- Preserve deserialization filtering, transform fallback, stable row order,
  avoidance-index construction, and empty-input behavior.

No navigation query, avoidance, bake, or editor contract changes are included.

## TDD and deterministic evidence

The Python source/model contract was intentionally run RED before the
reservations and lower module existed, then GREEN after both were wired
(`4/4`). The folder-backed lower module checks reservation ordering, empty
zero-capacity behavior, and emits the ignored
`RUNTIME846_NAVIGATION_PROJECTION_CAPACITY_BENCH_V1` marker. A dense 4,096-row
model changes each zero-capacity collector's geometric growth from `11→0`
events.

## Local validation

- `tools/tests/test_runtime_navigation_projection_capacity_performance_contract.py`:
  `4/4`.
- Exact-file Rustfmt, Python compilation, and scoped diff checks pass.
- The focused six-contract Runtime/Editor batch (Runtime842/843/845/846/847
  and Editor844) passes `24/24` tests in `0.013s`, with zero failures, errors,
  or skips.
- The one-process broad non-tooling Runtime/Editor performance/pressure loader
  covers `651` modules and passes `2382/2382` tests in `5.334s`, with zero load
  errors, failures, errors, or skips. This is source/model evidence only.
- Managed Windows Cargo/Release, allocator, and navigation projection product
  p50/p95/p99 evidence remain pending behind the external worktree gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/navigation/runtime/world_scan.rs` | `7D2C378679079250DC651502D48A9F0D62A4A2C3F791DBA2B690798C227D3984` |
| `zircon_runtime/src/navigation/runtime/world_scan/capacity_tests.rs` | `9670173E0C4EADDB4167B1FFBBCC388B86A7CD1E2D3A210332FD548EAFE445EE` |
| `tools/tests/test_runtime_navigation_projection_capacity_performance_contract.py` | `6BD373A0049B8AEEBBDB270E13FCC242C6145F9C0E8CB3214EDAFCEA5F00D296` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
the owner-attributed managed Windows Release lane compiles the current tree,
executes the lower regression and ignored marker, and supplies allocator plus
navigation projection product p50/p95/p99 evidence. Tooling production remains
deferred for the later Rust migration.
