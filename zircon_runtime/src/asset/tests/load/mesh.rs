use crate::asset::load::mesh::{decode_mesh_file, generate_cube_mesh, MeshLoadError};

#[test]
fn builtin_cube_mesh_has_triangles() {
    let payload = generate_cube_mesh();

    assert_eq!(payload.indices.len() % 3, 0);
    assert!(!payload.vertices.is_empty());
}

#[test]
fn unsupported_mesh_file_reports_typed_mesh_load_error() {
    let path = "res://meshes/cube.fbx";

    let error = decode_mesh_file(path).expect_err("unsupported mesh format should fail");

    match error {
        MeshLoadError::UnsupportedFormat {
            path: actual_path,
            extension,
        } => {
            assert_eq!(actual_path, path);
            assert_eq!(extension, "fbx");
        }
        MeshLoadError::Obj { .. } => panic!("expected unsupported format error"),
    }
}
