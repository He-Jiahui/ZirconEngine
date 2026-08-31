use super::super::behavior_calls::NativePluginBehaviorCallbacks;
use super::super::{NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR};
use super::callback::{NativePluginCallbackLeaseError, NativePluginLibraryGenerationOwner};

#[derive(Clone)]
pub(in crate::plugin::native_plugin_loader) struct NativePluginBehaviorSnapshot {
    behavior: Option<NativePluginBehaviorCallbacks>,
    module_kind: &'static str,
    generation_owner: NativePluginLibraryGenerationOwner,
}

#[derive(Clone)]
pub struct NativePluginEditorCommandBinding {
    plugin_id: String,
    command_name: String,
    payload_schema: String,
    max_output_bytes: usize,
    snapshot: NativePluginBehaviorSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NativePluginEditorCommandBindingError {
    CallbackSnapshotUnavailable {
        plugin_id: String,
        detail: String,
    },
    MissingEditorBehavior {
        plugin_id: String,
    },
    MissingInvokeCommandCallback {
        plugin_id: String,
    },
    UndeclaredCommand {
        plugin_id: String,
        command_name: String,
    },
}

impl NativePluginBehaviorSnapshot {
    pub(super) fn new(
        behavior: Option<NativePluginBehaviorCallbacks>,
        module_kind: &'static str,
        generation_owner: NativePluginLibraryGenerationOwner,
    ) -> Self {
        Self {
            behavior,
            module_kind,
            generation_owner,
        }
    }

    pub(in crate::plugin::native_plugin_loader) fn invoke_command(
        &self,
        name: &str,
        payload: &[u8],
    ) -> NativePluginBehaviorCallReport {
        let Some(behavior) = self.behavior.as_ref() else {
            return missing_behavior_report(self.module_kind);
        };
        self.invoke_measured(|| behavior.invoke_command(name, payload))
    }

    pub(in crate::plugin::native_plugin_loader) fn bind_editor_command(
        self,
        plugin_id: String,
        command_name: String,
    ) -> Result<NativePluginEditorCommandBinding, NativePluginEditorCommandBindingError> {
        let Some(behavior) = self.behavior.as_ref() else {
            return Err(NativePluginEditorCommandBindingError::MissingEditorBehavior { plugin_id });
        };
        if !behavior.has_invoke_command() {
            return Err(
                NativePluginEditorCommandBindingError::MissingInvokeCommandCallback { plugin_id },
            );
        }
        if !behavior.declares_command(&command_name) {
            return Err(NativePluginEditorCommandBindingError::UndeclaredCommand {
                plugin_id,
                command_name,
            });
        }
        let (payload_schema, max_output_bytes) = behavior
            .command_metadata(&command_name)
            .expect("declared command must have manifest metadata");
        Ok(NativePluginEditorCommandBinding {
            plugin_id,
            command_name,
            payload_schema,
            max_output_bytes,
            snapshot: self,
        })
    }

    pub(in crate::plugin::native_plugin_loader) fn save_state(
        &self,
    ) -> NativePluginBehaviorCallReport {
        let Some(behavior) = self.behavior.as_ref() else {
            return missing_behavior_report(self.module_kind);
        };
        self.invoke_measured(|| behavior.save_state())
    }

    pub(in crate::plugin::native_plugin_loader) fn restore_state(
        &self,
        state: &[u8],
    ) -> NativePluginBehaviorCallReport {
        let Some(behavior) = self.behavior.as_ref() else {
            return missing_behavior_report(self.module_kind);
        };
        self.invoke_measured(|| behavior.restore_state(state))
    }

    pub(in crate::plugin::native_plugin_loader) fn unload(&self) -> NativePluginBehaviorCallReport {
        let Some(behavior) = self.behavior.as_ref() else {
            return missing_behavior_report(self.module_kind);
        };
        self.invoke_measured(|| behavior.unload())
    }

    fn invoke_measured(
        &self,
        callback: impl FnOnce() -> NativePluginBehaviorCallReport,
    ) -> NativePluginBehaviorCallReport {
        let callback_lease = match self.generation_owner.acquire_callback() {
            Ok(lease) => lease,
            Err(error) => return callback_rejected_report(self.module_kind, error),
        };
        let started_at = callback_lease.begin_callback_measurement();
        let report = callback();
        callback_lease.complete_callback_measurement(started_at);
        report
    }
}

impl NativePluginEditorCommandBinding {
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn command_name(&self) -> &str {
        &self.command_name
    }

    pub fn payload_schema_id(&self) -> &str {
        &self.payload_schema
    }

    pub fn max_output_bytes(&self) -> usize {
        self.max_output_bytes
    }

    pub fn invoke(&self, payload: &[u8]) -> NativePluginBehaviorCallReport {
        self.snapshot.invoke_command(&self.command_name, payload)
    }
}

impl std::fmt::Debug for NativePluginEditorCommandBinding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativePluginEditorCommandBinding")
            .field("plugin_id", &self.plugin_id)
            .field("command_name", &self.command_name)
            .field("payload_schema", &self.payload_schema)
            .field("max_output_bytes", &self.max_output_bytes)
            .finish_non_exhaustive()
    }
}

impl std::fmt::Display for NativePluginEditorCommandBindingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CallbackSnapshotUnavailable { plugin_id, detail } => write!(
                formatter,
                "native editor plugin `{plugin_id}` callback snapshot is unavailable: {detail}"
            ),
            Self::MissingEditorBehavior { plugin_id } => write!(
                formatter,
                "native editor plugin `{plugin_id}` has no editor behavior"
            ),
            Self::MissingInvokeCommandCallback { plugin_id } => write!(
                formatter,
                "native editor plugin `{plugin_id}` has no editor invoke_command callback"
            ),
            Self::UndeclaredCommand {
                plugin_id,
                command_name,
            } => write!(
                formatter,
                "native editor plugin `{plugin_id}` does not declare command `{command_name}` in its command manifest"
            ),
        }
    }
}

impl std::error::Error for NativePluginEditorCommandBindingError {}

pub(super) fn callback_rejected_report(
    module_kind: &str,
    error: NativePluginCallbackLeaseError,
) -> NativePluginBehaviorCallReport {
    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec![format!(
            "native plugin {module_kind} behavior callback rejected: {error}"
        )],
        payload: None,
    }
}

pub(super) fn missing_behavior_report(module_kind: &str) -> NativePluginBehaviorCallReport {
    NativePluginBehaviorCallReport {
        status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: vec![format!("native plugin {module_kind} behavior is missing")],
        payload: None,
    }
}
