use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
#[cfg(test)]
use super::style_selector::{
    is_workbench_slider_state_hot, WORKBENCH_SLIDER_TEXT as SLIDER_TEXT,
    WORKBENCH_SLIDER_THUMB as SLIDER_THUMB, WORKBENCH_SLIDER_TRACK as SLIDER_TRACK,
    WORKBENCH_SLIDER_TRACK_DISABLED as SLIDER_TRACK_DISABLED,
};
use super::style_selector::{select_workbench_slider_style, WorkbenchSliderStyle};
#[cfg(test)]
use super::theme::PALETTE;
#[cfg(test)]
use crate::ui::retained_host::primitives::Color;
#[cfg(test)]
use zircon_runtime_interface::ui::style::UiPainterResolvedState;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SLIDER_TRACK_HEIGHT: f32 = 3.0;
const SLIDER_TRACK_RADIUS: f32 = 2.0;
const SLIDER_THUMB_SIZE: f32 = 11.0;
const SLIDER_THUMB_HALO_SIZE: f32 = 20.0;
const SLIDER_HORIZONTAL_INSET: f32 = 8.0;
const SLIDER_LABEL_WIDTH: f32 = 50.0;
const SLIDER_LABEL_GAP: f32 = 12.0;
const SLIDER_VALUE_WIDTH: f32 = 44.0;
const SLIDER_VALUE_GAP: f32 = 10.0;
const SLIDER_FONT_SIZE: f32 = 11.0;
const SLIDER_LINE_HEIGHT: f32 = SLIDER_FONT_SIZE * 1.2;

pub(super) fn push_slider_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_slider(node) {
        return false;
    }
    let rect = pixel_aligned_rect(rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }

    let label = slider_label(node);
    let value_rect = slider_value_rect(&rect);
    let track_rect = slider_track_rect(&rect, value_rect.as_ref(), label.is_some(), node);
    if track_rect.width <= 1.0 {
        return true;
    }

    let percent = slider_percent(node);
    let range_min_percent = slider_range_min_percent(node);
    let style = slider_style(node);
    if let Some(label) = label {
        push_slider_label(commands, &rect, clip, order + 3, label, &style, opacity);
    }
    push_slider_track(
        commands,
        &style,
        &track_rect,
        clip,
        order,
        percent,
        range_min_percent,
        opacity,
    );
    if let Some(tick_count) = slider_tick_count(node) {
        push_slider_ticks(
            commands,
            &track_rect,
            clip,
            order + 2,
            tick_count,
            &style,
            opacity,
        );
    }
    if let Some(range_min_percent) = range_min_percent {
        push_slider_thumb(
            commands,
            node,
            &style,
            &track_rect,
            clip,
            order + 3,
            range_min_percent,
            opacity,
        );
    }
    push_slider_thumb(
        commands,
        node,
        &style,
        &track_rect,
        clip,
        order + 4,
        percent,
        opacity,
    );
    if let Some(range_min_percent) = range_min_percent {
        push_slider_range_min_value(
            commands,
            node,
            &style,
            &rect,
            &track_rect,
            clip,
            order + 5,
            range_min_percent,
            opacity,
        );
    }
    if let Some(value_rect) = value_rect {
        push_slider_value(
            commands,
            node,
            &style,
            &value_rect,
            clip,
            order + 5,
            percent,
            opacity,
        );
    }
    true
}

fn is_workbench_slider(node: &TemplatePaneNodeData) -> bool {
    uses_workbench_visual_language(node)
        && is_component_family(node, TemplateComponentFamily::Slider)
}

fn push_slider_track(
    commands: &mut Vec<HostPaintCommand>,
    style: &WorkbenchSliderStyle,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    range_min_percent: Option<f32>,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        track_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.track),
        None,
        0.0,
        SLIDER_TRACK_RADIUS,
        opacity,
    ));

    let (fill_start, fill_end) = slider_fill_span(percent, range_min_percent);
    let fill_width = (track_rect.width * (fill_end - fill_start)).max(0.0);
    if fill_width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: track_rect.x + track_rect.width * fill_start,
            y: track_rect.y,
            width: fill_width.max(1.0),
            height: track_rect.height,
        },
        Some(clip.clone()),
        order + 1,
        Some(style.fill),
        None,
        0.0,
        SLIDER_TRACK_RADIUS,
        opacity,
    ));
}

