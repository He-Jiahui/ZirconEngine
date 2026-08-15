use crate::ui::retained_host::host_contract::data::{FrameRect, SceneNodeData};
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_primitives::draw_text_bars_clipped;
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

use super::super::super::super::{ACCENT, MUTED_TEXT};

pub(super) fn draw_hierarchy_row_text(
    frame: &mut HostRgbaFrame,
    row: &FrameRect,
    node: &SceneNodeData,
    row_clip: &FrameRect,
    inline_rename_value: Option<&str>,
) {
    let (text, color) = inline_rename_value
        .map_or_else(|| (node.name.as_str(), MUTED_TEXT), |value| (value, ACCENT));
    let metrics = current_host_metrics();
    draw_text_bars_clipped(
        frame,
        row_text_x(row, node.depth, metrics),
        row_text_y(row, metrics),
        text,
        Some(row_clip),
        color,
    );
}

fn row_text_x(row: &FrameRect, depth: i32, metrics: HostControlMetrics) -> f32 {
    let indent_step = metrics.gap_l + metrics.border_width * 2.0;
    let indent = depth.max(0) as f32 * indent_step;
    row.x + metrics.gap_m + indent.min(row.width.max(0.0) * 0.5)
}

fn row_text_y(row: &FrameRect, metrics: HostControlMetrics) -> f32 {
    row.y + metrics.gap_s.min(row.height.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::{row_text_x, row_text_y};
    use crate::ui::retained_host::host_contract::{data::FrameRect, paint_theme::METRICS};

    #[test]
    fn hierarchy_row_text_offsets_use_host_metrics_and_preserve_depth_budget() {
        let row = FrameRect {
            x: 4.0,
            y: 8.0,
            width: 60.0,
            height: 18.0,
        };

        assert_eq!(row_text_x(&row, 0, METRICS), 12.0);
        assert_eq!(row_text_y(&row, METRICS), 12.0);
        assert_eq!(row_text_x(&row, 99, METRICS), 42.0);
    }
}
