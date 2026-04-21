---
related_code:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/extract_registration.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/prepare_frame/build_scene_prepare_frame.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/scene_frame.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_cell.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_clipmap.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/test_accessors.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/input_set.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/representation.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/surface_cache_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/voxel_scene_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/hybrid_gi/build_hybrid_gi_scene_prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/update_hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/card_capture_shading.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_bind_group.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/queue_params.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/readback/hybrid_gi_gpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/hybrid_gi_gpu_pending_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_hybrid_gi_buffers/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_representation.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
implementation_files:
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/backend_types.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/extract_registration.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/declarations/hybrid_gi_runtime_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/snapshot.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/prepare_frame/build_scene_prepare_frame.rs
  - zircon_runtime/src/graphics/types/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/mod.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/scene_frame.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_cell.rs
  - zircon_runtime/src/graphics/types/hybrid_gi_prepare/voxel_clipmap.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/test_accessors.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/input_set.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/representation.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/surface_cache_state.rs
  - zircon_runtime/src/graphics/runtime/hybrid_gi/scene_representation/voxel_scene_state.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/prepare_runtime_submission/hybrid_gi/build_hybrid_gi_scene_prepare.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submission_record_update.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/record_submission/update_hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/runtime_prepare/execute_runtime_prepare_passes.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/post_process/execute_post_process_stack.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/collect_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/card_capture_shading.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/voxel_clipmap_debug.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/create_bind_group.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_buffers.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/hybrid_gi_prepare_execution_inputs.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/execute_prepare/execute/queue_params.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_resolve_texture_id.rs
  - zircon_runtime/src/graphics/scene/resources/runtime/material_runtime.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/hybrid_gi_completion_params.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_resources/new/bind_group_layout/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/readback/hybrid_gi_gpu_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/hybrid_gi_gpu_pending_readback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/new.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/gpu_readback/pending_readback/collect.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/hybrid_gi/shaders/update_completion.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/run/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/execute/write_hybrid_gi_buffers/write.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/encode.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_material_fixtures.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_prepare_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/runtime_parent_chain.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_irradiance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_rt_lighting.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/resources/execute_post_process/encode_hybrid_gi_probes/hybrid_gi_hierarchy_resolve_weight.rs
plan_sources:
  - user: 2026-04-21 continue Hybrid GI / Lumen-style implementation and keep advancing the approved three-phase plan
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_gpu.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_resolve_render.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_scene_representation.rs
  - zircon_runtime/src/graphics/tests/hybrid_gi_runtime.rs
  - zircon_runtime/src/graphics/tests/render_framework_bridge.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - cargo test -p zircon_runtime --locked scene::tests::world_basics::render_extract_separates_directional_point_and_spot_lights -- --exact
  - cargo test -p zircon_runtime --locked asset::tests::assets::scene::scene_asset_toml_roundtrip_preserves_point_and_spot_lights -- --exact
  - cargo test -p zircon_runtime --locked core::framework::tests::render_frame_extract_roundtrip_preserves_split_light_lists -- --exact
  - cargo test -p zircon_runtime --locked core::framework::tests::hybrid_gi_extract_defaults_to_public_settings_and_empty_internal_fixture -- --exact
  - cargo test -p zircon_runtime --locked graphics::tests::hybrid_gi_scene_representation::hybrid_gi_input_contract_stays_complete_for_deferred_and_forward_plus -- --exact
  - cargo test -p zircon_runtime --locked exact_runtime_ -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_resolve_blends_nonzero_exact_ -- --nocapture
  - cargo test -p zircon_runtime --locked page_table_and_capture_slots -- --nocapture
  - cargo test -p zircon_runtime --locked reuses_surface_cache_slots_after_invalidation -- --nocapture
  - cargo test -p zircon_runtime --locked card_capture_requests -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_card_capture_requests_quantize_scene_prepare_requests -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_voxel_clipmaps_quantize_scene_prepare_clipmaps -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_prepare_descriptors_include_runtime_voxel_cells -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_prepare_descriptors_pack_explicit_card_capture_seed_rgb -- --nocapture
  - cargo test -p zircon_runtime --locked --lib gpu_scene_prepare_descriptors_preserve_explicit_black_card_capture_seed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_radiance_changes_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_changes_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_matches_different_card_capture_seed_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_card_capture_material_seed_changes_with_fixed_layout -- --nocapture
  - cargo test -p zircon_runtime --locked --lib collect_inputs_preserves_scene_prepare_contract_for_renderer_consumption -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_runtime_state_builds_scene_prepare_voxel_cells_from_scene_representation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_uses_runtime_voxel_cell_payload_without_scene_meshes -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_requires_runtime_voxel_cells_for_occupancy_and_count_truth -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_card_capture_samples_change_with_material_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_samples_change_with_material_emissive -- --nocapture
  - cargo test -p zircon_runtime --locked --lib graphics::tests::hybrid_gi_scene_prepare_resources -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_occupancy_changes_with_mesh_translation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_samples_follow_mesh_translation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_occupancy_counts_accumulate_overlapping_meshes -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_dominant_node_prefers_brighter_overlap -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_voxel_cell_dominant_sample_matches_brighter_overlap -- --nocapture
  - cargo test -p zircon_runtime --locked --lib scene_voxel_clipmap_occupancy_mask_moves_when_mesh_crosses_cells -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_readback_reports_scene_prepare_card_capture_resource_snapshot -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_reports_completed_probe_updates_and_traces -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_card_capture_requests_move_near_or_far_from_probe -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_clipmaps_move_near_or_far_from_probe -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_cells_move_near_or_far_from_probe -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_runtime_state_builds_scene_prepare_frame_from_scene_representation -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_resolve_uses_runtime_scene_voxel_point_light_seed_when_layout_and_tint_stay_fixed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_resolve_uses_runtime_scene_voxel_tint_when_layout_stays_fixed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_resolve_uses_runtime_scene_voxel_owner_card_capture_seed_when_layout_and_owner_stay_fixed -- --nocapture
  - cargo test -p zircon_runtime --locked --lib scene_prepare_present_black_voxel_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib spatial_fallback -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_scene_representation -- --nocapture
  - cargo test -p zircon_runtime --locked hybrid_gi_runtime_state_ -- --nocapture
  - cargo test -p zircon_runtime --locked --lib hybrid_gi_scene_prepare_resources -- --nocapture
  - cargo check -p zircon_runtime --locked --lib
