use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::identity::SectionTitleIcon;
use super::super::style;
use super::cube::push_cube_icon;
use super::mesh::push_mesh_icon;
use super::transform::push_transform_icon;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_section_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    icon: SectionTitleIcon,
    opacity: f32,
) {
    let color = style::section_icon_color(icon);
    match icon {
        SectionTitleIcon::Cube => push_cube_icon(commands, rect, clip, order, color, opacity),
        SectionTitleIcon::Transform => {
            push_transform_icon(commands, rect, clip, order, color, opacity)
        }
        SectionTitleIcon::Mesh => push_mesh_icon(commands, rect, clip, order, color, opacity),
    }
}
