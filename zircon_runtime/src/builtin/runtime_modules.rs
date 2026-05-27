use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::asset::AssetImporterRegistry;
use crate::engine_module::EngineModule;
use crate::graphics::{
    HybridGiRuntimeProviderRegistration, RenderFeatureDescriptor, RenderPassExecutorRegistration,
    RuntimePrepareCollectorRegistration, SolariRuntimeProviderRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::{
    RuntimePluginAvailabilityReport, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
    RuntimeProfileDescriptor, RuntimeProfileId,
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
    Solari,
    ZrVmLanguage,
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
            Self::Solari => "solari",
            Self::ZrVmLanguage => "zr_vm_language",
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
            Self::Solari => "Solari",
            Self::ZrVmLanguage => "ZrVM Language",
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
            "solari" => Some(Self::Solari),
            "zr_vm_language" | "zr_vm" | "zrvmlanguage" => Some(Self::ZrVmLanguage),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeModuleLoadReport {
    pub modules: Vec<Arc<dyn EngineModule>>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
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
            runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
            required_missing: Vec::new(),
        }
    }

    fn with_runtime_plugin_availability(
        mut self,
        runtime_plugin_availability: RuntimePluginAvailabilityReport,
    ) -> Self {
        self.runtime_plugin_availability = runtime_plugin_availability;
        self
    }

    pub fn required_missing(&self) -> &[RuntimeRequiredPluginMissing] {
        &self.required_missing
    }

    pub fn effective_required_missing(&self) -> Vec<RuntimeRequiredPluginMissing> {
        let mut missing = self.required_missing.clone();
        for entry in &self.runtime_plugin_availability.missing_required {
            let structured_missing = RuntimeRequiredPluginMissing {
                id: entry.runtime_id,
                reason: entry.reason.clone(),
            };
            if !missing
                .iter()
                .any(|existing| existing.id == structured_missing.id)
            {
                missing.push(structured_missing);
            }
        }
        missing
    }

    pub fn required_missing_summary(&self) -> String {
        self.effective_required_missing()
            .into_iter()
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

    pub fn effective_errors(&self) -> Vec<String> {
        let mut errors = self.errors.clone();
        for missing in self.effective_required_missing() {
            let diagnostic = format!(
                "required runtime plugin {} is unavailable: {}",
                missing.id.label(),
                missing.reason
            );
            if !errors.iter().any(|existing| existing == &diagnostic) {
                errors.push(diagnostic);
            }
        }
        errors
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.effective_errors().is_empty()
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
        &[],
        &[],
    )
}

fn minimal_profile_runtime_modules() -> Vec<Arc<dyn EngineModule>> {
    vec![
        Arc::new(foundation::FoundationModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::TasksModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::TimeModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::FrameCountModule) as Arc<dyn EngineModule>,
        Arc::new(crate::core::modules::DiagnosticsCoreModule) as Arc<dyn EngineModule>,
    ]
}

fn runtime_core_modules_for_target_with_render_features(
    target: RuntimeTargetMode,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    runtime_prepare_collectors: &[RuntimePrepareCollectorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    solari_runtime_providers: &[SolariRuntimeProviderRegistration],
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
                runtime_prepare_collectors.iter().cloned(),
                hybrid_gi_runtime_providers.iter().cloned(),
                solari_runtime_providers.iter().cloned(),
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
    let runtime_prepare_collectors = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .runtime_prepare_collectors()
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
    let solari_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .solari_runtime_providers()
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
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    let mut report =
        runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
            target,
            &manifest,
            linked_plugin_ids,
            &asset_importers,
            &render_features,
            &render_pass_executors,
            &runtime_prepare_collectors,
            &hybrid_gi_runtime_providers,
            &solari_runtime_providers,
            &virtual_geometry_runtime_providers,
        );
    report.errors.extend(asset_importer_errors);
    report.runtime_plugin_availability =
        target_manifest_availability_for_registration_reports(target, &manifest, registrations);
    report
}

pub fn runtime_modules_for_runtime_profile(
    profile_id: RuntimeProfileId,
) -> RuntimeModuleLoadReport {
    if profile_id == RuntimeProfileId::Minimal {
        let profile = RuntimeProfileDescriptor::for_id(profile_id);
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_availability(&profile));
    }

    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    let manifest = profile.project_manifest();
    runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
        profile.target_mode,
        &manifest,
        std::iter::empty::<String>(),
        &AssetImporterRegistry::default(),
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
    )
    .with_runtime_plugin_availability(runtime_profile_availability(&profile))
}