doc_type: module-detail
---

# Hybrid GI Lumen-Style Scene Representation

## Purpose

这份文档记录 `Hybrid GI / Lumen-Style V1` 当前已经落地到 `zircon_runtime` 的第一阶段切口。重点不是最终 GI 质量，而是把“公共 extract 合同”和“renderer/runtime 内部 scene representation 真源”开始分开。

当前这轮实现只推进到 milestone 1 的基础层：

- 通用 scene extract 已经扩成 `directional + point + spot`
- `RenderHybridGiExtract` 已经收口成 public settings / budget / debug payload
- renderer/runtime 内部已经有独立的 `HybridGiSceneRepresentation / HybridGiSurfaceCacheState / HybridGiVoxelSceneState / HybridGiInputSet`
- cards、surface cache、voxel clipmap 的状态机开始进入内部权威状态，而不是继续把 authored probe / trace-region 当长期真源

## Public Contract Cutover

### Split Light Scene Extract

`RenderSceneGeometryExtract` 不再把所有灯型塞进单一 `lights` 列表，而是显式拆成：

- `directional_lights`
- `point_lights`
- `spot_lights`

这一步是后续 clustered direct-light injection 和 surface-cache direct-light seed 的前置条件。scene/world roundtrip 与 frame extract roundtrip 也已经一起升级，所以 scene authoring、asset roundtrip、frame contract 现在看到的是同一套 split-light 语义。

### Hybrid GI Public Extract Now Carries Settings Only

`RenderHybridGiExtract` 的公开面已经改成：

- `enabled`
- `quality`
- `trace_budget`
- `card_budget`
- `voxel_budget`
- `debug_view`

旧的 `probe_budget / tracing_budget / probes / trace_regions` 仍然临时存在，但只保留为 crate-internal fixture bridge，用来维持当前 runtime host 与旧测试夹具的迁移期可编译性。它们不再代表长期 authoring API，也不应该再被上抬成 render façade 的正式场景输入。

## Internal Scene Representation State

### Shared GI Input Contract

`HybridGiInputSet` 现在固定了 Lumen-style GI 需要的最小输入集：

- depth
- normal
- roughness
- base color
- emissive
- history validity
- motion vectors

当前实现把 Deferred 和 Forward+ 都约束到同一份输入合同上。Deferred 直接喂完整 GBuffer；Forward+ 的最终目标是补一套 `GBuffer-lite` 附件来满足同一 contract，而不是另做第二套 GI 算法。

### Scene Representation Skeleton

`HybridGiSceneRepresentation` 当前已经成为 runtime host 内部的聚合状态，负责持有：

- public settings mirror
- registered card descriptors
- surface cache state
- voxel scene state
- fixed GI input contract

这意味着 `Hybrid GI` 已经开始从“纯 probe runtime cache”向“scene-driven internal representation”过渡，哪怕当前 cards 还没有完全从真实 renderer scene registration 自动派生。

这轮继续往前推进后，这个描述已经不再只是骨架声明。`build_hybrid_gi_runtime(...)` 现在会把当前 frame 的真实 `meshes + directional/point/spot lights` 一起送进 runtime host，`HybridGiRuntimeState` 不再只消费 `RenderHybridGiExtract` 的 settings。

## Milestone 1 State Behavior

### Surface Cache Budgeting

`HybridGiSceneRepresentation::synchronize_cards(...)` 目前提供了第一版 deterministic card-state 迁移逻辑：

- scene 中 active card id 会被规范成稳定去重后的 card descriptor 列表
- `card_budget` 决定本帧可 resident 的 surface-cache page 数
- 已 resident 且仍然有效的 page 会优先保留
- 新进入 resident 的 page 会被标记为 dirty capture
- 超出 budget 的 card 会进入 feedback 列表，表示后续需要 capture / residency 机会
- 离开 active 集合或因为 budget 收缩而被挤出的 resident page 会进入 invalidation 列表

当前 surface-cache 还不是最终 atlas/page-table 实现，但已经具备 milestone 1 需要的最小状态机语义：注册、复用、失效、反馈。

这一段继续推进后，surface-cache 已经不再只是 resident/dirty 列表：

- resident page 现在会持有 deterministic `page table -> atlas slot` 映射
- dirty resident page 会持有 deterministic `card capture atlas slot` 映射
- page 继续 resident 时会尽量保留原 atlas slot
- invalidated page 会释放 atlas / capture 槽位，后续新 page 复用最低可用槽位
- resident page 的 capture slot reservation 现在也会独立保留，所以 page 在 clean frame 之后再次变 dirty 时仍会回到原 capture slot，而不是被重新压到最低空槽

也就是说，Milestone 1 里的 `page table / atlas / card capture atlas` bookkeeping 已经开始落到 runtime 内部权威状态。renderer 这一侧现在也已经补上了第一版 per-frame GPU atlas / capture RT scaffold，而且不再只是“空纹理 + slot-truth”：当前已经会把 scene-driven request 写成第一版 scene-driven direct-light seed texel 内容并做 sample readback。只要 request 能解析到 matching mesh，renderer 就会直接消费当前 frame 的 `meshes + directional/point/spot lights`，并进一步经由材质解析拿到 `base_color + emissive`，再和 mesh tint 一起合成最小 capture radiance；只有解析不到 matching mesh 时才退回 deterministic debug texel。它仍然不是最终的 surface-cache shading pass。

最新这层又把材质真值拉进了同一条 seam，而不是继续停在 mesh-instance tint 代理：

