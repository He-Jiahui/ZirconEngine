//! 内部 ABI 载体版本与冻结的 Runtime 函数表版本。
//!
//! 各 DTO 保留自己的版本编号；函数表版本来自 InterfaceSpec，二者不能互相代替。
//! 当前边界仍按 crate 根模块声明的内部 lockstep 策略演进。

/// First stable runtime dynamic-library ABI version.
/// 未改布局的 DTO 与 [`crate::runtime_api::ZrHostApiV1`] 继续使用的历史 V1 编号。
pub const ZIRCON_RUNTIME_ABI_VERSION_V1: u32 = 1;

/// ABI version for the changed V2 runtime session config carrier.
/// V2 载体版本；当前 [`crate::runtime_api::ZrRuntimeFrameV2`] 也显式携带此编号。
pub const ZIRCON_RUNTIME_ABI_VERSION_V2: u32 = 2;

/// ABI version for the V3 runtime session configuration carrier.
/// [`crate::runtime_api::ZrRuntimeSessionConfigV3`] 的载体版本，由创建会话入口核对。
pub const ZIRCON_RUNTIME_ABI_VERSION_V3: u32 = 3;

/// Current runtime dynamic-library function-table version, generated from the frozen InterfaceSpec.
/// 冻结 [`crate::runtime_api::ZrRuntimeApiV8`] 的函数表版本；重导出生成目录的唯一来源。
pub use crate::runtime_build_set::ZIRCON_RUNTIME_API_VERSION_V8;