pub fn runtime_modules_for_runtime_profile_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports(
        profile_id,
        &profile.project_manifest(),
        registrations,
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().collect::<Vec<_>>();
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    if profile_id == RuntimeProfileId::Minimal {
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_manifest_availability(
                &profile,
                manifest,
                registrations.iter().copied(),
            ));
    }

    runtime_modules_for_profile_manifest_with_plugin_registration_reports(
        &profile,
        profile.target_mode,
        manifest,
        registrations.iter().copied(),
    )
}

pub fn runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports<'a>(
    profile_id: RuntimeProfileId,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
        profile_id,
        &profile.project_manifest(),
        registrations,
        feature_registrations,
    )
}

pub fn runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports<
    'a,
>(
    profile_id: RuntimeProfileId,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
    feature_registrations: impl IntoIterator<Item = &'a RuntimePluginFeatureRegistrationReport>,
) -> RuntimeModuleLoadReport {
    let registrations = registrations.into_iter().cloned().collect::<Vec<_>>();
    let feature_registrations = feature_registrations
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let profile = RuntimeProfileDescriptor::for_id(profile_id);
    if profile_id == RuntimeProfileId::Minimal {
        return RuntimeModuleLoadReport::new(minimal_profile_runtime_modules())
            .with_runtime_plugin_availability(runtime_profile_manifest_availability(
                &profile,
                manifest,
                registrations.iter(),
            ));
    }

    let mut report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
        profile.target_mode,
        Some(manifest),
        registrations.iter(),
        feature_registrations.iter(),
    );
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(&profile, manifest, registrations.iter());
    report
}

fn runtime_modules_for_profile_manifest_with_plugin_registration_reports<'a>(
    profile: &RuntimeProfileDescriptor,
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
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
    let runtime_prepare_collectors = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .runtime_prepare_collectors()
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
    let solari_runtime_providers = registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .solari_runtime_providers()
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
    let mut report =
        runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
            target,
            manifest,
            linked_plugin_ids,
            &asset_importers,
            &render_features,
            &render_pass_executors,
            &runtime_prepare_collectors,
            &hybrid_gi_runtime_providers,
            &solari_runtime_providers,
            &virtual_geometry_runtime_providers,
        );
    report.errors.extend(asset_importer_errors);
    report.runtime_plugin_availability =
        runtime_profile_manifest_availability(profile, manifest, registrations.iter().copied());
    report
}

fn runtime_profile_availability(
    profile: &RuntimeProfileDescriptor,
) -> RuntimePluginAvailabilityReport {
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_with_providers(
        descriptors.iter(),
        std::iter::empty::<String>(),
        std::iter::empty::<String>(),
    )
}

fn runtime_profile_manifest_availability<'a>(
    profile: &RuntimeProfileDescriptor,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimePluginAvailabilityReport {
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_and_registration_reports(
        descriptors.iter(),
        manifest,
        registrations,
    )
}

fn target_manifest_availability<'a>(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    linked_plugin_ids: impl IntoIterator<Item = &'a String>,
) -> RuntimePluginAvailabilityReport {
    let profile = RuntimeProfileDescriptor::new(
        runtime_profile_id_for_target_availability(target),
        "target module selection",
        target,
    );
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_with_providers(
        descriptors.iter(),
        manifest,
        linked_plugin_ids,
        std::iter::empty::<String>(),
    )
}

