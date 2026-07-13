use zircon_runtime::core::framework::navigation::{
    NavLinkMotion, AREA_JUMP, AREA_WALKABLE, DEFAULT_AGENT_TYPE,
};
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavMeshLinkAsset, NavMeshLinkCapacity,
};

pub(super) fn two_island_navmesh(with_link: bool) -> NavMeshAsset {
    let mut asset = NavMeshAsset::from_triangle_mesh(
        DEFAULT_AGENT_TYPE,
        vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [7.0, 0.0, -1.0],
            [9.0, 0.0, -1.0],
            [9.0, 0.0, 1.0],
            [7.0, 0.0, 1.0],
        ],
        vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
        AREA_WALKABLE,
    );
    if with_link {
        asset.off_mesh_links.push(NavMeshLinkAsset {
            id: 1,
            owner_entity: 1,
            lane_index: 0,
            capacity: NavMeshLinkCapacity::Unbounded,
            motion: NavLinkMotion::Parabolic,
            arc_height: 1.0,
            start: [1.0, 0.0, 0.0],
            end: [7.0, 0.0, 0.0],
            width: 0.5,
            bidirectional: true,
            area: AREA_JUMP,
            cost_override: None,
            traversal_mode: Default::default(),
        });
    }
    asset
}
