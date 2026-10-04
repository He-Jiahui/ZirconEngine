//! 未启用 physics-contracts 时保留统一诊断形状，但明确返回不可用原因而不触碰 manager registry。
use crate::core::diagnostics::RuntimePhysicsDiagnostics;
use crate::core::CoreHandle;

// cfg 选择在编译期完成；该分支的 CoreHandle 仅用于保持 collector 形状一致。
pub(super) fn collect(_core: &CoreHandle) -> RuntimePhysicsDiagnostics {
    RuntimePhysicsDiagnostics::unavailable("physics contracts are not compiled")
}
