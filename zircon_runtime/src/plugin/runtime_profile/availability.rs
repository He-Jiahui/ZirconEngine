use std::collections::{HashMap, HashSet};

use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginManifest};
use crate::plugin::{PluginMaturity, RuntimePluginDescriptor, RuntimePluginRegistrationReport};
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

use super::availability_report::{RuntimePluginAvailabilityEntry, RuntimePluginAvailabilityReport};
use super::descriptor::RuntimeProfileDescriptor;

impl RuntimeProfileDescriptor {
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
        self.availability_report_for_manifest_and_registration_reports(
            descriptors,
            &self.project_manifest(),
            registrations,
        )
    }

    pub fn availability_report_for_manifest_and_registration_reports<'a, 'b>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
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
        }
        self.availability_report_for_manifest_with_providers(
            descriptors,
            manifest,
            linked_plugin_ids.iter(),
            native_dynamic_plugin_ids.iter(),
        )
    }

    pub fn availability_report_for_manifest_with_providers<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        manifest: &ProjectPluginManifest,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> RuntimePluginAvailabilityReport {
        let mut plugins = Vec::<(RuntimePluginId, bool)>::new();
        for selection in manifest.enabled_for_target(self.target_mode) {
            let Some(runtime_id) = RuntimePluginId::parse_key(&selection.id) else {
                continue;
            };
            if let Some((_, required)) = plugins.iter_mut().find(|(id, _)| *id == runtime_id) {
                *required = *required || selection.required;
            } else {
                plugins.push((runtime_id, selection.required));
            }
        }
        self.availability_report_for_runtime_plugins_with_provider_gate(
            plugins,
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
            true,
        )
    }

    fn availability_report_with_provider_gate<'a>(
        &self,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityReport {
        let plugins = self
            .default_plugins
            .iter()
            .map(|plugin| (plugin.id, plugin.required))
            .chain(
                self.optional_plugins
                    .iter()
                    .copied()
                    .map(|plugin_id| (plugin_id, false)),
            );
        self.availability_report_for_runtime_plugins_with_provider_gate(
            plugins,
            descriptors,
            linked_plugin_ids,
            native_dynamic_plugin_ids,
            require_external_provider,
        )
    }

    fn availability_report_for_runtime_plugins_with_provider_gate<'a>(
        &self,
        plugins: impl IntoIterator<Item = (RuntimePluginId, bool)>,
        descriptors: impl IntoIterator<Item = &'a RuntimePluginDescriptor>,
        linked_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        native_dynamic_plugin_ids: impl IntoIterator<Item = impl AsRef<str>>,
        require_external_provider: bool,
    ) -> RuntimePluginAvailabilityReport {
        let descriptors = descriptors
            .into_iter()
            .map(|descriptor| (descriptor.runtime_id(), descriptor))
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
        for (plugin_id, required) in plugins {
            self.report_plugin_availability(
                plugin_id,
                required,
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
            if let Some(entry) = builtin_unavailable_entry(plugin_id, required) {
                push_blocked(&mut report.stub, &mut report.missing_required, entry);
                return;
            }
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
        if descriptor.maturity() == PluginMaturity::Externalized {
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
        if descriptor.maturity() == PluginMaturity::Stub {
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
        if !descriptor.maturity().meets_minimum(self.minimum_maturity) {
            push_blocked(
                &mut report.blocked_by_maturity,
                &mut report.missing_required,
                RuntimePluginAvailabilityEntry {
                    reason: format!(
                        "plugin maturity {:?} is below profile minimum {:?}",
                        descriptor.maturity(),
                        self.minimum_maturity
                    ),
                    ..entry
                },
            );
            return;
        }
        if linked_plugin_ids.contains(descriptor.package_id()) {
            report.linked.push(RuntimePluginAvailabilityEntry {
                reason: "plugin runtime was supplied by linked registration".to_string(),
                ..entry
            });
            return;
        }
        if native_dynamic_plugin_ids.contains(descriptor.package_id()) {
            report.native_dynamic.push(RuntimePluginAvailabilityEntry {
                reason: "plugin runtime was supplied by native dynamic registration".to_string(),
                ..entry
            });
            return;
        }
        if require_external_provider
            && !builtin_runtime_domain_is_available(descriptor.runtime_id())
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
}

fn builtin_available_entry(
    id: RuntimePluginId,
    required: bool,
) -> Option<RuntimePluginAvailabilityEntry> {
    match id {
        RuntimePluginId::Ui if cfg!(feature = "ui") => Some(RuntimePluginAvailabilityEntry {
            id: id.key().to_string(),
            runtime_id: id,
            required,
            maturity: PluginMaturity::Core,
            reason: "plugin is provided by the built-in runtime domain".to_string(),
        }),
        _ => None,
    }
}

fn builtin_unavailable_entry(
    id: RuntimePluginId,
    required: bool,
) -> Option<RuntimePluginAvailabilityEntry> {
    match id {
        RuntimePluginId::Ui if !cfg!(feature = "ui") => Some(RuntimePluginAvailabilityEntry {
            id: id.key().to_string(),
            runtime_id: id,
            required,
            maturity: PluginMaturity::Core,
            reason: "built-in UI runtime is unavailable because the ui feature is disabled"
                .to_string(),
        }),
        _ => None,
    }
}

fn builtin_runtime_domain_is_available(id: RuntimePluginId) -> bool {
    matches!(id, RuntimePluginId::Ui) && cfg!(feature = "ui")
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
        id: descriptor.package_id().to_string(),
        runtime_id: descriptor.runtime_id(),
        required,
        maturity: descriptor.maturity(),
        reason,
    }
}

fn supports_target(descriptor: &RuntimePluginDescriptor, target: RuntimeTargetMode) -> bool {
    descriptor.target_modes().is_empty() || descriptor.target_modes().contains(&target)
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
