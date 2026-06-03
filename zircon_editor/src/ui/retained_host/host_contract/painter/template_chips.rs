use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_node_labels::template_node_label;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const CHIP_FONT_SIZE: f32 = 12.0;
const CHIP_LINE_HEIGHT: f32 = CHIP_FONT_SIZE * 1.2;
const CHIP_RADIUS: f32 = 5.0;
const CHIP_TEXT_LEFT: f32 = 10.0;
const CHIP_TEXT_RIGHT: f32 = 8.0;
const CHIP_CHEVRON_SIZE: f32 = 12.0;
const CHIP_CHEVRON_RIGHT: f32 = 8.0;
const CHIP_CHEVRON_RESERVE: f32 = CHIP_CHEVRON_SIZE + CHIP_CHEVRON_RIGHT + 4.0;
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
        push_chip_chevron(commands, node, &rect, clip, order + 3, opacity);
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

fn push_chip_chevron(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let chevron = FrameRect {
        x: rect.x + rect.width - CHIP_CHEVRON_RIGHT - CHIP_CHEVRON_SIZE,
        y: rect.y + (rect.height - CHIP_CHEVRON_SIZE).max(0.0) * 0.5,
        width: CHIP_CHEVRON_SIZE,
        height: CHIP_CHEVRON_SIZE,
    };
    push_segments(
        commands,
        &chevron,
        clip,
        order,
        chip_glyph_color(node),
        opacity,
        &[
            (3.0, 4.0, 2.0, 2.0),
            (5.0, 6.0, 2.0, 2.0),
            (7.0, 4.0, 2.0, 2.0),
        ],
    );
}

fn chip_has_chevron(node: &TemplatePaneNodeData) -> bool {
    node.popup_open
        || node.options.row_count() > 0
        || matches!(
            node.control_id.as_str(),
            "WorkbenchViewportMode" | "WorkbenchViewportAngle" | "WorkbenchViewportSpeed"
        )
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

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[(f32, f32, f32, f32)],
) {
    for (x, y, width, height) in segments {
        commands.push(HostPaintCommand::quad(
            scaled_rect(origin, *x, *y, *width, *height),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn scaled_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    let scale_x = origin.width / 12.0;
    let scale_y = origin.height / 12.0;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
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

#[cfg(test)]
mod tests {
    use super::super::super::data::{TemplateNodeFrameData, TemplatePaneNodeData};
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn workbench_chip_matches_viewport_chips_but_not_status_chips() {
        assert!(is_workbench_chip(&chip_node(
            "WorkbenchViewportMode",
            "Perspective"
        )));
        assert!(is_workbench_chip(&chip_node("WorkbenchChipRoot", "Chip")));
        assert!(!is_workbench_chip(&chip_node(
            "WorkbenchStatusGrid",
            "Grid: 10 cm"
        )));
    }

    #[test]
    fn viewport_chip_paints_surface_border_text_and_chevron() {
        let bytes = paint_template_nodes_for_test(
            150,
            48,
            model_rc(vec![chip_node("WorkbenchViewportMode", "Perspective")]),
        );

        assert_eq!(pixel_at(&bytes, 150, 110, 24), CHIP_SURFACE);
        assert_eq!(pixel_at(&bytes, 150, 54, 8), CHIP_BORDER);
        assert!(changed_pixel_count(&bytes, 150, 22, 16, 62, 18) > 0);
        assert!(changed_pixel_count(&bytes, 150, 102, 15, 18, 18) > 0);
    }

    #[test]
    fn focused_chip_uses_focus_border() {
        let mut node = chip_node("WorkbenchViewportAngle", "10 deg");
        node.focused = true;
        let bytes = paint_template_nodes_for_test(120, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 120, 54, 8), PALETTE.focus_ring);
    }

    fn chip_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Label".into(),
            text: text.into(),
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 8.0,
                width: 104.0,
                height: 30.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn changed_pixel_count(
        bytes: &[u8],
        frame_width: u32,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> usize {
        let mut changed = 0;
        for py in y..(y + height) {
            for px in x..(x + width) {
                let index = ((py as usize * frame_width as usize) + px as usize) * 4;
                if bytes[index..index + 4] != [0, 0, 0, 255] {
                    changed += 1;
                }
            }
        }
        changed
    }

    fn pixel_at(bytes: &[u8], frame_width: u32, x: u32, y: u32) -> [u8; 4] {
        let index = ((y as usize * frame_width as usize) + x as usize) * 4;
        [
            bytes[index],
            bytes[index + 1],
            bytes[index + 2],
            bytes[index + 3],
        ]
    }
}
