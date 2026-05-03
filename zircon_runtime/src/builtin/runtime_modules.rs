use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::asset::AssetImporterRegistry;
use crate::engine_module::EngineModule;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::{
    RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
#[cfg(feature = "plugin-ui")]
use crate::ui;
use crate::{asset, foundation, graphics, input, platform, scene, script};
use crate::{plugin::ProjectPluginManifest, plugin::ProjectPluginSelection};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTargetMode {
    ClientRuntime,
    ServerRuntime,
    EditorHost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePluginId {
    Ui,
    Physics,
    Sound,
    Texture,
    Net,
    Navigation,
    Particles,
    Animation,
    Terrain,
    Tilemap2d,
    PrefabTools,
    GltfImporter,
    ObjImporter,
    TextureImporter,
    AudioImporter,
    ShaderWgslImporter,
    UiDocumentImporter,
    Rendering,
    VirtualGeometry,
    HybridGi,
}

impl RuntimePluginId {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Physics => "physics",
            Self::Sound => "sound",
            Self::Texture => "texture",
            Self::Net => "net",
            Self::Navigation => "navigation",
            Self::Particles => "particles",
            Self::Animation => "animation",
            Self::Terrain => "terrain",
            Self::Tilemap2d => "tilemap_2d",
            Self::PrefabTools => "prefab_tools",
            Self::GltfImporter => "gltf_importer",
            Self::ObjImporter => "obj_importer",
            Self::TextureImporter => "texture_importer",
            Self::AudioImporter => "audio_importer",
            Self::ShaderWgslImporter => "shader_wgsl_importer",
            Self::UiDocumentImporter => "ui_document_importer",
            Self::Rendering => "rendering",
            Self::VirtualGeometry => "virtual_geometry",
            Self::HybridGi => "hybrid_gi",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Ui => "Ui",
            Self::Physics => "Physics",
            Self::Sound => "Sound",
            Self::Texture => "Texture",
            Self::Net => "Net",
            Self::Navigation => "Navigation",
            Self::Particles => "Particles",
            Self::Animation => "Animation",
            Self::Terrain => "Terrain",
            Self::Tilemap2d => "Tilemap2d",
            Self::PrefabTools => "PrefabTools",
            Self::GltfImporter => "GltfImporter",
            Self::ObjImporter => "ObjImporter",
            Self::TextureImporter => "TextureImporter",
            Self::AudioImporter => "AudioImporter",
            Self::ShaderWgslImporter => "ShaderWgslImporter",
            Self::UiDocumentImporter => "UiDocumentImporter",
            Self::Rendering => "Rendering",
            Self::VirtualGeometry => "VirtualGeometry",
            Self::HybridGi => "HybridGi",
        }
    }

    pub fn parse_key(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ui" => Some(Self::Ui),
            "physics" => Some(Self::Physics),
            "sound" | "audio" => Some(Self::Sound),
            "texture" => Some(Self::Texture),
            "net" | "network" => Some(Self::Net),
            "navigation" | "nav" => Some(Self::Navigation),
            "particles" => Some(Self::Particles),
            "animation" => Some(Self::Animation),
            "terrain" => Some(Self::Terrain),
            "tilemap_2d" | "tilemap" | "tile_map_2d" => Some(Self::Tilemap2d),
            "prefab_tools" | "prefab" | "prefabs" => Some(Self::PrefabTools),
            "gltf_importer" | "gltf" | "glb_importer" => Some(Self::GltfImporter),
            "obj_importer" | "obj" | "wavefront_obj" => Some(Self::ObjImporter),
            "texture_importer" | "image_importer" => Some(Self::TextureImporter),
            "audio_importer" | "sound_importer" | "wav_importer" => Some(Self::AudioImporter),
            "shader_wgsl_importer" | "wgsl_importer" => Some(Self::ShaderWgslImporter),
            "ui_document_importer" | "ui_importer" | "ui_asset_importer" => {
                Some(Self::UiDocumentImporter)
            }
            "rendering" | "renderer" | "graphics" => Some(Self::Rendering),
            "vg" | "virtual_geometry" => Some(Self::VirtualGeometry),
            "gi" | "hybrid_gi" => Some(Self::HybridGi),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeModuleLoadReport {
    pub modules: Vec<Arc<dyn EngineModule>>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    required_missing: Vec<RuntimeRequiredPluginMissing>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRequiredPluginMissing {
    pub id: RuntimePluginId,
    pub reason: String,
}

impl RuntimeModuleLoadReport {
    fn new(modules: Vec<Arc<dyn EngineModule>>) -> Self {
        Self {
            modules,
            warnings: Vec::new(),
            errors: Vec::new(),
            required_missing: Vec::new(),
        }
    }

    pub fn required_missing(&self) -> &[RuntimeRequiredPluginMissing] {
        &self.required_missing
    }

    pub fn required_missing_summary(&self) -> String {
        self.required_missing
            .iter()
            .map(|missing| {
                format!(
                    "required runtime plugin {} is unavailable: {}",
                    missing.id.label(),
                    missing.reason
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }
}

pub fn builtin_runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    runtime_modules_for_target(RuntimeTargetMode::ClientRuntime, None).modules
}

pub fn runtime_core_modules() -> Vec<Arc<dyn EngineModule>> {
    runtime_core_modules_for_target(RuntimeTargetMode::ClientRuntime)
}

fn runtime_core_modules_for_target(target: RuntimeTargetMode) -> Vec<Arc<dyn EngineModule>> {
    runtime_core_modules_for_target_with_render_features(
        target,
        &AssetImporterRegistry::default(),
        &[],
        &[],
        &[],
        &[],
    )
}

fn runtime_core_modules_for_target_with_render_features(
    target: RuntimeTargetMode,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> Vec<Arc<dyn EngineModule>> {
    let mut modules: Vec<Arc<dyn EngineModule>> = vec![
        Arc::new(foundation::FoundationModule),
        Arc::new(platform::PlatformModule),
        Arc::new(input::InputModule),
        Arc::new(asset::AssetModule::with_asset_importers(
            asset_importers.clone(),
        )),
        Arc::new(scene::SceneModule),
    ];
    if target != RuntimeTargetMode::ServerRuntime {
        modules.push(Arc::new(
            graphics::GraphicsModule::with_render_extensions_and_runtime_providers(
                render_features.iter().cloned(),
                render_pass_executors.iter().cloned(),
                hybrid_gi_runtime_providers.iter().cloned(),
                virtual_geometry_runtime_providers.iter().cloned(),
            ),
        ));
    }
    modules.push(Arc::new(script::ScriptModule));
    modules
}

pub fn runtime_modules_for_target(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_target_with_linked_plugins(
        target,
        manifest_override,
        std::iter::empty::<String>(),
    )
}

pub fn runtime_modules_for_target_with_linked_plugins(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
) -> RuntimeModuleLoadReport {
    runtime_modules_for_target_with_linked_plugins_and_render_features(
        target,
        manifest_override,
        linked_plugin_ids,
        &AssetImporterRegistry::default(),
        &[],
        &[],
        &[],
        &[],
    )
}

pub fn runtime_modules_for_target_with_plugin_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations
        .into_iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
        })
        .collect::<Vec<_>>();
    let linked_plugin_ids = registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();
    let render_features = registrations
        .iter()
        .flat_map(|registration| registration.extensions.render_features().iter().cloned())
        .collect::<Vec<_>>();
    let render_pass_executors = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .render_pass_executors()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let hybrid_gi_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .hybrid_gi_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let virtual_geometry_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .virtual_geometry_runtime_providers()
                .iter()
                .cloned()
        })
        .collect::<Vec<_>>();
    let (asset_importers, asset_importer_errors) = asset_importers_from_extension_registries(
        registrations
            .iter()
            .map(|registration| &registration.extensions),
    );
    let mut report = runtime_modules_for_target_with_linked_plugins_and_render_features(
        target,
        manifest_override,
        linked_plugin_ids,
        &asset_importers,
        &render_features,
        &render_pass_executors,
        &hybrid_gi_runtime_providers,
        &virtual_geometry_runtime_providers,
    );
    report.errors.extend(asset_importer_errors);
    report
}

pub fn runtime_modules_for_target_with_plugin_and_feature_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let catalog = RuntimePluginCatalog::from_registration_reports(
        registrations.clone(),
        feature_registrations.clone(),
    );
    let active_registrations = registrations
        .iter()
        .filter(|registration| {
            registration.project_selection.enabled
                && registration.project_selection.supports_target(target)
        })
        .collect::<Vec<_>>();
    let feature_report = catalog.feature_dependency_report(&manifest, target);
    let active_feature_registrations = feature_registrations
        .iter()
        .filter(|registration| {
            feature_report
                .available_features
                .iter()
                .any(|id| id == &registration.manifest.id)
        })
        .collect::<Vec<_>>();
    let linked_plugin_ids = active_registrations
        .iter()
        .map(|registration| registration.package_manifest.id.as_str())
        .collect::<Vec<_>>();
    let render_features = active_registrations
        .iter()
        .flat_map(|registration| registration.extensions.render_features().iter().cloned())
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| registration.extensions.render_features().iter().cloned()),
        )
        .collect::<Vec<_>>();
    let render_pass_executors = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .render_pass_executors()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .render_pass_executors()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let hybrid_gi_runtime_providers = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .hybrid_gi_runtime_providers()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .hybrid_gi_runtime_providers()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let virtual_geometry_runtime_providers = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .virtual_geometry_runtime_providers()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .virtual_geometry_runtime_providers()
                        .iter()
                        .cloned()
                }),
        )
        .collect::<Vec<_>>();
    let (asset_importers, asset_importer_errors) = asset_importers_from_extension_registries(
        active_registrations
            .iter()
            .map(|registration| &registration.extensions)
            .chain(
                active_feature_registrations
                    .iter()
                    .map(|registration| &registration.extensions),
            ),
    );
    let mut report = runtime_modules_for_target_with_linked_plugins_and_render_features(
        target,
        Some(&manifest),
        linked_plugin_ids,
        &asset_importers,
        &render_features,
        &render_pass_executors,
        &hybrid_gi_runtime_providers,
        &virtual_geometry_runtime_providers,
    );
    for blocked in feature_report.blocked_features {
        if blocked.required {
            report.errors.push(blocked.to_diagnostic());
        } else {
            report.warnings.push(blocked.to_diagnostic());
        }
    }
    report.errors.extend(feature_report.diagnostics);
    report.errors.extend(asset_importer_errors);
    report
}

