# Runtime01 Concurrent Activation Join Optimization Record

- Date: 2026-08-20
- Implementation owner: `optimize-runtime01-lifecycle-p0-01a012f4-20260818`
- Integration owner: `optimize-runtime22-virtual-delta-01a012f4-20260818`
- Source plan: `docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md`
- Status: implementation and 21-sample release-gate definition complete; managed revalidation pending

## Problem

Activation preflight accepted only `Registered`, `Running`, and `Unloaded` modules. Once the
first caller moved a module to `Initializing`, every concurrent caller was rejected before it
could reach the lifecycle coordinator's existing in-flight join path. Contending callers
therefore observed a false invalid-transition error instead of sharing the active build.

## Change

- Activation preflight now admits `Initializing` so the lifecycle coordinator can attach the
  caller to the in-flight `Activate` command.
- `Stopping` and all other invalid states remain rejected before dependency activation starts.
- The behavior gate waits until exactly seven joiners are registered before releasing the owner
  build, eliminating scheduler timing as an explanation for a missing join.
- All joiners must complete successfully, the module must finish `Running`, and the lifecycle
  factory must report exactly one build with no second build attempt.

## Performance Protocol

- Workload: one activation owner plus seven already-waiting joiners.
- Deterministic gate: `7` joined callers, `1` lifecycle build, `0` duplicate builds.
- Release evidence: `21` independent modules, nearest-rank P50/P95 over the interval from owner
  build release until all seven joiners report completion.
- Performance target: activation-join P95 must be at most `750 ms`. The ordinary contention
  regression applies the same per-sample bound.
- Actual P50/P95 values remain pending the serialized Windows coordinator run.

## Acceptance

- `concurrent_activation_joiners_share_one_build_within_contention_budget`
- `concurrent_activation_joiners_release_benchmark_evidence`
- Runtime01 validator requires `sample_count=21`, `joiners=7`, `builds=1`, P50/P95 fields, and a
  successful focused activation regression stage.
- Runtime01 and Runtime22 share the single five-Cargo-group parent validator
  `zircon-validation-runtime01-runtime22-batch.ps1`, SHA-256
  `2D7FB2C9FD91318524A139B008F7D851D40871E7651DA8798D966C5D72ADAAC7`.
- The earlier ticket `376315553d784c17a6d3a08294313cfc` is queue history only. Its stale shared
  baseline is not completion evidence; Runtime01 will be rematerialized from the post-Main HEAD.

## Remaining Scope

This record closes concurrent activation joining only. Cancellation, activation deadlines,
cross-module dependency rollback, unload admission, native-plugin quiescence, and broader
Runtime01 lifecycle qualification remain owned by the parent plan.
