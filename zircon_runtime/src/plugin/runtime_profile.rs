use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    plugin::ExportPackagingStrategy, plugin::ProjectPluginManifest, plugin::ProjectPluginSelection,
    plugin::RuntimePluginDescriptor, plugin::RuntimePluginRegistrationReport, RuntimePluginId,
    RuntimeTargetMode,
};

use super::PluginMaturity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeProfileId {
    Minimal,
    Client2d,
    Client3d,
    Editor,
    Dev,
    Server,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProfilePluginSelection {
    pub id: RuntimePluginId,
    #[serde(default)]
    pub required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProfileDescriptor {
    pub id: RuntimeProfileId,
    pub name: String,
    pub target_mode: RuntimeTargetMode,
    #[serde(default)]
    pub default_plugins: Vec<RuntimeProfilePluginSelection>,
    #[serde(default)]
    pub optional_plugins: Vec<RuntimePluginId>,
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    pub minimum_maturity: PluginMaturity,
    #[serde(default)]
    pub allow_externalized_required_plugins: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePluginAvailabilityReport {
    pub available: Vec<RuntimePluginAvailabilityEntry>,
    pub linked: Vec<RuntimePluginAvailabilityEntry>,
    pub native_dynamic: Vec<RuntimePluginAvailabilityEntry>,
    pub externalized_missing: Vec<RuntimePluginAvailabilityEntry>,
    pub stub: Vec<RuntimePluginAvailabilityEntry>,
    pub blocked_by_target: Vec<RuntimePluginAvailabilityEntry>,
    pub blocked_by_maturity: Vec<RuntimePluginAvailabilityEntry>,
    pub missing_required: Vec<RuntimePluginAvailabilityEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePluginAvailabilityEntry {
    pub id: String,
    pub runtime_id: RuntimePluginId,
    pub required: bool,
    pub maturity: PluginMaturity,
    pub reason: String,
}

impl RuntimeProfilePluginSelection {
    pub fn new(id: RuntimePluginId, required: bool) -> Self {
        Self { id, required }
    }
}

impl RuntimeProfileDescriptor {
    pub fn new(
        id: RuntimeProfileId,
        name: impl Into<String>,
        target_mode: RuntimeTargetMode,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            target_mode,
            default_plugins: Vec::new(),
            optional_plugins: Vec::new(),
            required_capabilities: Vec::new(),
            minimum_maturity: PluginMaturity::Experimental,
            allow_externalized_required_plugins: false,
        }
    }

    pub fn for_id(id: RuntimeProfileId) -> Self {
        Self::builtin_profiles()
            .into_iter()
            .find(|profile| profile.id == id)
            .unwrap_or_else(|| panic!("missing built-in runtime profile {id:?}"))
    }

    pub fn builtin_profiles() -> Vec<Self> {
        vec![
            Self::minimal(),
            Self::client_2d(),
            Self::client_3d(),
            Self::editor(),
            Self::dev(),
            Self::server(),
        ]
    }

    pub fn with_default_plugin(mut self, id: RuntimePluginId, required: bool) -> Self {
        self.default_plugins
            .push(RuntimeProfilePluginSelection::new(id, required));
        self
    }

    pub fn with_optional_plugin(mut self, id: RuntimePluginId) -> Self {
        if !self.optional_plugins.contains(&id) {
            self.optional_plugins.push(id);
        }
        self
    }

    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }

    pub fn with_minimum_maturity(mut self, maturity: PluginMaturity) -> Self {
        self.minimum_maturity = maturity;
        self
    }

    pub fn allow_externalized_required_plugins(mut self, allow: bool) -> Self {
        self.allow_externalized_required_plugins = allow;
        self
    }

    pub fn project_manifest(&self) -> ProjectPluginManifest {
        ProjectPluginManifest {
            selections: self
                .default_plugins
                .iter()
                .map(|plugin| {
                    ProjectPluginSelection::runtime_plugin(plugin.id, true, plugin.required)
                        .with_target_modes([self.target_mode])
                })
                .collect(),
        }
    }

    pub fn availability_report<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        self.availability_report_with_provider_gate(
            descriptors,
            linked_plugin_ids,
            std::iter::empty::<String>(),
            false,
        )
    }

    pub fn availability_report_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        self.availability_report_with_provider_gate(
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
            true,
        )
    }

    pub fn availability_report_for_registration_reports<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        registrations: impl IntoIterator<Item = &'b RuntimePluginRegistrationReport>,
    ) -> RuntimePluginAvailabilityReport {
        let mut linked_plugin_ids = Vec::new();
        let mut native_dynamic_plugin_ids = Vec::new();
        for registration in registrations {
            if !registration.project_selection.enabled
                || !registration
                    .project_selection
                    .supports_target(self.target_mode)
            {
                continue;
            }
            let target_ids = if registration.project_selection.packaging
                == ExportPackagingStrategy::NativeDynamic
            {
                &mut native_dynamic_plugin_ids
            } else {
                &mut linked_plugin_ids
            };
            push_provider_id(target_ids, &registration.package_manifest.id);
            push_provider_id(target_ids, &registration.project_selection.id);
        }
        self.availability_report_with_providers(
            descriptors,
            linked_plugin_ids.iter(),
            native_dynamic_plugin_ids.iter(),
        )
    }

    fn availability_report_with_provider_gate<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityReport {
        let descriptors = descriptors
            .into_iter()
            .map(|descriptor| (descriptor.runtime_id, descriptor))
            .collect::<HashMap<_, _>>();
        let linked_plugin_ids = linked_plugin_ids
            .into_iter()
            .map(|id| id.as_ref().to_string())
            .collect::<HashSet<_>>();
        let native_dynamic_plugin_ids = native_dynamic_plugin_ids
            .into_iter()
            .map(|id| id.as_ref().to_string())
            .collect::<HashSet<_>>();
        let mut report = RuntimePluginAvailabilityReport::default();
        for plugin in &self.default_plugins {
            self.report_plugin_availability(
                plugin.id,
                plugin.required,
                &descriptors,
                &linked_plugin_ids,
                &native_dynamic_plugin_ids,
                require_external_provider,
                &mut report,
            );
        }
        for plugin_id in &self.optional_plugins {
            self.report_plugin_availability(
                *plugin_id,
                false,
                &descriptors,
                &linked_plugin_ids,
                &native_dynamic_plugin_ids,
                require_external_provider,
                &mut report,
            );
        }
        report
    }

    fn report_plugin_availability(
        &self,
        plugin_id: RuntimePluginId,
        required: bool,
        descriptors: &HashMap<RuntimePluginId, &RuntimePluginDescriptor>,
        linked_plugin_ids: &HashSet<String>,
        native_dynamic_plugin_ids: &HashSet<String>,
        require_external_provider: bool,
        report: &mut RuntimePluginAvailabilityReport,
    ) {
        let Some(descriptor) = descriptors.get(&plugin_id) else {
            if let Some(entry) = builtin_available_entry(plugin_id, required) {
                report.available.push(entry);
                return;
            }
            push_blocked(
                &mut report.stub,
                &mut report.missing_required,
                RuntimePluginAvailabilityEntry {
                    id: plugin_id.key().to_string(),
                    runtime_id: plugin_id,
                    required,
                    maturity: PluginMaturity::Stub,
                    reason: "plugin is missing from runtime catalog".to_string(),
                },
            );
            return;
        };
        let entry = availability_entry(descriptor, required, String::new());
        if !supports_target(descriptor, self.target_mode) {
            push_blocked(
                &mut report.blocked_by_target,
                &mut report.missing_required,
                RuntimePluginAvailabilityEntry {
                    reason: format!("target {:?} is not supported", self.target_mode),
                    ..entry
                },
            );
            return;
        }
        if descriptor.maturity == PluginMaturity::Externalized {
            let entry = RuntimePluginAvailabilityEntry {
                reason: "plugin runtime is externalized and no linked registration was supplied"
                    .to_string(),
                ..entry
            };
            if self.allow_externalized_required_plugins {
                report.externalized_missing.push(entry);
            } else {
                push_blocked(
                    &mut report.externalized_missing,
                    &mut report.missing_required,
                    entry,
                );
            }
            return;
        }
        if descriptor.maturity == PluginMaturity::Stub {
            push_blocked(
                &mut report.stub,
                &mut report.missing_required,
                RuntimePluginAvailabilityEntry {
                    reason: "plugin catalog entry is a stub".to_string(),
                    ..entry
                },
            );
            return;
        }
        if !descriptor.maturity.meets_minimum(self.minimum_maturity) {
            push_blocked(
                &mut report.blocked_by_maturity,
                &mut report.missing_required,
                RuntimePluginAvailabilityEntry {
                    reason: format!(
                        "plugin maturity {:?} is below profile minimum {:?}",
                        descriptor.maturity, self.minimum_maturity
                    ),
                    ..entry
                },
            );
            return;
        }
        if linked_plugin_ids.contains(&descriptor.package_id)
            || linked_plugin_ids.contains(descriptor.runtime_id.key())
        {
            report.linked.push(RuntimePluginAvailabilityEntry {
                reason: "plugin runtime was supplied by linked registration".to_string(),
                ..entry
            });
            return;
        }
        if native_dynamic_plugin_ids.contains(&descriptor.package_id)
            || native_dynamic_plugin_ids.contains(descriptor.runtime_id.key())
        {
            report.native_dynamic.push(RuntimePluginAvailabilityEntry {
                reason: "plugin runtime was supplied by native dynamic registration".to_string(),
                ..entry
            });
            return;
        }
        if require_external_provider && !builtin_runtime_domain_is_available(descriptor.runtime_id)
        {
            let entry = RuntimePluginAvailabilityEntry {
                reason: "plugin runtime has no linked or native dynamic provider registration"
                    .to_string(),
                ..entry
            };
            if self.allow_externalized_required_plugins {
                report.externalized_missing.push(entry);
            } else {
                push_blocked(
                    &mut report.externalized_missing,
                    &mut report.missing_required,
                    entry,
                );
            }
            return;
        }
        report.available.push(RuntimePluginAvailabilityEntry {
            reason: "plugin descriptor satisfies profile gates".to_string(),
            ..entry
        });
    }

    fn minimal() -> Self {
        Self::new(
            RuntimeProfileId::Minimal,
            "minimal",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Core)
        .with_required_capability("runtime.core.lifecycle")
        .with_required_capability("runtime.core.tasks")
        .with_required_capability("runtime.core.time")
        .with_required_capability("runtime.core.frame_count")
        .with_required_capability("runtime.core.diagnostics")
    }

    fn client_2d() -> Self {
        Self::new(
            RuntimeProfileId::Client2d,
            "client_2d",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Ui, true)
        .with_default_plugin(RuntimePluginId::Sound, true)
        .with_default_plugin(RuntimePluginId::Rendering, true)
        .with_default_plugin(RuntimePluginId::Texture, false)
        .with_optional_plugin(RuntimePluginId::Tilemap2d)
        .with_optional_plugin(RuntimePluginId::Particles)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_required_capability("runtime.core.asset")
        .with_required_capability("runtime.core.scene")
        .with_required_capability("runtime.core.render.base")
        .with_required_capability("runtime.plugin.sound")
        .with_required_capability("runtime.plugin.rendering")
    }

    fn client_3d() -> Self {
        Self::new(
            RuntimeProfileId::Client3d,
            "client_3d",
            RuntimeTargetMode::ClientRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Ui, true)
        .with_default_plugin(RuntimePluginId::Sound, true)
        .with_default_plugin(RuntimePluginId::Rendering, true)
        .with_default_plugin(RuntimePluginId::Texture, false)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_optional_plugin(RuntimePluginId::Navigation)
        .with_optional_plugin(RuntimePluginId::Particles)
        .with_optional_plugin(RuntimePluginId::VirtualGeometry)
        .with_optional_plugin(RuntimePluginId::HybridGi)
        .with_optional_plugin(RuntimePluginId::Solari)
        .with_required_capability("runtime.core.asset")
        .with_required_capability("runtime.core.scene")
        .with_required_capability("runtime.core.render.base")
        .with_required_capability("runtime.plugin.sound")
        .with_required_capability("runtime.plugin.rendering")
    }

    fn editor() -> Self {
        Self::new(
            RuntimeProfileId::Editor,
            "editor",
            RuntimeTargetMode::EditorHost,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Ui, true)
        .with_default_plugin(RuntimePluginId::Sound, true)
        .with_default_plugin(RuntimePluginId::Rendering, true)
        .with_default_plugin(RuntimePluginId::Texture, false)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_optional_plugin(RuntimePluginId::Navigation)
        .with_optional_plugin(RuntimePluginId::Particles)
        .with_optional_plugin(RuntimePluginId::Net)
        .with_required_capability("editor.host.ui_shell")
        .with_required_capability("editor.host.plugin_management")
    }

    fn dev() -> Self {
        Self::new(RuntimeProfileId::Dev, "dev", RuntimeTargetMode::EditorHost)
            .with_minimum_maturity(PluginMaturity::Experimental)
            .with_default_plugin(RuntimePluginId::Ui, true)
            .with_default_plugin(RuntimePluginId::Sound, true)
            .with_default_plugin(RuntimePluginId::Rendering, true)
            .with_default_plugin(RuntimePluginId::Texture, false)
            .with_default_plugin(RuntimePluginId::Net, false)
            .with_optional_plugin(RuntimePluginId::Animation)
            .with_optional_plugin(RuntimePluginId::Navigation)
            .with_optional_plugin(RuntimePluginId::Particles)
            .with_optional_plugin(RuntimePluginId::VirtualGeometry)
            .with_optional_plugin(RuntimePluginId::HybridGi)
            .with_optional_plugin(RuntimePluginId::Solari)
            .with_required_capability("runtime.core.diagnostics")
            .with_required_capability("editor.host.plugin_management")
    }

    fn server() -> Self {
        Self::new(
            RuntimeProfileId::Server,
            "server",
            RuntimeTargetMode::ServerRuntime,
        )
        .with_minimum_maturity(PluginMaturity::Beta)
        .with_default_plugin(RuntimePluginId::Net, false)
        .with_optional_plugin(RuntimePluginId::Physics)
        .with_optional_plugin(RuntimePluginId::Animation)
        .with_optional_plugin(RuntimePluginId::Navigation)
        .with_required_capability("runtime.core.lifecycle")
        .with_required_capability("runtime.core.scene")
    }
}

