use std::collections::BTreeMap;
use std::fs;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetImporter, AssetUri, ImportedAsset, MeshAsset, MeshAttributeValues, MeshIndices,
    MeshMorphTargetAsset, MeshSkinAsset, MeshValidationError, MeshVertex, ModelPrimitiveAsset,
    VirtualGeometryAsset, ZMeshDocument, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::math::{Vec2, Vec3};

#[test]
fn zmesh_document_roundtrip_preserves_mesh_payload() {
    let document = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]));

    let encoded = document.to_toml_string().unwrap();
    let decoded = ZMeshDocument::from_toml_str(&encoded).unwrap();
    let mesh = decoded
        .into_mesh_asset(AssetUri::parse("res://meshes/triangle.zmesh").unwrap())
        .unwrap();

    assert_eq!(mesh.topology, RenderMeshTopology::TriangleList);
    assert_eq!(mesh.vertex_count().unwrap(), 3);
    assert_eq!(mesh.index_count(), 3);
    assert!(mesh.asset_usage.main_world);
    assert!(mesh.asset_usage.render_world);
    assert_eq!(
        mesh.virtual_geometry
            .as_ref()
            .unwrap()
            .debug
            .source_hint
            .as_deref(),
        Some("zmesh-roundtrip")
    );
    assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
}

#[test]
fn zmesh_document_roundtrip_preserves_morph_targets_and_skin_inverse_bindposes() {
    let mut document = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]));
    document.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Smile".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_TANGENT.to_string(),
            MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 1.0]; 3]),
        )]),
    }];
    document.skin = Some(MeshSkinAsset {
        inverse_bind_matrices: vec![identity_matrix()],
    });

    let encoded = document.to_toml_string().unwrap();
    let decoded = ZMeshDocument::from_toml_str(&encoded).unwrap();
    let mesh = decoded
        .into_mesh_asset(AssetUri::parse("res://meshes/skinned.zmesh").unwrap())
        .unwrap();

    assert_eq!(mesh.morph_targets.len(), 1);
    assert_eq!(mesh.morph_targets[0].name.as_deref(), Some("Smile"));
    assert_eq!(
        mesh.morph_targets[0]
            .attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        mesh.skin.as_ref().unwrap().inverse_bind_matrices,
        vec![identity_matrix()]
    );
}

#[test]
fn mesh_asset_rejects_missing_position_attribute() {
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/no-position.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::new(),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::MissingPositionAttribute
    );
}

#[test]
fn mesh_asset_rejects_attribute_length_mismatch() {
    let mut attributes = triangle_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_NORMAL.to_string(),
        MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/bad-normal.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U32(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::AttributeLengthMismatch {
            attribute: MESH_ATTRIBUTE_NORMAL.to_string(),
            expected: 3,
            actual: 1,
        }
    );
}

#[test]
fn mesh_asset_rejects_morph_target_attribute_length_mismatch() {
    let mut mesh = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/bad-morph.zmesh").unwrap())
        .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Short".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.1]]),
        )]),
    }];

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::MorphTargetAttributeLengthMismatch {
            target_index: 0,
            attribute: MESH_ATTRIBUTE_POSITION.to_string(),
            expected: 3,
            actual: 1,
        }
    );
}

#[test]
fn mesh_asset_rejects_out_of_range_indices() {
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/bad-index.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: triangle_attributes(),
        indices: Some(MeshIndices::U32(vec![0, 1, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::IndexOutOfRange {
            max_index: 3,
            vertex_count: 3,
        }
    );
}

#[test]
fn model_primitive_converts_to_mesh_asset_with_builtin_attributes() {
    let primitive = ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::ZERO, Vec3::Z, Vec2::ZERO)
                .with_skinning([0, 1, 0, 0], [0.75, 0.25, 0.0, 0.0]),
            MeshVertex::new(Vec3::X, Vec3::Z, Vec2::X),
            MeshVertex::new(Vec3::Y, Vec3::Z, Vec2::Y),
        ],
        indices: vec![0, 1, 2],
        virtual_geometry: Some(sample_virtual_geometry()),
    };

    let mesh = MeshAsset::from_model_primitive(
        AssetUri::parse("res://models/triangle.obj#Mesh0/Primitive0").unwrap(),
        &primitive,
    );

    assert_eq!(mesh.vertex_count().unwrap(), 3);
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_POSITION));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_NORMAL));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_UV0));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_JOINT_INDEX));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_JOINT_WEIGHT));
    assert_eq!(mesh.to_model_primitive().unwrap(), primitive);
}

#[test]
fn mesh_render_descriptor_uses_bounds_topology_and_indices() {
    let mesh = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/triangle.zmesh").unwrap())
        .unwrap();

    let descriptor = mesh.render_mesh_descriptor();

    assert_eq!(descriptor.topology, RenderMeshTopology::TriangleList);
    assert_eq!(descriptor.vertex_count, 3);
    assert_eq!(descriptor.index_count, 3);
    assert_eq!(descriptor.primitive_count, 1);
    assert_eq!(descriptor.bounds.min, [0.0, 0.0, 0.0]);
    assert_eq!(descriptor.bounds.max, [1.0, 1.0, 0.0]);
    assert!(descriptor.suitable_for_2d);
    assert!(descriptor.has_virtual_geometry_payload);
}

#[test]
fn default_importer_routes_zmesh_to_mesh_asset() {
    let root = unique_temp_project_root("zmesh_import");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("triangle.zmesh");
    fs::write(
        &path,
        sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
            .to_toml_string()
            .unwrap(),
    )
    .unwrap();

    let imported = AssetImporter::default()
        .import_from_source(
            &path,
            &AssetUri::parse("res://meshes/triangle.zmesh").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

fn sample_zmesh_document(indices: MeshIndices) -> ZMeshDocument {
    ZMeshDocument {
        version: crate::asset::ZMESH_DOCUMENT_VERSION,
        name: Some("Triangle".to_string()),
        topology: RenderMeshTopology::TriangleList,
        attributes: triangle_attributes(),
        indices: Some(indices),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: Some(sample_virtual_geometry()),
    }
}

fn triangle_attributes() -> BTreeMap<String, MeshAttributeValues> {
    BTreeMap::from([
        (
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        ),
        (
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]),
        ),
        (
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        ),
    ])
}

fn sample_virtual_geometry() -> VirtualGeometryAsset {
    VirtualGeometryAsset {
        debug: crate::asset::VirtualGeometryDebugMetadataAsset {
            mesh_name: Some("Triangle".to_string()),
            source_hint: Some("zmesh-roundtrip".to_string()),
            notes: vec!["unit-test".to_string()],
        },
        ..Default::default()
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
