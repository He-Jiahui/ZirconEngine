use std::collections::BTreeMap;

use crate::asset::importer::cook_mesh_asset_derived_data;
use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset, MeshSdfCookBudget,
    MeshSdfCookRequest, MeshSdfCookSettings, MeshSkinAsset, MeshValidationError,
    ModelPrimitiveAsset, VirtualGeometryCookRequest, VirtualGeometryCookSettings,
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
    MESH_ATTRIBUTE_UV1,
};
use crate::core::framework::render::RenderMeshTopology;

#[test]
fn mesh_asset_generates_missing_tangents_for_unindexed_triangle_list() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/unindexed-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: quad_unindexed_attributes(),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(mesh.try_generate_missing_tangents().unwrap());

    assert_tangents_approx(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        &vec![[1.0, 0.0, 0.0, 1.0]; 6],
    );
    assert_eq!(mesh.validate(), Ok(()));
}

#[test]
fn mesh_asset_generates_missing_tangents_for_indexed_triangle_list() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/indexed-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: quad_indexed_attributes(),
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 2, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(mesh.try_generate_missing_tangents().unwrap());

    assert_tangents_approx(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        &[[1.0, 0.0, 0.0, 1.0]; 4],
    );
    assert_eq!(mesh.validate(), Ok(()));
}

#[test]
fn mesh_asset_generates_mikktspace_tangents_from_selected_uv1() {
    let mut attributes = quad_indexed_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_UV0.to_string(),
        MeshAttributeValues::Float32x2(vec![[0.0, 0.0]; 4]),
    );
    attributes.insert(
        MESH_ATTRIBUTE_UV1.to_string(),
        MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
    );
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/uv1-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 2, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(mesh
        .try_generate_missing_tangents_for_uv(MESH_ATTRIBUTE_UV1)
        .unwrap());

    assert_tangents_approx(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        &[[1.0, 0.0, 0.0, 1.0]; 4],
    );
}

#[test]
fn mesh_asset_mikktspace_preserves_mirrored_uv_handedness() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/mirrored-uv-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]]),
            ),
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(mesh.try_generate_missing_tangents().unwrap());

    assert_tangents_approx(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        &[[0.0, 1.0, 0.0, -1.0]; 3],
    );
}

#[test]
fn mesh_asset_splits_indexed_vertex_across_mikktspace_corner_groups() {
    let mut mesh = mikktspace_corner_split_mesh();

    assert!(mesh.try_generate_missing_tangents().unwrap());

    assert_eq!(mesh.vertex_count().unwrap(), 6);
    assert_eq!(mesh.indices, Some(MeshIndices::U16(vec![0, 1, 2, 5, 3, 4])));
    let tangents = mesh.attributes[MESH_ATTRIBUTE_TANGENT]
        .as_float32x4()
        .unwrap();
    assert_ne!(tangents[0].map(f32::to_bits), tangents[5].map(f32::to_bits));
    assert_eq!(
        mesh.attributes[MESH_ATTRIBUTE_COLOR]
            .as_float32x4()
            .unwrap()[5],
        [0.25, 0.5, 0.75, 1.0]
    );
    assert_eq!(
        mesh.morph_targets[0].attributes[MESH_ATTRIBUTE_POSITION]
            .as_float32x3()
            .unwrap()[5],
        [0.0, 0.0, 0.5]
    );
    assert_eq!(mesh.validate(), Ok(()));
}

