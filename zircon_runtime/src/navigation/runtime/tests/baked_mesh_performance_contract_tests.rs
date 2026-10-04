use super::*;

#[test]
fn adjacency_uses_the_shared_edge_index_not_rectangle_overlap() {
    let shared_edge = polygon_edge_key(10, 11);
    let adjacency = build_adjacency(&[
        test_polygon(vec![shared_edge]),
        test_polygon(vec![shared_edge]),
        test_polygon(vec![polygon_edge_key(20, 21)]),
    ]);

    assert_eq!(adjacency, vec![vec![1], vec![0], Vec::new()]);
}

#[test]
fn triangle_indices_produce_canonical_undirected_edge_keys() {
    assert_eq!(
        polygon_edge_keys(&[7, 3, 5]),
        vec![
            polygon_edge_key(3, 5),
            polygon_edge_key(3, 7),
            polygon_edge_key(5, 7),
        ]
    );
}

#[test]
fn mesh_builder_connects_triangles_through_their_shared_edge() {
    let mesh = BakedNavMesh::new(NavMeshAsset::from_triangle_mesh(
        "fallback-test",
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
        vec![0, 1, 2, 2, 1, 3],
        AREA_WALKABLE,
    ));

    assert_eq!(mesh.adjacency, vec![vec![1], vec![0]]);
}

