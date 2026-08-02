use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::style_selector::is_hot_workbench_table_row_state;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::super::identity::is_table_header;
use super::super::layers::action_icon_order;
use super::super::style::table_row_style;
use super::geometry::{table_action_button_rect, table_action_icon_rect};
use super::glyphs::{push_table_gear, push_table_kebab};
use super::metrics::table_action_metrics;
use super::palette::{WorkbenchTableActionPalette, table_action_palette};
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
    let button_rect = table_action_button_rect(node, rect);
    let action_rect = table_action_icon_rect(&button_rect);
    if !frame_is_within(rect, &button_rect) || !frame_is_within(rect, &action_rect) {
        return;
    }
    if is_table_header(node) {
        push_table_action_button_slot(commands, &button_rect, clip, order, false, opacity);
        if push_icon_asset_pixels(
            commands,
            TABLE_HEADER_ACTION_ICON,
            &action_rect,
            clip,
            action_icon_order(order),
            Some(action_color),
            opacity,
        ) {
            return;
        }
        push_table_gear(
            commands,
            &action_rect,
            clip,
            action_icon_order(order),
            action_color,
            opacity,
        );
    } else {
        if !should_paint_table_row_action(node, style.state) {
            return;
        }
        push_table_action_button_slot(commands, &button_rect, clip, order, true, opacity);
        if push_icon_asset_pixels(
            commands,
            TABLE_ROW_ACTION_ICON,
            &action_rect,
            clip,
            action_icon_order(order),
            Some(action_color),
            opacity,
        ) {
            return;
        }
        push_table_kebab(
            commands,
            &action_rect,
            clip,
            action_icon_order(order),
            action_color,
            opacity,
        );
    }
}

fn frame_is_within(outer: &FrameRect, inner: &FrameRect) -> bool {
    inner.x.is_finite()
        && inner.y.is_finite()
        && inner.width.is_finite()
        && inner.height.is_finite()
        && inner.width > 0.0
        && inner.height > 0.0
        && inner.x >= outer.x
        && inner.y >= outer.y
        && inner.x + inner.width <= outer.x + outer.width
        && inner.y + inner.height <= outer.y + outer.height
}

fn push_table_action_button_slot(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    data_row: bool,
    opacity: f32,
) {
    let metrics = table_action_metrics();
    let palette = table_action_palette();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(table_action_button_background(data_row, palette)),
        Some(palette.slot_border),
        metrics.border_width,
        metrics.radius,
        opacity,
    ));
}

fn table_action_button_background(data_row: bool, palette: WorkbenchTableActionPalette) -> [u8; 4] {
    if data_row {
        palette.data_row_slot_surface
    } else {
        palette.header_slot_surface
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
