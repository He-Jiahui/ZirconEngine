use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{is_component_family, TemplateComponentFamily};
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    select_workbench_segmented_control_style, WorkbenchSegmentedControlKind as SegmentedStyleKind,
    WorkbenchSegmentedControlStyle, WORKBENCH_SEGMENT_IDLE_BACKGROUND,
};
use super::template_node_labels::template_node_label;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SEGMENT_FONT_SIZE: f32 = 11.0;
const SEGMENT_TEXT_INSET_X: f32 = 8.0;
const SEGMENT_TEXT_INSET_Y: f32 = 5.0;
const SEGMENT_RADIUS: f32 = 5.0;
const SEGMENT_SELECTED_INSET: f32 = 2.0;
const SEGMENT_IDLE_BACKGROUND: [u8; 4] = WORKBENCH_SEGMENT_IDLE_BACKGROUND;
const SEGMENT_GROUP_LABEL_FONT_SIZE: f32 = 11.0;
const SEGMENT_GROUP_LABEL_HEIGHT: f32 = 14.0;
const SEGMENT_GROUP_LABEL_GAP: f32 = 4.0;
const TAB_FONT_SIZE: f32 = 12.0;
const TAB_TEXT_INSET_X: f32 = 12.0;
const TAB_UNDERLINE_HEIGHT: f32 = 2.0;

pub(super) fn push_segmented_control_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_segmented_control(node) {
        let options = segmented_options(node);
        if options.is_empty() {
            return false;
        }
        push_segmented_control(commands, node, rect, clip, order, opacity, &options);
        return true;
    }

    if is_workbench_tab(node) {
        push_workbench_tab(commands, node, rect, clip, order, opacity);
        return true;
    }

    false
}

fn is_segmented_control(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::SegmentedControl)
}

fn is_workbench_tab(node: &TemplatePaneNodeData) -> bool {
    is_component_family(node, TemplateComponentFamily::Tab)
}

fn push_segmented_control(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    options: &[String],
) {
    push_segmented_group_label(commands, node, rect, clip, order + 3, opacity);
    let body_rect = segmented_body_rect(node, rect);
    let style = segmented_control_style(node);
    commands.push(HostPaintCommand::quad(
        body_rect.clone(),
        Some(clip.clone()),
        order,
        style.background,
        style.border,
        style.border_width,
        SEGMENT_RADIUS,
        opacity,
    ));

    let selected = selected_segment_value(node);
    for (index, option) in options.iter().enumerate() {
        let segment = segment_rect(&body_rect, index, options.len());
        if index > 0 {
            push_segment_divider(commands, &segment, clip, order + 1, opacity);
        }
        if option_is_selected(option, selected.as_deref()) {
            push_selected_segment(commands, node, &segment, clip, order + 2, opacity);
        }
        push_segment_label(
            commands,
            node,
            option,
            &segment,
            clip,
            order + 4,
            option_is_selected(option, selected.as_deref()),
            opacity,
        );
    }
}

fn push_segmented_group_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let label = node.label_text.trim();
    if label.is_empty() {
        return;
    }

    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: SEGMENT_GROUP_LABEL_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label.to_string(),
        segmented_group_label_color(node),
        SEGMENT_GROUP_LABEL_FONT_SIZE,
        SEGMENT_GROUP_LABEL_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn segmented_body_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    let label_block_height = if node.label_text.trim().is_empty() {
        0.0
    } else {
        SEGMENT_GROUP_LABEL_HEIGHT + SEGMENT_GROUP_LABEL_GAP
    };

    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + label_block_height + node.layout_offset_y,
        width: rect.width,
        height: (rect.height - label_block_height).max(1.0),
    }
}

fn push_workbench_tab(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let rect = tab_paint_rect(node, rect);
    let style = tab_style(node);
    if let Some(background) = style.background {
        commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(background),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
    if node.checked || node.selected {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x,
                y: rect.y + (rect.height - TAB_UNDERLINE_HEIGHT).max(0.0),
                width: rect.width,
                height: TAB_UNDERLINE_HEIGHT,
            },
            Some(clip.clone()),
            order + 2,
            Some(style.selected_underline),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }

    let label = template_node_label(node, None);
    if !label.trim().is_empty() {
        let line_height = TAB_FONT_SIZE * 1.2;
        commands.push(HostPaintCommand::text(
            FrameRect {
                x: rect.x + TAB_TEXT_INSET_X,
                y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
                width: (rect.width - TAB_TEXT_INSET_X * 2.0).max(1.0),
                height: line_height,
            },
            Some(clip.clone()),
            order + 3,
            label,
            tab_text_color(node),
            TAB_FONT_SIZE,
            line_height,
            UiTextRunPaintStyle::default(),
            opacity,
        ));
    }
}

