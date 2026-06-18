use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::template_component_family::{
    is_component_family, uses_workbench_visual_language, TemplateComponentFamily,
};
use super::render_commands::HostPaintCommand;
use super::style_selector::{select_workbench_slider_style, WorkbenchSliderStyle};
use super::template_slider_geometry::{
    centered_rect, pixel_aligned_rect, slider_fill_span, slider_label, slider_percent,
    slider_range_min_label, slider_range_min_percent, slider_range_min_value_rect,
    slider_thumb_size, slider_tick_count, slider_track_rect, slider_value_label, slider_value_rect,
    SLIDER_FONT_SIZE, SLIDER_HORIZONTAL_INSET, SLIDER_LABEL_WIDTH, SLIDER_LINE_HEIGHT,
    SLIDER_THUMB_HALO_SIZE, SLIDER_TRACK_RADIUS,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

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

fn slider_style(node: &TemplatePaneNodeData) -> WorkbenchSliderStyle {
    select_workbench_slider_style(node)
}

#[cfg(test)]
#[path = "template_sliders_tests.rs"]
mod tests;
