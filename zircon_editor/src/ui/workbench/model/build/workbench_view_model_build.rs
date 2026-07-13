use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
use crate::core::editor_extension::EditorExtensionRegistry;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

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
    #[cfg(test)]
    pub fn build(command_registry: &EditorCommandRegistry, chrome: &EditorChromeSnapshot) -> Self {
        let context = super::super::host_command_eval_ctx_for_test(chrome, &[]);
        Self::build_with_context(command_registry, chrome, &context)
    }

    pub fn build_with_context(
        command_registry: &EditorCommandRegistry,
        chrome: &EditorChromeSnapshot,
        context: &CommandEvalCtx,
    ) -> Self {
        Self::build_with_extensions_and_context(command_registry, chrome, &[], context)
    }

    #[cfg(test)]
    pub fn build_with_extensions(
        command_registry: &EditorCommandRegistry,
        chrome: &EditorChromeSnapshot,
        extensions: &[EditorExtensionRegistry],
    ) -> Self {
        let context = super::super::host_command_eval_ctx_for_test(chrome, &[]);
        Self::build_with_extensions_and_context(command_registry, chrome, extensions, &context)
    }

    #[cfg(test)]
    pub fn build_with_extensions_and_capabilities(
        command_registry: &EditorCommandRegistry,
        chrome: &EditorChromeSnapshot,
        extensions: &[EditorExtensionRegistry],
        enabled_capabilities: &[String],
    ) -> Self {
        let context = super::super::host_command_eval_ctx_for_test(chrome, enabled_capabilities);
        Self::build_with_extensions_and_context(command_registry, chrome, extensions, &context)
    }

    pub fn build_with_extensions_and_context(
        command_registry: &EditorCommandRegistry,
        chrome: &EditorChromeSnapshot,
        extensions: &[EditorExtensionRegistry],
        context: &CommandEvalCtx,
    ) -> Self {
        let active_page = active_page_snapshot(chrome);
        let host_strip = host_strip_model(&active_page, chrome);
        let drawer_visible = !chrome.workbench.drawers.is_empty();
        let document_tabs = document_tabs_for_page(&active_page, chrome);

        Self {
            menu_bar: default_menu_bar_with_extensions(command_registry, extensions, context),
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
