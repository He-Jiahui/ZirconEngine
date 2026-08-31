//! Editor-owned boundary for in-process and serialized runtime access.

mod capabilities;
mod contract;
mod detached;
mod error;
mod handle;
mod highlight_set;
mod in_process;
mod operation_route;
mod session;
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
