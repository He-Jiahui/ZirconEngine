use zircon_runtime::graphics::{RenderFeatureCapabilityRequirement, RenderFeatureDescriptor};

pub const FEATURE_ID: &str = "rendering.ray_tracing_policy";
pub const FEATURE_NAME: &str = "ray_tracing_policy";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RayTracingBackendCapabilities {
    pub acceleration_structures: bool,
    pub inline_ray_query: bool,
    pub ray_tracing_pipeline: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RayTracingPath {
    Disabled,
    InlineQuery,
    Pipeline,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RayTracingPolicyReport {
    pub requested_path: RayTracingPath,
    pub supported: bool,
    pub missing_gates: Vec<RenderFeatureCapabilityRequirement>,
}

impl RayTracingPolicyReport {
    pub fn from_backend(
        requested_path: RayTracingPath,
        backend: RayTracingBackendCapabilities,
    ) -> Self {
        let mut missing_gates = Vec::new();
        if requested_path != RayTracingPath::Disabled && !backend.acceleration_structures {
            missing_gates.push(RenderFeatureCapabilityRequirement::AccelerationStructures);
        }
        if requested_path == RayTracingPath::InlineQuery && !backend.inline_ray_query {
            missing_gates.push(RenderFeatureCapabilityRequirement::InlineRayQuery);
        }
        if requested_path == RayTracingPath::Pipeline && !backend.ray_tracing_pipeline {
            missing_gates.push(RenderFeatureCapabilityRequirement::RayTracingPipeline);
        }
        Self {
            requested_path,
            supported: missing_gates.is_empty(),
            missing_gates,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RenderingRayTracingPolicyRuntimeFeature;

impl zircon_runtime::plugin::RuntimePluginFeature for RenderingRayTracingPolicyRuntimeFeature {
    fn manifest(&self) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
        feature_manifest()
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_render_feature(render_feature_descriptor())
    }
}

pub fn runtime_plugin_feature() -> RenderingRayTracingPolicyRuntimeFeature {
    RenderingRayTracingPolicyRuntimeFeature
}

pub fn plugin_feature_registration(
) -> zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport {
    zircon_runtime::plugin::RuntimePluginFeatureRegistrationReport::from_feature(
        &runtime_plugin_feature(),
    )
}

pub fn feature_manifest() -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_plugin_rendering_runtime::feature_manifest(
        zircon_plugin_rendering_runtime::RenderingFeatureKind::RayTracingPolicy,
    )
}

pub fn render_feature_descriptor() -> RenderFeatureDescriptor {
    RenderFeatureDescriptor::new(FEATURE_NAME, Vec::new(), Vec::new(), Vec::new())
        .with_capability_requirement(RenderFeatureCapabilityRequirement::AccelerationStructures)
        .with_capability_requirement(RenderFeatureCapabilityRequirement::InlineRayQuery)
        .with_capability_requirement(RenderFeatureCapabilityRequirement::RayTracingPipeline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_report_lists_missing_gates() {
        let report = RayTracingPolicyReport::from_backend(
            RayTracingPath::Pipeline,
            RayTracingBackendCapabilities {
                acceleration_structures: true,
                inline_ray_query: false,
                ray_tracing_pipeline: false,
            },
        );

        assert!(!report.supported);
        assert_eq!(
            report.missing_gates,
            vec![RenderFeatureCapabilityRequirement::RayTracingPipeline]
        );
    }

    #[test]
    fn policy_feature_is_opt_in_and_capability_gated() {
        let report = plugin_feature_registration();

        assert!(report.is_success(), "{:?}", report.diagnostics);
        assert!(!report.manifest.enabled_by_default);
        assert_eq!(
            report.extensions.render_features()[0].capability_requirements,
            vec![
                RenderFeatureCapabilityRequirement::AccelerationStructures,
                RenderFeatureCapabilityRequirement::InlineRayQuery,
                RenderFeatureCapabilityRequirement::RayTracingPipeline,
            ]
        );
    }
}
