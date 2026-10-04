use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        UiContainerKind, UiFrame, UiLayoutEngineBackend, UiLayoutEngineSupport,
        UiLayoutEngineTaffyTreeBuildStats, UiLinearBoxConfig, UiSize,
    },
    tree::UiTreeNode,
};

#[test]
fn taffy_layout_report_exports_transient_tree_build_stats() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.taffy.diagnostics"));
    surface.tree.insert_root(
        UiTreeNode::new(UiNodeId::new(1), UiNodePath::new("root")).with_container(
            UiContainerKind::HorizontalBox(UiLinearBoxConfig { gap: 0.0 }),
        ),
    );
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(2), UiNodePath::new("root/a")),
        )
        .unwrap();
    surface
        .tree
        .insert_child(
            UiNodeId::new(1),
            UiTreeNode::new(UiNodeId::new(3), UiNodePath::new("root/b")),
        )
        .unwrap();

    surface.compute_layout(UiSize::new(120.0, 24.0)).unwrap();

    let report = &surface.layout_engine_report;
    assert_eq!(report.request_count, 1, "{report:#?}");
    assert_eq!(report.taffy_selected_count, 1, "{report:#?}");
    assert_eq!(report.taffy_tree_build_count, 1, "{report:#?}");
    assert_eq!(report.taffy_tree_node_count, 3, "{report:#?}");

    let root = report
        .selections
        .iter()
        .find(|selection| selection.node_id == Some(UiNodeId::new(1)))
        .expect("root layout route selection");
    assert_eq!(root.selected_backend, UiLayoutEngineBackend::Taffy);
    assert_eq!(root.support, UiLayoutEngineSupport::Native);
    assert_eq!(
        root.taffy_tree_build,
        Some(UiLayoutEngineTaffyTreeBuildStats::new(3))
    );
}

#[test]
fn retained_taffy_surface_moves_between_threads_and_reuses_layout() {
    let root_id = UiNodeId::new(1);
    let child_id = UiNodeId::new(2);
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.taffy.thread-transfer"));
    surface.tree.insert_root(
        UiTreeNode::new(root_id, UiNodePath::new("root"))
            .with_container(UiContainerKind::HorizontalBox(Default::default())),
    );
    surface
        .tree
        .insert_child(
            root_id,
            UiTreeNode::new(child_id, UiNodePath::new("root/child")),
        )
        .unwrap();
    surface.compute_layout(UiSize::new(120.0, 24.0)).unwrap();
    assert_eq!(surface.layout_engine_report.taffy_tree_build_count, 1);
    assert_eq!(
        surface.tree.node(child_id).unwrap().layout_cache.frame,
        UiFrame::new(0.0, 0.0, 120.0, 24.0),
    );

    std::thread::spawn(move || {
        surface.compute_layout(UiSize::new(120.0, 24.0)).unwrap();
        assert_eq!(surface.layout_engine_report.taffy_tree_build_count, 0);
        assert_eq!(
            surface.tree.node(child_id).unwrap().layout_cache.frame,
            UiFrame::new(0.0, 0.0, 120.0, 24.0),
        );

        surface.compute_layout(UiSize::new(200.0, 40.0)).unwrap();
        assert_eq!(surface.layout_engine_report.taffy_tree_build_count, 0);
        assert_eq!(
            surface.tree.node(child_id).unwrap().layout_cache.frame,
            UiFrame::new(0.0, 0.0, 200.0, 40.0),
        );
    })
    .join()
    .expect("retained layout remains valid after transfer and resize");
}
