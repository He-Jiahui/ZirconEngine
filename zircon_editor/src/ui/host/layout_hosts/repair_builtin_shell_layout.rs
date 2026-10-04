use std::collections::{BTreeMap, HashMap, HashSet};

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
    let open_instance_index = OpenInstanceIndex::new(open_instances);
    let mut present: HashSet<_> = collect_instance_hosts(layout).into_keys().collect();
    let workbench_window_id = ActivityWindowId::workbench();
    let baseline_workbench_window = baseline
        .activity_windows
        .get(&workbench_window_id)
        .expect("built-in layout must own the workbench activity window");

    if !layout.activity_windows.contains_key(&workbench_window_id) {
        layout.activity_windows.insert(
            workbench_window_id.clone(),
            baseline_workbench_window.clone(),
        );
    }

    if let Some(workbench_window) = layout.activity_windows.get_mut(&workbench_window_id) {
        let mut activity_present = present.clone();
        repair_drawers(
            &mut workbench_window.activity_drawers,
            &baseline_workbench_window.activity_drawers,
            &open_instance_index,
            &mut activity_present,
        );
        present.extend(activity_present);
    }

    let Some(baseline_stack) = baseline_main_page_tabs(&baseline) else {
        return;
    };

    let stack = first_tab_stack_mut(ensure_host_document_root(layout));
    for instance_id in baseline_stack.tabs {
        if let Some(repaired_id) = open_instance_index.matching(&instance_id) {
            if admit_present_instance(&mut present, &repaired_id) {
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
            .and_then(|active| open_instance_index.matching(active))
            .filter(|active| stack.tabs.contains(active))
            .or_else(|| stack.tabs.first().cloned());
    }
}

struct OpenInstanceIndex<'a> {
    by_instance_id: HashMap<&'a ViewInstanceId, &'a ViewInstance>,
    by_descriptor_id: HashMap<&'a str, &'a ViewInstance>,
}

impl<'a> OpenInstanceIndex<'a> {
    fn new(open_instances: &'a [ViewInstance]) -> Self {
        let mut by_instance_id = HashMap::with_capacity(open_instances.len());
        let mut by_descriptor_id = HashMap::with_capacity(open_instances.len());
        for instance in open_instances {
            by_instance_id
                .entry(&instance.instance_id)
                .or_insert(instance);
            by_descriptor_id
                .entry(instance.descriptor_id.0.as_str())
                .or_insert(instance);
        }
        Self {
            by_instance_id,
            by_descriptor_id,
        }
    }

    fn matching(&self, instance_id: &ViewInstanceId) -> Option<ViewInstanceId> {
        self.by_instance_id
            .get(instance_id)
            .copied()
            .or_else(|| {
                let descriptor_id = instance_id.0.rsplit_once('#')?.0;
                self.by_descriptor_id.get(descriptor_id).copied()
            })
            .map(|instance| instance.instance_id.clone())
    }
}

fn admit_present_instance(
    present: &mut HashSet<ViewInstanceId>,
    instance_id: &ViewInstanceId,
) -> bool {
    if present.contains(instance_id) {
        return false;
    }
    present.insert(instance_id.clone());
    true
}

fn repair_drawers(
    drawers: &mut BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    baseline_drawers: &BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout>,
    open_instance_index: &OpenInstanceIndex<'_>,
    present: &mut HashSet<ViewInstanceId>,
) {
    for (slot, baseline_drawer) in baseline_drawers {
        let target_drawer = drawers
            .entry(*slot)
            .or_insert_with(|| ActivityDrawerLayout::new(*slot));
        let mut inserted_baseline_tab = false;

        for instance_id in &baseline_drawer.tab_stack.tabs {
            if let Some(repaired_id) = open_instance_index.matching(instance_id) {
                if admit_present_instance(present, &repaired_id) {
                    target_drawer.tab_stack.tabs.push(repaired_id);
                    inserted_baseline_tab = true;
                }
            }
        }

        if inserted_baseline_tab
            || has_repaired_shell_tab(target_drawer, baseline_drawer, open_instance_index)
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
                .and_then(|active| open_instance_index.matching(active))
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
    open_instance_index: &OpenInstanceIndex<'_>,
) -> bool {
    (!drawer.visible || !drawer.extent.is_finite() || drawer.extent <= 0.0)
        && baseline_drawer.tab_stack.tabs.iter().any(|instance_id| {
            open_instance_index
                .matching(instance_id)
                .is_some_and(|repaired_id| drawer.tab_stack.tabs.contains(&repaired_id))
        })
}

#[cfg(test)]
#[path = "tests/repair_builtin_shell_layout_optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "repair_builtin_shell_layout/tests/optimization_batch_iy_editor636_tests.rs"]
mod optimization_batch_iy_editor636_tests;
