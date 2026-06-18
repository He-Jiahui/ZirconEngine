use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::render_commands::HostPaintCommand;
use super::template_chip_glyphs::{chip_has_chevron, push_chip_chevron, CHIP_CHEVRON_RESERVE};
#[cfg(test)]
#[path = "template_chips_tests.rs"]
mod tests;
use super::template_node_labels::template_node_label;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const CHIP_FONT_SIZE: f32 = 12.0;
const CHIP_LINE_HEIGHT: f32 = CHIP_FONT_SIZE * 1.2;
const CHIP_RADIUS: f32 = 5.0;
const CHIP_TEXT_LEFT: f32 = 10.0;
const CHIP_TEXT_RIGHT: f32 = 8.0;
const CHIP_SURFACE: [u8; 4] = [31, 38, 44, 255];
const CHIP_HOVER_SURFACE: [u8; 4] = [38, 49, 56, 255];
const CHIP_PRESSED_SURFACE: [u8; 4] = [18, 52, 61, 255];
const CHIP_BORDER: [u8; 4] = [48, 59, 66, 255];
const CHIP_TEXT: [u8; 4] = [211, 223, 228, 255];

pub(super) fn push_chip_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_chip(node) {
        return false;
    }
    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(chip_surface(node)),
        Some(chip_border(node)),
        1.0,
        CHIP_RADIUS,
        opacity,
    ));
    push_chip_label(commands, node, &rect, clip, order + 2, opacity);
    if chip_has_chevron(node) {
        push_chip_chevron(
            commands,
            &rect,
            clip,
            order + 3,
            chip_glyph_color(node),
            opacity,
        );
    }
    true
}

fn is_workbench_chip(node: &TemplatePaneNodeData) -> bool {
    if node.control_id.as_str().starts_with("WorkbenchStatus") {
        return false;
    }
    matches!(node.control_id.as_str(), "WorkbenchChipRoot")
        || matches!(
            node.control_id.as_str(),
            "WorkbenchViewportMode"
                | "WorkbenchViewportLit"
                | "WorkbenchViewportAngle"
                | "WorkbenchViewportSpeed"
        )
        || (node.control_id.as_str().starts_with("Workbench")
            && matches!(node.component_role.as_str(), "chip" | "pill"))
}

fn push_chip_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = template_node_label(node, None);
    if label.trim().is_empty() {
        return;
    }
    let right_reserve = if chip_has_chevron(node) {
        CHIP_CHEVRON_RESERVE
    } else {
        CHIP_TEXT_RIGHT
    };
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + CHIP_TEXT_LEFT,
            y: rect.y + (rect.height - CHIP_LINE_HEIGHT).max(0.0) * 0.5,
            width: (rect.width - CHIP_TEXT_LEFT - right_reserve).max(1.0),
            height: CHIP_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        chip_text_color(node),
        CHIP_FONT_SIZE,
        CHIP_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn chip_surface(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.surface_disabled
    } else if node.pressed || node.popup_open {
        CHIP_PRESSED_SURFACE
    } else if node.hovered || node.focused {
        CHIP_HOVER_SURFACE
    } else {
        CHIP_SURFACE
    }
}

fn chip_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.border_disabled
    } else if node.focused || node.pressed || node.popup_open {
        PALETTE.focus_ring
    } else {
        CHIP_BORDER
    }
}

fn chip_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if matches!(node.text_tone.as_str(), "muted" | "subtle") {
        PALETTE.text_muted
    } else {
        CHIP_TEXT
    }
}

fn chip_glyph_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        PALETTE.text_disabled
    } else if node.focused || node.pressed || node.popup_open {
        PALETTE.focus_ring
    } else {
        PALETTE.text_muted
    }
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(1.0),
        height: rect.height.round().max(1.0),
    }
}
