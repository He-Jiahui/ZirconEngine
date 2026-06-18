mod shading_models;

pub(crate) use shading_models::builtin_shading_model_registry;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialDomain {
    Surface,
    PostProcess,
    DebugOverlay,
    LightFunction,
}
