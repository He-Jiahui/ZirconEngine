//! Memory-only reference correspondence, never an imported model or asset/preview producer.

use zircon_runtime::asset::assets::{
    ModelAsset, ModelPrimitiveAsset, SceneMeshInstanceAsset, SceneMeshLodLevelAsset,
    SceneMeshPrimitiveBindingAsset,
};
use zircon_runtime::asset::{AssetReference, AssetUri};

use super::*;

fn reference(path: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(path).unwrap())
}

fn inputs(count: usize) -> (ModelAsset, SceneMeshInstanceAsset) {
    let uri = AssetUri::parse("res://unit-memory.glb#Mesh4").unwrap();
    let primitives = (0..count)
        .map(|index| ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: Some(reference(&format!(
                "res://unit-memory.glb#Mesh4/Primitive{index}"
            ))),
            mesh_sdf: None,
            virtual_geometry: None,
        })
        .collect::<Vec<_>>();
    let bindings = primitives
        .iter()
        .enumerate()
        .map(|(index, primitive)| SceneMeshPrimitiveBindingAsset {
            mesh: primitive.mesh.clone().unwrap(),
            material: reference(&format!("res://unit-memory.glb#Material{index}")),
        })
        .collect();
    (
        ModelAsset {
            uri: uri.clone(),
            primitives,
        },
        SceneMeshInstanceAsset {
            model: AssetReference::from_locator(uri),
            mesh: None,
            material: reference("res://unit-memory.glb#Material0"),
            render_queue: 5,
            material_queue: 2,
            order_in_layer: 3,
            depth_bias: 0.25,
            morph_weights: vec![0.25, 0.75],
            primitives: bindings,
            lods: Vec::new(),
        },
    )
}

#[test]
fn static_mesh_source_missing_instance_is_unknown_not_zero_bindings() {
    let (model, _) = inputs(2);
    let source = StaticMeshSourceAdapter::read(&model, None).unwrap();
    assert_eq!(source.instance, StaticMeshInstanceObservation::Unknown);
    assert_eq!(source.geometry.primitive_count, 2);
    assert_eq!(source.geometry.mesh_reference_count, 2);
    assert_eq!(source.geometry.uri, model.uri);
}

#[test]
fn static_mesh_source_retains_all_material_bindings_lods_and_instance_fields() {
    let (model, mut instance) = inputs(2);
    instance.mesh = Some(reference("res://unit-memory.glb#Mesh4/Primitive0"));
    instance.lods.push(SceneMeshLodLevelAsset {
        min_distance: 20.0,
        model: reference("res://unit-memory.glb#Mesh9"),
        mesh: None,
        material: reference("res://unit-memory.glb#Material7"),
        primitives: instance.primitives.clone(),
    });
    let original_model = model.clone();
    let original_instance = instance.clone();
    let source = StaticMeshSourceAdapter::read(&model, Some(&instance)).unwrap();
    assert_eq!(
        source.instance,
        StaticMeshInstanceObservation::Known(Box::new(original_instance.clone()))
    );
    assert_eq!(model, original_model);
    assert_eq!(instance, original_instance);
}

#[test]
fn static_mesh_source_requires_exact_model_subasset_context() {
    let (model, mut instance) = inputs(2);
    instance.model = reference("res://unit-memory.glb#Mesh0");
    let before = instance.clone();
    assert!(matches!(
        StaticMeshSourceAdapter::read(&model, Some(&instance)),
        Err(StaticMeshSourceError::ModelMismatch { .. })
    ));
    assert_eq!(instance, before);
}

#[test]
fn static_mesh_source_rejects_legacy_first_material_and_partial_extra_bindings() {
    let (model, instance) = inputs(2);
    for count in [0, 1, 3] {
        let mut changed = instance.clone();
        changed
            .primitives
            .resize(count, instance.primitives[0].clone());
        assert_eq!(
            StaticMeshSourceAdapter::read(&model, Some(&changed)),
            Err(StaticMeshSourceError::BindingCountMismatch {
                expected: 2,
                actual: count
            })
        );
    }
}

#[test]
fn static_mesh_source_rejects_missing_and_reordered_primitive_references() {
    let (mut model, mut instance) = inputs(2);
    instance.primitives.swap(0, 1);
    assert!(matches!(
        StaticMeshSourceAdapter::read(&model, Some(&instance)),
        Err(StaticMeshSourceError::MeshReferenceMismatch {
            primitive_index: 0,
            ..
        })
    ));
    instance.primitives.swap(0, 1);
    model.primitives[1].mesh = None;
    assert_eq!(
        StaticMeshSourceAdapter::read(&model, Some(&instance)),
        Err(StaticMeshSourceError::MissingMeshReference { primitive_index: 1 })
    );
}

#[test]
fn static_mesh_source_keeps_nine_exact_bindings_and_distinct_material_references() {
    let (model, instance) = inputs(9);
    let source = StaticMeshSourceAdapter::read(&model, Some(&instance)).unwrap();
    let StaticMeshInstanceObservation::Known(observed) = source.instance else {
        panic!("complete original instance must be retained");
    };
    assert_eq!(observed.primitives.len(), 9);
    assert_eq!(observed.primitives, instance.primitives);
    assert_ne!(
        observed.primitives[0].material,
        observed.primitives[8].material
    );
    assert_eq!(
        source.geometry.uri.to_string(),
        "res://unit-memory.glb#Mesh4"
    );
}

#[test]
fn static_mesh_source_rejects_same_locator_with_different_mesh_identity() {
    let (model, mut instance) = inputs(1);
    instance.primitives[0].mesh.uuid = reference("res://unit-other-identity.mesh").uuid;
    assert_eq!(
        instance.primitives[0].mesh.locator,
        model.primitives[0].mesh.as_ref().unwrap().locator
    );
    assert!(matches!(
        StaticMeshSourceAdapter::read(&model, Some(&instance)),
        Err(StaticMeshSourceError::MeshReferenceMismatch {
            primitive_index: 0,
            ..
        })
    ));
}

#[test]
fn static_mesh_source_distinguishes_known_empty_from_unknown_context() {
    let (model, instance) = inputs(0);
    assert_eq!(
        StaticMeshSourceAdapter::read(&model, None)
            .unwrap()
            .instance,
        StaticMeshInstanceObservation::Unknown
    );
    assert_eq!(
        StaticMeshSourceAdapter::read(&model, Some(&instance))
            .unwrap()
            .instance,
        StaticMeshInstanceObservation::Known(Box::new(instance))
    );
}
