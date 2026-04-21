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
            compiled_pipeline: None,
            last_capture: None,
            history: None,
            hybrid_gi_runtime: None,
            virtual_geometry_runtime: None,
        }
    }
}
