use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;
use super::style_selector::{
    WorkbenchStatusSignalKind as StatusSignalKind, WorkbenchStatusSignalStyle,
};

pub(super) const STATUS_ITEM_ICON_SIZE: f32 = 14.0;
pub(super) const STATUS_ICON_GLYPH_SIZE: f32 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StatusIconKind {
    Snap,
    World,
    Target,
}

pub(super) fn push_status_signal_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusSignalKind,
    style: WorkbenchStatusSignalStyle,
    mark_width: f32,
    opacity: f32,
) {
    match kind {
        StatusSignalKind::Ready => commands.push(HostPaintCommand::quad(
            rect.clone(),
            Some(clip.clone()),
            order,
            Some(style.icon_fill),
            None,
            0.0,
            rect.height * 0.5,
            opacity,
        )),
        StatusSignalKind::Success => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(style.icon_fill),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_check_mark(commands, rect, clip, order + 1, style.mark, opacity);
        }
        StatusSignalKind::Warning => {
            push_warning_triangle(
                commands,
                rect,
                clip,
                order,
                style.icon_fill,
                style.mark,
                mark_width,
                opacity,
            );
        }
        StatusSignalKind::Info => {
            commands.push(HostPaintCommand::quad(
                rect.clone(),
                Some(clip.clone()),
                order,
                Some(style.icon_fill),
                None,
                0.0,
                rect.height * 0.5,
                opacity,
            ));
            push_segments(
                commands,
                clip,
                order + 1,
                style.mark,
                opacity,
                &[
                    local_rect_scaled(rect, 6.0, 3.0, 2.0, 2.0, STATUS_ITEM_ICON_SIZE),
                    local_rect_scaled(rect, 6.0, 6.0, 2.0, 5.0, STATUS_ITEM_ICON_SIZE),
                ],
            );
        }
    }
}

pub(super) fn push_status_icon_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    kind: StatusIconKind,
    color: [u8; 4],
    opacity: f32,
) {
    match kind {
        StatusIconKind::Snap => push_snap_icon(commands, rect, clip, order, color, opacity),
        StatusIconKind::World => push_world_icon(commands, rect, clip, order, color, opacity),
        StatusIconKind::Target => push_target_icon(commands, rect, clip, order, color, opacity),
    }
}

pub(super) fn push_down_chevron(
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

pub(super) fn centered_rect(rect: &FrameRect, size: f32) -> FrameRect {
    FrameRect {
        x: rect.x + (rect.width - size).max(0.0) * 0.5,
        y: rect.y + (rect.height - size).max(0.0) * 0.5,
        width: size.min(rect.width.max(1.0)).max(1.0),
        height: size.min(rect.height.max(1.0)).max(1.0),
    }
}

fn push_snap_icon(
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
            local_rect(rect, 3.0, 4.0, 3.0, 8.0),
            local_rect(rect, 10.0, 4.0, 3.0, 8.0),
            local_rect(rect, 3.0, 11.0, 10.0, 3.0),
            local_rect(rect, 4.0, 2.0, 2.0, 3.0),
            local_rect(rect, 10.0, 2.0, 2.0, 3.0),
        ],
    );
}

fn push_world_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        rect.height * 0.5,
        opacity,
    ));
    push_segments(
        commands,
        clip,
        order + 1,
        color,
        opacity,
        &[
            local_rect(rect, 7.0, 2.0, 2.0, 12.0),
            local_rect(rect, 3.0, 7.0, 10.0, 2.0),
            local_rect(rect, 4.0, 4.0, 8.0, 1.0),
            local_rect(rect, 4.0, 11.0, 8.0, 1.0),
        ],
    );
}

fn push_target_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        rect.height * 0.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        centered_rect(rect, 4.0),
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        2.0,
        opacity,
    ));
    push_segments(
        commands,
        clip,
        order + 1,
        color,
        opacity,
        &[
            local_rect(rect, 7.0, 0.0, 2.0, 4.0),
            local_rect(rect, 7.0, 12.0, 2.0, 4.0),
            local_rect(rect, 0.0, 7.0, 4.0, 2.0),
            local_rect(rect, 12.0, 7.0, 4.0, 2.0),
        ],
    );
}

fn push_warning_triangle(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    mark_color: [u8; 4],
    mark_width: f32,
    opacity: f32,
) {
    let center_x = rect.x + rect.width * 0.5;
    let scale_x = rect.width / STATUS_ITEM_ICON_SIZE;
    let scale_y = rect.height / STATUS_ITEM_ICON_SIZE;
    for (row, width) in [2.0, 4.0, 6.0, 8.0, 10.0, 12.0].into_iter().enumerate() {
        let width = width * scale_x;
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: center_x - width * 0.5,
                y: rect.y + (2.0 + row as f32 * 1.7) * scale_y,
                width,
                height: 2.0 * scale_y,
            },
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
    push_segments(
        commands,
        clip,
        order + 1,
        mark_color,
        opacity,
        &warning_mark_segments(rect, mark_width),
    );
}

pub(super) fn warning_mark_segments(rect: &FrameRect, mark_width: f32) -> [FrameRect; 2] {
    let mark_width = normalized_status_mark_width(mark_width);
    let x = 7.0 - mark_width * 0.5;
    [
        local_rect_scaled(rect, x, 6.0, mark_width, 4.0, STATUS_ITEM_ICON_SIZE),
        local_rect_scaled(rect, x, 11.0, mark_width, mark_width, STATUS_ITEM_ICON_SIZE),
    ]
}

fn push_check_mark(
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
            local_rect_scaled(rect, 3.0, 7.0, 3.0, 2.0, STATUS_ITEM_ICON_SIZE),
            local_rect_scaled(rect, 5.0, 9.0, 3.0, 2.0, STATUS_ITEM_ICON_SIZE),
            local_rect_scaled(rect, 8.0, 4.0, 3.0, 7.0, STATUS_ITEM_ICON_SIZE),
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

fn local_rect_scaled(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    base_size: f32,
) -> FrameRect {
    let scale_x = origin.width / base_size;
    let scale_y = origin.height / base_size;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: width * scale_x,
        height: height * scale_y,
    }
}

pub(super) fn normalized_status_mark_width(width: f32) -> f32 {
    if width.is_finite() && width > 0.0 {
        width
    } else {
        2.0
    }
}
