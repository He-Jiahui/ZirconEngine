---
related_code:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/parameter_map.rs
  - zircon_runtime/src/core/framework/animation/parameter_value.rs
  - zircon_runtime/src/core/framework/animation/playback_settings.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/physics/settings.rs
  - zircon_runtime/src/core/framework/physics/world_sync_state.rs
  - zircon_runtime/src/animation/animation_interface.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/asset/assets/model.rs
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_obj.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/pipeline/types.rs
  - zircon_runtime/src/physics/physics_interface.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/components/schedule.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/component_access.rs
  - zircon_runtime/src/scene/world/property_access/mod.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/animation/service_types.rs
  - zircon_runtime/src/animation/sequence_runtime.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_from_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_animation_skeleton_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/physics/service_types.rs
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/model.rs
  - zircon_runtime/src/asset/tests/assets/physics_material.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/animation/tests.rs
  - zircon_runtime/src/physics/tests.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - zircon_runtime/tests/scene_world_driver_runtime_contract.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/catalog_snapshot.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/record_to_facade.rs
  - zircon_editor/src/ui/workbench/reflection/activity_actions/inspector_actions.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration/action_route.rs
  - zircon_editor/src/ui/workbench/reflection/animation_route.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/host/editor_manager_animation_editor.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/slint_host/viewport/test_render_framework.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/fake_render_framework.rs
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/src/tests/host/asset_references.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
implementation_files:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/parameter_map.rs
  - zircon_runtime/src/core/framework/animation/parameter_value.rs
  - zircon_runtime/src/core/framework/animation/playback_settings.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/physics/settings.rs
  - zircon_runtime/src/core/framework/physics/world_sync_state.rs
  - zircon_runtime/src/animation/animation_interface.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/asset/assets/model.rs
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_obj.rs
  - zircon_runtime/src/asset/importer/ingest/primitive_from_indexed_mesh.rs
  - zircon_runtime/src/asset/pipeline/types.rs
  - zircon_runtime/src/physics/physics_interface.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/components/schedule.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/level_system_render_extract.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/scene/world/component_access.rs
  - zircon_runtime/src/scene/world/property_access/mod.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/animation/service_types.rs
  - zircon_runtime/src/animation/sequence_runtime.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_from_mesh_vertex.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_vertex_layout.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_load_animation_skeleton_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/physics/service_types.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/catalog_snapshot.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/record_to_facade.rs
  - zircon_editor/src/ui/workbench/reflection/activity_actions/inspector_actions.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration/action_route.rs
  - zircon_editor/src/ui/workbench/reflection/animation_route.rs
  - zircon_editor/src/ui/animation_editor/session.rs
  - zircon_editor/src/ui/host/animation_editor_sessions/save.rs
  - zircon_editor/src/ui/host/editor_manager_animation_editor.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/slint_host/viewport/test_render_framework.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/fake_render_framework.rs
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/src/tests/host/asset_references.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
plan_sources:
  - user: 2026-04-20 继续正在runtime/editor/framework实现完整的物理和动画系统
  - user: 2026-04-20 physics和animation吸收进runtime
  - user: 2026-04-22 继续
  - .codex/plans/Physics + Full Animation Support 新计划.md
  - .codex/plans/Physics  Full Animation Support Plan.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
