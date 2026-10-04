//! 插件域拥有接口的导入绑定、冻结导出表和可失效弱句柄；框架域只持中立的 slot、状态与调用契约。
//! 注册表合并完成后绑定导入，运行时生命周期操作只改变同一冻结表的代际与 provider。
mod import;
mod table;
mod weak;

pub use import::BridgeImport;
pub(crate) use import::InterfaceImport;
pub use table::{
    BridgeDiagnosticsMatrix, BridgeEntry, BridgeInterfaceSnapshot, BridgeOwnerTransitionReport,
    BridgeTableDiagnosticsSummary, FrozenBridgeTable, InterfaceExport,
};
pub use weak::{BridgeGuard, WeakBridge};
