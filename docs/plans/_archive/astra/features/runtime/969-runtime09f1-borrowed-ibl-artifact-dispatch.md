---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/09f1/2026-08-27-borrowed-ibl-artifact-dispatch.md
related_records:
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
  - docs/plans/astra/features/runtime/972-runtime09d-to-09h1-completion-list.md
implementation_files:
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests.rs
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests.rs
---

# Runtime969 · borrowed IBL artifact dispatch

| Area | Change | Evidence | Status |
| --- | --- | --- | --- |
| Runtime09F1 artifact selection | Runtime dispatch borrows asset and cache blobs, projects only copy-sized descriptors for canonical selection, and clones exactly the selected blob into the owned result. Priority, stale rejection, cache fallback, compute fallback, and public owned-candidate behavior remain unchanged. | Behavior coverage preserves asset priority, cache fallback, stale-to-compute fallback, and dispatch counts. The three local model runs preserve checksum `18440808946618895332`; worst-case P50/P95 reductions are `95.107%`/`80.767%`, with allocations `11→2` and bytes `2,360,136→262,272`. | implemented_pending_validation |

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/asset/artifact/ibl_bake_artifact_runtime_dispatch.rs` | `9561F9A62FECD80E1552E732A211D04AF0D5BE419C383C71D29CC5E6F4744F44` |
| `zircon_runtime/src/graphics/scene/scene_renderer/environment/ibl_bake_runtime_writeback/tests.rs` | `D2054E620DF58780099A4916160A92743CBFE014001149655A6542E5F683F130` |

## Validation handoff

The borrowed behavior tests and source/model contracts join the grouped Runtime
validation. Managed allocation/P50/P95 receipts and IBL product evidence remain
pending; no terminal result is inferred from the local model.