- `collect_inputs(...)` 仍然只传 scene mesh / split-light snapshot，不新增 public extract DTO
- `SceneRendererCore::execute_runtime_prepare_passes(...)` 现在会把 `ResourceStreamer` 一起交给 `HybridGiGpuResources::execute_prepare(...)`
- card-capture 着色逻辑已经从 `create_buffers.rs` 拆到独立的 `card_capture_shading.rs`
- `ResourceStreamer` 现在能通过已准备好的 `MaterialRuntime`，或必要时回退到 `ProjectAssetManager::load_material_asset(...)`，解析 card capture 需要的 `base_color + emissive` 种子
- atlas / capture texel 的最小 radiance 现在由 `mesh tint * material base_color + material emissive + directional/point/spot direct-light seed` 共同决定，而不再只是 mesh tint 乘灯光
- `create_buffers.rs` 现在会在创建 atlas / capture 纹理之前就把同一份真实 texel 颜色写进 `scene_prepare_resources.atlas_slot_rgba_samples / capture_slot_rgba_samples`，因此这份 snapshot 不再只在 pending-readback collect 之后才有意义；它在当前 frame 的 post-process 阶段就已经能代表 authoritative card-capture seed truth
- 同一份 `scene_prepare_resources` snapshot 现在还会为每个 resident voxel clipmap 派生一条最小 `voxel_clipmap_rgba_samples` 调试样本，并额外记录 `voxel_clipmap_occupancy_masks`、`voxel_clipmap_cell_rgba_samples`、`voxel_clipmap_cell_occupancy_counts`、`voxel_clipmap_cell_dominant_node_ids` 与 `voxel_clipmap_cell_dominant_rgba_samples`。前者用 clipmap 包围内的 scene mesh/material/light 种子聚合成 deterministic radiance；occupancy 会把 mesh translation 粗量化进固定 `4x4x4 -> u64` occupancy grid；cell sample 会在同一固定 `4x4x4` grid 的每个 occupied cell center 上重用同一份 material/light 着色种子，形成 cell-level volume-content readback；cell count 会把重叠 mesh 对同一 coarse voxel cell 的占用次数直接压回 readback；dominant node id 则会把当前 cell 内 radiance 更强的 mesh authority 固定下来；dominant RGBA sample 则把这份更亮 contributor 自己的 radiance 颜色保留下来，从而和 aggregate cell sample 分离，方便在 Milestone 1 阶段同时验证 voxel scene 的 radiance seed、空间驻留、粗体素内容、cell-level residency density、coarse contributor ownership 与 authority color truth 都已经接到 scene-driven capture 链路

### Card Capture Request Descriptors

在这层 bookkeeping 之上，scene representation 现在还会继续派生一份真正面向 renderer seam 的 `card capture request` 描述，而不只是“有哪些 dirty page/capture slot”：

- 每条 request 都会同时带上 `card_id / page_id / atlas_slot_id / capture_slot_id`
- request 还会携带 card 当前的 `bounds_center / bounds_radius`
- request 集合只覆盖当前 dirty resident page，不会把 clean resident page 混进 capture 队列
- 当 resident page 保持不变、只有其中一张 card 再次变 dirty 时，request 会继续复用原 `atlas_slot_id + capture_slot_id`

这意味着 Milestone 1 现在不只是有 page-table/capture-slot bookkeeping，还已经把 “哪张 card 该被 capture 到哪个 atlas/capture slot，以及它当前代表的几何包围” 固定成内部真源。当前 renderer 创建 per-frame card-capture atlas / capture textures 时，已经直接消费这份 scene-driven descriptor，因此后续真正接入 capture shading pass 时，不需要重新发明一套 slot 对齐逻辑。

### Runtime-Owned Voxel Cell Residency Contract

这一轮继续把 voxel residency 的权威来源从 renderer-local mesh iteration 往 runtime host 收了一层，但只先收结构性真值，不提前伪装最终 voxel shading authority 已经完成：

- `HybridGiVoxelSceneState` 现在会在 runtime `synchronize(...)` 阶段，按每个 resident clipmap 固定的 `4x4x4` cell grid 生成 `HybridGiPrepareVoxelCell { clipmap_id, cell_index, occupancy_count, dominant_card_id, radiance_rgb }`
- `HybridGiScenePrepareFrame` 现在除了 `card_capture_requests + voxel_clipmaps` 之外，还会继续导出 `voxel_cells`
- runtime 为每个 resident clipmap 固定导出完整 `64` 个 cell entry，而不是只导出 occupied cell；这样 renderer/readback 侧可以保持 deterministic cell ordering
- 这份 payload 现在已经固定了 coarse residency density、dominant contributor ownership，以及 dominant tint + split direct-light seed；但它仍然不编码完整材质/表面 cache/emissive 真值，也还不是最终的软件 voxel lighting authority

这条 cutover 的目标很明确：Milestone 1 先让 `scene representation -> runtime scene prepare` 真正拥有 voxel cell occupancy/count truth，而不是继续把 renderer 里的 `scene_meshes` 重算当长期权威。

### Renderer Scene-Prepare GPU Contract

Milestone 1 这轮又把这份 scene-driven truth 继续推进到了 renderer prepare seam，而不是只停在 runtime host：

- `HybridGiRuntimeState::build_scene_prepare_frame()` 会从 `HybridGiSceneRepresentation` 导出 `HybridGiScenePrepareFrame`
- `submit_frame_extract` 会把这份内部 frame 放进 `ViewportRenderFrame.hybrid_gi_scene_prepare`
- `SceneRendererCore::execute_runtime_prepare_passes(...)` 再把它和旧的 `HybridGiPrepareFrame / HybridGiResolveRuntime` 一起送进 Hybrid GI GPU prepare
- `collect_inputs(...)` 现在除了透传 `card_capture_requests / voxel_clipmaps` 之外，也会把 `voxel_cells` 一起带进 renderer prepare 输入；因此 renderer 当前已经能在不重新扫描 scene mesh 的情况下消费 runtime-owned voxel occupancy/count/ownership/color truth
- renderer prepare 仍然会把 `frame.extract.geometry.meshes` 与 split-light `directional/point/spot` 一并送进 `collect_inputs(...)`，但这条 mesh 输入当前只继续服务 card-capture shading、voxel radiance sample 和 dominant-contributor debug，不再负责长期持有 voxel occupancy/count authority

renderer 端没有再为 cards 和 voxel clipmaps 各开一条独立 storage-buffer，而是显式收束成一条统一的 `scene_prepare_descriptor_buffer`：

- binding `4` 现在固定给只读 scene-prepare descriptor buffer
- 原本的 completed/irradiance/trace-lighting 输出顺延到 bindings `5..8`
- 之所以这样收束，是因为当前机器上的 `wgpu` compute-stage storage-buffer 上限只有 `8`；如果 cards 和 voxels 各占一条独立 buffer，会直接超过 binding limit

`update_completion.wgsl` 也不再只是“把 scene descriptor 绑上去但完全不读”。当前 shader 已经开始真实消费这份统一 descriptor：