#[test]
fn area_cost_lookup_is_precomputed_for_astar_edge_expansion() {
    let source = include_str!("../baked_mesh.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("area_costs: [Real; MAX_NAV_AREAS]"));
    assert!(source.contains("self.area_costs[usize::from(area.min"));
    assert!(!source.contains("\n    asset: NavMeshAsset,\n"));
}

#[test]
fn spatial_index_bounds_nearest_and_sample_polygon_candidates() {
    let mesh = dense_grid_mesh(40);
    let position = Vec3::new(17.25, 0.0, 13.25);

    let mut nearest_work = MeshQueryWork::default();
    let nearest = mesh.best_polygon_with_work(position, u64::MAX, &mut nearest_work);
    assert!(nearest.is_some());
    assert!(mesh.polygons[nearest.unwrap()].contains_xz(position));
    assert!(
        nearest_work.polygon_candidates < mesh.polygons.len() / 16,
        "nearest query visited {} of {} polygons",
        nearest_work.polygon_candidates,
        mesh.polygons.len()
    );

    let mut sample_work = MeshQueryWork::default();
    let sample = mesh.sample_position_with_work(position, 0.25, u64::MAX, &mut sample_work);
    assert_eq!(sample.map(|hit| hit.position), Some(position.to_array()));
    assert!(
        sample_work.polygon_candidates < mesh.polygons.len() / 16,
        "sample query visited {} of {} polygons",
        sample_work.polygon_candidates,
        mesh.polygons.len()
    );
}

#[test]
fn spatial_index_work_stays_bounded_from_one_to_one_hundred_thousand_polygons() {
    let cases = [
        (single_triangle_mesh(), Vec3::new(0.25, 0.0, 0.25), 1..=1),
        (
            dense_grid_mesh(23),
            Vec3::new(11.75, 0.0, 11.75),
            1_000..=1_100,
        ),
        (
            dense_grid_mesh(224),
            Vec3::new(112.25, 0.0, 112.25),
            100_000..=101_000,
        ),
    ];
    for (mesh, position, expected_polygon_count) in cases {
        assert!(expected_polygon_count.contains(&mesh.polygons.len()));

        let mut nearest_work = MeshQueryWork::default();
        let nearest = mesh.best_polygon_with_work(position, u64::MAX, &mut nearest_work);
        assert!(
            nearest.is_some(),
            "{}-polygon mesh should resolve a nearest polygon",
            mesh.polygons.len()
        );
        assert!(
            nearest_work.polygon_candidates <= 64,
            "nearest query visited {} candidates in a {}-polygon mesh",
            nearest_work.polygon_candidates,
            mesh.polygons.len()
        );
        assert!(
            nearest_work.bvh_nodes <= 256,
            "nearest query visited {} BVH nodes in a {}-polygon mesh",
            nearest_work.bvh_nodes,
            mesh.polygons.len()
        );

        let mut sample_work = MeshQueryWork::default();
        let sample = mesh.sample_position_with_work(position, 0.25, u64::MAX, &mut sample_work);
        assert!(
            sample.is_some(),
            "{}-polygon mesh should sample its containing polygon",
            mesh.polygons.len()
        );
        assert!(
            sample_work.polygon_candidates <= 64,
            "sample query visited {} candidates in a {}-polygon mesh",
            sample_work.polygon_candidates,
            mesh.polygons.len()
        );
        assert!(
            sample_work.bvh_nodes <= 256,
            "sample query visited {} BVH nodes in a {}-polygon mesh",
            sample_work.bvh_nodes,
            mesh.polygons.len()
        );
    }
}

#[test]
fn query_scratch_uses_epochs_instead_of_clearing_every_polygon_slot() {
    let source = include_str!("../baked_mesh/query_scratch.rs");

    assert!(source.contains("query_epoch"));
    assert!(!source.contains("best_cost.fill("));
    assert!(!source.contains("previous.fill("));
    assert!(!source.contains("visited.fill("));
}

#[test]
fn path_queries_reuse_the_mesh_owned_bounded_scratch_slot() {
    let mesh = dense_grid_mesh(8);
    let first = mesh.find_path(grid_path_query());
    assert_eq!(first.status, NavPathStatus::Complete);
    let first_capacity = {
        let scratch = mesh.query_scratch.lock().unwrap();
        (
            scratch.best_cost.capacity(),
            scratch.previous.capacity(),
            scratch.query_count,
        )
    };

    let second = mesh.find_path(grid_path_query());
    assert_eq!(second.status, NavPathStatus::Complete);
    let scratch = mesh.query_scratch.lock().unwrap();
    assert_eq!(scratch.best_cost.capacity(), first_capacity.0);
    assert_eq!(scratch.previous.capacity(), first_capacity.1);
    assert_eq!(scratch.query_count, first_capacity.2 + 1);
}

#[test]
fn optimization_batch_20260920_runtime861_navigation_path_deduplicates_in_place() {
    const POINT_COUNT: usize = 4_096;
    let mut points = Vec::with_capacity(POINT_COUNT);
    for index in 0..POINT_COUNT {
        points.push(test_path_point(index as Real));
    }
    let input_capacity = points.capacity();

    let deduplicated = deduplicate_path_points(points);

    assert_eq!(deduplicated.len(), POINT_COUNT);
    assert_eq!(deduplicated.capacity(), input_capacity);
}

#[test]
fn optimization_batch_20260920_runtime861_navigation_path_preserves_duplicate_semantics() {
    let mut points = Vec::with_capacity(4);
    points.push(test_path_point(0.0));
    points.push(test_path_point(0.01));
    points.push(test_path_point(1.0));
    points.push(test_path_point(1.01));

    let deduplicated = deduplicate_path_points(points);

    assert_eq!(
        deduplicated
            .iter()
            .map(|point| point.position[0])
            .collect::<Vec<_>>(),
        vec![0.0, 1.0]
    );

    let mut transitive_points = Vec::with_capacity(3);
    transitive_points.push(test_path_point(0.0));
    transitive_points.push(test_path_point(0.03));
    transitive_points.push(test_path_point(0.06));

    let transitive_deduplicated = deduplicate_path_points(transitive_points);

    assert_eq!(
        transitive_deduplicated
            .iter()
            .map(|point| point.position[0])
            .collect::<Vec<_>>(),
        vec![0.0, 0.06]
    );
}

#[test]
fn optimization_batch_20260920_runtime862_navigation_vertex_projection_reserves_index_bound() {
    let mut asset = NavMeshAsset::empty("runtime862");
    asset.vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
    ];
    let index_set = [0, 1, 2, 2, 1, 3];

    let vertices = polygon_vertices(&asset, &index_set);

    assert_eq!(vertices.len(), index_set.len());
    assert!(vertices.capacity() >= index_set.len());
}