tests:
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/assets/model.rs
  - zircon_runtime/src/asset/tests/assets/physics_material.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/animation/tests.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/physics/tests.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - zircon_runtime/tests/scene_world_driver_runtime_contract.rs
  - cargo test -p zircon_runtime --test scene_world_driver_runtime_contract --locked
  - cargo test -p zircon_runtime --test scene_world_driver_runtime_contract --locked level_render_extract_carries_cached_clip_pose_for_skinned_entity -- --exact --nocapture
  - cargo test -p zircon_runtime importer_preserves_gltf_skinning_channels_on_model_vertices --locked -- --nocapture
  - cargo test -p zircon_runtime model_asset_toml_roundtrip_preserves_virtual_geometry_payload --locked -- --nocapture
  - cargo test -p zircon_runtime gpu_mesh_vertex_conversion_preserves_skinning_channels --locked -- --nocapture
  - cargo test -p zircon_runtime skin_model_primitive_rotates_weighted_vertex_around_joint_bind_origin --locked -- --nocapture
  - cargo test -p zircon_runtime animation_manager_samples_clip_pose_against_skeleton --locked -- --nocapture
  - cargo test -p zircon_runtime apply_sequence_to_world_resolves_track_paths_and_updates_scene_properties --locked -- --nocapture
  - cargo test -p zircon_runtime world_driver --locked
  - cargo test -p zircon_runtime --test scene_world_driver_runtime_contract --locked
  - cargo test -p zircon_runtime directory_project_scene_renders_non_background_frame_with_gizmo_overlay --locked -- --nocapture
  - cargo test -p zircon_editor --locked --lib ui::slint_host::app::helpers::tests::derive_animation_assets_from_model_source_preserves_project_asset_ids_across_reimport_with_gltf_buffer_sidecars -- --exact --nocapture
  - cargo test -p zircon_editor --locked --lib ui::slint_host::app::helpers::tests::derive_animation_assets_from_model_source_writes_stable_sibling_skeleton_and_clip_files -- --exact --nocapture
  - cargo test -p zircon_editor --locked tests::host::asset_references::editor_asset_manager_tracks_scene_animation_and_physics_references -- --exact --nocapture
  - cargo test -p zircon_editor --locked slint_asset_pointer --target-dir target/codex-asset-filter-validation -- --nocapture
  - cargo test -p zircon_editor --locked asset_surface_templates_map_no_preview_physics_and_animation_assets_to_specific_icons --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked binding_dispatch --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked editor_event::runtime --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked slint_asset_pointer --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked workbench_reflection_ -- --nocapture
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - cargo test -p zircon_editor --lib save_sequence_session_persists_track_changes_and_clears_dirty -- --nocapture
  - cargo test -p zircon_editor --lib save_graph_session_persists_parameter_changes_and_clears_dirty -- --nocapture
  - cargo test -p zircon_editor --lib save_state_machine_session_persists_entry_state_changes_and_clears_dirty -- --nocapture
  - cargo test -p zircon_editor --lib editor_manager_saves_animation_sequence_editor_session_and_clears_dirty_metadata -- --nocapture
  - zircon_runtime/src/scene/tests/world_basics.rs
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b scene_asset_toml_roundtrip_preserves_physics_and_animation_components -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b scene_assets_roundtrip_asset_bound_physics_and_animation_components -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b world_project_roundtrip_preserves_physics_and_animation_components -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b apply_sequence_to_world_resolves_track_paths_and_updates_scene_properties -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b animation_manager_persists_playback_settings_to_runtime_config -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation physics_manager_syncs_world_snapshot_and_exposes_queries_and_contacts --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation animation_manager_evaluates_graphs_and_parameter_overrides --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation animation_manager_resolves_state_machine_transitions_from_parameters --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation animation_manager_samples_clip_pose_against_skeleton --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b physics_manager_persists_settings_to_runtime_config -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b physics_manager_tracks_fixed_step_accumulator_per_world -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime --locked --offline core::framework::tests --target-dir D:/cargo-targets/zircon-workspace-hard-cutover -- --nocapture
  - cargo test -p zircon_runtime --locked --offline physics_and_animation_managers_resolve_through_framework_facades --target-dir D:/cargo-targets/zircon-workspace-hard-cutover -- --nocapture
doc_type: module-detail
---

# Runtime Physics And Animation Assets

## Purpose

这份文档记录已经真正吸收进 `zircon_runtime` 的物理/动画底层能力。当前范围已经不止资产与 scene 组件，还包括第一波执行层：

- 共享属性路径与 scene property runtime access
- sequence property application
- animation / physics manager 的 runtime config 持久化与 fixed-step bookkeeping

当前完成的是三条底层真源：

- `zircon_runtime::core::framework::{scene, physics, animation}` 提供共享 DTO、路径契约与 manager-facing settings
- `zircon_runtime::{asset, scene}` 负责 physics material、五类动画资产、scene 组件、以及 `SceneAsset <-> World` 的 typed roundtrip
- `zircon_runtime::{animation, physics}` 提供 sequence/property runtime 与 manager state/runtime config 接口

同时，这两块 framework root 也已经完成结构收口：

- `zircon_runtime/src/core/framework/physics/mod.rs`
- `zircon_runtime/src/core/framework/animation/mod.rs`

现在都只保留 `mod` 声明和受控 `pub use`；原先堆在 root `mod.rs` 里的 DTO、trait、default 行为和路径 helper 都已经下沉到 folder-backed 子文件。

这仍然不等于完整 physics backend、Jolt 驱动、skeleton graph runtime 或 editor sequence UI 都已落地，但 v1 已经不再只是“资产壳 + DTO 壳”。

## Ownership

当前物理和动画的稳定 ownership 固定为：

- `zircon_runtime::core::framework::scene`
  - `EntityPath`
  - `ComponentPropertyPath`
  - `ScenePropertyValue`
  - `ScenePropertyEntry`
- `zircon_runtime::core::framework::physics`
  - `PhysicsCombineRule`
  - `PhysicsMaterialMetadata`
  - `PhysicsSettings`
  - `PhysicsBackendStatus`
  - `PhysicsWorldSyncState`
  - `PhysicsRayCastQuery`
  - `PhysicsContactEvent`
- `zircon_runtime::core::framework::animation`
  - `AnimationTrackPath`
  - `AnimationParameterValue`
  - `AnimationPlaybackSettings`
  - `AnimationGraphEvaluation`
  - `AnimationStateMachineEvaluation`
  - `AnimationPoseOutput`
