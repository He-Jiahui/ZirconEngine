use std::collections::BTreeMap;

use crate::ui::workbench::layout::{MainHostPageLayout, WorkbenchLayout};
use crate::ui::workbench::view::{ViewHost, ViewInstanceId};

use super::collect_document_hosts::collect_document_hosts;

pub(in crate::ui::host) fn collect_instance_hosts(
    layout: &WorkbenchLayout,
) -> BTreeMap<ViewInstanceId, ViewHost> {
    let mut placements = BTreeMap::new();

    for (slot, drawer) in &layout.drawers {
        for instance_id in &drawer.tab_stack.tabs {
            placements.insert(instance_id.clone(), ViewHost::Drawer(*slot));
        }
    }

    for page in &layout.main_pages {
        match page {
            MainHostPageLayout::WorkbenchPage {
                id,
                document_workspace,
                ..
            } => collect_document_hosts(document_workspace, &mut placements, |path| {
                ViewHost::Document(id.clone(), path)
            }),
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
