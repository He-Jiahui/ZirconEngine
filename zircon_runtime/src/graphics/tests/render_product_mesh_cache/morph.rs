use std::collections::BTreeMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{
    AssetUri, MaterialAsset, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset,
    MeshSkinAsset, MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use crate::core::framework::animation::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};
use crate::core::framework::render::{
    AntiAliasSettings, CameraRenderDescriptor, CapturedFrame, GeometryExtract, ProjectionMode,
    RenderCameraClear, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMaterialLightingModel, RenderMeshSnapshot, RenderMeshStaticState,
    RenderMotionBlurSettings, RenderPipelineHandle, RenderQualityProfile,
    RenderSkeletalPoseExtract, RenderViewportDescriptor, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Quat, Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind,
    ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;

use super::super::render_product_submit::{
    material_with_import_note, snapshot_with_projection_for_mesh_cache_tests,
};

mod direct_velocity;
mod skinned_velocity;
mod velocity_png;

#[test]
fn render_product_direct_mesh_active_morph_weights_use_gpu_morphed_source() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material_uri = AssetUri::parse("res://materials/product-direct-morph.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    register_material_revision(
        &asset_manager,
        material_id,
        material_uri,
        "product-direct-morph-material-v1",
    );
    let mesh_uri = AssetUri::parse("res://meshes/product-direct-morph.zmesh").unwrap();
    let mesh_id = ResourceId::from_locator(&mesh_uri);
    register_direct_morph_mesh_revision(
        &asset_manager,
        mesh_id,
        mesh_uri,
        "product-direct-morph-mesh-v1",
    );

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("product-direct-morph-gpu-source")
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, direct_morph_extract(material_id, mesh_id, 2301))
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_mesh_draw_count, 1);
    assert_eq!(stats.last_mesh_gpu_morphed_source_draw_count, 1);
    assert_eq!(stats.last_mesh_gpu_skinned_morphed_source_draw_count, 0);
    assert_eq!(
        stats.last_mesh_dynamic_geometry_draw_count, 1,
        "active morph weights must keep the draw on a dynamic GPU morph geometry source",
    );
}

