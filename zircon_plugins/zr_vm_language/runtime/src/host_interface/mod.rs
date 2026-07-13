//! Rust-side access to the four capability-gated VM extension channels.

mod bt_node;
mod editor_op;
mod rpc;
mod system;

pub use bt_node::register_bt_node;
pub use editor_op::register_editor_operation;
pub use rpc::register_rpc_handler;
pub use system::register_system;
pub use zircon_runtime::script::{
    VmBehaviorNodeRegistration, VmCallbackHandle, VmEditorOperationRegistration,
    VmHostInterfaceError, VmHostInterfaceRegistry, VmInterfaceCaller, VmRpcHandlerRegistration,
    VmSystemRegistration, VmSystemStage, VM_BT_NODE_CAPABILITY, VM_EDITOR_OPERATION_CAPABILITY,
    VM_RPC_HANDLER_CAPABILITY, VM_SYSTEM_CAPABILITY,
};