fn runtime_modules_for_target_with_linked_plugins_and_render_features(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> RuntimeModuleLoadReport {
    let linked_plugin_ids = linked_plugin_ids
        .into_iter()
        .map(|id| id.as_ref().to_string())
        .collect::<HashSet<_>>();
    let mut report =
        RuntimeModuleLoadReport::new(runtime_core_modules_for_target_with_render_features(
            target,
            asset_importers,
            render_features,
            render_pass_executors,
            hybrid_gi_runtime_providers,
            virtual_geometry_runtime_providers,
        ));
    let manifest = manifest_with_mode_baseline(target, manifest_override);

    for selection in manifest.enabled_for_target(target) {
        let Some(runtime_id) = selection.runtime_id() else {
            let reason = format!("plugin {} has no known runtime id", selection.id);
            if selection.required {
                report.errors.push(format!(
                    "required runtime plugin {} is unavailable: {}",
                    selection.id, reason
                ));
            } else {
                report.warnings.push(reason);
            }
            continue;
        };
        if builtin_runtime_domain_is_available(runtime_id) {
            report
                .warnings
                .push(builtin_runtime_domain_message(runtime_id.key()));
            continue;
        }
        if linked_plugin_is_available(selection, runtime_id, &linked_plugin_ids) {
            continue;
        }
        let warning_start = report.warnings.len();
        if let Some(module) = module_for_plugin(runtime_id, &mut report.warnings) {
            report.modules.push(module);
            continue;
        }
        if selection.required {
            let reason = report.warnings[warning_start..]
                .last()
                .cloned()
                .unwrap_or_else(|| format!("plugin {} is unavailable", runtime_id.label()));
            let message = format!(
                "required runtime plugin {} is unavailable: {}",
                runtime_id.label(),
                reason.clone()
            );
            report.required_missing.push(RuntimeRequiredPluginMissing {
                id: runtime_id,
                reason,
            });
            report.errors.push(message);
        }
    }
    report
}

fn asset_importers_from_extension_registries<'a>(
    registries: impl IntoIterator<Item = &'a crate::plugin::RuntimeExtensionRegistry>,
) -> (AssetImporterRegistry, Vec<String>) {
    let mut asset_importers = AssetImporterRegistry::default();
    let mut errors = Vec::new();
    for registry in registries {
        for importer in registry.asset_importers().importers() {
            if let Err(error) = asset_importers.register_arc(importer) {
                errors.push(format!("asset importer registration failed: {error}"));
            }
        }
    }
    (asset_importers, errors)
}

