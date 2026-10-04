//! 质量档位到图编译选项的转换集中处理能力与 provider 降级，避免各提交路径各自解释档位。
use crate::core::framework::render::{
    AdvancedProviderAvailability, RenderCapabilitySummary, RenderQualityProfile,
};

use crate::graphics::RenderPipelineCompileOptions;

use super::apply_disabled_profile_features::apply_disabled_profile_features;
use super::apply_flagship_profile_features::apply_flagship_profile_features;
use super::new_compile_options::new_compile_options;

pub(in crate::graphics::runtime::render_framework) fn compile_options_for_profile(
    profile: Option<&RenderQualityProfile>,
    capabilities: &RenderCapabilitySummary,
    availability: &AdvancedProviderAvailability,
) -> RenderPipelineCompileOptions {
    let options = new_compile_options(profile, capabilities);
    let options = apply_disabled_profile_features(profile, options);
    apply_flagship_profile_features(profile, capabilities, availability, options)
}

#[cfg(test)]
#[path = "tests/compile_options_for_profile.rs"]
mod tests;
