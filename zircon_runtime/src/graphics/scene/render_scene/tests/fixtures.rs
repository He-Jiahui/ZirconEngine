use std::sync::Arc;

use crate::core::framework::render::{
    MaterialPropertyOverrideBlock, RenderMaterialAlphaMode, RenderMeshBounds,
    RenderWorldSnapshotHandle, RendererCommon, render_mesh_stable_instance_key,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Mat4, Vec4};
use crate::core::resource::{MaterialMarker, MeshMarker, ModelMarker, ResourceHandle, ResourceId};

use super::super::{
    RenderScene, RenderSceneMeshLod, RenderSceneMeshSource, RenderSceneMeshSourceLevel,
    RenderScenePrimitive, RenderScenePrimitiveDescriptor, RenderScenePrimitiveLocalBounds,
    RenderScenePrimitiveRevisions,
};

pub(super) fn test_primitive(entity: u64) -> RenderScenePrimitive {
    test_primitive_with(entity, |_| {})
}

pub(super) fn test_primitive_with(
    entity: u64,
    mutate: impl FnOnce(&mut RenderScenePrimitiveDescriptor),
) -> RenderScenePrimitive {
    test_primitive_with_revisions(entity, test_revisions(1, 1, 1, 1, 1), mutate)
}

pub(super) fn test_primitive_with_revisions(
    entity: u64,
    revisions: RenderScenePrimitiveRevisions,
    mutate: impl FnOnce(&mut RenderScenePrimitiveDescriptor),
) -> RenderScenePrimitive {
    let mut descriptor = test_descriptor(entity, stable_key(entity));
    mutate(&mut descriptor);
    RenderScenePrimitive::new(
        descriptor,
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-1.0, -2.0, -3.0],
            [1.0, 2.0, 3.0],
        )),
        revisions,
    )
    .expect("finite test primitive")
}

pub(super) fn test_primitive_with_key(entity: u64, key: u64) -> RenderScenePrimitive {
    RenderScenePrimitive::new(
        test_descriptor(entity, key),
        RenderScenePrimitiveLocalBounds::base_only(RenderMeshBounds::from_min_max(
            [-1.0; 3], [1.0; 3],
        )),
        test_revisions(1, 1, 1, 1, 1),
    )
    .expect("finite test primitive")
}

pub(super) fn test_descriptor(entity: u64, key: u64) -> RenderScenePrimitiveDescriptor {
    RenderScenePrimitiveDescriptor {
        node_id: entity,
        stable_instance_key: key,
        world_from_local: Mat4::IDENTITY,
        mesh_source: test_mesh_source(),
        morph_weights: Arc::from([]),
        skeletal_pose: None,
        tint: Vec4::ONE,
        material_property_overrides: MaterialPropertyOverrideBlock::default(),
        material_alpha_mode: RenderMaterialAlphaMode::Opaque,
        render_queue: 0,
        material_queue: 0,
        order_in_layer: 0,
        depth_bias: 0.0,
        mobility: Mobility::Static,
        transform_static: true,
        common: RendererCommon {
            is_static: true,
            ..RendererCommon::default()
        },
    }
}

fn test_mesh_source() -> RenderSceneMeshSource {
    RenderSceneMeshSource::new(
        test_mesh_source_level("base"),
        Vec::<RenderSceneMeshLod>::new(),
    )
}

pub(super) fn test_mesh_source_level(label: &str) -> RenderSceneMeshSourceLevel {
    test_mesh_source_level_with_labels(label, label, label)
}

pub(super) fn test_mesh_source_level_with_labels(
    model_label: &str,
    mesh_label: &str,
    material_label: &str,
) -> RenderSceneMeshSourceLevel {
    RenderSceneMeshSourceLevel::new(
        ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(&format!(
            "tests/render-scene/{model_label}/model"
        ))),
        Some(ResourceHandle::<MeshMarker>::new(
            ResourceId::from_stable_label(&format!("tests/render-scene/{mesh_label}/mesh")),
        )),
        ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(&format!(
            "tests/render-scene/{material_label}/material"
        ))),
        Vec::new(),
    )
}

pub(super) const fn test_revisions(
    transform: u64,
    geometry: u64,
    material: u64,
    bounds: u64,
    deformation: u64,
) -> RenderScenePrimitiveRevisions {
    RenderScenePrimitiveRevisions::new(transform, geometry, material, bounds, deformation)
}

pub(super) fn stable_key(entity: u64) -> u64 {
    render_mesh_stable_instance_key(entity, 0)
}

pub(super) fn test_scene() -> RenderScene {
    RenderScene::new(RenderWorldSnapshotHandle::new(1))
}
