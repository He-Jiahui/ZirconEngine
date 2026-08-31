mod shading_models;

pub(crate) use shading_models::{
    ShadingModelIncludeSource, ShadingModelIncludeSourceError, ShadingModelIncludeSourceSet,
    ShadingModelRegistry, builtin_shading_model_registry,
    shading_model_registry_with_plugin_descriptors,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialDomain {
    Surface,
    PostProcess,
    DebugOverlay,
    LightFunction,
}