- `zircon_runtime::asset::assets`
  - `PhysicsMaterialAsset`
  - `AnimationSkeletonAsset`
  - `AnimationClipAsset`
  - `AnimationSequenceAsset`
  - `AnimationGraphAsset`
  - `AnimationStateMachineAsset`
  - `SceneAsset` 上的 physics/animation entity component DTO
- `zircon_runtime::scene`
  - 运行时权威 `World`
  - scene 组件存储、typed getter/setter、project JSON roundtrip
- `zircon_runtime::animation`
  - sequence track sampling
  - property-path based scene mutation
  - animation playback settings manager surface
- `zircon_runtime::physics`
  - physics settings manager surface
  - per-world fixed-step accumulator bookkeeping
  - backend status / world sync / ray-cast / contact fallback contract

这里的 structure 规则也一起固定下来：

- `core::framework::physics` 与 `core::framework::animation` 内部现在都和 `input/`、`render/`、`scene/` 一样采用 folder-backed subtree
- 顶层 `zircon_runtime/src/physics/mod.rs` 与 `zircon_runtime/src/animation/mod.rs` 也降成结构入口
- `PhysicsConfig` / `AnimationConfig` 进入 `config.rs`，module registration 与 service-name 常量进入 `module.rs`
- root `mod.rs` 不再允许重新吸收 DTO、trait、default impl 或 parse helper
- 后续新增 physics/animation contract 时，应该继续进入子文件，而不是回到 umbrella root

这里已经不再走旧的独立 `zircon_framework / zircon_manager / zircon_scene` crate 路径。

## Asset Kinds

当前 runtime 认可的 physics/animation 资产种类是：

- `PhysicsMaterial`
- `AnimationSkeleton`
- `AnimationClip`
- `AnimationSequence`
- `AnimationGraph`
- `AnimationStateMachine`

格式约束保持不变：

- physics material: `*.physics_material.toml`
- animation binary assets: `*.skeleton.zranim` / `*.clip.zranim` / `*.sequence.zranim` / `*.graph.zranim` / `*.state_machine.zranim`

动画资产仍然是版本化二进制 envelope，`AnimationSequenceAsset` 继续通过 `EntityPath + ComponentPropertyPath` 派生 canonical `AnimationTrackPath`。

## Framework Contract Subtrees

这轮之后，framework contract 形状已经显式固定：

- `core/framework/physics/`
  - `combine_rule.rs`
  - `material_metadata.rs`
  - `simulation_mode.rs`
  - `settings.rs`
  - `backend_state.rs`
  - `backend_status.rs`
  - `world_step_plan.rs`
  - `body_type.rs`
  - `collider_shape.rs`
  - `joint_type.rs`
  - `body_sync_state.rs`
  - `collider_sync_state.rs`
  - `joint_sync_state.rs`
  - `material_sync_state.rs`
  - `world_sync_state.rs`
  - `ray_cast_query.rs`
  - `ray_cast_hit.rs`
  - `contact_event.rs`
  - `manager.rs`
- `core/framework/animation/`
  - `parameter_value.rs`
  - `parameter_map.rs`
  - `track_path.rs`
  - `track_path_error.rs`
  - `playback_settings.rs`
  - `graph_clip_instance.rs`
  - `graph_evaluation.rs`
  - `state_machine_evaluation.rs`
  - `pose_source.rs`
  - `pose_bone.rs`
  - `pose_output.rs`
  - `manager.rs`

这里的目标不是把文件拆散，而是让 physics/animation framework contract 真正变成可持续增长的 owner subtree：

- `AnimationTrackPath` 的 parse/split 逻辑与 error 类型进入显式 owner file
- physics world sync / query / contact contract 不再和 backend settings/enum 声明共住一个 root
- root `mod.rs` 只负责结构声明和公开面，不再承载行为实现

## Model Import Derived Animation Assets

当前模型导入链没有把 runtime importer 改成 multi-output。相反，editor 在 `ImportModel` 流程里对 `gltf/glb` 做派生输出：

- 同目录 sibling skeleton: `*.skeleton.zranim`
- 同目录 sibling clip: `*.<clip>.clip.zranim`

关键约束如下：

- 生成路径稳定，重复导入同一模型时 locator 保持不变
- 生成文件落在 project `assets/` 内，再走现有单输出 `import_asset(...)` 链
- clip 内部对 skeleton 的引用直接指向派生 sibling locator，而不是额外的临时 ID
- `ProjectManager::scan_and_import()` 会忽略 glTF 外部 buffer sidecar `.bin`
  - 这些文件是模型源辅助物，不是独立 runtime 资产；否则项目重导入会在真正处理派生 `.zranim` 前就因为 unsupported format 失败
- 现有 editor/runtime 联合回归测试已经钉住：重复执行 `derive_animation_assets_from_model_source(...)` 再调用 `scan_and_import()` 时，派生 skeleton/clip 的 `.meta.toml` `asset_uuid` 和 registry `AssetId` 都保持稳定