- `card_capture_requests` 会按 `card/page/atlas/capture/bounds` 量化后进入 GPU
- `create_buffers.rs` 现在也会在 card descriptor staging 阶段复用同一份 `scene_card_capture_rgba(...)` 真实场景 seed，把 `RGB` 打进 card descriptor 的 `_padding0`，并用 `_padding1` 明确标记“这是显式 packed seed 而不是缺省值”；因此 `RGB = [0, 0, 0]` 的黑色 seed 不会再和“没有 packed seed，只能退回旧逻辑”混为一谈
- `voxel_clipmaps` 会按 `clipmap_id/center/half_extent` 量化后进入 GPU
- `voxel_cells` 现在也会按 `clipmap_id/cell_index/occupancy_count/dominant_card_id/radiance_rgb/cell_center/cell_half_extent` 量化后进入 GPU；`create_buffers.rs` 会把 runtime-owned `radiance_rgb` 打进 unified descriptor 的 `quaternary_id`，并把 `dominant_card_id` 打进 `_padding0`
- `update_completion.wgsl` 对 card-capture descriptor 现在也不再只靠 `card/page/slot/bounds` 的 synthetic 数学推色；当 `_padding1 != 0` 时，它会优先直接解出 `_padding0` 里的 packed card seed，只有旧 fixture 或缺失 packed seed 的 descriptor 才继续退回旧的 synthetic card color
- 这条 real-seed authority 现在也继续贯穿到了当前 frame 的 final resolve：`render/render.rs -> execute_post_process_stack.rs -> execute/run/execute.rs -> write_hybrid_gi_buffers/write.rs -> encode_hybrid_gi_probes/encode.rs` 会把本帧 `HybridGiGpuPendingReadback` 持有的 `scene_prepare_resources` snapshot 只读透传进 `hybrid_gi_hierarchy_rt_lighting.rs`，owner-matched voxel miss fallback 会先按 `capture_slot_id`、再按 `atlas_slot_id` 读取真实 slot sample，而不再直接退回 `scene_prepare_card_capture_request_rgb(...)` 的 synthetic request math
- `update_completion.wgsl` 对 voxel-cell descriptor 不再只用 synthetic color math；当 `quaternary_id` 非零时，它会优先把这份 runtime `radiance_rgb` 当成 cell color authority，只有 authority 缺失时才回退到旧的 synthetic voxel-cell 色彩
- 当 `quaternary_id == 0` 但 `_padding0` 带有非零 `dominant_card_id` 时，`update_completion.wgsl` 现在会先尝试复用同帧 `card_capture_request` 里匹配 `card_id` 的 scene seed。由于 card descriptor 本身已经先吃到真实 packed card seed，这条 owner reuse 路径不再只跟着 `capture_slot_id` 之类的 synthetic layout 信息走，而是能在 fixed-layout 下继续反映 material/base-color/emissive/direct-light 变化；只有找不到匹配 card request 时才退回 owner-based hash fallback
- shader 会对附近 probe 叠加一层 scene-driven radiance boost，所以 near/far scene descriptor 现在会真实改变 GPU readback

在 unified descriptor buffer 之外，renderer prepare 现在还会继续创建一份 per-frame `scene_prepare_resources` scaffold：

- atlas 纹理尺寸由 `atlas_slot_count` 和固定列数直接推导
- capture 纹理会按 `capture_slot_count` 生成 `2D-array` 资源与逐 layer view
- scene-driven `card_capture_requests` 现在会被编码成第一版 scene-driven direct-light seed RGBA，并真实写进 atlas tile 与 capture layer；当前 seed 来源是 matching mesh 的 `tint` 加上当前 frame 的 `directional/point/spot` lights，缺失 matching mesh 时才会回退到 deterministic debug RGBA
- 同一份 `scene_card_capture_rgba(...)` 结果现在不只写进 atlas/capture 纹理，也会同步进入 unified card descriptor；scene-prepare texture path 和 GPU completion descriptor path 因此开始共用同一份 card seed 真值，而不是前者写真实 texel、后者继续靠 slot/id 公式猜色
- 这些纹理、views 和 upload buffers 当前通过 `HybridGiGpuPendingReadback` 保活到 frame 完成，再以 `HybridGiScenePrepareResourcesSnapshot` 形式进入 `HybridGiGpuReadback`
- snapshot 会显式暴露 `occupied_atlas_slots / occupied_capture_slots / atlas_slot_count / capture_slot_count / atlas_texture_extent / capture_texture_extent / capture_layer_count`
- snapshot 现在还会带回 `atlas_slot_rgba_samples / capture_slot_rgba_samples`，用于验证每个 occupied slot/layer 的真实 texel 内容
- snapshot 现在还会带回 `voxel_clipmap_rgba_samples`，用于验证每个 resident clipmap 的最小 radiance seed 样本，而不必只通过 `update_completion.wgsl` 对 probe readback 的间接偏置来判断 voxel scene 有没有活起来；当 runtime 为 clipmap 内的 occupied cells 提供了非零 `radiance_rgb` 时，这个 aggregate clipmap sample 现在也会优先从 runtime `voxel_cells` 按 `occupancy_count` 加权聚合，而不是继续只依赖 renderer-local mesh/material/light path
- snapshot 现在还会带回 `voxel_clipmap_occupancy_masks`，用固定 `4x4x4` clipmap-local occupancy grid 的 `u64` bitmask 去证明 scene mesh 平移时，voxel residency/readback 也会同步变化，而不是只剩颜色样本会变
- snapshot 现在还会带回 `voxel_clipmap_cell_rgba_samples`，把固定 `4x4x4` clipmap-local grid 的每个 cell RGBA 样本都压回 readback；这让 Milestone 1 不只知道 clipmap 是否被激活，还能观察最粗一层 voxel volume content 是否跟着 scene mesh/material/light translation 一起迁移
- snapshot 现在还会带回 `voxel_clipmap_cell_occupancy_counts`，并且其数据源已经完全 cutover 到 runtime-owned `voxel_cells`：renderer 只负责把这份 fixed-grid payload 展开成 per-cell count 与 occupancy mask；当 payload 为空时，occupancy/count 就保持为零，不再回退到旧的 mesh-derived cell count 路径
- snapshot 现在也会在 `voxel_clipmap_cell_rgba_samples / voxel_clipmap_cell_dominant_rgba_samples` 上优先消费 runtime-owned `voxel_cells` 的独立 radiance presence 合同：当 runtime 为某个 occupied cell 提供了 `radiance_present == true` 的 scene authority，scene-prepare readback 会直接把这份颜色权威写回；即使 `radiance_rgb == [0,0,0]` 也会保留为显式黑色 authority，只有 `radiance_present == false` 时才继续退回 renderer-local mesh/material/light voxel debug sample
- snapshot 现在还会带回 `voxel_clipmap_cell_dominant_node_ids`，把同一固定 `4x4x4` grid 下每个 cell 当前由哪个 mesh 主导也压回 readback；当 runtime 为某个 occupied cell 提供了非零 `dominant_card_id` 时，这份 dominant-node readback 现在也会优先消费 runtime voxel payload，而不是继续只从 renderer-local scene meshes 回推 ownership。这让 Milestone 1 可以在重叠 contributor 存在时区分“一个 cell 被多少 mesh 命中”与“最终哪一个 mesh 是 coarse voxel authority”
- snapshot 现在还会带回 `voxel_clipmap_cell_dominant_rgba_samples`，把同一固定 `4x4x4` grid 下每个 cell 当前 dominant contributor 自己的 radiance 颜色也压回 readback；这让 Milestone 1 可以继续区分“整个 coarse voxel cell 聚合后的能量/颜色”与“当前真正主导这个 cell 的 contributor 颜色”，避免 overlapping mesh 只剩 authority id 而没有 authority color truth
- renderer-local `voxel_clipmap_rgba_samples / voxel_clipmap_cell_rgba_samples / voxel_clipmap_cell_dominant_rgba_samples` 现在还共享了一条显式 presence 合同：只要当前 frame 真有 scene mesh 为该 clipmap/cell 贡献样本，RGBA 的 `alpha` 就写成 `255`；完全没有 renderer-side sample 时才写成 `0`。这样当前 frame 的“显式黑色 voxel radiance”终于可以和“没有样本”分开表示，而不会再被压扁成同一个 `[0,0,0]`

