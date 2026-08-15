use zircon_runtime::core::framework::navigation::{NavAgentTickReport, NavigationManager};

use crate::tests::support::two_island_navmesh;
use crate::DefaultNavigationManager;

#[test]
fn manager_overlay_frame_projects_loaded_navmesh_and_owner_generation() {
    let manager = DefaultNavigationManager::new();
    let first = manager.navigation_overlay_frame(NavAgentTickReport::default());
    assert_eq!(first.owner_generation, 0);
    assert!(first.nav_mesh.triangles.is_empty());

    NavigationManager::load_nav_mesh(&manager, two_island_navmesh(true)).unwrap();
    let frame = manager.navigation_overlay_frame(NavAgentTickReport {
        moved_agents: 3,
        ..NavAgentTickReport::default()
    });
    assert_eq!(frame.owner_generation, 1);
    assert_eq!(frame.nav_mesh.triangles.len(), 4);
    assert_eq!(frame.nav_mesh.off_mesh_links.len(), 1);
    assert_eq!(frame.tick_report.moved_agents, 3);

    NavigationManager::load_nav_mesh(&manager, two_island_navmesh(false)).unwrap();
    assert_eq!(
        manager
            .navigation_overlay_frame(NavAgentTickReport::default())
            .owner_generation,
        2
    );
}
