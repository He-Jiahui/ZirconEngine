use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::template_node_labels::template_node_label;
use super::template_style::resolved_style_color;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const FIELD_FONT_SIZE: f32 = 11.0;
const FIELD_LINE_HEIGHT: f32 = FIELD_FONT_SIZE * 1.25;
const FIELD_RADIUS: f32 = 4.0;
const FIELD_TEXT_LEFT: f32 = 10.0;
const FIELD_TEXT_RIGHT: f32 = 8.0;
const STEPPER_WIDTH: f32 = 18.0;
const STEPPER_DIVIDER: [u8; 4] = [42, 53, 60, 255];
const FIELD_SURFACE: [u8; 4] = [16, 22, 26, 255];
const FIELD_HOVER_SURFACE: [u8; 4] = [20, 27, 31, 255];
const FIELD_FOCUSED_SURFACE: [u8; 4] = [15, 24, 28, 255];
const FIELD_DISABLED_SURFACE: [u8; 4] = [36, 41, 45, 255];
const FIELD_BORDER: [u8; 4] = [50, 63, 71, 255];
const FIELD_FOCUSED_BORDER: [u8; 4] = [27, 152, 160, 255];
const FIELD_DISABLED_BORDER: [u8; 4] = [48, 56, 62, 255];
const FIELD_TEXT: [u8; 4] = [205, 216, 221, 255];
const FIELD_PLACEHOLDER: [u8; 4] = [122, 134, 142, 255];
const FIELD_DISABLED_TEXT: [u8; 4] = [125, 135, 141, 255];

pub(super) fn push_field_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_field(node) {
        return false;
    }
    let rect = field_paint_rect(node, rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }
    let opacity = field_opacity(node, opacity);

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(field_surface(node)),
        Some(field_border(node)),
        1.0,
        FIELD_RADIUS,
        opacity,
    ));

    let stepper = is_stepper_field(node);
    if stepper {
        push_stepper(commands, node, &rect, clip, order + 2, opacity);
    }
    push_field_text(commands, node, &rect, clip, order + 3, stepper, opacity);
    true
}

fn field_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    rect
}

fn is_workbench_field(node: &TemplatePaneNodeData) -> bool {
    uses_workbench_visual_language(node)
        && !node.control_id.as_str().starts_with("WorkbenchTransform")
        && is_component_family(node, TemplateComponentFamily::TextInput)
}

fn push_field_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    stepper: bool,
    opacity: f32,
) {
    let label = field_label(node);
    if label.trim().is_empty() {
        return;
    }
    let right_reserve = if stepper {
        STEPPER_WIDTH + FIELD_TEXT_RIGHT
    } else {
        FIELD_TEXT_RIGHT
    };
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + FIELD_TEXT_LEFT,
            y: rect.y + (rect.height - FIELD_LINE_HEIGHT).max(0.0) * 0.5,
            width: (rect.width - FIELD_TEXT_LEFT - right_reserve).max(1.0),
            height: FIELD_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        field_text_color(node),
        FIELD_FONT_SIZE,
        FIELD_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_stepper(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let left = rect.x + rect.width - STEPPER_WIDTH;
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: left,
            y: rect.y + 4.0,
            width: 1.0,
            height: (rect.height - 8.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        Some(STEPPER_DIVIDER),
        None,
        0.0,
        0.0,
        opacity,
    ));
    let glyph = FrameRect {
        x: left + 4.0,
        y: rect.y + (rect.height - 16.0).max(0.0) * 0.5,
        width: 10.0,
        height: 16.0,
    };
    let color = if node.disabled {
        PALETTE.text_disabled
    } else {
        PALETTE.text_muted
    };
    push_segments(
        commands,
        &glyph,
        clip,
        order + 1,
        color,
        opacity,
        &[
            (4.0, 2.0, 2.0, 2.0),
            (2.0, 4.0, 6.0, 1.4),
            (2.0, 11.0, 6.0, 1.4),
            (4.0, 13.0, 2.0, 2.0),
        ],
    );
}

fn field_surface(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        FIELD_DISABLED_SURFACE
    } else if node.focused || node.control_id.as_str() == "WorkbenchInputFocused" {
        FIELD_FOCUSED_SURFACE
    } else if node.pressed || node.hovered {
        FIELD_HOVER_SURFACE
    } else {
        FIELD_SURFACE
    }
}

fn field_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        FIELD_DISABLED_BORDER
    } else if matches!(node.validation_level.as_str(), "error" | "danger") {
        PALETTE.error
    } else if node.focused || node.control_id.as_str() == "WorkbenchInputFocused" {
        declared_border_color(node).unwrap_or(FIELD_FOCUSED_BORDER)
    } else if node.pressed {
        PALETTE.focus_ring
    } else if node.hovered {
        PALETTE.border
    } else {
        declared_border_color(node).unwrap_or(FIELD_BORDER)
    }
}

fn field_opacity(node: &TemplatePaneNodeData, inherited_opacity: f32) -> f32 {
    (inherited_opacity * node.button_style.element.opacity).clamp(0.0, 1.0)
}

fn declared_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref())
}

fn field_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        FIELD_DISABLED_TEXT
    } else if field_label_is_placeholder(node) {
        FIELD_PLACEHOLDER
    } else {
        FIELD_TEXT
    }
}

