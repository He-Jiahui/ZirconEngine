use crate::core::framework::render::EnvironmentCubemapUploadReport;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn persistent_texture_resident_bytes(&self) -> u64 {
        self.streamer.persistent_texture_resident_bytes()
    }

    pub(crate) fn set_mip_streaming_residency_budget(&mut self, bytes: u64) {
        self.streamer.set_mip_streaming_residency_budget(bytes);
    }

    pub(crate) fn environment_cubemap_upload_report(&self) -> EnvironmentCubemapUploadReport {
        self.core.scene_environment_cubemap.upload_report()
    }
}
