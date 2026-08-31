//! Shared editor command execution metadata and native endpoint registrations.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use super::{EditorCommandAction, EditorCommandDescriptor};
use crate::core::editor_operation::EditorOperationPath;

pub use zircon_runtime::plugin::native::NativePluginEditorCommandBinding;
use zircon_runtime::plugin::native::{
    NativePluginBehaviorCallReport, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
pub use zircon_runtime_interface::editor_command_execution::{
    EditorCommandExecutionContract, EditorCommandResourceBudget, EditorCommandResourceBudgetError,
    EditorCommandResultCodecId, EditorCommandResultCodecIdError,
    MAX_EDITOR_COMMAND_EXECUTION_TIME_MS, MAX_EDITOR_COMMAND_INPUT_BYTES,
    MAX_EDITOR_COMMAND_OUTPUT_BYTES,
};

/// Runtime-only map of executable native endpoints keyed by canonical command id.
///
/// This map is deliberately not serialized and is not copied when a command projection is
/// cloned. A projection owns metadata; the live generation owns callback bindings.
#[derive(Debug, Default)]
pub struct EditorCommandExecutorRegistry {
    registrations: BTreeMap<EditorOperationPath, NativeCommandExecutorRegistration>,
}

impl EditorCommandExecutorRegistry {
    pub fn register_native(
        &mut self,
        descriptor: &EditorCommandDescriptor,
        binding: NativePluginEditorCommandBinding,
    ) -> Result<(), EditorCommandExecutorRegistryError> {
        if !matches!(descriptor.action(), EditorCommandAction::NativeEndpoint) {
            return Err(EditorCommandExecutorRegistryError::NonNativeAction {
                command_id: descriptor.id().clone(),
            });
        }
        let Some(contract) = descriptor.execution_contract().cloned() else {
            return Err(
                EditorCommandExecutorRegistryError::MissingExecutionContract {
                    command_id: descriptor.id().clone(),
                },
            );
        };
        contract.validate().map_err(|error| {
            EditorCommandExecutorRegistryError::InvalidExecutionContract {
                command_id: descriptor.id().clone(),
                detail: error.to_string(),
            }
        })?;
        if self.registrations.contains_key(descriptor.id()) {
            return Err(EditorCommandExecutorRegistryError::DuplicateExecutor {
                command_id: descriptor.id().clone(),
            });
        }
        if binding.command_name() != descriptor.id().as_str() {
            return Err(EditorCommandExecutorRegistryError::CommandNameMismatch {
                command_id: descriptor.id().clone(),
                binding_name: binding.command_name().to_owned(),
            });
        }
        if descriptor.payload_schema_id() != Some(binding.payload_schema_id()) {
            return Err(EditorCommandExecutorRegistryError::PayloadSchemaMismatch {
                command_id: descriptor.id().clone(),
                descriptor_schema: descriptor.payload_schema_id().map(str::to_owned),
                binding_schema: binding.payload_schema_id().to_owned(),
            });
        }
        if binding.max_output_bytes() > contract.resource_budget().max_output_bytes() {
            return Err(EditorCommandExecutorRegistryError::OutputBudgetTooSmall {
                command_id: descriptor.id().clone(),
                descriptor_limit: contract.resource_budget().max_output_bytes(),
                binding_limit: binding.max_output_bytes(),
            });
        }
        self.registrations.insert(
            descriptor.id().clone(),
            NativeCommandExecutorRegistration {
                command_id: descriptor.id().clone(),
                binding,
                contract,
                admitted: Arc::new(AtomicBool::new(true)),
            },
        );
        Ok(())
    }

    pub fn get(
        &self,
        command_id: &EditorOperationPath,
    ) -> Option<&NativeCommandExecutorRegistration> {
        self.registrations.get(command_id)
    }

    pub fn revoke(&mut self, command_id: &EditorOperationPath) -> bool {
        let Some(registration) = self.registrations.remove(command_id) else {
            return false;
        };
        registration.admitted.store(false, Ordering::Release);
        true
    }

    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }
}

impl Drop for EditorCommandExecutorRegistry {
    fn drop(&mut self) {
        for registration in self.registrations.values() {
            registration.admitted.store(false, Ordering::Release);
        }
    }
}