fn tab_paint_rect(node: &TemplatePaneNodeData, rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + node.layout_offset_x,
        y: rect.y + node.layout_offset_y,
        width: rect.width,
        height: rect.height,
    }
}

fn push_segment_divider(
    commands: &mut Vec<HostPaintCommand>,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: segment.x,
            y: segment.y + 4.0,
            width: 1.0,
            height: (segment.height - 8.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        Some(PALETTE.border),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_selected_segment(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let selected_rect = inset_rect(segment, SEGMENT_SELECTED_INSET);
    let style = segmented_control_style(node);
    let border_width = style.selected_border_width;
    commands.push(HostPaintCommand::quad(
        selected_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.selected_surface),
        (border_width > 0.0).then_some(style.selected_border),
        border_width,
        (SEGMENT_RADIUS - 1.0).max(0.0),
        opacity,
    ));

    let underline_height = style.selected_underline_height;
    if underline_height <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: selected_rect.x,
            y: selected_rect.y + (selected_rect.height - underline_height).max(0.0),
            width: selected_rect.width,
            height: underline_height.min(selected_rect.height).max(1.0),
        },
        Some(clip.clone()),
        order + 1,
        Some(style.selected_underline),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_segment_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    option: &str,
    segment: &FrameRect,
    clip: &FrameRect,
    order: i32,
    selected: bool,
    opacity: f32,
) {
    let label = segment_label(option);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: segment.x + SEGMENT_TEXT_INSET_X,
            y: segment.y + SEGMENT_TEXT_INSET_Y,
            width: (segment.width - SEGMENT_TEXT_INSET_X * 2.0).max(1.0),
            height: (segment.height - SEGMENT_TEXT_INSET_Y * 2.0).max(1.0),
        },
        Some(clip.clone()),
        order,
        label,
        segment_text_color(node, selected),
        SEGMENT_FONT_SIZE,
        SEGMENT_FONT_SIZE * 1.2,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn segmented_options(node: &TemplatePaneNodeData) -> Vec<String> {
    (0..node.options.row_count())
        .filter_map(|row| node.options.row_data(row))
        .map(|option| option.to_string())
        .filter(|option| !option.trim().is_empty())
        .collect()
}

fn selected_segment_value(node: &TemplatePaneNodeData) -> Option<String> {
    [
        node.value_text.as_str(),
        node.options_text.as_str(),
        node.text.as_str(),
    ]
    .into_iter()
    .find(|value| !value.trim().is_empty())
    .map(|value| value.trim().to_ascii_lowercase())
}

fn option_is_selected(option: &str, selected: Option<&str>) -> bool {
    selected.is_some_and(|value| option.trim().eq_ignore_ascii_case(value))
}

fn segment_rect(rect: &FrameRect, index: usize, count: usize) -> FrameRect {
    let count = count.max(1);
    let width = rect.width / count as f32;
    FrameRect {
        x: rect.x + width * index as f32,
        y: rect.y,
        width: if index + 1 == count {
            rect.x + rect.width - (rect.x + width * index as f32)
        } else {
            width
        }
        .max(1.0),
        height: rect.height,
    }
}

fn inset_rect(rect: &FrameRect, inset: f32) -> FrameRect {
    FrameRect {
        x: rect.x + inset,
        y: rect.y + inset,
        width: (rect.width - inset * 2.0).max(1.0),
        height: (rect.height - inset * 2.0).max(1.0),
    }
}

fn segment_label(option: &str) -> String {
    let trimmed = option.trim();
    let mut chars = trimmed.chars();
    match chars.next() {
        Some(first) => {
            let mut label = first.to_ascii_uppercase().to_string();
            label.push_str(chars.as_str());
            label
        }
        None => String::new(),
    }
}

fn segmented_background(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node)
        .background
        .unwrap_or(SEGMENT_IDLE_BACKGROUND)
}

fn segmented_border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node)
        .border
        .unwrap_or(PALETTE.border)
}

