//! Errors raised while consuming runtime-owned output.
//!
//! 本模块定义 Host 消费 Runtime 拥有输出时的错误分类和状态诊断转换。
//!
//! `RuntimeCall` 表示 ABI 调用本身返回失败；`ProtocolViolation` 表示所有权、预算、解码或
//! 清理不符合共享契约。`from_status` 只在 Runtime 仍由调用方保持存活时读取诊断字节。

use zircon_runtime_interface::{
    ZrStatus, ZrStatusCode, ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 外来输出消费失败的两类可观察原因。
pub enum RuntimeForeignOutputErrorKind {
    /// Runtime ABI 返回了失败状态。
    RuntimeCall,
    /// Host 发现输出或生命周期违反了协议。
    ProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 带分类的 Host 输出错误；消息会传递给 App/Editor 的上层错误类型。
pub struct RuntimeForeignOutputError {
    /// 用于上层决定 Runtime 调用错误还是协议错误的分类。
    kind: RuntimeForeignOutputErrorKind,
    /// 面向日志和调用方的诊断文本。
    message: String,
}

impl RuntimeForeignOutputError {
    /// 创建 Runtime 调用失败错误。
    pub fn runtime_call(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeForeignOutputErrorKind::RuntimeCall,
            message: message.into(),
        }
    }

    /// 创建协议违规错误，并由共享状态决定是否熔断会话。
    pub fn protocol_violation(message: impl Into<String>) -> Self {
        Self {
            kind: RuntimeForeignOutputErrorKind::ProtocolViolation,
            message: message.into(),
        }
    }

    /// 返回错误分类。
    pub const fn kind(&self) -> RuntimeForeignOutputErrorKind {
        self.kind
    }

    /// 将释放失败追加到原错误，并将结果归类为协议违规。
    pub fn with_cleanup_failure(self, cleanup: &impl std::fmt::Display) -> Self {
        Self::protocol_violation(format!("{}; cleanup also failed: {cleanup}", self.message))
    }

    /// 将非成功 ABI 状态和有界诊断字节转换为 Runtime 调用错误。
    ///
    /// # Safety
    ///
    /// On failure, `status.diagnostics` must point to synchronously readable bytes supplied by the
    /// still-loaded runtime provider that produced this status.
    pub(super) unsafe fn from_status(status: ZrStatus, operation: &'static str) -> Option<Self> {
        if status.is_ok() {
            return None;
        }
        // SAFETY: 调用者保证提供方诊断在本次同步读取期间有效；checked_slice 在构造借用前检查指针形状、地址空间和字节上限。
        let diagnostics = match unsafe {
            status
                .diagnostics
                .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
        } {
            Ok(diagnostics) => diagnostics,
            Err(error) => {
                return Some(Self::protocol_violation(format!(
                    "failed to {operation}: runtime returned invalid status diagnostics: {error:?}"
                )));
            }
        };
        let diagnostics = String::from_utf8_lossy(diagnostics);
        let code = match status.status_code() {
            ZrStatusCode::Ok => "ok",
            ZrStatusCode::Error => "error",
            ZrStatusCode::UnsupportedVersion => "unsupported-version",
            ZrStatusCode::InvalidArgument => "invalid-argument",
            ZrStatusCode::NotFound => "not-found",
            ZrStatusCode::CapabilityDenied => "capability-denied",
            ZrStatusCode::BridgeNotEnabled => "bridge-not-enabled",
            ZrStatusCode::Panic => "panic",
            ZrStatusCode::LimitExceeded => "limit-exceeded",
        };
        Some(Self::runtime_call(format!(
            "failed to {operation}: {code}: {diagnostics}"
        )))
    }
}

impl std::fmt::Display for RuntimeForeignOutputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuntimeForeignOutputError {}
