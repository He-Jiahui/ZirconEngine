use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetToolkitDescriptor,
    AssetTypeId, AssetTypePresentation, ThumbnailProviderDescriptor,
};
use crate::core::asset::AssetSourceWritePolicy;

/// Serializable plugin contribution merged into one materialized asset type definition.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetTypeContribution {
    pub(super) asset_type: AssetTypeId,
    pub(super) runtime_kind: Option<ResourceKind>,
    pub(super) source_write_policy: Option<AssetSourceWritePolicy>,
    pub(super) presentation: Option<AssetTypePresentation>,
    pub(super) thumbnail: Option<ThumbnailProviderDescriptor>,
    pub(super) toolkit: Option<AssetToolkitDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) creation_templates: Vec<AssetCreationTemplateDescriptor>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) context_commands: Vec<AssetContextCommandDescriptor>,
}

impl AssetTypeContribution {
    pub fn augment(asset_type: AssetTypeId) -> Self {
        Self {
            asset_type,
            runtime_kind: None,
            source_write_policy: None,
            presentation: None,
            thumbnail: None,
            toolkit: None,
            creation_templates: Vec::new(),
            context_commands: Vec::new(),
        }
    }

    pub fn define(
        asset_type: AssetTypeId,
        presentation: AssetTypePresentation,
        thumbnail: ThumbnailProviderDescriptor,
    ) -> Self {
        Self::augment(asset_type)
            .with_source_write_policy(AssetSourceWritePolicy::ProjectOnly)
            .with_presentation(presentation)
            .with_thumbnail(thumbnail)
    }

    pub fn with_runtime_kind(mut self, runtime_kind: ResourceKind) -> Self {
        self.runtime_kind = Some(runtime_kind);
        self
    }

    pub fn with_source_write_policy(mut self, policy: AssetSourceWritePolicy) -> Self {
        self.source_write_policy = Some(policy);
        self
    }

    pub fn with_presentation(mut self, presentation: AssetTypePresentation) -> Self {
        self.presentation = Some(presentation);
        self
    }

    pub fn with_thumbnail(mut self, thumbnail: ThumbnailProviderDescriptor) -> Self {
        self.thumbnail = Some(thumbnail);
        self
    }

    pub fn with_toolkit(mut self, toolkit: AssetToolkitDescriptor) -> Self {
        self.toolkit = Some(toolkit);
        self
    }

    pub fn with_creation_template(mut self, template: AssetCreationTemplateDescriptor) -> Self {
        self.creation_templates.push(template);
        self
    }

    pub fn with_context_command(mut self, command: AssetContextCommandDescriptor) -> Self {
        self.context_commands.push(command);
        self
    }

    pub fn asset_type(&self) -> &AssetTypeId {
        &self.asset_type
    }

    pub fn toolkit(&self) -> Option<&AssetToolkitDescriptor> {
        self.toolkit.as_ref()
    }

    pub fn creation_templates(&self) -> &[AssetCreationTemplateDescriptor] {
        &self.creation_templates
    }

    pub fn context_commands(&self) -> &[AssetContextCommandDescriptor] {
        &self.context_commands
    }
}
