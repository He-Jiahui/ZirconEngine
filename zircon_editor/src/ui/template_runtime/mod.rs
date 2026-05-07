pub(crate) mod builtin;
pub(crate) mod component_adapter;
mod harness;
mod host_nodes;
mod model;
mod runtime;
mod showcase_demo_state;
mod slint_adapter;

#[cfg(test)]
pub(crate) use builtin::UI_HOST_WINDOW_DOCUMENT_ID;
pub use harness::{EditorUiCompatibilityHarness, EditorUiCompatibilitySnapshot};
pub use host_nodes::{SlintUiHostBindingProjection, SlintUiHostModel, SlintUiHostNodeProjection};
pub use model::{SlintUiBindingProjection, SlintUiNodeProjection, SlintUiProjection};
pub(crate) use runtime::collect_builtin_template_imports;
pub use runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};
pub(crate) use showcase_demo_state::{
    UiComponentShowcaseDemoError, UiComponentShowcaseDemoEventInput, UiComponentShowcaseDemoState,
    SHOWCASE_DOCUMENT_ID,
};
pub use slint_adapter::{
    SlintUiHostAdapter, SlintUiHostComponentKind, SlintUiHostNodeModel, SlintUiHostProjection,
    SlintUiHostRouteProjection, SlintUiHostValue,
};
