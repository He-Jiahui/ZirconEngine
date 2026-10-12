/// 场景执行层从已编译渲染管线取得的功能权限，用于资源绑定、历史维护和效果执行。
/// 它描述管线能力；本帧设置和资源是否可用仍须在执行入口检查，默认值关闭全部功能。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SceneRuntimeFeatureFlags {
    pub(crate) deferred_lighting_enabled: bool,
    pub(crate) ssao_enabled: bool,
    pub(crate) contact_shadow_enabled: bool,
    pub(crate) clustered_lighting_enabled: bool,
    pub(crate) hybrid_global_illumination_enabled: bool,
    pub(crate) temporal_history_enabled: bool,
    pub(crate) bloom_enabled: bool,
    pub(crate) color_grading_enabled: bool,
    pub(crate) anti_alias_enabled: bool,
    pub(crate) reflection_probes_enabled: bool,
    pub(crate) baked_lighting_enabled: bool,
    pub(crate) sprite_rendering_enabled: bool,
    pub(crate) particle_rendering_enabled: bool,
    pub(crate) virtual_geometry_enabled: bool,
}
