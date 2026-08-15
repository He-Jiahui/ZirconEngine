//! Capability-gated registration and stable callback handles for VM-owned engine extensions.

mod callback;
mod descriptor;
mod error;
mod registry;

pub use callback::{VmCallbackHandle, VmInterfaceCaller};
pub use descriptor::{
    VmBehaviorNodeRegistration, VmEditorOperationRegistration, VmRpcHandlerRegistration,
    VmSystemRegistration, VmSystemStage,
};
pub use error::VmHostInterfaceError;
pub use registry::VmHostInterfaceRegistry;
pub(crate) use registry::{
    VmHostInterfaceActiveOwner, VmHostInterfaceActiveSnapshot, VmHostInterfaceGenerationSnapshot,
};

/// Capability required for a VM package to register scheduled systems.
pub const VM_SYSTEM_CAPABILITY: &str = "runtime.script.extension.system";
/// Capability required for a VM package to register behavior-tree nodes.
pub const VM_BT_NODE_CAPABILITY: &str = "runtime.script.extension.bt_node";
/// Capability required for a VM package to register RPC handlers.
pub const VM_RPC_HANDLER_CAPABILITY: &str = "runtime.script.extension.rpc_handler";
/// Capability required for a VM package to register editor operations.
pub const VM_EDITOR_OPERATION_CAPABILITY: &str = "runtime.script.extension.editor_operation";
/// Native ZrVM module that exposes the four extension registration channels.
pub const VM_HOST_INTERFACE_MODULE: &str = "zr.zircon.extensions";
