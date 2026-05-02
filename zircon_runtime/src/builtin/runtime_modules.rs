use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::engine_module::EngineModule;
use crate::graphics::{
    RenderFeatureDescriptor, RenderPassExecutorRegistration,
    VirtualGeometryRuntimeProviderRegistration,
};
use crate::plugin::RuntimePluginRegistrationReport;
#[cfg(feature = "plugin-ui")]
use crate::ui;
use crate::{animation, asset, foundation, graphics, input, physics, platform, scene, script};
use crate::{ProjectPluginManifest, ProjectPluginSelection};

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
    runtime_core_modules_for_target_with_render_features(target, &[], &[], &[])
}

fn runtime_core_modules_for_target_with_render_features(
    target: RuntimeTargetMode,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> Vec<Arc<dyn EngineModule>> {
    let mut modules: Vec<Arc<dyn EngineModule>> = vec![
        Arc::new(foundation::FoundationModule),
        Arc::new(platform::PlatformModule),
        Arc::new(input::InputModule),
        Arc::new(asset::AssetModule),
        Arc::new(scene::SceneModule),
        Arc::new(physics::PhysicsModule),
        Arc::new(animation::AnimationModule),
    ];
    if target != RuntimeTargetMode::ServerRuntime {
        modules.push(Arc::new(graphics::GraphicsModule::with_render_extensions(
            render_features.iter().cloned(),
            render_pass_executors.iter().cloned(),
            virtual_geometry_runtime_providers.iter().cloned(),
        )));
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
    runtime_modules_for_target_with_linked_plugins_and_render_features(
        target,
        manifest_override,
        linked_plugin_ids,
        &render_features,
        &render_pass_executors,
        &virtual_geometry_runtime_providers,
    )
}

fn runtime_modules_for_target_with_linked_plugins_and_render_features(
    target: RuntimeTargetMode,
    manifest_override: Option<&ProjectPluginManifest>,
    linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    render_features: &[RenderFeatureDescriptor],
    render_pass_executors: &[RenderPassExecutorRegistration],
    virtual_geometry_runtime_providers: &[VirtualGeometryRuntimeProviderRegistration],
) -> RuntimeModuleLoadReport {
    let linked_plugin_ids = linked_plugin_ids
        .into_iter()
        .map(|id| id.as_ref().to_string())
        .collect::<HashSet<_>>();
    let mut report =
        RuntimeModuleLoadReport::new(runtime_core_modules_for_target_with_render_features(
            target,
            render_features,
            render_pass_executors,
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
    matches!(id, RuntimePluginId::Physics | RuntimePluginId::Animation)
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
            warnings.push(builtin_runtime_domain_message("physics"));
            Some(Arc::new(physics::PhysicsModule))
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
            warnings.push(builtin_runtime_domain_message("animation"));
            Some(Arc::new(animation::AnimationModule))
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

fn builtin_runtime_domain_message(plugin_id: &str) -> String {
    format!("runtime implementation is built into zircon_runtime::{plugin_id}")
}

#[cfg(test)]
mod tests {
    use super::{
        default_manifest_for_target, manifest_with_mode_baseline, RuntimePluginId,
        RuntimeTargetMode,
    };
    use crate::{ProjectPluginManifest, ProjectPluginSelection};

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
