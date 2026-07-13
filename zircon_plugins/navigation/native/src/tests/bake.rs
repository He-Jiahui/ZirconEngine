use zircon_runtime::core::framework::navigation::{NavigationErrorKind, AREA_WALKABLE};
use zircon_runtime::core::math::Real;

use crate::{RecastBackend, RecastBakeMeshInput, RecastTiledBakeInput};

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

#[test]
fn tile_bake_matches_simple_bake_geometry() {
    let backend = RecastBackend;
    let mesh = RecastBakeMeshInput {
        agent_type: "humanoid".to_string(),
        vertices: vec![
            [-2.0, 0.0, -2.0],
            [2.0, 0.0, -2.0],
            [2.0, 0.0, 2.0],
            [-2.0, 0.0, 2.0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        triangle_areas: vec![AREA_WALKABLE; 2],
        default_area: AREA_WALKABLE,
    };

    let simple = backend.bake_triangle_mesh(mesh.clone()).unwrap();
    let tiled = backend
        .bake_tiled_mesh(RecastTiledBakeInput {
            mesh,
            tile_size: 2.0,
        })
        .unwrap();

    assert_eq!(tiled.tiles.len(), 4);
    assert!(tiled.polygons.iter().all(|polygon| polygon.tile < 4));
    assert_bounds_close(asset_bounds(&simple), asset_bounds(&tiled), 0.25);
    let simple_area = triangle_area(&simple);
    let tiled_area = triangle_area(&tiled);
    assert!(
        (simple_area - tiled_area).abs() <= 2.0,
        "simple area {simple_area} differs from tiled area {tiled_area}"
    );
}

fn asset_bounds(
    asset: &zircon_runtime::core::framework::navigation::NavMeshAsset,
) -> ([Real; 3], [Real; 3]) {
    let first = asset.vertices[0];
    asset
        .vertices
        .iter()
        .copied()
        .fold((first, first), |(mut min, mut max), vertex| {
            for axis in 0..3 {
                min[axis] = min[axis].min(vertex[axis]);
                max[axis] = max[axis].max(vertex[axis]);
            }
            (min, max)
        })
}

fn assert_bounds_close(
    (actual_min, actual_max): ([Real; 3], [Real; 3]),
    (expected_min, expected_max): ([Real; 3], [Real; 3]),
    tolerance: Real,
) {
    for axis in [0, 2] {
        assert!((actual_min[axis] - expected_min[axis]).abs() <= tolerance);
        assert!((actual_max[axis] - expected_max[axis]).abs() <= tolerance);
    }
}

fn triangle_area(asset: &zircon_runtime::core::framework::navigation::NavMeshAsset) -> Real {
    asset
        .debug_triangles()
        .iter()
        .map(|triangle| {
            let [a, b, c] = triangle.vertices;
            let ab = [b[0] - a[0], b[2] - a[2]];
            let ac = [c[0] - a[0], c[2] - a[2]];
            (ab[0] * ac[1] - ab[1] * ac[0]).abs() * 0.5
        })
        .sum()
}
