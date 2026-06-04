use zircon_runtime::core::framework::navigation::{NavRaycastQuery, DEFAULT_AREA_MASK};

use crate::RecastBackend;

use super::support::two_island_asset;

#[test]
fn raycast_ignores_offmesh_links_for_straight_visibility() {
    let backend = RecastBackend;
    let asset = two_island_asset(true);

    let result = backend
        .raycast(
            &asset,
            &NavRaycastQuery {
                nav_mesh: None,
                start: [0.0, 0.0, 0.0],
                end: [8.0, 0.0, 0.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap();

    assert!(result.hit);
}

#[test]
fn raycast_reports_gap_between_connected_islands_as_hit() {
    let backend = RecastBackend;
    let asset = two_island_asset(false);

    let result = backend
        .raycast(
            &asset,
            &NavRaycastQuery {
                nav_mesh: None,
                start: [0.0, 0.0, 0.0],
                end: [1.5, 0.0, 0.0],
                agent_type: "humanoid".to_string(),
                area_mask: DEFAULT_AREA_MASK,
            },
        )
        .unwrap();

    assert!(result.hit);
    assert!(result.position[0] > 1.0);
}
