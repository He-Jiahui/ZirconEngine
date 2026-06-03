use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_dropdown_style, WorkbenchDropdownStyle};
#[cfg(test)]
use super::style_selector::{
    WORKBENCH_DROPDOWN_BORDER as DROPDOWN_BORDER,
    WORKBENCH_DROPDOWN_FOCUS_BORDER as DROPDOWN_FOCUS_BORDER,
    WORKBENCH_DROPDOWN_PLACEHOLDER as DROPDOWN_PLACEHOLDER,
    WORKBENCH_DROPDOWN_SURFACE as DROPDOWN_SURFACE,
};
use super::template_node_labels::template_node_label;
#[cfg(test)]
use zircon_runtime_interface::ui::style::UiPainterResolvedState;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const DROPDOWN_FONT_SIZE: f32 = 11.0;
const DROPDOWN_LINE_HEIGHT: f32 = DROPDOWN_FONT_SIZE * 1.25;
const DROPDOWN_RADIUS: f32 = 4.0;
const DROPDOWN_TEXT_LEFT: f32 = 10.0;
const DROPDOWN_CHEVRON_SIZE: f32 = 14.0;
const DROPDOWN_CHEVRON_RIGHT: f32 = 7.0;
const DROPDOWN_CHEVRON_RESERVE: f32 = DROPDOWN_CHEVRON_SIZE + DROPDOWN_CHEVRON_RIGHT + 4.0;

pub(super) fn push_dropdown_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_dropdown(node) {
        return false;
    }
    let rect = dropdown_paint_rect(node, rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }
    let style = dropdown_style(node);

    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.surface),
        Some(style.border),
        1.0,
        DROPDOWN_RADIUS,
        opacity,
    ));
    push_dropdown_label(commands, node, &rect, clip, order + 2, opacity, &style);
    push_dropdown_chevron(commands, &rect, clip, order + 3, opacity, &style);
    true
}

pub(super) fn dropdown_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let mut rect = pixel_aligned_rect(rect);
    rect.x += node.layout_offset_x;
    rect.y += node.layout_offset_y;
    rect
}

fn is_workbench_dropdown(node: &TemplatePaneNodeData) -> bool {
    uses_workbench_visual_language(node)
        && is_component_family(node, TemplateComponentFamily::Dropdown)
}

fn push_dropdown_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
) {
    let label = dropdown_label(node);
    if label.trim().is_empty() {
        return;
    }
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + DROPDOWN_TEXT_LEFT,
            y: rect.y + (rect.height - DROPDOWN_LINE_HEIGHT).max(0.0) * 0.5,
            width: (rect.width - DROPDOWN_TEXT_LEFT - DROPDOWN_CHEVRON_RESERVE).max(1.0),
            height: DROPDOWN_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        style.text,
        DROPDOWN_FONT_SIZE,
        DROPDOWN_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_dropdown_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    style: &WorkbenchDropdownStyle,
) {
    let chevron = FrameRect {
        x: rect.x + rect.width - DROPDOWN_CHEVRON_RIGHT - DROPDOWN_CHEVRON_SIZE,
        y: rect.y + (rect.height - DROPDOWN_CHEVRON_SIZE).max(0.0) * 0.5,
        width: DROPDOWN_CHEVRON_SIZE,
        height: DROPDOWN_CHEVRON_SIZE,
    };
    push_segments(
        commands,
        &chevron,
        clip,
        order,
        style.chevron,
        opacity,
        &[
            (3.0, 5.0, 2.0, 2.0),
            (5.0, 7.0, 2.0, 2.0),
            (7.0, 5.0, 2.0, 2.0),
        ],
    );
}

fn dropdown_surface(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).surface
}

fn dropdown_border(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).border
}

fn dropdown_chevron_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).chevron
}

fn dropdown_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    dropdown_style(node).text
}

#[cfg(test)]
fn dropdown_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    dropdown_style(node).state
}

fn dropdown_label(node: &TemplatePaneNodeData) -> String {
    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        return label;
    }
    node.options
        .row_data(0)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn dropdown_label_is_placeholder(node: &TemplatePaneNodeData) -> bool {
    template_node_label(node, None).trim().is_empty()
}

