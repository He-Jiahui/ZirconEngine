//! ABI 调用的原始返回码与同步借用的诊断视图。
//!
//! Host 先检查状态，再按诊断预算读取；输出分配的释放责任由其载体和入口协议另行规定。
//! 诊断不携带释放权，Runtime 同线程暂存区可能在后续错误调用时被覆盖。

use crate::buffer::ZrByteSlice;

/// C ABI 返回码；未知数值按 Error 处理，避免新版本返回码被旧 Host 当成成功。
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrStatusCode {
    /// 调用成功；是否产生输出由对应入口协议决定。
    Ok = 0,
    /// 未细分的失败，也是未知原始返回码的保守解释。
    Error = 1,
    /// 提供方拒绝了载体或协议版本。
    UnsupportedVersion = 2,
    /// 参数形状、标识或请求内容无效。
    InvalidArgument = 3,
    /// 会话、分配、视口或票据等所请求对象不存在。
    NotFound = 4,
    /// 当前上下文没有请求能力的授权。
    CapabilityDenied = 5,
    /// FFI 包装器捕获了 Rust 展开式 panic；此码不证明外来指针可读。
    Panic = 6,
    /// 所请求的桥接入口未启用。
    BridgeNotEnabled = 7,
    /// 请求或结果超过该入口的资源预算。
    LimitExceeded = 8,
}

impl ZrStatusCode {
    /// 解释参数 `value`；只接受已知数值，未知值返回 [`Self::Error`]。
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::Error,
            2 => Self::UnsupportedVersion,
            3 => Self::InvalidArgument,
            4 => Self::NotFound,
            5 => Self::CapabilityDenied,
            6 => Self::Panic,
            7 => Self::BridgeNotEnabled,
            8 => Self::LimitExceeded,
            _ => Self::Error,
        }
    }

    /// 返回 ABI 固定使用的原始整数，不附带诊断或输出所有权。
    pub const fn as_raw(self) -> u32 {
        self as u32
    }
}

/// 一次 ABI 调用的状态及借用型诊断字节；接收方不拥有 diagnostics。
/// Runtime 的诊断可能来自同线程暂存区，Host 应在下一次调用前限额读取或复制。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrStatus {
    /// ABI 原始数值；接收方用 [`Self::status_code`] 保守解释未知值。
    pub code: u32,
    /// 提供方借出的诊断字节；须在提供方有效期内按预算读取，不能自行释放。
    pub diagnostics: ZrByteSlice,
}

impl ZrStatus {
    /// 返回没有诊断负载的成功状态，供正常 ABI 返回路径使用。
    pub const fn ok() -> Self {
        Self::new(ZrStatusCode::Ok, ZrByteSlice::empty())
    }

    /// 组合状态与诊断借用；参数 `diagnostics` 不被复制，提供方仍负责其有效期。
    pub const fn new(code: ZrStatusCode, diagnostics: ZrByteSlice) -> Self {
        Self {
            code: code.as_raw(),
            diagnostics,
        }
    }

    /// 把原始状态码转换为已知枚举；不读取诊断，也不消费任何输出分配。
    pub const fn status_code(self) -> ZrStatusCode {
        ZrStatusCode::from_raw(self.code)
    }

    /// 仅原始零码表示成功；所有已知失败及未知码均返回 false。
    pub const fn is_ok(self) -> bool {
        self.code == ZrStatusCode::Ok.as_raw()
    }
}
