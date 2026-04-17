#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneRuntimeFeatureFlags {
    pub(crate) deferred_lighting_enabled: bool,
    pub(crate) ssao_enabled: bool,
    pub(crate) clustered_lighting_enabled: bool,
    pub(crate) hybrid_global_illumination_enabled: bool,
    pub(crate) history_resolve_enabled: bool,
    pub(crate) bloom_enabled: bool,
    pub(crate) color_grading_enabled: bool,
    pub(crate) reflection_probes_enabled: bool,
    pub(crate) baked_lighting_enabled: bool,
    pub(crate) particle_rendering_enabled: bool,
    pub(crate) virtual_geometry_enabled: bool,
}
