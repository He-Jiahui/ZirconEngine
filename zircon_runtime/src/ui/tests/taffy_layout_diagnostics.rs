use crate::ui::{surface::UiSurface, tree::UiRuntimeTreeAccessExt};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath, UiTreeId},
    layout::{
        UiContainerKind, UiLayoutEngineBackend, UiLayoutEngineSupport,
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
