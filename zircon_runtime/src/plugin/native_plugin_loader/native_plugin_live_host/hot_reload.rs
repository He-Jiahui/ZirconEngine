use crate::plugin::PluginModuleKind;

use super::super::LoadedNativePlugin;
use super::keys::module_kind_label;

#[derive(Debug)]
pub(super) struct NativePluginHotReloadState {
    pub(super) module_kind: PluginModuleKind,
    pub(super) key: String,
    existing: Option<LoadedNativePlugin>,
    previous_unloaded: bool,
    diagnostics: Vec<String>,
}

impl NativePluginHotReloadState {
    pub(super) fn new(
        module_kind: PluginModuleKind,
        key: String,
        existing: Option<LoadedNativePlugin>,
    ) -> Self {
        Self {
            module_kind,
            key,
            existing,
            previous_unloaded: false,
            diagnostics: Vec::new(),
        }
    }

    pub(super) fn take_existing_for_unload(&mut self) -> Option<LoadedNativePlugin> {
        self.existing.take()
    }

    pub(super) fn mark_existing_unloaded(&mut self, diagnostics: Vec<String>) {
        self.previous_unloaded = true;
        self.diagnostics.extend(diagnostics);
    }

    pub(super) fn rollback_error(&mut self, error: String) -> String {
        let rollback = if self.existing.is_some() {
            format!(
                "rolled back to the previously loaded {} native package",
                module_kind_label(self.module_kind)
            )
        } else if self.previous_unloaded {
            format!(
                "rollback unavailable because previous {} native package was already unloaded",
                module_kind_label(self.module_kind)
            )
        } else {
            format!(
                "rollback not needed because no {} native package was previously loaded",
                module_kind_label(self.module_kind)
            )
        };
        let diagnostics = if self.diagnostics.is_empty() {
            rollback
        } else {
            format!("{rollback}; {}", self.diagnostics.join("; "))
        };
        format!("{error}; {diagnostics}")
    }

    pub(super) fn into_rollback_plugin(self) -> Option<LoadedNativePlugin> {
        self.existing
    }
}
