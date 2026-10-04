use crate::ui::workbench::view::ViewRegistry;

use super::super::{
    ActivityDrawerMode, DocumentNode, LayoutManager, LayoutNormalizationReport, MainPageId,
    WorkbenchLayout,
};

const DEFAULT_SPLIT_RATIO: f32 = 0.5;
const MIN_SPLIT_RATIO: f32 = 0.1;
const MAX_SPLIT_RATIO: f32 = 0.9;

impl LayoutManager {
    pub fn normalize(
        &self,
        layout: &mut WorkbenchLayout,
        _registry: &ViewRegistry,
    ) -> LayoutNormalizationReport {
        let mut removed_missing_active_tabs = 0;
        layout.normalize_document_node_ids();
        for activity_window in layout.activity_windows.values_mut() {
            for drawer in activity_window.activity_drawers.values_mut() {
                normalize_drawer(drawer, &mut removed_missing_active_tabs);
            }
            normalize_document_splits(&mut activity_window.content_workspace);
        }
        for window in &mut layout.floating_windows {
            normalize_document_splits(&mut window.workspace);
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

fn normalize_document_splits(node: &mut DocumentNode) {
    let DocumentNode::SplitNode {
        ratio,
        first,
        second,
        ..
    } = node
    else {
        return;
    };
    *ratio = if ratio.is_finite() {
        ratio.clamp(MIN_SPLIT_RATIO, MAX_SPLIT_RATIO)
    } else {
        DEFAULT_SPLIT_RATIO
    };
    normalize_document_splits(first);
    normalize_document_splits(second);
}

fn normalize_drawer(
    drawer: &mut super::super::ActivityDrawerLayout,
    removed_missing_active_tabs: &mut usize,
) {
    if drawer
        .tab_stack
        .active_tab
        .as_ref()
        .is_some_and(|active| !drawer.tab_stack.tabs.contains(active))
    {
        drawer.tab_stack.active_tab = drawer.tab_stack.tabs.first().cloned();
        *removed_missing_active_tabs += 1;
    }
    if drawer
        .active_view
        .as_ref()
        .is_some_and(|active| !drawer.tab_stack.tabs.contains(active))
    {
        drawer.active_view = drawer.tab_stack.active_tab.clone();
        *removed_missing_active_tabs += 1;
    }
    if drawer.mode == ActivityDrawerMode::Collapsed {
        drawer.tab_stack.active_tab = None;
        drawer.active_view = None;
    }
}

#[cfg(test)]
#[path = "tests/normalize.rs"]
mod tests;

#[cfg(test)]
#[path = "normalize/tests/split_ratio_tests.rs"]
mod split_ratio_tests;
