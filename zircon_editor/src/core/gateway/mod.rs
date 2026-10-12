//! Editor-owned boundary for in-process and serialized runtime access.

#[path = "capabilities.rs"]
mod capabilities;
#[path = "contract.rs"]
mod contract;
#[path = "detached.rs"]
mod detached;
#[path = "error.rs"]
mod error;
#[path = "handle.rs"]
mod handle;
#[path = "highlight_set.rs"]
mod highlight_set;
#[path = "in_process.rs"]
mod in_process;
#[path = "operation_route.rs"]
mod operation_route;
#[path = "session/mod.rs"]
mod session;
#[path = "viewport_pick_route.rs"]
mod viewport_pick_route;

pub use capabilities::{
    PluginActivationState, PluginSummaryEntry, RuntimeCapabilities, SessionProfileKind,
};
pub use contract::{
    EditorRuntimeFrame, EditorRuntimeFrameDemand, EditorRuntimeGateway,
    EditorRuntimePluginEventPage,
};
pub use detached::DetachedEditorRuntimeGateway;
pub use error::GatewayError;
pub use handle::EditorRuntimeGatewayHandle;
pub(crate) use handle::{GatewayLease, GatewayOrigin};
pub use highlight_set::EditorRuntimeHighlightSet;
pub use in_process::InProcessGateway;
pub use operation_route::EditorRuntimeOperationRoute;
pub use session::SessionGateway;
pub use viewport_pick_route::EditorRuntimeViewportPickRoute;
pub use zircon_runtime_interface::GatewaySessionIdentity;

pub type SharedEditorRuntimeGateway = std::sync::Arc<dyn EditorRuntimeGateway>;
