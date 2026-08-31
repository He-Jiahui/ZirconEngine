use crate::ui::workbench::view::ViewInstanceId;

use super::super::{
    ActivityDrawerMode, DocumentNode, LayoutManager, MainHostPageLayout, MainPageId,
    WorkbenchLayout,
};

fn active_main_page_differs(active: &MainPageId, candidate: &MainPageId) -> bool {
    active != candidate
}

impl LayoutManager {
    pub(crate) fn focus_instance(
        &self,
        layout: &mut WorkbenchLayout,
        instance_id: &ViewInstanceId,
    ) -> bool {
        let mut focused_activity_window = None;
        for (activity_window_id, activity_window) in layout.activity_windows.iter_mut() {
            let target_slot = activity_window
                .activity_drawers
                .iter()
                .find_map(|(slot, drawer)| {
                    drawer.tab_stack.tabs.contains(instance_id).then_some(*slot)
                });
            let Some(target_slot) = target_slot else {
                continue;
            };
            let Some(drawer) = activity_window.activity_drawers.get_mut(&target_slot) else {
                continue;
            };
            let mut changed = drawer.tab_stack.active_tab.as_ref() != Some(instance_id)
                || drawer.active_view.as_ref() != Some(instance_id)
                || drawer.mode == ActivityDrawerMode::Collapsed;
            drawer.tab_stack.active_tab = Some(instance_id.clone());
            drawer.active_view = Some(instance_id.clone());
            if drawer.mode == ActivityDrawerMode::Collapsed {
                drawer.mode = ActivityDrawerMode::Pinned;
            }
            changed |= activity_window.collapse_drawer_region_siblings(target_slot);
            focused_activity_window = Some((activity_window_id.clone(), changed));
            break;
        }
        if let Some((activity_window_id, mut changed)) = focused_activity_window {
            if let Some(page_id) = layout.page_id_for_activity_window(&activity_window_id) {
                changed |= layout.active_main_page != page_id;
                if changed {
                    layout.active_main_page = page_id;
                }
            }
            return changed;
        }

        let mut focused_content_window = None;
        for (activity_window_id, activity_window) in &mut layout.activity_windows {
            if let Some(changed) =
                Self::focus_in_document_node(&mut activity_window.content_workspace, instance_id)
            {
                focused_content_window = Some((activity_window_id.clone(), changed));
                break;
            }
        }
        if let Some((activity_window_id, mut changed)) = focused_content_window {
            if let Some(page_id) = layout.page_id_for_activity_window(&activity_window_id) {
                changed |= layout.active_main_page != page_id;
                if changed {
                    layout.active_main_page = page_id;
                }
            }
            return changed;
        }

        for page in &layout.main_pages {
            if let MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } = page
            {
                if window_instance == instance_id {
                    let changed = active_main_page_differs(&layout.active_main_page, id);
                    if changed {
                        layout.active_main_page = id.clone();
                    }
                    return changed;
                }
            }
        }

        for window in &mut layout.floating_windows {
            if window.workspace.contains(instance_id) {
                let changed = window.focused_view.as_ref() != Some(instance_id);
                if changed {
                    window.focused_view = Some(instance_id.clone());
                }
                return changed;
            }
        }

        false
    }

    fn focus_in_document_node(
        node: &mut DocumentNode,
        instance_id: &ViewInstanceId,
    ) -> Option<bool> {
        match node {
            DocumentNode::Tabs(stack) => {
                if stack.tabs.contains(instance_id) {
                    let changed = stack.active_tab.as_ref() != Some(instance_id);
                    if changed {
                        stack.active_tab = Some(instance_id.clone());
                    }
                    Some(changed)
                } else {
                    None
                }
            }
            DocumentNode::SplitNode { first, second, .. } => {
                Self::focus_in_document_node(first, instance_id)
                    .or_else(|| Self::focus_in_document_node(second, instance_id))
            }
        }
    }
}

#[cfg(test)]
#[path = "focus/borrowed_exclusive_page_comparison_tests.rs"]
mod borrowed_exclusive_page_comparison_tests;

#[cfg(test)]
mod source_guards {
    #[test]
    fn production_focus_path_is_fail_closed_without_legacy_window_synthesis() {
        let source = include_str!("focus.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production focus source");

        assert!(!production.contains(".expect("));
        assert!(!production.contains("activity_windows.is_empty()"));
        assert!(!production.contains("default_activity_window_mut()"));
    }
}