#[test]
fn mesh_asset_cooks_virtual_geometry_after_mikktspace_corner_splits() {
    let mut mesh = mikktspace_corner_split_mesh();
    mesh.try_generate_missing_tangents().unwrap();

    cook_mesh_asset_derived_data(
        &mut mesh,
        Some("corner_split"),
        "res://meshes/mikktspace-corner-split.zmesh",
        &VirtualGeometryCookRequest::Enabled(VirtualGeometryCookSettings::default()),
        &MeshSdfCookRequest::Enabled(MeshSdfCookSettings {
            max_dimension: 4,
            max_voxel_count: 64,
            max_payload_bytes: 256,
            surface_band_voxels: 1,
            two_sided: true,
        }),
        &mut MeshSdfCookBudget::default(),
    )
    .unwrap();

    assert!(mesh.virtual_geometry.is_some());
    let primitive = mesh.to_model_primitive().unwrap();
    let ordinals = primitive
        .vertices
        .iter()
        .map(|vertex| {
            ModelPrimitiveAsset::decode_virtual_geometry_vertex_ordinal(vertex.joint_indices)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ordinals,
        (0..primitive.vertices.len() as u32).collect::<Vec<_>>()
    );
    mesh.mesh_sdf
        .as_ref()
        .expect("SDF must be cooked from the final split geometry")
        .validate_for_source(&primitive.vertices, &primitive.indices)
        .unwrap();
}

#[test]
fn mesh_asset_preserves_authored_joint_slots_for_zero_weight_skin_owner() {
    let mut mesh = mikktspace_corner_split_mesh();
    mesh.try_generate_missing_tangents().unwrap();
    let authored_joint_indices = vec![[7, 8, 9, 10]; mesh.vertex_count().unwrap()];
    mesh.attributes.insert(
        MESH_ATTRIBUTE_JOINT_INDEX.to_string(),
        MeshAttributeValues::Uint16x4(authored_joint_indices.clone()),
    );
    mesh.attributes.insert(
        MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
        MeshAttributeValues::Float32x4(vec![[0.0; 4]; mesh.vertex_count().unwrap()]),
    );
    mesh.skin = Some(MeshSkinAsset {
        inverse_bind_matrices: Vec::new(),
    });

    cook_mesh_asset_derived_data(
        &mut mesh,
        Some("zero_weight_skin"),
        "res://meshes/zero-weight-skin.zmesh",
        &VirtualGeometryCookRequest::Enabled(VirtualGeometryCookSettings::default()),
        &MeshSdfCookRequest::default(),
        &mut MeshSdfCookBudget::default(),
    )
    .unwrap();

    assert!(mesh.virtual_geometry.is_none());
    assert_eq!(
        mesh.attributes[MESH_ATTRIBUTE_JOINT_INDEX]
            .as_uint16x4()
            .unwrap(),
        authored_joint_indices.as_slice()
    );
}

#[test]
fn mesh_asset_rebuilds_flat_morph_normal_and_mikktspace_tangent_deltas() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/morph-tangent-frames.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
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
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: vec![MeshMorphTargetAsset {
            name: Some("bent".to_string()),
            attributes: BTreeMap::from([
                (
                    MESH_ATTRIBUTE_POSITION.to_string(),
                    MeshAttributeValues::Float32x3(vec![
                        [0.0, 0.0, 0.0],
                        [0.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0],
                    ]),
                ),
                (
                    MESH_ATTRIBUTE_NORMAL.to_string(),
                    MeshAttributeValues::Float32x3(vec![[9.0, 9.0, 9.0]; 3]),
                ),
                (
                    MESH_ATTRIBUTE_TANGENT.to_string(),
                    MeshAttributeValues::Float32x3(vec![[9.0, 9.0, 9.0]; 3]),
                ),
            ]),
        }],
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(mesh.try_generate_missing_tangents().unwrap());
    assert!(mesh
        .try_rebuild_morph_tangent_frames_for_uv(true, Some(MESH_ATTRIBUTE_UV0))
        .unwrap());

    let normal_deltas = mesh.morph_targets[0].attributes[MESH_ATTRIBUTE_NORMAL]
        .as_float32x3()
        .unwrap();
    for delta in normal_deltas {
        assert!((delta[0] - 0.0).abs() < 0.000001);
        assert!((delta[1] + std::f32::consts::FRAC_1_SQRT_2).abs() < 0.000001);
        assert!((delta[2] - (std::f32::consts::FRAC_1_SQRT_2 - 1.0)).abs() < 0.000001);
    }
    assert_tangents_approx(
        &mesh.morph_targets[0].attributes[MESH_ATTRIBUTE_TANGENT]
            .as_float32x3()
            .unwrap()
            .iter()
            .map(|tangent| [tangent[0], tangent[1], tangent[2], 0.0])
            .collect::<Vec<_>>(),
        &[[0.0; 4]; 3],
    );
}

