mod breadcrumb_model;
mod build;
mod document_tab_model;
mod document_tabs;
mod document_workspace_model;
mod drawer_ring_model;
mod empty_state;
mod floating_window_focus;
mod floating_window_model;
mod host_page_tab_model;
mod main_host_strip;
mod main_host_strip_model;
mod main_host_strip_view_model;
mod menu;
mod pane_action_model;
mod pane_empty_state_model;
mod pane_tab;
mod pane_tab_model;
mod status_bar_model;
mod tool_window_stack_model;
mod workbench_view_model;

pub use breadcrumb_model::BreadcrumbModel;
pub use document_tab_model::DocumentTabModel;
pub use document_workspace_model::DocumentWorkspaceModel;
pub use drawer_ring_model::DrawerRingModel;
pub use floating_window_model::FloatingWindowModel;
pub use host_page_tab_model::HostPageTabModel;
pub use main_host_strip_model::MainHostStripModel;
pub use main_host_strip_view_model::MainHostStripViewModel;
pub use pane_action_model::PaneActionModel;
pub use pane_empty_state_model::PaneEmptyStateModel;
pub use pane_tab_model::PaneTabModel;
pub use status_bar_model::StatusBarModel;
pub use tool_window_stack_model::ToolWindowStackModel;
pub use workbench_view_model::WorkbenchViewModel;

#[cfg(test)]
fn host_command_eval_ctx_for_test(
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    capabilities: &[String],
) -> crate::core::commands::CommandEvalCtx {
    crate::ui::host::command_eval_projection::command_eval_ctx_from_chrome(
        chrome,
        capabilities.iter().cloned(),
    )
}
