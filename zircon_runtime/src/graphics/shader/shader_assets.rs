use crate::core::resource::ResourceId;
use crate::graphics::material::MaterialDomain;

// TODO: [CR-GRAPHICS-SHADER-VIS-0002] 确认这些图与程序描述在资产管线中的所有者及加载入口；当前只见公开重导出，没有生产构建/消费，且 MaterialGraphAsset 与资产 authoring 类型同名。
/// 预留给 shader authoring 的程序描述；实际运行时管线由 invocation 与 template 合同构建。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderProgramAsset {
    pub source_uri: String,
    pub entry_point: String,
    pub domain: MaterialDomain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderGraphAsset {
    pub name: String,
    pub output_domain: MaterialDomain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterialGraphAsset {
    pub name: String,
    pub output_domain: MaterialDomain,
}

/// Shader authoring 维度的标识；使用前应先与核心渲染的 ShaderVariantKey 缓存合同区分。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderVariantKey {
    pub shader_id: ResourceId,
    pub domain: MaterialDomain,
    pub keywords: Vec<String>,
}
