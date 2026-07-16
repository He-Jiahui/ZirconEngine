use std::collections::BTreeMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AlphaMode, AssetUri, MaterialAsset, MeshAsset, MeshAttributeValues, MeshIndices, MeshSkinAsset,
    MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use crate::core::framework::animation::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};
use crate::core::framework::render::{
    AntiAliasSettings, GeometryExtract, GeometryPhaseInput, ProjectionMode, RenderFrameExtract,
    RenderFramework, RenderLayerSet, RenderMaterialAlphaMode, RenderMeshSnapshot,
    RenderMeshStaticState, RenderQualityProfile, RenderSkeletalPoseExtract, RenderStats,
    RenderViewportDescriptor, RenderWorldSnapshotHandle, ShaderFeatureBits,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec4};
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind,
    ResourceRecord,
};
use crate::graphics::shader::standard_material_surface_source_for_features;
use crate::graphics::WgpuRenderFramework;

use super::render_product_submit::{
    material_with_import_note, snapshot_with_projection_for_mesh_cache_tests,
};

mod morph;
#[cfg(feature = "dynamic-api")]
mod project_plugin_registry_material_passes_staged_cache;
#[cfg(feature = "dynamic-api")]
mod project_plugin_registry_staged_cache;
mod shading_model_parity;
#[cfg(feature = "dynamic-api")]
mod staged_prewarm;
mod virtual_geometry;

#[test]
fn render_product_static_mesh_second_submit_reports_pre_mesh_command_cache_reuse() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/static-cache.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    register_material_revision(&asset_manager, material_id, material_uri, "static-cache-v1");

    let framework = WgpuRenderFramework::new_for_test(asset_manager.clone()).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("static-cache-reuse")
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, static_cache_extract(material_id, 97))
        .unwrap();
    let first = framework.query_stats().unwrap();
    assert!(
        first.last_mesh_pending_static_command_cache_draw_candidate_count >= 1,
        "first submit should identify the static draw as a cache candidate",
    );
    assert!(
        first.last_mesh_pending_static_command_cache_phase_candidate_count >= 1,
        "first submit should identify at least one cacheable phase",
    );
    assert_eq!(first.last_mesh_cached_command_hit_count, 0);

    framework
        .submit_frame_extract(viewport, static_cache_extract(material_id, 98))
        .unwrap();
    let second = framework.query_stats().unwrap();
    assert!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count >= 1,
        "second submit should skip the already cached static draw before MeshDraw",
    );
    assert!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count >= 1,
        "second submit should reuse at least one cached static command phase",
    );
    assert!(
        second.last_mesh_cached_command_hit_count
            >= second.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count,
        "cache hit stats should cover every phase skipped by pre-MeshDraw extraction",
    );
    assert_eq!(second.last_mesh_command_cache_miss_count, 0);
    assert_eq!(second.last_mesh_command_rebuild_count, 0);
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_residual_material_phase_draw_count,
        0,
    );
    assert_eq!(
        second
            .last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_input_missing_draw_count,
        0,
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_residual_rebuild_rejected_draw_count,
        0,
    );
}

#[test]
fn render_product_static_mesh_material_revision_invalidates_pre_mesh_cache() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri =
        AssetUri::parse("res://materials/static-cache-material-revision.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    register_material_revision(
        &asset_manager,
        material_id,
        material_uri.clone(),
        "material-revision-v1",
    );

    let framework = WgpuRenderFramework::new_for_test(asset_manager.clone()).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("static-cache-material-invalidation")
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, static_cache_extract(material_id, 197))
        .unwrap();
    register_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "material-revision-v2",
    );
    framework
        .submit_frame_extract(viewport, static_cache_extract(material_id, 198))
        .unwrap();

    let changed = framework.query_stats().unwrap();
    assert_eq!(
        changed.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count, 0,
        "a material revision change must not skip the residual MeshDraw construction path",
    );
    assert_eq!(
        changed.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count, 0,
        "material-bound invalidation must not be reported as a pre-MeshDraw cache hit",
    );
    assert!(
        changed.last_mesh_pre_mesh_draw_static_command_cache_residual_material_phase_draw_count
            >= 1,
        "pre-MeshDraw extraction should report the material-bound residual reason",
    );
    assert_eq!(changed.last_mesh_cached_command_hit_count, 0);
    assert_eq!(changed.last_mesh_command_cache_miss_count, 0);
    assert_eq!(
        changed.last_mesh_command_cache_invalidated_transform_count,
        0
    );
    assert_eq!(
        changed.last_mesh_command_cache_invalidated_geometry_count,
        0
    );
    assert!(
        changed.last_mesh_command_cache_invalidated_material_count >= 1,
        "residual command building should observe the changed material revision",
    );
    assert!(
        changed.last_mesh_command_rebuild_count
            >= changed.last_mesh_command_cache_invalidated_material_count,
        "every material invalidation must rebuild a command with current material resources",
    );
}

