//! 资源工作区和派发端消费的已物化类型快照；它组合来源策略、展示、工具及操作描述，调用方应查询此结果而非重新拼接插件贡献。
use zircon_runtime_interface::resource::ResourceKind;

use super::{
    AssetContextCommandDescriptor, AssetCreationTemplateDescriptor, AssetToolkitDescriptor,
    AssetTypeId, AssetTypePresentation, ThumbnailProviderDescriptor,
};
use crate::core::asset::AssetSourceWritePolicy;

/// 插件贡献完成物化后的只读类型契约，供操作派发与资产工作区共享查询。
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
