mod dispatch;
mod feature_descriptors;

use super::builtin_render_feature;
use super::render_feature_descriptor;
use super::render_feature_pass_descriptor;

pub(crate) use feature_descriptors::SsaoParams;

pub(crate) fn configure_screen_space_ambient_occlusion_for_profile(
    descriptors: &mut [super::RenderFeatureDescriptor],
    profile: &crate::graphics::pipeline::CompiledAoProfile,
) -> Result<(), String> {
    feature_descriptors::configure_screen_space_ambient_occlusion_for_profile(descriptors, profile)
}

pub fn screen_space_ambient_occlusion_render_feature_descriptor() -> super::RenderFeatureDescriptor
{
    feature_descriptors::screen_space_ambient_occlusion_descriptor()
}
