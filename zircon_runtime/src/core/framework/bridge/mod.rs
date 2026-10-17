//! Neutral contracts for plugin-to-plugin interface calls.
//! 框架层提供接口身份、表内槽位、可见状态、入口诊断与强引用句柄的中立契约。
//! 插件域负责导出注册、导入绑定和生命周期发布；表视图供 VM 等调用入口查询状态和记录准入事件。

#[path = "contract.rs"]
mod contract;
#[path = "diagnostics.rs"]
mod diagnostics;
#[path = "interface_slot.rs"]
mod interface_slot;
#[path = "strong.rs"]
mod strong;

pub use contract::{
    BridgeError, BridgeInterfaceStatus, BridgeInvocationTable, BridgeOwnerTransitionMode,
    PluginInterface,
};
pub(crate) use diagnostics::BridgeDiagnostics;
pub use diagnostics::BridgeDiagnosticsSnapshot;
pub use interface_slot::InterfaceSlot;
pub use strong::StrongBridge;
