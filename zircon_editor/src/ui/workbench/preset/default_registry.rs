use std::collections::HashSet;

use serde_json::Value;

use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use crate::ui::workbench::window_registry::EditorWindowRegistry;

use super::default_layout::view_instance_id_for_window;
use super::{EditorFunctionalWindowKind, EditorUiDesignStack};

impl EditorUiDesignStack {
    pub fn default_view_instances(&self) -> Vec<ViewInstance> {
        let mut seen = HashSet::new();
        let mut instances = Vec::new();

        for window in &self.window_model.windows {
            for view in &window.primary_views {
                let instance = self.view_instance_for_window(window.kind, view, true);
                if admit_view_instance_id(&mut seen, &instance.instance_id) {
                    instances.push(instance);
                }
            }
            for view in &window.drawer_views {
                let instance = self.view_instance_for_window(window.kind, view, false);
                if admit_view_instance_id(&mut seen, &instance.instance_id) {
                    instances.push(instance);
                }
            }
        }

        instances
    }

    pub fn default_window_registry(&self) -> EditorWindowRegistry {
        let layout = self.default_workbench_layout();
        let instances = self.default_view_instances();
        EditorWindowRegistry::sync_from_layout(&layout, &instances)
    }

    fn view_instance_for_window(
        &self,
        window_kind: EditorFunctionalWindowKind,
        view: &str,
        primary_view: bool,
    ) -> ViewInstance {
        ViewInstance {
            instance_id: view_instance_id_for_window(window_kind, view),
            descriptor_id: ViewDescriptorId::new(view),
            title: title_from_view(view),
            serializable_payload: Value::Null,
            dirty: false,
            host: self.view_host_for_window(window_kind, view, primary_view),
        }
    }

    fn view_host_for_window(
        &self,
        window_kind: EditorFunctionalWindowKind,
        view: &str,
        primary_view: bool,
    ) -> ViewHost {
        if !primary_view {
            return ViewHost::Drawer(self.drawer_slot_for_view(view));
        }

        if window_kind == EditorFunctionalWindowKind::Workbench {
            ViewHost::Document(MainPageId::workbench(), vec![])
        } else {
            ViewHost::FloatingWindow(
                MainPageId::new(format!("window:{}", window_kind.slug())),
                vec![],
            )
        }
    }
}

fn admit_view_instance_id(
    seen: &mut HashSet<ViewInstanceId>,
    instance_id: &ViewInstanceId,
) -> bool {
    if seen.contains(instance_id) {
        return false;
    }
    seen.insert(instance_id.clone());
    true
}

