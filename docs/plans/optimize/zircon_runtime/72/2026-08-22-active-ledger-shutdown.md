# Runtime72 partial-activation shutdown record

- Date: 2026-08-22
- Owner: `optimize-runtime72-shutdown-active-ledger-m3-r1-01a00797-20260822`
- Source plan: `docs/plans/optimize/zircon_runtime/72-runtime-core-lifecycle-registry-concurrency-shutdown-current-source-review.md`, RCL-P0-006
- Status: current-source ledger owner wiring and deterministic gate contract complete; grouped managed validation and measurements pending

## Problem

`shutdown_registered_modules_with_drain_timeout` traversed every declared module in reverse graph
order and returned on the first deactivation error. A valid partial activation could therefore leave
an independent module in `Registered`; when that module appeared first in shutdown order, its
invalid lifecycle transition prevented cleanup of modules that were actually `Running`.

## Change

- The shutdown callsite now consumes an active-module order snapshot instead of traversing the
  frozen declaration graph.
- The shared state field, real `CoreHandle` accessor, and successful activation/deactivation commit
  hooks are now present in current source. The ledger remains Core Runtime-owned; no registry scan,
  compatibility facade, or duplicated state was introduced.
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
- Cargo execution remains coordinator-owned and pending as one grouped rerun with the three
  optimization batches that exposed the shared compile failure.

## 2026-08-31 current-source reconciliation

- Terminal tickets `d72cc9062ff744438f9365d44f26147d`,
  `7588c06a8fec4897b2165481148bb0f3`, and
  `12bc20b3c9fb4a7699972e4431789cd4` all exited `101` before their selected tests. Their three
  managed stderr logs contain the same seven `E0599` diagnostics for the missing
  `CoreHandle::active_module_shutdown_order`; no task-local behavior or performance gate ran.
- Archived/cancelled owner paths transferred to
  `root-runtime-editor-optimize-20260829-r5` through preview
  `039f7e494cb64fb09da1efcabdbacad6` and apply
  `be48188eb41c43408303288bc3f2fb34`. The two remaining exact paths had no live lease and were
  claimed and freshly attributed without override through requests
  `6fe47e3206e3424fbbc0044aa876cbd6` and
  `4c5179a4f5194177a6bcd15094eccd5d`.
- Current owner hashes are `01f66971...6618fae` (`activation.rs`),
  `0cac0c5b...474452` (`core_handle.rs`), `51c155cf...219a63` (`runtime.rs`),
  `11f8da56...d6855e3` (`core_runtime_state.rs`), and
  `c31ec199...a648347` (`veto_atomicity.rs`). Rust 1.94.1 exact-file `rustfmt --check`, scoped
  `git diff --check`, and static gate-shape checks pass.
- The failed tickets share base `74a79925014e5c8cc32710d4bd534d99ebb5b08e`, where the
  Runtime72 callsite and RED tests exist but the ledger owner does not. Current owner files also
  contain later task-graph, random-service, time-authority, and state-machine migrations. A narrow
  six-file overlay onto that base would therefore be an invalid source closure; absorbing the
  surrounding Runtime migration merely to force a retry is prohibited. The grouped managed rerun
  must bind the exact integrated dependency closure after those lower architecture sources land.

## Performance and remaining scope

No measured shutdown-speed improvement is claimed yet. The callsite and current-source provider are
wired, but the clean-copy dependency closure is not integrated on the ticket baseline. Only a
terminal grouped managed pass may publish the raw samples and P50/P95 measurements.

Best-effort continuation after an individual cleanup failure, structured `RuntimeShutdownReport`,
and the unique product runtime owner remain in Runtime72 M3 scope.
