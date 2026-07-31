use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::plugin::PluginModuleKind;

pub(super) type PluginLoadResult<T> = std::result::Result<T, PluginLoadError>;

pub(super) const DESCRIPTOR_EXPORT_HINT: &str = "ensure the dist crate exports native_dist_runtime_plugin_v3!, native_dist_editor_plugin_v3!, or native_dist_plugin_v3!";
pub(super) const ENTRY_EXPORT_HINT: &str =
    "ensure the dist crate macro and plugin module declaration export the same entry symbol";
pub(super) const ABI_CONTRACT_HINT: &str =
    "rebuild the plugin dist crate with the current Zircon plugin SDK";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginLoadStage {
    LibraryOpen,
    DescriptorProbe,
    RuntimeEntry,
    EditorEntry,
}

impl From<PluginModuleKind> for PluginLoadStage {
    fn from(module_kind: PluginModuleKind) -> Self {
        match module_kind {
            PluginModuleKind::Runtime => Self::RuntimeEntry,
            PluginModuleKind::Editor => Self::EditorEntry,
            PluginModuleKind::Native | PluginModuleKind::Vm => Self::DescriptorProbe,
        }
    }
}

impl fmt::Display for PluginLoadStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LibraryOpen => "library-open",
            Self::DescriptorProbe => "descriptor-probe",
            Self::RuntimeEntry => "runtime-entry",
            Self::EditorEntry => "editor-entry",
        })
    }
}

#[derive(Debug, Error)]
pub enum PluginLoadError {
    #[error(
        "native plugin {plugin_id} {stage} failed at {}: expected {expected}, actual {actual}; {hint}",
        path.display()
    )]
    MissingSymbol {
        plugin_id: String,
        stage: PluginLoadStage,
        expected: String,
        actual: String,
        path: PathBuf,
        hint: &'static str,
        #[source]
        source: libloading::Error,
    },
    #[error(
        "native plugin {plugin_id} {stage} contract {contract} mismatch at {}: expected {expected}, actual {actual}; {hint}",
        path.display()
    )]
    ContractMismatch {
        plugin_id: String,
        stage: PluginLoadStage,
        contract: &'static str,
        expected: String,
        actual: String,
        path: PathBuf,
        hint: &'static str,
    },
    #[error(
        "native plugin {plugin_id} {stage} capability negotiation failed at {}: expected {expected}, actual {actual}; missing_required={missing_required:?}, denied={denied:?}, diagnostics={diagnostics:?}; {hint}",
        path.display()
    )]
    CapabilityNegotiation {
        plugin_id: String,
        stage: PluginLoadStage,
        expected: String,
        actual: String,
        missing_required: Vec<String>,
        denied: Vec<String>,
        diagnostics: Vec<String>,
        path: PathBuf,
        hint: &'static str,
    },
    #[error(
        "native plugin {plugin_id} {stage} payload {field} is invalid at {}: expected {expected}, actual {actual}; {hint}",
        path.display()
    )]
    InvalidPayload {
        plugin_id: String,
        stage: PluginLoadStage,
        field: &'static str,
        expected: String,
        actual: String,
        path: PathBuf,
        hint: &'static str,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
    #[error(
        "native plugin {plugin_id} {stage} pointer mismatch at {}: expected {expected}, actual {actual}; {hint}",
        path.display()
    )]
    NullPointer {
        plugin_id: String,
        stage: PluginLoadStage,
        expected: String,
        actual: String,
        path: PathBuf,
        hint: &'static str,
    },
    #[error(
        "native plugin {plugin_id} {stage} artifact mismatch at {}: expected {expected}, actual {actual}; {hint}",
        path.display()
    )]
    MissingArtifact {
        plugin_id: String,
        stage: PluginLoadStage,
        expected: String,
        actual: String,
        path: PathBuf,
        hint: &'static str,
    },
    #[error(
        "native plugin {plugin_id} {stage} failed at {}: expected {expected}, actual {actual}; {hint}",
        path.display()
    )]
    LibraryOpen {
        plugin_id: String,
        stage: PluginLoadStage,
        expected: String,
        actual: String,
        path: PathBuf,
        hint: &'static str,
        #[source]
        source: libloading::Error,
    },
}

impl PluginLoadError {
    pub(super) fn missing_symbol(
        plugin_id: &str,
        stage: PluginLoadStage,
        expected: impl Into<String>,
        path: &Path,
        hint: &'static str,
        source: libloading::Error,
    ) -> Self {
        Self::MissingSymbol {
            plugin_id: plugin_id.to_string(),
            stage,
            expected: expected.into(),
            actual: "symbol not exported".to_string(),
            path: path.to_path_buf(),
            hint,
            source,
        }
    }

