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
            last_capture_pipeline: None,
            capture_mailbox: Default::default(),
            pending_capture_profiles: Default::default(),
            last_promoted_capture_generation: None,
            hybrid_gi_runtimes: Default::default(),
            virtual_geometry_runtimes: Default::default(),
            light_grid_reports: Default::default(),
            virtual_geometry_debug_snapshots: Default::default(),
            last_capture: None,
            last_visible_spatial_query: None,
            camera_histories: Default::default(),
            motion_vector_cameras: Default::default(),
            particle_previous_sprites: Default::default(),
            surface: None,
        }
    }
}
