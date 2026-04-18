use zircon_ui::UiFrame;

use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::{WorkbenchChromeMetrics, WorkbenchViewModel};

use super::constants::{STRIP_X, TAB_GAP, TAB_HEIGHT, TAB_MIN_WIDTH};
use super::workbench_host_page_pointer_item::WorkbenchHostPagePointerItem;
use super::workbench_host_page_pointer_layout::WorkbenchHostPagePointerLayout;

pub(crate) fn build_workbench_host_page_pointer_layout(
    model: &WorkbenchViewModel,
    metrics: &WorkbenchChromeMetrics,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
) -> WorkbenchHostPagePointerLayout {
    let estimated_width = STRIP_X * 2.0
        + model.host_strip.pages.len() as f32 * TAB_MIN_WIDTH
        + model.host_strip.pages.len().saturating_sub(1) as f32 * TAB_GAP;
    let shared_strip_frame = shared_root_frames.and_then(|frames| frames.host_page_strip_frame);
    let shared_shell_frame = shared_root_frames.and_then(|frames| frames.shell_frame);
    let strip_x = shared_strip_frame
        .map(|frame| frame.x)
        .or_else(|| shared_shell_frame.map(|frame| frame.x))
        .unwrap_or(0.0);
    let strip_y = shared_strip_frame
        .map(|frame| frame.y)
        .or_else(|| {
            shared_shell_frame
                .map(|frame| frame.y + metrics.top_bar_height + metrics.separator_thickness)
        })
        .unwrap_or(0.0);
    let strip_width = shared_strip_frame
        .map(|frame| frame.width.max(1.0))
        .or_else(|| shared_shell_frame.map(|frame| frame.width.max(estimated_width.max(1.0))))
        .unwrap_or(estimated_width.max(1.0));
    let strip_height = shared_strip_frame
        .map(|frame| frame.height.max(0.0))
        .unwrap_or_else(|| metrics.host_bar_height.max(TAB_HEIGHT));
    WorkbenchHostPagePointerLayout {
        strip_frame: UiFrame::new(strip_x, strip_y, strip_width, strip_height),
        items: model
            .host_strip
            .pages
            .iter()
            .map(|page| WorkbenchHostPagePointerItem {
                page_id: page.id.0.clone(),
            })
            .collect(),
    }
}
