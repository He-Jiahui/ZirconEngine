use std::sync::Arc;

use super::super::material_capture_source::{
    HybridGiMaterialCaptureSeed, HybridGiMaterialCaptureTextureKey,
};
use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderMeshSnapshot, RenderPointLightSnapshot,
    RenderSpotLightSnapshot,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::*;

struct EmptyMaterialCaptureSource;

impl HybridGiMaterialCaptureSource for EmptyMaterialCaptureSource {
    fn material_capture_seed(&self, _id: &ResourceId) -> Option<HybridGiMaterialCaptureSeed> {
        None
    }

    fn sample_texture_rgba(
        &self,
        _texture: Option<HybridGiMaterialCaptureTextureKey>,
        _uv: [f32; 2],
    ) -> Option<Vec4> {
        None
    }
}

#[test]
fn scene_voxel_clipmap_occupancy_mask_moves_when_mesh_crosses_cells() {
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 8.0,
    };
    let left = inputs_with_mesh(clipmap.clone(), Vec3::new(-3.0, 0.0, 0.0));
    let right = inputs_with_mesh(clipmap.clone(), Vec3::new(3.0, 0.0, 0.0));

    let left_mask = scene_voxel_clipmap_occupancy_mask(&clipmap, &left);
    let right_mask = scene_voxel_clipmap_occupancy_mask(&clipmap, &right);

    assert_ne!(left_mask, right_mask);
}

fn inputs_with_mesh(
    clipmap: HybridGiPrepareVoxelClipmap,
    translation: Vec3,
) -> HybridGiPrepareExecutionInputs {
    HybridGiPrepareExecutionInputs {
        cache_entries: Vec::new(),
        resident_probe_inputs: Vec::new(),
        pending_probe_inputs: Vec::new(),
        radiance_cache_update_inputs: Vec::new(),
        radiance_cache_consume_inputs: Vec::new(),
        trace_region_inputs: Vec::new(),
        scene_card_capture_requests: Vec::new(),
        scene_surface_cache_depth_source_samples: Vec::new(),
        scene_surface_cache_page_contents: Vec::new(),
        scene_card_capture_descriptor_count: 0,
        scene_voxel_clipmaps: vec![clipmap],
        scene_voxel_cells: Vec::new(),
        scene_mesh_world_bounds: vec![(
            render_mesh_stable_instance_key(11, 0),
            RenderMeshBounds::from_min_max(
                (translation - Vec3::splat(0.5)).to_array(),
                (translation + Vec3::splat(0.5)).to_array(),
            ),
        )]
        .into(),
        scene_meshes: vec![mesh_at(translation)].into(),
        directional_lights: Vec::<RenderDirectionalLightSnapshot>::new(),
        point_lights: Vec::<RenderPointLightSnapshot>::new(),
        spot_lights: Vec::<RenderSpotLightSnapshot>::new(),
        cache_word_count: 0,
        completed_probe_word_count: 0,
        completed_trace_word_count: 0,
        irradiance_word_count: 0,
        trace_lighting_word_count: 0,
        trace_diagnostic_word_count: 0,
    }
}

#[test]
fn elongated_mesh_bounds_only_cover_intersecting_cell_rows() {
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 8.0,
    };
    let mut inputs = inputs_with_mesh(clipmap.clone(), Vec3::ZERO);
    Arc::make_mut(&mut inputs.scene_mesh_world_bounds)[0].1 =
        RenderMeshBounds::from_min_max([-7.5, -0.25, -0.25], [7.5, 0.25, 0.25]);

    let mask = scene_voxel_clipmap_occupancy_mask(&clipmap, &inputs);

    assert_eq!(mask.count_ones(), 16);
}

#[test]
fn missing_authoritative_bounds_do_not_project_voxel_cell_samples() {
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 8.0,
    };
    let mut inputs = inputs_with_mesh(clipmap.clone(), Vec3::ZERO);
    inputs.scene_mesh_world_bounds = Arc::from([]);
    let streamer = EmptyMaterialCaptureSource;

    let samples = scene_voxel_clipmap_cell_samples(&clipmap, &streamer, &inputs);

    assert!(samples
        .rgba_samples
        .iter()
        .all(|(_, rgba)| *rgba == [0, 0, 0, 0]));
    assert!(samples
        .dominant_node_ids
        .iter()
        .all(|(_, node_id)| *node_id == 0));
    assert!(samples
        .dominant_rgba_samples
        .iter()
        .all(|(_, rgba)| *rgba == [0, 0, 0, 0]));
}

#[test]
fn aggregated_voxel_cell_samples_preserve_all_legacy_projections() {
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 8.0,
    };
    let inputs = inputs_with_mesh(clipmap.clone(), Vec3::new(1.5, 0.0, 0.0));
    let streamer = EmptyMaterialCaptureSource;

    let aggregated = scene_voxel_clipmap_cell_samples(&clipmap, &streamer, &inputs);

    assert_eq!(
        aggregated.rgba_samples,
        scene_voxel_clipmap_cell_rgba_samples(&clipmap, &streamer, &inputs)
    );
    assert_eq!(
        aggregated.dominant_node_ids,
        scene_voxel_clipmap_cell_dominant_node_ids(&clipmap, &streamer, &inputs)
    );
    assert_eq!(
        aggregated.dominant_rgba_samples,
        scene_voxel_clipmap_cell_dominant_rgba_samples(&clipmap, &streamer, &inputs)
    );
}

#[test]
fn aggregated_voxel_cell_samples_keep_the_larger_node_id_on_equal_strength() {
    let clipmap = HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 8.0,
    };
    let translation = Vec3::ZERO;
    let mut inputs = inputs_with_mesh(clipmap.clone(), translation);
    let higher_node_id = 19;
    let mut meshes = inputs.scene_meshes.to_vec();
    meshes.push(mesh_at_node(higher_node_id, translation));
    inputs.scene_meshes = meshes.into();
    let mut bounds = inputs.scene_mesh_world_bounds.to_vec();
    bounds.push((
        render_mesh_stable_instance_key(higher_node_id, 0),
        RenderMeshBounds::from_min_max(
            (translation - Vec3::splat(0.5)).to_array(),
            (translation + Vec3::splat(0.5)).to_array(),
        ),
    ));
    bounds.sort_unstable_by_key(|(stable_instance_key, _)| *stable_instance_key);
    inputs.scene_mesh_world_bounds = bounds.into();
    let streamer = EmptyMaterialCaptureSource;

    let samples = scene_voxel_clipmap_cell_samples(&clipmap, &streamer, &inputs);

    assert!(samples
        .dominant_node_ids
        .iter()
        .filter(|(_, node_id)| *node_id != 0)
        .all(|(_, node_id)| *node_id == higher_node_id));
}

fn mesh_at(translation: Vec3) -> RenderMeshSnapshot {
    mesh_at_node(11, translation)
}

fn mesh_at_node(node_id: u64, translation: Vec3) -> RenderMeshSnapshot {
    let transform = Transform::from_translation(translation);
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "builtin://material/default",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::from_transform_static(true),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            is_static: true,
            ..RendererCommon::default()
        },
    }
}
