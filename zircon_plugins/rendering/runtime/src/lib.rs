pub const PLUGIN_ID: &str = "rendering";
pub const RENDERING_MODULE_NAME: &str = "RenderingPluginModule";
pub const RENDERING_RUNTIME_CAPABILITY: &str = "runtime.plugin.rendering";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingFeatureKind {
    PostProcess,
    Ssao,
    Decals,
    ReflectionProbes,
    BakedLighting,
    RayTracingPolicy,
    ShaderGraph,
    VfxGraph,
}

pub const RENDERING_FEATURES: &[RenderingFeatureKind] = &[
    RenderingFeatureKind::PostProcess,
    RenderingFeatureKind::Ssao,
    RenderingFeatureKind::Decals,
    RenderingFeatureKind::ReflectionProbes,
    RenderingFeatureKind::BakedLighting,
    RenderingFeatureKind::RayTracingPolicy,
    RenderingFeatureKind::ShaderGraph,
    RenderingFeatureKind::VfxGraph,
];

impl RenderingFeatureKind {
    pub const fn id_suffix(self) -> &'static str {
        match self {
            Self::PostProcess => "post_process",
            Self::Ssao => "ssao",
            Self::Decals => "decals",
            Self::ReflectionProbes => "reflection_probes",
            Self::BakedLighting => "baked_lighting",
            Self::RayTracingPolicy => "ray_tracing_policy",
            Self::ShaderGraph => "shader_graph",
            Self::VfxGraph => "vfx_graph",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::PostProcess => "Post Process",
            Self::Ssao => "SSAO",
            Self::Decals => "Decals",
            Self::ReflectionProbes => "Reflection Probes",
            Self::BakedLighting => "Baked Lighting",
            Self::RayTracingPolicy => "Ray Tracing Policy",
            Self::ShaderGraph => "Shader Graph",
            Self::VfxGraph => "VFX Graph",
        }
    }

    pub const fn enabled_by_default(self) -> bool {
        matches!(
            self,
            Self::PostProcess | Self::Ssao | Self::ReflectionProbes | Self::BakedLighting
        )
    }

    pub fn feature_id(self) -> String {
        format!("rendering.{}", self.id_suffix())
    }

    pub fn runtime_capability(self) -> String {
        format!("runtime.feature.rendering.{}", self.id_suffix())
    }

    pub fn runtime_crate(self) -> String {
        format!("zircon_plugin_rendering_{}_runtime", self.id_suffix())
    }

    pub fn editor_crate(self) -> String {
        format!("zircon_plugin_rendering_{}_editor", self.id_suffix())
    }
}

#[derive(Clone, Debug)]
pub struct RenderingRuntimePlugin {
    descriptor: zircon_runtime::plugin::RuntimePluginDescriptor,
}

impl RenderingRuntimePlugin {
    pub fn new() -> Self {
        Self {
            descriptor: runtime_plugin_descriptor(),
        }
    }
}

impl zircon_runtime::plugin::RuntimePlugin for RenderingRuntimePlugin {
    fn descriptor(&self) -> &zircon_runtime::plugin::RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register_runtime_extensions(
        &self,
        registry: &mut zircon_runtime::plugin::RuntimeExtensionRegistry,
    ) -> Result<(), zircon_runtime::plugin::RuntimeExtensionRegistryError> {
        registry.register_module(module_descriptor())
    }
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        RENDERING_MODULE_NAME,
        "Rendering umbrella plugin and feature owner",
    )
}

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    let mut descriptor = zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Rendering",
        zircon_runtime::RuntimePluginId::Rendering,
        "zircon_plugin_rendering_runtime",
    )
    .with_category("rendering")
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_capability(RENDERING_RUNTIME_CAPABILITY);

    for feature in RENDERING_FEATURES {
        descriptor = descriptor.with_optional_feature(feature_manifest(*feature));
    }
    descriptor
}

pub fn feature_manifest(
    feature: RenderingFeatureKind,
) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    let feature_id = feature.feature_id();
    let capability = feature.runtime_capability();
    let mut manifest = zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        feature_id.clone(),
        feature.display_name(),
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        RENDERING_RUNTIME_CAPABILITY,
    ))
    .with_capability(capability.clone())
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            format!("{feature_id}.runtime"),
            feature.runtime_crate(),
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([capability]),
    )
    .with_editor_module(zircon_runtime::plugin::PluginModuleManifest::editor(
        format!("{feature_id}.editor"),
        feature.editor_crate(),
    ))
    .enabled_by_default(feature.enabled_by_default());

    if feature == RenderingFeatureKind::VfxGraph {
        manifest = manifest
            .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
                "particles",
                "runtime.plugin.particles",
            ))
            .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
                PLUGIN_ID,
                RenderingFeatureKind::ShaderGraph.runtime_capability(),
            ));
    }

    manifest
}

pub fn runtime_plugin() -> RenderingRuntimePlugin {
    RenderingRuntimePlugin::new()
}

pub fn package_manifest() -> zircon_runtime::plugin::PluginPackageManifest {
    zircon_runtime::plugin::RuntimePlugin::package_manifest(&runtime_plugin())
}

pub fn runtime_selection() -> zircon_runtime::plugin::ProjectPluginSelection {
    zircon_runtime::plugin::RuntimePlugin::project_selection(&runtime_plugin())
}

pub fn plugin_registration() -> zircon_runtime::plugin::RuntimePluginRegistrationReport {
    zircon_runtime::plugin::RuntimePluginRegistrationReport::from_plugin(&runtime_plugin())
}

pub fn runtime_capabilities() -> &'static [&'static str] {
    &[RENDERING_RUNTIME_CAPABILITY]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendering_descriptor_declares_eight_owner_features() {
        let descriptor = runtime_plugin_descriptor();

        assert_eq!(descriptor.category, "rendering");
        assert_eq!(descriptor.optional_features.len(), 8);
        assert_eq!(
            descriptor
                .optional_features
                .iter()
                .filter(|feature| feature.enabled_by_default)
                .map(|feature| feature.id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "rendering.post_process",
                "rendering.ssao",
                "rendering.reflection_probes",
                "rendering.baked_lighting",
            ]
        );
    }

    #[test]
    fn vfx_graph_requires_particles_and_shader_graph() {
        let manifest = feature_manifest(RenderingFeatureKind::VfxGraph);

        assert!(manifest.dependencies.iter().any(|dependency| {
            dependency.plugin_id == "particles"
                && dependency.capability == "runtime.plugin.particles"
        }));
        assert!(manifest.dependencies.iter().any(|dependency| {
            dependency.plugin_id == PLUGIN_ID
                && dependency.capability == "runtime.feature.rendering.shader_graph"
        }));
    }
}
