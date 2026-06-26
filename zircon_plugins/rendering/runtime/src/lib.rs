pub const PLUGIN_ID: &str = "rendering";
pub const RENDERING_MODULE_NAME: &str = "RenderingPluginModule";

mod capability;
mod plugin;

pub use capability::{RENDERING_RUNTIME_CAPABILITY, RUNTIME_CAPABILITIES};
pub use plugin::{
    package_manifest, plugin_registration, runtime_capabilities, runtime_plugin,
    runtime_plugin_descriptor, RenderingRuntimePlugin, RENDERING_DIST_CRATE_NAME,
    RENDERING_DIST_RUNTIME_ENTRY,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderingFeatureKind {
    PostProcess,
    Ssao,
    ContactShadow,
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
    RenderingFeatureKind::ContactShadow,
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
            Self::ContactShadow => "contact_shadow",
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
            Self::ContactShadow => "Contact Shadow",
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

    pub fn editor_capability(self) -> String {
        format!("editor.feature.rendering.{}", self.id_suffix())
    }

    pub fn runtime_crate(self) -> String {
        format!("zircon_plugin_rendering_{}_runtime", self.id_suffix())
    }

    pub fn editor_crate(self) -> String {
        format!("zircon_plugin_rendering_{}_editor", self.id_suffix())
    }
}

pub fn module_descriptor() -> zircon_runtime::core::ModuleDescriptor {
    zircon_runtime::core::ModuleDescriptor::new(
        RENDERING_MODULE_NAME,
        "Rendering umbrella plugin and feature owner",
    )
}

pub fn feature_manifest(
    feature: RenderingFeatureKind,
) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    let feature_id = feature.feature_id();
    let capability = feature.runtime_capability();
    let editor_capability = feature.editor_capability();
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
            zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([capability]),
    )
    .with_editor_module(
        zircon_runtime::plugin::PluginModuleManifest::editor(
            format!("{feature_id}.editor"),
            feature.editor_crate(),
        )
        .with_capabilities([editor_capability]),
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendering_descriptor_declares_nine_owner_features() {
        let descriptor = runtime_plugin_descriptor();

        assert_eq!(descriptor.category(), "rendering");
        assert_eq!(
            descriptor.maturity(),
            zircon_runtime::plugin::PluginMaturity::Stable
        );
        assert_eq!(descriptor.optional_features().len(), 9);
        assert!(
            descriptor
                .optional_features()
                .iter()
                .any(|feature| feature.id == "rendering.contact_shadow"
                    && !feature.enabled_by_default)
        );
        assert_eq!(
            descriptor
                .optional_features()
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
    fn rendering_feature_manifests_declare_editor_capabilities() {
        for feature_kind in RENDERING_FEATURES {
            let manifest = feature_manifest(*feature_kind);
            let editor_capability = feature_kind.editor_capability();
            let editor_module = manifest
                .modules
                .iter()
                .find(|module| module.kind == zircon_runtime::plugin::PluginModuleKind::Editor)
                .expect("rendering feature editor module");

            assert!(
                editor_module.capabilities.contains(&editor_capability),
                "{} editor module should project {editor_capability}",
                manifest.id
            );
        }
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

    #[test]
    fn rendering_package_manifest_declares_dist_contract() {
        let manifest = package_manifest();

        assert!(manifest
            .default_packaging
            .contains(&zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic));

        let distribution = manifest
            .distribution
            .as_ref()
            .expect("rendering distribution manifest");
        assert_eq!(distribution.forms, vec!["dist".to_string()]);
        assert_eq!(
            distribution.default_packaging,
            vec![zircon_runtime::plugin::ExportPackagingStrategy::NativeDynamic]
        );
        assert_eq!(distribution.abi_version, Some(3));
        assert_eq!(distribution.engine_compat, ">=0.1, <0.2");
        assert_eq!(distribution.dist_crate, RENDERING_DIST_CRATE_NAME);
        assert_eq!(
            distribution.descriptor_symbol,
            "zircon_native_plugin_descriptor_v3"
        );
        assert_eq!(distribution.runtime_entry, RENDERING_DIST_RUNTIME_ENTRY);

        let native_module = manifest
            .modules
            .iter()
            .find(|module| module.name == "rendering.dist")
            .expect("rendering native dist module");
        assert_eq!(
            native_module.kind,
            zircon_runtime::plugin::PluginModuleKind::Native
        );
        assert_eq!(native_module.crate_name, RENDERING_DIST_CRATE_NAME);
        assert_eq!(
            native_module.target_modes,
            vec![
                zircon_runtime::builtin::RuntimeTargetMode::ClientRuntime,
                zircon_runtime::builtin::RuntimeTargetMode::EditorHost,
            ]
        );
        for capability in RUNTIME_CAPABILITIES {
            assert!(native_module.capabilities.contains(&capability.to_string()));
        }
    }
}
