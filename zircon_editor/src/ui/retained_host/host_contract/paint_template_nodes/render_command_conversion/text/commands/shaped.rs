use zircon_runtime_interface::ui::surface::{
    UiShapedTextCluster, UiShapedTextLine, UiTextPaint, UiTextRange, UiTextRunPaintStyle,
};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::style::{
    frame_from_ui, text_paint_style_from_font_weight,
};

use super::metrics::{resolved_font_size, resolved_line_height};

pub(super) fn push_shaped_text_commands(
    output: &mut Vec<HostPaintCommand>,
    text: &UiTextPaint,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
    color: [u8; 4],
) -> bool {
    let Some(shaped) = text.shaped.canonical() else {
        return false;
    };

    for line in &shaped.lines {
        if let Some(text_style) = uniform_cluster_text_style(line) {
            output.push(HostPaintCommand::text(
                frame_from_ui(line.frame),
                clip_frame.clone(),
                z_index,
                line.text.clone(),
                color,
                resolved_font_size(text.font_size),
                resolved_line_height(text.font_size, text.line_height),
                text_style,
                opacity,
            ));
            continue;
        }

        if !line.clusters.is_empty() {
            push_shaped_line_cluster_commands(
                output,
                line,
                clip_frame.clone(),
                z_index,
                opacity,
                color,
                text.font_size,
                text.line_height,
            );
            continue;
        }

        output.push(HostPaintCommand::text(
            frame_from_ui(line.frame),
            clip_frame.clone(),
            z_index,
            line.text.clone(),
            color,
            resolved_font_size(text.font_size),
            resolved_line_height(text.font_size, text.line_height),
            text_paint_style_from_font_weight(text.font_weight),
            opacity,
        ));
    }
    true
}

fn uniform_cluster_text_style(line: &UiShapedTextLine) -> Option<UiTextRunPaintStyle> {
    let mut visible_cluster_count = 0;
    let mut line_style = None;
    for cluster in &line.clusters {
        if cluster.text.is_empty() {
            continue;
        }

        let cluster_style = UiTextRunPaintStyle::from_run_kind(cluster.kind);
        if let Some(style) = line_style.replace(cluster_style) {
            if style != cluster_style {
                return None;
            }
        }
        visible_cluster_count += 1;
    }

    (visible_cluster_count > 1).then_some(line_style?)
}

fn push_shaped_line_cluster_commands(
    output: &mut Vec<HostPaintCommand>,
    line: &UiShapedTextLine,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
    color: [u8; 4],
    font_size: f32,
    line_height: f32,
) {
    for cluster in &line.clusters {
        if cluster.text.is_empty() {
            continue;
        }
        output.push(HostPaintCommand::text(
            shaped_cluster_frame(line, cluster),
            clip_frame.clone(),
            z_index,
            cluster.text.clone(),
            color,
            resolved_font_size(font_size),
            resolved_line_height(font_size, line_height),
            UiTextRunPaintStyle::from_run_kind(cluster.kind),
            opacity,
        ));
    }
}

fn shaped_cluster_frame(line: &UiShapedTextLine, cluster: &UiShapedTextCluster) -> FrameRect {
    union_glyph_frames_for_range(line, cluster.source_range)
        .or_else(|| union_glyph_frames_for_range(line, cluster.visual_range))
        .unwrap_or_else(|| frame_from_ui(line.frame))
}

fn union_glyph_frames_for_range(line: &UiShapedTextLine, range: UiTextRange) -> Option<FrameRect> {
    let mut left = f32::INFINITY;
    let mut top = f32::INFINITY;
    let mut right = f32::NEG_INFINITY;
    let mut bottom = f32::NEG_INFINITY;
    let mut any = false;

    for glyph in &line.glyphs {
        if !ranges_overlap(glyph.source_range, range) {
            continue;
        }
        let frame = frame_from_ui(glyph.visual_frame);
        left = left.min(frame.x);
        top = top.min(frame.y);
        right = right.max(frame.x + frame.width);
        bottom = bottom.max(frame.y + frame.height);
        any = true;
    }

    any.then(|| FrameRect {
        x: left,
        y: top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    })
}

fn ranges_overlap(lhs: UiTextRange, rhs: UiTextRange) -> bool {
    lhs.start < rhs.end && rhs.start < lhs.end
}
