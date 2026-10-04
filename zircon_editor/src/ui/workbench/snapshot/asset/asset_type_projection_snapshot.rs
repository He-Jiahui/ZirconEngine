use crate::core::asset::{
    builtin_asset_type_definition, AssetSourceWritePolicy, AssetTypeDefinition,
};
use zircon_runtime_interface::resource::ResourceKind;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 类型注册表的UI呈现与写入策略投影；类型ID连接操作/工具契约，kind仅提供内建回退。
pub struct AssetTypeProjectionSnapshot {
    pub asset_type_id: String,
    pub display_name: String,
    pub badge: String,
    pub icon_name: String,
    pub color_token: String,
    pub source_write_policy: AssetSourceWritePolicy,
}

impl AssetTypeProjectionSnapshot {
    /// 目录初始投影的内建回退；插件扩展需通过当前注册类型定义投影。
    pub fn from_resource_kind(kind: ResourceKind) -> Self {
        builtin_asset_type_definition(kind)
            .map(Self::from_definition)
            .unwrap_or_default()
    }

    /// 发布当前注册定义的呈现和写入策略，不从文件扩展名重新推断类型权限。
    pub fn from_definition(definition: &AssetTypeDefinition) -> Self {
        Self {
            asset_type_id: definition.id().to_string(),
            display_name: definition.presentation().display_name().to_owned(),
            badge: definition.presentation().badge().to_owned(),
            icon_name: definition.presentation().icon_name().to_owned(),
            color_token: definition.presentation().color_token().to_owned(),
            source_write_policy: definition.source_write_policy(),
        }
    }
}