fn dropdown_style(node: &TemplatePaneNodeData) -> WorkbenchDropdownStyle {
    select_workbench_dropdown_style(node, dropdown_label_is_placeholder(node))
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
    let scale_x = origin.width / 14.0;
    let scale_y = origin.height / 14.0;
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
    use super::super::super::data::{
        TemplateNodeFrameData, TemplatePaneNodeData, TemplatePaneOptionData,
    };
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use zircon_runtime_interface::ui::style::{
        ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
    };

    #[test]
    fn workbench_dropdown_matches_dropdown_nodes_only() {
        assert!(is_workbench_dropdown(&dropdown_node(false)));
        assert!(is_workbench_dropdown(&TemplatePaneNodeData {
            control_id: "WorkbenchDropdownRoot".into(),
            role: "ComboBox".into(),
            component_role: "combo-box".into(),
            ..TemplatePaneNodeData::default()
        }));
        assert!(!is_workbench_dropdown(&TemplatePaneNodeData {
            control_id: "WorkbenchInputDropdownRow".into(),
            role: "HorizontalGroup".into(),
            ..TemplatePaneNodeData::default()
        }));
    }

    #[test]
    fn closed_workbench_dropdown_paints_surface_border_text_and_chevron() {
        let bytes = paint_template_nodes_for_test(140, 48, model_rc(vec![dropdown_node(false)]));

        assert_eq!(pixel_at(&bytes, 140, 88, 24), DROPDOWN_SURFACE);
        assert_eq!(pixel_at(&bytes, 140, 54, 8), DROPDOWN_BORDER);
        assert!(changed_pixel_count(&bytes, 140, 22, 16, 50, 18) > 0);
        assert!(changed_pixel_count(&bytes, 140, 96, 15, 18, 18) > 0);
    }

    #[test]
    fn open_workbench_dropdown_uses_focus_border_and_keeps_popup_rows() {
        let mut node = dropdown_node(true);
        node.popup_open = true;
        node.structured_options = model_rc(vec![
            option("dropdown", true, false, true, false),
            option("option_a", false, true, false, false),
            option("option_b", false, false, false, true),
        ]);
        let bytes = paint_template_nodes_for_test(160, 140, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 160, 54, 8), DROPDOWN_FOCUS_BORDER);
        assert!(changed_pixel_count(&bytes, 160, 18, 44, 110, 78) > 0);
    }

    #[test]
    fn workbench_dropdown_honors_declared_layout_offset() {
        let mut node = dropdown_node(false);
        node.layout_offset_x = 20.0;
        node.layout_offset_y = 12.0;
        let bytes = paint_template_nodes_for_test(160, 80, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 160, 88, 24), DROPDOWN_SURFACE);
        assert_eq!(pixel_at(&bytes, 160, 54, 8), [0, 0, 0, 255]);
    }

    #[test]
    fn workbench_dropdown_applies_declared_visual_brightness() {
        let mut node = dropdown_node(false);
        node.label_brightness = 1.2;
        let expected_surface = scaled_test_color(DROPDOWN_SURFACE, 1.2);
        let expected_border = scaled_test_color(DROPDOWN_BORDER, 1.2);
        let bytes = paint_template_nodes_for_test(140, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 140, 88, 24), expected_surface);
        assert_eq!(pixel_at(&bytes, 140, 54, 8), expected_border);
    }

    #[test]
    fn workbench_dropdown_preserves_half_pixel_declared_height() {
        let rect = pixel_aligned_rect(&FrameRect {
            x: 12.3,
            y: 8.4,
            width: 95.2,
            height: 30.5,
        });

        assert_eq!(rect.x, 12.0);
        assert_eq!(rect.y, 8.0);
        assert_eq!(rect.width, 95.0);
        assert_eq!(rect.height, 30.5);
    }

    #[test]
    fn workbench_dropdown_uses_declared_style_text_and_chevron_colors() {
        let mut node = dropdown_node(false);
        node.button_style = resolved_background_and_border([32, 38, 42, 255], [31, 39, 46, 255]);
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(127, 138, 145);
        node.icon_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(103, 115, 122);

        assert_eq!(dropdown_surface(&node), [32, 38, 42, 255]);
        assert_eq!(dropdown_border(&node), [31, 39, 46, 255]);
        assert_eq!(dropdown_text_color(&node), [127, 138, 145, 255]);
        assert_eq!(dropdown_chevron_color(&node), [103, 115, 122, 255]);
    }

    #[test]
    fn workbench_dropdown_selector_uses_shared_state_priority() {
        let pressed_open = TemplatePaneNodeData {
            popup_open: true,
            focused: true,
            pressed: true,
            ..dropdown_node(false)
        };
        let disabled_pressed = TemplatePaneNodeData {
            disabled: true,
            pressed: true,
            ..dropdown_node(false)
        };

        assert_eq!(
            dropdown_visual_state(&pressed_open),
            UiPainterResolvedState::Pressed
        );
        assert_eq!(dropdown_surface(&pressed_open), [15, 24, 28, 255]);
        assert_eq!(dropdown_border(&pressed_open), DROPDOWN_FOCUS_BORDER);
        assert_eq!(
            dropdown_visual_state(&disabled_pressed),
            UiPainterResolvedState::Disabled
        );
    }

    fn dropdown_node(focused: bool) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchInputDropdown".into(),
            role: "Dropdown".into(),
            component_role: "dropdown".into(),
            value_text: "Dropdown".into(),
            focused,
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 8.0,
                width: 104.0,
                height: 32.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn resolved_background_and_border(background: [u8; 4], border: [u8; 4]) -> ResolvedButtonStyle {
        ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    background[0],
                    background[1],
                    background[2],
                    background[3],
                ))),
                border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    border[0], border[1], border[2], border[3],
                ))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        }
    }

    fn option(
        id: &str,
        selected: bool,
        hovered: bool,
        special: bool,
        disabled: bool,
    ) -> TemplatePaneOptionData {
        TemplatePaneOptionData {
            id: id.into(),
            label: id.into(),
            selected,
            hovered,
            special,
            disabled,
            ..TemplatePaneOptionData::default()
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

    fn scaled_test_color(color: [u8; 4], brightness: f32) -> [u8; 4] {
        [
            (f32::from(color[0]) * brightness).round().clamp(0.0, 255.0) as u8,
            (f32::from(color[1]) * brightness).round().clamp(0.0, 255.0) as u8,
            (f32::from(color[2]) * brightness).round().clamp(0.0, 255.0) as u8,
            color[3],
        ]
    }
}
