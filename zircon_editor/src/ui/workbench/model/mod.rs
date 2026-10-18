#[path = "breadcrumb_model.rs"]
mod breadcrumb_model;
#[path = "build/mod.rs"]
mod build;
#[path = "document_tab_model.rs"]
mod document_tab_model;
#[path = "document_tabs/mod.rs"]
mod document_tabs;
#[path = "document_workspace_model.rs"]
mod document_workspace_model;
#[path = "drawer_ring_model.rs"]
mod drawer_ring_model;
#[path = "empty_state/mod.rs"]
mod empty_state;
#[path = "floating_window_focus.rs"]
mod floating_window_focus;
#[path = "floating_window_model.rs"]
mod floating_window_model;
#[path = "host_page_tab_model.rs"]
mod host_page_tab_model;
#[path = "main_host_strip/mod.rs"]
mod main_host_strip;
#[path = "main_host_strip_model.rs"]
mod main_host_strip_model;
#[path = "main_host_strip_view_model.rs"]
mod main_host_strip_view_model;
#[path = "menu/mod.rs"]
mod menu;
#[path = "pane_action_model.rs"]
mod pane_action_model;
#[path = "pane_empty_state_model.rs"]
mod pane_empty_state_model;
#[path = "pane_tab.rs"]
mod pane_tab;
#[path = "pane_tab_model.rs"]
mod pane_tab_model;
#[path = "status_bar_model.rs"]
mod status_bar_model;
#[path = "tool_window_stack_model.rs"]
mod tool_window_stack_model;
#[path = "workbench_view_model.rs"]
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
pub use workbench_view_model::{RunControlsModel, WorkbenchViewModel};

#[cfg(test)]
/// 测试简化构造仍走宿主评估投影，防止菜单测试另造命令可用性规则。
fn host_command_eval_ctx_for_test(
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    play_mode: crate::core::play::PlayModeKind,
    capabilities: &[String],
) -> crate::core::commands::CommandEvalCtx {
    crate::ui::host::command_eval_projection::command_eval_ctx_from_chrome(
        chrome,
        play_mode,
        capabilities.iter().cloned(),
    )
}