这让 scene / graph / state-machine 对 clip 和 skeleton 的引用可以继续依赖现有 sidecar meta 稳定 UUID 规则，而不需要改写 `ImportedAsset` 或 `ProjectManager::scan_and_import()`。

## Scene Component Surface

`zircon_runtime::scene::components` 现在已经有第一波 physics/animation 组件：

- physics
  - `RigidBodyComponent`
  - `ColliderComponent`
  - `JointComponent`
- animation
  - `AnimationSkeletonComponent`
  - `AnimationPlayerComponent`
  - `AnimationSequencePlayerComponent`
  - `AnimationGraphPlayerComponent`
  - `AnimationStateMachinePlayerComponent`

这些组件都进入了三层权威数据面：

- `World` 内部 `HashMap<EntityId, Component>`
- `SceneNode` cache
- `NodeRecord`

因此它们现在能通过统一的 runtime world 查询、项目 JSON roundtrip、和 scene asset roundtrip 被观察到，而不是只停在 asset DTO。

## Scene Asset Mapping

`zircon_runtime/src/asset/assets/scene.rs` 现在对 entity 暴露了这些新字段：

- `rigid_body`
- `collider`
- `joint`
- `animation_skeleton`
- `animation_player`
- `animation_sequence_player`
- `animation_graph_player`
- `animation_state_machine_player`

设计约束如下：

- 所有新字段都带 `#[serde(default)]`
  - 旧 scene TOML 不会因为缺少这些字段而失效
- `TransformAsset` 现在有 identity `Default`
  - collider local transform 可以稳定省略
- collider 允许两种材质来源
  - `material: AssetReference`
  - `material_override: PhysicsMaterialMetadata`
- graph/state-machine 参数统一走 `BTreeMap<String, AnimationParameterValue>`
  - 这样序列化顺序稳定，editor/runtime roundtrip 可比对

## World Access And Mutation

`zircon_runtime::scene::world::component_access` 新增了 typed API：

- getters
  - `rigid_body(...)`
  - `collider(...)`
  - `joint(...)`
  - `animation_skeleton(...)`
  - `animation_player(...)`
  - `animation_sequence_player(...)`
  - `animation_graph_player(...)`
  - `animation_state_machine_player(...)`
- setters
  - `set_rigid_body(...)`
  - `set_collider(...)`
  - `set_joint(...)`
  - `set_animation_skeleton(...)`
  - `set_animation_player(...)`
  - `set_animation_sequence_player(...)`
  - `set_animation_graph_player(...)`
  - `set_animation_state_machine_player(...)`

这些 setter 只做最小 runtime 校验：

- entity 必须存在
- joint 不能连回自己

其余 physics/backend 级校验还没有下沉到这里。

## Shared Property Paths And Runtime Access

`zircon_runtime::core::framework::scene` 现在不只提供 `EntityPath + ComponentPropertyPath` DTO，还补齐了 runtime property surface：

- `ScenePropertyValue`
- `ScenePropertyEntry`

`zircon_runtime::scene::world::property_access` 则把这组 DTO 接到运行时世界：

- `entity_path(entity)` 从 hierarchy/name 反推稳定 `EntityPath`
- `resolve_entity_path(path)` 走 canonical 路径查回实体
- `property(entity, path)` 读取统一属性值
- `set_property(entity, path, value)` 通过路径回写组件属性
- `property_entries(entity)` 枚举 inspector / sequence 共用的属性条目

当前第一波 runtime 覆盖面已经包含：

- scene 基础组件
  - `Name`
  - `Hierarchy.parent`
  - `Transform.translation/rotation/scale`
  - `Active.enabled`
  - `RenderLayer.mask`
  - `Mobility.kind`
  - `Camera.*`
  - `MeshRenderer.*`
  - `DirectionalLight.*`
- physics 组件
  - `RigidBody.*`
  - `Collider.*`
  - `Joint.*`
- animation 组件
  - `AnimationSkeleton.skeleton`
  - `AnimationPlayer.*`
  - sequence/graph/state-machine player 的 clip、speed、time、weight、looping、playing 以及参数字典

这组路径 runtime access 是 editor inspector、asset track path 和 runtime sequence apply 的共同基础，而不是 animation 自己再走一条私有反射链。

## Sequence Runtime

`zircon_runtime::animation::sequence_runtime` 现在提供了第一波 sequence property runtime：

- `apply_sequence_to_world(world, sequence, time_seconds, looping)`
- `AnimationSequenceApplyReport`
- `AnimationChannelAsset::sample(time_seconds)`

当前行为约束：

- binding 先通过 `EntityPath` 找到 world entity
- track 再通过 `ComponentPropertyPath` 走 `World::set_property(...)`
- sequence sample time 现在先走统一的 runtime resolve
  - `looping = false` 时，时间会被夹到 `[0, duration_seconds]`
  - `looping = true` 时，超出 duration 的时间会按 `rem_euclid(duration_seconds)` 回绕