#[test]
fn render_product_direct_mesh_gpu_morph_matches_cpu_baked_reference_pixels() {
    let material_uri =
        AssetUri::parse("res://materials/product-direct-morph-parity.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    let gpu_mesh_uri = AssetUri::parse("res://meshes/product-direct-morph-gpu.zmesh").unwrap();
    let gpu_mesh_id = ResourceId::from_locator(&gpu_mesh_uri);
    let cpu_mesh_uri = AssetUri::parse("res://meshes/product-direct-morph-cpu.zmesh").unwrap();
    let cpu_mesh_id = ResourceId::from_locator(&cpu_mesh_uri);

    let gpu = capture_morph_parity_frame(
        material_id,
        material_uri.clone(),
        gpu_mesh_id,
        gpu_mesh_uri,
        "product-direct-morph-parity-material-gpu",
        true,
        2401,
    );
    let cpu = capture_morph_parity_frame(
        material_id,
        material_uri,
        cpu_mesh_id,
        cpu_mesh_uri,
        "product-direct-morph-parity-material-cpu",
        false,
        2402,
    );

    assert_eq!(
        gpu.stats.last_mesh_gpu_morphed_source_draw_count, 1,
        "GPU reference must exercise the morphed GPU geometry source"
    );
    assert_eq!(
        cpu.stats.last_mesh_gpu_morphed_source_draw_count, 0,
        "CPU-baked reference must not use the GPU morph source"
    );
    assert_rgba_frames_nearly_equal(&gpu.frame, &cpu.frame, 2, 24);
}

#[test]
fn render_product_skinned_mesh_gpu_morph_matches_cpu_baked_reference_pixels() {
    let material_uri =
        AssetUri::parse("res://materials/product-skinned-morph-parity.zmaterial").unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    let gpu_mesh_uri = AssetUri::parse("res://meshes/product-skinned-morph-gpu.zmesh").unwrap();
    let gpu_mesh_id = ResourceId::from_locator(&gpu_mesh_uri);
    let cpu_mesh_uri = AssetUri::parse("res://meshes/product-skinned-morph-cpu.zmesh").unwrap();
    let cpu_mesh_id = ResourceId::from_locator(&cpu_mesh_uri);
    let skeleton_uri =
        AssetUri::parse("res://animation/product-skinned-morph.skeleton.zranim").unwrap();
    let skeleton_id = ResourceId::from_locator(&skeleton_uri);

    let gpu = capture_skinned_morph_parity_frame(
        material_id,
        material_uri.clone(),
        gpu_mesh_id,
        gpu_mesh_uri,
        skeleton_id,
        skeleton_uri.clone(),
        "product-skinned-morph-parity-material-gpu",
        true,
        2501,
    );
    let cpu = capture_skinned_morph_parity_frame(
        material_id,
        material_uri,
        cpu_mesh_id,
        cpu_mesh_uri,
        skeleton_id,
        skeleton_uri,
        "product-skinned-morph-parity-material-cpu",
        false,
        2502,
    );

    assert_eq!(
        gpu.stats.last_mesh_gpu_skinned_morphed_source_draw_count, 1,
        "GPU reference must exercise the skinned morphed GPU geometry source"
    );
    assert_eq!(
        gpu.stats.last_mesh_skinned_gpu_skinning_draw_count, 1,
        "GPU reference must keep skinning on the shader path"
    );
    assert_eq!(
        cpu.stats.last_mesh_gpu_skinned_morphed_source_draw_count, 0,
        "CPU-baked reference must not use the skinned morphed GPU source"
    );
    assert_eq!(
        cpu.stats.last_mesh_skinned_gpu_skinning_draw_count, 1,
        "CPU-baked reference should still exercise shader skinning"
    );
    assert_rgba_frames_nearly_equal(&gpu.frame, &cpu.frame, 2, 24);
}

struct MorphParityCapture {
    frame: CapturedFrame,
    stats: crate::core::framework::render::RenderStats,
}

fn capture_morph_parity_frame(
    material_id: ResourceId,
    material_uri: AssetUri,
    mesh_id: ResourceId,
    mesh_uri: AssetUri,
    material_hash: &str,
    gpu_morphed: bool,
    world: u64,
) -> MorphParityCapture {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    register_unlit_material_revision(&asset_manager, material_id, material_uri, material_hash);
    if gpu_morphed {
        register_direct_morph_mesh_revision(
            &asset_manager,
            mesh_id,
            mesh_uri,
            "product-direct-morph-parity-gpu-mesh",
        );
    } else {
        register_cpu_baked_morph_mesh_revision(
            &asset_manager,
            mesh_id,
            mesh_uri,
            "product-direct-morph-parity-cpu-mesh",
        );
    }

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(128, 128)))
        .unwrap();
    framework
        .set_quality_profile(viewport, morph_parity_quality_profile())
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            direct_morph_parity_extract(material_id, mesh_id, world, gpu_morphed),
        )
        .unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("morph parity frame should be capturable");
    let stats = framework.query_stats().unwrap();
    framework.destroy_viewport(viewport).unwrap();

    MorphParityCapture { frame, stats }
}

