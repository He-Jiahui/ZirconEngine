use crate::core::editor_extension::EditorExtensionRegistry;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, MainPageSnapshot};

use super::super::document_tabs::document_tabs_for_page;
use super::super::drawer_ring_model::DrawerRingModel;
use super::super::main_host_strip::{active_page_snapshot, host_strip_model};
use super::super::menu::default_menu_bar_with_extensions;
use super::super::workbench_view_model::WorkbenchViewModel;
use super::document::build_document_workspace;
use super::floating_windows::build_floating_windows;
use super::status_bar::build_status_bar;
use super::tool_windows::build_tool_windows;

impl WorkbenchViewModel {
    pub fn build(chrome: &EditorChromeSnapshot) -> Self {
        Self::build_with_extensions(chrome, &[])
    }

    pub fn build_with_extensions(
        chrome: &EditorChromeSnapshot,
        extensions: &[EditorExtensionRegistry],
    ) -> Self {
        let active_page = active_page_snapshot(chrome);
        let host_strip = host_strip_model(&active_page, chrome);
        let drawer_visible = matches!(active_page, MainPageSnapshot::Workbench { .. });
        let document_tabs = document_tabs_for_page(&active_page, chrome);

        Self {
            menu_bar: default_menu_bar_with_extensions(chrome, extensions),
            host_strip,
            drawer_ring: DrawerRingModel {
                visible: drawer_visible,
                drawers: chrome.workbench.drawers.clone(),
            },
            tool_windows: build_tool_windows(chrome),
            document_tabs,
            floating_windows: build_floating_windows(chrome),
            document: build_document_workspace(active_page),
            status_bar: build_status_bar(chrome),
        }
    }
}
