use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

const TREE_OBJECT_BLUE: [u8; 4] = [82, 148, 240, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TreeIconKind {
    Cube,
    PlayerStart,
    Audio,
}

pub(super) fn push_tree_disclosure_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if node.expanded {
        push_down_chevron(commands, rect, clip, order, color, opacity);
    } else {
        push_right_chevron(commands, rect, clip, order, color, opacity);
    }
}

pub(super) fn push_tree_object_icon_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    state: UiPainterResolvedState,
    opacity: f32,
) {
    match tree_icon_kind(node) {
        TreeIconKind::Audio => push_audio_icon(commands, rect, clip, order, color, opacity),
        TreeIconKind::PlayerStart => {
            let icon_color = if is_unavailable_tree_row_state(state) {
                color
            } else {
                TREE_OBJECT_BLUE
            };
            push_player_start_icon(commands, rect, clip, order, icon_color, opacity)
        }
        TreeIconKind::Cube => push_cube_icon(commands, rect, clip, order, color, opacity),
    }
}

pub(super) fn push_tree_eye_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_eye_icon(commands, rect, clip, order, color, opacity);
}

pub(super) fn push_tree_lock_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_lock_icon(commands, rect, clip, order, color, opacity);
}

pub(super) fn push_tree_kebab_action_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_kebab_icon(commands, rect, clip, order, color, opacity);
}

fn tree_icon_kind(node: &TemplatePaneNodeData) -> TreeIconKind {
    let id = node.control_id.as_str();
    let label = node.text.as_str();
    if id.contains("Audio") || label.contains("Audio") {
        TreeIconKind::Audio
    } else if id.contains("Player") || label.contains("Player") {
        TreeIconKind::PlayerStart
    } else {
        TreeIconKind::Cube
    }
}

fn is_unavailable_tree_row_state(state: UiPainterResolvedState) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
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
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 2.0, 8.0, 1.0),
            local_rect(rect, 2.0, 3.0, 1.0, 7.0),
            local_rect(rect, 11.0, 3.0, 1.0, 7.0),
            local_rect(rect, 3.0, 10.0, 8.0, 1.0),
            local_rect(rect, 6.0, 0.0, 1.0, 3.0),
            local_rect(rect, 6.0, 10.0, 1.0, 3.0),
            local_rect(rect, 2.0, 6.0, 10.0, 1.0),
        ],
    );
}

fn push_player_start_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 6.0, 1.0, 2.0, 3.0),
            local_rect(rect, 3.0, 4.0, 8.0, 2.0),
            local_rect(rect, 2.0, 7.0, 4.0, 4.0),
            local_rect(rect, 8.0, 7.0, 4.0, 4.0),
            local_rect(rect, 5.0, 11.0, 4.0, 2.0),
        ],
    );
}

fn push_audio_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 2.0, 5.0, 3.0, 4.0),
            local_rect(rect, 5.0, 3.0, 2.0, 8.0),
            local_rect(rect, 8.0, 4.0, 1.0, 2.0),
            local_rect(rect, 10.0, 3.0, 1.0, 4.0),
            local_rect(rect, 8.0, 8.0, 1.0, 2.0),
            local_rect(rect, 10.0, 7.0, 1.0, 4.0),
        ],
    );
}

fn push_eye_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 2.0, 6.0, 2.0, 2.0),
            local_rect(rect, 4.0, 4.0, 6.0, 1.0),
            local_rect(rect, 4.0, 9.0, 6.0, 1.0),
            local_rect(rect, 10.0, 6.0, 2.0, 2.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
        ],
    );
}

fn push_lock_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 4.0, 6.0, 7.0, 6.0),
            local_rect(rect, 5.0, 3.0, 5.0, 1.0),
            local_rect(rect, 4.0, 4.0, 1.0, 3.0),
            local_rect(rect, 10.0, 4.0, 1.0, 3.0),
        ],
    );
}

fn push_kebab_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 6.0, 2.0, 2.0, 2.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
            local_rect(rect, 6.0, 10.0, 2.0, 2.0),
        ],
    );
}

fn push_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 4.0, 2.0, 2.0),
            local_rect(rect, 5.0, 6.0, 2.0, 2.0),
            local_rect(rect, 7.0, 4.0, 2.0, 2.0),
        ],
    );
}

fn push_right_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 4.0, 3.0, 2.0, 3.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
            local_rect(rect, 4.0, 8.0, 2.0, 3.0),
        ],
    );
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[FrameRect],
) {
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment.clone(),
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

fn local_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x: origin.x + x,
        y: origin.y + y,
        width,
        height,
    }
}
