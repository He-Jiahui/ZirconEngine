use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    ActivityDrawerMode, DocumentNode, LayoutManager, MainHostPageLayout, MainPageId,
    WorkbenchLayout,
};

impl LayoutManager {
    pub(crate) fn detach_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: &ViewInstanceId,
    ) -> bool {
        let mut changed = false;

        for activity_window in layout.activity_windows.values_mut() {
            for drawer in activity_window.activity_drawers.values_mut() {
                changed |= drawer.tab_stack.remove(instance_id);
                if drawer.active_view.as_ref() == Some(instance_id) {
                    drawer.active_view = drawer.tab_stack.active_tab.clone();
                }
                if drawer.active_view.is_none() {
                    drawer.mode = ActivityDrawerMode::Collapsed;
                }
            }
        }

        for activity_window in layout.activity_windows.values_mut() {
            changed |= activity_window
                .content_workspace
                .remove_instance(instance_id);
        }

        for window in &mut layout.floating_windows {
            changed |= window.workspace.remove_instance(instance_id);
            if window.focused_view.as_ref() == Some(instance_id) {
                window.focused_view = None;
            }
        }

        let previous_main_page_count = layout.main_pages.len();
        layout.main_pages.retain(|page| match page {
            MainHostPageLayout::WorkbenchPage { .. } => true,
            MainHostPageLayout::ExclusiveActivityWindowPage {
                window_instance, ..
            } => window_instance != instance_id,
        });
        changed |= layout.main_pages.len() != previous_main_page_count;
        if !layout
            .main_pages
            .iter()
            .any(|page| page.id() == &layout.active_main_page)
        {
            changed = true;
            layout.active_main_page = layout
                .main_pages
                .iter()
                .find(|page| matches!(page, MainHostPageLayout::WorkbenchPage { .. }))
                .or_else(|| layout.main_pages.first())
                .map(|page| page.id().clone())
                .unwrap_or_else(MainPageId::workbench);
        }
        let previous_floating_window_count = layout.floating_windows.len();
        layout
            .floating_windows
            .retain(|window| match &window.workspace {
                DocumentNode::Tabs(stack) => !stack.tabs.is_empty(),
                DocumentNode::SplitNode { .. } => true,
            });
        changed |= layout.floating_windows.len() != previous_floating_window_count;

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::workbench::layout::{LayoutCommand, MainPageId};
    use crate::ui::workbench::view::ViewHost;

    #[test]
    fn closing_the_active_exclusive_page_reports_change_and_restores_a_valid_active_page() {
        let manager = LayoutManager::default();
        let mut layout = WorkbenchLayout::default();
        let instance_id = ViewInstanceId::new("editor.asset_browser#1");
        let page_id = MainPageId::new("page:editor.asset_browser#1");
        manager
            .apply(
                &mut layout,
                LayoutCommand::AttachView {
                    instance_id: instance_id.clone(),
                    target: ViewHost::ExclusivePage(page_id.clone()),
                    anchor: None,
                },
            )
            .expect("exclusive page should attach");
        assert_eq!(layout.active_main_page, page_id);

        let close = manager
            .apply(&mut layout, LayoutCommand::CloseView { instance_id })
            .expect("exclusive page should close");

        assert!(close.changed);
        assert_eq!(layout.active_main_page, MainPageId::workbench());
        assert!(layout.main_pages.iter().all(|page| page.id() != &page_id));
    }
}
