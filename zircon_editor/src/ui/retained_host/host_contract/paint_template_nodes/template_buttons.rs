use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_button_style, WorkbenchButtonKind, WorkbenchButtonStyle,
};
use super::template_button_glyphs::{
    button_glyph_for_key, push_button_glyph, ButtonGlyph, BUTTON_ICON_SIZE,
};
use super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const BUTTON_FONT_SIZE: f32 = 12.0;
const BUTTON_LINE_HEIGHT: f32 = BUTTON_FONT_SIZE * 1.2;
const BUTTON_RADIUS: f32 = 7.0;
const BUTTON_TEXT_INSET_X: f32 = 12.0;
const BUTTON_ICON_GAP: f32 = 7.0;
const BUTTON_CHEVRON_RESERVE: f32 = 18.0;
const ADD_COMPONENT_OFFSET_Y: f32 = 1.5;

pub(super) fn push_button_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_button(node) {
        return false;
    }
    let rect = button_paint_rect(node, rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    let kind = button_kind(node);
    let opacity = button_opacity(node, opacity);
    push_button_surface(commands, node, &rect, clip, order, kind, opacity);
    push_button_content(commands, node, &rect, clip, order + 2, kind, opacity);
    true
}

fn is_workbench_button(node: &TemplatePaneNodeData) -> bool {
    let control_id = node.control_id.as_str();
    uses_workbench_visual_language(node)
        && !control_id.starts_with("WorkbenchDrawerTab")
        && !control_id.starts_with("WorkbenchTool")
        && !control_id.starts_with("WorkbenchToolbar")
        && !control_id.starts_with("WorkbenchRail")
        && !control_id.starts_with("WorkbenchStatus")
        && !control_id.starts_with("WorkbenchMini")
        && !control_id.contains("IconButton")
        && is_component_family(node, TemplateComponentFamily::Button)
}

fn button_kind(node: &TemplatePaneNodeData) -> WorkbenchButtonKind {
    let key = button_key(node);
    if key.contains("danger") || key.contains("delete") || key.contains("trash") {
        WorkbenchButtonKind::Danger
    } else if key.contains("primary") || key.contains("filled") || key.contains("accent") {
        WorkbenchButtonKind::Primary
    } else if key.contains("tertiary") || key.contains("text") {
        WorkbenchButtonKind::Tertiary
    } else {
        WorkbenchButtonKind::Secondary
    }
}

fn button_glyph(node: &TemplatePaneNodeData) -> ButtonGlyph {
    button_glyph_for_key(&button_key(node))
}

fn is_add_component_button(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchAddComponent"
}

fn button_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    if is_add_component_button(node) {
        rect.y += ADD_COMPONENT_OFFSET_Y;
    }
    rect
}

fn push_button_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: WorkbenchButtonKind,
    opacity: f32,
) {
    let style = button_style(node, kind);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        style.border_width,
        button_radius(node, rect),
        opacity,
    ));
}

fn push_button_content(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: WorkbenchButtonKind,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    let style = button_style(node, kind);
    let glyph = button_glyph(node);
    let estimated_label_width = label.chars().count() as f32 * BUTTON_FONT_SIZE * 0.56;
    let glyph_width = match glyph {
        ButtonGlyph::Plus | ButtonGlyph::Trash => BUTTON_ICON_SIZE + BUTTON_ICON_GAP,
        ButtonGlyph::ChevronDown | ButtonGlyph::None => 0.0,
    };
    let chevron_width = if glyph == ButtonGlyph::ChevronDown {
        BUTTON_CHEVRON_RESERVE
    } else {
        0.0
    };
    let content_width = (estimated_label_width + glyph_width + chevron_width)
        .min((rect.width - BUTTON_TEXT_INSET_X * 2.0).max(1.0));
    let mut x = rect.x + (rect.width - content_width).max(0.0) * 0.5;

    if matches!(glyph, ButtonGlyph::Plus | ButtonGlyph::Trash) {
        let glyph_rect = FrameRect {
            x,
            y: rect.y + (rect.height - BUTTON_ICON_SIZE).max(0.0) * 0.5,
            width: BUTTON_ICON_SIZE,
            height: BUTTON_ICON_SIZE,
        };
        push_button_glyph(
            commands,
            &glyph_rect,
            clip,
            order,
            glyph,
            style.glyph,
            opacity,
        );
        x += BUTTON_ICON_SIZE + BUTTON_ICON_GAP;
    }

    if !label.trim().is_empty() {
        let text_width = (content_width - glyph_width - chevron_width).max(1.0);
        commands.push(HostPaintCommand::text(
            FrameRect {
                x,
                y: rect.y + (rect.height - BUTTON_LINE_HEIGHT).max(0.0) * 0.5,
                width: text_width,
                height: BUTTON_LINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 1,
            label,
            style.text,
            if node.font_size.is_finite() && node.font_size > 0.0 {
                node.font_size.min(rect.height.max(1.0))
            } else {
                BUTTON_FONT_SIZE
            },
            BUTTON_LINE_HEIGHT,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }

    if glyph == ButtonGlyph::ChevronDown {
        let glyph_rect = FrameRect {
            x: rect.x + rect.width - BUTTON_TEXT_INSET_X - BUTTON_ICON_SIZE,
            y: rect.y + (rect.height - BUTTON_ICON_SIZE).max(0.0) * 0.5,
            width: BUTTON_ICON_SIZE,
            height: BUTTON_ICON_SIZE,
        };
        push_button_glyph(
            commands,
            &glyph_rect,
            clip,
            order,
            glyph,
            style.glyph,
            opacity,
        );
    }
}

fn button_style(node: &TemplatePaneNodeData, kind: WorkbenchButtonKind) -> WorkbenchButtonStyle {
    select_workbench_button_style(node, kind, is_add_component_button(node))
}

fn button_opacity(node: &TemplatePaneNodeData, opacity: f32) -> f32 {
    let declared = node.button_style.element.opacity;
    if declared.is_finite() {
        opacity * declared.clamp(0.0, 1.0)
    } else {
        opacity
    }
}

fn button_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let radius = if node.corner_radius.is_finite() && node.corner_radius > 0.0 {
        node.corner_radius
    } else {
        BUTTON_RADIUS
    };
    radius.min(rect.height * 0.5).max(0.0)
}

fn button_key(node: &TemplatePaneNodeData) -> String {
    format!(
        "{} {} {} {} {} {}",
        node.control_id.as_str(),
        node.text.as_str(),
        node.value_text.as_str(),
        node.button_variant.as_str(),
        node.surface_variant.as_str(),
        node.validation_level.as_str()
    )
    .to_ascii_lowercase()
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}

#[cfg(test)]
#[path = "template_buttons_tests.rs"]
mod tests;
