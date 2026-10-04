//! Capability validation result for an editor-plugin catalog.

use zircon_runtime_interface::RegistrationDiagnostic;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// 一代插件目录的能力校验诊断集合；警告仍返回给调用者，只有错误诊断阻止成功判定。
pub struct EditorCapabilityReport {
    pub diagnostics: Vec<RegistrationDiagnostic>,
}

impl EditorCapabilityReport {
    pub fn is_success(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(RegistrationDiagnostic::is_error)
    }
}
