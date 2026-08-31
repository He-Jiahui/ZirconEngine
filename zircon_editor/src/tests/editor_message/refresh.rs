use crate::core::editor_event::ViewInstanceId;
use crate::core::editor_message::EditorViewInvalidationMask;
use crate::core::play::WorldDomain;
use crate::ui::host::editor_asset_manager::EditorAssetCatalogGeneration;
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;

use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use std::sync::Arc;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::ui::layout::UiSize;
use zircon_runtime_interface::world_sync::{WatchKey, WatchRegistration};

#[test]
fn data_only_catalog_sync_does_not_publish_workbench_invalidation() {
    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_data_only_catalog_sync");

    harness
        .runtime
        .sync_asset_catalog_data(Arc::new(EditorAssetCatalogGeneration::default()));

    let refresh = harness.runtime.drain_pending_view_refreshes();
    assert!(refresh.dirty().is_empty());
    assert!(refresh.deltas().is_empty());
    assert!(!refresh.used_full_snapshot_fallback());
}

#[test]
fn data_only_resource_sync_does_not_publish_workbench_invalidation() {
    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_data_only_resource_sync");

    assert!(harness.runtime.sync_asset_resources_data(Arc::new(
        zircon_runtime::core::resource::ResourceManagementGeneration::default(),
    )));

    let refresh = harness.runtime.drain_pending_view_refreshes();
    assert!(refresh.dirty().is_empty());
    assert!(refresh.deltas().is_empty());
    assert!(!refresh.used_full_snapshot_fallback());
}

#[test]
fn refresh_view_marks_view_dirty_and_materializes_current_snapshot_backend() {
    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_refresh_view");
    let view = ViewInstanceId::new("scene.workspace");
    let mask =
        EditorViewInvalidationMask::PRESENTATION_DATA.union(EditorViewInvalidationMask::HIT_TEST);

    let report = harness.runtime.refresh_view(view.clone(), mask);

    assert_eq!(report.dirty().len(), 1);
    assert_eq!(report.dirty().mask_for(&view), Some(mask));
    assert!(report.deltas().is_empty());
    assert!(report.used_full_snapshot_fallback());

    let empty_report = harness.runtime.drain_pending_view_refreshes();
    assert!(empty_report.dirty().is_empty());
    assert!(empty_report.deltas().is_empty());
    assert!(!empty_report.used_full_snapshot_fallback());
}

#[test]
fn structure_only_refresh_keeps_the_workbench_snapshot_and_uses_scene_delta_delivery() {
    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_refresh_tree_structure");
    let hierarchy = ViewInstanceId::new("scene.hierarchy");

    let report = harness.runtime.refresh_view(
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    );

    assert_eq!(
        report.dirty().mask_for(&hierarchy),
        Some(EditorViewInvalidationMask::TREE_STRUCTURE)
    );
    assert!(report.deltas().is_empty());
    assert!(
        !report.used_full_snapshot_fallback(),
        "world hierarchy dirtiness must be delivered through the scene-inspection delta channel"
    );
}

#[test]
fn hierarchy_world_watch_delivers_structure_changes_without_reflection_fallback() {
    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_hierarchy_world_watch");
    let hierarchy = ViewInstanceId::new("scene.hierarchy");
    let token = harness
        .runtime
        .watch_world_for_view(
            WorldDomain::Edit,
            WatchRegistration::new(WatchKey::WorldStructure),
            hierarchy.clone(),
            EditorViewInvalidationMask::TREE_STRUCTURE,
        )
        .expect("hierarchy watch should register against the current runtime generation");

    let spawned_entity = {
        let shell = harness.runtime.shell().lock();
        shell.state.world.expect_with_world_mut(|scene| {
            scene
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed")
        })
    };
    let sync = harness
        .runtime
        .pump_world_invalidations(WorldDomain::Edit)
        .expect("world invalidation pump should consume the hierarchy mutation");
    assert_eq!(sync.dirty_views(), 1);
    let refresh = harness.runtime.drain_pending_view_refreshes();
    assert_eq!(
        refresh.dirty().mask_for(&hierarchy),
        Some(EditorViewInvalidationMask::TREE_STRUCTURE)
    );
    assert!(
        !refresh.used_full_snapshot_fallback(),
        "the hierarchy watch must use scene-inspection delivery rather than complete reflection"
    );

    let message = harness
        .runtime
        .take_retained_scene_inspection_message()
        .expect("watch-driven hierarchy dirtiness must publish an inspection message");
    assert!(message
        .added_anchors()
        .iter()
        .any(|anchor| anchor.entity() == spawned_entity));
    drop(token);
}

