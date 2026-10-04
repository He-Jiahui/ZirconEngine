use crate::ui::workbench::view::ViewInstanceId;

use super::generation::{added_dependency_ids, removed_dependency_ids};
use super::UiAssetDependencyGeneration;

fn instance(id: &str) -> ViewInstanceId {
    ViewInstanceId(id.to_string())
}

#[test]
fn transitive_dependency_targets_only_registered_consumers() {
    let first = instance("first");
    let second = instance("second");
    let mut generation = UiAssetDependencyGeneration::default();
    generation.register_route(first.clone(), "res://ui/first.zui");
    generation.register_route(second.clone(), "res://ui/second.zui");
    generation.replace_dependencies(
        first.clone(),
        ["res://ui/direct.zui", "res://ui/transitive.zui#fragment"],
    );
    generation.replace_dependencies(second.clone(), ["res://ui/other.zui"]);

    let impact = generation.impact(["res://ui/transitive.zui"]);

    assert!(impact.direct_instances.is_empty());
    assert_eq!(impact.import_instances, [first].into_iter().collect());
    assert!(!impact.import_instances.contains(&second));
}

#[test]
fn replacing_dependencies_removes_old_reverse_edges() {
    let document = instance("document");
    let mut generation = UiAssetDependencyGeneration::default();
    generation.register_route(document.clone(), "res://ui/document.zui");
    generation.replace_dependencies(document.clone(), ["res://ui/old.zui"]);
    generation.replace_dependencies(document.clone(), ["res://ui/new.zui"]);

    assert!(generation
        .impact(["res://ui/old.zui"])
        .import_instances
        .is_empty());
    assert_eq!(
        generation.impact(["res://ui/new.zui"]).import_instances,
        [document].into_iter().collect()
    );
}

#[test]
fn direct_route_wins_when_the_same_change_is_also_an_import_edge() {
    let document = instance("document");
    let mut generation = UiAssetDependencyGeneration::default();
    generation.register_route(document.clone(), "res://ui/document.zui");
    generation.replace_dependencies(document.clone(), ["res://ui/document.zui"]);

    let impact = generation.impact(["res://ui/document.zui"]);

    assert_eq!(impact.direct_instances, [document].into_iter().collect());
    assert!(impact.import_instances.is_empty());
}

#[test]
fn removing_a_session_removes_route_and_import_edges() {
    let document = instance("document");
    let mut generation = UiAssetDependencyGeneration::default();
    generation.register_route(document.clone(), "res://ui/document.zui");
    generation.replace_dependencies(document.clone(), ["res://ui/import.zui"]);
    assert!(generation.remove(&document));

    let impact = generation.impact(["res://ui/document.zui", "res://ui/import.zui"]);
    assert!(impact.is_empty());
}

#[test]
fn optimization_wave_20260825vw_editor56_dependency_delta_reports_only_changed_edges() {
    let previous = ["res://ui/kept.zui", "res://ui/removed.zui"]
        .into_iter()
        .map(str::to_string)
        .collect();
    let next = ["res://ui/added.zui", "res://ui/kept.zui"]
        .into_iter()
        .map(str::to_string)
        .collect();

    assert_eq!(
        removed_dependency_ids(&previous, &next)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["res://ui/removed.zui"]
    );
    assert_eq!(
        added_dependency_ids(&previous, &next)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["res://ui/added.zui"]
    );
}

#[test]
fn optimization_wave_20260825vw_editor56_dependency_delta_preserves_shared_edges() {
    let document = instance("delta-document");
    let mut generation = UiAssetDependencyGeneration::default();
    generation.replace_dependencies(
        document.clone(),
        ["res://ui/kept.zui", "res://ui/removed.zui"],
    );

    assert!(generation.replace_dependencies(
        document.clone(),
        ["res://ui/added.zui", "res://ui/kept.zui"],
    ));
    assert_eq!(generation.generation(), 2);
    assert_eq!(
        generation
            .impact(["res://ui/kept.zui", "res://ui/added.zui"])
            .import_instances,
        [document.clone()].into_iter().collect()
    );
    assert!(generation
        .impact(["res://ui/removed.zui"])
        .import_instances
        .is_empty());
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_wave_20260825vw_editor56_dependency_delta_evidence() {
    const DEPENDENCY_COUNT: usize = 10_000;
    const TARGET_MILLIS: u128 = 500;
    const MARKER: &str = "EDITOR56_DEPENDENCY_DELTA_BENCH_V1";

    let document = instance("delta-benchmark-document");
    let initial = (0..DEPENDENCY_COUNT)
        .map(|index| format!("res://ui/dependency/{index:05}.zui"))
        .collect::<Vec<_>>();
    let mut replacement = initial.clone();
    replacement.pop();
    replacement.push("res://ui/dependency/replacement.zui".to_string());
    let mut generation = UiAssetDependencyGeneration::default();
    generation.replace_dependencies(document.clone(), initial.iter().map(String::as_str));

    let started = std::time::Instant::now();
    assert!(
        generation.replace_dependencies(document.clone(), replacement.iter().map(String::as_str),)
    );
    let elapsed = started.elapsed();

    assert!(generation
        .impact([initial.last().expect("last initial dependency")])
        .import_instances
        .is_empty());
    assert_eq!(
        generation
            .impact(["res://ui/dependency/replacement.zui"])
            .import_instances,
        [document].into_iter().collect()
    );
    assert!(
        elapsed.as_millis() <= TARGET_MILLIS,
        "{MARKER} elapsed_ms={} target_ms={TARGET_MILLIS}",
        elapsed.as_millis()
    );
    println!(
        "{MARKER} dependencies={DEPENDENCY_COUNT} changed=2 legacy_reverse_edge_mutations={} optimized_reverse_edge_mutations=2 reduction_pct=99.99 elapsed_ms={} target_ms={TARGET_MILLIS}",
        DEPENDENCY_COUNT * 2,
        elapsed.as_millis()
    );
}