fn target_manifest_availability_for_registration_reports<'a>(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    registrations: impl IntoIterator<Item = &'a RuntimePluginRegistrationReport>,
) -> RuntimePluginAvailabilityReport {
    let profile = RuntimeProfileDescriptor::new(
        runtime_profile_id_for_target_availability(target),
        "target module selection",
        target,
    );
    let descriptors = runtime_plugin_descriptors();
    profile.availability_report_for_manifest_and_registration_reports(
        descriptors.iter(),
        manifest,
        registrations,
    )
}

fn runtime_profile_id_for_target_availability(target: RuntimeTargetMode) -> RuntimeProfileId {
    match target {
        RuntimeTargetMode::ClientRuntime => RuntimeProfileId::Client2d,
        RuntimeTargetMode::ServerRuntime => RuntimeProfileId::Server,
        RuntimeTargetMode::EditorHost => RuntimeProfileId::Editor,
    }
}

fn runtime_plugin_descriptors() -> Vec<RuntimePluginDescriptor> {
    RuntimePluginDescriptor::builtin_catalog()
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
    let runtime_prepare_collectors = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .runtime_prepare_collectors()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .runtime_prepare_collectors()
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
    let solari_runtime_providers = active_registrations
        .iter()
        .flat_map(|registration| {
            registration
                .extensions
                .solari_runtime_providers()
                .iter()
                .cloned()
        })
        .chain(
            active_feature_registrations
                .iter()
                .flat_map(|registration| {
                    registration
                        .extensions
                        .solari_runtime_providers()
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
        &runtime_prepare_collectors,
        &hybrid_gi_runtime_providers,
        &solari_runtime_providers,
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
    report.runtime_plugin_availability = target_manifest_availability_for_registration_reports(
        target,
        &manifest,
        registrations.iter(),
    );
    report
}

fn runtime_modules_for_target_with_linked_plugins_and_render_features(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    runtime_prepare_collectors: &[RuntimePrepareCollectorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    solari_runtime_providers: &[SolariRuntimeProviderRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> RuntimeModuleLoadReport {
    let manifest = manifest_with_mode_baseline(target, manifest_override);
    runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
        target,
        &manifest,
        linked_plugin_ids,
        asset_importers,
        render_features,
        render_pass_executors,
        runtime_prepare_collectors,
        hybrid_gi_runtime_providers,
        solari_runtime_providers,
        virtual_geometry_runtime_providers,
    )
}

fn runtime_modules_for_target_with_linked_plugins_and_render_features_for_manifest(
    target: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    asset_importers: &AssetImporterRegistry,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    runtime_prepare_collectors: &[RuntimePrepareCollectorRegistration],
    hybrid_gi_runtime_providers: &[HybridGiRuntimeProviderRegistration],
    solari_runtime_providers: &[SolariRuntimeProviderRegistration],
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
            runtime_prepare_collectors,
            hybrid_gi_runtime_providers,
            solari_runtime_providers,
            virtual_geometry_runtime_providers,
        ));
    report.runtime_plugin_availability =
        target_manifest_availability(target, manifest, linked_plugin_ids.iter());

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

pub fn manifest_for_runtime_profile(profile_id: RuntimeProfileId) -> ProjectPluginManifest {
    RuntimeProfileDescriptor::for_id(profile_id).project_manifest()
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
        RuntimePluginId::Solari => {
            warnings.push(externalized_runtime_plugin_message("solari"));
            None
        }
        RuntimePluginId::ZrVmLanguage => {
            warnings.push(externalized_runtime_plugin_message("zr_vm_language"));
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
        default_manifest_for_target, manifest_with_mode_baseline,
        runtime_modules_for_runtime_profile,
        runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports,
        runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports,
        runtime_modules_for_target_with_linked_plugins,
        runtime_modules_for_target_with_plugin_and_feature_registration_reports, RuntimePluginId,
        RuntimeProfileId, RuntimeTargetMode,
    };
    use crate::{
        plugin::PluginModuleManifest, plugin::PluginPackageManifest, plugin::ProjectPluginManifest,
        plugin::ProjectPluginSelection, plugin::RuntimeExtensionRegistry,
        plugin::RuntimePluginAvailabilityCategory, plugin::RuntimePluginRegistrationReport,
    };

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

    #[test]
    fn runtime_profile_load_report_surfaces_structured_availability() {
        let report = runtime_modules_for_runtime_profile(RuntimeProfileId::Client2d);

        assert!(availability_contains(
            &report.runtime_plugin_availability.externalized_missing,
            RuntimePluginId::Sound
        ));
        assert!(availability_contains(
            &report.runtime_plugin_availability.missing_required,
            RuntimePluginId::Sound
        ));
        assert!(report.has_fatal_diagnostics());
        assert!(report
            .effective_errors()
            .iter()
            .any(|diagnostic| diagnostic.contains("required runtime plugin Sound is unavailable")));
        assert!(report
            .required_missing_summary()
            .contains("required runtime plugin Sound is unavailable"));
        assert!(report
            .required_missing()
            .iter()
            .any(|missing| missing.id == RuntimePluginId::Sound));
        assert!(report
            .effective_required_missing()
            .iter()
            .any(|missing| missing.id == RuntimePluginId::Sound));
    }

    #[test]
    fn minimal_runtime_profile_load_report_has_structured_core_availability() {
        let report = runtime_modules_for_runtime_profile(RuntimeProfileId::Minimal);

        assert!(!report.has_fatal_diagnostics());
        assert!(report
            .runtime_plugin_availability
            .missing_required
            .is_empty());
        assert!(report
            .runtime_plugin_availability
            .externalized_missing
            .is_empty());
    }

    #[test]
    fn target_linked_plugin_report_surfaces_structured_availability() {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::VirtualGeometry,
                true,
                true,
            )],
        };
        let report = runtime_modules_for_target_with_linked_plugins(
            RuntimeTargetMode::ClientRuntime,
            Some(&manifest),
            [RuntimePluginId::VirtualGeometry.key()],
        );

        assert!(availability_contains(
            &report.runtime_plugin_availability.linked,
            RuntimePluginId::VirtualGeometry
        ));
        assert!(report.runtime_plugin_availability.contains(
            RuntimePluginAvailabilityCategory::Linked,
            RuntimePluginId::VirtualGeometry
        ));
        assert_eq!(
            report
                .runtime_plugin_availability
                .category_count(RuntimePluginAvailabilityCategory::Linked),
            1
        );
        assert_eq!(
            report
                .runtime_plugin_availability
                .entry_for(
                    RuntimePluginAvailabilityCategory::Linked,
                    RuntimePluginId::VirtualGeometry
                )
                .map(|entry| entry.id.as_str()),
            Some(RuntimePluginId::VirtualGeometry.key())
        );
        let diagnostic_lines = report.runtime_plugin_availability.diagnostic_lines();
        assert!(diagnostic_lines
            .iter()
            .any(|line| line == "runtime_plugin_availability.linked.count=1"));
        assert!(diagnostic_lines
            .iter()
            .any(|line| line.contains("runtime_plugin_availability.linked=virtual_geometry")));
        assert!(!availability_contains(
            &report.runtime_plugin_availability.missing_required,
            RuntimePluginId::VirtualGeometry
        ));
        assert!(!report.runtime_plugin_availability.has_missing_required());
        assert!(report.effective_required_missing().is_empty());
    }

    #[test]
    fn target_native_dynamic_registration_report_preserves_availability_category() {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::VirtualGeometry,
                true,
                true,
            )],
        };
        let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
            PluginPackageManifest::new("virtual_geometry", "Virtual Geometry").with_runtime_module(
                PluginModuleManifest::runtime(
                    "virtual_geometry.runtime",
                    "zircon_plugin_virtual_geometry_runtime",
                )
                .with_target_modes([RuntimeTargetMode::ClientRuntime]),
            ),
        );

        let report = runtime_modules_for_target_with_plugin_and_feature_registration_reports(
            RuntimeTargetMode::ClientRuntime,
            Some(&manifest),
            [&registration],
            std::iter::empty(),
        );

        assert!(report.runtime_plugin_availability.contains(
            RuntimePluginAvailabilityCategory::NativeDynamic,
            RuntimePluginId::VirtualGeometry
        ));
        assert!(!report.runtime_plugin_availability.contains(
            RuntimePluginAvailabilityCategory::Linked,
            RuntimePluginId::VirtualGeometry
        ));
        assert!(report.effective_required_missing().is_empty());
    }

    #[test]
    fn target_required_missing_is_deduped_between_legacy_and_structured_reports() {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::VirtualGeometry,
                true,
                true,
            )],
        };
        let report = runtime_modules_for_target_with_linked_plugins(
            RuntimeTargetMode::ClientRuntime,
            Some(&manifest),
            std::iter::empty::<String>(),
        );
        let missing = report.effective_required_missing();

        assert_eq!(
            missing
                .iter()
                .filter(|entry| entry.id == RuntimePluginId::VirtualGeometry)
                .count(),
            1
        );
        assert!(report
            .effective_errors()
            .iter()
            .any(|diagnostic| diagnostic.contains("required runtime plugin VirtualGeometry")));
    }

    #[test]
    fn runtime_profile_plugin_and_feature_bootstrap_uses_profile_availability() {
        let sound_registration = linked_runtime_registration(RuntimePluginId::Sound);
        let report =
            runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports(
                RuntimeProfileId::Client2d,
                [&sound_registration],
                std::iter::empty::<&crate::plugin::RuntimePluginFeatureRegistrationReport>(),
            );

        assert!(availability_contains(
            &report.runtime_plugin_availability.linked,
            RuntimePluginId::Sound
        ));
        assert!(!availability_contains(
            &report.runtime_plugin_availability.missing_required,
            RuntimePluginId::Sound
        ));
        assert!(!report
            .effective_required_missing()
            .iter()
            .any(|missing| missing.id == RuntimePluginId::Sound));
    }

    #[test]
    fn runtime_profile_manifest_bootstrap_reports_manifest_optional_provider_availability() {
        let profile = crate::plugin::RuntimeProfileDescriptor::for_id(RuntimeProfileId::Client3d);
        let mut manifest = profile.project_manifest();
        manifest
            .selections
            .push(ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Animation,
                true,
                false,
            ));
        let animation_registration = linked_runtime_registration(RuntimePluginId::Animation);

        let report =
            runtime_modules_for_runtime_profile_manifest_with_plugin_and_feature_registration_reports(
                RuntimeProfileId::Client3d,
                &manifest,
                [&animation_registration],
                std::iter::empty::<&crate::plugin::RuntimePluginFeatureRegistrationReport>(),
            );

        assert!(availability_contains(
            &report.runtime_plugin_availability.linked,
            RuntimePluginId::Animation
        ));
        assert!(!availability_contains(
            &report.runtime_plugin_availability.externalized_missing,
            RuntimePluginId::Animation
        ));
    }

    fn linked_runtime_registration(plugin_id: RuntimePluginId) -> RuntimePluginRegistrationReport {
        RuntimePluginRegistrationReport {
            package_manifest: PluginPackageManifest::new(
                plugin_id.key(),
                format!("{} runtime", plugin_id.key()),
            )
            .with_runtime_crate(format!("zircon_plugin_{}_runtime", plugin_id.key())),
            project_selection: ProjectPluginSelection::runtime_plugin(plugin_id, true, true),
            extensions: RuntimeExtensionRegistry::default(),
            diagnostics: Vec::new(),
        }
    }

    fn availability_contains(
        entries: &[crate::plugin::RuntimePluginAvailabilityEntry],
        plugin_id: RuntimePluginId,
    ) -> bool {
        entries.iter().any(|entry| entry.runtime_id == plugin_id)
    }
}
