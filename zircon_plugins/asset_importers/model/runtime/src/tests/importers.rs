use zircon_runtime::asset::ImportedAsset;

use super::support::{
    ascii_dxf_3dface_fixture, ascii_ply_fixture, ascii_stl_fixture, assert_single_mesh_subasset,
    import_fixture_outcome, root_imported,
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
