//! Editor-owned boundary for in-process and serialized runtime access.

mod contract;
mod detached;
mod error;
mod handle;

pub use contract::EditorRuntimeGateway;
pub use detached::DetachedEditorRuntimeGateway;
pub use error::GatewayError;
pub use handle::EditorRuntimeGatewayHandle;

pub type SharedEditorRuntimeGateway = std::sync::Arc<dyn EditorRuntimeGateway>;
