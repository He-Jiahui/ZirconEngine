//! 注册、切换管线与质量档位用设备摘要预检；提交时仍核对按当前视图编译的实际图。
use crate::core::framework::render::{
    RenderCapabilityMismatchDetail, RenderCapabilitySummary, RenderFrameworkError,
    RenderPipelineHandle, RenderQualityProfile, SolariCapabilityRequirement,
};
use crate::graphics::{CompiledRenderPipeline, RenderFeatureCapabilityRequirement};

pub(in crate::graphics::runtime::render_framework) fn validate_quality_profile_capabilities(
    pipeline: Option<RenderPipelineHandle>,
    profile: &RenderQualityProfile,
    capabilities: &RenderCapabilitySummary,
) -> Result<(), RenderFrameworkError> {
    let missing = missing_capability_details(profile.capability_requirements(), capabilities);

    if missing.is_empty() {
        return Ok(());
    }

    let missing_labels = missing_labels(&missing);

    Err(RenderFrameworkError::CapabilityMismatch {
        pipeline: pipeline.map(RenderPipelineHandle::raw).unwrap_or(0),
        reason: format!(
            "quality profile `{}` requires {}",
            profile.name,
            missing_labels.join(", ")
        ),
        missing,
    })
}

pub(in crate::graphics::runtime::render_framework) fn validate_compiled_pipeline_capabilities(
    pipeline: &CompiledRenderPipeline,
    capabilities: &RenderCapabilitySummary,
) -> Result<(), RenderFrameworkError> {
    let missing = missing_capability_details(
        pipeline.capability_requirements.iter().copied(),
        capabilities,
    );

    if missing.is_empty() {
        return Ok(());
    }

    let missing_labels = missing_labels(&missing);

    Err(RenderFrameworkError::CapabilityMismatch {
        pipeline: pipeline.handle.raw(),
        reason: format!(
            "pipeline `{}` requires {}",
            pipeline.name,
            missing_labels.join(", ")
        ),
        missing,
    })
}

fn missing_capability_details(
    requirements: impl IntoIterator<Item = RenderFeatureCapabilityRequirement>,
    capabilities: &RenderCapabilitySummary,
) -> Vec<RenderCapabilityMismatchDetail> {
    requirements
        .into_iter()
        .filter(|requirement| !requirement.is_satisfied_by(capabilities))
        .map(|requirement| RenderCapabilityMismatchDetail::new(requirement.capability_kind()))
        .collect()
}

fn missing_labels(missing: &[RenderCapabilityMismatchDetail]) -> Vec<&'static str> {
    missing.iter().map(|detail| (*detail).label()).collect()
}

trait RenderQualityProfileCapabilityRequirements {
    fn capability_requirements(&self) -> Vec<RenderFeatureCapabilityRequirement>;
}

impl RenderQualityProfileCapabilityRequirements for RenderQualityProfile {
    fn capability_requirements(&self) -> Vec<RenderFeatureCapabilityRequirement> {
        let mut requirements = Vec::with_capacity(quality_profile_capability_capacity(
            self.features.anti_alias,
            self.features.solari,
        ));
        if self.features.anti_alias {
            requirements.push(RenderFeatureCapabilityRequirement::ScreenSpaceAntiAlias);
        }
        if self.features.solari {
            for requirement in SolariCapabilityRequirement::ALL {
                push_unique_requirement(
                    &mut requirements,
                    RenderFeatureCapabilityRequirement::from_capability_kind(
                        requirement.capability_kind(),
                    ),
                );
            }
        }
        requirements
    }
}

fn quality_profile_capability_capacity(anti_alias: bool, solari: bool) -> usize {
    usize::from(anti_alias)
        + if solari {
            SolariCapabilityRequirement::ALL.len()
        } else {
            0
        }
}

fn push_unique_requirement(
    requirements: &mut Vec<RenderFeatureCapabilityRequirement>,
    requirement: RenderFeatureCapabilityRequirement,
) {
    if !requirements.contains(&requirement) {
        requirements.push(requirement);
    }
}

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;

#[cfg(test)]
#[path = "tests/profile_capacity_tests.rs"]
mod profile_capacity_tests;
