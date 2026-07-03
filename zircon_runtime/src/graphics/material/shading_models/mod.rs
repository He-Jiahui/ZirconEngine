mod builtins;
mod include_sources;
mod registry;

pub(crate) use builtins::{
    builtin_shading_model_registry, shading_model_registry_with_plugin_descriptors,
};
pub(crate) use include_sources::{
    ShadingModelIncludeSource, ShadingModelIncludeSourceError, ShadingModelIncludeSourceSet,
};
pub(crate) use registry::ShadingModelRegistry;
