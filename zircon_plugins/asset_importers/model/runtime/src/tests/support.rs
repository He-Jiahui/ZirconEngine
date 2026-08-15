use zircon_runtime::asset::{AssetImportContext, AssetImportOutcome, ImportedAsset};

use crate::plugin_registration;

pub(super) fn assert_single_mesh_subasset(outcome: &AssetImportOutcome, path: &str) {
    let mesh_uri =
        zircon_runtime::asset::AssetUri::parse(&format!("res://models/{path}#Mesh0/Primitive0"))
            .expect("test mesh subasset uri");
    let root = outcome.root_entry().expect("root model asset entry");
    assert!(
        root.dependencies.contains(&mesh_uri),
        "root dependencies should include {mesh_uri}"
    );
    match &root.asset {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].mesh.as_ref().unwrap().locator, mesh_uri);
        }
        other => panic!("unexpected root model asset: {other:?}"),
    }
    let mesh_entry = outcome
        .entries
        .iter()
        .find(|entry| entry.locator == mesh_uri)
        .unwrap_or_else(|| panic!("missing mesh subasset {mesh_uri}"));
    match &mesh_entry.asset {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
            assert!(
                mesh.virtual_geometry.is_some(),
                "{mesh_uri} should preserve cooked virtual geometry"
            );
        }
        other => panic!("unexpected mesh subasset {mesh_uri}: {other:?}"),
    }
}

pub(super) fn root_imported(outcome: &AssetImportOutcome) -> ImportedAsset {
    outcome
        .root_entry()
        .expect("root model asset entry")
        .asset
        .clone()
}

pub(super) fn import_fixture_outcome(path: &str, source: &str) -> AssetImportOutcome {
    import_fixture_outcome_with_settings(path, source, Default::default())
}

pub(super) fn import_fixture_outcome_with_settings(
    path: &str,
    source: &str,
    import_settings: toml::Table,
) -> AssetImportOutcome {
    let report = plugin_registration();
    let importer = report
        .extensions
        .asset_importers()
        .select(std::path::Path::new(path))
        .unwrap();
    let uri = format!("res://models/{path}");
    let context = AssetImportContext::new(
        path.into(),
        zircon_runtime::asset::AssetUri::parse(&uri).unwrap(),
        source.as_bytes().to_vec(),
        import_settings,
    );
    importer.import(&context).unwrap()
}

pub(super) fn ascii_stl_fixture() -> &'static str {
    r#"solid triangle
facet normal 0 0 1
  outer loop
    vertex 0 0 0
    vertex 1 0 0
    vertex 0 1 0
  endloop
endfacet
endsolid triangle
"#
}

pub(super) fn ascii_ply_fixture() -> &'static str {
    r#"ply
format ascii 1.0
element vertex 3
property float x
property float y
property float z
property float nx
property float ny
property float nz
property float u
property float v
element face 1
property list uchar int vertex_indices
end_header
0 0 0 0 0 1 0 0
1 0 0 0 0 1 1 0
0 1 0 0 0 1 0 1
3 0 1 2
"#
}

pub(super) fn ascii_dxf_3dface_fixture() -> &'static str {
    r#"0
SECTION
2
ENTITIES
0
3DFACE
8
0
10
0.0
20
0.0
30
0.0
11
1.0
21
0.0
31
0.0
12
0.0
22
1.0
32
0.0
13
0.0
23
1.0
33
0.0
0
ENDSEC
0
EOF
"#
}