fn push_slider_ticks(
    commands: &mut Vec<HostPaintCommand>,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    tick_count: usize,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    if tick_count < 2 {
        return;
    }
    let last = tick_count - 1;
    for index in 0..tick_count {
        let fraction = index as f32 / last as f32;
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: track_rect.x + track_rect.width * fraction - 0.5,
                y: track_rect.y + 8.0,
                width: 1.0,
                height: 4.0,
            },
            Some(clip.clone()),
            order,
            Some(style.tick),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}

fn push_slider_thumb(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    style: &WorkbenchSliderStyle,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    opacity: f32,
) {
    let center_x = track_rect.x + track_rect.width * percent;
    let center_y = track_rect.y + track_rect.height * 0.5;
    let thumb_size = slider_thumb_size(node);
    if let Some(halo_color) = style.thumb_halo {
        commands.push(HostPaintCommand::quad(
            centered_rect(center_x, center_y, SLIDER_THUMB_HALO_SIZE),
            Some(clip.clone()),
            order,
            Some(halo_color),
            None,
            0.0,
            SLIDER_THUMB_HALO_SIZE * 0.5,
            opacity,
        ));
    }
    commands.push(HostPaintCommand::quad(
        centered_rect(center_x, center_y, thumb_size),
        Some(clip.clone()),
        order + 1,
        Some(style.thumb),
        Some(style.thumb_outline),
        1.0,
        thumb_size * 0.5,
        opacity,
    ));
}

fn push_slider_value(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    style: &WorkbenchSliderStyle,
    value_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    percent: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        value_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.value_surface),
        Some(style.value_border),
        1.0,
        4.0,
        opacity,
    ));
    let label = slider_value_label(node, percent);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: value_rect.x + 6.0,
            y: value_rect.y + (value_rect.height - SLIDER_LINE_HEIGHT).max(0.0) * 0.5,
            width: (value_rect.width - 12.0).max(1.0),
            height: SLIDER_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 1,
        label,
        style.value_text,
        SLIDER_FONT_SIZE,
        SLIDER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_slider_range_min_value(
    commands: &mut Vec<HostPaintCommand>,
    _node: &TemplatePaneNodeData,
    style: &WorkbenchSliderStyle,
    rect: &FrameRect,
    track_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    range_min_percent: f32,
    opacity: f32,
) {
    let Some(value_rect) = slider_range_min_value_rect(rect, track_rect) else {
        return;
    };
    commands.push(HostPaintCommand::quad(
        value_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.value_surface),
        Some(style.range_value_border),
        1.0,
        4.0,
        opacity,
    ));
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: value_rect.x + 6.0,
            y: value_rect.y + (value_rect.height - SLIDER_LINE_HEIGHT).max(0.0) * 0.5,
            width: (value_rect.width - 12.0).max(1.0),
            height: SLIDER_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order + 1,
        slider_range_min_label(range_min_percent),
        style.value_text,
        SLIDER_FONT_SIZE,
        SLIDER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_slider_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: String,
    style: &WorkbenchSliderStyle,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + SLIDER_HORIZONTAL_INSET,
            y: rect.y + (rect.height - SLIDER_LINE_HEIGHT).max(0.0) * 0.5,
            width: SLIDER_LABEL_WIDTH,
            height: SLIDER_LINE_HEIGHT,
        },
        Some(clip.clone()),
        order,
        label,
        style.label_text,
        SLIDER_FONT_SIZE,
        SLIDER_LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn slider_track_rect(
    rect: &FrameRect,
    value_rect: Option<&FrameRect>,
    has_label: bool,
    node: &TemplatePaneNodeData,
) -> FrameRect {
    let label_lane_width = if has_label {
        SLIDER_LABEL_WIDTH + SLIDER_LABEL_GAP
    } else {
        0.0
    };
    let left = rect.x + label_lane_width + SLIDER_HORIZONTAL_INSET + slider_track_offset_x(node);
    let right = (value_rect
        .map(|value| value.x - SLIDER_VALUE_GAP)
        .unwrap_or(rect.x + rect.width - SLIDER_HORIZONTAL_INSET)
        + slider_track_width_delta(node))
    .max(left);
    FrameRect {
        x: left,
        y: rect.y + (rect.height - SLIDER_TRACK_HEIGHT).max(0.0) * 0.5,
        width: right - left,
        height: SLIDER_TRACK_HEIGHT,
    }
}

