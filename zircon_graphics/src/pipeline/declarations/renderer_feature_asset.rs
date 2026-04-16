use crate::feature::BuiltinRenderFeature;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RendererFeatureAsset {
    pub feature: BuiltinRenderFeature,
    pub enabled: bool,
}
