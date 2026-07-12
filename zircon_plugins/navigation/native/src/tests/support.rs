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

pub(super) fn two_route_area_asset() -> NavMeshAsset {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [3.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [3.0, 0.0, 1.0],
        [4.0, 0.0, 1.0],
        [0.0, 0.0, 2.0],
        [1.0, 0.0, 2.0],
        [3.0, 0.0, 2.0],
        [4.0, 0.0, 2.0],
    ];
    let quads = [
        ([0, 1, 5, 0, 5, 4], AREA_WALKABLE),
        ([4, 5, 9, 4, 9, 8], AREA_WALKABLE),
        ([1, 2, 6, 1, 6, 5], 4),
        ([5, 6, 10, 5, 10, 9], 3),
        ([2, 3, 7, 2, 7, 6], AREA_WALKABLE),
        ([6, 7, 11, 6, 11, 10], AREA_WALKABLE),
    ];
    let mut indices = Vec::new();
    let mut polygons = Vec::new();
    for (quad, area) in quads {
        let first_index = indices.len() as u32;
        indices.extend(quad);
        polygons.push(NavMeshPolygonAsset {
            first_index,
            index_count: 6,
            area,
            tile: 0,
        });
    }
    NavMeshAsset {
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
                area: 3,
                cost: 1.0,
                walkable: true,
            },
            NavMeshAreaCostAsset {
                area: 4,
                cost: 1.0,
                walkable: true,
            },
        ],
        vertices,
        indices,
        polygons,
        tiles: Vec::new(),
        off_mesh_links: Vec::new(),
    }
}

pub(super) fn two_route_area_fallback_asset() -> NavMeshAsset {
    let mut asset = two_route_area_asset();
    // A same-polygon link with an explicit cost keeps the route graph unchanged while forcing
    // the pure-Rust query backend, whose directed area-cost behavior must match Detour.
    asset.off_mesh_links.push(NavMeshLinkAsset {
        start: [0.25, 0.0, 0.25],
        end: [0.5, 0.0, 0.25],
        width: 0.1,
        bidirectional: false,
        area: AREA_WALKABLE,
        cost_override: Some(1.0),
        traversal_mode: Default::default(),
    });
    asset
}