pub fn manifest_with_mode_baseline(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
) -> ProjectPluginManifest {
    let mut manifest = default_manifest_for_target(target);
    if let Some(override_manifest) = manifest_override {
        for selection in &override_manifest.selections {
            manifest.set_enabled(selection.clone());
        }
    }
    manifest
}

fn linked_plugin_is_available(
    selection: &ProjectPluginSelection,
    runtime_id: RuntimePluginId,
    linked_plugin_ids: &HashSet<String>,
) -> bool {
    linked_plugin_ids.contains(&selection.id) || linked_plugin_ids.contains(runtime_id.key())
}

fn builtin_runtime_domain_is_available(id: RuntimePluginId) -> bool {
    let _ = id;
    false
}

fn builtin_runtime_domain_message(id: &str) -> String {
    format!("runtime plugin {id} is provided by the built-in runtime domain")
}

pub fn default_manifest_for_target(target: RuntimeTargetMode) -> ProjectPluginManifest {
    let selections = match target {
        RuntimeTargetMode::ClientRuntime => default_ui_plugin_selection(),
        RuntimeTargetMode::ServerRuntime => Vec::new(),
        RuntimeTargetMode::EditorHost => default_ui_plugin_selection(),
    };
    ProjectPluginManifest { selections }
}

fn default_ui_plugin_selection() -> Vec<ProjectPluginSelection> {
    #[cfg(feature = "plugin-ui")]
    {
        vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Ui,
            true,
            true,
        )]
    }
    #[cfg(not(feature = "plugin-ui"))]
    {
        Vec::new()
    }
}

