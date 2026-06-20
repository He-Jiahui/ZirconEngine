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
    model
        .floating_windows
        .iter()
        .filter_map(|window| {
            floating_window_projection_bundle
                .frames(&window.window_id)
                .filter(|frames| frames.native_host_present)
                .and_then(|frames| {
                    let surface_tree_id = frames.surface_tree_id.clone()?;
                    let frame = frames.outer_frame;
                    let bounds = [frame.x, frame.y, frame.width, frame.height];
                    Some(NativeFloatingWindowTarget {
                        window_id: window.window_id.clone(),
                        title: window.title.clone(),
                        bounds,
                        surface_tree_id,
                    })
                })
        })
        .collect()
}
