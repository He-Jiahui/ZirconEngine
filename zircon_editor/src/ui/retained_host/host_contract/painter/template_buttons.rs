use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_button_style, WorkbenchButtonKind, WorkbenchButtonStyle,
};
#[cfg(test)]
use super::style_selector::{
    ADD_COMPONENT_GLYPH, ADD_COMPONENT_TEXT, OUTLINED_BORDER, OUTLINED_SURFACE, OUTLINED_TEXT,
    PRIMARY_SURFACE,
};
use super::template_node_labels::template_node_label;
#[cfg(test)]
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const BUTTON_FONT_SIZE: f32 = 12.0;
const BUTTON_LINE_HEIGHT: f32 = BUTTON_FONT_SIZE * 1.2;
const BUTTON_RADIUS: f32 = 7.0;
const BUTTON_TEXT_INSET_X: f32 = 12.0;
const BUTTON_ICON_SIZE: f32 = 14.0;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ButtonGlyph {
    None,
    Plus,
    Trash,
    ChevronDown,
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
    let key = button_key(node);
    if key.contains("delete") || key.contains("trash") || key.contains("danger") {
        ButtonGlyph::Trash
    } else if key.contains("dropdown") || key.contains("drop-down") || key.contains("menu") {
        ButtonGlyph::ChevronDown
    } else if key.contains("icon") || key.contains("add") || key.contains("plus") {
        ButtonGlyph::Plus
    } else {
        ButtonGlyph::None
    }
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

fn push_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    glyph: ButtonGlyph,
    color: [u8; 4],
    opacity: f32,
) {
    match glyph {
        ButtonGlyph::Plus => push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[(6.0, 2.0, 2.0, 10.0), (2.0, 6.0, 10.0, 2.0)],
        ),
        ButtonGlyph::Trash => push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[
                (3.0, 4.0, 8.0, 1.2),
                (4.0, 2.0, 6.0, 1.2),
                (4.0, 5.0, 1.2, 7.0),
                (9.0, 5.0, 1.2, 7.0),
                (5.0, 12.0, 4.0, 1.2),
            ],
        ),
        ButtonGlyph::ChevronDown => push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[
                (3.0, 5.0, 2.0, 2.0),
                (5.0, 7.0, 4.0, 2.0),
                (9.0, 5.0, 2.0, 2.0),
            ],
        ),
        ButtonGlyph::None => {}
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
    let scale_x = origin.width / BUTTON_ICON_SIZE;
    let scale_y = origin.height / BUTTON_ICON_SIZE;
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
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use zircon_runtime_interface::ui::style::{
        ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
    };

    #[test]
    fn workbench_button_matches_button_nodes_without_icon_or_tab_nodes() {
        assert!(is_workbench_button(&button_node(
            "WorkbenchPrimaryButton",
            "Primary",
            "filled"
        )));
        assert!(is_workbench_button(&button_node(
            "WorkbenchButtonRoot",
            "Button",
            "outlined"
        )));
        assert!(!is_workbench_button(&button_node(
            "WorkbenchDrawerTabComponents",
            "UI Components",
            "tab"
        )));
        assert!(!is_workbench_button(&TemplatePaneNodeData {
            control_id: "WorkbenchMiniAdd".into(),
            role: "IconButton".into(),
            ..TemplatePaneNodeData::default()
        }));
    }

    #[test]
    fn primary_workbench_button_paints_filled_surface_and_center_text() {
        let bytes = paint_template_nodes_for_test(
            152,
            48,
            model_rc(vec![positioned_button_node(
                "WorkbenchPrimaryButton",
                "Primary",
                "filled",
                12.0,
                8.0,
                120.0,
                34.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 152, 24, 24), PRIMARY_SURFACE);
        assert!(changed_pixel_count(&bytes, 152, 48, 16, 56, 18) > 0);
        assert_eq!(pixel_at(&bytes, 152, 140, 24), [0, 0, 0, 255]);
    }

    #[test]
    fn outlined_workbench_button_paints_dark_surface_and_border() {
        let bytes = paint_template_nodes_for_test(
            152,
            48,
            model_rc(vec![positioned_button_node(
                "WorkbenchSecondaryButton",
                "Secondary",
                "outlined",
                12.0,
                8.0,
                120.0,
                34.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 152, 24, 24), OUTLINED_SURFACE);
        assert_eq!(pixel_at(&bytes, 152, 72, 8), OUTLINED_BORDER);
        assert!(changed_pixel_count(&bytes, 152, 42, 16, 70, 18) > 0);
    }

    #[test]
    fn disabled_workbench_button_uses_disabled_surface_and_text() {
        let mut node = positioned_button_node(
            "WorkbenchDisabledButton",
            "Disabled",
            "outlined",
            12.0,
            8.0,
            120.0,
            34.0,
        );
        node.disabled = true;
        let bytes = paint_template_nodes_for_test(152, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 152, 24, 24), PALETTE.surface_disabled);
        assert_eq!(pixel_at(&bytes, 152, 72, 8), PALETTE.border_disabled);
        assert!(changed_pixel_count(&bytes, 152, 45, 16, 62, 18) > 0);
    }

    #[test]
    fn disabled_workbench_button_uses_declared_style_and_opacity() {
        let mut node = positioned_button_node(
            "WorkbenchDisabledButton",
            "Disabled",
            "outlined",
            12.0,
            8.0,
            120.0,
            34.0,
        );
        node.disabled = true;
        node.button_style = resolved_button_style(
            [45, 51, 55, 255],
            [52, 61, 68, 255],
            [116, 127, 134, 255],
            0.72,
        );

        let style = button_style(&node, button_kind(&node));

        assert_eq!(style.surface, [45, 51, 55, 255]);
        assert_eq!(style.border, [52, 61, 68, 255]);
        assert_eq!(style.text, [116, 127, 134, 255]);
        assert_eq!(style.glyph, [116, 127, 134, 255]);
        assert!((button_opacity(&node, 1.0) - 0.72).abs() < 0.001);
    }

    #[test]
    fn dropdown_workbench_button_paints_trailing_chevron() {
        let bytes = paint_template_nodes_for_test(
            152,
            48,
            model_rc(vec![positioned_button_node(
                "WorkbenchDropdownButton",
                "Dropdown",
                "outlined",
                12.0,
                8.0,
                120.0,
                34.0,
            )]),
        );

        assert!(changed_pixel_count(&bytes, 152, 106, 18, 16, 12) > 0);
    }

    #[test]
    fn add_component_button_uses_audited_offset_and_content_tones() {
        let mut node = positioned_button_node(
            "WorkbenchAddComponent",
            "Add Component",
            "outlined",
            12.0,
            8.0,
            180.0,
            34.0,
        );
        node.button_style = resolved_border([54, 64, 71, 255]);
        let style = button_style(&node, button_kind(&node));
        let rect = button_paint_rect(
            &node,
            &FrameRect {
                x: node.frame.x,
                y: node.frame.y,
                width: node.frame.width,
                height: node.frame.height,
            },
        );

        assert_eq!(rect.y, 9.5);
        assert_eq!(style.border, [54, 64, 71, 255]);
        assert_eq!(style.text, ADD_COMPONENT_TEXT);
        assert_eq!(style.glyph, ADD_COMPONENT_GLYPH);
        assert_eq!(button_glyph(&node), ButtonGlyph::Plus);
    }

    #[test]
    fn workbench_secondary_button_uses_declared_surface_color() {
        let mut node = positioned_button_node(
            "WorkbenchSecondaryButton",
            "Secondary",
            "outlined",
            12.0,
            8.0,
            82.0,
            32.0,
        );
        node.button_style = resolved_background([26, 31, 35, 255]);

        let style = button_style(&node, button_kind(&node));

        assert_eq!(style.surface, [26, 31, 35, 255]);
        assert_eq!(style.border, OUTLINED_BORDER);
        assert_eq!(style.text, OUTLINED_TEXT);
    }

    #[test]
    fn workbench_primary_row_uses_declared_metrics_and_brightness() {
        let mut primary = positioned_button_node(
            "WorkbenchPrimaryButton",
            "Primary",
            "filled",
            12.0,
            8.0,
            80.0,
            32.0,
        );
        primary.layout_offset_x = 3.0;
        primary.layout_offset_y = -1.0;
        primary.button_style =
            resolved_background_and_border([41, 164, 184, 255], [28, 135, 152, 255]);

        let primary_rect = button_paint_rect(
            &primary,
            &FrameRect {
                x: primary.frame.x,
                y: primary.frame.y,
                width: primary.frame.width,
                height: primary.frame.height,
            },
        );
        let primary_style = button_style(&primary, button_kind(&primary));

        assert_eq!(primary_rect.x, 15.0);
        assert_eq!(primary_rect.y, 7.0);
        assert_eq!(primary_style.surface, [41, 164, 184, 255]);
        assert_eq!(primary_style.border, [28, 135, 152, 255]);

        let mut secondary = positioned_button_node(
            "WorkbenchSecondaryButton",
            "Secondary",
            "outlined",
            12.0,
            8.0,
            82.0,
            32.0,
        );
        secondary.layout_offset_x = 1.0;
        secondary.layout_offset_y = -1.0;
        secondary.label_brightness = 1.01;
        secondary.button_style = resolved_background([26, 31, 35, 255]);

        let secondary_rect = button_paint_rect(
            &secondary,
            &FrameRect {
                x: secondary.frame.x,
                y: secondary.frame.y,
                width: secondary.frame.width,
                height: secondary.frame.height,
            },
        );
        let secondary_style = button_style(&secondary, button_kind(&secondary));

        assert_eq!(secondary_rect.x, 13.0);
        assert_eq!(secondary_rect.y, 7.0);
        assert_eq!(secondary_style.surface, [26, 31, 35, 255]);
        assert_eq!(secondary_style.border, [59, 71, 79, 255]);
    }

    #[test]
    fn workbench_variant_row_uses_declared_surface_and_border() {
        let mut tertiary = positioned_button_node(
            "WorkbenchTertiaryButton",
            "Tertiary",
            "text",
            12.0,
            8.0,
            80.0,
            32.0,
        );
        tertiary.layout_offset_x = 1.0;
        tertiary.corner_radius = 9.0;
        tertiary.button_style = resolved_button_style(
            [23, 28, 32, 255],
            [37, 46, 53, 255],
            [135, 146, 153, 255],
            1.0,
        );
        let tertiary_style = button_style(&tertiary, button_kind(&tertiary));

        assert_eq!(tertiary_style.surface, [23, 28, 32, 255]);
        assert_eq!(tertiary_style.border, [37, 46, 53, 255]);
        assert_eq!(tertiary_style.text, [135, 146, 153, 255]);
        assert_eq!(button_radius(&tertiary, &tertiary.frame_rect()), 9.0);

        let mut outline = positioned_button_node(
            "WorkbenchOutlineButton",
            "Outline",
            "outlined",
            12.0,
            8.0,
            82.0,
            32.0,
        );
        outline.layout_offset_x = 1.0;
        outline.corner_radius = 9.0;
        outline.button_style =
            resolved_button_style([0, 0, 0, 0], [37, 46, 53, 255], [135, 146, 153, 255], 1.0);
        let outline_style = button_style(&outline, button_kind(&outline));

        assert_eq!(outline_style.surface, OUTLINED_SURFACE);
        assert_eq!(outline_style.border, [37, 46, 53, 255]);
        assert_eq!(outline_style.text, [135, 146, 153, 255]);
        assert_eq!(button_radius(&outline, &outline.frame_rect()), 9.0);
    }

    #[test]
    fn workbench_icon_button_uses_declared_surface_and_border() {
        let mut node = positioned_button_node(
            "WorkbenchButtonIcon",
            "Icon",
            "outlined",
            12.0,
            8.0,
            80.0,
            32.0,
        );
        node.button_style = resolved_background_and_border([32, 38, 42, 255], [48, 56, 64, 255]);

        let style = button_style(&node, button_kind(&node));

        assert_eq!(style.surface, [32, 38, 42, 255]);
        assert_eq!(style.border, [48, 56, 64, 255]);
        assert_eq!(style.text, OUTLINED_TEXT);
        assert_eq!(style.glyph, OUTLINED_TEXT);
    }

    #[test]
    fn workbench_icon_delete_row_uses_declared_content_tones_and_radius() {
        let mut icon = positioned_button_node(
            "WorkbenchButtonIcon",
            "Icon",
            "outlined",
            12.0,
            8.0,
            80.0,
            32.0,
        );
        icon.corner_radius = 9.0;
        icon.label_brightness = 1.02;
        icon.button_style = resolved_foreground([127, 138, 145, 255]);
        let icon_style = button_style(&icon, button_kind(&icon));
        assert_eq!(button_radius(&icon, &icon.frame_rect()), 9.0);
        assert_eq!(icon_style.text, [130, 141, 148, 255]);
        assert_eq!(icon_style.glyph, [130, 141, 148, 255]);

        let mut delete = positioned_button_node(
            "WorkbenchButtonDelete",
            "",
            "outlined",
            12.0,
            8.0,
            82.0,
            32.0,
        );
        delete.validation_level = "danger".into();
        delete.corner_radius = 9.0;
        delete.label_brightness = 1.02;
        delete.button_style = resolved_foreground([208, 90, 80, 255]);
        let delete_style = button_style(&delete, button_kind(&delete));
        assert_eq!(button_radius(&delete, &delete.frame_rect()), 9.0);
        assert_eq!(delete_style.text, [212, 92, 82, 255]);
        assert_eq!(delete_style.glyph, [212, 92, 82, 255]);
    }

    #[test]
    fn workbench_button_applies_declared_visual_brightness() {
        let mut node = positioned_button_node(
            "WorkbenchButtonIcon",
            "Icon",
            "outlined",
            12.0,
            8.0,
            120.0,
            34.0,
        );
        node.label_brightness = 0.96;

        let style = button_style(&node, WorkbenchButtonKind::Secondary);

        assert_eq!(style.surface, [24, 30, 34, 255]);
        assert_eq!(style.border, [56, 67, 75, 255]);
        assert_eq!(style.text, [193, 204, 209, 255]);
        assert_eq!(style.glyph, [193, 204, 209, 255]);
    }

    #[test]
    fn workbench_button_style_selector_applies_state_priority_before_painting() {
        let mut node = positioned_button_node(
            "WorkbenchPrimaryButton",
            "Primary",
            "filled",
            12.0,
            8.0,
            120.0,
            34.0,
        );
        node.hovered = true;
        node.focused = true;
        let focused = button_style(&node, button_kind(&node));
        assert_eq!(
            focused.interaction,
            zircon_runtime_interface::ui::style::ButtonInteractionState::Focused
        );

        node.pressed = true;
        let pressed = button_style(&node, button_kind(&node));
        assert_eq!(
            pressed.interaction,
            zircon_runtime_interface::ui::style::ButtonInteractionState::Pressed
        );

        node.disabled = true;
        let disabled = button_style(&node, button_kind(&node));
        assert_eq!(
            disabled.interaction,
            zircon_runtime_interface::ui::style::ButtonInteractionState::Disabled
        );
    }

    #[test]
    fn workbench_button_honors_declared_layout_offset() {
        let mut node = positioned_button_node(
            "WorkbenchPrimaryButton",
            "Primary",
            "filled",
            12.4,
            8.4,
            80.0,
            32.0,
        );
        node.layout_offset_x = 3.0;
        node.layout_offset_y = -1.0;

        let rect = button_paint_rect(
            &node,
            &FrameRect {
                x: node.frame.x,
                y: node.frame.y,
                width: node.frame.width,
                height: node.frame.height,
            },
        );

        assert_eq!(rect.x, 15.0);
        assert_eq!(rect.y, 7.0);
        assert_eq!(rect.width, 80.0);
        assert_eq!(rect.height, 32.0);
    }

    fn button_node(control_id: &str, text: &str, variant: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Button".into(),
            component_role: "button".into(),
            text: text.into(),
            button_variant: variant.into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 120.0,
                height: 34.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn positioned_button_node(
        control_id: &str,
        text: &str,
        variant: &str,
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
            ..button_node(control_id, text, variant)
        }
    }

    trait TemplatePaneNodeDataTestExt {
        fn frame_rect(&self) -> FrameRect;
    }

    impl TemplatePaneNodeDataTestExt for TemplatePaneNodeData {
        fn frame_rect(&self) -> FrameRect {
            FrameRect {
                x: self.frame.x,
                y: self.frame.y,
                width: self.frame.width,
                height: self.frame.height,
            }
        }
    }

    fn resolved_background(color: [u8; 4]) -> ResolvedButtonStyle {
        ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    color[0], color[1], color[2], color[3],
                ))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
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

    fn resolved_button_style(
        background: [u8; 4],
        border: [u8; 4],
        foreground: [u8; 4],
        opacity: f32,
    ) -> ResolvedButtonStyle {
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
                foreground_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    foreground[0],
                    foreground[1],
                    foreground[2],
                    foreground[3],
                ))),
                opacity,
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        }
    }

    fn resolved_foreground(color: [u8; 4]) -> ResolvedButtonStyle {
        ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                foreground_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    color[0], color[1], color[2], color[3],
                ))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        }
    }

    fn resolved_border(color: [u8; 4]) -> ResolvedButtonStyle {
        ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(
                    color[0], color[1], color[2], color[3],
                ))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
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
