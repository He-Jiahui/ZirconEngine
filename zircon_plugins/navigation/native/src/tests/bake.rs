use zircon_runtime::core::framework::navigation::{NavigationErrorKind, AREA_WALKABLE};
use zircon_runtime::core::math::Real;

use crate::{RecastBackend, RecastBakeMeshInput};

#[test]
fn triangle_mesh_bake_filters_steep_faces_through_recast_rasterization() {
    let backend = RecastBackend;

    let asset = backend
        .bake_triangle_mesh(RecastBakeMeshInput {
            agent_type: "humanoid".to_string(),
            vertices: vec![
                [-2.0, 0.0, -2.0],
                [2.0, 0.0, -2.0],
                [2.0, 0.0, 2.0],
                [-2.0, 0.0, 2.0],
                [6.0, 0.0, -1.0],
                [6.0, 3.0, -1.0],
                [6.0, 3.0, 1.0],
                [6.0, 0.0, 1.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
            triangle_areas: vec![AREA_WALKABLE; 4],
            default_area: AREA_WALKABLE,
        })
        .unwrap();

    assert!(asset.polygons.len() < 4);
    let baked_vertices = asset
        .debug_triangles()
        .iter()
        .flat_map(|triangle| triangle.vertices)
        .collect::<Vec<_>>();
    let min_y = baked_vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(Real::INFINITY, Real::min);
    let max_y = baked_vertices
        .iter()
        .map(|vertex| vertex[1])
        .fold(Real::NEG_INFINITY, Real::max);
    assert!(max_y - min_y < 0.5);
}

#[test]
fn triangle_mesh_bake_rejects_non_finite_vertices_before_native_ffi() {
    let backend = RecastBackend;

    let error = backend
        .bake_triangle_mesh(RecastBakeMeshInput {
            agent_type: "humanoid".to_string(),
            vertices: vec![[0.0, 0.0, 0.0], [Real::NAN, 0.0, 0.0], [0.0, 0.0, 1.0]],
            indices: vec![0, 1, 2],
            triangle_areas: Vec::new(),
            default_area: AREA_WALKABLE,
        })
        .unwrap_err();

    assert_eq!(error.kind, NavigationErrorKind::BakeFailed);
    assert!(error.message.contains("non-finite"));
}
