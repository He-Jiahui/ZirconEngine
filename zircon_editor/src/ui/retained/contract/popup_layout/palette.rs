//! Shared internal CommandPalette frames for native painting, pointer and keyboard routing.

#[path = "palette/metrics.rs"]
mod metrics;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_geometry::intersect;

pub(in crate::ui::retained_host::host_contract) use metrics::{
    command_palette_metrics, command_palette_metrics_from_host, WorkbenchCommandPaletteMetrics,
};

pub(in crate::ui::retained_host::host_contract) fn is_command_palette(
    node: &TemplatePaneNodeData,
) -> bool {
    node.role.as_str() == "CommandPalette" || node.component_role.as_str() == "command-palette"
}

pub(in crate::ui::retained_host::host_contract) fn command_palette_search_frame(
    panel: &FrameRect,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: panel.x + metrics.panel_padding_x,
        y: panel.y + metrics.search_top,
        width: (panel.width - metrics.panel_padding_x * 2.0).max(metrics.min_frame_extent),
        height: metrics.search_height,
    }
}

pub(in crate::ui::retained_host::host_contract) fn command_palette_option_frame(
    panel: &FrameRect,
    row: usize,
) -> FrameRect {
    let metrics = command_palette_metrics();
    FrameRect {
        x: panel.x + metrics.row_inset_x,
        y: panel.y + metrics.list_top + row as f32 * metrics.row_height,
        width: (panel.width - metrics.row_inset_x * 2.0).max(metrics.min_frame_extent),
        height: metrics.row_height,
    }
}

pub(in crate::ui::retained_host::host_contract) fn command_palette_option_frame_within(
    panel: &FrameRect,
    row: usize,
    bounds: &FrameRect,
) -> Option<FrameRect> {
    intersect(
        &intersect(&command_palette_option_frame(panel, row), panel)?,
        bounds,
    )
}