fn title_from_view(view: &str) -> String {
    let view = view.strip_prefix("editor.").unwrap_or(view);
    view.split(['.', '_'])
        .filter(|part| !part.is_empty())
        .map(capitalize_ascii)
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_ascii(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut title = first.to_ascii_uppercase().to_string();
    title.extend(chars);
    title
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use crate::ui::workbench::layout::{ActivityWindowHostMode, ActivityWindowId};
    use crate::ui::workbench::view::ViewInstanceId;
    use crate::ui::workbench::window_registry::{DrawerDockPosition, WindowKind};

    use super::*;

    const VIEW_ADMISSION_COUNT: usize = 65_536;
    const UNIQUE_VIEW_COUNT: usize = 8_192;
    const SAMPLE_COUNT: usize = 17;

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() - 1) * 95 / 100]
    }

    fn view_instance_ids() -> Vec<ViewInstanceId> {
        (0..VIEW_ADMISSION_COUNT)
            .map(|index| {
                ViewInstanceId::new(format!(
                    "editor.synthetic.{:04}",
                    (index * 4_099) % UNIQUE_VIEW_COUNT
                ))
            })
            .collect()
    }

    fn legacy_view_admission_count(instance_ids: &[ViewInstanceId]) -> usize {
        let mut seen = BTreeSet::new();
        instance_ids
            .iter()
            .filter(|instance_id| seen.insert((*instance_id).clone()))
            .count()
    }

    fn optimized_view_admission_count(instance_ids: &[ViewInstanceId]) -> usize {
        let mut seen = HashSet::new();
        instance_ids
            .iter()
            .filter(|instance_id| admit_view_instance_id(&mut seen, *instance_id))
            .count()
    }

    #[test]
    fn default_view_instances_are_unique_per_functional_window() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let instances = stack.default_view_instances();
        let ids = instances
            .iter()
            .map(|instance| instance.instance_id.clone())
            .collect::<BTreeSet<_>>();

        assert_eq!(ids.len(), instances.len());
        assert!(ids.contains(&ViewInstanceId::new("editor.inspector#1")));
        assert!(ids.contains(&ViewInstanceId::new("editor.inspector#material_editor")));
        assert!(ids.contains(&ViewInstanceId::new("editor.asset_browser#material_editor")));
    }

    #[test]
    fn default_window_registry_syncs_preset_windows_drawers_and_titles() {
        let stack = EditorUiDesignStack::material_fyrox_jetbrains_unreal();
        let registry = stack.default_window_registry();

        let workbench = registry
            .get_window(&ActivityWindowId::workbench())
            .expect("workbench window");
        assert_eq!(workbench.kind, WindowKind::DrawerCapable);
        assert_eq!(
            workbench.selected_drawer,
            Some(ViewInstanceId::new("editor.hierarchy#1"))
        );

        let material = registry
            .get_window(&ActivityWindowId::new("window:material_editor"))
            .expect("material editor window");
        assert_eq!(material.kind, WindowKind::DrawerCapable);
        assert_eq!(
            material.host_mode,
            ActivityWindowHostMode::NativeWindowHandle
        );
        assert_eq!(
            material
                .drawer_views
                .get(&DrawerDockPosition::RightTop)
                .expect("right drawer"),
            &vec![ViewInstanceId::new("editor.inspector#material_editor")]
        );

        let inspector = registry
            .get_drawer_view(&ViewInstanceId::new("editor.inspector#material_editor"))
            .expect("material inspector drawer view");
        assert_eq!(inspector.title, "Inspector");
        assert_eq!(
            inspector.owner_window,
            ActivityWindowId::new("window:material_editor")
        );
    }

    #[test]
    fn optimization_batch_20260826s_editor13_hash_admission_preserves_first_seen_order() {
        let mut seen = HashSet::new();
        let mut admitted = Vec::new();
        for instance_id in ["editor.b#1", "editor.a#1", "editor.b#1"] {
            let instance_id = ViewInstanceId::new(instance_id);
            if admit_view_instance_id(&mut seen, &instance_id) {
                admitted.push(instance_id);
            }
        }

        assert_eq!(
            admitted,
            vec![
                ViewInstanceId::new("editor.b#1"),
                ViewInstanceId::new("editor.a#1")
            ]
        );
        assert_eq!(seen.len(), 2);
    }

    #[test]
    fn optimization_batch_20260826s_editor13_default_views_use_hash_admission() {
        let source = include_str!("default_registry.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();

        assert!(production.contains("use std::collections::HashSet;"));
        assert!(production.contains("HashSet<ViewInstanceId>"));
        assert!(production.contains("seen.contains(instance_id)"));
        assert!(production.contains("seen.insert(instance_id.clone())"));
        assert!(!production.contains("BTreeSet"));
        assert!(!production.contains("seen.insert(instance.instance_id.clone())"));
    }

    #[test]
    #[ignore = "release performance evidence"]
    fn optimization_batch_20260826s_editor13_default_view_hash_admission_performance_evidence() {
        let instance_ids = view_instance_ids();
        assert_eq!(
            legacy_view_admission_count(&instance_ids),
            UNIQUE_VIEW_COUNT
        );
        assert_eq!(
            optimized_view_admission_count(&instance_ids),
            UNIQUE_VIEW_COUNT
        );

        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            if sample % 2 == 0 {
                let started = Instant::now();
                black_box(legacy_view_admission_count(black_box(&instance_ids)));
                legacy_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(optimized_view_admission_count(black_box(&instance_ids)));
                optimized_samples.push(started.elapsed());
            } else {
                let started = Instant::now();
                black_box(optimized_view_admission_count(black_box(&instance_ids)));
                optimized_samples.push(started.elapsed());

                let started = Instant::now();
                black_box(legacy_view_admission_count(black_box(&instance_ids)));
                legacy_samples.push(started.elapsed());
            }
        }

        let legacy_p95 = percentile_95(&mut legacy_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "EDITOR13_DEFAULT_VIEW_HASH_ADMISSION_BENCH_V1 admissions={VIEW_ADMISSION_COUNT} \
             unique_views={UNIQUE_VIEW_COUNT} legacy_id_clones={VIEW_ADMISSION_COUNT} \
             optimized_id_clones={UNIQUE_VIEW_COUNT} legacy_p95_ns={} optimized_p95_ns={}",
            legacy_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 60,
            "hash-admission P95 {:?} exceeded 60% of tree-admission P95 {:?}",
            optimized_p95,
            legacy_p95,
        );
    }
}