fn segment_text_color(node: &TemplatePaneNodeData, selected: bool) -> [u8; 4] {
    let style = segmented_control_style(node);
    if selected {
        style.selected_text
    } else {
        style.idle_text
    }
}

fn selected_segment_border_width(node: &TemplatePaneNodeData) -> f32 {
    segmented_control_style(node).selected_border_width
}

fn selected_segment_underline_height(node: &TemplatePaneNodeData) -> f32 {
    segmented_control_style(node).selected_underline_height
}

fn selected_segment_underline_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node).selected_underline
}

fn segmented_group_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    segmented_control_style(node).group_label
}

fn tab_background(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    tab_style(node).background
}

fn tab_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    let style = tab_style(node);
    if node.checked || node.selected {
        style.selected_text
    } else {
        style.idle_text
    }
}

fn segmented_control_style(node: &TemplatePaneNodeData) -> WorkbenchSegmentedControlStyle {
    select_workbench_segmented_control_style(node, SegmentedStyleKind::SegmentedControl)
}

fn tab_style(node: &TemplatePaneNodeData) -> WorkbenchSegmentedControlStyle {
    select_workbench_segmented_control_style(node, SegmentedStyleKind::Tab)
}

#[cfg(test)]
mod tests {
    use super::super::super::data::TemplateNodeFrameData;
    use super::super::template_nodes::paint_template_nodes_for_test;
    use super::*;
    use crate::ui::layouts::common::model_rc;
    use crate::ui::retained_host::primitives::{Color, SharedString};
    use zircon_runtime_interface::ui::style::UiPainterResolvedState;

    #[test]
    fn segmented_options_prefer_declared_option_cells() {
        let node = segmented_node();

        assert_eq!(
            segmented_options(&node),
            vec![
                "left".to_string(),
                "center".to_string(),
                "right".to_string()
            ]
        );
        assert_eq!(selected_segment_value(&node).as_deref(), Some("center"));
    }

    #[test]
    fn segment_rects_split_available_width_evenly() {
        let rect = FrameRect {
            x: 6.0,
            y: 4.0,
            width: 150.0,
            height: 30.0,
        };

        assert_eq!(segment_rect(&rect, 0, 3).x, 6.0);
        assert_eq!(segment_rect(&rect, 1, 3).x, 56.0);
        assert_eq!(segment_rect(&rect, 2, 3).width, 50.0);
    }

    #[test]
    fn selected_segment_style_defaults_to_legacy_border_without_declaration() {
        let node = segmented_node();

        assert_eq!(selected_segment_border_width(&node), 1.0);
        assert_eq!(selected_segment_underline_height(&node), 0.0);
        assert_eq!(selected_segment_underline_color(&node), PALETTE.accent);
    }

    #[test]
    fn selected_segment_style_honors_declared_border_suppression_and_underline() {
        let mut node = segmented_node();
        node.has_selected_segment_border_width = true;
        node.selected_segment_border_width = 0.0;
        node.selected_segment_underline_height = 1.0;
        node.selected_segment_underline_color = Color::from_argb_u8(122, 50, 211, 222);

        assert_eq!(selected_segment_border_width(&node), 0.0);
        assert_eq!(selected_segment_underline_height(&node), 1.0);
        assert_eq!(selected_segment_underline_color(&node), [50, 211, 222, 122]);
    }

