use super::*;

#[test]
fn importer_preserves_gltf_skinning_channels_on_model_vertices() {
    let root = unique_temp_project_root("skinned_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_skinned_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/skinned_triangle.gltf").unwrap();

    let gltf = importer.import_from_source(&gltf_path, &root_uri).unwrap();

    match gltf {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert!(
                model.primitives[0].virtual_geometry.is_none(),
                "skinned glTF primitives should not consume joint slots as automatic VG ordinals"
            );
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                label_uri(&root_uri, "Mesh0/Primitive0")
            );
            assert_eq!(model.primitives[0].vertices[0].joint_indices, [0, 1, 0, 0]);
            assert_eq!(model.primitives[0].vertices[1].joint_indices, [1, 0, 0, 0]);
            assert_eq!(
                model.primitives[0].vertices[0].joint_weights,
                [0.75, 0.25, 0.0, 0.0]
            );
            assert_eq!(
                model.primitives[0].vertices[1].joint_weights,
                [1.0, 0.0, 0.0, 0.0]
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_preserves_gltf_tangent_and_color_channels_on_model_vertices() {
    let root = unique_temp_project_root("tangent_color_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_tangent_color_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/tangent_color_triangle.gltf").unwrap();

    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let expected_tangents = vec![
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, -1.0],
        [0.0, 0.0, 1.0, 1.0],
    ];
    let expected_colors = vec![
        [1.0, 0.25, 0.5, 0.75],
        [0.25, 1.0, 0.5, 0.5],
        [0.1, 0.2, 1.0, 1.0],
    ];

    match &outcome.root_entry().expect("root gltf entry").asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            let vertices = &model.primitives[0].vertices;
            assert_eq!(vertices.len(), 3);
            for (index, vertex) in vertices.iter().enumerate() {
                assert_eq!(vertex.tangent, expected_tangents[index]);
                assert_eq!(vertex.color, expected_colors[index]);
            }
        }
        other => panic!("unexpected root gltf asset: {other:?}"),
    }

    match &entry_for_label(&outcome, &root_uri, "Mesh0/Primitive0").asset {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(
                mesh.attributes.get(MESH_ATTRIBUTE_TANGENT),
                Some(&MeshAttributeValues::Float32x4(expected_tangents))
            );
            assert_eq!(
                mesh.attributes.get(MESH_ATTRIBUTE_COLOR),
                Some(&MeshAttributeValues::Float32x4(expected_colors))
            );
        }
        other => panic!("unexpected Mesh0/Primitive0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_preserves_gltf_texcoord_1_on_model_vertices_and_mesh_subasset() {
    let root = unique_temp_project_root("uv_channel_model_import");
    fs::create_dir_all(&root).unwrap();
    let gltf_path = write_uv_channel_triangle_gltf(&root);
    let importer = importer_with_first_wave_plugin_fixtures();
    let root_uri = AssetUri::parse("res://models/uv_channel_triangle.gltf").unwrap();

    let outcome = importer
        .import_with_settings(&gltf_path, &root_uri, Default::default())
        .unwrap();

    let expected_uv0 = vec![[0.0, 0.0], [0.5, 0.0], [0.0, 0.5]];
    let expected_uv1 = vec![[1.0, 0.25], [0.25, 1.0], [0.75, 0.75]];

    match &outcome.root_entry().expect("root gltf entry").asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            let vertices = &model.primitives[0].vertices;
            assert_eq!(vertices.len(), 3);
            for (index, vertex) in vertices.iter().enumerate() {
                assert_eq!(vertex.uv, expected_uv0[index]);
                assert_eq!(vertex.uv1, expected_uv1[index]);
            }
        }
        other => panic!("unexpected root gltf asset: {other:?}"),
    }

    match &entry_for_label(&outcome, &root_uri, "Mesh0/Primitive0").asset {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(
                mesh.attributes.get(MESH_ATTRIBUTE_UV0),
                Some(&MeshAttributeValues::Float32x2(expected_uv0))
            );
            assert_eq!(
                mesh.attributes.get(MESH_ATTRIBUTE_UV1),
                Some(&MeshAttributeValues::Float32x2(expected_uv1))
            );
        }
        other => panic!("unexpected Mesh0/Primitive0 asset: {other:?}"),
    }

    match &entry_for_label(&outcome, &root_uri, "Material0").asset {
        ImportedAsset::Material(material) => {
            let base_color = material
                .texture_slots
                .get("base_color")
                .expect("base_color texture slot should be imported");
            assert_eq!(
                base_color.reference.as_ref().unwrap().locator,
                label_uri(&root_uri, "Texture0")
            );
            assert_eq!(base_color.texture_uv_channel(), 1);
            assert_eq!(
                material
                    .standard_material_descriptor()
                    .base_color_texture_uv_channel,
                1
            );
        }
        other => panic!("unexpected Material0 asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
