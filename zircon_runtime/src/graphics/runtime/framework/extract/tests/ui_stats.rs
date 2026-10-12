use super::runtime_ui_graph_pass_order;

#[test]
fn runtime_ui_graph_pass_order_reports_default_3d_terminal_ui_order() {
    let passes = ["uber", "overlay-gizmo", "runtime-ui"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(
        runtime_ui_graph_pass_order(&passes, 1).as_deref(),
        Some("postprocess-overlay-ui")
    );
}

#[test]
fn runtime_ui_graph_pass_order_preserves_2d_ui_before_overlay_order() {
    let passes = ["uber", "runtime-ui", "overlay-gizmo"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(
        runtime_ui_graph_pass_order(&passes, 1).as_deref(),
        Some("postprocess-ui-overlay")
    );
}

#[test]
fn runtime_ui_graph_pass_order_rejects_unordered_graphs() {
    let unordered = ["runtime-ui", "uber", "overlay-gizmo"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(runtime_ui_graph_pass_order(&unordered, 1), None);
}

#[test]
fn runtime_ui_graph_pass_order_is_absent_without_ui_execution() {
    let passes = ["uber", "overlay-gizmo", "runtime-ui"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    assert_eq!(runtime_ui_graph_pass_order(&passes, 0), None);
}
