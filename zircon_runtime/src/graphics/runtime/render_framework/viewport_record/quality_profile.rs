use crate::core::framework::render::RenderQualityProfile;

use super::viewport_record::ViewportRecord;

impl ViewportRecord {
    pub(in crate::graphics::runtime::render_framework) fn quality_profile(
        &self,
    ) -> Option<&RenderQualityProfile> {
        self.quality_profile.as_ref()
    }

    pub(in crate::graphics::runtime::render_framework) fn set_quality_profile(
        &mut self,
        profile: RenderQualityProfile,
    ) {
        self.quality_profile = Some(profile);
    }
}