#[derive(Debug)]
pub struct NativeCommandExecutorRegistration {
    command_id: EditorOperationPath,
    binding: NativePluginEditorCommandBinding,
    contract: EditorCommandExecutionContract,
    admitted: Arc<AtomicBool>,
}

impl NativeCommandExecutorRegistration {
    pub fn command_id(&self) -> &EditorOperationPath {
        &self.command_id
    }

    pub fn plugin_id(&self) -> &str {
        self.binding.plugin_id()
    }

    pub fn command_name(&self) -> &str {
        self.binding.command_name()
    }

    pub fn contract(&self) -> &EditorCommandExecutionContract {
        &self.contract
    }

    pub fn invoke(&self, payload: &[u8]) -> EditorCommandExecutionReceipt {
        if !self.admitted.load(Ordering::Acquire) {
            return EditorCommandExecutionReceipt::rejected(
                self.command_id.clone(),
                self.plugin_id().to_owned(),
                "native editor command executor has been revoked",
            );
        }
        let budget = self.contract.resource_budget();
        if payload.len() > budget.max_input_bytes() {
            return EditorCommandExecutionReceipt::rejected(
                self.command_id.clone(),
                self.plugin_id().to_owned(),
                format!(
                    "native editor command input uses {} bytes; budget is {}",
                    payload.len(),
                    budget.max_input_bytes()
                ),
            );
        }

        let started_at = Instant::now();
        let report = self.binding.invoke(payload);
        let mut receipt = EditorCommandExecutionReceipt::from_report(
            self.command_id.clone(),
            self.plugin_id().to_owned(),
            budget.max_output_bytes(),
            report,
        );
        if started_at.elapsed() > Duration::from_millis(budget.max_execution_time_ms()) {
            receipt.status_code = ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
            receipt.payload = None;
            receipt.diagnostics.push(format!(
                "native editor command exceeded execution budget of {} ms",
                budget.max_execution_time_ms()
            ));
        }
        receipt
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorCommandExecutionReceipt {
    command_id: EditorOperationPath,
    plugin_id: String,
    status_code: u32,
    payload: Option<Vec<u8>>,
    diagnostics: Vec<String>,
}

impl EditorCommandExecutionReceipt {
    fn from_report(
        command_id: EditorOperationPath,
        plugin_id: String,
        max_output_bytes: usize,
        mut report: NativePluginBehaviorCallReport,
    ) -> Self {
        if max_output_bytes == 0 {
            if report
                .payload
                .as_ref()
                .is_some_and(|payload| !payload.is_empty())
            {
                report.status_code = ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
                report.payload = None;
                report.diagnostics.push(
                    "native editor command returned output despite a zero-byte output budget"
                        .to_owned(),
                );
            } else {
                report.payload = None;
            }
        }
        if report
            .payload
            .as_ref()
            .is_some_and(|payload| payload.len() > max_output_bytes)
        {
            report.status_code = ZIRCON_NATIVE_PLUGIN_STATUS_ERROR;
            report.payload = None;
            report.diagnostics.push(format!(
                "native editor command output exceeds budget of {max_output_bytes} bytes"
            ));
        }
        Self {
            command_id,
            plugin_id,
            status_code: report.status_code,
            payload: report.payload,
            diagnostics: report.diagnostics,
        }
    }

    fn rejected(
        command_id: EditorOperationPath,
        plugin_id: String,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            command_id,
            plugin_id,
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            payload: None,
            diagnostics: vec![diagnostic.into()],
        }
    }

    pub fn command_id(&self) -> &EditorOperationPath {
        &self.command_id
    }

    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_deref()
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorCommandExecutorRegistryError {
    MissingCommand {
        command_id: EditorOperationPath,
    },
    MissingExecutionContract {
        command_id: EditorOperationPath,
    },
    InvalidExecutionContract {
        command_id: EditorOperationPath,
        detail: String,
    },
    DuplicateExecutor {
        command_id: EditorOperationPath,
    },
    MissingExecutor {
        command_id: EditorOperationPath,
    },
    CommandNameMismatch {
        command_id: EditorOperationPath,
        binding_name: String,
    },
    NonNativeAction {
        command_id: EditorOperationPath,
    },
    PayloadSchemaMismatch {
        command_id: EditorOperationPath,
        descriptor_schema: Option<String>,
        binding_schema: String,
    },
    OutputBudgetTooSmall {
        command_id: EditorOperationPath,
        descriptor_limit: usize,
        binding_limit: usize,
    },
}

impl std::fmt::Display for EditorCommandExecutorRegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCommand { command_id } => {
                write!(formatter, "editor command {command_id} is not registered")
            }
            Self::MissingExecutionContract { command_id } => {
                write!(
                    formatter,
                    "editor command {command_id} has no execution contract"
                )
            }
            Self::InvalidExecutionContract { command_id, detail } => write!(
                formatter,
                "editor command {command_id} has an invalid execution contract: {detail}"
            ),
            Self::DuplicateExecutor { command_id } => {
                write!(
                    formatter,
                    "editor command {command_id} already has an executor"
                )
            }
            Self::MissingExecutor { command_id } => {
                write!(
                    formatter,
                    "editor command {command_id} has no admitted executor"
                )
            }
            Self::CommandNameMismatch {
                command_id,
                binding_name,
            } => write!(
                formatter,
                "editor command {command_id} cannot use native binding `{binding_name}`"
            ),
            Self::NonNativeAction { command_id } => write!(
                formatter,
                "editor command {command_id} is not a native endpoint"
            ),
            Self::PayloadSchemaMismatch {
                command_id,
                descriptor_schema,
                binding_schema,
            } => write!(
                formatter,
                "editor command {command_id} payload schema {:?} does not match native binding schema `{binding_schema}`",
                descriptor_schema.as_deref()
            ),
            Self::OutputBudgetTooSmall {
                command_id,
                descriptor_limit,
                binding_limit,
            } => write!(
                formatter,
                "editor command {command_id} contract output budget {descriptor_limit} is below native binding limit {binding_limit}"
            ),
        }
    }
}

