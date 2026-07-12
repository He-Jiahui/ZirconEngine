use std::ffi::CStr;

use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    nav_area_flag, NavPathQuery, NavPathStatus, NavQueryFilter, NavigationErrorKind, AREA_WALKABLE,
    DEFAULT_AREA_MASK,
};

use crate::ffi::{
    zr_nav_detour_create_query, ZrNavDetourQueryCreateResult, ZrNavRecastBakePolygon,
};
use crate::RecastBackend;

use super::support::{corner_touching_fan_polygon_asset, two_island_asset};

#[test]
fn simple_surface_path_uses_baked_asset() {
    let backend = RecastBackend;
    let asset = backend
        .bake_simple_surface(crate::RecastBakeInput {
            agent_type: "humanoid".to_string(),
            source_vertices: 4,
            source_triangles: 2,
            half_extent: 5.0,
        })
        .unwrap();

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [3.0, 0.0, 4.0]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
    assert_eq!(result.length, 5.0);
    assert_eq!(result.points.len(), 2);
}

#[test]
fn area_mask_can_block_walkable_area() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    query.area_mask = DEFAULT_AREA_MASK & !(1_u64 << AREA_WALKABLE);

    let result = backend.find_path(&asset, &query).unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

#[test]
fn disconnected_polygons_return_no_path_without_link() {
    let backend = RecastBackend;
    let asset = two_island_asset(false);

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

#[test]
fn off_mesh_link_bridges_disconnected_polygons() {
    let backend = RecastBackend;
    let asset = two_island_asset(true);

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
    assert!(result
        .points
        .iter()
        .any(|point| point.flags.iter().any(|flag| flag == "off_mesh_link")));
    assert!(result
        .points
        .iter()
        .any(|point| point.off_mesh_link_id == Some(1)));
}

#[test]
fn direct_abi_rejects_missing_off_mesh_link_pointer() {
    let vertices = [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0];
    let indices = [0_u32, 1, 2];
    let polygons = [ZrNavRecastBakePolygon {
        first_index: 0,
        index_count: 3,
        area: AREA_WALKABLE,
        tile: 0,
    }];
    let mut result = ZrNavDetourQueryCreateResult {
        status: 0,
        message: [0; 256],
        query: std::ptr::null_mut(),
        polygon_count: 0,
    };

    unsafe {
        zr_nav_detour_create_query(
            vertices.as_ptr(),
            3,
            indices.as_ptr(),
            indices.len() as u32,
            polygons.as_ptr(),
            polygons.len() as u32,
            std::ptr::null(),
            0,
            std::ptr::null(),
            1,
            &mut result,
        );
    }

    assert!(result.query.is_null());
    let message = unsafe { CStr::from_ptr(result.message.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    assert_eq!(message, "off-mesh link count requires link data");
}

#[test]
fn mismatched_agent_type_returns_structured_error() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    query.agent_type = "large_creature".to_string();

    let error = backend.find_path(&asset, &query).unwrap_err();

    assert_eq!(error.kind, NavigationErrorKind::InvalidConfiguration);
}

#[test]
fn polygon_graph_requires_shared_edge_not_repeated_fan_root() {
    let backend = RecastBackend;
    let asset = corner_touching_fan_polygon_asset();

    let result = backend
        .find_path(
            &asset,
            &NavPathQuery::new([1.0, 0.0, 1.0], [-0.25, 0.0, -0.25]),
        )
        .unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

#[test]
fn area_cost_biases_path_choice() {
    let backend = RecastBackend;
    let asset = super::support::two_route_area_asset();
    let lower_route = NavPathQuery::new([0.5, 0.0, 0.5], [3.5, 0.0, 0.5]);
    let lower_filter = NavQueryFilter::default().with_area_cost(3, 100.0);

    let lower = backend
        .find_path_with_filter(&asset, &lower_route, &lower_filter)
        .unwrap();

    assert_eq!(lower.status, NavPathStatus::Complete);
    assert!(lower.points.iter().any(|point| point.area == 4));
    assert!(!lower.points.iter().any(|point| point.area == 3));

    let upper_route = NavPathQuery::new([0.5, 0.0, 0.5], [3.5, 0.0, 0.5]);
    let upper_filter = NavQueryFilter::default()
        .with_area_cost(4, 100.0)
        .with_area_cost(3, 1.0);

    let upper = backend
        .find_path_with_filter(&asset, &upper_route, &upper_filter)
        .unwrap();

    assert_eq!(upper.status, NavPathStatus::Complete);
    assert!(upper.points.iter().any(|point| point.area == 3));
}

#[test]
fn default_path_query_preserves_baked_area_costs() {
    let backend = RecastBackend;
    let mut asset = super::support::two_route_area_asset();
    asset
        .area_costs
        .iter_mut()
        .find(|cost| cost.area == 3)
        .expect("test asset must define area 3")
        .cost = 100.0;

    let result = backend
        .find_path(&asset, &NavPathQuery::new([0.5, 0.0, 0.5], [3.5, 0.0, 0.5]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
    assert!(result.points.iter().any(|point| point.area == 4));
    assert!(!result.points.iter().any(|point| point.area == 3));
}

#[test]
fn query_filter_flags_can_exclude_walkable_polygons() {
    let backend = RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 5.0);
    let query = NavPathQuery::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let filter = NavQueryFilter {
        exclude_flags: nav_area_flag(AREA_WALKABLE),
        ..NavQueryFilter::default()
    };

    let result = backend
        .find_path_with_filter(&asset, &query, &filter)
        .unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

#[test]
fn fallback_area_cost_is_direction_invariant() {
    let backend = RecastBackend;
    let asset = super::support::two_route_area_fallback_asset();
    let filter = NavQueryFilter::default().with_area_cost(3, 100.0);

    for (start, end) in [
        ([0.5, 0.0, 0.5], [3.5, 0.0, 0.5]),
        ([3.5, 0.0, 0.5], [0.5, 0.0, 0.5]),
    ] {
        let result = backend
            .find_path_with_filter(&asset, &NavPathQuery::new(start, end), &filter)
            .unwrap();

        assert_eq!(result.status, NavPathStatus::Complete);
        assert!(result.points.iter().any(|point| point.area == 4));
        assert!(!result.points.iter().any(|point| point.area == 3));
    }
}
