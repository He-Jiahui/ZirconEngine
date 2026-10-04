---
doc_type: milestone-detail
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/587/2026-09-01-shader-entry-point-name-move.md
  - docs/plans/optimize/zircon_runtime/588/2026-09-01-render-graph-dependency-move.md
  - docs/plans/optimize/zircon_runtime/589/2026-09-01-ui-v2-cache-source-move.md
  - docs/plans/optimize/zircon_runtime/596/2026-09-01-ui-v2-component-document-reuse.md
  - docs/plans/optimize/zircon_runtime/597/2026-09-01-prototype-frame-ownership.md
  - docs/plans/optimize/zircon_runtime/598/2026-09-01-pointer-drag-target-stream.md
  - docs/plans/optimize/zircon_runtime/602/2026-09-01-sprite-phase-stream.md
  - docs/plans/optimize/zircon_runtime/603/2026-09-01-state-hook-single-buffer.md
  - docs/plans/optimize/zircon_runtime/604/2026-09-01-pointer-id-index-union.md
  - docs/plans/optimize/zircon_runtime/607/2026-09-01-borrowed-source-id-prune.md
  - docs/plans/optimize/zircon_runtime/608/2026-09-01-borrowed-texture-dedup-index.md
  - docs/plans/optimize/zircon_runtime/609/2026-09-01-bounded-schema-slot-scan.md
  - docs/plans/optimize/zircon_runtime/609/2026-09-01-counter-hotspot-borrowed-group-key.md
  - docs/plans/optimize/zircon_runtime/610/2026-09-01-hash-delta-membership.md
  - docs/plans/optimize/zircon_runtime/610/2026-09-01-ui-hotspot-borrowed-scenario-key.md
  - docs/plans/optimize/zircon_runtime/611/2026-09-01-hash-reachability-membership.md
  - docs/plans/optimize/zircon_runtime/612/2026-09-01-ordered-level-registry.md
  - docs/plans/optimize/zircon_runtime/613/2026-09-01-counted-dependency-path-removal.md
  - docs/plans/optimize/zircon_runtime/614/2026-09-01-hash-scene-count-membership.md
  - docs/plans/optimize/zircon_runtime/615/2026-09-01-reused-chunk-hash-index.md
  - docs/plans/optimize/zircon_runtime/616/2026-09-01-adaptive-shader-contract-index.md
  - docs/plans/optimize/zircon_runtime/617/2026-09-01-preallocated-scene-validation.md
  - docs/plans/optimize/zircon_runtime/618/2026-09-01-preallocated-native-resource-validation.md
  - docs/plans/optimize/zircon_runtime/619/2026-09-01-preallocated-component-property-validation.md
  - docs/plans/optimize/zircon_runtime/620/2026-09-01-preallocated-plugin-event-validation.md
  - docs/plans/optimize/zircon_runtime/621/2026-09-01-preallocated-plugin-option-validation.md
  - docs/plans/optimize/zircon_runtime/622/2026-09-01-preallocated-interface-owner-dedup.md
  - docs/plans/optimize/zircon_runtime/623/2026-09-01-preallocated-module-closure.md
  - docs/plans/optimize/zircon_runtime/624/2026-09-01-preallocated-animation-graph-traversal.md
  - docs/plans/optimize/zircon_runtime/624/2026-09-01-preallocated-volume-param-validation.md
  - docs/plans/optimize/zircon_runtime/625/2026-09-01-preallocated-frame-batching-collections.md
  - docs/plans/optimize/zircon_runtime/626/2026-09-01-preallocated-gpu-scene-sync-indexes.md
  - docs/plans/optimize/zircon_runtime/627/2026-09-01-preallocated-shader-module-traversal.md
related_code:
  - zircon_runtime/src/graphics/shader/ide_validation.rs
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/ui/v2/component_instancer.rs
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer.rs
  - zircon_runtime/src/core/framework/picking/pointer_event_state.rs
  - zircon_runtime/src/core/framework/render/frame_extract/sprite_extract.rs
  - zircon_runtime/src/core/runtime/state_machine/hook_index.rs
  - zircon_runtime/src/core/framework/picking/report.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/counter_hotspot.rs
  - zircon_runtime/src/asset/pack/delta/optimization_tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot/tests/mod.rs
  - zircon_runtime/src/asset/mutation/tests.rs
  - zircon_runtime/src/scene/module/level_manager_lifecycle.rs
  - zircon_runtime/src/asset/pack/trim/optimization_tests.rs
  - zircon_runtime/src/asset/assets/scene/management.rs
  - zircon_runtime/src/asset/pack/manifest.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/validation.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/extension_registry/validation/component.rs
  - zircon_runtime/src/plugin/extension_registry/validation/plugin_event_catalog.rs
  - zircon_runtime/src/plugin/extension_registry/validation/plugin_option.rs
  - zircon_runtime/src/plugin/extension_registry/access/interface_owner_dedup_tests.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
  - zircon_runtime/src/animation/manager/graph.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_registry.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/shader/template/module_registry.rs
