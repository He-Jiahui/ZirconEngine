use zircon_framework::render::{RenderCapabilitySummary, RenderQualityProfile};

use crate::RenderPipelineCompileOptions;

use super::apply_disabled_profile_features::apply_disabled_profile_features;
use super::apply_flagship_profile_features::apply_flagship_profile_features;
use super::new_compile_options::new_compile_options;

pub(in crate::runtime::render_framework) fn compile_options_for_profile(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
) -> RenderPipelineCompileOptions {
    let options = new_compile_options(profile, capabilities);
    let options = apply_disabled_profile_features(profile, options);
    apply_flagship_profile_features(profile, capabilities, options)
}