- Step 通道返回前一个 key
- Hermite 通道使用 cubic Hermite 标量/向量采样
- quaternion 通道当前走归一化 `slerp`

因此 `AnimationSequenceAsset` 已经可以直接驱动第一波 scene/physics/animation 组件属性，而不是停留在只可保存不可执行的资产格式。

## Runtime Manager State

runtime-owned physics / animation 模块现在也有了真正的 manager 状态，而不是只有 facade trait：

- `DefaultAnimationManager`
  - 从 `ANIMATION_PLAYBACK_CONFIG_KEY` 读取 `AnimationPlaybackSettings`
  - `store_playback_settings(...)` 会同时更新 runtime 内存态和 foundation config
  - `parameter_defaults(...)` / `parameter_value(...)` / `set_parameter(...)` 统一 graph/state-machine 参数面
  - `evaluate_graph(...)` 返回合并默认参数后的 clip 输出列表
  - `evaluate_state_machine(...)` 按当前状态和条件计算迁移与目标 graph
  - `sample_clip_pose(...)` 按 skeleton + clip 生成局部骨骼 pose
- `DefaultPhysicsManager`
  - 从 `PHYSICS_SETTINGS_CONFIG_KEY` 读取 `PhysicsSettings`
  - `store_settings(...)` 会同时更新 runtime 内存态和 foundation config
  - `backend_status()` 统一报告 requested backend、active backend、feature gate 和降级原因
  - `advance_clock(world, delta_seconds)` 维护每个 `WorldHandle` 的 fixed-step accumulator
  - `PhysicsTickPlan` 返回本帧 `steps / step_seconds / remaining_seconds`
  - `sync_world(...)` / `synchronized_world(...)` 保存当前 world 的中性 physics snapshot
  - `ray_cast(...)` / `drain_contacts(...)` 提供 contract-level query/contact 输出

`advance_clock(...)` 当前已经做了浮点边界保护，避免 `fixed_hz` 离散步进在临界帧长下少算 step。这样 physics module 至少已经有了后续 pre-sync / simulate / writeback 调度真正需要的 runtime bookkeeping。

## Scene Schedule

scene 默认阶段顺序已经按当前计划固定为：

- `PreUpdate`
- `FixedUpdate`
- `Update`
- `LateUpdate`
- `RenderExtract`

也就是说，physics fixed-step bookkeeping 先于 update / late-update 的 animation 参数写入阶段，而 render extract 始终在这两类运行时写入之后。

## Scene Tick Runtime

`zircon_runtime::scene::LevelSystem::tick(...)` 现在已经会通过 `WorldDriver` 实际推进第一波 physics / animation runtime，而不是只把 world 暴露给外部 manager：

- physics
  - 规划当前 world 的 fixed-step 计划
  - 同步当前 world snapshot 到 physics manager
  - drain contact event
  - 把 `PhysicsWorldStepPlan` 和 `PhysicsContactEvent` 缓存在 level runtime state
- animation
  - 推进 `AnimationPlayerComponent` / `AnimationSequencePlayerComponent` 组件时钟
  - 解析并应用 asset-backed sequence property track
  - 对 clip / graph / state machine 生成 `AnimationPoseOutput` 并缓存到 level runtime state

这条 tick 主干里，`looping` 现在已经不再只停在 scene component 字段上，而是会被真正传进采样层：

- clip player 把 `AnimationPlayerComponent.looping` 传给 `sample_clip_pose(...)`
- sequence player 把 `AnimationSequencePlayerComponent.looping` 传给 `apply_sequence_to_world(...)`
- graph / state-machine 不再默认一律回绕；它们会复用 dominant clip instance 自带的 `looping`
- 因此 non-looping clip / graph / state-machine 在 overshoot 后会钉住最后一帧，looping sequence 仍会按 duration 回绕
- 当前 paused 语义没有一起改动
  - paused graph/state-machine 仍不会发出 pose
  - paused state-machine 仍不会推进或回写新的 `active_state`

graph/state-machine 这次也正式接到了同一条 tick 主干：

- graph player 会在 tick 中加载 `AnimationGraphAsset`
- runtime 通过 `evaluate_graph(...)` 取 dominant clip 后采样 pose
- state-machine player 会在 tick 中加载 `AnimationStateMachineAsset`
- runtime 通过 `evaluate_state_machine(...)` 计算 `active_state` 和目标 graph，再采样 dominant clip pose
- `AnimationStateMachinePlayerComponent.active_state` 会被 tick 回写到 world

为了让 graph/state-machine 连续多帧播放不再每帧重采样同一个 `delta_seconds`，`LevelSystem` 现在还会维护 runtime-owned graph/state-machine playback clock。由于这两类 scene 组件当前还没有和 clip/sequence 对齐的持久 `time_seconds` 字段，时钟先放在 level runtime state，而不是写回组件本身。

