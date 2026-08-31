use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry, EditorKeymap};
use crate::core::extension::{CapabilitySet, ContributionSnapshot, DocumentToolkitDescriptor};
use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::startup::EditorSessionMode;

use super::super::document_tabs::document_tabs_for_page;
use super::super::drawer_ring_model::DrawerRingModel;
use super::super::main_host_strip::{active_page_snapshot, host_strip_model};
use super::super::menu::default_menu_bar_with_sources;
use super::super::workbench_view_model::WorkbenchViewModel;
use super::document::build_document_workspace;
use super::floating_windows::build_floating_windows;
use super::status_bar::build_status_bar;
use super::tool_windows::build_tool_windows;

impl WorkbenchViewModel {
    #[cfg(test)]
    pub fn build(command_registry: &EditorCommandRegistry, chrome: &EditorChromeSnapshot) -> Self {
        let context = super::super::host_command_eval_ctx_for_test(
            chrome,
            crate::core::play::PlayModeKind::Edit,
            &[],
        );
        Self::build_with_context(command_registry, chrome, &context)
    }

    pub fn build_with_context(
        command_registry: &EditorCommandRegistry,
        chrome: &EditorChromeSnapshot,
        context: &CommandEvalCtx,
    ) -> Self {
        let keymap = EditorKeymap::default_workbench();
        let i18n = EditorI18nService::default();
        let locale = i18n.active_locale();
        Self::build_with_contributions_and_context(
            command_registry,
            &keymap,
            &i18n,
            &locale,
            chrome,
            &ContributionSnapshot::default(),
            &CapabilitySet::default(),
            None,
            context,
        )
    }

    #[cfg(test)]
    pub fn build_with_contributions(
        command_registry: &EditorCommandRegistry,
        chrome: &EditorChromeSnapshot,
        contributions: &ContributionSnapshot,
    ) -> Self {
        let keymap = EditorKeymap::default_workbench();
        let i18n = EditorI18nService::default();
        let locale = i18n.active_locale();
        let context = super::super::host_command_eval_ctx_for_test(
            chrome,
            crate::core::play::PlayModeKind::Edit,
            &[],
        );
        Self::build_with_contributions_and_context(
            command_registry,
            &keymap,
            &i18n,
            &locale,
            chrome,
            contributions,
            &CapabilitySet::default(),
            None,
            &context,
        )
    }

    #[cfg(test)]
    pub fn build_with_contributions_and_capabilities(
        command_registry: &EditorCommandRegistry,
        chrome: &EditorChromeSnapshot,
        contributions: &ContributionSnapshot,
        enabled_capabilities: &[String],
    ) -> Self {
        let keymap = EditorKeymap::default_workbench();
        let i18n = EditorI18nService::default();
        let locale = i18n.active_locale();
        let context = super::super::host_command_eval_ctx_for_test(
            chrome,
            crate::core::play::PlayModeKind::Edit,
            enabled_capabilities,
        );
        Self::build_with_contributions_and_context(
            command_registry,
            &keymap,
            &i18n,
            &locale,
            chrome,
            contributions,
            &enabled_capabilities.iter().cloned().collect(),
            None,
            &context,
        )
    }

    pub fn build_with_contributions_and_context(
        command_registry: &EditorCommandRegistry,
        keymap: &EditorKeymap,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        chrome: &EditorChromeSnapshot,
        contributions: &ContributionSnapshot,
        capabilities: &CapabilitySet,
        focused_toolkit: Option<&DocumentToolkitDescriptor>,
        context: &CommandEvalCtx,
    ) -> Self {
        let active_page = active_page_snapshot(chrome);
        let host_strip = host_strip_model(&active_page, chrome);
        let drawer_visible = !chrome.workbench.drawers.is_empty();
        let document_tabs = document_tabs_for_page(&active_page, chrome);

        Self {
            is_playing: chrome.session_mode == EditorSessionMode::Playing,
            asset_creation_menu: chrome.asset_browser.creation_menu.clone(),
            keymap: keymap.clone(),
            menu_bar: default_menu_bar_with_sources(
                command_registry,
                keymap,
                i18n,
                locale,
                contributions,
                capabilities,
                focused_toolkit,
                context,
            ),
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