换句话说，这个 checkpoint 已经把 runtime-owned `voxel_cells` 从单纯的 `occupancy/count authority` 推到 `occupancy/count + dominant contributor id + dominant tint/direct-light seed authority`，而且不再只停在 resolve/GPU completion 两条链上：`HybridGiVoxelSceneState` 现在会把 scene mesh 的 dominant contributor id 与 `tint + split direct-light seed` 一起量化进每个 cell，resolve miss fallback、GPU completion readback、以及 `scene_prepare` debug/readback 都会优先消费这份 scene-driven 体素真值。最新这层又把 dominant owner 本身进一步连到了同帧 `card_capture_request` scene seed：只要 runtime 没给出 `radiance_present == true` 的 cell radiance，但仍给出 dominant owner，GPU completion 不会再直接压回空间启发式，而是优先复用匹配 card 的 capture seed；而一旦 runtime 明确给出 `radiance_present == true`，即便颜色是 `[0,0,0]`，resolve/GPU/readback 也会把它当成显式黑色 authority 保留下来。clipmap aggregate sample 本身同样已经收回到 runtime cell authority，只要 runtime 给出了 `radiance_present == true` 的 cells，`voxel_clipmap_rgba_samples` 就会按 runtime cell occupancy 加权聚合出 clipmap 颜色，而不是继续只依赖 renderer-local voxel debug 着色。更深一层的 material/emissive/direct-light shading authority 仍然刻意留在 `voxel_clipmap_debug.rs` 的 renderer-local mesh/material/light path，作为下一层更深的 authority 收束点。

这一步仍然主要是 Milestone 1 的 seam 验证，不代表 surface cache 或 voxel fallback 已经达到最终 lighting 质量；但它已经把 “scene representation -> runtime frame -> renderer descriptor buffer + per-frame atlas scaffold -> shader consumption / readback observability” 这条链路真正闭合起来了。

### First Resolve-Side Software Voxel Fallback

在此基础上，resolve 侧现在也不再只有 probe-style trace continuation。`hybrid_gi_hierarchy_rt_lighting(...)` 已经开始在“当前帧没有 scheduled trace region 命中”时读取 `ViewportRenderFrame.hybrid_gi_scene_prepare`：

- runtime exact / ancestor / descendant RT-lighting continuation 仍然优先，保持原有 runtime history 语义不变
- 只有当前 trace 路径给不出有效 RT-lighting 时，才会转到 `scene_prepare` 的 voxel fallback
- fallback 当前先使用 `voxel_clipmaps + voxel_cells` 的 fixed-grid 空间真值来重建 cell center / cell extent，并对附近 probe 给出第一版 cell-level 软件 voxel RT-lighting
- `voxel_cells` 现在不再只是 occupancy/count；runtime scene representation 会把每个 cell 的 dominant mesh tint 量化成 per-cell `radiance_rgb`，resolve 在有这份 scene-driven 色彩权威时优先使用它
- 当 `radiance_rgb` 缺失但 `dominant_card_id` 有效时，resolve 现在也会先尝试复用 `scene_prepare.card_capture_requests` 里匹配 `card_id` 的 scene seed；只有 owner 找不到匹配 card request 时，才继续退回 clipmap-local 空间启发式
- 如果当前 frame 只有 clipmap descriptor 而没有有效 `voxel_cells`，resolve 也会退回 clipmap-level coarse fallback，而不是直接回到纯黑
- 当前的 runtime voxel authority 仍然不是完整的材质/表面 cache 采样；当 `radiance_rgb` 缺失时，resolve 现在会先走 matched card-capture seed，再退回 clipmap-local 空间启发式来避免 miss-path 直接变黑
- 这一层现在又往前收了一步：只要当前帧 `scene_prepare_resources` 里已经有 renderer 侧生成的 voxel sample，resolve 在 runtime `radiance_present == false` 时会优先尝试 `voxel_clipmap_cell_dominant_rgba_samples / voxel_clipmap_cell_rgba_samples`，而 coarse clipmap fallback 也会优先尝试 `voxel_clipmap_rgba_samples`，不再一上来就掉回纯空间启发式。renderer-side 资源路径继续按 sample `alpha > 0` 判断 authority presence，而 runtime-owned `voxel_cells` 则新增了独立 `radiance_present` 位；因此无论 authority 来自当前 frame 的 renderer sample 还是 runtime voxel payload，显式黑色 GI 样本都不会再和“没有 authority”混成同一个 `[0,0,0]`。

