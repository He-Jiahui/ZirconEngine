use crate::layout::WorkspaceTarget;
use crate::snapshot::{EditorChromeSnapshot, MainPageSnapshot};

use super::document_tabs::{document_tabs_for_page, workspace_tabs};
use super::document_workspace_model::DocumentWorkspaceModel;
use super::drawer_ring_model::DrawerRingModel;
use super::floating_window_model::FloatingWindowModel;
use super::main_host_strip::{active_page_snapshot, host_strip_model};
use super::menu::default_menu_bar;
use super::pane_tab::pane_tab_model;
use super::status_bar_model::StatusBarModel;
use super::tool_window_stack_model::ToolWindowStackModel;
use super::workbench_view_model::WorkbenchViewModel;

impl WorkbenchViewModel {
    pub fn build(chrome: &EditorChromeSnapshot) -> Self {
        let active_page = active_page_snapshot(chrome);
        let host_strip = host_strip_model(&active_page, chrome);
        let drawer_visible = matches!(active_page, MainPageSnapshot::Workbench { .. });
        let tool_windows = chrome
            .workbench
            .drawers
            .iter()
            .map(|(slot, drawer)| {
                (
                    *slot,
                    ToolWindowStackModel {
                        slot: *slot,
                        mode: drawer.mode,
                        visible: drawer.visible,
                        active_tab: drawer.active_tab.clone(),
                        tabs: drawer
                            .tabs
                            .iter()
                            .map(|tab| {
                                pane_tab_model(
                                    tab,
                                    drawer.active_tab.as_ref() == Some(&tab.instance_id),
                                    chrome,
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect();
        let document_tabs = document_tabs_for_page(&active_page, chrome);
        let floating_windows = chrome
            .workbench
            .floating_windows
            .iter()
            .map(|window| FloatingWindowModel {
                window_id: window.window_id.clone(),
                title: window.title.clone(),
                focused_view: window.focused_view.clone(),
                tabs: workspace_tabs(
                    &window.workspace,
                    WorkspaceTarget::FloatingWindow(window.window_id.clone()),
                    chrome,
                ),
            })
            .collect();

        Self {
            menu_bar: default_menu_bar(chrome),
            host_strip,
            drawer_ring: DrawerRingModel {
                visible: drawer_visible,
                drawers: chrome.workbench.drawers.clone(),
            },
            tool_windows,
            document_tabs,
            floating_windows,
            document: match active_page {
                MainPageSnapshot::Workbench {
                    id,
                    title,
                    workspace,
                } => DocumentWorkspaceModel::Workbench {
                    page_id: id,
                    title,
                    workspace,
                },
                MainPageSnapshot::Exclusive { id, title, view } => {
                    DocumentWorkspaceModel::Exclusive {
                        page_id: id,
                        title,
                        view,
                    }
                }
            },
            status_bar: StatusBarModel {
                primary_text: chrome.status_line.clone(),
                secondary_text: chrome
                    .inspector
                    .as_ref()
                    .map(|inspector| format!("Selection {}", inspector.name)),
                viewport_label: format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
            },
        }
    }
}
