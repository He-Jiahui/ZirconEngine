# Runtime72 partial-activation shutdown record

- Date: 2026-08-22
- Owner: `optimize-runtime72-shutdown-active-ledger-m3-r1-01a00797-20260822`
- Source plan: `docs/plans/optimize/zircon_runtime/72-runtime-core-lifecycle-registry-concurrency-shutdown-current-source-review.md`, RCL-P0-006
- Status: shutdown callsite, correctness regression, and activation-ledger gate contract complete; shared ledger owner wiring, grouped managed validation, and measurements pending

## Problem

`shutdown_registered_modules_with_drain_timeout` traversed every declared module in reverse graph
order and returned on the first deactivation error. A valid partial activation could therefore leave
an independent module in `Registered`; when that module appeared first in shutdown order, its
invalid lifecycle transition prevented cleanup of modules that were actually `Running`.

## Change

- The shutdown callsite now consumes an active-module order snapshot instead of traversing the
  frozen declaration graph.
- The shared state field, handle accessor, and successful activation/deactivation commit hooks are
  intentionally pending because those three owner files remain leased by the running Runtime+Plugin
  aggregate Session; the current Runtime72 source is therefore a TDD red state, not a compile claim.
- `Running` and `Stopping` modules retain the existing reverse dependency order, shared total drain
  deadline, lifecycle coordinator, veto, service-drain, cleanup, and error behavior.
- A deterministic regression registers the active module before a lexically later dormant module,
  activates only the first module, and requires shutdown to clean it exactly once while preserving
  the dormant module as `Registered`.
- The next ledger contract now requires dependency-ordered successful activation, removal after a
  successful unload, and tail insertion after reactivation; a regression fixes those transitions
  before the production owner fields are added.

## Deterministic contract

| Scenario | Previous result | Current result |
| --- | --- | --- |
| Dormant module precedes active module during reverse shutdown | `InvalidModuleLifecycleTransition`; active cleanup not reached | dormant module skipped; active module becomes `Unloaded` |
| Already unloaded module | accepted by deactivation | skipped before lifecycle coordination |
| Running or stopping module | reverse-order deactivation | unchanged |

## Acceptance

- `runtime_shutdown_skips_registered_modules_and_cleans_running_modules` locks the partial
  activation regression, cleanup count, and final lifecycle states.
- `runtime_shutdown_active_module_order_tracks_successful_reactivation` locks provider-before-
  consumer order, successful removal, and deterministic reactivation order.
- Existing reverse-dependency shutdown and deactivation atomicity tests remain in the same grouped
  test module.
- The ignored release gate fixes 16,384 declared modules, 8 active modules, 21 alternating sample
  pairs, raw nanosecond samples, nearest-rank P50/P95 recomputation, and optimized P95 at no more
  than 25% of the legacy full-registry scan.
- The external Runtime72 validator has SHA-256
  `CCF7280A7093CCF8694A093B5D2D09D652F7234615701136E09D83096F95FE23` and passes PowerShell AST
  parsing; it independently checks cardinalities, sample arrays, percentile values, operation
  counts, order declaration, and the 75% P95 reduction threshold.
- Exact-file Rustfmt and scoped `git diff --check` pass for the currently owned test and runtime
  files.
- Cargo execution is intentionally deferred to the next multi-task managed copy while the
  Runtime+Plugin 69-task validation batch owns the single Cargo lane.

## Performance and remaining scope

No measured shutdown-speed improvement is claimed yet. The callsite no longer contains the declared
graph scan, but the active-ledger provider is not wired until the aggregate Session releases its
shared owner paths. The release contract is encoded; only after those lifecycle commit hooks are in
place can the grouped Runtime72/81/89 copy produce accepted measurements.

Best-effort continuation after an individual cleanup failure, structured `RuntimeShutdownReport`,
and the unique product runtime owner remain in Runtime72 M3 scope.
