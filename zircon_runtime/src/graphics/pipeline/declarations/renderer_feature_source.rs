use crate::graphics::feature::BuiltinRenderFeature;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RendererFeatureSource {
    Builtin(BuiltinRenderFeature),
    Plugin(String),
}

impl RendererFeatureSource {
    pub fn builtin(feature: BuiltinRenderFeature) -> Self {
        Self::Builtin(feature)
    }

    pub fn plugin(name: impl Into<String>) -> Self {
        Self::Plugin(name.into())
    }

    pub fn builtin_feature(&self) -> Option<BuiltinRenderFeature> {
        match self {
            Self::Builtin(feature) => Some(*feature),
            Self::Plugin(_) => None,
        }
    }

    pub fn feature_name(&self) -> String {
        match self {
            Self::Builtin(feature) => feature.descriptor().name,
            Self::Plugin(name) => name.clone(),
        }
    }
}
