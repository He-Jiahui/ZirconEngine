use std::sync::Arc;

use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiTreeId},
    layout::UiFrame,
    surface::{UiHitRouteNode, UiHitTestEntry, UiHitTestGrid, UiSurfaceFrame},
    tree::UiInputPolicy,
};

use super::super::{build_viewport_toolbar_pointer_layout_with_size, ViewportToolbarPointerBridge};

#[test]
fn stable_frame_identity_skips_reprojecting_toolbar_controls() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    assert!(bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
        ["scene.main"],
        zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
    )));

    let frame = surface_frame(7, true);
    assert!(bridge.sync_surface_frame("scene.main", &frame).unwrap());
    assert_eq!(bridge.controls_by_surface["scene.main"].len(), 1);
    assert!(!bridge.sync_surface_frame("scene.main", &frame).unwrap());

    let mut focus_only_frame = frame.as_ref().clone();
    focus_only_frame.generation = 8;
    let focus_only_frame = Arc::new(focus_only_frame);
    assert!(!bridge
        .sync_surface_frame("scene.main", &focus_only_frame)
        .unwrap());

    let changed_frame_with_same_local_generation = surface_frame(7, false);
    assert!(bridge
        .sync_surface_frame("scene.main", &changed_frame_with_same_local_generation)
        .unwrap());
    assert!(bridge.controls_by_surface["scene.main"].is_empty());

    let next_generation = surface_frame(8, true);
    assert!(bridge
        .sync_surface_frame("scene.main", &next_generation)
        .unwrap());
    assert_eq!(bridge.controls_by_surface["scene.main"].len(), 1);
}

#[test]
fn legacy_zero_generation_never_reuses_the_applied_cursor() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
        ["scene.main"],
        zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
    ));

    assert!(bridge
        .sync_surface_frame("scene.main", &surface_frame(0, true))
        .unwrap());
    assert!(bridge
        .sync_surface_frame("scene.main", &surface_frame(0, false))
        .unwrap());
    assert!(bridge.controls_by_surface["scene.main"].is_empty());
}

#[test]
fn stable_control_identity_publishes_only_the_changed_control_geometry() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
        ["scene.main"],
        zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
    ));
    let initial = surface_frame_with_control_frame(1, UiFrame::new(12.0, 4.0, 24.0, 20.0));
    assert!(bridge.sync_surface_frame("scene.main", &initial).unwrap());

    let moved = surface_frame_with_control_frame(2, UiFrame::new(48.0, 4.0, 24.0, 20.0));
    assert!(bridge.sync_surface_frame("scene.main", &moved).unwrap());

    assert_eq!(bridge.surface.last_rebuild_report.dirty_node_count, 1);
    assert_eq!(
        bridge
            .surface
            .last_rebuild_report
            .arranged_outer_node_visit_count,
        1
    );
    assert_eq!(
        bridge.controls_by_surface["scene.main"][0].action_key,
        "EnterPlayMode"
    );
}

#[test]
fn changed_control_identity_selects_topology_rebuild() {
    let mut bridge = ViewportToolbarPointerBridge::new();
    bridge.sync(build_viewport_toolbar_pointer_layout_with_size(
        ["scene.main"],
        zircon_runtime_interface::ui::layout::UiSize::new(640.0, 28.0),
    ));
    assert!(bridge
        .sync_surface_frame("scene.main", &surface_frame(1, true))
        .unwrap());

    let mut changed = surface_frame(2, true).as_ref().clone();
    changed.hit_grid = Arc::new(UiHitTestGrid {
        entries: vec![UiHitTestEntry {
            control_id: Some("Select".to_string()),
            ..changed.hit_grid.entries[0].clone()
        }]
        .into(),
        ..changed.hit_grid.as_ref().clone()
    });
    let changed = Arc::new(changed);
    let topology_generation = bridge.surface.tree.layout_order_generation();

    assert!(bridge.sync_surface_frame("scene.main", &changed).unwrap());
    assert!(bridge.surface.tree.layout_order_generation() > topology_generation);
    assert_eq!(
        bridge.controls_by_surface["scene.main"][0].action_key,
        "Select"
    );
}

fn surface_frame(generation: u64, with_control: bool) -> Arc<UiSurfaceFrame> {
    let entries = with_control
        .then(|| UiHitTestEntry {
            node_id: UiNodeId::new(1),
            frame: UiFrame::new(12.0, 4.0, 24.0, 20.0),
            clip_frame: UiFrame::new(12.0, 4.0, 24.0, 20.0),
            z_index: 0,
            paint_order: 0,
            control_id: Some("EnterPlayMode".to_string()),
            route_node_index: 0,
        })
        .into_iter()
        .collect::<Vec<_>>();
    Arc::new(UiSurfaceFrame {
        generation,
        tree_id: UiTreeId::new("toolbar"),
        hit_grid: Arc::new(UiHitTestGrid {
            route_nodes: Arc::new(
                with_control
                    .then(|| {
                        vec![UiHitRouteNode {
                            node_id: UiNodeId::new(1),
                            parent_index: UiHitRouteNode::NO_PARENT_INDEX,
                            effective_input_policy: UiInputPolicy::Receive,
                            pointer_path_visible: true,
                            descendant_pointer_path_visible: true,
                            route_valid: true,
                        }]
                    })
                    .unwrap_or_default(),
            ),
            entries: entries.into(),
            ..UiHitTestGrid::default()
        }),
        ..UiSurfaceFrame::default()
    })
}