impl std::error::Error for EditorCommandExecutorRegistryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_receipt_rejects_output_over_contract_budget() {
        let command_id = EditorOperationPath::parse("test.command.receipt").unwrap();
        let report = NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            diagnostics: vec!["callback completed".to_owned()],
            payload: Some(vec![1, 2, 3, 4, 5]),
        };

        let receipt = EditorCommandExecutionReceipt::from_report(
            command_id.clone(),
            "fixture.plugin".to_owned(),
            4,
            report,
        );

        assert_eq!(receipt.command_id(), &command_id);
        assert_eq!(receipt.plugin_id(), "fixture.plugin");
        assert_eq!(receipt.status_code(), ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
        assert_eq!(receipt.payload(), None);
        assert!(receipt
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("output exceeds budget")));
    }

    #[test]
    fn execution_receipt_preserves_bounded_success_payload_and_diagnostics() {
        let report = NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            diagnostics: vec!["callback completed".to_owned()],
            payload: Some(vec![1, 2, 3]),
        };

        let receipt = EditorCommandExecutionReceipt::from_report(
            EditorOperationPath::parse("test.command.receipt_success").unwrap(),
            "fixture.plugin".to_owned(),
            3,
            report,
        );

        assert_eq!(receipt.status_code(), ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(receipt.payload(), Some([1, 2, 3].as_slice()));
        assert_eq!(receipt.diagnostics().len(), 1);
        assert_eq!(receipt.diagnostics()[0], "callback completed");
    }

    #[test]
    fn zero_output_contract_normalizes_empty_callback_payload_to_no_result() {
        let report = NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            diagnostics: Vec::new(),
            payload: Some(Vec::new()),
        };

        let receipt = EditorCommandExecutionReceipt::from_report(
            EditorOperationPath::parse("test.command.no_result").unwrap(),
            "fixture.plugin".to_owned(),
            0,
            report,
        );

        assert_eq!(receipt.status_code(), ZIRCON_NATIVE_PLUGIN_STATUS_OK);
        assert_eq!(receipt.payload(), None);
    }
}
