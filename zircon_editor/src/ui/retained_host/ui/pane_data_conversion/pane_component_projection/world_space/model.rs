pub(in super::super) struct ProjectedWorldSpace {
    pub(in super::super) enabled: bool,
    pub(in super::super) position_x: f32,
    pub(in super::super) position_y: f32,
    pub(in super::super) position_z: f32,
    pub(in super::super) rotation_x: f32,
    pub(in super::super) rotation_y: f32,
    pub(in super::super) rotation_z: f32,
    pub(in super::super) scale_x: f32,
    pub(in super::super) scale_y: f32,
    pub(in super::super) scale_z: f32,
    pub(in super::super) width: f32,
    pub(in super::super) height: f32,
    pub(in super::super) pixels_per_meter: f32,
    pub(in super::super) billboard: bool,
    pub(in super::super) depth_test: bool,
    pub(in super::super) render_order: i32,
    pub(in super::super) camera_target: String,
}

impl Default for ProjectedWorldSpace {
    fn default() -> Self {
        Self {
            enabled: false,
            position_x: 0.0,
            position_y: 0.0,
            position_z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            width: 0.0,
            height: 0.0,
            pixels_per_meter: 0.0,
            billboard: false,
            depth_test: false,
            render_order: 0,
            camera_target: String::new(),
        }
    }
}