fn capture_skinned_morph_parity_frame(
    material_id: ResourceId,
    material_uri: AssetUri,
    mesh_id: ResourceId,
    mesh_uri: AssetUri,
    skeleton_id: ResourceId,
    skeleton_uri: AssetUri,
    material_hash: &str,
    gpu_morphed: bool,
    world: u64,
) -> MorphParityCapture {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    register_unlit_material_revision(&asset_manager, material_id, material_uri, material_hash);
    register_skinned_morph_skeleton_revision(
        &asset_manager,
        skeleton_id,
        skeleton_uri,
        "product-skinned-morph-parity-skeleton",
    );
    if gpu_morphed {
        register_skinned_morph_mesh_revision(
            &asset_manager,
            mesh_id,
            mesh_uri,
            "product-skinned-morph-parity-gpu-mesh",
        );
    } else {
        register_skinned_cpu_baked_morph_mesh_revision(
            &asset_manager,
            mesh_id,
            mesh_uri,
            "product-skinned-morph-parity-cpu-mesh",
        );
    }

    let framework = WgpuRenderFramework::new_for_test(asset_manager).unwrap();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(128, 128)))
        .unwrap();
    framework
        .set_quality_profile(viewport, morph_parity_quality_profile())
        .unwrap();

    framework
        .submit_frame_extract(
            viewport,
            skinned_morph_parity_extract(material_id, mesh_id, skeleton_id, world, gpu_morphed),
        )
        .unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("skinned morph parity frame should be capturable");
    let stats = framework.query_stats().unwrap();
    framework.destroy_viewport(viewport).unwrap();

    MorphParityCapture { frame, stats }
}

fn register_material_revision(
    asset_manager: &ProjectAssetManager,
    material_id: ResourceId,
    material_uri: AssetUri,
    source_hash: &str,
) {
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri)
                .with_source_hash(source_hash),
            material_with_import_note(),
        )
        .expect("material insert");
}

fn register_unlit_material_revision(
    asset_manager: &ProjectAssetManager,
    material_id: ResourceId,
    material_uri: AssetUri,
    source_hash: &str,
) {
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri)
                .with_source_hash(source_hash),
            unlit_material_with_import_note(),
        )
        .expect("unlit material insert");
}

fn register_direct_morph_mesh_revision(
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
            direct_morph_mesh_asset(mesh_uri),
        )
        .expect("direct morph mesh insert");
}

fn register_cpu_baked_morph_mesh_revision(
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
            cpu_baked_morph_mesh_asset(mesh_uri),
        )
        .expect("CPU-baked morph mesh insert");
}

fn register_skinned_morph_mesh_revision(
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
            skinned_morph_mesh_asset(mesh_uri),
        )
        .expect("skinned morph mesh insert");
}

fn register_skinned_cpu_baked_morph_mesh_revision(
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
            skinned_cpu_baked_morph_mesh_asset(mesh_uri),
        )
        .expect("skinned CPU-baked morph mesh insert");
}

fn register_skinned_morph_skeleton_revision(
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
            skinned_morph_skeleton_asset(),
        )
        .expect("skinned morph skeleton insert");
}

fn direct_morph_extract(
    material_id: ResourceId,
    mesh_id: ResourceId,
    world: u64,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![direct_morph_mesh_snapshot(material_id, mesh_id)],
    );
    extract
}

fn direct_morph_parity_extract(
    material_id: ResourceId,
    mesh_id: ResourceId,
    world: u64,
    gpu_morphed: bool,
) -> RenderFrameExtract {
    let mut camera = ViewportCameraSnapshot {
        projection_mode: ProjectionMode::Orthographic,
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
        ..ViewportCameraSnapshot::default()
    };
    camera.apply_viewport_size(UVec2::new(128, 128));
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(camera.projection_mode),
    );
    let mut descriptor = CameraRenderDescriptor::from_camera_payload(Some(2401), camera);
    descriptor.clear = RenderCameraClear::Color(Vec4::ZERO);
    extract.view.select_camera_descriptor(descriptor);
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![direct_morph_parity_mesh_snapshot(
            material_id,
            mesh_id,
            gpu_morphed,
        )],
    );
    extract
}

fn skinned_morph_parity_extract(
    material_id: ResourceId,
    mesh_id: ResourceId,
    skeleton_id: ResourceId,
    world: u64,
    gpu_morphed: bool,
) -> RenderFrameExtract {
    let mut extract = direct_morph_parity_extract(material_id, mesh_id, world, gpu_morphed);
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![skinned_morph_parity_mesh_snapshot(
            material_id,
            mesh_id,
            gpu_morphed,
        )],
    );
    extract.animation_poses = vec![RenderSkeletalPoseExtract {
        entity: SKINNED_MORPH_PARITY_NODE_ID,
        skeleton: skeleton_id,
        pose: skinned_morph_pose(),
    }];
    extract
}

