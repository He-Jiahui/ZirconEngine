use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::template_node_labels::template_node_label;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SECTION_FONT_SIZE: f32 = 13.0;
const SECTION_LINE_HEIGHT: f32 = SECTION_FONT_SIZE * 1.2;
const SECTION_TEXT_LEFT: f32 = 8.0;
const SECTION_ICON_SIZE: f32 = 14.0;
const SECTION_ICON_GAP: f32 = 8.0;
const SECTION_TEXT: [u8; 4] = [225, 236, 240, 255];
const SECTION_TEXT_MUTED: [u8; 4] = [186, 201, 207, 255];
const SECTION_MESH_TEXT: [u8; 4] = [176, 186, 191, 255];
const SECTION_GLYPH: [u8; 4] = [155, 173, 181, 255];
const SECTION_TRANSFORM_GLYPH: [u8; 4] = [155, 173, 181, 97];

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SectionTitleIcon {
    Cube,
    Transform,
    Mesh,
}

fn is_workbench_section_title(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.control_id.as_str(),
        "WorkbenchSectionTitleRoot" | "WorkbenchTransformLabel" | "WorkbenchMeshLabel"
    ) || (node.control_id.as_str().starts_with("Workbench")
        && node.control_id.as_str().ends_with("Title"))
}

fn section_title_icon(node: &TemplatePaneNodeData) -> Option<SectionTitleIcon> {
    match node.control_id.as_str() {
        "WorkbenchInspectorTitle" => Some(SectionTitleIcon::Cube),
        "WorkbenchTransformLabel" => Some(SectionTitleIcon::Transform),
        "WorkbenchMeshLabel" => Some(SectionTitleIcon::Mesh),
        _ => None,
    }
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

fn push_section_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    icon: SectionTitleIcon,
    opacity: f32,
) {
    let color = section_icon_color(icon);
    match icon {
        SectionTitleIcon::Cube => push_cube_icon(commands, rect, clip, order, color, opacity),
        SectionTitleIcon::Transform => {
            push_transform_icon(commands, rect, clip, order, color, opacity)
        }
        SectionTitleIcon::Mesh => push_mesh_icon(commands, rect, clip, order, color, opacity),
    }
}

fn section_icon_color(icon: SectionTitleIcon) -> [u8; 4] {
    match icon {
        SectionTitleIcon::Transform => SECTION_TRANSFORM_GLYPH,
        SectionTitleIcon::Cube | SectionTitleIcon::Mesh => SECTION_GLYPH,
    }
}

fn push_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (5.0, 1.0, 4.0, 2.0),
            (3.0, 3.0, 2.0, 7.0),
            (9.0, 3.0, 2.0, 7.0),
            (5.0, 11.0, 4.0, 2.0),
            (1.0, 5.0, 2.0, 4.0),
            (11.0, 5.0, 2.0, 4.0),
        ],
    );
}

fn push_transform_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (6.0, 1.0, 2.0, 12.0),
            (1.0, 6.0, 12.0, 2.0),
            (3.0, 3.0, 2.0, 2.0),
            (9.0, 9.0, 2.0, 2.0),
        ],
    );
}

fn push_mesh_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (2.0, 2.0, 10.0, 2.0),
            (2.0, 6.0, 10.0, 2.0),
            (2.0, 10.0, 10.0, 2.0),
            (2.0, 2.0, 2.0, 10.0),
            (6.0, 2.0, 2.0, 10.0),
            (10.0, 2.0, 2.0, 10.0),
        ],
    );
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
    fn workbench_section_title_matches_titles_without_row_labels() {
        assert!(is_workbench_section_title(&title_node(
            "WorkbenchButtonsTitle",
            "Buttons"
        )));
        assert!(is_workbench_section_title(&title_node(
            "WorkbenchTransformLabel",
            "Transform"
        )));
        assert!(!is_workbench_section_title(&title_node(
            "WorkbenchTransformPositionLabel",
            "Position"
        )));
    }

    #[test]
    fn component_drawer_section_title_paints_bold_label() {
        let bytes = paint_template_nodes_for_test(
            180,
            48,
            model_rc(vec![title_node("WorkbenchButtonsTitle", "Buttons")]),
        );

        assert!(changed_pixel_count(&bytes, 180, 18, 14, 72, 20) > 0);
        assert_eq!(pixel_at(&bytes, 180, 12, 8), [0, 0, 0, 255]);
    }

    #[test]
    fn inspector_section_title_paints_leading_icon_and_label() {
        let bytes = paint_template_nodes_for_test(
            180,
            48,
            model_rc(vec![title_node("WorkbenchInspectorTitle", "Props")]),
        );

        assert!(changed_pixel_count(&bytes, 180, 18, 17, 18, 18) > 0);
        assert!(changed_pixel_count(&bytes, 180, 43, 14, 58, 20) > 0);
    }

    #[test]
    fn mesh_renderer_section_title_uses_audited_title_tone() {
        assert_eq!(
            section_text_color(&title_node("WorkbenchMeshLabel", "Mesh Renderer")),
            SECTION_MESH_TEXT
        );
    }

    #[test]
    fn section_title_uses_declared_title_tone() {
        let mut node = title_node("WorkbenchSelectionTitle", "Checkboxes & Radios");
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(152, 163, 168);

        assert_eq!(section_text_color(&node), [152, 163, 168, 255]);
    }

    #[test]
    fn transform_section_title_uses_audited_icon_opacity() {
        assert_eq!(
            section_icon_color(SectionTitleIcon::Transform),
            SECTION_TRANSFORM_GLYPH
        );
        assert_eq!(SECTION_TRANSFORM_GLYPH[3], 97);
    }

    fn title_node(control_id: &str, text: &str) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "Label".into(),
            text: text.into(),
            font_weight: 700,
            frame: TemplateNodeFrameData {
                x: 10.0,
                y: 8.0,
                width: 150.0,
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
