use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::is_hot_workbench_table_row_state;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::identity::is_table_header;
use super::super::style::table_row_style;
use super::geometry::table_action_rect;
use super::glyphs::{push_table_gear, push_table_kebab};
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

const TABLE_HEADER_ACTION_ICON: &str = "zircon_editor_shell/activity/settings.svg";
const TABLE_ROW_ACTION_ICON: &str = "zircon_editor_shell/toolbar/more-horizontal.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_table_action(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let style = table_row_style(node);
    let action_color = style.action;
    let action_rect = table_action_rect(node, rect);
    if is_table_header(node) {
        if push_icon_asset_pixels(
            commands,
            TABLE_HEADER_ACTION_ICON,
            &action_rect,
            clip,
            order,
            Some(action_color),
            opacity,
        ) {
            return;
        }
        push_table_gear(commands, &action_rect, clip, order, action_color, opacity);
    } else {
        if !should_paint_table_row_action(node, style.state) {
            return;
        }
        if push_icon_asset_pixels(
            commands,
            TABLE_ROW_ACTION_ICON,
            &action_rect,
            clip,
            order,
            Some(action_color),
            opacity,
        ) {
            return;
        }
        push_table_kebab(commands, &action_rect, clip, order, action_color, opacity);
    }
}

fn should_paint_table_row_action(
    node: &TemplatePaneNodeData,
    state: UiPainterResolvedState,
) -> bool {
    node.selected
        || node.checked
        || is_hot_workbench_table_row_state(state)
        || matches!(
            state,
            UiPainterResolvedState::Pressed
                | UiPainterResolvedState::Selected
                | UiPainterResolvedState::Checked
        )
}
