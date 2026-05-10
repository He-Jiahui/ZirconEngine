pub(crate) mod builtin;
pub(crate) mod component_adapter;
mod harness;
mod host_nodes;
mod model;
mod retained_adapter;
mod runtime;
mod showcase_demo_state;

#[cfg(test)]
pub(crate) use builtin::UI_HOST_WINDOW_DOCUMENT_ID;
pub use harness::{EditorUiCompatibilityHarness, EditorUiCompatibilitySnapshot};
pub use host_nodes::{
    RetainedUiHostBindingProjection, RetainedUiHostModel, RetainedUiHostNodeProjection,
};
pub use model::{RetainedUiBindingProjection, RetainedUiNodeProjection, RetainedUiProjection};
pub use retained_adapter::{
    RetainedUiHostAdapter, RetainedUiHostComponentKind, RetainedUiHostNodeModel,
    RetainedUiHostProjection, RetainedUiHostRouteProjection, RetainedUiHostValue,
};
pub(crate) use runtime::collect_builtin_template_imports;
pub use runtime::{EditorUiHostRuntime, EditorUiHostRuntimeError};
pub(crate) use showcase_demo_state::{
    UiComponentShowcaseDemoError, UiComponentShowcaseDemoEventInput, UiComponentShowcaseDemoState,
    SHOWCASE_DOCUMENT_ID,
};