因此，当前仓库已经不再是“trace miss 就只能回 probe-only continuation 或纯黑”。即便还没有正式的 screen-trace 命中链，这条 resolve-side software voxel fallback 已经把 milestone 2 里最核心的 miss-path 语义先打通了一版。

### Stats And Readback Surface

Milestone 1 的验收要求之一是 debug/readback。当前仓库没有把 scene/surface/voxel 内部结构直接上抬成 façade DTO，而是沿现有 `HybridGiRuntimeSnapshot -> SubmissionRecordUpdate -> RenderStats` 主链，把这些 scene-driven计数暴露给外部：

- `last_hybrid_gi_scene_card_count`
- `last_hybrid_gi_surface_cache_resident_page_count`
- `last_hybrid_gi_surface_cache_dirty_page_count`
- `last_hybrid_gi_surface_cache_feedback_card_count`
- `last_hybrid_gi_surface_cache_capture_slot_count`
- `last_hybrid_gi_surface_cache_invalidated_page_count`
- `last_hybrid_gi_voxel_resident_clipmap_count`
- `last_hybrid_gi_voxel_dirty_clipmap_count`
- `last_hybrid_gi_voxel_invalidated_clipmap_count`

这里的 `surface_cache_capture_slot_count` 现在语义上等价于“当前待执行的 card capture request 数量”，因为统计链已经改为从 `HybridGiSceneRepresentation::card_capture_request_count()` 取值，而不再只是盲读 surface-cache dirty slot 容器长度。

### Scene-Driven Card Registration And Dirty Scope

当前 card registration 已经开始直接从通用 scene extract 派生：

- 每个 `RenderMeshSnapshot.node_id` 会成为一张内部 card 的初始 authority
- runtime host 不再在每帧 `register_extract(...)` 时重建整个 `scene_representation`
- public Hybrid GI settings 更新只会刷新 settings/budget/debug 侧语义
- 真实 meshes/lights 则通过独立 scene-sync 步骤更新 cards、surface cache 和 voxel scene

这一点很关键，因为 milestone 1 的目标不是“把一份新的 settings DTO 塞进 runtime”，而是让 scene representation 真正开始以通用 scene extract 为真源。

### Mesh / Material / Light Change Invalidation

当前 scene sync 已经具备第一版脏化粒度：

- mesh/card 首次出现时，对应 resident page 会被标记为 dirty capture
- mesh 保持同一个 `node_id`，但 transform / model / material / tint / render-layer 等 snapshot 内容变化时，只会把对应 card/page 重新标 dirty
- directional / point / spot lights 的 scene snapshot 变化时，当前 resident pages 会整体重标 dirty，表示 direct-light seed 需要重新 capture
- voxel clipmaps 则会在 card 集合或光照集合变化时整组重标 dirty

这还不是最终“只失效空间受影响 clipmap brick”的细化实现，但它已经把 milestone 1 需要的语义固定下来：scene change 不再等价于 runtime host 全量重建，也不再是 completely stateless。

### Runtime Continuation Resolve Now Blends Exact, Ancestor, And Descendant Lineage

当前 scene-representation 虽然已经开始 scene-driven 化，但旧 probe-style runtime continuation 仍然参与最终 resolve，所以 lineage 组合规则必须稳定：

- probe 自身的 exact runtime hierarchy entry 不能再直接遮蔽 descendant continuation
- ancestor gather、exact entry、requested descendant continuation 现在会在同一轮中统一加权混合
- RGB lineage source 只在最终输出阶段做一次 support clamp，避免中途先 blend 再 clamp 导致 descendant 贡献被提前压扁
- resolve-weight lineage 也遵循同一条规则，保证 descendant continuation 不会因为 parent exact weight 非零就完全失效

这一步把 renderer resolve 和 runtime host 的 continuation 语义重新对齐，不再出现“parent exact entry 一旦非零就把 child continuation 硬切掉”的结果。

### Voxel Clipmap Budgeting

`HybridGiVoxelSceneState` 当前也开始跟随 scene card 集合变化维护 clipmap residency：

- 有 active cards 时，`voxel_budget` 决定 resident clipmap 数
- resident clipmap 会从当前 card bounds 计算 deterministic descriptor：`center` 来自 scene bounds 中心，`half_extent` 从 scene 最大跨度向上取整后按 clipmap 层级倍增
- card 集合变化会把当前 resident clipmaps 全部标记为 dirty
- 没有 active cards 时，resident clipmaps 会清空
- runtime host 级 scene registration 现在也能在测试里直接读回这些 descriptor 与 invalidation 结果，确保 scene extract -> runtime host -> voxel scene 这条链路是闭合的
- runtime host 现在还会随 resident clipmap 一起导出固定 `4x4x4` 的 `voxel_cells` occupancy/count/ownership/color payload；renderer scene-prepare 会把这份 payload 压成 `u64` occupancy mask、per-cell count/dominant-node readback、以及 unified descriptor buffer 的 owner/color authority，不再自行从 `scene_meshes` 回算这些长期真值

这仍然是 milestone 1 的 skeleton，不是最终软件 voxelization 结果；但它已经把“scene change 会驱动 voxel fallback 更新”这条状态语义固定下来。

## Current Verification

这轮已经明确通过的定向验证包括：

