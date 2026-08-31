---
title: Plugins12 Presized Owned World-Sync Projections
category: zircon_plugins
report_id: Plugins12-presized-world-sync-projections-2026-08-26
date: 2026-08-26
session_id: root-runtime-interface03-activate-link-failure-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Plugins12 presized owned world-sync projections

## Scope

- Parent scope: Plugins12 physics scene synchronization, specifically the projection from the already-owned `World::node_records()` snapshot into `PhysicsWorldSyncState`.
- Baseline: `f6f2fa1141da112c1a43abb5031dbbe6dec5b69d`, epoch `445`.
- Owned paths: `world_sync.rs`, its focused source contract, the shared projection/sort/discovery pressure model and contract, and this record.
- This slice preserves node ordering, validation, body/collider/joint/material contents, invalid-row filtering, transform resolution, public types, backend selection, and scene writeback. It does not replace the plan's required incremental scene delta, persistent backend query state, native solver work, or full-tick qualification.

## Change

`build_world_sync_state` now captures the owned node snapshot once, counts the four possible output row families, and gives each result vector its final upper-bound capacity before projection. It then consumes the snapshot rows instead of borrowing them and cloning nested payloads a second time.

The owned collider conversion moves triangle-mesh and height-field asset references into their physics DTOs. Joint constraint and skeleton-binding metadata are also moved. Material locator and override values are created once and shared only where both collider and material rows require separate ownership. The existing borrowed collider conversion remains available to command paths that do not own their scene component.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_plugins12_presized_world_sync_performance_contract -v` initially failed 5/5 because the snapshot was projected inline, all output vectors grew from zero, collider/joint rows borrowed their owned components, and no direct Rust contract covered the owned conversion.
- GREEN: the same focused source contract passes 5/5 after implementation.
- `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true` passes for `world_sync.rs`.
- Scoped `git diff --check` passes.
- The standalone model compiles with `rustc 1.94.1 -O`; it does not use Cargo or a shared build target.

The deterministic model measures 31 alternating legacy/owned sample pairs over 65,536 dense physics nodes. Input snapshot construction is outside the timed and allocation-counted region, so the result isolates the changed `SceneNode -> PhysicsWorldSyncState` projection. Each node carries a body, height-field asset reference, joint skeleton binding, and parent bone path. All four runs produced checksum `6649329941810118656` for both representations.

| Metric | Legacy borrowed projection | Presized owned projection | Change |
|---|---:|---:|---:|
| P50 | 57.4550 ms | 5.3678 ms | -90.66% |
| P95 | 90.9359 ms | 14.8975 ms | -83.62% |
| allocations / 65,536 nodes | 196,653 | 3 | -99.998474% |

The other three runs produced P50 reductions of 91.34%, 90.95%, and 90.90%; P95 reductions of 78.91%, 81.38%, and 93.53%; and the same allocation counts. These numbers cover CPU projection only and do not claim complete physics synchronization or frame latency.

## Async validation

### Current-source convergence receipt

- Ownership transfer preview request: `d7b0d0d9f6674ac29b4a1c63800c853c`.
- Ownership transfer apply request: `c776f5230c1449a7ad53d90d3b5ece4a`.
- Applied fingerprint: `0caacce1f20a3d0c48dd892678b0040ce9a71872faa7cc03e9a89dee4d792f0f`.
- Current session: `root-runtime-interface03-activate-link-failure-20260831`.
- Shared static/model ticket: `4c6aa5481e1440819e427ac1568979ab` (queued, 20 Python tests).
- Plugins12 Cargo behavior ticket: `0497da2f7bfd47e0bc66108bb2631eaf` (queued; exact owned-projection test).
- Shared model: `tools/plugins_projection_sort_discovery_pressure.py`, source manifest `ECAB605B9EC8C342B696C4CEB59AE2FB270F75976140B7086BFDD497CC18CC05`.
- Current source hashes: `world_sync.rs` `3FE13312870C08CEB0BC275ABB61A7039FCA88C1FCE48315612397FD4C3863FA`; shared model `2F51A8F953D139FE68E7DB1AB87653DFCC769DA01571BC6CC429040974D34323`.

The current-source model is deterministic structural evidence, not wall-clock timing. For 65,536 dense physics nodes it preserves one snapshot capture and 65,536 projection visits, adds the explicit 65,536-row capacity count, presizes four outputs, and changes modeled nested payload clones `196,608 -> 0` into `196,608` ownership moves. Historical alternating release evidence remains the performance claim: P50 `57.4550 -> 5.3678 ms` (`-90.66%`), P95 `90.9359 -> 14.8975 ms` (`-83.62%`), allocations `196,653 -> 3` (`-99.998474%`), checksum `6649329941810118656`.

The coordinator must run the five focused source contracts, the exact Rust test `manager::world_sync::tests::owned_projection_preserves_nested_payloads`, Rust formatting, scoped diff checks, checksum parity, and the enlarged standalone model. Acceptance requires 5/5 source contracts, exactly one filtered Rust test, checksum `6649329941810118656`, P50 and P95 reductions of at least 60%, and allocation reduction of at least 99%.

The validation ticket joins the outstanding optimization batch and the business Session does not wait for it before subsequent work. Cargo remains coordinator-owned. At the time of this record, foreign unmanaged wave85-wave87 build directories can stop managed copies at `artifact_governance`, and the foreign tracked deletion of `zircon_runtime/src/core/framework/render/environment/skybox.rs` can stop Cargo input-closure planning. Neither path is part of this candidate.

Integration and automatic WeCom publication remain coordinator-owned after managed validation and independent review succeed. The WeCom message must include the managed P50/P95 and allocation reductions and label them as CPU owned world-sync projection evidence for 65,536 nodes, not full physics-tick performance.
