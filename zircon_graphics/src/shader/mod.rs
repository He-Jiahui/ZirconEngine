use zircon_resource::ResourceId;

use crate::material::MaterialDomain;

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderVariantKey {
    pub shader_id: ResourceId,
    pub domain: MaterialDomain,
    pub keywords: Vec<String>,
}
