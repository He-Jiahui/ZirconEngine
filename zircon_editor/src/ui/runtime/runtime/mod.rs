#[path = "build_session.rs"]
mod build_session;
#[path = "compiled_template_action.rs"]
mod compiled_template_action;
#[path = "pane_payload_projection.rs"]
mod pane_payload_projection;
#[path = "plugin_documents.rs"]
mod plugin_documents;
#[path = "projection.rs"]
mod projection;
#[path = "runtime_host.rs"]
mod runtime_host;
#[path = "action_registry.rs"]
mod template_action_registry;
#[path = "action_slot.rs"]
mod template_action_slot;

pub(crate) use build_session::collect_builtin_template_imports;
pub use plugin_documents::EditorPluginV2DocumentSourceError;
pub use runtime_host::{EditorUiHostRuntime, EditorUiHostRuntimeError};