## Render Extract Bridge

这轮又把 level-owned animation pose cache 正式接到了 shared render extract 边界：

- `LevelSystem` 在 `build_render_frame_extract(...)` 时会读取 tick 缓存的 `AnimationPoseOutput`
- 只有真正还挂着 mesh 和 `AnimationSkeletonComponent` 的 entity 会进入这条 render seam
- render extract 现在会额外携带 `entity + skeleton + pose` 的 animation pose 列表
- `World::build_render_frame_extract(...)` 仍保持中性，不会自己伪造 animation pose

这一步的意义不是“已经做完 skinning”，而是把 animation runtime 的输出从 level 内部缓存推进到 render-side contract：

- runtime tick 继续负责算 pose
- render extract 开始负责携带 pose 到渲染侧
- 后续 mesh skinning / shader consume 将直接建立在这条 extract seam 上，而不是重新回头读 scene component 或再造一条私有缓存线

## Skinned Model Resource Surface

这轮又把 skinned mesh 真正缺失的下层资源面补到了 runtime 里：

- `MeshVertex` 不再只有 `position/normal/uv`
  - 现在会稳定携带 `joint_indices: [u16; 4]`
  - 以及 `joint_weights: [f32; 4]`
- `ModelAsset` / `ModelPrimitiveAsset` 的 TOML roundtrip 会保留这两组 skinning 通道
- glTF importer 现在会读取 primitive 上的 `JOINTS_0` / `WEIGHTS_0`
  - 这些通道不会再在导入时直接丢失
- OBJ 和 builtin mesh 继续走零权重默认值
  - 因此现有非 skinned 模型资产不需要额外迁移

GPU 资源侧也一起对齐到了同一条数据面：

- `GpuMeshVertex` 现在带同样的 `joint_indices` / `joint_weights`
- mesh pipeline vertex layout 新增了：
  - `@location(3)` -> `Uint16x4`
  - `@location(4)` -> `Float32x4`
- `GpuMeshResource::from_asset(...)` 和相关 hash/order signature 也会把 skinning 通道一起纳入

这还不等于 shader 已经做完骨骼变形，但它解决了一个更底层的问题：渲染侧终于开始有资格拿到 skinned vertex 必需的 joint/weight 数据，而不是在 asset import 那一层就被抹平。

## Raster Skinned Mesh Consumption

render extract seam 和 skinned vertex resource surface 现在已经接成一条真正可消费的渲染路径，而不是停在“边界上有 pose，但 mesh draw 不看它”的状态：

- mesh draw builder 现在会先检查 `RenderFrameExtract.animation_poses`
- 如果当前 mesh entity 同时带有：
  - cached `AnimationPoseOutput`
  - 可加载的 `AnimationSkeletonAsset`
  - 可回读的 CPU `ModelAsset` primitive
  - 那么该 entity 会改走 CPU skinning raster fallback
- skinning helper 会：
  - 从 skeleton bind-local transform 重建 bind-world matrix
  - 从 render extract pose 重建 posed-world matrix
  - 计算 `posed_world * inverse(bind_world)` joint matrix
  - 用 vertex `joint_indices / joint_weights` 对 position 和 normal 做线性混合
- skinned primitive 会在 draw build 阶段生成临时 GPU mesh resource
  - 不复用静态 virtual-geometry indirect path
  - `virtual_geometry` payload 会在这条 override primitive 上被清空，显式退回 raster-only draw

这条路径的设计边界是刻意收紧的：

- 只有真正拿到 pose+skeleton+CPU primitive 的 entity 才会进入 CPU skinning
- 如果 skeleton 或 model asset 无法加载，draw builder 会保留原来的静态 mesh 路径
- current virtual geometry prepare/indirect submission 仍然只适合静态 mesh
  - animated/skinned entity 当前不会走 VG indirect draw ref
  - 因此这轮完成的是“raster path 能消费 animation pose”，不是“VG path 已支持 skinned mesh”

## Roundtrip Guarantees

当前已经通过测试钉住的行为：

- scene TOML roundtrip
  - physics/animation component DTO 不丢字段
- `SceneAsset <-> World`
  - asset reference 正确解析成 typed `ResourceHandle`
  - 再保存时还能回写到正确的 `res://...` locator
- `World` project JSON roundtrip
  - component maps、`SceneNode` cache、以及 getter API 都能恢复同一份数据
- shared property path
  - `EntityPath` / `ComponentPropertyPath` 可以跨 world 查询、路径写回和序列应用保持同一 canonical target
