#[path = "activity_drawer_snapshot.rs"]
mod activity_drawer_snapshot;
#[path = "descriptor_content_kind.rs"]
mod descriptor_content_kind;
#[path = "document_workspace_snapshot.rs"]
mod document_workspace_snapshot;
#[path = "floating_window_snapshot.rs"]
mod floating_window_snapshot;
#[path = "main_page_snapshot.rs"]
mod main_page_snapshot;
#[path = "placeholder_view.rs"]
mod placeholder_view;
#[path = "resolve_document_workspace.rs"]
mod resolve_document_workspace;
#[path = "resolve_view_tab.rs"]
mod resolve_view_tab;
#[path = "view_content_kind.rs"]
mod view_content_kind;
#[path = "view_tab_snapshot.rs"]
mod view_tab_snapshot;
#[path = "workbench_snapshot.rs"]
mod workbench_snapshot;

pub use activity_drawer_snapshot::ActivityDrawerSnapshot;
pub use document_workspace_snapshot::DocumentWorkspaceSnapshot;
pub use floating_window_snapshot::FloatingWindowSnapshot;
pub use main_page_snapshot::MainPageSnapshot;
pub(crate) use resolve_document_workspace::resolve_document_workspace;
pub(crate) use resolve_view_tab::resolve_view_tab;
pub use view_content_kind::ViewContentKind;
pub use view_tab_snapshot::ViewTabSnapshot;
pub use workbench_snapshot::WorkbenchSnapshot;