#[test]
fn render_product_static_mesh_taa_reactive_mask_keeps_residual_mesh_draw_path() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/static-cache-taa-reactive.zmaterial")
        .expect("reactive material uri");
    let material_id = ResourceId::from_locator(&material_uri);
    register_taa_reactive_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "taa-reactive-v1",
        1.0,
    );

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("static-cache-taa-reactive")
                .with_screen_space_ambient_occlusion(false)
                .with_temporal_history(true)
                .with_anti_alias(true),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, static_cache_taa_extract(material_id, 297))
        .unwrap();
    let first = framework.query_stats().unwrap();
    assert!(
        first.last_mesh_pending_static_command_cache_draw_candidate_count >= 1,
        "the ordinary static phases still remain command-cache candidates",
    );
    assert_eq!(
        first.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count, 0,
        "TAA reactive material state must keep the first submit on the residual MeshDraw path",
    );
    assert_eq!(first.last_mesh_cached_command_hit_count, 0);
    assert_eq!(first.last_mesh_taa_reactive_mask_command_count, 1);

    framework
        .submit_frame_extract(viewport, static_cache_taa_extract(material_id, 298))
        .unwrap();
    let second = framework.query_stats().unwrap();
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count, 0,
        "TAA reactive material state must keep the second submit on the residual MeshDraw path",
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count, 0,
        "reactive-mask draws cannot be removed before material-bound MeshDraw construction",
    );
    assert!(
        second.last_mesh_cached_command_hit_count >= 1,
        "residual MeshDraw construction may still reuse ordinary cached static phases",
    );
    assert_eq!(second.last_mesh_command_cache_miss_count, 0);
    assert_residual_dynamic_commands_accounted(
        &second,
        "the reactive-mask command remains an uncached per-frame command",
    );
    assert_eq!(second.last_mesh_taa_reactive_mask_command_count, 1);
}

#[test]
fn render_product_static_transparent_mesh_stays_out_of_pre_mesh_cache() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri =
        AssetUri::parse("res://materials/static-cache-transparent.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    register_alpha_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "transparent-cache-v1",
        AlphaMode::Blend,
    );

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("static-cache-transparent-residual")
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, static_cache_transparent_extract(material_id, 397))
        .unwrap();
    let first = framework.query_stats().unwrap();
    assert_eq!(
        first.last_mesh_pending_static_command_cache_draw_candidate_count, 0,
        "transparent static meshes must not be advertised as static command-cache candidates",
    );
    assert_eq!(first.last_mesh_transparent_draw_count, 1);
    assert_eq!(
        first.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(first.last_mesh_cached_command_hit_count, 0);

    framework
        .submit_frame_extract(viewport, static_cache_transparent_extract(material_id, 398))
        .unwrap();
    let second = framework.query_stats().unwrap();
    assert_eq!(
        second.last_mesh_pending_static_command_cache_draw_candidate_count, 0,
        "transparent static meshes keep their camera-depth sorted dynamic command path",
    );
    assert_eq!(second.last_mesh_transparent_draw_count, 1);
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count,
        0,
    );
    assert_eq!(second.last_mesh_cached_command_hit_count, 0);
    assert_eq!(second.last_mesh_command_cache_miss_count, 0);
    assert_residual_dynamic_commands_accounted(
        &second,
        "transparent mesh commands remain dynamic so depth ordering can be rebuilt per frame",
    );
}