fn slider_value_rect(rect: &FrameRect) -> Option<FrameRect> {
    if rect.width < 132.0 {
        return None;
    }
    let height = (rect.height - 6.0).clamp(18.0, 24.0);
    Some(FrameRect {
        x: rect.x + rect.width - SLIDER_HORIZONTAL_INSET - SLIDER_VALUE_WIDTH,
        y: rect.y + (rect.height - height).max(0.0) * 0.5,
        width: SLIDER_VALUE_WIDTH,
        height,
    })
}

fn slider_range_min_value_rect(rect: &FrameRect, track_rect: &FrameRect) -> Option<FrameRect> {
    if rect.height < 42.0 || track_rect.width < SLIDER_VALUE_WIDTH {
        return None;
    }
    Some(FrameRect {
        x: track_rect.x,
        y: track_rect.y + 10.0,
        width: SLIDER_VALUE_WIDTH,
        height: 20.0,
    })
}

fn slider_percent(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent.is_finite() {
        node.value_percent.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn slider_label(node: &TemplatePaneNodeData) -> Option<String> {
    let label = node.label_text.trim();
    (!label.is_empty()).then(|| label.to_owned())
}

fn slider_value_label(node: &TemplatePaneNodeData, percent: f32) -> String {
    let value = node.value_text.trim();
    if value.is_empty() {
        format!("{:.2}", percent.clamp(0.0, 1.0))
    } else {
        value.to_owned()
    }
}

fn slider_range_min_label(percent: f32) -> String {
    format!("{:.2}", percent.clamp(0.0, 1.0))
}

fn slider_range_min_percent(node: &TemplatePaneNodeData) -> Option<f32> {
    let is_range_row = node.control_id.as_str().contains("RangeSlider");
    if !is_range_row && node.layout_second_cell_offset_x <= 0.0 {
        return None;
    }
    Some(slider_declared_percent(node.layout_second_cell_offset_x))
}

fn slider_tick_count(node: &TemplatePaneNodeData) -> Option<usize> {
    let declared = node.layout_third_cell_offset_x.round() as usize;
    if declared >= 2 {
        Some(declared)
    } else if node.control_id.as_str().contains("StepsSlider") {
        Some(5)
    } else {
        None
    }
}

fn slider_declared_percent(value: f32) -> f32 {
    if value > 1.0 {
        (value / 100.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn slider_fill_span(percent: f32, range_min_percent: Option<f32>) -> (f32, f32) {
    let end = percent.clamp(0.0, 1.0);
    let start = range_min_percent.unwrap_or(0.0).clamp(0.0, 1.0);
    if start <= end {
        (start, end)
    } else {
        (end, start)
    }
}

fn slider_style(node: &TemplatePaneNodeData) -> WorkbenchSliderStyle {
    select_workbench_slider_style(node)
}

#[cfg(test)]
fn slider_label_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).label_text
}

#[cfg(test)]
fn slider_accent(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).fill
}

#[cfg(test)]
fn slider_track_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).track
}

fn slider_thumb_size(node: &TemplatePaneNodeData) -> f32 {
    if node.layout_icon_size > 0.0 {
        node.layout_icon_size
    } else {
        SLIDER_THUMB_SIZE
    }
}

#[cfg(test)]
fn slider_thumb_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).thumb
}

#[cfg(test)]
fn slider_thumb_outline_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    slider_style(node).thumb_outline
}

