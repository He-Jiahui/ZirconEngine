use crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::WorkbenchViewModel;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NativeFloatingWindowTarget {
    pub window_id: MainPageId,
    pub title: String,
    pub bounds: [f32; 4],
    pub surface_tree_id: UiTreeId,
}

pub(crate) fn collect_native_floating_window_targets(
    model: &WorkbenchViewModel,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) -> Vec<NativeFloatingWindowTarget> {
    let mut targets = Vec::new();
    for window in &model.floating_windows {
        let Some(frames) = floating_window_projection_bundle.frames(&window.window_id) else {
            continue;
        };
        if !frames.native_host_present {
            continue;
        }
        let Some(surface_tree_id) = frames.surface_tree_id.clone() else {
            continue;
        };
        let frame = frames.outer_frame;
        let bounds = [frame.x, frame.y, frame.width, frame.height];
        if targets.is_empty() {
            targets.reserve_exact(model.floating_windows.len());
        }
        targets.push(NativeFloatingWindowTarget {
            window_id: window.window_id.clone(),
            title: window.title.clone(),
            bounds,
            surface_tree_id,
        });
    }
    targets
}

#[cfg(test)]
#[path = "target/preallocated_targets_tests.rs"]
mod preallocated_targets_tests;