    #[test]
    fn segmented_control_paints_selected_middle_segment() {
        let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![segmented_node()]));

        assert_eq!(
            segmented_background(&segmented_node()),
            SEGMENT_IDLE_BACKGROUND
        );
        assert_eq!(pixel_at(&bytes, 180, 17, 15), SEGMENT_IDLE_BACKGROUND);
        assert!(changed_pixel_count(&bytes, 180, 62, 8, 48, 22) > 0);
        assert!(changed_pixel_count(&bytes, 180, 14, 8, 40, 22) > 0);
    }

    #[test]
    fn segmented_control_paints_group_label_and_offsets_body() {
        let node = labeled_segmented_node();
        let body = segmented_body_rect(&node, &frame_rect(&node.frame));

        assert_eq!(body.x, 18.0);
        assert_eq!(body.y, 22.0);
        assert_eq!(body.height, 30.0);

        let bytes = paint_template_nodes_for_test(190, 60, model_rc(vec![node]));

        assert!(changed_pixel_count(&bytes, 190, 12, 4, 132, 14) > 0);
        assert!(changed_pixel_count(&bytes, 190, 18, 22, 144, 30) > 0);
        assert_eq!(pixel_at(&bytes, 190, 12, 22), [0, 0, 0, 255]);
    }

    #[test]
    fn selected_tab_paints_accent_underline_without_filling_right_edge() {
        let bytes = paint_template_nodes_for_test(180, 48, model_rc(vec![tab_node()]));

        assert!(changed_pixel_count(&bytes, 180, 0, 40, 150, 4) > 0);
        assert_eq!(pixel_at(&bytes, 180, 148, 8), [0, 0, 0, 255]);
    }

    #[test]
    fn selected_tab_honors_declared_layout_offset() {
        let mut node = tab_node();
        node.control_id = "WorkbenchLabsTabOne".into();
        node.layout_offset_x = 3.0;
        node.layout_offset_y = 2.0;
        let paint_rect = tab_paint_rect(&node, &frame_rect(&node.frame));

        assert!(is_workbench_tab(&node));
        assert_eq!(paint_rect.x, 3.0);
        assert_eq!(paint_rect.y, 6.0);

        let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 180, 0, 44), [0, 0, 0, 255]);
        assert!(changed_pixel_count(&bytes, 180, 3, 44, 150, 2) > 0);
    }

    #[test]
    fn workbench_tab_uses_declared_idle_background() {
        use zircon_runtime_interface::ui::style::{
            ResolvedButtonStyle, UiResolvedElementStyle, UiRgbaColor, UiStyleColor,
        };

        let mut node = tab_node();
        node.control_id = "WorkbenchLabsTabs".into();
        node.text = "".into();
        node.checked = false;
        node.selected = false;
        node.button_style = ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                background_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(28, 34, 38, 255))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        };

        assert!(is_workbench_tab(&node));
        assert_eq!(tab_background(&node), Some([28, 34, 38, 255]));

        let bytes = paint_template_nodes_for_test(180, 52, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 180, 8, 12), [28, 34, 38, 255]);
    }

    #[test]
    fn segmented_and_tab_styles_use_shared_state_priority() {
        let mut node = segmented_node();
        node.hovered = true;
        node.focused = true;
        node.pressed = true;
        node.disabled = true;

        let segmented = segmented_control_style(&node);
        assert_eq!(segmented.background, Some(PALETTE.surface_disabled));
        assert_eq!(segmented.border, Some(PALETTE.border_disabled));
        assert_eq!(segmented.selected_text, PALETTE.text_disabled);

        node.disabled = false;
        let segmented = segmented_control_style(&node);
        assert_eq!(segmented.state, UiPainterResolvedState::Pressed);
        assert_eq!(segmented.background, Some(PALETTE.surface_pressed));
        assert_eq!(segmented.border, Some(PALETTE.accent));

        let mut tab = tab_node();
        tab.checked = true;
        tab.hovered = true;
        let style = tab_style(&tab);
        assert_eq!(style.state, UiPainterResolvedState::Hovered);
        assert_eq!(style.background, Some(PALETTE.surface_hover));
        assert_eq!(tab_text_color(&tab), PALETTE.text);
    }

    fn segmented_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchInputSegmented".into(),
            role: "Mount".into(),
            component_role: "".into(),
            value_text: "center".into(),
            options: model_rc(vec![
                SharedString::from("left"),
                SharedString::from("center"),
                SharedString::from("right"),
            ]),
            frame: TemplateNodeFrameData {
                x: 12.0,
                y: 8.0,
                width: 150.0,
                height: 30.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn labeled_segmented_node() -> TemplatePaneNodeData {
        let mut node = segmented_node();
        node.label_text = "Segmented Control".into();
        node.label_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(161, 172, 178);
        node.label_brightness = 0.94;
        node.layout_offset_x = 6.0;
        node.frame = TemplateNodeFrameData {
            x: 12.0,
            y: 4.0,
            width: 150.0,
            height: 48.0,
        };
        node
    }

    fn tab_node() -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: "WorkbenchDrawerTabComponents".into(),
            role: "Mount".into(),
            text: "UI Components".into(),
            checked: true,
            selected: true,
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 4.0,
                width: 150.0,
                height: 40.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn frame_rect(frame: &TemplateNodeFrameData) -> FrameRect {
        FrameRect {
            x: frame.x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
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