#[cfg(test)]
fn slider_thumb_halo_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    slider_style(node).thumb_halo
}

#[cfg(test)]
fn slider_visual_state(node: &TemplatePaneNodeData) -> UiPainterResolvedState {
    slider_style(node).state
}

#[cfg(test)]
fn slider_visual_hot(node: &TemplatePaneNodeData) -> bool {
    is_workbench_slider_state_hot(slider_visual_state(node))
}

fn slider_track_offset_x(node: &TemplatePaneNodeData) -> f32 {
    node.layout_content_offset_x
}

fn slider_track_width_delta(node: &TemplatePaneNodeData) -> f32 {
    node.layout_first_cell_offset_x
}

fn centered_rect(center_x: f32, center_y: f32, size: f32) -> FrameRect {
    FrameRect {
        x: center_x - size * 0.5,
        y: center_y - size * 0.5,
        width: size,
        height: size,
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
    fn workbench_slider_recognizes_slider_ids_and_roles() {
        assert!(is_workbench_slider(&slider_node(
            "WorkbenchInputSlider",
            0.75
        )));
        assert!(is_workbench_slider(&slider_node(
            "WorkbenchInputRangeSlider",
            0.8
        )));
        assert!(is_workbench_slider(&slider_node(
            "WorkbenchInputStepsSlider",
            0.86
        )));
        assert!(is_workbench_slider(&slider_node(
            "WorkbenchSliderRoot",
            0.5
        )));
        assert!(!is_workbench_slider(&slider_node("PlainSlider", 0.5)));
    }

    #[test]
    fn declared_workbench_slider_variant_is_recognized_without_workbench_id() {
        let mut node = slider_node("PlainSlider", 0.5);
        node.component_variant = "workbench-slider".into();

        assert!(is_workbench_slider(&node));
    }

    #[test]
    fn workbench_slider_paints_track_fill_thumb_and_value() {
        let bytes = paint_template_nodes_for_test(
            220,
            48,
            model_rc(vec![positioned_slider_node(
                "WorkbenchInputSlider",
                0.75,
                8.0,
                8.0,
                184.0,
                30.0,
            )]),
        );

        assert_eq!(pixel_at(&bytes, 220, 24, 23), PALETTE.accent);
        assert_eq!(pixel_at(&bytes, 220, 126, 23), SLIDER_TRACK);
        assert_eq!(pixel_at(&bytes, 220, 104, 23), SLIDER_THUMB);
        assert_ne!(pixel_at(&bytes, 220, 152, 23), [0, 0, 0, 255]);
        assert!(changed_pixel_count(&bytes, 220, 157, 16, 28, 16) > 0);
    }

    #[test]
    fn hovered_workbench_slider_paints_thumb_halo() {
        let mut node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 160.0, 30.0);
        node.hovered = true;
        let bytes = paint_template_nodes_for_test(190, 48, model_rc(vec![node]));

        assert_ne!(pixel_at(&bytes, 190, 61, 15), [0, 0, 0, 255]);
    }

    #[test]
    fn workbench_slider_uses_shared_selector_for_drop_hover_halo() {
        let mut node = positioned_slider_node("WorkbenchInputSlider", 0.5, 8.0, 8.0, 160.0, 30.0);
        node.drop_hovered = true;

        assert_eq!(
            slider_visual_state(&node),
            UiPainterResolvedState::DropHovered
        );
        assert!(slider_visual_hot(&node));

        let bytes = paint_template_nodes_for_test(190, 48, model_rc(vec![node]));
        assert_ne!(pixel_at(&bytes, 190, 61, 15), [0, 0, 0, 255]);
    }

    #[test]
    fn workbench_slider_uses_declared_fill_and_thumb_size() {
        let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
        node.value_color = crate::ui::retained_host::primitives::Color::from_rgb_u8(32, 153, 162);
        node.layout_icon_size = 9.0;
        node.button_style = resolved_background([42, 51, 56, 255]);

        assert_eq!(slider_track_color(&node), [42, 51, 56, 255]);
        assert_eq!(slider_accent(&node), [32, 153, 162, 255]);
        assert_eq!(slider_thumb_size(&node), 9.0);

        node.disabled = true;
        assert_eq!(slider_track_color(&node), SLIDER_TRACK_DISABLED);
    }

    #[test]
    fn workbench_slider_uses_declared_thumb_colors() {
        let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
        node.icon_color = Color::from_rgb_u8(183, 241, 248);
        node.state_layer_color = Color::from_argb_u8(61, 50, 211, 222);
        node.button_style = ResolvedButtonStyle {
            element: UiResolvedElementStyle {
                border_color: Some(UiStyleColor::Rgba(UiRgbaColor::from_u8(42, 177, 188, 51))),
                ..UiResolvedElementStyle::default()
            },
            ..ResolvedButtonStyle::default()
        };

        assert_eq!(slider_thumb_color(&node), [183, 241, 248, 255]);
        assert_eq!(slider_thumb_outline_color(&node), [42, 177, 188, 51]);
        assert_eq!(slider_thumb_halo_color(&node), Some([50, 211, 222, 61]));

        let bytes = paint_template_nodes_for_test(220, 48, model_rc(vec![node]));

        assert_eq!(pixel_at(&bytes, 220, 104, 23), [183, 241, 248, 255]);
        assert_ne!(pixel_at(&bytes, 220, 96, 23), [0, 0, 0, 255]);
    }

    #[test]
    fn workbench_slider_uses_declared_label_value_and_track_proportion() {
        let mut node = positioned_slider_node("WorkbenchInputSlider", 0.75, 8.0, 8.0, 184.0, 30.0);
        node.label_text = "Value".into();
        node.value_text = "0.75".into();
        node.label_color = Color::from_rgb_u8(136, 147, 153);
        node.layout_content_offset_x = -10.0;
        node.layout_first_cell_offset_x = 18.0;

        let rect = FrameRect {
            x: 8.0,
            y: 8.0,
            width: 184.0,
            height: 30.0,
        };
        let value_rect = slider_value_rect(&rect);
        let track_rect = slider_track_rect(
            &rect,
            value_rect.as_ref(),
            slider_label(&node).is_some(),
            &node,
        );

        assert_eq!(slider_label(&node).as_deref(), Some("Value"));
        assert_eq!(slider_value_label(&node, 0.75), "0.75");
        assert_eq!(slider_label_color(&node), [136, 147, 153, 255]);
        assert_eq!(track_rect.x, 68.0);
        assert_eq!(track_rect.width, 80.0);
    }

    #[test]
    fn workbench_range_slider_uses_declared_minimum_span() {
        let mut node =
            positioned_slider_node("WorkbenchInputRangeSlider", 0.8, 8.0, 8.0, 184.0, 46.0);
        node.layout_second_cell_offset_x = 20.0;

        let range_min = slider_range_min_percent(&node).expect("range minimum");
        let (fill_start, fill_end) = slider_fill_span(slider_percent(&node), Some(range_min));

        assert!((range_min - 0.2).abs() < 0.001);
        assert!((fill_start - 0.2).abs() < 0.001);
        assert!((fill_end - 0.8).abs() < 0.001);
    }

    #[test]
    fn workbench_steps_slider_uses_declared_tick_count() {
        let mut node =
            positioned_slider_node("WorkbenchInputStepsSlider", 0.86, 8.0, 8.0, 184.0, 30.0);
        node.layout_third_cell_offset_x = 5.0;

        assert_eq!(slider_tick_count(&node), Some(5));
        assert_eq!(slider_range_min_percent(&node), None);
    }

    fn slider_node(control_id: &str, percent: f32) -> TemplatePaneNodeData {
        TemplatePaneNodeData {
            control_id: control_id.into(),
            role: "RangeField".into(),
            component_role: "range-field".into(),
            value_percent: percent,
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 160.0,
                height: 30.0,
            },
            ..TemplatePaneNodeData::default()
        }
    }

    fn positioned_slider_node(
        control_id: &str,
        percent: f32,
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
            ..slider_node(control_id, percent)
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