#[test]
fn render_product_static_skinned_mesh_stays_out_of_pre_mesh_cache() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/static-cache-skinned.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    register_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "skinned-cache-material-v1",
    );
    let mesh_uri = AssetUri::parse("res://meshes/static-cache-skinned.zmesh").unwrap();
    let mesh_id = ResourceId::from_locator(&mesh_uri);
    register_static_skinned_mesh_revision(
        &asset_manager,
        mesh_id,
        mesh_uri,
        "skinned-cache-mesh-v1",
    );
    let skeleton_uri =
        AssetUri::parse("res://animation/static-cache-skinned.skeleton.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);
    register_static_skinned_skeleton_revision(
        &asset_manager,
        skeleton_id,
        skeleton_uri,
        "skinned-cache-skeleton-v1",
    );

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("static-cache-skinned-residual")
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            static_cache_skinned_extract(material_id, mesh_id, skeleton_id, 497),
        )
        .unwrap();
    let first = framework.query_stats().unwrap();
    assert_eq!(
        first.last_mesh_pending_static_command_cache_draw_candidate_count, 0,
        "static skinned meshes must not be advertised as static command-cache candidates",
    );
    assert_eq!(first.last_mesh_skinned_draw_count, 1);
    assert_eq!(first.last_mesh_skinned_gpu_source_candidate_count, 1);
    assert_eq!(
        first.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(first.last_mesh_cached_command_hit_count, 0);

    framework
        .submit_frame_extract(
            viewport,
            static_cache_skinned_extract(material_id, mesh_id, skeleton_id, 498),
        )
        .unwrap();
    let second = framework.query_stats().unwrap();
    assert_eq!(
        second.last_mesh_pending_static_command_cache_draw_candidate_count, 0,
        "skinned GPU-source draws remain residual because their palette/source can change per frame",
    );
    assert_eq!(second.last_mesh_skinned_draw_count, 1);
    assert_eq!(second.last_mesh_skinned_gpu_source_candidate_count, 1);
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count,
        0,
    );
    assert_eq!(second.last_mesh_cached_command_hit_count, 0);
    assert_eq!(second.last_mesh_command_cache_miss_count, 0);
    assert_residual_dynamic_commands_accounted(
        &second,
        "skinned mesh commands remain dynamic so skinning bindings can be rebuilt per frame",
    );
}

fn assert_residual_dynamic_commands_accounted(stats: &RenderStats, dynamic_path_reason: &str) {
    assert!(
        stats.last_mesh_dynamic_command_count >= 1,
        "{dynamic_path_reason}",
    );
    assert!(
        stats.last_mesh_command_rebuild_count >= stats.last_mesh_dynamic_command_count,
        "dynamic residual mesh commands should be reflected in command rebuild stats",
    );
}

fn register_material_revision(
    asset_manager: &ProjectAssetManager,
    material_id: ResourceId,
    material_uri: AssetUri,
    source_hash: &str,
) {
    register_material_asset_revision(
        asset_manager,
        material_id,
        material_uri,
        source_hash,
        material_with_import_note(),
    );
}

fn register_taa_reactive_material_revision(
    asset_manager: &ProjectAssetManager,
    material_id: ResourceId,
    material_uri: AssetUri,
    source_hash: &str,
    strength: f32,
) {
    let mut material = material_with_import_note();
    material.property_values.insert(
        "taa_reactive_mask_strength".to_string(),
        toml::Value::Float(strength as f64),
    );
    register_material_asset_revision(
        asset_manager,
        material_id,
        material_uri,
        source_hash,
        material,
    );
}

fn register_alpha_material_revision(
    asset_manager: &ProjectAssetManager,
    material_id: ResourceId,
    material_uri: AssetUri,
    source_hash: &str,
    alpha_mode: AlphaMode,
) {
    let mut material = material_with_import_note();
    material.alpha_mode = alpha_mode;
    register_material_asset_revision(
        asset_manager,
        material_id,
        material_uri,
        source_hash,
        material,
    );
}

fn register_static_skinned_mesh_revision(
    asset_manager: &ProjectAssetManager,
    mesh_id: ResourceId,
    mesh_uri: AssetUri,
    source_hash: &str,
) {
    asset_manager
        .assets::<MeshAsset>()
        .insert(
            ResourceRecord::new(mesh_id, ResourceKind::Mesh, mesh_uri.clone())
                .with_source_hash(source_hash),
            static_skinned_mesh_asset(mesh_uri),
        )
        .expect("skinned mesh insert");
}