#[test]
fn optimization_batch_20260920_runtime862_navigation_vertex_projection_filters_invalid_indices() {
    let mut asset = NavMeshAsset::empty("runtime862");
    asset.vertices = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
    let index_set = [2, 99, 0, 1];

    let vertices = polygon_vertices(&asset, &index_set);

    assert_eq!(
        vertices.iter().map(|vertex| vertex.x).collect::<Vec<_>>(),
        vec![2.0, 0.0, 1.0]
    );
}

#[test]
#[ignore = "performance evidence; run in the managed Windows release lane"]
fn runtime862_navigation_vertex_projection_capacity_bench_v1() {
    const INDEX_COUNT: usize = 4_096;
    let mut asset = NavMeshAsset::empty("runtime862");
    asset.vertices = vec![[0.0, 0.0, 0.0]];
    let index_set = vec![0; INDEX_COUNT];
    let vertices = polygon_vertices(&asset, &index_set);

    println!(
        "RUNTIME862_NAVIGATION_VERTEX_PROJECTION_CAPACITY_BENCH_V1 input_indices={} output_vertices={} capacity={} modeled_legacy_growth_events=11 modeled_reserved_growth_events=0",
        INDEX_COUNT,
        vertices.len(),
        vertices.capacity(),
    );
}

#[test]
#[ignore = "performance evidence; run in the managed Windows release lane"]
fn runtime861_navigation_path_dedup_in_place_bench_v1() {
    const POINT_COUNT: usize = 4_096;
    let mut points = Vec::with_capacity(POINT_COUNT);
    for index in 0..POINT_COUNT {
        points.push(test_path_point(index as Real));
    }
    let deduplicated = deduplicate_path_points(points);

    println!(
        "RUNTIME861_NAVIGATION_PATH_DEDUP_IN_PLACE_BENCH_V1 input_points={} output_points={} capacity={} modeled_legacy_growth_events=11 modeled_in_place_growth_events=0",
        POINT_COUNT,
        deduplicated.len(),
        deduplicated.capacity(),
    );
}

fn test_path_point(x: Real) -> NavPathPoint {
    NavPathPoint {
        position: [x, 0.0, x],
        area: AREA_WALKABLE,
        off_mesh_link_id: None,
        flags: Vec::new(),
    }
}

fn test_polygon(edge_keys: Vec<PolygonEdgeKey>) -> BakedPolygon {
    BakedPolygon {
        area: AREA_WALKABLE,
        center: Vec3::ZERO,
        min: Vec3::ZERO,
        max: Vec3::ZERO,
        edge_keys,
    }
}

fn single_triangle_mesh() -> BakedNavMesh {
    BakedNavMesh::new(NavMeshAsset::from_triangle_mesh(
        "fallback-single-polygon",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        vec![0, 1, 2],
        AREA_WALKABLE,
    ))
}

fn dense_grid_mesh(side: usize) -> BakedNavMesh {
    let mut vertices = Vec::with_capacity((side + 1) * (side + 1));
    for z in 0..=side {
        for x in 0..=side {
            vertices.push([x as Real, 0.0, z as Real]);
        }
    }

    let mut indices = Vec::with_capacity(side * side * 6);
    for z in 0..side {
        for x in 0..side {
            let row = side + 1;
            let lower_left = (z * row + x) as u32;
            let lower_right = lower_left + 1;
            let upper_left = lower_left + row as u32;
            let upper_right = upper_left + 1;
            indices.extend_from_slice(&[
                lower_left,
                lower_right,
                upper_left,
                upper_left,
                lower_right,
                upper_right,
            ]);
        }
    }
    BakedNavMesh::new(NavMeshAsset::from_triangle_mesh(
        "fallback-spatial-index",
        vertices,
        indices,
        AREA_WALKABLE,
    ))
}

fn grid_path_query() -> NavPathQuery {
    NavPathQuery {
        nav_mesh: None,
        start: [0.1, 0.0, 0.1],
        end: [7.5, 0.0, 7.5],
        agent_type: "fallback-spatial-index".to_owned(),
        area_mask: u64::MAX,
    }
}
