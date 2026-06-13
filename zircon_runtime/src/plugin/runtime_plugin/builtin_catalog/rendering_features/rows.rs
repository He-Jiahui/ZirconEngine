pub(super) struct RenderingFeatureRow {
    pub id_suffix: &'static str,
    pub display_name: &'static str,
    pub enabled_by_default: bool,
    pub extra_dependencies: &'static [RenderingFeatureDependencyRow],
}

pub(super) struct RenderingFeatureDependencyRow {
    pub provider_plugin_id: &'static str,
    pub capability: &'static str,
}

const VFX_GRAPH_DEPENDENCIES: &[RenderingFeatureDependencyRow] = &[
    RenderingFeatureDependencyRow {
        provider_plugin_id: "particles",
        capability: "runtime.plugin.particles",
    },
    RenderingFeatureDependencyRow {
        provider_plugin_id: "rendering",
        capability: "runtime.feature.rendering.shader_graph",
    },
];

pub(super) const RENDERING_FEATURE_ROWS: &[RenderingFeatureRow] = &[
    RenderingFeatureRow {
        id_suffix: "post_process",
        display_name: "Post Process",
        enabled_by_default: true,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "ssao",
        display_name: "SSAO",
        enabled_by_default: true,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "contact_shadow",
        display_name: "Contact Shadow",
        enabled_by_default: false,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "decals",
        display_name: "Decals",
        enabled_by_default: false,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "reflection_probes",
        display_name: "Reflection Probes",
        enabled_by_default: true,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "baked_lighting",
        display_name: "Baked Lighting",
        enabled_by_default: true,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "ray_tracing_policy",
        display_name: "Ray Tracing Policy",
        enabled_by_default: false,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "shader_graph",
        display_name: "Shader Graph",
        enabled_by_default: false,
        extra_dependencies: &[],
    },
    RenderingFeatureRow {
        id_suffix: "vfx_graph",
        display_name: "VFX Graph",
        enabled_by_default: false,
        extra_dependencies: VFX_GRAPH_DEPENDENCIES,
    },
];
