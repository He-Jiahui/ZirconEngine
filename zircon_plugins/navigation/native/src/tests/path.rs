use std::ffi::CStr;

use zircon_runtime::core::framework::navigation::NavMeshAsset;
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
fn ordinary_native_keeps_link_metadata_at_the_attached_start() {
    let asset = two_island_asset(true);
    let owner = crate::detour::DetourQuery::from_asset(&asset).expect("ordinary native owner");
    let filter = crate::asset_ffi::asset_query_filter(&asset);
    require_link_at_first_portal("detour", |query| raw_detour_path(&owner, &filter, query));
}

#[test]
fn tile_cache_keeps_link_metadata_at_the_attached_start() {
    let asset = two_island_asset(true);
    let owner = crate::RecastTileCache::from_asset(&asset).expect("native TileCache owner");
    require_link_at_first_portal("tile_cache", |query| owner.find_path(query));
}

fn raw_detour_path(
    owner: &crate::detour::DetourQuery,
    filter: &NavQueryFilter,
    query: &NavPathQuery,
) -> zircon_runtime::core::framework::navigation::NavPathResult {
    use crate::ffi::{
        zr_nav_detour_find_path, zr_nav_detour_free_path_result, ZrNavDetourPathResult,
    };
    use zircon_runtime::core::framework::navigation::NavPathResult;

    let filter = crate::detour::detour_query_filter(filter);
    let mut result = ZrNavDetourPathResult::default();
    // SAFETY: The owned native handle, endpoint arrays and filter live through this call.
    unsafe {
        zr_nav_detour_find_path(
            owner.as_raw(),
            query.start.as_ptr(),
            query.end.as_ptr(),
            query.area_mask,
            &filter,
            &mut result,
        );
    }
    let status = result.status;
    // SAFETY: The native API initializes this fixed message buffer with a terminated string.
    let message = unsafe { CStr::from_ptr(result.message.as_ptr()) }
        .to_string_lossy()
        .into_owned();
    let raw = if result.points.is_null() || result.point_count == 0 {
        Vec::new()
    } else {
        // SAFETY: Successful native queries own this many path points until free below.
        unsafe { std::slice::from_raw_parts(result.points, result.point_count as usize) }.to_vec()
    };
    let converted = crate::detour_result::convert_path_result(&result);
    // SAFETY: Both observers copied their data before releasing the one native allocation.
    unsafe {
        zr_nav_detour_free_path_result(&mut result);
    }
    match status {
        1 => {
            let path = converted.expect("ordinary native success must return points");
            assert_eq!(path.points.len(), raw.len());
            for (point, original) in path.points.iter().zip(raw) {
                assert_eq!(point.position, original.position);
                assert_eq!(
                    point.off_mesh_link_id,
                    (original.off_mesh_user_id != 0).then_some(original.off_mesh_user_id)
                );
                assert_eq!(
                    point.flags.iter().any(|v| v == "off_mesh_link"),
                    original.flags & 0x04 != 0
                );
            }
            path
        }
        2 => {
            assert_eq!(message, "Detour path query found no complete path");
            assert!(raw.is_empty());
            NavPathResult::no_path()
        }
        _ => panic!("unexpected ordinary native status {status}: {message}"),
    }
}

fn require_link_at_first_portal(
    label: &str,
    query_native: impl Fn(&NavPathQuery) -> zircon_runtime::core::framework::navigation::NavPathResult,
) {
    use zircon_runtime::core::framework::navigation::AREA_JUMP;

    let xz_gap = |a: [f32; 3], b: [f32; 3]| (a[0] - b[0]).hypot(a[2] - b[2]);
    let left = [0.0, 0.0, 0.3];
    let right = [8.0, 0.0, 0.3];
    let mut both_directions = true;
    for (direction, start, destination) in [("forward", left, right), ("reverse", right, left)] {
        let ordinary = query_native(&NavPathQuery::new(start, destination));
        assert_eq!(ordinary.status, NavPathStatus::Complete);
        assert!(ordinary.visited_nodes >= 3 && ordinary.length > 6.0);
        let attached = ordinary.points.iter().find(|point| {
            point.off_mesh_link_id == Some(1) && point.flags.iter().any(|flag| flag == "off_mesh_link")
        }).expect("interior crossing must establish a native attached portal, not just authored coordinates").position;
        assert!(xz_gap(ordinary.points.first().unwrap().position, start) < 0.05);
        assert!(xz_gap(ordinary.points.last().unwrap().position, destination) < 0.05);
        assert!(xz_gap(start, attached) > 0.3 && xz_gap(start, attached) < 1.5);

        // Same immutable native mesh, including both ground owners. Only mask the only link.
        let mut disabled = NavPathQuery::new(attached, destination);
        disabled.area_mask &= !(1_u64 << AREA_JUMP);
        let blocked = query_native(&disabled);
        assert_eq!(blocked.status, NavPathStatus::NoPath);
        let mut same_island = NavPathQuery::new(attached, start);
        same_island.area_mask = disabled.area_mask;
        let local = query_native(&same_island);
        assert_eq!(local.status, NavPathStatus::Complete);
        assert!(xz_gap(local.points.first().unwrap().position, attached) < 1e-4);
        assert!(xz_gap(local.points.last().unwrap().position, start) < 0.05);
        assert!(local
            .points
            .iter()
            .all(|point| point.off_mesh_link_id.is_none()));

        let coincident = query_native(&NavPathQuery::new(attached, destination));
        assert_eq!(coincident.status, NavPathStatus::Complete);
        assert!(coincident.visited_nodes >= 3 && coincident.length > 5.0);
        let gap = xz_gap(coincident.points.first().unwrap().position, attached);
        assert!(gap < 1e-4, "must still start on the same native portal");
        assert!(xz_gap(coincident.points.last().unwrap().position, destination) < 0.05);
        let ids: Vec<_> = coincident
            .points
            .iter()
            .filter_map(|point| point.off_mesh_link_id)
            .collect();
        println!("NATIVE_FIRST_PORTAL label={label} direction={direction} attached={attached:?} gap={gap} corridor={} mask={:?} local={:?} cross={:?} ids={ids:?}",
            coincident.visited_nodes, blocked.status, local.status, coincident.status);
        both_directions &= ids == [1]
            && coincident.points.iter().take(2).any(|point| {
                point.off_mesh_link_id == Some(1)
                    && point.flags.iter().any(|flag| flag == "off_mesh_link")
                    && xz_gap(point.position, attached) < 1e-4
            });
    }
    assert!(both_directions, "the complete native crossing must retain the first attached link marker in both directions");
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

    // SAFETY: 测试数组在同步调用期间存活，计数与数组一致；空连接指针用于验证拒绝路径。
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
    // SAFETY: 结果消息是默认零值初始化的固定数组，原生错误写入以 NUL 终止。
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
