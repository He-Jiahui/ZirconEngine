use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::{current_host_palette, HostMaterialPalette};
use super::super::super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

type ChatComposerColors = [[u8; 4]; 2];

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chat_composer(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = super::super::node_radius(node).max(rect.height * 0.5);
    let [surface_color, send_color] = chat_composer_colors_from_host(node, current_host_palette());
    super::super::push_quad(
        commands,
        rect.clone(),
        clip,
        order,
        surface_color,
        1.0,
        radius,
        opacity,
    );
    push_chat_composer_text(commands, node, rect, clip, order + 1, opacity);
    super::super::push_quad(
        commands,
        FrameRect {
            x: rect.x + rect.width - rect.height + 4.0,
            y: rect.y + 4.0,
            width: (rect.height - 8.0).max(1.0),
            height: (rect.height - 8.0).max(1.0),
        },
        clip,
        order + 2,
        send_color,
        0.0,
        rect.height,
        opacity,
    );
}

fn push_chat_composer_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let text = if !node.value_text.trim().is_empty() {
        node.value_text.to_string()
    } else {
        node.text.to_string()
    };
    if text.trim().is_empty() {
        return;
    }

    let metrics = super::super::super::super::paint_theme::current_host_metrics();
    let font_size = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        metrics.font_body
    };
    let line_height = metrics.line_height(font_size).max(font_size);
    let right_reserve = rect.height.max(8.0) + 10.0;
    let available_width = rect.width - right_reserve - 16.0;
    if available_width <= 0.0 {
        return;
    }
    let text_frame = FrameRect {
        x: rect.x + 8.0,
        y: rect.y + ((rect.height - line_height) * 0.5).max(4.0),
        width: available_width,
        height: line_height,
    };
    if text_frame.height > rect.height - 8.0 {
        return;
    }
    let palette = current_host_palette();
    let color = if node.disabled {
        palette.text_disabled
    } else {
        super::super::super::template_style_color::resolved_style_color(
            node.button_style.element.foreground_color.as_ref(),
        )
        .unwrap_or(palette.text)
    };
    commands.push(HostPaintCommand::text(
        text_frame,
        Some(clip.clone()),
        order,
        text,
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn chat_composer_colors_from_host(
    node: &TemplatePaneNodeData,
    palette: HostMaterialPalette,
) -> ChatComposerColors {
    [
        super::super::node_background(node).unwrap_or(palette.surface_inset),
        palette.accent,
    ]
}

#[cfg(test)]
#[path = "tests/composer.rs"]
mod tests;
