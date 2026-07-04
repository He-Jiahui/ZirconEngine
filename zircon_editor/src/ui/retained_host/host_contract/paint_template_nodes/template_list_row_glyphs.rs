use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_icon_assets::push_icon_asset_pixels;
use super::template_row_metrics::workbench_row_palette;

mod geometry;
mod segments;
mod selection;
mod shapes;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use selection::{
    list_row_adornment_kind, ListRowAdornmentKind,
};

const LIST_ROW_CHECK_ICON: &str = "zircon_editor_shell/controls/check.svg";
const LIST_ROW_CHEVRON_ICON: &str = "zircon_editor_shell/toolbar/chevron-right.svg";
const LIST_ROW_DISABLED_ICON: &str = "zircon_editor_shell/status/disabled.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_list_row_adornment(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let adornment = geometry::list_row_adornment_rect(rect);
    match list_row_adornment_kind(node) {
        ListRowAdornmentKind::Check => {
            if push_icon_asset_pixels(
                commands,
                LIST_ROW_CHECK_ICON,
                &adornment,
                clip,
                order,
                Some(color),
                opacity,
            ) {
                return;
            }
            shapes::push_check_mark(commands, &adornment, clip, order, color, opacity);
        }
        ListRowAdornmentKind::Chevron => {
            if push_icon_asset_pixels(
                commands,
                LIST_ROW_CHEVRON_ICON,
                &adornment,
                clip,
                order,
                Some(color),
                opacity,
            ) {
                return;
            }
            shapes::push_right_chevron(commands, &adornment, clip, order, color, opacity);
        }
        ListRowAdornmentKind::DisabledDiamond => {
            let palette = workbench_row_palette();
            if push_icon_asset_pixels(
                commands,
                LIST_ROW_DISABLED_ICON,
                &adornment,
                clip,
                order,
                Some(palette.disabled_adornment_tint),
                opacity,
            ) {
                return;
            }
            shapes::push_disabled_diamond(commands, &adornment, clip, order, opacity);
        }
    }
}
