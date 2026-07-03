use super::*;

#[test]
fn importer_validates_wgsl_and_reports_errors() {
    let root = unique_temp_project_root("shader_import");
    fs::create_dir_all(&root).unwrap();
    let valid_path = root.join("pbr.wgsl");
    let invalid_path = root.join("broken.wgsl");
    fs::write(&valid_path, valid_wgsl()).unwrap();
    fs::write(&invalid_path, "@vertex fn vs_main( {").unwrap();

    let importer = importer_with_first_wave_plugin_fixtures();
    let valid = importer
        .import_from_source(
            &valid_path,
            &AssetUri::parse("res://shaders/pbr.wgsl").unwrap(),
        )
        .unwrap();
    let invalid = importer.import_from_source(
        &invalid_path,
        &AssetUri::parse("res://shaders/broken.wgsl").unwrap(),
    );

    match valid {
        ImportedAsset::Shader(shader) => {
            assert!(shader.source.contains("vs_main"));
            assert_eq!(shader.uri.to_string(), "res://shaders/pbr.wgsl");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
    assert!(invalid.is_err());
    assert!(invalid.unwrap_err().to_string().contains("wgsl"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_decodes_obj_into_model_asset() {
    let root = unique_temp_project_root("model_import");
    fs::create_dir_all(&root).unwrap();
    let obj_path = root.join("triangle.obj");
    fs::write(
        &obj_path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
",
    )
    .unwrap();

    let importer = importer_with_first_wave_plugin_fixtures();

    let obj = importer
        .import_from_source(
            &obj_path,
            &AssetUri::parse("res://models/triangle.obj").unwrap(),
        )
        .unwrap();

    match obj {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                AssetUri::parse("res://models/triangle.obj#Mesh0/Primitive0").unwrap()
            );
            assert_cooked_virtual_geometry(&model.primitives[0], "res://models/triangle.obj");
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_emits_mesh_subassets_for_model_imports() {
    let root = unique_temp_project_root("model_import_mesh_subassets");
    fs::create_dir_all(&root).unwrap();
    let obj_path = root.join("triangle.obj");
    fs::write(
        &obj_path,
        "\
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
f 1 2 3
",
    )
    .unwrap();

    let importer = importer_with_first_wave_plugin_fixtures();
    let outcome = importer
        .import_with_settings(
            &obj_path,
            &AssetUri::parse("res://models/triangle.obj").unwrap(),
            Default::default(),
        )
        .unwrap();
    let mesh_uri = AssetUri::parse("res://models/triangle.obj#Mesh0/Primitive0").unwrap();

    match &outcome.root_entry().unwrap().asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].mesh.as_ref().unwrap().locator, mesh_uri);
        }
        other => panic!("unexpected root model asset: {other:?}"),
    }
    assert!(outcome
        .root_entry()
        .unwrap()
        .dependencies
        .contains(&mesh_uri));
    let mesh_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == mesh_uri)
        .expect("mesh subasset entry");
    match &mesh_entry.asset {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected mesh subasset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn importer_backfills_virtual_geometry_for_model_toml_without_dropping_base_mesh() {
    let root = unique_temp_project_root("model_toml_virtual_geometry_import");
    fs::create_dir_all(&root).unwrap();
    let model_path = root.join("triangle.model.toml");
    let base_vertices = vec![
        MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO),
        MeshVertex::new(Vec3::X, Vec3::Y, Vec2::X),
        MeshVertex::new(Vec3::Y, Vec3::Y, Vec2::Y),
    ];
    let base_indices = vec![0, 1, 2];
    let source_model = ModelAsset {
        uri: AssetUri::parse("res://models/triangle.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: base_vertices.clone(),
            indices: base_indices.clone(),
            mesh: None,
            virtual_geometry: None,
        }],
    };
    fs::write(&model_path, source_model.to_toml_string().unwrap()).unwrap();

    let imported = AssetImporter::default()
        .import_from_source(
            &model_path,
            &AssetUri::parse("res://models/triangle.model.toml").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(
                model.primitives[0]
                    .vertices
                    .iter()
                    .map(|vertex| vertex.position)
                    .collect::<Vec<_>>(),
                base_vertices
                    .iter()
                    .map(|vertex| vertex.position)
                    .collect::<Vec<_>>()
            );
            assert_eq!(model.primitives[0].indices, base_indices);
            assert_eq!(
                model.primitives[0].mesh.as_ref().unwrap().locator,
                AssetUri::parse("res://models/triangle.model.toml#Mesh0/Primitive0").unwrap()
            );
            assert_cooked_virtual_geometry(
                &model.primitives[0],
                "res://models/triangle.model.toml",
            );
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}
