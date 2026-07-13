use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetToolkitDescriptor,
    AssetTypeId, AssetTypePresentation, ThumbnailProviderDescriptor,
};
use crate::core::asset::AssetSourceWritePolicy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetTypeDefinition {
    pub(super) id: AssetTypeId,
    pub(super) runtime_kind: Option<ResourceKind>,
    pub(super) source_write_policy: AssetSourceWritePolicy,
    pub(super) presentation: AssetTypePresentation,
    pub(super) thumbnail: ThumbnailProviderDescriptor,
    pub(super) toolkit: Option<AssetToolkitDescriptor>,
    pub(super) creation_templates: Vec<AssetCreationTemplateDescriptor>,
    pub(super) context_commands: Vec<AssetContextCommandDescriptor>,
}

impl AssetTypeDefinition {
    pub fn id(&self) -> &AssetTypeId {
        &self.id
    }

    pub fn runtime_kind(&self) -> Option<ResourceKind> {
        self.runtime_kind
    }

    pub fn source_write_policy(&self) -> AssetSourceWritePolicy {
        self.source_write_policy
    }

    pub fn presentation(&self) -> &AssetTypePresentation {
        &self.presentation
    }

    pub fn thumbnail(&self) -> &ThumbnailProviderDescriptor {
        &self.thumbnail
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
