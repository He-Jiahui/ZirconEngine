use super::{FrameDiagnostics, RuntimePhysicsBackendDiagnostics};

/// 物理服务的只读工具视图；可选后端的具体状态与服务解析失败分开表达。
/// 采集器可能因编译特性关闭或管理器缺失而构造 unavailable，面板据此展示原因。
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimePhysicsDiagnostics {
    pub available: bool,
    pub backend_name: Option<String>,
    pub backend_status: Option<RuntimePhysicsBackendDiagnostics>,
    pub fixed_hz: Option<u32>,
    pub error: Option<String>,
}

impl RuntimePhysicsDiagnostics {
    /// 物理域不可采集时保留可展示的失败原因，不伪造后端和固定步率。
    pub fn unavailable(error: impl Into<String>) -> Self {
        Self {
            available: false,
            backend_name: None,
            backend_status: None,
            fixed_hz: None,
            error: Some(error.into()),
        }
    }
}

impl FrameDiagnostics for RuntimePhysicsDiagnostics {
    fn diagnostics_domain(&self) -> &'static str {
        "physics"
    }

    fn diagnostics_available(&self) -> bool {
        self.available
    }

    fn diagnostics_error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
