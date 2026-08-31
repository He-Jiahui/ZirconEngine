use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::geometry::{frame_is_within, has_paintable_field_extent};
use super::super::metrics::workbench_field_metrics;

const SEARCH_FIELD_ICON: &str = "search";
const SEARCH_FIELD_CLEAR_ICON: &str = "close-outline";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_search_field_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    color: [u8; 4],
) {
    if !super::is_search_field(node) {
        return;
    }

    let Some(icon) = search_icon_rect(rect) else {
        return;
    };
    push_icon_asset_pixels(
        commands,
        SEARCH_FIELD_ICON,
        &icon,
        clip,
        order,
        Some(color),
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_search_field_clear_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    color: [u8; 4],
) {
    let Some(action) = super::search_field_clear_action_rect(node, rect) else {
        return;
    };
    push_icon_asset_pixels(
        commands,
        SEARCH_FIELD_CLEAR_ICON,
        &action,
        clip,
        order,
        Some(color),
        opacity,
    );
}

fn search_icon_rect(rect: &FrameRect) -> Option<FrameRect> {
    if !has_paintable_field_extent(rect) {
        return None;
    }
    let metrics = workbench_field_metrics();
    let icon_left = rect.x + metrics.input_pad_left;
    let icon_top = rect.y + (rect.height - metrics.search_icon_size).max(0.0) * 0.5;
    let icon = FrameRect {
        x: icon_left,
        y: icon_top,
        width: metrics.search_icon_size,
        height: metrics.search_icon_size,
    };
    frame_is_within(&icon, rect).then_some(icon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_icon_preserves_fractional_post_dpi_origin() {
        let rect = FrameRect {
            x: 10.25,
            y: 20.5,
            width: 160.0,
            height: 31.25,
        };
        let metrics = workbench_field_metrics();

        let icon = search_icon_rect(&rect).expect("search icon frame");

        assert_eq!(icon.x, rect.x + metrics.input_pad_left);
        assert_eq!(
            icon.y,
            rect.y + (rect.height - metrics.search_icon_size).max(0.0) * 0.5
        );
        assert_ne!(icon.x.fract(), 0.0);
        assert_ne!(icon.y.fract(), 0.0);
    }
}
