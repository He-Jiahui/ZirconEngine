use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;

pub(super) const SECTION_ICON_SIZE: f32 = 14.0;
pub(super) const SECTION_ICON_GAP: f32 = 8.0;
const SECTION_GLYPH: [u8; 4] = [155, 173, 181, 255];
pub(super) const SECTION_TRANSFORM_GLYPH: [u8; 4] = [155, 173, 181, 97];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SectionTitleIcon {
    Cube,
    Transform,
    Mesh,
}

pub(super) fn section_title_icon(node: &TemplatePaneNodeData) -> Option<SectionTitleIcon> {
    match node.control_id.as_str() {
        "WorkbenchInspectorTitle" => Some(SectionTitleIcon::Cube),
        "WorkbenchTransformLabel" => Some(SectionTitleIcon::Transform),
        "WorkbenchMeshLabel" => Some(SectionTitleIcon::Mesh),
        _ => None,
    }
}

pub(super) fn push_section_icon(
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

pub(super) fn section_icon_color(icon: SectionTitleIcon) -> [u8; 4] {
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
