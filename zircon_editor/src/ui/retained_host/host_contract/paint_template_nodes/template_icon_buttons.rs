use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
#[cfg(test)]
use super::style_selector::WORKBENCH_ICON_PANEL_RADIUS as ICON_PANEL_RADIUS;
use super::style_selector::{
    select_workbench_icon_button_style, WorkbenchIconButtonContext as IconButtonContext,
    WorkbenchIconButtonStyle,
};
use super::template_icon_button_glyphs::push_icon_button_glyph;

pub(super) fn push_icon_button_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_icon_button(node) {
        return false;
    }

    let rect = icon_button_paint_rect(node, rect);
    let context = icon_button_context(node);
    let style = icon_button_style(node, context);
    push_icon_button_surface(commands, &rect, clip, order, style, opacity);
    let glyph = icon_glyph_rect(node, &rect, context);
    push_icon_button_glyph(
        commands,
        node,
        &glyph,
        clip,
        order + 2,
        style.glyph,
        style.state,
        opacity,
    );
    true
}

fn is_workbench_icon_button(node: &TemplatePaneNodeData) -> bool {
    let control_id = node.control_id.as_str();
    is_component_family(node, TemplateComponentFamily::IconButton)
        && uses_workbench_visual_language(node)
        && !control_id.starts_with("WorkbenchStatus")
}

fn icon_button_context(node: &TemplatePaneNodeData) -> IconButtonContext {
    let control_id = node.control_id.as_str();
    if control_id.starts_with("WorkbenchRail") {
        IconButtonContext::Rail
    } else if control_id.starts_with("WorkbenchToolbar")
        || control_id.starts_with("WorkbenchTool")
        || control_id.starts_with("WorkbenchRun")
        || control_id.starts_with("WorkbenchLayout")
        || control_id.starts_with("WorkbenchTheme")
    {
        IconButtonContext::Toolbar
    } else {
        IconButtonContext::Panel
    }
}

fn push_icon_button_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    style: WorkbenchIconButtonStyle,
    opacity: f32,
) {
    let Some(background) = style.background else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        style.border,
        style.border_width,
        style.radius,
        opacity,
    ));
}

fn icon_button_style(
    node: &TemplatePaneNodeData,
    context: IconButtonContext,
) -> WorkbenchIconButtonStyle {
    select_workbench_icon_button_style(node, context)
}

fn icon_button_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

fn icon_glyph_rect(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    context: IconButtonContext,
) -> FrameRect {
    let max_size = rect.width.min(rect.height).max(1.0);
    let default_size = match context {
        IconButtonContext::Rail => (max_size * 0.48).clamp(18.0, 24.0),
        IconButtonContext::Toolbar | IconButtonContext::Panel => {
            (max_size * 0.50).clamp(15.0, 21.0)
        }
    };
    let size = if node.value_number.is_finite() && node.value_number > 0.0 {
        node.value_number
    } else {
        default_size
    }
    .min((max_size - 6.0).max(1.0));
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size,
        height: size,
    }
}

#[cfg(test)]
#[path = "template_icon_buttons_tests.rs"]
mod tests;
