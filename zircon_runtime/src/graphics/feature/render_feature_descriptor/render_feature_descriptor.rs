use crate::graphics::FrameHistoryBinding;

use super::super::render_feature_capability_requirement::RenderFeatureCapabilityRequirement;
use super::super::render_feature_pass_descriptor::RenderFeaturePassDescriptor;
use super::super::render_feature_pass_descriptor::RenderFeatureResourceDescriptor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RenderFeaturePassResourceExtension {
    pub(crate) target_pass_name: String,
    pub(crate) resource: RenderFeatureResourceDescriptor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RenderFeatureGraphMutation {
    PassResource(RenderFeaturePassResourceExtension),
    ReplacePass { target_pass_name: String },
    RequireAdvancedLightingOit,
    RequireAdvancedLightingPlanarCapture,
    RequireAdvancedLightingSubsurface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderFeatureDescriptor {
    pub name: String,
    pub required_extract_sections: Vec<String>,
    pub capability_requirements: Vec<RenderFeatureCapabilityRequirement>,
    pub history_bindings: Vec<FrameHistoryBinding>,
    pub stage_passes: Vec<RenderFeaturePassDescriptor>,
    pub(crate) pass_resource_extensions: Vec<RenderFeatureGraphMutation>,
}

impl RenderFeatureDescriptor {
    pub(crate) fn resource_extensions(
        &self,
    ) -> impl Iterator<Item = &RenderFeaturePassResourceExtension> {
        self.pass_resource_extensions.iter().filter_map(|mutation| {
            if let RenderFeatureGraphMutation::PassResource(extension) = mutation {
                Some(extension)
            } else {
                None
            }
        })
    }

    pub(crate) fn replaced_passes(&self) -> impl Iterator<Item = &str> {
        self.pass_resource_extensions.iter().filter_map(|mutation| {
            if let RenderFeatureGraphMutation::ReplacePass { target_pass_name } = mutation {
                Some(target_pass_name.as_str())
            } else {
                None
            }
        })
    }

    pub(crate) fn requires_advanced_lighting_oit(&self) -> bool {
        self.pass_resource_extensions
            .contains(&RenderFeatureGraphMutation::RequireAdvancedLightingOit)
    }

    pub(crate) fn requires_advanced_lighting_planar_capture(&self) -> bool {
        self.pass_resource_extensions
            .contains(&RenderFeatureGraphMutation::RequireAdvancedLightingPlanarCapture)
    }

    pub(crate) fn requires_advanced_lighting_subsurface(&self) -> bool {
        self.pass_resource_extensions
            .contains(&RenderFeatureGraphMutation::RequireAdvancedLightingSubsurface)
    }
}
