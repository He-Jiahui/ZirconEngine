---
title: Runtime05 Streamed Scene Binding Root Invalidation
category: zircon_runtime
report_id: Runtime05-streamed-scene-binding-root-invalidation-2026-08-27
date: 2026-08-31
session_id: root-runtime05-streamed-removal-release-r2-20260831
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime05 Streamed Scene Binding Root Invalidation

## Scope

This slice removes redundant sorting and deduplication when an entity removal invalidates its
tombstone identifier and old ancestor chain. A valid world hierarchy is acyclic, so that stream is
intrinsically unique. Reparent invalidation can combine three overlapping chains and deliberately
keeps its existing sort/dedup path. Scene binding identity, generation increments, tombstone
retention, and replacement-world behavior do not change.

## Change

- Feed the removed entity and old ancestor chain directly to `advance_roots`.
- Remove the removal-only `sort_unstable` and `dedup` passes.
- Preserve reparent overlap deduplication after an exploratory fully streamed variant showed a
  high-overlap P95 regression.
- Add a Rust behavior test proving the removed entity, its parent, and its root receive one shared
  invalidation generation.
- Release-r2 shares `scene_binding_removal_roots` between the production removal path and the
  in-crate release benchmark, without changing the reparent path.

## Deterministic Performance Evidence

The optimized Rust model uses a preheated persistent root-generation map with the same hasher for
both paths, 65,536 unique non-monotonic ancestor identifiers, and 31 alternating samples. Both
paths publish identical root maps. The model reproduces `advance_roots` collection on both sides;
it does not claim an allocation reduction.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Root buffer allocations | 1 | 1 | 0% |
| Root sort calls | 1 | 0 | 100% |
| Root dedup calls | 1 | 0 | 100% |
| Removal invalidation P50 | 12.020 ms | 10.159 ms | 15.483% |
| Removal invalidation P95 | 17.607 ms | 17.114 ms | 2.801% |

Evidence marker: `RUNTIME05_STREAMED_SCENE_BINDING_ROOT_INVALIDATION_MODEL_V1`.
The acceptance target is at least 10% P50 reduction with no P95 regression; both gates pass.

Release-r2 executes the same comparison against real `SceneBindingGenerations`: 65,536 unique,
non-monotonic roots preheat both persistent maps, then four paired warmups and 21 alternating
sample pairs compare the legacy `Vec + sort_unstable + dedup` path with the production streamed
helper. The gate emits both raw arrays and nearest-rank P50/P95, requires checksum parity, requires
`1 -> 0` sort and dedup calls per sample, and retains the same 10% P50/no-P95-regression targets.

## Validation

- Current baseline is Git HEAD `14c89f9776bed828cc85e05e4b9914b3f8d1e784`, coordinator epoch
  `575`.
- Release-r2 TDD RED: the benchmark guard failed and the production-helper guard raised one
  expected missing-helper error, while the other two contracts passed.
- Release-r2 GREEN: `python -m unittest
  tools.tests.test_runtime05_streamed_scene_binding_root_invalidation_performance_contract -v`
  passes 4/4 contracts.
- Exact-file `rustfmt +1.94.1 --check` and scoped diff checks pass.
- Release-r2 validation request `c846dd3e8c8346ac9ca0ab7a6f10b856` batches the behavior test and
  ignored release benchmark. The managed command is `cargo +1.94.1 test -p zircon_runtime
  --locked --release --jobs 1 -- streamed_scene_binding_root_invalidation_ --include-ignored
  --nocapture --test-threads=1`, with two expected tests.
- Managed Rust compilation, exact P50/P95, commit, push, and WeCom remain pending the asynchronous
  coordinator ticket.

## Remaining Parent-plan Work

Runtime05 still owns the broader ECS storage, schedule, query, lifecycle, snapshot, and product-scale
world validation gaps recorded in the canonical review.