fn builtin_available_entry(
    id: RuntimePluginId,
    required: bool,
) -> Option<RuntimePluginAvailabilityEntry> {
    match id {
        RuntimePluginId::Ui => Some(RuntimePluginAvailabilityEntry {
            id: id.key().to_string(),
            runtime_id: id,
            required,
            maturity: PluginMaturity::Core,
            reason: "plugin is provided by the built-in runtime domain".to_string(),
        }),
        _ => None,
    }
}

fn builtin_runtime_domain_is_available(id: RuntimePluginId) -> bool {
    matches!(id, RuntimePluginId::Ui)
}

fn push_provider_id(ids: &mut Vec<String>, id: &str) {
    if !ids.iter().any(|existing| existing == id) {
        ids.push(id.to_string());
    }
}

fn availability_entry(
    descriptor: &RuntimePluginDescriptor,
    required: bool,
    reason: String,
) -> RuntimePluginAvailabilityEntry {
    RuntimePluginAvailabilityEntry {
        id: descriptor.package_id.clone(),
        runtime_id: descriptor.runtime_id,
        required,
        maturity: descriptor.maturity,
        reason,
    }
}

fn supports_target(descriptor: &RuntimePluginDescriptor, target: RuntimeTargetMode) -> bool {
    descriptor.target_modes.is_empty() || descriptor.target_modes.contains(&target)
}

fn push_blocked(
    category: &mut Vec<RuntimePluginAvailabilityEntry>,
    missing_required: &mut Vec<RuntimePluginAvailabilityEntry>,
    entry: RuntimePluginAvailabilityEntry,
) {
    push_missing_required(missing_required, entry.clone());
    category.push(entry);
}

fn push_missing_required(
    missing_required: &mut Vec<RuntimePluginAvailabilityEntry>,
    entry: RuntimePluginAvailabilityEntry,
) {
    if entry.required {
        missing_required.push(entry);
    }
}
