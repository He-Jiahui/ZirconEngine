use std::collections::{BTreeMap, BTreeSet};

use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerSlot, ActivityWindowId, WorkbenchLayout,
};
use crate::ui::workbench::view::{ViewInstance, ViewInstanceId};

use super::super::builtin_layout::builtin_hybrid_layout_for_subsystems;
use super::super::editor_subsystems::EditorSubsystemReport;
use super::baseline_main_page_tabs::baseline_main_page_tabs;
use super::collect_instance_hosts::collect_instance_hosts;
use super::ensure_host_document_root::ensure_host_document_root;
use super::first_tab_stack_mut::first_tab_stack_mut;

pub(in crate::ui::host) fn repair_builtin_shell_layout(
    layout: &mut WorkbenchLayout,
    open_instances: &[ViewInstance],
    subsystems: &EditorSubsystemReport,
) {
    let baseline = builtin_hybrid_layout_for_subsystems(subsystems);
    let mut present: BTreeSet<_> = collect_instance_hosts(layout).into_keys().collect();

    if layout.activity_windows.is_empty() {
        let _ = layout.default_activity_window_mut();
    }

    if let Some(workbench_window) = layout
        .activity_windows
        .get_mut(&ActivityWindowId::workbench())
    {
        let mut activity_present = present.clone();
        repair_drawers(
            &mut workbench_window.activity_drawers,
            &baseline.drawers,
            open_instances,
            &mut activity_present,
        );
        present.extend(activity_present);
    }

    let Some(baseline_stack) = baseline_main_page_tabs(&baseline) else {
        return;
    };

    let stack = first_tab_stack_mut(ensure_host_document_root(layout));
    for instance_id in baseline_stack.tabs {
        if let Some(repaired_id) = matching_open_instance(&instance_id, open_instances) {
            if present.insert(repaired_id.clone()) {
                stack.tabs.push(repaired_id);
            }
        }
    }

    if stack
        .active_tab
        .as_ref()
        .is_none_or(|active| !stack.tabs.contains(active))
    {
        stack.active_tab = baseline_stack
            .active_tab
            .as_ref()
            .and_then(|active| matching_open_instance(active, open_instances))
            .filter(|active| stack.tabs.contains(active))
            .or_else(|| stack.tabs.first().cloned());
    }
}

fn matching_open_instance(
    instance_id: &ViewInstanceId,
    open_instances: &[ViewInstance],
) -> Option<ViewInstanceId> {
    open_instances
        .iter()
        .find(|instance| &instance.instance_id == instance_id)
        .or_else(|| {
            let descriptor_id = instance_id.0.rsplit_once('#')?.0;
            open_instances
                .iter()
                .find(|instance| instance.descriptor_id.0 == descriptor_id)
        })
        .map(|instance| instance.instance_id.clone())
}

fn repair_drawers(
    drawers: &mut BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    baseline_drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    open_instances: &[ViewInstance],
    present: &mut BTreeSet<ViewInstanceId>,
) {
    for (slot, baseline_drawer) in baseline_drawers {
        let target_drawer = drawers
            .entry(*slot)
            .or_insert_with(|| ActivityDrawerLayout::new(*slot));
        let mut inserted_baseline_tab = false;

        for instance_id in &baseline_drawer.tab_stack.tabs {
            if let Some(repaired_id) = matching_open_instance(instance_id, open_instances) {
                if present.insert(repaired_id.clone()) {
                    target_drawer.tab_stack.tabs.push(repaired_id);
                    inserted_baseline_tab = true;
                }
            }
        }

        if inserted_baseline_tab
            || has_repaired_shell_tab(target_drawer, baseline_drawer, open_instances)
        {
            target_drawer.mode = baseline_drawer.mode;
            target_drawer.extent = baseline_drawer.extent;
            target_drawer.visible = baseline_drawer.visible;
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
                .as_ref()
                .and_then(|active| matching_open_instance(active, open_instances))
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
}

fn has_repaired_shell_tab(
    drawer: &ActivityDrawerLayout,
    baseline_drawer: &ActivityDrawerLayout,
    open_instances: &[ViewInstance],
) -> bool {
    (!drawer.visible || !drawer.extent.is_finite() || drawer.extent <= 0.0)
        && baseline_drawer.tab_stack.tabs.iter().any(|instance_id| {
            matching_open_instance(instance_id, open_instances)
                .is_some_and(|repaired_id| drawer.tab_stack.tabs.contains(&repaired_id))
        })
}
