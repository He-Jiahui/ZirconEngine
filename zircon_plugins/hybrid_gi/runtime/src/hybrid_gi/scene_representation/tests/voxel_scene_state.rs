use super::*;
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderLayerSet,
    RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Transform, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

#[test]
fn persisted_surface_cache_page_samples_override_voxel_cells_by_owner_card_id_not_page_id() {
    let mut voxel_cells = vec![HybridGiPrepareVoxelCell {
        clipmap_id: 0,
        cell_index: 7,
        occupancy_count: 1,
        dominant_card_id: 11,
        radiance_present: false,
        radiance_rgb: [0, 0, 0],
    }];

    apply_surface_cache_page_contents_to_voxel_cells(
        &mut voxel_cells,
        &[(21, 11, 0, 0, [10, 20, 30, 255], [40, 50, 60, 255])],
        &[],
    );

    assert_eq!(voxel_cells[0].radiance_rgb, [40, 50, 60]);
    assert!(
        voxel_cells[0].radiance_present,
        "expected voxel radiance reuse to match the persisted owner card id even when the persisted page id differs"
    );
}

#[test]
fn scene_prepare_voxel_cell_readback_merges_into_voxel_cells() {
    let mut state = HybridGiVoxelSceneState::default();
    let readback_cell = HybridGiPrepareVoxelCell {
        clipmap_id: 2,
        cell_index: 5,
        occupancy_count: 4,
        dominant_card_id: 11,
        radiance_present: true,
        radiance_rgb: [32, 48, 64],
    };

    state.apply_scene_prepare_voxel_cells(&[readback_cell]);

    assert_eq!(state.voxel_cells_snapshot(), vec![readback_cell]);
    assert_eq!(state.scene_revision(), 1);
}

#[test]
fn scene_prepare_voxel_cell_readback_survives_stable_scene_synchronization() {
    let mut state = HybridGiVoxelSceneState::default();
    let cards = vec![card_descriptor(11, Vec3::ZERO)];
    state.synchronize(&cards, &[], &[], &[], &[], &[], 1, true);
    let base_cell = state
        .voxel_cells_snapshot()
        .into_iter()
        .find(|cell| cell.occupancy_count > 0)
        .expect("card should occupy at least one voxel cell");
    let readback_cell = HybridGiPrepareVoxelCell {
        radiance_present: true,
        radiance_rgb: [96, 48, 24],
        ..base_cell
    };

    state.apply_scene_prepare_voxel_cells(&[readback_cell]);
    state.synchronize(&cards, &[], &[], &[], &[], &[], 1, false);

    let persisted = state
        .voxel_cells_snapshot()
        .into_iter()
        .find(|cell| {
            cell.clipmap_id == readback_cell.clipmap_id
                && cell.cell_index == readback_cell.cell_index
        })
        .expect("stable scene sync should keep the readback cell key");
    assert_eq!(persisted, readback_cell);
}

fn card_descriptor(card_id: u32, center: Vec3) -> HybridGiCardDescriptor {
    let node_id = card_id as u64;
    let transform = Transform::from_translation(center).with_scale(Vec3::splat(2.0));
    HybridGiCardDescriptor::new(
        card_id,
        RenderMeshSnapshot {
            node_id,
            stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
            transform_revision: render_mesh_transform_revision(&transform),
            transform,
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/hgi-card.obj",
            )),
            mesh: None,
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/hgi-card.mat",
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
        },
        center,
        1.0,
    )
}
