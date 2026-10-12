#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct WorldSpaceUiSurfaceSubmission {
    pub surface_id: String,
    pub node_id: String,
    pub control_id: String,
    pub viewport_x: f32,
    pub viewport_y: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub world_position: [f32; 3],
    pub world_rotation: [f32; 3],
    pub world_scale: [f32; 3],
    pub world_width: f32,
    pub world_height: f32,
    pub pixels_per_meter: f32,
    pub billboard: bool,
    pub depth_test: bool,
    pub render_order: i32,
    pub camera_target: String,
}

impl WorldSpaceUiSurfaceSubmission {
    pub(crate) fn contains_viewport_point(&self, x: f32, y: f32) -> bool {
        x >= self.viewport_x
            && y >= self.viewport_y
            && x <= self.viewport_x + self.viewport_width.max(0.0)
            && y <= self.viewport_y + self.viewport_height.max(0.0)
    }
}
