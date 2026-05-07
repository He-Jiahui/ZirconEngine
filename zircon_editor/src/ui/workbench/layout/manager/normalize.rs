use crate::ui::workbench::view::ViewRegistry;

use super::super::{
    ActivityDrawerMode, LayoutManager, LayoutNormalizationReport, MainPageId, WorkbenchLayout,
};

impl LayoutManager {
    pub fn normalize(
        &self,
        layout: &mut WorkbenchLayout,
        _registry: &ViewRegistry,
    ) -> LayoutNormalizationReport {
        if layout.activity_windows.is_empty() && !layout.drawers.is_empty() {
            layout.activity_windows =
                super::super::workbench_layout::activity_windows_from_legacy_drawers(
                    layout.drawers.clone(),
                );
        }

        let mut removed_missing_active_tabs = 0;
        for activity_window in layout.activity_windows.values_mut() {
            activity_window.activity_drawers =
                super::super::workbench_layout::canonical_activity_drawers(std::mem::take(
                    &mut activity_window.activity_drawers,
                ));
            for drawer in activity_window.activity_drawers.values_mut() {
                normalize_drawer(drawer, &mut removed_missing_active_tabs);
            }
        }

        if !layout
            .main_pages
            .iter()
            .any(|page| page.id() == &layout.active_main_page)
        {
            layout.active_main_page = layout
                .main_pages
                .first()
                .map(|page| page.id().clone())
                .unwrap_or_else(MainPageId::workbench);
        }

        layout.sync_legacy_drawers_from_active_activity_window();

        LayoutNormalizationReport {
            placeholders: Vec::new(),
            removed_missing_active_tabs,
        }
    }
}

fn normalize_drawer(
    drawer: &mut super::super::ActivityDrawerLayout,
    removed_missing_active_tabs: &mut usize,
) {
    if let Some(active) = drawer.tab_stack.active_tab.clone() {
        if !drawer.tab_stack.tabs.contains(&active) {
            drawer.tab_stack.active_tab = drawer.tab_stack.tabs.first().cloned();
            *removed_missing_active_tabs += 1;
        }
    }
    if let Some(active) = drawer.active_view.clone() {
        if !drawer.tab_stack.tabs.contains(&active) {
            drawer.active_view = drawer.tab_stack.active_tab.clone();
            *removed_missing_active_tabs += 1;
        }
    }
    if drawer.mode == ActivityDrawerMode::Collapsed {
        drawer.tab_stack.active_tab = None;
        drawer.active_view = None;
    }
}