#[test]
fn hierarchy_dirty_refresh_publishes_and_consumes_a_fragment_without_snapshot_fallback() {
    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_hierarchy_fragment_refresh");
    let hierarchy = ViewInstanceId::new("scene.hierarchy");
    let initial_chrome = harness.runtime.chrome_snapshot();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 720.0))
        .expect("workbench bridge should initialize");
    bridge
        .sync_from_chrome(&initial_chrome)
        .expect("workbench bridge should synchronize its initial hierarchy");

    let spawned_entity = {
        let shell = harness.runtime.shell().lock();
        shell.state.world.expect_with_world_mut(|scene| {
            scene
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed")
        })
    };
    let report = harness
        .runtime
        .refresh_view(hierarchy, EditorViewInvalidationMask::TREE_STRUCTURE);
    assert!(
        !report.used_full_snapshot_fallback(),
        "hierarchy-only refresh must not rebuild the complete workbench reflection"
    );

    let message = harness
        .runtime
        .take_retained_scene_inspection_message()
        .expect("hierarchy-only refresh must publish a scene inspection message");
    assert!(
        message
            .added_anchors()
            .iter()
            .any(|anchor| anchor.entity() == spawned_entity),
        "the publication must retain the added hierarchy entity identity"
    );
    let fragment = harness
        .runtime
        .scene_inspection_hierarchy_fragment(message)
        .expect("published inspection data must resolve into a retained hierarchy fragment");
    let entries = fragment
        .reflow_entries()
        .expect("an added hierarchy entity must request an explicit retained reflow");
    assert!(
        entries.iter().any(|entry| entry.entity == spawned_entity),
        "the retained fragment must contain the added hierarchy row"
    );

    bridge
        .resync_scene_hierarchy(entries)
        .expect("retained bridge should apply the explicit structural hierarchy reflow");
}

#[test]
fn ten_thousand_node_name_refresh_applies_a_patch_without_materializing_sparse_runtime_rows() {
    const NODE_COUNT: usize = 10_000;

    let _lock = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("editor_message_sparse_name_fragment_refresh");
    let hierarchy = ViewInstanceId::new("scene.hierarchy");
    let renamed_entity = {
        let shell = harness.runtime.shell().lock();
        shell.state.world.expect_with_world_mut(|scene| {
            let renamed = scene
                .spawn_node(NodeKind::Empty)
                .expect("test scene spawn should succeed");
            for _ in 1..NODE_COUNT {
                scene
                    .spawn_node(NodeKind::Empty)
                    .expect("test scene spawn should succeed");
            }
            renamed
        })
    };
    let initial_chrome = harness.runtime.chrome_snapshot();
    let mut bridge = BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1280.0, 720.0))
        .expect("workbench bridge should initialize");
    bridge
        .sync_from_chrome(&initial_chrome)
        .expect("initial complete hierarchy projection should synchronize");
    let initial_report = harness.runtime.refresh_view(
        hierarchy.clone(),
        EditorViewInvalidationMask::TREE_STRUCTURE,
    );
    assert!(!initial_report.used_full_snapshot_fallback());
    let initial_fragment = harness
        .runtime
        .take_retained_scene_inspection_message()
        .and_then(|message| harness.runtime.scene_inspection_hierarchy_fragment(message))
        .expect("initial large hierarchy mutation should publish a structural reflow");
    let initial_entries = initial_fragment
        .reflow_entries()
        .expect("initial large hierarchy mutation should require a reflow");
    bridge
        .resync_scene_hierarchy(initial_entries)
        .expect("initial large hierarchy should synchronize before sparse rename coverage");
    let before = {
        let shell = harness.runtime.shell().lock();
        shell
            .state
            .world
            .expect_with_world(|scene| scene.inspection_artifact_diagnostics())
    };

    {
        let shell = harness.runtime.shell().lock();
        shell
            .state
            .world
            .expect_with_world_mut(|scene| {
                scene.rename_node(renamed_entity, "Renamed large scene item")
            })
            .expect("the test entity should remain available for rename");
    }
    let report = harness
        .runtime
        .refresh_view(hierarchy, EditorViewInvalidationMask::TREE_STRUCTURE);
    assert!(!report.used_full_snapshot_fallback());

    let message = harness
        .runtime
        .take_retained_scene_inspection_message()
        .expect("name edit should publish a retained hierarchy message");
    assert_eq!(message.changed_anchors().len(), 1);
    assert!(!message.requires_hierarchy_reflow());
    let fragment = harness
        .runtime
        .scene_inspection_hierarchy_fragment(message)
        .expect("current runtime generation should resolve its sparse patch");
    assert!(fragment.reflow_entries().is_none());
    assert!(fragment.changed_rows().is_some_and(|rows| rows.len() == 1));
    let applied = bridge
        .apply_scene_hierarchy_fragment(&fragment)
        .expect("retained bridge should apply the sparse hierarchy patch");
    assert!(applied.applied());
    assert_eq!(applied.updated_rows(), 1);
    let authoritative_row = harness
        .runtime
        .scene_inspection_hierarchy_row(renamed_entity)
        .expect("sparse hierarchy lookup should resolve the renamed entity");
    assert_eq!(authoritative_row.display_name, "Renamed large scene item");

    let after = {
        let shell = harness.runtime.shell().lock();
        shell
            .state
            .world
            .expect_with_world(|scene| scene.inspection_artifact_diagnostics())
    };
    assert_eq!(
        after.hierarchy_full_materializations(),
        before.hierarchy_full_materializations(),
        "the retained Patch path must not request a complete sparse hierarchy view"
    );
    assert_eq!(
        after.hierarchy_rows_materialized(),
        before.hierarchy_rows_materialized(),
        "the retained Patch path must not copy all hierarchy rows"
    );
}
