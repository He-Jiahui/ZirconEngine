use crate::graphics::FrameHistoryBinding;

use super::render_feature_descriptor::RenderFeatureDescriptor;
use super::render_feature_descriptor::RenderFeatureGraphMutation;
use super::render_feature_descriptor::RenderFeaturePassResourceExtension;
use crate::graphics::feature::{
    RenderFeatureCapabilityRequirement, RenderFeaturePassDescriptor, RenderFeatureResourceAccess,
    RenderFeatureResourceDescriptor, RenderFeatureResourceKind, RenderFeatureResourceWriteMode,
};
use crate::render_graph::RenderGraphExternalResourceBinding;

impl RenderFeatureDescriptor {
    pub fn new(
        name: impl Into<String>,
        required_extract_sections: Vec<String>,
        history_bindings: Vec<FrameHistoryBinding>,
        stage_passes: Vec<RenderFeaturePassDescriptor>,
    ) -> Self {
        Self {
            name: name.into(),
            required_extract_sections,
            capability_requirements: Vec::new(),
            history_bindings,
            stage_passes,
            pass_resource_extensions: Vec::new(),
        }
    }

    /// Replaces one existing graph pass while this feature is active. The
    /// compiler requires the target to exist exactly once and rejects
    /// competing replacement owners.
    pub fn with_replaced_pass(mut self, pass_name: impl Into<String>) -> Self {
        let pass_name = pass_name.into();
        if !self.replaced_passes().any(|existing| existing == pass_name) {
            self.pass_resource_extensions
                .push(RenderFeatureGraphMutation::ReplacePass {
                    target_pass_name: pass_name,
                });
        }
        self
    }

    /// Activates this descriptor only when the selected camera supplies OIT
    /// settings in the advanced-lighting extract.
    pub fn when_advanced_lighting_oit_enabled(mut self) -> Self {
        if !self.requires_advanced_lighting_oit() {
            self.pass_resource_extensions
                .push(RenderFeatureGraphMutation::RequireAdvancedLightingOit);
        }
        self
    }

    /// Activates this descriptor only for a mirror camera whose selected
    /// texture target is owned by an extracted planar reflection probe.
    pub fn when_advanced_lighting_planar_capture_enabled(mut self) -> Self {
        if !self.requires_advanced_lighting_planar_capture() {
            self.pass_resource_extensions
                .push(RenderFeatureGraphMutation::RequireAdvancedLightingPlanarCapture);
        }
        self
    }

    /// Activates this descriptor only for a deferred view with at least one
    /// resolved subsurface profile in the advanced-lighting extract.
    pub fn when_advanced_lighting_subsurface_enabled(mut self) -> Self {
        if !self.requires_advanced_lighting_subsurface() {
            self.pass_resource_extensions
                .push(RenderFeatureGraphMutation::RequireAdvancedLightingSubsurface);
        }
        self
    }

    pub fn with_capability_requirement(
        mut self,
        requirement: RenderFeatureCapabilityRequirement,
    ) -> Self {
        if !self.capability_requirements.contains(&requirement) {
            self.capability_requirements.push(requirement);
        }
        self
    }

    /// Extends an existing pass owned by another feature with a transient
    /// texture read. The extension is applied only when both features are
    /// active in the compiled pipeline.
    pub fn with_pass_read_texture(
        mut self,
        target_pass_name: impl Into<String>,
        resource_name: impl Into<String>,
    ) -> Self {
        self.pass_resource_extensions
            .push(RenderFeatureGraphMutation::PassResource(
                RenderFeaturePassResourceExtension {
                    target_pass_name: target_pass_name.into(),
                    resource: RenderFeatureResourceDescriptor {
                        name: resource_name.into(),
                        kind: RenderFeatureResourceKind::Texture,
                        access: RenderFeatureResourceAccess::Read,
                        minimum_size_bytes: None,
                        attachment_ops: None,
                        write_mode: RenderFeatureResourceWriteMode::Attachment,
                        external_binding: RenderGraphExternalResourceBinding::report_only(),
                    },
                },
            ));
        self
    }

    /// Extends an existing graphics pass with an attachment write while this
    /// feature is active. This keeps feature-owned MRTs absent from disabled
    /// frame graphs.
    pub fn with_pass_write_texture(
        mut self,
        target_pass_name: impl Into<String>,
        resource_name: impl Into<String>,
        attachment_ops: crate::render_graph::RenderGraphAttachmentOps,
    ) -> Self {
        self.pass_resource_extensions
            .push(RenderFeatureGraphMutation::PassResource(
                RenderFeaturePassResourceExtension {
                    target_pass_name: target_pass_name.into(),
                    resource: RenderFeatureResourceDescriptor {
                        name: resource_name.into(),
                        kind: RenderFeatureResourceKind::Texture,
                        access: RenderFeatureResourceAccess::Write,
                        minimum_size_bytes: None,
                        attachment_ops: Some(attachment_ops),
                        write_mode: RenderFeatureResourceWriteMode::Attachment,
                        external_binding: RenderGraphExternalResourceBinding::report_only(),
                    },
                },
            ));
        self
    }
}