fn field_label(node: &TemplatePaneNodeData) -> String {
    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        return label;
    }
    match node.control_id.as_str() {
        "WorkbenchInputDisabled" => "Disabled input".to_string(),
        _ => String::new(),
    }
}

fn field_label_is_placeholder(node: &TemplatePaneNodeData) -> bool {
    template_node_label(node, None).trim().is_empty()
        && node.control_id.as_str() == "WorkbenchInputDisabled"
}

fn is_stepper_field(node: &TemplatePaneNodeData) -> bool {
    node.control_id.as_str() == "WorkbenchInputStepper"
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
    let scale_x = origin.width / 10.0;
    let scale_y = origin.height / 16.0;
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
        height: rect.height.max(1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;

    #[test]
    fn workbench_field_matches_component_fields_but_not_axis_fields() {
        assert!(is_workbench_field(&field_node(
            "WorkbenchInputText",
            "Text field"
        )));
        assert!(is_workbench_field(&field_node("WorkbenchFieldRoot", "")));
        assert!(!is_workbench_field(&field_node(
            "WorkbenchTransformPositionX",
            "128.4"
        )));
    }

    #[test]
    fn workbench_field_paints_surface_border_and_text() {
        let bytes = paint_template_nodes_for_test(
            200,
            48,
            model_rc(vec![positioned_field_node(
                "WorkbenchInputText",
                "Text field",
                12.0,
                8.0,
                170.0,
                32.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 200, 170, 24), FIELD_SURFACE);
        assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_BORDER);
        assert!(changed_pixel_count(&bytes, 200, 22, 16, 64, 18) > 0);
    }

    #[test]
    fn focused_workbench_field_uses_focused_border() {
        let mut node = positioned_field_node(
            "WorkbenchInputFocused",
            "Focused input",
            12.0,
            8.0,
            170.0,
            32.0,
        );
        node.focused = true;
        let bytes = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_FOCUSED_BORDER);
    }

    #[test]
    fn focused_workbench_field_uses_declared_focus_border() {
        let mut node = positioned_field_node(
            "WorkbenchInputFocused",
            "Focused input",
            12.0,
            8.0,
            170.0,
            32.0,
        );
        node.focused = true;
        node.button_style.element.border_color =
            Some(zircon_runtime_interface::ui::style::UiStyleColor::Rgba(
                zircon_runtime_interface::ui::style::UiRgbaColor::from_u8(27, 152, 160, 255),
            ));

        assert_eq!(field_border(&node), [27, 152, 160, 255]);
    }

    #[test]
    fn disabled_workbench_field_paints_placeholder_tone() {
        let mut node = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
        node.disabled = true;
        let text_color = field_text_color(&node);
        let bytes = paint_template_nodes_for_test(200, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 200, 170, 24), FIELD_DISABLED_SURFACE);
        assert_eq!(pixel_at(&bytes, 200, 80, 8), FIELD_DISABLED_BORDER);
        assert_eq!(text_color, FIELD_DISABLED_TEXT);
        assert!(changed_pixel_count(&bytes, 200, 22, 16, 90, 18) > 0);
    }

    #[test]
    fn disabled_workbench_field_uses_declared_opacity() {
        let mut node = positioned_field_node("WorkbenchInputDisabled", "", 12.0, 8.0, 170.0, 32.0);
        node.disabled = true;
        node.button_style.element.opacity = 0.94;

        assert!((field_opacity(&node, 1.0) - 0.94).abs() < 0.001);
        assert!((field_opacity(&node, 0.5) - 0.47).abs() < 0.001);
    }

    #[test]
    fn stepper_workbench_field_paints_right_arrows() {
        let bytes = paint_template_nodes_for_test(
            112,
            48,
            model_rc(vec![positioned_field_node(
                "WorkbenchInputStepper",
                "42",
                12.0,
                8.0,
                67.0,
                32.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 112, 61, 16), STEPPER_DIVIDER);
        assert!(changed_pixel_count(&bytes, 112, 64, 15, 12, 20) > 0);
    }

    #[test]
    fn stepper_workbench_field_honors_declared_layout_offset() {
        let mut node = positioned_field_node("WorkbenchInputStepper", "42", 12.0, 8.0, 67.0, 32.0);
        node.layout_offset_x = 5.0;
        node.layout_offset_y = 6.0;
        let bytes = paint_template_nodes_for_test(128, 72, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 128, 66, 20), STEPPER_DIVIDER);
        assert_eq!(pixel_at(&bytes, 128, 14, 24), [0, 0, 0, 255]);
    }

    #[test]
    fn workbench_field_preserves_half_pixel_declared_height() {
        let rect = pixel_aligned_rect(&FrameRect {
            x: 12.3,
            y: 8.4,
            width: 67.2,
            height: 30.5,
        });

        assert_eq!(rect.x, 12.0);
        assert_eq!(rect.y, 8.0);
        assert_eq!(rect.width, 67.0);
        assert_eq!(rect.height, 30.5);
    }

    fn field_node(control_id: &str, value: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "InputField".into(),
            component_role: "input-field".into(),
            value_text: value.into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 170.0,
                height: 32.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn positioned_field_node(
        control_id: &str,
        value: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            frame: TemplateNodeFrameData {
                x,
                y,
                width,
                height,
            },
            ..field_node(control_id, value)
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
