use crate::load::mesh::generate_cube_mesh;

#[test]
fn builtin_cube_mesh_has_triangles() {
    let payload = generate_cube_mesh();

    assert_eq!(payload.indices.len() % 3, 0);
    assert!(!payload.vertices.is_empty());
}
