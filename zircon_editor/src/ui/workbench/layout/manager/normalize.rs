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
        let mut removed_missing_active_tabs = 0;
        for activity_window in layout.activity_windows.values_mut() {
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
mod tests {
    use super::*;
    use crate::ui::workbench::layout::{ActivityDrawerLayout, ActivityDrawerSlot};
    use crate::ui::workbench::view::ViewInstanceId;

    #[test]
    fn drawer_selection_only_clones_when_repair_is_required() {
        let selected = ViewInstanceId::new("editor.selection#stable");
        let mut drawer = ActivityDrawerLayout::new(ActivityDrawerSlot::LeftTop);
        drawer.tab_stack.tabs.push(selected.clone());
        drawer.tab_stack.active_tab = Some(selected.clone());
        drawer.active_view = Some(selected.clone());
        let mut repairs = 0;

        normalize_drawer(&mut drawer, &mut repairs);

        assert_eq!(repairs, 0);
        assert_eq!(drawer.tab_stack.active_tab, Some(selected.clone()));
        assert_eq!(drawer.active_view, Some(selected.clone()));

        drawer.tab_stack.active_tab = Some(ViewInstanceId::new("editor.missing#tab"));
        drawer.active_view = Some(ViewInstanceId::new("editor.missing#view"));
        normalize_drawer(&mut drawer, &mut repairs);

        assert_eq!(repairs, 2);
        assert_eq!(drawer.tab_stack.active_tab, Some(selected.clone()));
        assert_eq!(drawer.active_view, Some(selected));
    }
}
