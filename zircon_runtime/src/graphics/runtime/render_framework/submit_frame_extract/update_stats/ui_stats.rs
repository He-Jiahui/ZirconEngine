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
        if postprocess.is_some() && runtime_ui.is_some() && overlay.is_some() {
            break;
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
#[path = "tests/ui_stats.rs"]
mod tests;

#[cfg(test)]
#[path = "ui_stats/tests/early_exit_tests.rs"]
mod early_exit_tests;