#[test]
fn mesh_asset_does_not_overwrite_existing_tangents() {
    let mut attributes = quad_indexed_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_TANGENT.to_string(),
        MeshAttributeValues::Float32x4(vec![[0.0, 1.0, 0.0, -1.0]; 4]),
    );
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/existing-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 2, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(!mesh.try_generate_missing_tangents().unwrap());
    assert_eq!(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        [[0.0, 1.0, 0.0, -1.0]; 4].as_slice()
    );
}

#[test]
fn mesh_asset_rejects_tangent_generation_after_virtual_geometry_cook() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/cooked-before-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: quad_indexed_attributes(),
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 2, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: Some(Default::default()),
    };

    let error = mesh.try_generate_missing_tangents().unwrap_err();

    assert!(error
        .to_string()
        .contains("before Virtual Geometry is cooked"));
    assert!(!mesh.attributes.contains_key(MESH_ATTRIBUTE_TANGENT));
}

#[test]
fn mesh_asset_rejects_tangent_generation_for_missing_inputs_or_topology() {
    let mut missing_normal = MeshAsset {
        uri: AssetUri::parse("res://meshes/tangents-no-normal.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
            ),
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };
    let mut missing_uv = MeshAsset {
        uri: AssetUri::parse("res://meshes/tangents-no-uv.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };
    let mut lines = MeshAsset {
        uri: AssetUri::parse("res://meshes/line-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::LineList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 2]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0]]),
            ),
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert_eq!(
        missing_normal.try_generate_missing_tangents().unwrap_err(),
        MeshValidationError::TangentGenerationMissingAttribute {
            attribute: MESH_ATTRIBUTE_NORMAL,
        }
    );
    assert_eq!(
        missing_uv.try_generate_missing_tangents().unwrap_err(),
        MeshValidationError::TangentGenerationMissingAttribute {
            attribute: MESH_ATTRIBUTE_UV0,
        }
    );
    assert_eq!(
        lines.try_generate_missing_tangents().unwrap_err(),
        MeshValidationError::TangentGenerationRequiresTriangleList {
            topology: RenderMeshTopology::LineList,
        }
    );
}

fn quad_indexed_attributes() -> BTreeMap<String, MeshAttributeValues> {
    BTreeMap::from([
        (
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]),
        ),
        (
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 4]),
        ),
        (
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        ),
    ])
}

fn quad_unindexed_attributes() -> BTreeMap<String, MeshAttributeValues> {
    BTreeMap::from([
        (
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]),
        ),
        (
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 6]),
        ),
        (
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
            ]),
        ),
    ])
}

fn mikktspace_corner_split_mesh() -> MeshAsset {
    MeshAsset {
        uri: AssetUri::parse("res://meshes/mikktspace-corner-split.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [-1.0, 0.0, 0.0],
                    [0.0, -1.0, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 5]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![
                    [0.0, 0.0],
                    [1.0, 0.0],
                    [0.0, 1.0],
                    [1.0, 0.0],
                    [0.0, -1.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_COLOR.to_string(),
                MeshAttributeValues::Float32x4(vec![
                    [0.25, 0.5, 0.75, 1.0],
                    [1.0; 4],
                    [1.0; 4],
                    [1.0; 4],
                    [1.0; 4],
                ]),
            ),
        ]),
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 3, 4])),
        asset_usage: Default::default(),
        morph_targets: vec![MeshMorphTargetAsset {
            name: Some("offset".to_string()),
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.5],
                    [0.0; 3],
                    [0.0; 3],
                    [0.0; 3],
                    [0.0; 3],
                ]),
            )]),
        }],
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}

fn assert_tangents_approx(actual: &[[f32; 4]], expected: &[[f32; 4]]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        for component in 0..4 {
            assert!(
                (actual[component] - expected[component]).abs() < 0.000001,
                "tangent component {component} expected {:?} but got {:?}",
                expected,
                actual
            );
        }
    }
}