- runtime manager state
  - physics / animation settings 会同步进入 runtime config store
  - physics fixed-step accumulator 按 `WorldHandle` 分桶追踪
  - physics backend unavailable 会显式降级成 status，而不是隐式失败
  - physics world sync / ray cast / contact DTO 可以在无真实 backend 时继续跑 contract 验证
  - animation graph / state-machine / clip pose evaluator 能在 runtime 内直接执行
  - `LevelSystem::tick(...)` 会缓存 clip / graph / state-machine pose 输出
  - `LevelSystem::build_render_frame_extract(...)` 会把 skinned mesh entity 的 cached pose 投影到 render extract
  - skinned glTF primitive 的 `JOINTS_0` / `WEIGHTS_0` 会保留到 `ModelAsset`、`MeshVertex` 和 `GpuMeshVertex`
  - render-side mesh draw builder 会在 pose+skeleton+CPU primitive 齐备时对 skinned mesh 做 CPU skinning，并以 raster fallback 上传临时 GPU mesh
  - graph / state-machine 连续 tick 会累积 runtime playback clock
  - paused graph / state-machine player 当前不会进入 tick-owned graph/state-machine pose/update 分支
  - non-looping clip / graph / state-machine overshoot 会 clamp 到最后一帧
  - looping sequence overshoot 会 wrap 到 sequence duration 内的新 sample time

这意味着 physics/animation 现在已经进入 runtime 的 scene authority 与最小执行链，而不是停留在“只有 asset kind，没有实体组件”的空壳状态。

## Editor Asset Reference Analysis

editor asset manager 现在会把 scene 里的 physics/animation 组件资产一起纳入 direct-reference 分析，而不是只识别 mesh：

- `SceneColliderAsset.material`
- `SceneAnimationSkeletonAsset.skeleton`
- `SceneAnimationPlayerAsset.clip`
- `SceneAnimationSequencePlayerAsset.sequence`
- `SceneAnimationGraphPlayerAsset.graph`
- `SceneAnimationStateMachinePlayerAsset.state_machine`

这些引用全部走 `zircon_editor::ui::host::editor_asset_manager::manager::reference_analysis` 的 `ImportedAsset::Scene` 分支，进入与 graph/state-machine 相同的一套 reference graph。结果是：

- scene 资产的 `direct_reference_uuids` / `direct_references` 会同时展示 physics material、skeleton、clip、sequence、graph、state machine
- `catalog_snapshot.direct_reference_uuids` 会优先规范化成项目 catalog 内真实 `asset_uuid`
  - 不再直接暴露 `AssetReference::from_locator(...)` 生成的 locator-derived UUID
- `AnimationClip` 的 `referenced_by` 不再只来自 `AnimationGraph`，scene 直接播放器也会回指它
- `AnimationGraph` 的 `referenced_by` 不再只来自 state machine，scene graph player 也会回指它
- `AnimationStateMachine` 现在会在 scene state-machine player 挂载时被 editor 反向引用图捕获

这一步保证了 editor catalog、asset details 和后续 authoring 入口都建立在 runtime 同源的 scene 组件资产引用上，而不是另维护一套 editor-only 的补丁引用表。

## Inspector Animation Track Route

workbench reflection 的 inspector activity 现在也暴露动画轨道创建动作，但没有新增 inspector-only payload：

- action id: `create_animation_track`
- binding symbol: `AnimationCommand.CreateTrack`
- route registration: `zircon_editor::ui::workbench::reflection::animation_route`

这条路径的约束是刻意和现有 animation authoring pipeline 对齐的：

- inspector remote-call 只传一个字符串参数 `track_path`
- route stub 直接构造 `AnimationCommand::CreateTrack`
- binding normalization / dispatch 继续复用现有 animation binding 分支
- `track_path` 仍然走共享 `zircon_runtime::core::framework::animation::AnimationTrackPath::parse(...)`

因此 inspector 发起的“加轨道”不会再引入第二套 inspector mapping，也不会绕过 runtime 已接受的 canonical property path 模型。现有 workbench reflection 回归已经钉住：

- inspector 节点会公开 `create_animation_track`
- 该 action 可被远程调用并带有 route id
- 在 sequence editor 已打开时，通过 inspector 调这个 action 会真正把 canonical `AnimationTrackPath` 加入动画会话

## Asset Browser Kind Filters

physics / animation 资产虽然不会各自打开第三个专用 editor，但现在 generic asset surfaces 已经正式暴露这些 kind filter，而不是只能靠搜索字符串碰运气：

- `AssetsActivityPane` 和 `AssetBrowserPane` 都保留统一 `SetKindFilter` generic control route
- 两个 surface 现在都各自补齐了：
  - `PhysicsMaterial`
  - `AnimationSkeleton`
  - `AnimationClip`
  - `AnimationSequence`
  - `AnimationGraph`
  - `AnimationStateMachine`
- 这些 chip 直接回传 runtime/editor 已认可的 canonical kind 字符串
  - 没有再引入 UI-only alias 或第二套 animation/physics 过滤枚举
- 现有 host Slint 回归已经钉住：上面六类过滤入口在 activity 和 browser 两个 surface 中都必须各出现一次