fn surface_frame_with_control_frame(generation: u64, frame: UiFrame) -> Arc<UiSurfaceFrame> {
    let mut surface_frame = surface_frame(generation, true).as_ref().clone();
    surface_frame.hit_grid = Arc::new(UiHitTestGrid {
        entries: vec![UiHitTestEntry {
            frame,
            clip_frame: frame,
            ..surface_frame.hit_grid.entries[0].clone()
        }]
        .into(),
        ..surface_frame.hit_grid.as_ref().clone()
    });
    Arc::new(surface_frame)
}

#[test]
fn authored_clip_only_delta_updates_the_at_point_toolbar_route() {
    use super::super::ViewportToolbarPointerRoute;
    use std::collections::BTreeSet;
    use zircon_runtime::ui::surface::{
        hit_test_surface_frame, UiAuthoredGeometryPublication, UiSurface,
    };
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodePath, UiStateFlags},
        layout::{UiPoint, UiSize},
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    };

    let size = UiSize::new(120.0, 28.0);
    let key = "scene.inner-clip";
    let root = UiNodeId::new(1);
    let parent = UiNodeId::new(2);
    let button = UiNodeId::new(3);
    let mut source = UiSurface::new(UiTreeId::new("test.scene.toolbar.clip"));
    source.tree.insert_root(
        UiTreeNode::new(root, UiNodePath::new("toolbar"))
            .with_frame(UiFrame::new(0.0, 0.0, size.width, size.height))
            .with_input_policy(UiInputPolicy::Ignore),
    );
    source
        .tree
        .insert_child(
            root,
            UiTreeNode::new(parent, UiNodePath::new("toolbar/clip"))
                .with_frame(UiFrame::new(0.0, 0.0, 45.0, size.height))
                .with_input_policy(UiInputPolicy::Ignore)
                .with_clip_to_bounds(true),
        )
        .unwrap();
    source
        .tree
        .insert_child(
            parent,
            UiTreeNode::new(button, UiNodePath::new("toolbar/clip/button"))
                .with_frame(UiFrame::new(12.0, 4.0, 24.0, 20.0))
                .with_state_flags(UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: true,
                    ..Default::default()
                })
                .with_input_policy(UiInputPolicy::Receive)
                .with_template_metadata(UiTemplateNodeMetadata {
                    control_id: Some("grid.cycle".into()),
                    ..Default::default()
                }),
        )
        .unwrap();
    source.rebuild_authored_frames(size);
    let full = source.surface_frame();
    let publish = |source: &mut UiSurface, width| {
        let topology = source.tree.layout_order_generation();
        source
            .tree
            .node_mut(parent)
            .unwrap()
            .layout_cache
            .frame
            .width = width;
        assert!(matches!(
            source.publish_authored_geometry(size, &BTreeSet::from([parent]), topology),
            UiAuthoredGeometryPublication::Local(_)
        ));
        source.surface_frame()
    };
    let partial = publish(&mut source, 20.0);
    let empty = publish(&mut source, 8.0);
    assert_eq!(
        full.arranged_tree.get(button).unwrap().frame,
        partial.arranged_tree.get(button).unwrap().frame
    );
    assert_eq!(
        full.arranged_tree.get(button).unwrap().frame,
        empty.arranged_tree.get(button).unwrap().frame
    );
    let inside = UiPoint::new(16.0, 14.0);
    let outside = UiPoint::new(30.0, 14.0);
    for (frame, wanted) in [
        (&full, [true, true]),
        (&partial, [true, false]),
        (&empty, [false, false]),
    ] {
        assert_eq!(
            [inside, outside]
                .map(|point| hit_test_surface_frame(frame, point).top_hit == Some(button)),
            wanted,
            "INNER_ADMISSION: actual Runtime hit grid must witness both clip-only changes"
        );
    }
    let mut bridge = ViewportToolbarPointerBridge::new();
    bridge.sync(build_viewport_toolbar_pointer_layout_with_size([key], size));
    let mut observed = Vec::new();
    let mut changes = Vec::new();
    let mut topology_at_full = None;
    for frame in [&full, &partial, &empty] {
        changes.push(bridge.sync_surface_frame(key, frame).unwrap());
        assert!(
            !bridge.sync_surface_frame(key, frame).unwrap(),
            "unchanged grid must skip re-projection"
        );
        let topology = bridge.surface.tree.layout_order_generation();
        assert_eq!(
            *topology_at_full.get_or_insert(topology),
            topology,
            "only one control's geometry changes, not its route/node identity"
        );
        assert_eq!(bridge.controls_by_surface[key].len(), 1);
        observed.push([inside, outside].map(|point| {
            bridge.handle_click_at_point(key, point).unwrap().route
                == Some(ViewportToolbarPointerRoute::CycleGridMode {
                    surface_key: key.into(),
                })
        }));
    }
    eprintln!("SCENE_TOOLBAR_INNER_CLIP_ONLY changes={changes:?} routes={observed:?}");
    assert_eq!(observed[0], [true, true]);
    assert_eq!(
        (changes, observed),
        (
            vec![true, true, true],
            vec![[true, true], [true, false], [false, false]]
        ),
        "SCENE_TOOLBAR_INNER_CLIP_ONLY: actual route must narrow even when raw geometry is stable"
    );
    assert_eq!(
        hit_test_surface_frame(&full, outside).top_hit,
        Some(button),
        "old publication must remain immutable"
    );
}
