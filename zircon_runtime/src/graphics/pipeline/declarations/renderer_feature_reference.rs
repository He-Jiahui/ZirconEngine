use crate::asset::AssetReference;

/// Shader/material expectations declared by a renderer feature asset.
/// 文档转换和运行时 contract diagnostic 共同消费这些字段来检查资源是否匹配。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RendererFeatureAssetReferences {
    /// 可选 shader 资产；声明接口列表时由文档校验要求存在。
    pub shader: Option<AssetReference>,
    pub material: Option<AssetReference>,
    /// shader 必须导出的入口名称，要求无重复项。
    pub required_entry_points: Vec<String>,
    /// shader/material 期望的属性名称，要求无重复项。
    pub expected_properties: Vec<String>,
    /// shader 期望的纹理槽位名称，要求无重复项。
    pub expected_texture_slots: Vec<String>,
}
