use zircon_runtime::asset::ImportedAsset;

use super::support::{
    ascii_dxf_3dface_fixture, ascii_ply_fixture, ascii_stl_fixture, assert_single_mesh_subasset,
    import_fixture_outcome, import_fixture_outcome_with_settings, root_imported,
};

#[test]
fn stl_importer_decodes_ascii_triangle() {
    let outcome = import_fixture_outcome("triangle.stl", ascii_stl_fixture());
    let imported = root_imported(&outcome);

    assert_single_mesh_subasset(&outcome, "triangle.stl");

    match imported {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert!(model.primitives[0].virtual_geometry.is_some());
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn ply_importer_decodes_ascii_triangle() {
    let outcome = import_fixture_outcome("triangle.ply", ascii_ply_fixture());
    let imported = root_imported(&outcome);

    assert_single_mesh_subasset(&outcome, "triangle.ply");

    match imported {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert_eq!(model.primitives[0].vertices[1].uv[0], 1.0);
            assert!(model.primitives[0].virtual_geometry.is_some());
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn dxf_importer_decodes_3dface_triangle() {
    let outcome = import_fixture_outcome("triangle.dxf", ascii_dxf_3dface_fixture());
    let imported = root_imported(&outcome);

    assert_single_mesh_subasset(&outcome, "triangle.dxf");

    match imported {
        ImportedAsset::Model(model) => {
            assert_eq!(model.primitives.len(), 1);
            assert_eq!(model.primitives[0].vertices.len(), 3);
            assert_eq!(model.primitives[0].indices, vec![0, 1, 2]);
            assert!(model.primitives[0].virtual_geometry.is_some());
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }
}

#[test]
fn dxf_mesh_sdf_cook_respects_explicit_enablement() {
    let disabled = import_fixture_outcome("disabled.dxf", ascii_dxf_3dface_fixture());
    let enabled_settings = toml::from_str(
        r#"
            [mesh_sdf]
            enabled = true
            max_dimension = 8
            max_voxel_count = 512
            max_payload_bytes = 4096
            surface_band_voxels = 2
        "#,
    )
    .unwrap();
    let enabled = import_fixture_outcome_with_settings(
        "enabled.dxf",
        ascii_dxf_3dface_fixture(),
        enabled_settings,
    );

    let mesh_sdf = |outcome: &zircon_runtime::asset::AssetImportOutcome| {
        outcome
            .entries
            .iter()
            .find_map(|entry| match &entry.asset {
                ImportedAsset::Mesh(mesh) => Some(mesh.mesh_sdf.is_some()),
                _ => None,
            })
            .expect("DXF import should produce one mesh subasset")
    };
    assert!(!mesh_sdf(&disabled));
    assert!(mesh_sdf(&enabled));
}