tests:
  - zircon_runtime/src/ui/template/asset/compiler/prototype_instancer/optimization_batch_hp_runtime597_tests.rs
  - zircon_runtime/src/core/framework/picking/pointer_event_state/optimization_batch_hq_runtime598_tests.rs
  - zircon_runtime/src/core/framework/render/frame_extract/sprite_extract/optimization_batch_ht_runtime602_tests.rs
  - zircon_runtime/src/core/runtime/state_machine/hook_index/optimization_batch_hu_runtime603_tests.rs
  - zircon_runtime/src/core/framework/picking/report/optimization_batch_hv_runtime604_tests.rs
  - zircon_runtime/src/core/framework/render/shader/variant_prewarm/optimization_batch_hx_runtime607_tests.rs
  - zircon_runtime/src/asset/assets/material/dependency_set/optimization_batch_hy_runtime608_tests.rs
  - zircon_runtime/src/asset/assets/material/material_asset/optimization_batch_hz_runtime609_tests.rs
  - zircon_runtime/src/asset/pack/delta/optimization_tests.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/ui_hotspot/tests/mod.rs
  - zircon_runtime/src/asset/mutation/tests.rs
  - zircon_runtime/src/asset/pack/trim/optimization_tests.rs
  - zircon_runtime/src/plugin/extension_registry/access/interface_owner_dedup_tests.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order/optimization_batch_im_runtime623_tests.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_registry/optimization_batch_in_runtime624_tests.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/collect_batching_result/optimization_batch_io_runtime625_tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync/optimization_batch_ip_runtime626_tests.rs
  - zircon_runtime/src/graphics/shader/template/module_registry/optimization_batch_iq_runtime627_tests.rs
---

# Runtime September Micro-Optimization Completion

This ledger closes the Astra-recording gap for the current Runtime587-627 micro-optimization
batch. Each cited optimize plan declares `implementation_complete`; this ledger records the
current source and regression owners without promoting helper benchmarks to product evidence.

## Plan Completion List

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| Runtime587 | Move shader entry-point names instead of cloning them | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime588 | Move render-graph dependency lists through compilation | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime589 | Move UI v2 cache sources into the retained cache | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime596 | Reuse the UI v2 component document during instancing | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime597 | Retain prototype frames by ownership transfer | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime598 | Stream pointer drag targets without an intermediate collection | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime602 | Stream sprite phase extraction | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime603 | Reuse the state-hook output buffer | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime604 | Union pointer IDs through an indexed membership path | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime607 | Prune shader source IDs through borrowed membership | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime608 | Deduplicate material textures through borrowed keys | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime609 | Bound material schema slot scans | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime609 | Use borrowed counter-hotspot group keys | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime610 | Use preallocated hash membership for pack deltas | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime610 | Use borrowed UI-hotspot scenario keys | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime611 | Use hash membership for mutation reachability | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime612 | Preserve deterministic level order without a repeated registry rebuild | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime613 | Remove dependency paths through counted membership | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime614 | Use hash membership for scene-count validation | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime615 | Reuse the pack chunk-hash index | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime616 | Index adaptive shader contracts | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime617 | Reserve dynamic-scene validation membership sets | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime618 | Reserve native-resource validation output | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime619 | Reserve component-property validation membership | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime620 | Reserve plugin-event validation membership | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime621 | Reserve plugin-option validation membership | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime622 | Deduplicate interface owners with a reserved index | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime623 | Reserve module-closure construction | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime624 | Reserve animation graph traversal state | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime624 | Reserve post-process volume parameter validation | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime625 | Reserve frame-batching collections from extract counts | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime626 | Reserve GPU-scene synchronization indexes | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |
| Runtime627 | Reserve shader-module traversal state | implemented_pending_validation | Source regression and scoped formatting pass; managed Release measurement remains pending. |

Tooling is intentionally out of scope. Promotion requires the managed Windows Runtime caller
tests and the corresponding Release p50/p95/p99 workload evidence.
