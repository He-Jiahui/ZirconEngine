---
title: Runtime Resolved Dependency Output Capacity
category: zircon_runtime
report_id: Runtime866-resolved-dependency-output-capacity-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime866 Resolved Dependency Output Capacity

## Finding

`resolve_dependencies` already sized its membership `HashSet` from the authored
dependency count, but both owned outputs started at zero capacity. Dense valid
dependencies geometrically grew `dependency_ids`; dense missing dependencies
geometrically grew diagnostics even though the same input length is a safe
upper bound.

## Optimization

- Initialize the ordered resolved-ID output with the authored dependency upper
  bound.
- Keep diagnostics at zero capacity on empty/all-resolved paths, then reserve
  the dependency upper bound once on the first missing locator.
- Preserve registry lookup, first-seen ID deduplication, dependency order,
  missing-locator diagnostic text/order, and empty behavior.

## TDD and deterministic evidence

The Runtime866 source/model contract was observed RED with three failures (two
semantic/model checks were already true), then GREEN at `5/5`. A lower
regression covers the empty zero-capacity path and a dense missing-locator path,
including exact lengths and reserved bounds.

For 4,096 retained values, the retired zero-capacity vector model performs 11
geometric growth events and the upper-bound model performs zero. All-resolved
input still keeps diagnostics at zero capacity. The ignored 101-pair Release
marker `RUNTIME866_RESOLVED_DEPENDENCY_OUTPUT_CAPACITY_BENCH_V1` emits
alternating p50/p95/p99 samples while the deterministic growth assertion remains
the non-flaky gate.

## Local validation boundary

- Exact-file Rustfmt and scoped `git diff --check` pass.
- Runtime864/865/866 adjacent asset contracts pass `13/13` in one batch.
- Runtime866 landed after v8 submission and remains queued with later Runtime
  work for another multi-task current-source lane; it was not submitted alone.
- Local source/model evidence does not establish Windows compilation, actual
  allocator behavior, or project-import product p50/p95/p99 latency.
- Tooling production remains deferred for the later Rust migration.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution.rs` | `A1207221C7FB47BA6F8595843484D73CD723C3CC7876C3449537213FF3D03A73` |
| `zircon_runtime/src/asset/project/manager/scan_and_import/dependency_resolution/optimization_batch_runtime866_resolved_dependency_capacity_tests.rs` | `A358CA0A29F73A1AEBF7E0BC261634104DEF2E075B143E79D164BEF338A556D5` |
| `tools/tests/test_runtime866_resolved_dependency_output_capacity_performance_contract.py` | `26C5042A0F1A4DDFD4CC52281555F0D648F52A87DB65AB21DA7588C61AE9D347` |

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Runtime, executes the lower
regression and ignored Release marker, and supplies allocator plus project-
import p50/p95/p99 evidence. The growth model is not product acceptance.