- split-light scene extract world roundtrip
- scene asset TOML roundtrip for point / spot light
- frame extract split-light roundtrip
- `RenderHybridGiExtract` 默认 public settings 语义
- `HybridGiInputSet` 的 Deferred / Forward+ 完整性
- `HybridGiSceneRepresentation::from_extract(...)` 对 public settings 和 internal fixture bridge 的分离
- runtime host scene-card registration from real mesh extract
- runtime host selective dirtying for material changes and full resident relight for scene-light changes
- deterministic surface-cache page-table slot reuse and card-capture atlas slot reuse
- resident page capture-slot reservation across clean-to-dirty transitions
- scene-driven card-capture request descriptors carrying `card/page/atlas/capture/bounds` truth
- runtime-host test accessors exposing those card-capture requests end-to-end
- `RenderStats` readback for scene-card / surface-cache / voxel-scene Milestone 1 counters
- unified `scene_prepare_descriptor_buffer` staging for card-capture requests, voxel clipmaps, and runtime-owned voxel cells
- fixed-layout GPU owner fallback when only runtime `dominant_card_id` changes and `radiance_rgb` stays zero
- fixed-layout GPU owner fallback reusing matched scene card-capture seed when only that card seed changes
- `update_completion.wgsl` consuming near-field scene descriptors so renderer readback changes when card / clipmap / voxel-cell scene-prepare data moves
- resolve helper lineage blending for exact parent entries plus descendant continuation
- resolve-side software voxel fallback from `hybrid_gi_scene_prepare` when no current trace support exists
- coarse clipmap-level resolve fallback surviving even when runtime omits voxel-cell payload
- resolve-side voxel fallback now preferring nonzero current-frame `scene_prepare_resources` voxel cell / clipmap samples before falling back to matched owner-card seed or pure spatial heuristic
- render-level regression coverage for irradiance / RT lighting / resolve-weight descendant continuation
- targeted `page_table_and_capture_slots` and `reuses_surface_cache_slots_after_invalidation` coverage
- targeted `card_capture_requests` coverage
- full `hybrid_gi_scene_representation` coverage including scene-bounds-driven voxel clipmap descriptors
- runtime-host coverage for scene clipmap descriptor construction and scene-clear invalidation
- targeted renderer seam coverage for:
  - scene-prepare quantization
  - collect-inputs scene-prepare passthrough
  - runtime scene-prepare voxel-cell export with deterministic `64`-cell occupancy payload per resident clipmap
  - scene-prepare atlas/capture resource snapshot readback
  - deterministic atlas/capture texel sample readback for occupied slots
  - atlas/capture samples responding to mesh tint plus directional-light changes
  - atlas/capture samples responding to point-light and spot-light changes
  - atlas/capture samples responding to material base-color differences
  - atlas/capture samples responding to material emissive differences without direct lights
  - voxel clipmap samples responding to material emissive differences without direct lights
  - voxel clipmap occupancy masks reacting to scene-mesh translation across clipmap cells
  - voxel clipmap cell radiance samples following scene-mesh translation across clipmap cells
  - voxel clipmap cell occupancy counts accumulating overlapping meshes inside the same coarse voxel cell
  - voxel clipmap occupancy/count readback honoring runtime-owned `voxel_cells` payload even when renderer-local scene meshes are absent
  - voxel clipmap occupancy/count readback staying zero when runtime omits `voxel_cells`, even if renderer-local scene meshes are present
  - voxel clipmap dominant node ids preferring the brighter overlapping contributor inside the same coarse voxel cell
  - voxel clipmap dominant RGBA samples preserving the brighter overlapping contributor separately from the aggregate coarse-cell sample
  - GPU readback reacting to near/far card-capture descriptors
  - GPU readback reacting to near/far voxel clipmaps
  - GPU readback reacting to near/far runtime `voxel_cells` while clipmap truth stays fixed
  - final GI resolve reacting to `scene_prepare` voxel-cell fallback when no trace schedule exists
  - final GI resolve reacting to `scene_prepare` voxel-clipmap fallback even when runtime voxel cells are absent
  - final GI resolve reacting to runtime scene voxel tint authority even when voxel layout stays fixed
  - final GI resolve reacting to matched scene card-capture seed when runtime voxel layout and owner stay fixed but per-cell radiance is absent
- `cargo check -p zircon_runtime --locked --lib`

