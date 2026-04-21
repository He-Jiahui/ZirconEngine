use std::collections::BTreeSet;

use crate::ui::workbench::layout::{ActivityDrawerLayout, WorkbenchLayout};

use super::super::builtin_layout::builtin_hybrid_layout;
use super::baseline_main_page_tabs::baseline_main_page_tabs;
use super::collect_instance_hosts::collect_instance_hosts;
use super::ensure_workbench_document_root::ensure_workbench_document_root;
use super::first_tab_stack_mut::first_tab_stack_mut;

pub(in crate::ui::host) fn repair_builtin_shell_layout(layout: &mut WorkbenchLayout) {
    let baseline = builtin_hybrid_layout();
    let mut present: BTreeSet<_> = collect_instance_hosts(layout).into_keys().collect();

    for (slot, baseline_drawer) in &baseline.drawers {
        let target_drawer = layout
            .drawers
            .entry(*slot)
            .or_insert_with(|| ActivityDrawerLayout::new(*slot));

        for instance_id in &baseline_drawer.tab_stack.tabs {
            if present.insert(instance_id.clone()) {
                target_drawer.tab_stack.tabs.push(instance_id.clone());
            }
        }

        if target_drawer
            .tab_stack
            .active_tab
            .as_ref()
            .is_none_or(|active| !target_drawer.tab_stack.tabs.contains(active))
        {
            target_drawer.tab_stack.active_tab = baseline_drawer
                .tab_stack
                .active_tab
                .clone()
                .filter(|active| target_drawer.tab_stack.tabs.contains(active))
                .or_else(|| target_drawer.tab_stack.tabs.first().cloned());
        }

        if target_drawer
            .active_view
            .as_ref()
            .is_none_or(|active| !target_drawer.tab_stack.tabs.contains(active))
        {
            target_drawer.active_view = target_drawer.tab_stack.active_tab.clone();
        }
    }

    let Some(baseline_stack) = baseline_main_page_tabs(&baseline) else {
        return;
    };

    let stack = first_tab_stack_mut(ensure_workbench_document_root(layout));
    for instance_id in baseline_stack.tabs {
        if present.insert(instance_id.clone()) {
            stack.tabs.push(instance_id);
        }
    }

    if stack
        .active_tab
        .as_ref()
        .is_none_or(|active| !stack.tabs.contains(active))
    {
        stack.active_tab = baseline_stack
            .active_tab
            .filter(|active| stack.tabs.contains(active))
            .or_else(|| stack.tabs.first().cloned());
    }
}
