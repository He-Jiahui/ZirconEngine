use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

use super::build_surface::build_surface;
use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;

pub(crate) fn build_host_drawer_header_pointer_layout(
    model: &WorkbenchViewModel,
) -> HostDrawerHeaderPointerLayout {
    let mut surfaces = Vec::with_capacity(3);
    if let Some(surface) = build_surface(
        "left",
        model,
        &[ActivityDrawerSlot::LeftTop, ActivityDrawerSlot::LeftBottom],
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface(
        "right",
        model,
        &[
            ActivityDrawerSlot::RightTop,
            ActivityDrawerSlot::RightBottom,
        ],
    ) {
        surfaces.push(surface);
    }
    if let Some(surface) = build_surface("bottom", model, &[ActivityDrawerSlot::Bottom]) {
        surfaces.push(surface);
    }

    zircon_runtime::profile_counter!("editor", "ui.drawer_header.receipt_projection_count", 1);
    zircon_runtime::profile_counter!(
        "editor",
        "ui.drawer_header.receipt_projection_surface_count",
        surfaces.len()
    );
    zircon_runtime::profile_counter!(
        "editor",
        "ui.drawer_header.receipt_projection_tab_count",
        surfaces
            .iter()
            .map(|surface| surface.items.len())
            .sum::<usize>()
    );
    HostDrawerHeaderPointerLayout { surfaces }
}
