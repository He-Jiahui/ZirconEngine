use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

use super::super::editor_capabilities::EditorCapabilitySnapshot;
use super::super::editor_subsystems::EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS;

pub(super) fn builtin_shell_view_instances(
    snapshot: &EditorCapabilitySnapshot,
) -> Vec<ViewInstance> {
    builtin_shell_view_instances_with_runtime_diagnostics(
        snapshot.is_enabled(EDITOR_SUBSYSTEM_RUNTIME_DIAGNOSTICS),
    )
}

fn builtin_shell_view_instances_with_runtime_diagnostics(
    runtime_diagnostics_enabled: bool,
) -> Vec<ViewInstance> {
    let mut instances = Vec::with_capacity(if runtime_diagnostics_enabled { 11 } else { 9 });
    instances.extend([
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.assets#1"),
            descriptor_id: ViewDescriptorId::new("editor.assets"),
            title: "Asset Browser".to_string(),
            serializable_payload: serde_json::json!({ "root": "crate://" }),
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.module_plugins#1"),
            descriptor_id: ViewDescriptorId::new("editor.module_plugins"),
            title: "Plugin Manager".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftBottom),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.hierarchy#1"),
            descriptor_id: ViewDescriptorId::new("editor.hierarchy"),
            title: "Hierarchy".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::LeftTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.inspector#1"),
            descriptor_id: ViewDescriptorId::new("editor.inspector"),
            title: "Inspector".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::RightTop),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.console#1"),
            descriptor_id: ViewDescriptorId::new("editor.console"),
            title: "Console".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        },
    ]);
    if runtime_diagnostics_enabled {
        instances.extend([
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.runtime_diagnostics#1"),
            descriptor_id: ViewDescriptorId::new("editor.runtime_diagnostics"),
            title: "Runtime Diagnostics".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.performance_timeline#1"),
            descriptor_id: ViewDescriptorId::new("editor.performance_timeline"),
            title: "Performance Timeline".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        },
        ]);
    }
    instances.extend([
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.build_export_desktop#1"),
            descriptor_id: ViewDescriptorId::new("editor.build_export_desktop"),
            title: "Desktop Export".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.generated_bottom#1"),
            descriptor_id: ViewDescriptorId::new("editor.generated_bottom"),
            title: "Generated Output".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Drawer(ActivityDrawerSlot::Bottom),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.game#1"),
            descriptor_id: ViewDescriptorId::new("editor.game"),
            title: "Game".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
        ViewInstance {
            instance_id: ViewInstanceId::new("editor.scene#1"),
            descriptor_id: ViewDescriptorId::new("editor.scene"),
            title: "Scene".to_string(),
            serializable_payload: serde_json::Value::Null,
            dirty: false,
            host: ViewHost::Document(MainPageId::workbench(), vec![]),
        },
    ]);
    instances
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_COUNT: usize = 17;
    const ITERATIONS: usize = 4_096;

    fn legacy_disabled_projection() -> Vec<ViewInstance> {
        let mut instances = builtin_shell_view_instances_with_runtime_diagnostics(true);
        instances.retain(|instance| {
            !matches!(
                instance.descriptor_id.0.as_str(),
                "editor.runtime_diagnostics" | "editor.performance_timeline"
            )
        });
        instances
    }

    fn percentile_95(mut samples: Vec<u128>) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100) - 1]
    }

    #[test]
    fn optimization_batch_hi_editor593_disabled_diagnostics_are_not_materialized() {
        let optimized = builtin_shell_view_instances_with_runtime_diagnostics(false);
        let legacy = legacy_disabled_projection();

        assert_eq!(optimized.len(), 9);
        assert_eq!(
            optimized
                .iter()
                .map(|instance| instance.descriptor_id.0.as_str())
                .collect::<Vec<_>>(),
            legacy
                .iter()
                .map(|instance| instance.descriptor_id.0.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn optimization_batch_hi_editor593_enabled_diagnostics_preserve_builtin_order() {
        let instances = builtin_shell_view_instances_with_runtime_diagnostics(true);
        let ids = instances
            .iter()
            .map(|instance| instance.descriptor_id.0.as_str())
            .collect::<Vec<_>>();

        assert_eq!(instances.len(), 11);
        assert_eq!(ids[5], "editor.runtime_diagnostics");
        assert_eq!(ids[6], "editor.performance_timeline");
        assert_eq!(ids[7], "editor.build_export_desktop");
    }

    #[test]
    fn optimization_batch_hi_editor593_capability_construction_source_contract() {
        let production = include_str!("builtin_shell_view_instances.rs")
            .split("#[cfg(test)]")
            .next()
            .unwrap();

        assert!(production.contains("if runtime_diagnostics_enabled"));
        assert!(!production.contains("instances.retain"));
    }

    #[test]
    #[ignore = "Windows-native release performance evidence"]
    fn optimization_batch_hi_editor593_capability_construction_bench() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
        for sample in 0..SAMPLE_COUNT {
            let measure_legacy = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    black_box(legacy_disabled_projection());
                }
                started.elapsed().as_nanos()
            };
            let measure_optimized = || {
                let started = Instant::now();
                for _ in 0..ITERATIONS {
                    black_box(builtin_shell_view_instances_with_runtime_diagnostics(false));
                }
                started.elapsed().as_nanos()
            };
            if sample % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = percentile_95(legacy_samples);
        let optimized_p95 = percentile_95(optimized_samples);
        println!(
            "EDITOR593_BUILTIN_VIEW_CAPABILITY_CONSTRUCTION_BENCH_V1 legacy_p95_ns={} optimized_p95_ns={} samples={} iterations={} constructed_views=11->9 filtered_views=2->0",
            legacy_p95, optimized_p95, SAMPLE_COUNT, ITERATIONS,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(85),
            "optimized P95 must be at most 85% of legacy P95"
        );
    }
}
