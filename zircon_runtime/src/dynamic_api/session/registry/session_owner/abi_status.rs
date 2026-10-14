//! 在诊断提供方的线程内复制 ABI status，避免把 TLS 指针作为跨线程结果传递。
//!
//! owner 动作结束时捕获，注册表在调用线程重建 ABI 诊断；本模块不拥有输出 allocation。

use zircon_runtime_interface::{
    ZrStatus, ZrStatusCode, ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
};

use crate::dynamic_api::session::status::error_status;

/// An owned message replaces the ABI diagnostic pointer before leaving its TLS owner.
/// 保存已解释的状态码与独立诊断字节，寿命不再依赖 owner 的下一次错误调用。
#[derive(Clone, Debug)]
pub(in crate::dynamic_api::session) struct OwnedSessionStatus {
    /// status_code 已把未知原始码保守解释为 Error。
    code: ZrStatusCode,
    /// 在 capture 返回前从提供方借用复制，不保留任何 ABI 指针。
    diagnostics: String,
}

impl OwnedSessionStatus {
    /// # Safety
    /// The status must come from a completed runtime call on the current thread;
    /// its diagnostics must remain readable until this function returns.
    ///
    /// # 参数与返回值
    /// status 在原提供线程同步读取，按诊断预算复制并以有损 UTF-8 转为 String。
    /// 形状或长度无效时使用固定诊断，状态码仍按 status_code 解释。
    ///
    /// # 约束与生命周期
    /// 非空诊断须为单个分配内的已初始化只读字节，且在复制期间保持有效；
    /// checked_slice 只检查载体和预算，不验证任意地址是否可读。
    ///
    /// # 参考
    /// `runtime.rs::RuntimeSessionOwner::dispatch/dispatch_scoped`；`session/status.rs::error_status`。
    pub(in crate::dynamic_api::session) unsafe fn capture(status: ZrStatus) -> Self {
        // SAFETY: capture 的调用方保证诊断借用前提；本作用域立即复制，不延长原字节寿命。
        let diagnostics = unsafe {
            status
                .diagnostics
                .checked_slice(ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1)
        }
        .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        .unwrap_or_else(|_| "runtime produced invalid status diagnostics".to_owned());
        Self {
            code: status.status_code(),
            diagnostics,
        }
    }

    /// 在接收线程重建 status；成功不带诊断，失败借用本线程 error_status 的 TLS 存储。
    /// 调用方应在下一次覆盖该 TLS 的错误调用前读取返回诊断。
    pub(in crate::dynamic_api::session) fn into_abi(self) -> ZrStatus {
        if self.code == ZrStatusCode::Ok {
            return ZrStatus::ok();
        }
        let mut status = error_status(self.diagnostics);
        status.code = self.code.as_raw();
        status
    }

    /// 把已有诊断移交给注册表的内部错误；此路径不创建 ABI 借用且不保留状态码。
    pub(in crate::dynamic_api::session) fn into_message(self) -> String {
        self.diagnostics
    }
}

#[cfg(test)]
#[path = "../tests/abi_status.rs"]
mod tests;
