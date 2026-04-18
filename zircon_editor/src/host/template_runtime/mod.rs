pub(crate) mod builtin;
mod harness;
mod host_nodes;
mod model;
mod runtime;
mod slint_adapter;

pub(crate) use builtin::UI_HOST_WINDOW_DOCUMENT_ID;
#[cfg(test)]
pub(crate) use builtin::WORKBENCH_SHELL_DOCUMENT_ID;
pub use harness::{EditorUiCompatibilityHarness, EditorUiCompatibilitySnapshot};
pub use host_nodes::{SlintUiHostBindingProjection, SlintUiHostModel, SlintUiHostNodeProjection};
pub use model::{SlintUiBindingProjection, SlintUiNodeProjection, SlintUiProjection};
pub use runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};
pub use slint_adapter::{
    SlintUiHostAdapter, SlintUiHostComponentKind, SlintUiHostNodeModel, SlintUiHostProjection,
    SlintUiHostRouteProjection, SlintUiHostValue,
};
