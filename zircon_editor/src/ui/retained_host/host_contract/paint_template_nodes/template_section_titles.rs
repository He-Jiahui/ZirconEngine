use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::template_node_labels::template_node_label;
use super::template_section_title_glyphs::{
    push_section_icon, section_title_icon, SECTION_ICON_GAP, SECTION_ICON_SIZE,
};
#[cfg(test)]
#[path = "template_section_titles_tests.rs"]
mod tests;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SECTION_FONT_SIZE: f32 = 13.0;
const SECTION_LINE_HEIGHT: f32 = SECTION_FONT_SIZE * 1.2;
const SECTION_TEXT_LEFT: f32 = 8.0;
const SECTION_TEXT: [u8; 4] = [225, 236, 240, 255];
const SECTION_TEXT_MUTED: [u8; 4] = [186, 201, 207, 255];
const SECTION_MESH_TEXT: [u8; 4] = [176, 186, 191, 255];

pub(super) fn push_section_title_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_section_title(node) {
        return false;
    }
    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    let icon = section_title_icon(node);
    if let Some(icon) = icon {
        let icon_rect = section_icon_rect(&rect);
        push_section_icon(commands, &icon_rect, clip, order, icon, opacity);
    }
    push_section_label(
        commands,
        node,
        &rect,
        clip,
        order + 2,
        icon.is_some(),
        opacity,
    );
    true
}

fn is_workbench_section_title(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "WorkbenchSectionTitleRoot" | "WorkbenchTransformLabel" | "WorkbenchMeshLabel"
    ) || (node.control_id.as_str().starts_with("Workbench")
        && node.control_id.as_str().ends_with("Title"))
}

fn push_section_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    has_icon: bool,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let x = if has_icon {
        rect.x + SECTION_TEXT_LEFT + SECTION_ICON_SIZE + SECTION_ICON_GAP
    } else {
        rect.x + SECTION_TEXT_LEFT
    };
    let text_rect = FrameRect {
        x,
        y: rect.y + (rect.height - SECTION_LINE_HEIGHT).max(0.0) * 0.5,
        width: (rect.x + rect.width - x - SECTION_TEXT_LEFT).max(1.0),
        height: SECTION_LINE_HEIGHT,
    };
    push_text(
        commands,
        text_rect.clone(),
        clip,
        order,
        &label,
        node,
        opacity,
    );
    if node.font_weight >= 600 {
        push_text(
            commands,
            FrameRect {
                x: text_rect.x + 0.45,
                ..text_rect
            },
            clip,
            order + 1,
            &label,
            node,
            opacity,
        );
    }
}

fn push_text(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        rect,
        Some(clip.clone()),
        order,
        label.to_string(),
        section_text_color(node),
        SECTION_FONT_SIZE,
        SECTION_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn section_icon_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + SECTION_TEXT_LEFT,
        y: rect.y + (rect.height - SECTION_ICON_SIZE).max(0.0) * 0.5,
        width: SECTION_ICON_SIZE,
        height: SECTION_ICON_SIZE,
    }
}

fn section_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if let Some(color) = declared_color(node.label_color) {
        color
    } else if node.control_id == "WorkbenchMeshLabel" {
        SECTION_MESH_TEXT
    } else if matches!(node.text_tone.as_str(), "muted" | "subtle") {
        SECTION_TEXT_MUTED
    } else {
        SECTION_TEXT
    }
}

fn declared_color(color: crate::ui::retained_host::primitives::Color) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
