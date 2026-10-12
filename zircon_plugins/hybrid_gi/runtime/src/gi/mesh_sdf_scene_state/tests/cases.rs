use super::*;
use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfClipmapBounds, HybridGiMeshSdfAssetState, HybridGiMeshSdfFallbackReason,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, CastShadowsMode,
    RenderLayerSet, RenderMeshBounds, RenderMeshSnapshot, RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::framework::scene::Mobility;
use zircon_runtime::core::math::{Quat, Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::graphics::{RuntimePrepareMeshSdfDeformationReason, RuntimePrepareMeshSdfSeed};

#[test]
fn mesh_sdf_object_uses_imported_bounds_and_combines_instance_material_flags() {
    let mut mesh = mesh_at(7, Vec3::new(10.0, 20.0, 30.0));
    mesh.transform = mesh
        .transform
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(2.0, 3.0, 4.0));
    mesh.transform_revision = render_mesh_transform_revision(&mesh.transform);
    mesh.mobility = Mobility::Dynamic;
    mesh.common.cast_shadows = CastShadowsMode::ShadowsOnly;
    let local_bounds = RenderMeshBounds::from_min_max([-1.0, -2.0, -0.5], [3.0, 2.0, 0.5]);

    let object = HybridGiMeshSdfObject::from_sources(
        &mesh,
        local_bounds,
        1,
        0,
        HybridGiMeshSdfAssetState::default(),
        HybridGiMeshSdfMaterialFlags {
            casts_shadows: true,
            emissive: true,
        },
        &[],
    );

    assert_eq!(object.bounds().center, [10.0, 22.0, 30.0]);
    assert!(!object.flags().visible);
    assert!(object.flags().movable);
    assert!(object.flags().casts_shadow);
    assert!(object.flags().emissive);
    assert!(object.flags().indirect_while_hidden);
}

#[test]
fn clipmap_influence_is_conservative_sorted_and_excludes_disabled_objects() {
    let clipmaps = [
        HybridGiGlobalSdfClipmapBounds::new(9, Vec3::new(20.0, 0.0, 0.0), 2.0),
        HybridGiGlobalSdfClipmapBounds::new(3, Vec3::ZERO, 4.0),
        HybridGiGlobalSdfClipmapBounds::new(5, Vec3::new(4.5, 0.0, 0.0), 1.0),
    ];
    let local_bounds = RenderMeshBounds::from_min_max([-1.0; 3], [1.0; 3]);
    let enabled = HybridGiMeshSdfObject::from_sources(
        &mesh_at(1, Vec3::new(3.5, 0.0, 0.0)),
        local_bounds,
        1,
        0,
        HybridGiMeshSdfAssetState::default(),
        HybridGiMeshSdfMaterialFlags::default(),
        &clipmaps,
    );
    let mut disabled_mesh = mesh_at(2, Vec3::ZERO);
    disabled_mesh.common.enabled = false;
    let disabled = HybridGiMeshSdfObject::from_sources(
        &disabled_mesh,
        local_bounds,
        1,
        0,
        HybridGiMeshSdfAssetState::default(),
        HybridGiMeshSdfMaterialFlags::default(),
        &clipmaps,
    );

    assert_eq!(enabled.influenced_clipmap_ids(), &[3, 5]);
    assert!(disabled.influenced_clipmap_ids().is_empty());
}

#[test]
fn scene_state_orders_objects_by_stable_instance_key() {
    let bounds = RenderMeshBounds::from_min_max([-0.5; 3], [0.5; 3]);
    let objects = [3_u64, 1, 2].map(|node_id| {
        HybridGiMeshSdfObject::from_sources(
            &mesh_at(node_id, Vec3::ZERO),
            bounds,
            1,
            0,
            HybridGiMeshSdfAssetState::default(),
            HybridGiMeshSdfMaterialFlags::default(),
            &[],
        )
    });
    let mut state = HybridGiMeshSdfSceneState::default();

    state.synchronize(objects);

    assert!(state
        .objects()
        .windows(2)
        .all(|pair| pair[0].stable_instance_key() < pair[1].stable_instance_key()));
}

