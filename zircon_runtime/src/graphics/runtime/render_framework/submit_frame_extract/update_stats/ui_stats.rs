pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn runtime_ui_graph_pass_order(
    executed_passes: &[String],
    ui_graph_executed_pass_count: usize,
) -> Option<&'static str> {
    if ui_graph_executed_pass_count == 0 {
        return None;
    }
    let mut postprocess = None;
    let mut runtime_ui = None;
    let mut overlay = None;
    for (index, pass) in executed_passes.iter().enumerate() {
        match pass.as_str() {
            "uber" if postprocess.is_none() => postprocess = Some(index),
            "runtime-ui" if runtime_ui.is_none() => runtime_ui = Some(index),
            "overlay-gizmo" if overlay.is_none() => overlay = Some(index),
            _ => {}
        }
    }
    let postprocess = postprocess?;
    let runtime_ui = runtime_ui?;
    let overlay = overlay?;

    if postprocess < overlay && overlay < runtime_ui {
        Some("postprocess-overlay-ui")
    } else if postprocess < runtime_ui && runtime_ui < overlay {
        Some("postprocess-ui-overlay")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
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
}
