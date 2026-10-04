---
title: Editor04 Watch Projection Unique Dirty Stage
category: zircon_editor
date: 2026-09-26
implementation_status: implemented
validation_status: batched_validation_pending
---

# Editor04 Watch Projection Unique Dirty Stage

## Scope and target

A watcher batch can contain repeated modify events or both rename sides for one asset. The catalog dirty-stage output already has one row per UUID, but the previous loop cloned its full `AssetCatalogRecord` for every event before replacing the same map entry. For `E` events affecting one clean asset, the target is **one full catalog-record clone rather than `E` clones**, with one published dirty row and unchanged watch-index event handling.

## Implementation and evidence

The dirty-stage loop checks the existing UUID entry before reading and cloning the current catalog record. The first event stages the dirty row; later events for that UUID leave it intact. The ordered Runtime watcher event vector still flows into `EditorAssetIndex::apply_watch_events`, so event ordering and rename handling are unchanged. This is a deterministic clone-count reduction on repeated event batches. No dynamic timing pass is claimed before the managed run.

A focused regression compares one modified event with duplicate modified events plus a same-asset rename. Both produce the same single dirty row while the original immutable catalog row remains clean.

The ignored `editor04_watch_storm_dirty_stage_release_benchmark` runs 4,096 duplicate modify events against one clean catalog row with 64 diagnostics of 256 bytes each. Its test-local baseline reproduces the former repeated-clone loop, checks equal results, then alternates 21 paired samples. `PERF_RESULT EDITOR04_WATCH_STORM_DIRTY_STAGE_BENCH_V1` reports both paths' p50/p95/p99 nanoseconds. The Windows Release acceptance gate is optimized p95 at most 70% of baseline p95; this is a dirty-stage gate, not an end-to-end watcher/index latency claim.

## Validation manifest

| Gate | Scope | State |
|---|---|---|
| Source | `zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/watch_projection.rs` | Rustfmt, scoped diff check, and structural clone guard passed |
| Cargo | coalesce with the Editor04 folder batch: one managed Windows `zircon_editor` package check plus focused asset watcher/catalog regressions | pending coordinator batch |
| Performance | Run ignored `editor04_watch_storm_dirty_stage_release_benchmark` in the managed Windows Release Editor batch; dirty-stage p95 ratio ≤ 0.70 with p50/p95/p99 output under `EDITOR04_WATCH_STORM_DIRTY_STAGE_BENCH_V1` | pending coordinator batch |

This record remains an implemented optimization candidate until the shared regression and performance batch reaches a terminal pass.
