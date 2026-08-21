# Runtime08C Borrowed Deferred IK Set Record

- Date: 2026-08-21
- Owner: `optimize-runtime08c-ik-deferred-set-r1-01a00797-20260820`
- Source review: `docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md`, P1-17
- Execution plan: `docs/plans/optimize/zircon_runtime/08c-p1-17-ik-deferred-set.md`
- Status: implementation and regression definition complete; managed validation pending

## Problem

Clip-event admission already owns deferred entities as a `BTreeSet`, but the
animation tick copied that set into a new `Vec` before IK drain. Both animation
managers then called slice `contains` for every queued command. At the 4,096
command hard limit this added one allocation and quadratic membership work to
the owner-thread commit phase.

## Change

- `AnimationManager::drain_ik_commands_excluding` now borrows
  `&BTreeSet<EntityId>`.
- The animation tick lends `admission.deferred_entities` directly to the
  manager. The intermediate `collect::<Vec<_>>()` is removed.
- The core fallback and plugin managers partition their command vectors with
  ordered-set membership. Stable `Iterator::partition` preserves admitted and
  retained queue order, and replacement-epoch behavior is unchanged.
- Empty drains use an allocation-free empty `BTreeSet` value.

## Performance Contract

The ignored release gate measures 4,096 command identities against 2,048
deferred identities for 21 alternating-order sample pairs. The legacy side
materializes a `Vec` and performs linear membership; the optimized side borrows
the ordered set. Marker `PERF-MVP-RUNTIME08C-IK-DEFERRED` emits raw arrays,
nearest-rank P95, ratio basis points, and materialized-entity counts. Acceptance
requires optimized P95 to be at most 25% of legacy P95. Absolute latency and
the measured ratio remain pending until the serialized coordinator batch runs.

## Acceptance

- `selective_ik_drain_retains_deferred_entity_commands` retains the focused
  two-command behavior.
- `selective_ik_drain_preserves_order_at_queue_scale` locks admitted and
  deferred ordering across 1,024 queued commands.
- `replacement_epoch_retires_deferred_ik_commands_and_rejects_late_old_epoch`
  remains active for both manager implementations.
- `borrowed_deferred_ik_set_release_benchmark_evidence` defines the 21-pair
  alternating nearest-rank release gate and 75% minimum P95 reduction.
- External validator `zircon-validation-runtime08c-ik-deferred-set.ps1` is
  pinned at `A05EDB592F84F589F673DC87D3C749B490BAFDE4B42C4C4D1E0515A6AD1B33A5`;
  the 30-task post-Main Runtime parent is
  `A163354BA9FE3419EA88F6265C3EA9DE6A2699E8A01482904D0DB03731930063`.
- Rust 1.94.1 formatting and scoped `git diff --check`: passed.
- Cargo regressions and release P50/P95: pending the next managed multi-task
  Runtime batch; no direct or competing Cargo process was started.

## Remaining Plan Work

This slice removes only deferred-entity materialization and linear membership.
Prepared skeleton leases, one model-pose build per rig, command priority and
conflict semantics, joint limits, orientation, and contact behavior remain
under Runtime08C P1-17.