fn module_for_plugin(
    id: RuntimePluginId,
    warnings: &mut Vec<String>,
) -> Option<Arc<dyn EngineModule>> {
    match id {
        RuntimePluginId::Ui => {
            #[cfg(feature = "plugin-ui")]
            {
                return Some(Arc::new(ui::UiModule));
            }
            #[cfg(not(feature = "plugin-ui"))]
            {
                warnings.push("plugin-ui feature is disabled".to_string());
                None
            }
        }
        RuntimePluginId::Physics => {
            warnings.push(externalized_runtime_plugin_message("physics"));
            None
        }
        RuntimePluginId::Sound => {
            warnings.push(externalized_runtime_plugin_message("sound"));
            None
        }
        RuntimePluginId::Texture => {
            warnings.push(externalized_runtime_plugin_message("texture"));
            None
        }
        RuntimePluginId::Net => {
            warnings.push(externalized_runtime_plugin_message("net"));
            None
        }
        RuntimePluginId::Navigation => {
            warnings.push(externalized_runtime_plugin_message("navigation"));
            None
        }
        RuntimePluginId::Particles => {
            warnings.push(externalized_runtime_plugin_message("particles"));
            None
        }
        RuntimePluginId::Animation => {
            warnings.push(externalized_runtime_plugin_message("animation"));
            None
        }
        RuntimePluginId::Terrain => {
            warnings.push(externalized_runtime_plugin_message("terrain"));
            None
        }
        RuntimePluginId::Tilemap2d => {
            warnings.push(externalized_runtime_plugin_message("tilemap_2d"));
            None
        }
        RuntimePluginId::PrefabTools => {
            warnings.push(externalized_runtime_plugin_message("prefab_tools"));
            None
        }
        RuntimePluginId::GltfImporter => {
            warnings.push(externalized_runtime_plugin_message("gltf_importer"));
            None
        }
        RuntimePluginId::ObjImporter => {
            warnings.push(externalized_runtime_plugin_message("obj_importer"));
            None
        }
        RuntimePluginId::TextureImporter => {
            warnings.push(externalized_runtime_plugin_message("texture_importer"));
            None
        }
        RuntimePluginId::AudioImporter => {
            warnings.push(externalized_runtime_plugin_message("audio_importer"));
            None
        }
        RuntimePluginId::ShaderWgslImporter => {
            warnings.push(externalized_runtime_plugin_message("shader_wgsl_importer"));
            None
        }
        RuntimePluginId::UiDocumentImporter => {
            warnings.push(externalized_runtime_plugin_message("ui_document_importer"));
            None
        }
        RuntimePluginId::Rendering => {
            warnings.push(externalized_runtime_plugin_message("rendering"));
            None
        }
        RuntimePluginId::VirtualGeometry => {
            warnings.push(externalized_runtime_plugin_message("virtual_geometry"));
            None
        }
        RuntimePluginId::HybridGi => {
            warnings.push(externalized_runtime_plugin_message("hybrid_gi"));
            None
        }
    }
}