fn direct_morph_mesh_snapshot(material_id: ResourceId, mesh_id: ResourceId) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 2301,
        stable_instance_key: 2301 << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: Some(ResourceHandle::<MeshMarker>::new(mesh_id)),
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: vec![1.0],
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn direct_morph_parity_mesh_snapshot(
    material_id: ResourceId,
    mesh_id: ResourceId,
    gpu_morphed: bool,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 2401,
        stable_instance_key: 2401 << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: Some(ResourceHandle::<MeshMarker>::new(mesh_id)),
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: if gpu_morphed { vec![1.0] } else { Vec::new() },
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn skinned_morph_parity_mesh_snapshot(
    material_id: ResourceId,
    mesh_id: ResourceId,
    gpu_morphed: bool,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: SKINNED_MORPH_PARITY_NODE_ID,
        stable_instance_key: SKINNED_MORPH_PARITY_NODE_ID << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: Some(ResourceHandle::<MeshMarker>::new(mesh_id)),
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: if gpu_morphed { vec![1.0] } else { Vec::new() },
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn morph_parity_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("product-direct-morph-parity")
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
        .with_clustered_lighting(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
}

fn morph_velocity_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("product-direct-morph-velocity")
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(true)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(true)
        .with_clustered_lighting(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(false)
        .with_virtual_geometry(false)
}

fn unlit_material_with_import_note() -> MaterialAsset {
    let mut material = material_with_import_note();
    material.base_color = [1.0, 1.0, 1.0, 1.0];
    material.emissive = [0.0, 0.0, 0.0];
    material.property_values.insert(
        "lighting_model".to_string(),
        toml::Value::String(RenderMaterialLightingModel::Unlit.to_string()),
    );
    material
}

fn direct_morph_mesh_asset(mesh_uri: AssetUri) -> MeshAsset {
    let mut mesh = morph_base_mesh_asset(mesh_uri, [[0.0, 0.0, 0.0]; 3]);
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Lift".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(MORPH_POSITION_DELTAS.to_vec()),
        )]),
    }];
    mesh
}

fn cpu_baked_morph_mesh_asset(mesh_uri: AssetUri) -> MeshAsset {
    morph_base_mesh_asset(mesh_uri, MORPH_POSITION_DELTAS)
}

fn skinned_morph_mesh_asset(mesh_uri: AssetUri) -> MeshAsset {
    let mut mesh = direct_morph_mesh_asset(mesh_uri);
    add_skinned_morph_attributes(&mut mesh);
    mesh
}

fn skinned_cpu_baked_morph_mesh_asset(mesh_uri: AssetUri) -> MeshAsset {
    let mut mesh = cpu_baked_morph_mesh_asset(mesh_uri);
    add_skinned_morph_attributes(&mut mesh);
    mesh
}

fn add_skinned_morph_attributes(mesh: &mut MeshAsset) {
    mesh.attributes.insert(
        MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
        MeshAttributeValues::Uint16x4(vec![[0, 0, 0, 0]; 3]),
    );
    mesh.attributes.insert(
        MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
        MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 0.0]; 3]),
    );
    mesh.skin = Some(MeshSkinAsset {
        inverse_bind_matrices: vec![identity_matrix()],
    });
}

fn skinned_morph_skeleton_asset() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("ProductSkinnedMorphSkeleton".to_string()),
        bones: vec![AnimationSkeletonBoneAsset {
            name: "root".to_string(),
            parent_index: None,
            local_translation: Vec3::ZERO.to_array(),
            local_rotation: Quat::IDENTITY.to_array(),
            local_scale: Vec3::ONE.to_array(),
        }],
    }
}

