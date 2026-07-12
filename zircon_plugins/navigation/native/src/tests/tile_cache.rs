use zircon_runtime::asset::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{NavPathQuery, NavPathStatus};

use crate::{RecastNavigationObstacle, RecastTileCache, RecastTileCacheObstacleHandle};

#[test]
fn obstacle_carving_changes_path() {
    let asset = NavMeshAsset::simple_quad("humanoid", 3.0);
    let mut cache = RecastTileCache::from_asset(&asset).expect("tile cache");
    let query = NavPathQuery::new([-2.5, 0.0, 0.0], [2.5, 0.0, 0.0]);
    assert_eq!(cache.find_path(&query).status, NavPathStatus::Complete);

    let handle = cache
        .add_obstacle(RecastNavigationObstacle::box_obstacle(
            [0.0, 0.0, 0.0],
            [0.75, 1.0, 3.5],
        ))
        .expect("obstacle handle");
    assert_ne!(handle, RecastTileCacheObstacleHandle::INVALID);
    cache.update().expect("carving update");

    assert_eq!(cache.find_path(&query).status, NavPathStatus::NoPath);
}

#[test]
fn obstacle_removal_restores_path() {
    let asset = NavMeshAsset::simple_quad("humanoid", 3.0);
    let mut cache = RecastTileCache::from_asset(&asset).expect("tile cache");
    let query = NavPathQuery::new([-2.5, 0.0, 0.0], [2.5, 0.0, 0.0]);
    let handle = cache
        .add_obstacle(RecastNavigationObstacle::box_obstacle(
            [0.0, 0.0, 0.0],
            [0.75, 1.0, 3.5],
        ))
        .expect("obstacle handle");
    cache.update().expect("carving update");
    assert_eq!(cache.find_path(&query).status, NavPathStatus::NoPath);

    cache.remove_obstacle(handle).expect("remove obstacle");
    cache.update().expect("removal update");

    assert_eq!(cache.find_path(&query).status, NavPathStatus::Complete);
}

#[test]
fn tile_cache_flushes_before_detour_request_queue_overflow() {
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let mut cache = RecastTileCache::from_asset(&asset).expect("tile cache");

    for index in 0..65 {
        cache
            .add_obstacle(RecastNavigationObstacle::cylinder(
                [
                    -7.0 + (index % 13) as f32,
                    0.0,
                    5.0 + (index / 13) as f32 * 0.4,
                ],
                0.08,
                1.0,
            ))
            .unwrap_or_else(|| panic!("obstacle {index} must fit in the runtime cache"));
    }
}

#[test]
fn tile_cache_batch_creation_flushes_before_request_queue_overflow() {
    let backend = crate::RecastBackend;
    let asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    let obstacles = (0..65)
        .map(|index| {
            RecastNavigationObstacle::cylinder(
                [
                    -7.0 + (index % 13) as f32,
                    0.0,
                    5.0 + (index / 13) as f32 * 0.4,
                ],
                0.08,
                1.0,
            )
        })
        .collect::<Vec<_>>();
    let query = NavPathQuery::new([-7.0, 0.0, 0.0], [7.0, 0.0, 0.0]);

    let result = backend
        .find_path_with_obstacles(&asset, &query, &obstacles)
        .unwrap();

    assert_eq!(result.status, NavPathStatus::Complete);
}

#[test]
fn obstacle_handle_cannot_cross_tile_cache_ownership() {
    let asset = NavMeshAsset::simple_quad("humanoid", 3.0);
    let mut first = RecastTileCache::from_asset(&asset).expect("first tile cache");
    let mut second = RecastTileCache::from_asset(&asset).expect("second tile cache");
    let first_handle = first
        .add_obstacle(RecastNavigationObstacle::cylinder(
            [0.0, 0.0, 0.0],
            0.25,
            1.0,
        ))
        .expect("first obstacle");

    assert!(second.remove_obstacle(first_handle).is_err());
}
