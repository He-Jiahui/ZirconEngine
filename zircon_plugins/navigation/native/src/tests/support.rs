use zircon_runtime::asset::{
    NavMeshAreaCostAsset, NavMeshAsset, NavMeshLinkAsset, NavMeshPolygonAsset,
};
use zircon_runtime::core::framework::navigation::{AREA_JUMP, AREA_WALKABLE};

pub(super) fn two_island_asset(with_link: bool) -> NavMeshAsset {
    let mut asset = NavMeshAsset {
        version: NavMeshAsset::VERSION,
        agent_type: "humanoid".to_string(),
        settings_hash: 0,
        area_costs: vec![
            NavMeshAreaCostAsset {
                area: AREA_WALKABLE,
                cost: 1.0,
                walkable: true,
            },
            NavMeshAreaCostAsset {
                area: AREA_JUMP,
                cost: 2.0,
                walkable: true,
            },
        ],
        vertices: vec![
            [-1.0, 0.0, -1.0],
            [1.0, 0.0, -1.0],
            [1.0, 0.0, 1.0],
            [-1.0, 0.0, 1.0],
            [7.0, 0.0, -1.0],
            [9.0, 0.0, -1.0],
            [9.0, 0.0, 1.0],
            [7.0, 0.0, 1.0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3, 4, 5, 6, 4, 6, 7],
        polygons: vec![
            NavMeshPolygonAsset {
                first_index: 0,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 0,
            },
            NavMeshPolygonAsset {
                first_index: 6,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 1,
            },
        ],
        tiles: Vec::new(),
        off_mesh_links: Vec::new(),
    };
    if with_link {
        asset.off_mesh_links.push(NavMeshLinkAsset {
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

pub(super) fn corner_touching_fan_polygon_asset() -> NavMeshAsset {
    NavMeshAsset {
        version: NavMeshAsset::VERSION,
        agent_type: "humanoid".to_string(),
        settings_hash: 0,
        area_costs: Vec::new(),
        vertices: vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 0.0, 2.0],
            [0.0, 0.0, 2.0],
            [-1.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
        ],
        indices: vec![0, 1, 2, 0, 2, 3, 0, 4, 5],
        polygons: vec![
            NavMeshPolygonAsset {
                first_index: 0,
                index_count: 6,
                area: AREA_WALKABLE,
                tile: 0,
            },
            NavMeshPolygonAsset {
                first_index: 6,
                index_count: 3,
                area: AREA_WALKABLE,
                tile: 0,
            },
        ],
        tiles: Vec::new(),
        off_mesh_links: Vec::new(),
    }
}