fn register_static_skinned_skeleton_revision(
    asset_manager: &ProjectAssetManager,
    skeleton_id: ResourceId,
    skeleton_uri: AssetUri,
    source_hash: &str,
) {
    asset_manager
        .assets::<AnimationSkeletonAsset>()
        .insert(
            ResourceRecord::new(skeleton_id, ResourceKind::AnimationSkeleton, skeleton_uri)
                .with_source_hash(source_hash),
            static_skinned_skeleton_asset(),
        )
        .expect("skinned skeleton insert");
}

fn register_material_asset_revision(
    asset_manager: &ProjectAssetManager,
    material_id: ResourceId,
    material_uri: AssetUri,
    source_hash: &str,
    material: MaterialAsset,
) {
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri)
                .with_source_hash(source_hash),
            material,
        )
        .expect("material insert");
}

fn registry_staged_cache_runtime_surface_source() -> String {
    let material_surface = standard_material_surface_source_for_features(
        ShaderFeatureBits::new(ShaderFeatureBits::RECEIVE_SHADOWS),
        0.0,
    );
    material_surface.source.replacen(
        &format!("fn {}(", material_surface.entry_point),
        "fn zr_material_surface(",
        1,
    )
}

fn static_cache_taa_extract(material_id: ResourceId, world: u64) -> RenderFrameExtract {
    let mut extract = static_cache_extract(material_id, world);
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract
}

fn static_cache_transparent_extract(material_id: ResourceId, world: u64) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    let mesh = static_command_cache_mesh(material_id);
    extract.geometry = GeometryExtract::from_meshes_and_phase_inputs(
        extract.view.core_pipeline,
        vec![mesh],
        vec![GeometryPhaseInput::new(
            603,
            0,
            RenderMaterialAlphaMode::Blend,
            0.0,
        )],
    );
    extract
}

fn static_cache_skinned_extract(
    material_id: ResourceId,
    mesh_id: ResourceId,
    skeleton_id: ResourceId,
    world: u64,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![static_skinned_command_cache_mesh(material_id, mesh_id)],
    );
    extract.animation_poses = vec![RenderSkeletalPoseExtract {
        entity: 703,
        skeleton: skeleton_id,
        pose: static_skinned_pose(),
    }];
    extract
}

fn static_cache_extract(material_id: ResourceId, world: u64) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![static_command_cache_mesh(material_id)],
    );
    extract
}

fn static_skinned_command_cache_mesh(
    material_id: ResourceId,
    mesh_id: ResourceId,
) -> RenderMeshSnapshot {
    let mut mesh = static_command_cache_mesh(material_id);
    mesh.node_id = 703;
    mesh.stable_instance_key = 703 << 16;
    mesh.mesh = Some(ResourceHandle::<MeshMarker>::new(mesh_id));
    mesh.static_state = RenderMeshStaticState::new(true, 11, 13);
    mesh
}

fn static_command_cache_mesh(material_id: ResourceId) -> RenderMeshSnapshot {
    // Static command reuse requires a static transform plus nonzero geometry/material revisions.
    RenderMeshSnapshot {
        node_id: 603,
        stable_instance_key: 603 << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::new(true, 1, 1),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn static_skinned_mesh_asset(mesh_uri: AssetUri) -> MeshAsset {
    let mut mesh = MeshAsset::new(
        mesh_uri,
        crate::core::framework::render::RenderMeshTopology::TriangleList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [0.25, 0.0, 0.0],
                    [0.0, 0.25, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
                MeshAttributeValues::Uint16x4(vec![[0, 0, 0, 0]; 3]),
            ),
            (
                MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
                MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 0.0]; 3]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0, 1, 2])),
    )
    .expect("static skinned mesh asset");
    mesh.skin = Some(MeshSkinAsset {
        inverse_bind_matrices: vec![identity_matrix()],
    });
    mesh
}

fn static_skinned_skeleton_asset() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("StaticCacheSkinnedSkeleton".to_string()),
        bones: vec![AnimationSkeletonBoneAsset {
            name: "root".to_string(),
            parent_index: None,
            local_translation: [0.0, 0.0, 0.0],
            local_rotation: [0.0, 0.0, 0.0, 1.0],
            local_scale: [1.0, 1.0, 1.0],
        }],
    }
}

fn static_skinned_pose() -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: vec![AnimationPoseBone {
            name: "root".to_string(),
            local_transform: Transform::default(),
        }],
    }
}

fn identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