    pub(super) fn contract_mismatch(
        plugin_id: &str,
        stage: PluginLoadStage,
        contract: &'static str,
        expected: impl Into<String>,
        actual: impl Into<String>,
        path: &Path,
        hint: &'static str,
    ) -> Self {
        Self::ContractMismatch {
            plugin_id: plugin_id.to_string(),
            stage,
            contract,
            expected: expected.into(),
            actual: actual.into(),
            path: path.to_path_buf(),
            hint,
        }
    }

    pub(super) fn invalid_payload(
        plugin_id: &str,
        stage: PluginLoadStage,
        field: &'static str,
        path: &Path,
        hint: &'static str,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        let actual = source.to_string();
        Self::InvalidPayload {
            plugin_id: plugin_id.to_string(),
            stage,
            field,
            expected: format!("valid {field}"),
            actual,
            path: path.to_path_buf(),
            hint,
            source: Box::new(source),
        }
    }

    pub(super) fn capability_negotiation(
        plugin_id: &str,
        stage: PluginLoadStage,
        missing_required: Vec<String>,
        denied: Vec<String>,
        diagnostics: Vec<String>,
        path: &Path,
    ) -> Self {
        let actual = format!(
            "missing_required={}, denied={}",
            missing_required.len(),
            denied.len()
        );
        Self::CapabilityNegotiation {
            plugin_id: plugin_id.to_string(),
            stage,
            expected: "all required capabilities granted and no denied capabilities granted"
                .to_string(),
            actual,
            missing_required,
            denied,
            diagnostics,
            path: path.to_path_buf(),
            hint: ABI_CONTRACT_HINT,
        }
    }

    pub(super) fn null_pointer(
        plugin_id: &str,
        stage: PluginLoadStage,
        expected: &'static str,
        path: &Path,
        hint: &'static str,
    ) -> Self {
        Self::NullPointer {
            plugin_id: plugin_id.to_string(),
            stage,
            expected: expected.to_string(),
            actual: "null pointer".to_string(),
            path: path.to_path_buf(),
            hint,
        }
    }

    pub(super) fn missing_artifact(plugin_id: &str, path: &Path, expected: &'static str) -> Self {
        Self::MissingArtifact {
            plugin_id: plugin_id.to_string(),
            stage: PluginLoadStage::LibraryOpen,
            expected: expected.to_string(),
            actual: "artifact missing".to_string(),
            path: path.to_path_buf(),
            hint: ABI_CONTRACT_HINT,
        }
    }

    pub(super) fn library_open(plugin_id: &str, path: &Path, source: libloading::Error) -> Self {
        let actual = source.to_string();
        Self::LibraryOpen {
            plugin_id: plugin_id.to_string(),
            stage: PluginLoadStage::LibraryOpen,
            expected: "loadable native dist library".to_string(),
            actual,
            path: path.to_path_buf(),
            hint: ABI_CONTRACT_HINT,
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_mismatch_reports_stage_expected_actual_path_and_hint() {
        let error = PluginLoadError::contract_mismatch(
            "fixture",
            PluginLoadStage::DescriptorProbe,
            "abi_version",
            "3",
            "2",
            Path::new("plugins/fixture/native/fixture.dll"),
            ABI_CONTRACT_HINT,
        );
        let message = error.to_string();

        assert!(message.contains("descriptor-probe"));
        assert!(message.contains("expected 3, actual 2"));
        assert!(message.contains("plugins/fixture/native/fixture.dll"));
        assert!(message.contains(ABI_CONTRACT_HINT));

        match error {
            PluginLoadError::ContractMismatch {
                expected, actual, ..
            } => {
                assert_eq!(expected, "3");
                assert_eq!(actual, "2");
            }
            other => panic!("unexpected plugin load error: {other}"),
        }
    }

    #[test]
    fn invalid_payload_preserves_typed_source() {
        let error = PluginLoadError::invalid_payload(
            "fixture",
            PluginLoadStage::RuntimeEntry,
            "granted_capabilities",
            Path::new("fixture.dll"),
            ABI_CONTRACT_HINT,
            std::ffi::CString::new("invalid\0capability")
                .expect_err("interior NUL must be rejected"),
        );

        assert!(std::error::Error::source(&error).is_some());
        match error {
            PluginLoadError::InvalidPayload {
                expected, actual, ..
            } => {
                assert_eq!(expected, "valid granted_capabilities");
                assert!(actual.contains("nul byte"));
            }
            other => panic!("unexpected plugin load error: {other}"),
        }
    }
}
