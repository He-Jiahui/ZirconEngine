use crate::core::framework::render::RenderViewportDescriptor;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn new(
        descriptor: RenderViewportDescriptor,
    ) -> Self {
        Self {
            descriptor,
            pipeline: None,
            quality_profile: None,
            generation: 0,
            temporal_frame_index: 0,
            compiled_pipeline: None,
            hybrid_gi_runtime: None,
            virtual_geometry_runtime: None,
            last_capture: None,
            history: None,
            visibility_static_index: None,
            motion_vector_camera: None,
            particle_previous_sprites: Vec::new(),
            surface: None,
        }
    }
}
