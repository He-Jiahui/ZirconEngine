use std::collections::{BTreeMap, HashSet};

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
    open_instances: &[ViewInstance],
    present: &mut HashSet<ViewInstanceId>,
) {
    for (slot, baseline_drawer) in baseline_drawers {
        let target_drawer = drawers
            .entry(*slot)
            .or_insert_with(|| ActivityDrawerLayout::new(*slot));
        let mut inserted_baseline_tab = false;

        for instance_id in &baseline_drawer.tab_stack.tabs {
            if let Some(repaired_id) = matching_open_instance(instance_id, open_instances) {
                if admit_present_instance(present, &repaired_id) {
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

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeSet;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;

    const ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_INSTANCE_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn instance_ids() -> Vec<ViewInstanceId> {
        (0..ADMISSION_COUNT)
            .map(|index| {
                ViewInstanceId::new(format!(
                    "editor.builtin.shell.instance.{:05}",
                    (index * 4_099) % UNIQUE_INSTANCE_COUNT
                ))
            })
            .collect()
    }

    fn ordered_admission_count(instance_ids: &[ViewInstanceId]) -> usize {
        let mut present: BTreeSet<ViewInstanceId> = BTreeSet::new();
        let mut admitted = 0;
        for instance_id in instance_ids {
            if present.insert(instance_id.clone()) {
                admitted += 1;
            }
        }
        admitted
    }

    fn hash_admission_count(instance_ids: &[ViewInstanceId]) -> usize {
        let mut present = HashSet::new();
        let mut admitted = 0;
        for instance_id in instance_ids {
            if admit_present_instance(&mut present, instance_id) {
                admitted += 1;
            }
        }
        admitted
    }

    #[test]
    fn optimization_batch_20260826x_editor13_shell_repair_hash_admission_preserves_first_seen_order(
    ) {
        let instance_ids = [
            ViewInstanceId::new("editor.b"),
            ViewInstanceId::new("editor.a"),
            ViewInstanceId::new("editor.b"),
            ViewInstanceId::new("editor.c"),
        ];
        let mut present = HashSet::new();
        let admitted = instance_ids
            .iter()
            .filter(|instance_id| admit_present_instance(&mut present, instance_id))
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(
            admitted,
            vec![
                ViewInstanceId::new("editor.b"),
                ViewInstanceId::new("editor.a"),
                ViewInstanceId::new("editor.c"),
            ]
        );
    }

    #[test]
    fn optimization_batch_20260826x_editor13_shell_repair_uses_borrowed_hash_admission() {
        let source = include_str!("repair_builtin_shell_layout.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("let mut present: HashSet<_>"));
        assert!(production.contains("present: &mut HashSet<ViewInstanceId>"));
        assert!(production.contains("present.contains(instance_id)"));
        assert!(production.contains("admit_present_instance(&mut present, &repaired_id)"));
        assert!(production.contains("admit_present_instance(present, &repaired_id)"));
        assert!(!production.contains("BTreeSet"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826x_editor13_shell_repair_hash_admission_performance_evidence() {
        let instance_ids = instance_ids();
        assert_eq!(
            ordered_admission_count(&instance_ids),
            hash_admission_count(&instance_ids)
        );

        let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(ordered_admission_count(black_box(&instance_ids)));
                ordered_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(hash_admission_count(black_box(&instance_ids)));
                hash_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(hash_admission_count(black_box(&instance_ids)));
                hash_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(ordered_admission_count(black_box(&instance_ids)));
                ordered_samples.push(started.elapsed());
            }
        }

        let ordered_p95 = percentile_95(&mut ordered_samples);
        let hash_p95 = percentile_95(&mut hash_samples);
        println!(
            "EDITOR13_SHELL_REPAIR_HASH_ADMISSION_BENCH_V1 admissions={ADMISSION_COUNT} \
             unique_instances={UNIQUE_INSTANCE_COUNT} ordered_set_clones={ADMISSION_COUNT} \
             hash_set_clones={UNIQUE_INSTANCE_COUNT} ordered_p95_ns={} hash_p95_ns={}",
            ordered_p95.as_nanos(),
            hash_p95.as_nanos(),
        );
        assert!(
            hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
            "hash-admission P95 {:?} exceeded 60% of ordered-admission P95 {:?}",
            hash_p95,
            ordered_p95,
        );
    }
}