这让计划里“`AnimationClip`、`AnimationSkeleton`、`PhysicsMaterial` 继续走现有 asset browser/details/preview，不单独做第三个动画 view 或 physics material 专用 editor”这一条不再只是路由层成立，而是连现有通用浏览面板的作者入口也真正可用。

这一轮又把无预览资产的图标回退链补齐到了同一完成线：

- `assets.slint` 的 activity/browser item thumbnail、list fallback、selection preview、details preview 现在都会显式识别 `PhysicsMaterial`
- 同一条回退链也覆盖 `AnimationSkeleton`、`AnimationClip`、`AnimationSequence`、`AnimationGraph`、`AnimationStateMachine`
- `chrome.slint` 新增了 `physics-material`、`animation-sequence`、`animation-graph` 这组 icon key 到 ionicon SVG 的固定映射

因此 physics / animation 资产即便没有 preview artifact，也不会再退回通用 `console` 占位图标，而是落到稳定、可识别的 kind-specific 视觉语义。

为了让这条 editor 回归真正能执行到断言，本轮还补齐了共享 render 契约扩展后的默认值路径：

- `zircon_runtime::ui::runtime_ui::runtime_ui_manager::empty_scene_snapshot(...)` 现在会给 `RenderSceneSnapshot.virtual_geometry_debug` 明确填 `None`
- `zircon_editor::scene::viewport::render_packet::build_render_packet(...)` 现在会给 `SceneViewportExtractRequest.virtual_geometry_debug` 明确填 `None`

这不是 physics / animation 资产新能力本身，但它避免了上层 asset/browser Slint 测试在编译阶段被共享 render DTO 漏补字段直接截断。

同一轮 editor acceptance sweep 里，`RenderFramework` 新增的 `query_virtual_geometry_debug_snapshot()` 也已经补到了 Slint viewport 的测试替身上：

- `zircon_editor::ui::slint_host::viewport::test_render_framework::TestRenderFramework`
- `zircon_editor::ui::slint_host::viewport::tests::fake_render_framework::FakeRenderFramework`

两个测试框架都显式返回 `Ok(None)`，这样 `binding_dispatch`、`editor_event::runtime` 和相关 viewport host 测试会继续通过同一条共享 render framework trait 路径，而不是因为测试替身落后于契约扩展而在编译期断掉。

## Animation Editor Save Path

animation editor 这条 authoring 链现在已经不再停在“只能把 session 标成 dirty”：

- `zircon_editor::ui::animation_editor::AnimationEditorSession::save()`
  - 会把底层 `AnimationSequenceAsset` / `AnimationGraphAsset` / `AnimationStateMachineAsset` 序列化回当前 `asset_path`
  - 只有底层写盘成功后才会清掉 dirty bit
- `zircon_editor::ui::host::animation_editor_sessions::save`
  - 把 save 能力接到 host registry
  - 如果源文件位于项目 `<project>/assets` 下，会推导成 `res://...` 并触发 `asset_manager.import_asset(...)`
  - 随后同步 workbench metadata，让已打开标签页的 dirty 状态和 payload path 与磁盘一致
- `zircon_editor::ui::host::EditorManager::save_animation_editor(...)`
  - 对上提供稳定 manager 入口，而不是让调用方直接越过 host/session 边界写文件

这条保存链的边界也保持刻意收紧：

- 当前只有底层动画资产文档会序列化落盘
- 当前帧、timeline 可见范围、选中 span、playback 开关和速度仍然是 editor-local 状态
- 因此它解决的是 animation authoring 文档持久化，而不是完整 pane UI 状态快照

这轮定向验证已经钉住四条直接证据：

- sequence session 保存后，新建 track 会真实写回 `.sequence.zranim`
- graph session 保存后，parameter 默认值变更会真实写回 `.graph.zranim`
- state-machine session 保存后，entry state 变更会真实写回 `.state_machine.zranim`
- host save 会清掉 workbench dirty metadata，同时保留原来的 payload path

## Not Done Yet

这份文档刻意不把未实现层包装成已完成：

- physics backend 驱动、fixed-step simulate/writeback、query/event、Jolt 接入
- Householder-based tangent fitting / key reduction editor math
- skinned mesh 的 shader-side joint palette / GPU skinning 路径
- skinned mesh 的 virtual geometry prepare / indirect draw / cluster culling 支持
- inspector canonical property model 与 sequence editor 的统一 authoring surface

当前仍未完成的是完整 backend 驱动接入、scene writeback 到真实刚体世界、以及 skinned path 的 GPU/VG 正式化。底层共享合同、fallback query/contact、graph/state-machine evaluator、clip pose sampling、level tick 内的 graph/state-machine runtime clock、level -> render extract 的 animation pose seam、skinned vertex 的 joint/weight runtime resource surface、以及 raster path 对 animation pose 的实际 mesh deformation 消费都已经进入 runtime 主干，不再属于未落地空壳。