fn externalized_runtime_plugin_message(plugin_id: &str) -> String {
    format!("runtime implementation is externalized to zircon_plugins/{plugin_id}")
}

#[cfg(test)]
mod tests {
    use super::{
        default_manifest_for_target, manifest_with_mode_baseline, RuntimePluginId,
        RuntimeTargetMode,
    };
    use crate::{plugin::ProjectPluginManifest, plugin::ProjectPluginSelection};

    #[test]
    fn default_server_manifest_avoids_ui() {
        let manifest = default_manifest_for_target(RuntimeTargetMode::ServerRuntime);
        assert!(manifest
            .selections
            .iter()
            .all(|selection| selection.id != RuntimePluginId::Ui.key()));
    }

    #[test]
    fn project_manifest_overlays_mode_baseline() {
        let manifest = manifest_with_mode_baseline(
            RuntimeTargetMode::ClientRuntime,
            Some(&ProjectPluginManifest {
                selections: vec![ProjectPluginSelection::runtime_plugin(
                    RuntimePluginId::Physics,
                    true,
                    false,
                )],
            }),
        );

        #[cfg(feature = "plugin-ui")]
        assert!(manifest
            .selections
            .iter()
            .any(|selection| selection.id == RuntimePluginId::Ui.key()));
        #[cfg(not(feature = "plugin-ui"))]
        assert!(manifest
            .selections
            .iter()
            .all(|selection| selection.id != RuntimePluginId::Ui.key()));
        assert!(manifest
            .selections
            .iter()
            .any(|selection| selection.id == RuntimePluginId::Physics.key()));
    }

    #[test]
    fn project_manifest_can_disable_mode_baseline_plugin() {
        let manifest = manifest_with_mode_baseline(
            RuntimeTargetMode::ClientRuntime,
            Some(&ProjectPluginManifest {
                selections: vec![ProjectPluginSelection::runtime_plugin(
                    RuntimePluginId::Ui,
                    false,
                    false,
                )],
            }),
        );

        assert!(manifest
            .enabled_for_target(RuntimeTargetMode::ClientRuntime)
            .all(|selection| selection.id != RuntimePluginId::Ui.key()));
    }
}
