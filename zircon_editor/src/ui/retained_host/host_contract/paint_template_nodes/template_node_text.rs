use super::super::data::{FrameRect, HostTextInputFocusData, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_node_images::{is_icon_node, is_icon_only_node, leading_icon_size};
use super::template_node_labels::template_node_label;
use super::template_style::text_color;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DEFAULT_TEMPLATE_FONT_SIZE: f32 = 12.0;
const TEXT_HORIZONTAL_INSET: f32 = 5.0;
const TEXT_VERTICAL_INSET: f32 = 5.0;
const MIN_TEXT_RECT_HEIGHT: f32 = 12.0;

pub(super) fn push_template_text_fallback_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    text_input_focus: Option<&HostTextInputFocusData>,
    property_row_text_painted: bool,
    table_row_text_painted: bool,
    opacity: f32,
) {
    let label = template_node_label(node, text_input_focus);
    if property_row_text_painted
        || table_row_text_painted
        || ((label.is_empty() || is_icon_only_node(node))
            && !matches!(node.role.as_str(), "Label" | "Button"))
    {
        return;
    }

    let text_rect = text_rect_for_node(node, rect);
    let font_size = node_font_size(node, text_rect.height);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: text_rect.x,
            y: text_rect.y,
            width: text_rect.width,
            height: text_rect.height,
        },
        Some(clip.clone()),
        order,
        label,
        text_color(node),
        font_size,
        font_size * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn text_rect_for_node(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let horizontal = TEXT_HORIZONTAL_INSET
        .min((rect.width * 0.25).max(0.0))
        .max(0.0);
    let vertical = TEXT_VERTICAL_INSET
        .min(((rect.height - MIN_TEXT_RECT_HEIGHT) * 0.5).max(1.0))
        .max(0.0);
    let mut x = rect.x + horizontal;
    let mut width = (rect.width - horizontal * 2.0).max(0.0);
    if is_leading_icon_text_node(node) {
        let leading = (leading_icon_size(rect) + TEXT_HORIZONTAL_INSET).min(width);
        x += leading;
        width = (width - leading).max(0.0);
    }
    FrameRect {
        x,
        y: rect.y + vertical,
        width,
        height: (rect.height - vertical * 2.0).max(0.0),
    }
}

fn is_leading_icon_text_node(node: &TemplatePaneNodeData) -> bool {
    is_icon_node(node) && !is_icon_only_node(node) && !template_node_label(node, None).is_empty()
}

fn node_font_size(node: &TemplatePaneNodeData, available_height: f32) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DEFAULT_TEMPLATE_FONT_SIZE
    };
    requested.min(available_height.max(1.0)).max(1.0)
}
