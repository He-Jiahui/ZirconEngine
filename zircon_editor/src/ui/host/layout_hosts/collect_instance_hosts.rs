use std::collections::HashMap;

use crate::ui::workbench::layout::{MainHostPageLayout, WorkbenchLayout};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

use super::collect_document_hosts::collect_document_hosts;

#[cfg(test)]
#[path = "collect_instance_hosts/hash_placement_tests.rs"]
mod hash_placement_tests;

pub(in crate::ui::host) fn collect_instance_hosts(
    layout: &WorkbenchLayout,
) -> HashMap<ViewInstanceId, ViewHost> {
    let mut placements = HashMap::new();

    for activity_window in layout.activity_windows().values() {
        for (slot, drawer) in &activity_window.activity_drawers {
            for instance_id in &drawer.tab_stack.tabs {
                placements.insert(instance_id.clone(), ViewHost::Drawer(*slot));
            }
        }
    }

    for page in &layout.main_pages {
        match page {
            MainHostPageLayout::WorkbenchPage { id, .. } => {
                if let Some(content_workspace) = layout.content_workspace_for_page(id) {
                    collect_document_hosts(content_workspace, &mut placements, |path| {
                        ViewHost::Document(id.clone(), path)
                    });
                }
            }
            MainHostPageLayout::ExclusiveActivityWindowPage {
                id,
                window_instance,
                ..
            } => {
                placements.insert(window_instance.clone(), ViewHost::ExclusivePage(id.clone()));
            }
        }
    }

    for window in &layout.floating_windows {
        collect_document_hosts(&window.workspace, &mut placements, |path| {
            ViewHost::FloatingWindow(window.window_id.clone(), path)
        });
    }

    placements
}
