use super::*;
use crate::core::framework::render::{
    render_mesh_stable_instance_key, RenderMaterialPropertyValue, RenderMeshStaticState,
};
use crate::core::math::{Transform, Vec4};

#[test]
fn geometry_extract_excludes_material_override_entities_from_static_batches() {
    let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
        "material:override-batch",
    ));
    let meshes = vec![test_static_mesh(1, material), test_static_mesh(2, material)];
    let geometry = GeometryExtract::from_meshes(CorePipelineKind::Core3d, meshes);
    assert_eq!(geometry.static_batches.len(), 1);

    let overrides = BTreeMap::from([(
        1,
        MaterialPropertyOverrideBlock::new()
            .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.0 }),
    )]);
    let geometry = geometry.with_material_property_overrides(overrides);

    assert!(geometry.static_batches.is_empty());
}

#[test]
fn geometry_extract_builds_static_batches_against_supplied_overrides_once() {
    let material = ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
        "material:constructor-override-batch",
    ));
    let meshes = vec![test_static_mesh(1, material), test_static_mesh(2, material)];
    let overrides = BTreeMap::from([(
        1,
        MaterialPropertyOverrideBlock::new()
            .with_value("gain", RenderMaterialPropertyValue::Float { value: 2.0 }),
    )]);

    let geometry = GeometryExtract::from_meshes_phase_inputs_and_overrides(
        CorePipelineKind::Core3d,
        meshes,
        Vec::new(),
        overrides,
    );

    assert!(geometry.static_batches.is_empty());
    assert_eq!(geometry.material_property_overrides.len(), 1);
}

#[test]
fn static_batch_key_borrows_render_layers_without_per_mesh_projection() {
    let source = include_str!("../geometry.rs");

    assert!(source.contains(concat!("render_layers: &'a", " RenderLayerSet")));
    assert!(!source.contains(concat!("render_layers:", " Vec<u32>")));
    assert!(!source.contains(concat!(
        "render_layers: mesh.common.layer_mask",
        ".iter().collect()"
    )));
}

fn test_static_mesh(
    node_id: EntityId,
    material: ResourceHandle<MaterialMarker>,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
            "model:override-batch",
        )),
        mesh: None,
        material,
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        static_state: RenderMeshStaticState::new(true, 1, 1),
        common: crate::core::framework::render::RendererCommon {
            layer_mask: RenderLayerSet::default(),
            is_static: true,
            ..Default::default()
        },
    }
}
