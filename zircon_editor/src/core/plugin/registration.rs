//! Registration report construction and lifecycle diagnostics.

use std::collections::BTreeMap;

use zircon_runtime::plugin::native::NativePluginEditorCommandBinding;
use zircon_runtime::plugin::PluginPackageManifest;

use crate::core::editor_extension::EditorExtensionRegistry;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::runtime_event_consumer::EditorRuntimeEventConsumerRegistry;

use super::descriptor::EditorPlugin;
use super::isolation::run_editor_plugin_boundary;
use super::sdk::lifecycle::{
    EditorPluginLifecycleEvent, EditorPluginLifecycleRecord, EditorPluginLifecycleReport,
    EditorPluginLifecycleStage,
};

#[derive(Clone, Debug)]
pub struct EditorPluginRegistrationReport {
    pub package_manifest: PluginPackageManifest,
    pub capabilities: Vec<String>,
    // Raw contributions are manager input, never a public catalog read surface.
    // `EditorPluginManagerSnapshot::active_extensions` is the phase-gated view.
    pub(crate) extensions: EditorExtensionRegistry,
    pub lifecycle: EditorPluginLifecycleReport,
    pub(crate) successful_lifecycle_stages: Vec<EditorPluginLifecycleStage>,
    pub(crate) failed_lifecycle_stages: Vec<EditorPluginLifecycleStage>,
    pub runtime_event_consumers: EditorRuntimeEventConsumerRegistry,
    pub(crate) native_command_bindings:
        BTreeMap<EditorOperationPath, NativePluginEditorCommandBinding>,
    pub diagnostics: Vec<String>,
}

impl EditorPluginRegistrationReport {
    pub fn from_plugin(plugin: &dyn EditorPlugin, runtime_manifest: PluginPackageManifest) -> Self {
        let package_id = plugin.descriptor().package_id.as_str();
        let mut diagnostics = Vec::new();
        let mut extensions = EditorExtensionRegistry::default();
        let mut candidate_extensions = EditorExtensionRegistry::default();
        match run_editor_plugin_boundary(package_id, "extension registration", || {
            plugin
                .register_editor_extensions(&mut candidate_extensions)
                .map_err(|error| error.to_string())
        }) {
            Ok(()) => {
                extensions = candidate_extensions;
            }
            Err(error) => diagnostics.push(error.to_string()),
        }
        let runtime_event_consumers =
            match run_editor_plugin_boundary(package_id, "runtime event consumer discovery", || {
                Ok(plugin.runtime_event_consumers())
            }) {
                Ok(consumers) => consumers,
                Err(error) => {
                    diagnostics.push(error.to_string());
                    EditorRuntimeEventConsumerRegistry::default()
                }
            };
        if runtime_event_consumers.manifests().as_slice()
            != plugin.descriptor().event_consumers.as_slice()
        {
            diagnostics.push(format!(
                "editor plugin `{}` runtime event consumer registry does not match its descriptor",
                plugin.descriptor().package_id
            ));
        }
        Self {
            package_manifest: plugin.package_manifest(runtime_manifest),
            capabilities: plugin.editor_capabilities().to_vec(),
            extensions,
            // Lifecycle callbacks are phase-scheduled by EditorPluginManager after
            // catalog admission. Catalog construction must remain side-effect free.
            lifecycle: EditorPluginLifecycleReport::default(),
            successful_lifecycle_stages: Vec::new(),
            failed_lifecycle_stages: Vec::new(),
            runtime_event_consumers,
            native_command_bindings: BTreeMap::new(),
            diagnostics,
        }
    }

    pub(super) fn record_lifecycle_event(
        &mut self,
        plugin: &dyn EditorPlugin,
        event: EditorPluginLifecycleEvent,
    ) -> EditorPluginLifecycleReport {
        let report = dispatch_lifecycle_event(plugin, event);
        self.record_lifecycle_report(report)
    }

    /// Records lifecycle ownership that is implemented by the native host boundary.
    pub(super) fn record_host_lifecycle_event(
        &mut self,
        event: EditorPluginLifecycleEvent,
    ) -> EditorPluginLifecycleReport {
        let mut report = EditorPluginLifecycleReport::default();
        report.record(EditorPluginLifecycleRecord::new(
            self.package_manifest.id.clone(),
            event,
        ));
        self.record_lifecycle_report(report)
    }

    fn record_lifecycle_report(
        &mut self,
        report: EditorPluginLifecycleReport,
    ) -> EditorPluginLifecycleReport {
        let stage = report
            .records()
            .last()
            .expect("every manager lifecycle dispatch records its event")
            .event()
            .stage()
            .clone();
        if report.is_success() {
            if !self.successful_lifecycle_stages.contains(&stage) {
                self.successful_lifecycle_stages.push(stage.clone());
            }
            remove_failed_lifecycle_stage(&mut self.failed_lifecycle_stages, &stage);
        } else if !self.failed_lifecycle_stages.contains(&stage) {
            self.failed_lifecycle_stages.push(stage);
        }
        self.lifecycle.extend(report.clone());
        report
    }

    pub(super) fn lifecycle_stage_succeeded(&self, stage: &EditorPluginLifecycleStage) -> bool {
        self.successful_lifecycle_stages.contains(stage)
    }

    pub(super) fn lifecycle_stage_failed(&self, stage: &EditorPluginLifecycleStage) -> bool {
        self.failed_lifecycle_stages.contains(stage)
    }

    pub fn is_success(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

fn remove_failed_lifecycle_stage(
    failed_stages: &mut Vec<EditorPluginLifecycleStage>,
    stage: &EditorPluginLifecycleStage,
) -> bool {
    let Some(index) = failed_stages
        .iter()
        .position(|failed_stage| failed_stage == stage)
    else {
        return false;
    };
    failed_stages.swap_remove(index);
    true
}

fn dispatch_lifecycle_event(
    plugin: &dyn EditorPlugin,
    event: EditorPluginLifecycleEvent,
) -> EditorPluginLifecycleReport {
    let mut lifecycle = EditorPluginLifecycleReport::default();
    lifecycle.record(EditorPluginLifecycleRecord::new(
        plugin.descriptor().package_id.clone(),
        event.clone(),
    ));
    if let Err(error) = run_editor_plugin_boundary(
        plugin.descriptor().package_id.as_str(),
        "lifecycle callback",
        || {
            plugin
                .on_lifecycle_event(&event)
                .map_err(|error| error.to_string())
        },
    ) {
        let diagnostic = error.to_string();
        lifecycle.push_diagnostic(diagnostic.clone());
    }
    lifecycle
}

#[cfg(test)]
#[path = "registration/optimization_tests.rs"]
mod optimization_tests;
