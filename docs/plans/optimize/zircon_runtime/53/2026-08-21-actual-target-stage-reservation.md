# Runtime53 actual target-stage reservation

- Owner: `optimize-runtime53-actual-stage-reservation-r1-01a00797-20260821`
- Source plan: `53-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-review.md`
- Finding: `DSRL-P1-035`
- Status: implementation and deterministic capacity evidence complete; combined managed Cargo validation pending

## Problem

The asynchronous Level target-stage path reserved `prepared bytes + target snapshot limit` for the
entire task lifetime. The snapshot limit is an admission ceiling, not resident memory. A small
captured Level snapshot therefore continued to consume up to the 32 MiB default apply limit in
ready-result admission, diagnostics, and later target-stage scheduling until commit or removal.

## Change

`DynamicSceneAssetReloadStageTask` now starts with the same conservative maximum reservation. After
the target snapshot is captured, the worker publishes `prepared bytes + actual target bytes` through
an atomic reservation. Capture or staging failure publishes zero after the owned payload is dropped.
The synchronous World path continues to start with its already-exact reservation.

The queue no longer keeps a second mutable reservation total that can race the worker publication.
It folds the current atomic task reservations whenever admission or diagnostics needs the total.
`max_active_tasks` keeps that fold bounded to at most 32 entries by default. Running tasks remain
charged at their maximum, completed captures become exact, and removing a task makes its reservation
disappear from the next read without a stale cached remainder.

## Deterministic evidence

The release workload models 65,536 staged scenes. Each prepared payload is 64 KiB, the legacy target
ceiling is `32 MiB - 64 KiB`, and actual target snapshots range from 32 KiB through 63.5 KiB in a
repeating deterministic distribution.

| Metric | Maximum-limit reservation | Actual-snapshot reservation | Reduction |
| --- | ---: | ---: | ---: |
| Total reserved bytes | 2,199,023,255,552 | 7,499,415,552 | 99.659% |
| Released over-reservation | - | 2,191,523,840,000 | 99.659% |

The benchmark runs 21 alternating legacy/optimized sample pairs and emits independently
recomputable nearest-rank P50/P95 fields. Timing is diagnostic only because both models perform one
bounded addition per row. The hard release gate requires actual-snapshot reservation to use at most
25% of the legacy bytes; this workload uses 0.341%.

## Acceptance

- `dynamic_scene_asset_reload_target_stage_reconciles_to_actual_snapshot_bytes` uses a real
  `PreparedDynamicSceneSpawn`, `LevelSystem`, asynchronous target worker, and public diagnostics to
  prove the completed stage reports the actual snapshot reservation and removal returns it to zero.
- `dynamic_scene_asset_reload_actual_target_reservation_capacity_benchmark` emits 21 alternating
  timing pairs plus exact capacity fields and enforces the 75% byte-reduction threshold.
- The managed Runtime Rust follow-up batch runs the existing dynamic-scene regressions and ignored
  release gate together; no per-task Cargo process is launched from this session.

Pinned validation artifacts:

- Runtime53 child: `zircon-validation-runtime53-actual-target-stage-reservation.ps1`, SHA-256
  `B9AD792084DA88012291F98AB7FF7B71733D73EBD46D07B0B795FF6995AD563F`.
- Nine-task Runtime batch: `zircon-validation-runtime-rust-followup-nine.ps1`, SHA-256
  `E31A9B32864DD647A744BE86A6FB4F6A7F262C6E16D7480E53FC2E55B0E9DE9C`.
- Both scripts parse with zero PowerShell AST errors. Windows release timing, compilation, and test
  results remain pending until the post-Main materialized batch executes.

## Remaining scope

This closes only `DSRL-P1-035`. Runtime53's three P0 blockers remain open: unrelated project scenes
can still be selected for the active Level, reload remains append-only without instance replacement
ownership, and revision truth can advance before a reliable terminal disposition. The remaining
selection, instance registry, atomic replacement, retry, cancellation, shutdown, and qualification
items stay owned by the source plan.