#[test]
fn object_change_reports_previous_and_next_dirty_regions_once() {
    let local_bounds = RenderMeshBounds::from_min_max([-0.5; 3], [0.5; 3]);
    let mut state = HybridGiMeshSdfSceneState::default();
    let initial = HybridGiMeshSdfObject::from_sources(
        &mesh_at(1, Vec3::ZERO),
        local_bounds,
        1,
        0,
        HybridGiMeshSdfAssetState::default(),
        HybridGiMeshSdfMaterialFlags::default(),
        &[],
    );
    let initial_report = state.synchronize([initial.clone()]);
    assert_eq!(initial_report.dirty_regions(), &[initial.bounds()]);
    let moved = HybridGiMeshSdfObject::from_sources(
        &mesh_at(1, Vec3::new(4.0, 0.0, 0.0)),
        local_bounds,
        1,
        0,
        HybridGiMeshSdfAssetState::default(),
        HybridGiMeshSdfMaterialFlags::default(),
        &[],
    );

    let moved_report = state.synchronize([moved.clone()]);

    assert_eq!(
        moved_report.dirty_regions(),
        &[initial.bounds(), moved.bounds()]
    );
    assert!(state.synchronize([moved]).dirty_regions().is_empty());
}

#[test]
fn runtime_mesh_sdf_seed_keeps_ready_missing_and_invalid_states_typed() {
    let ready = std::sync::Arc::<[zircon_runtime::asset::MeshSdfAsset]>::from([]);
    assert!(matches!(
        HybridGiMeshSdfAssetState::from_runtime(RuntimePrepareMeshSdfSeed::Ready(ready)),
        HybridGiMeshSdfAssetState::Ready(_)
    ));
    assert!(matches!(
        HybridGiMeshSdfAssetState::from_runtime(RuntimePrepareMeshSdfSeed::Missing {
            primitive_count: 2,
            payload_count: 1,
        }),
        HybridGiMeshSdfAssetState::VoxelFallback(HybridGiMeshSdfFallbackReason::Missing {
            primitive_count: 2,
            payload_count: 1,
        })
    ));
    assert!(matches!(
        HybridGiMeshSdfAssetState::from_runtime(RuntimePrepareMeshSdfSeed::Invalid {
            primitive_index: 3,
            error: zircon_runtime::asset::MeshSdfValidationError::InvalidDimensions,
        }),
        HybridGiMeshSdfAssetState::VoxelFallback(HybridGiMeshSdfFallbackReason::Invalid {
            primitive_index: 3,
            error: zircon_runtime::asset::MeshSdfValidationError::InvalidDimensions,
        })
    ));
}

#[test]
fn skinning_fallback_expands_to_all_active_clipmap_bounds() {
    let clipmaps = [
        HybridGiGlobalSdfClipmapBounds::new(0, Vec3::new(-8.0, 0.0, 0.0), 4.0),
        HybridGiGlobalSdfClipmapBounds::new(1, Vec3::new(8.0, 0.0, 0.0), 4.0),
    ];
    let object = HybridGiMeshSdfObject::from_sources(
        &mesh_at(21, Vec3::ZERO),
        RenderMeshBounds::from_min_max([-0.5; 3], [0.5; 3]),
        1,
        7,
        HybridGiMeshSdfAssetState::from_runtime(RuntimePrepareMeshSdfSeed::Deforming(
            RuntimePrepareMeshSdfDeformationReason::Skinning,
        )),
        HybridGiMeshSdfMaterialFlags::default(),
        &clipmaps,
    );

    assert_eq!(object.bounds().min, [-12.0, -4.0, -4.0]);
    assert_eq!(object.bounds().max, [12.0, 4.0, 4.0]);
    assert_eq!(object.influenced_clipmap_ids(), &[0, 1]);
}

fn mesh_at(node_id: u64, translation: Vec3) -> RenderMeshSnapshot {
    let transform = Transform::from_translation(translation);
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "res://models/mesh-sdf.model.toml",
        )),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "res://materials/mesh-sdf.zmaterial",
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