当前这轮 acceptance 仍然以 Hybrid GI 自身的 targeted evidence 为主：`hybrid_gi_scene_prepare_requires_runtime_voxel_cells_for_occupancy_and_count_truth` 已经证明空 `voxel_cells` 不会再触发 renderer fallback，`hybrid_gi_scene_prepare_uses_runtime_voxel_cell_payload_without_scene_meshes` 又把这条 contract 向前推进到完整 color-and-ownership truth，证明即便 renderer 本地完全没有 scene meshes，`scene_prepare` snapshot 也会直接把 runtime `radiance_rgb` 写回 clipmap aggregate sample、per-cell sample 与 dominant sample，并把 runtime `dominant_card_id` 直接写回 dominant-node readback。`gpu_scene_prepare_descriptors_include_runtime_voxel_cells` 与 `hybrid_gi_gpu_completion_readback_changes_when_scene_voxel_cells_move_near_or_far_from_probe` 先证明 runtime-owned cell payload 已经真正进入 unified descriptor buffer 和 shader 消费链，而最新加入的 `hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_radiance_changes_with_fixed_layout`、`hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_changes_with_fixed_layout` 与 `hybrid_gi_gpu_completion_readback_changes_when_runtime_scene_voxel_owner_matches_different_card_capture_seed_with_fixed_layout` 则进一步锁死“同一 voxel 布局、只改 runtime `radiance_rgb`、只改 runtime `dominant_card_id`，或者只改该 owner 对应 card-capture seed，GPU completion readback 都必须变化”的合同，说明 GPU seam 已经开始分别消费 runtime voxel color authority、owner authority 与 matched card-capture seed authority，而不是继续只看 synthetic descriptor math。这次新增的 `hybrid_gi_resolve_uses_scene_prepare_voxel_fallback_without_current_trace_schedule` 与 `hybrid_gi_resolve_uses_scene_prepare_voxel_clipmap_fallback_without_runtime_voxel_cells` 则证明 resolve 侧已经会在没有 current trace schedule 时消费 `hybrid_gi_scene_prepare` 的 voxel truth，而且在 runtime 缺失 cell payload 时还能退回 coarse clipmap fallback，而不是继续退回纯黑。最新加入的 `hybrid_gi_resolve_uses_runtime_scene_voxel_tint_when_layout_stays_fixed`、`hybrid_gi_resolve_uses_runtime_scene_voxel_point_light_seed_when_layout_and_tint_stay_fixed` 与 `hybrid_gi_resolve_uses_runtime_scene_voxel_owner_card_capture_seed_when_layout_and_owner_stay_fixed` 又把这条 runtime voxel authority 再往前推了一步：同一套 runtime voxel 布局固定时，不管只改 scene mesh tint、只改 point-light direct seed，还是只改 matched card-capture seed，最终 GI resolve 都会跟着变化，不再把两帧压回同一个空间启发式结果。最新的 `hybrid_gi_resolve_changes_when_runtime_scene_voxel_owner_matches_scene_card_capture_material_seed_with_fixed_layout` 则补上了 resolve 侧最后一段不对称 seam：当 `card_capture_request` 布局、voxel owner 和 per-cell radiance 都保持不变时，只改 scene material truth 也必须让 final resolve 改变；这条回归现在通过，是因为 post-process 已经开始消费当前 frame 的 `scene_prepare_resources.capture_slot_rgba_samples / atlas_slot_rgba_samples`，而不是继续依赖 synthetic request RGB。当前这个 checkpoint 再补上了四条关键回归：`gpu_scene_prepare_descriptors_preserve_explicit_black_runtime_voxel_radiance`、`hybrid_gi_gpu_completion_readback_preserves_explicit_black_runtime_voxel_radiance_with_fixed_layout`、`hybrid_gi_scene_prepare_preserves_explicit_black_runtime_voxel_radiance_without_scene_meshes` 和 `runtime_explicit_black_voxel_radiance_stays_authoritative_over_owner_card_and_spatial_fallback` 证明 runtime-owned `voxel_cells` 已经能用独立 `radiance_present` 位保留显式黑色 authority，而不会再错误退回 owner-card 或 spatial heuristic。与此同时，`hybrid_gi_scene_prepare_card_capture_samples_change_with_material_roughness` 与 `hybrid_gi_scene_prepare_voxel_samples_change_with_material_metallic` 又把 card/voxel capture 从 `base_color + emissive` 的最小 seed 推到了 `base_color + emissive + roughness + metallic` 的更丰富 surface-property capture，说明 scene-prepare atlas/voxel 现在已经会对材质表面响应而不只是颜色和自发光做出可观测变化。这个 checkpoint 再往前补上了第一批真实材质纹理：`MaterialCaptureSeed / MaterialRuntime` 现在会保留 `base_color_texture / metallic_roughness_texture / emissive_texture`，`card_capture_shading.rs` 会用稳定的 scene-prepare sample UV 读取 CPU texture asset，并把 `base_color_texture` 乘进 albedo、把 `metallic_roughness_texture` 的 `G/B` 通道乘进 `roughness/metallic`、把 `emissive_texture` 乘进 emissive seed。对应地，`hybrid_gi_scene_prepare_card_capture_samples_change_with_material_base_color_texture`、`hybrid_gi_scene_prepare_card_capture_samples_change_with_material_emissive_texture`、`hybrid_gi_scene_prepare_voxel_samples_change_with_material_emissive_texture` 与 `hybrid_gi_scene_prepare_voxel_samples_change_with_material_metallic_roughness_texture` 现在都通过，说明在标量材质参数固定时，仅修改贴图内容也能稳定改变 atlas/capture/voxel scene-prepare sample，而整组 `graphics::tests::hybrid_gi_scene_prepare_resources` 也重新回到绿色。相邻的 `hybrid_gi_resolve_uses_runtime_gpu_trace_lighting_source_without_current_trace_schedule`、`hybrid_gi_resolve_uses_runtime_hierarchy_rt_lighting_without_current_trace_schedule` 和 `hybrid_gi_resolve_uses_descendant_scene_driven_runtime_rt_for_parent_probe_after_schedule_clears` 也继续保持绿色，`cargo check -p zircon_runtime --locked --lib` 通过。

## Current Limits

这仍然不是完整的 Lumen scene pipeline，当前限制需要明确记录：

- surface cache 现在虽然已经有 per-frame GPU atlas / capture RT 资源 scaffold、scene-driven direct-light seed texel write 和 sample readback，但它们还没有升级成 persistent page-table / atlas residency manager，也还没有真正从完整材质/表面属性 capture 生成 surface cache 内容
- `card_capture_requests + voxel_clipmaps + voxel_cells` 现在都已经接进 renderer，而且 unified descriptor buffer 也已经开始真实承载这三类 scene-prepare payload；`voxel_cells` 已经不只是 occupancy/count/cell-center truth，还会把 runtime `radiance_rgb` 与 `dominant_card_id` authority 直接打进 descriptor，并分别被 shader 的 color path 与 owner-fallback path 消费；owner-fallback path 本身也会优先复用 matched card-capture seed，但它仍然只是 dominant tint + split direct-light seed 的近场 bias 来源，不是完整的 voxel material/surface cache authority
- voxel scene 现在已经多了一层 runtime-owned fixed-grid `voxel_cells` occupancy/count/dominant-tint contract，再叠加 per-clipmap debug/sample seed、occupancy mask、cell-level volume-content readback、renderer-local dominant contributor ids 与 dominant contributor color truth，并且 resolve 侧已经开始在 trace miss 时把 `voxel_cells` 与 `voxel_clipmaps` 一起用作第一版软件 fallback；但它仍然是 tint-driven + spatial fallback 的 clipmap/cell lighting，不是最终软件 voxelization，也还没有进入真正的 screen-trace hit/miss 合流
- `scene_prepare_resources -> resolve` 的 renderer-side voxel sample 路径和 runtime-owned `voxel_cells` 现在都已经有显式 presence contract，显式黑色 sample / radiance authority 不会再被误当成缺失；但它们当前仍然只是 minimal radiance seed，而不是完整的 texture-backed surface cache 内容，所以 resolve miss fallback 还没有进入真正的 page-reuse / surface-property reuse 合流
- renderer-side card/voxel capture 现在已经会同时消费 `base_color + emissive + metallic + roughness` 和首版完整材质纹理集：`base_color_texture / normal_texture / metallic_roughness_texture / occlusion_texture / emissive_texture` 都已经进入 scene-prepare capture；同一条 minimal capture BRDF 现在还会尊重 `double_sided` 与 `alpha_mode(mask/blend)`，所以 backface lighting、cutout reject 与 alpha-blend 衰减不再被错误压成“所有材质都等价于 opaque + double-sided”。但采样仍然只用稳定中心 UV，capture 结果也还没有沉淀成 persistent page content
- 旧 probe / trace-region runtime path 仍然存在于迁移层，主要用于 fixture、runtime host 兼容和旧测试面
