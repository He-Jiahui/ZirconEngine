//! Editor-owned boundary for in-process and serialized runtime access.

mod capabilities;
mod contract;
mod detached;
mod error;
mod handle;
mod in_process;
mod session;

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
pub use in_process::InProcessGateway;
pub use session::SessionGateway;

pub type SharedEditorRuntimeGateway = std::sync::Arc<dyn EditorRuntimeGateway>;
