use zircon_runtime_interface::ui::{
    event_ui::UiNodeId, layout::UiFrame, surface::UiRenderCommand, tree::UiTemplateNodeMetadata,
};

use super::{
    commands::{quad_command, text_command},
    metadata::{
        group_label, metric_attribute, option_is_selected, option_label, segmented_options,
        selected_segment_value,
    },
    state::SegmentedRenderState,
    style::{
        SegmentedVisual, divider_color, group_label_color, option_text_color, segmented_background,
        segmented_border, selected_surface, selected_underline,
    },
};

#[allow(clippy::too_many_arguments)]
pub(super) fn segmented_commands(
    node_id: UiNodeId,
    metadata: &UiTemplateNodeMetadata,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    frame: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) -> Vec<UiRenderCommand> {
    let options = segmented_options(metadata);
    if options.is_empty() {
        return Vec::new();
    }
    let mut commands = Vec::new();
    let label = group_label(metadata);
    let has_label = label.is_some();
    if let Some(label) = label {
        commands.push(text_command(
            node_id,
            UiFrame::new(frame.x, frame.y, frame.width, visual.group_label_height),
            clip,
            z.saturating_add(3),
            label,
            group_label_color(state, visual),
            visual.group_label_font_size,
            visual.group_label_line_height,
            state,
            opacity,
        ));
    }
    let body = segmented_body_frame(metadata, frame, has_label, visual);
    commands.push(quad_command(
        node_id,
        body,
        clip,
        z.saturating_add(1),
        segmented_background(state, visual),
        Some(segmented_border(state, visual)),
        visual.border_width,
        visual.corner_radius,
        state,
        opacity,
    ));
    let selected = selected_segment_value(metadata);
    for (index, option) in options.iter().enumerate() {
        let segment = segment_frame(body, index, options.len());
        if index > 0 {
            commands.push(quad_command(
                node_id,
                UiFrame::new(
                    segment.x,
                    segment.y + visual.segment_text_inset_y - visual.border_width,
                    visual.border_width,
                    (segment.height - (visual.segment_text_inset_y - visual.border_width) * 2.0)
                        .max(visual.min_frame_extent),
                ),
                clip,
                z.saturating_add(2),
                divider_color(state, visual),
                None,
                0.0,
                0.0,
                state,
                opacity,
            ));
        }
        let option_selected = option_is_selected(option, selected);
        if option_selected {
            push_selected_segment(
                &mut commands,
                node_id,
                state,
                visual,
                segment,
                clip,
                z.saturating_add(3),
                opacity,
            );
        }
        commands.push(text_command(
            node_id,
            UiFrame::new(
                segment.x + visual.segment_text_inset_x,
                segment.y + visual.segment_text_inset_y,
                (segment.width - visual.segment_text_inset_x * 2.0).max(visual.min_frame_extent),
                (segment.height - visual.segment_text_inset_y * 2.0).max(visual.line_height),
            ),
            clip,
            z.saturating_add(5),
            option_label(option),
            option_text_color(state, visual, option_selected),
            visual.font_size,
            visual.line_height,
            state,
            opacity,
        ));
    }
    commands
}

#[allow(clippy::too_many_arguments)]
fn push_selected_segment(
    commands: &mut Vec<UiRenderCommand>,
    node_id: UiNodeId,
    state: &SegmentedRenderState,
    visual: &SegmentedVisual,
    segment: UiFrame,
    clip: Option<UiFrame>,
    z: i32,
    opacity: f32,
) {
    let selected = inset_frame(segment, visual.selected_inset, visual.min_frame_extent);
    commands.push(quad_command(
        node_id,
        selected,
        clip,
        z,
        selected_surface(state, visual),
        (visual.selected_border_width > 0.0).then_some(visual.selected_border),
        visual.selected_border_width,
        (visual.corner_radius - visual.border_width).max(0.0),
        state,
        opacity,
    ));
    if visual.tab_underline_height > 0.0 {
        commands.push(quad_command(
            node_id,
            UiFrame::new(
                selected.x,
                selected.y + (selected.height - visual.tab_underline_height).max(0.0),
                selected.width,
                visual
                    .tab_underline_height
                    .min(selected.height)
                    .max(visual.min_frame_extent),
            ),
            clip,
            z.saturating_add(1),
            selected_underline(state, visual),
            None,
            0.0,
            0.0,
            state,
            opacity,
        ));
    }
}

fn segmented_body_frame(
    metadata: &UiTemplateNodeMetadata,
    frame: UiFrame,
    has_label: bool,
    visual: &SegmentedVisual,
) -> UiFrame {
    let label_block = if has_label {
        visual.group_label_height + visual.group_label_gap
    } else {
        0.0
    };
    UiFrame::new(
        frame.x + metric_attribute(metadata, "layout_offset_x").unwrap_or(0.0),
        frame.y + label_block + metric_attribute(metadata, "layout_offset_y").unwrap_or(0.0),
        frame.width,
        (frame.height - label_block).max(visual.min_frame_extent),
    )
}

fn segment_frame(frame: UiFrame, index: usize, count: usize) -> UiFrame {
    let count = count.max(1);
    let width = frame.width / count as f32;
    let x = frame.x + width * index as f32;
    UiFrame::new(
        x,
        frame.y,
        if index + 1 == count {
            frame.x + frame.width - x
        } else {
            width
        }
        .max(f32::EPSILON),
        frame.height,
    )
}

fn inset_frame(frame: UiFrame, inset: f32, min: f32) -> UiFrame {
    UiFrame::new(
        frame.x + inset,
        frame.y + inset,
        (frame.width - inset * 2.0).max(min),
        (frame.height - inset * 2.0).max(min),
    )
}
