use crate::ui::retained_host as host_contract;

use super::super::clip_frame::ProjectedClipFrame;
use super::super::world_space::ProjectedWorldSpace;

pub(super) fn assign_spatial_fields(
    node: &mut host_contract::TemplatePaneNodeData,
    world_space: ProjectedWorldSpace,
    clip_frame: ProjectedClipFrame,
) {
    node.world_space_enabled = world_space.enabled;
    node.world_position_x = world_space.position_x;
    node.world_position_y = world_space.position_y;
    node.world_position_z = world_space.position_z;
    node.world_rotation_x = world_space.rotation_x;
    node.world_rotation_y = world_space.rotation_y;
    node.world_rotation_z = world_space.rotation_z;
    node.world_scale_x = world_space.scale_x;
    node.world_scale_y = world_space.scale_y;
    node.world_scale_z = world_space.scale_z;
    node.world_width = world_space.width;
    node.world_height = world_space.height;
    node.world_pixels_per_meter = world_space.pixels_per_meter;
    node.world_billboard = world_space.billboard;
    node.world_depth_test = world_space.depth_test;
    node.world_render_order = world_space.render_order;
    node.world_camera_target = world_space.camera_target.into();

    node.has_clip_frame = clip_frame.has_clip_frame;
    node.clip_frame = clip_frame.frame;
}
