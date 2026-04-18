use crate::ViewRegistry;

use super::super::{
    ActivityDrawerLayout, ActivityDrawerSlot, LayoutManager, LayoutNormalizationReport, MainPageId,
    WorkbenchLayout,
};

impl LayoutManager {
    pub fn normalize(
        &self,
        layout: &mut WorkbenchLayout,
        _registry: &ViewRegistry,
    ) -> LayoutNormalizationReport {
        for slot in ActivityDrawerSlot::ALL {
            layout
                .drawers
                .entry(slot)
                .or_insert_with(|| ActivityDrawerLayout::new(slot));
        }

        let mut removed_missing_active_tabs = 0;
        for drawer in layout.drawers.values_mut() {
            if let Some(active) = drawer.tab_stack.active_tab.clone() {
                if !drawer.tab_stack.tabs.contains(&active) {
                    drawer.tab_stack.active_tab = drawer.tab_stack.tabs.first().cloned();
                    removed_missing_active_tabs += 1;
                }
            }
            if let Some(active) = drawer.active_view.clone() {
                if !drawer.tab_stack.tabs.contains(&active) {
                    drawer.active_view = drawer.tab_stack.active_tab.clone();
                    removed_missing_active_tabs += 1;
                }
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

        LayoutNormalizationReport {
            placeholders: Vec::new(),
            removed_missing_active_tabs,
        }
    }
}