fn skinned_morph_pose() -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: vec![AnimationPoseBone {
            name: "root".to_string(),
            local_transform: Transform::identity()
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        }],
    }
}

const MORPH_BASE_POSITIONS: [[f32; 3]; 3] = [[-0.8, -0.8, 0.0], [0.8, -0.8, 0.0], [-0.8, 0.8, 0.0]];
const MORPH_POSITION_DELTAS: [[f32; 3]; 3] = [[0.6, 0.2, 0.0], [0.0, 0.6, 0.0], [0.6, 0.0, 0.0]];
const SKINNED_MORPH_PARITY_NODE_ID: u64 = 2501;

fn morph_base_mesh_asset(mesh_uri: AssetUri, position_deltas: [[f32; 3]; 3]) -> MeshAsset {
    MeshAsset::new(
        mesh_uri,
        crate::core::framework::render::RenderMeshTopology::TriangleList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    add_position_delta(MORPH_BASE_POSITIONS[0], position_deltas[0]),
                    add_position_delta(MORPH_BASE_POSITIONS[1], position_deltas[1]),
                    add_position_delta(MORPH_BASE_POSITIONS[2], position_deltas[2]),
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
                MESH_ATTRIBUTE_COLOR.to_string(),
                MeshAttributeValues::Float32x4(vec![
                    [1.0, 0.2, 0.1, 1.0],
                    [0.1, 1.0, 0.2, 1.0],
                    [0.2, 0.1, 1.0, 1.0],
                ]),
            ),
        ]),
        Some(MeshIndices::U32(vec![0, 1, 2])),
    )
    .expect("morph parity mesh asset")
}

fn add_position_delta(position: [f32; 3], delta: [f32; 3]) -> [f32; 3] {
    [
        position[0] + delta[0],
        position[1] + delta[1],
        position[2] + delta[2],
    ]
}

fn identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn assert_rgba_frames_nearly_equal(
    actual: &CapturedFrame,
    expected: &CapturedFrame,
    per_channel_tolerance: u8,
    max_mismatched_pixels: usize,
) {
    assert_eq!(actual.width, expected.width, "frame width mismatch");
    assert_eq!(actual.height, expected.height, "frame height mismatch");
    assert_eq!(
        actual.rgba.len(),
        expected.rgba.len(),
        "rgba length mismatch"
    );

    let mut mismatch_count = 0usize;
    let mut first_mismatch = None;
    for (pixel_index, (actual_pixel, expected_pixel)) in actual
        .rgba
        .chunks_exact(4)
        .zip(expected.rgba.chunks_exact(4))
        .enumerate()
    {
        if pixel_diff_exceeds(actual_pixel, expected_pixel, per_channel_tolerance) {
            mismatch_count += 1;
            first_mismatch.get_or_insert((
                pixel_index,
                actual_pixel.to_vec(),
                expected_pixel.to_vec(),
            ));
        }
    }

    assert!(
        mismatch_count <= max_mismatched_pixels,
        "GPU morph frame differs from CPU-baked reference by {mismatch_count} pixels; first mismatch={first_mismatch:?}"
    );
}

fn assert_scene_velocity_readback_nonzero(
    stats: &crate::core::framework::render::RenderStats,
    viewport_size: UVec2,
    label: &str,
) {
    let report = stats.last_scene_velocity_readback_report;

    assert!(report.available, "{label}: scene-velocity readback missing");
    assert_eq!(
        report.size, viewport_size,
        "{label}: readback size mismatch"
    );
    assert_eq!(
        report.byte_len,
        (viewport_size.x * viewport_size.y * 4) as usize,
        "{label}: readback byte length mismatch",
    );
    assert!(
        report.nonzero_pixel_count > 0,
        "{label}: expected nonzero scene-velocity pixels"
    );
}

fn pixel_diff_exceeds(actual: &[u8], expected: &[u8], tolerance: u8) -> bool {
    actual
        .iter()
        .zip(expected)
        .any(|(actual, expected)| actual.abs_diff(*expected) > tolerance)
}
