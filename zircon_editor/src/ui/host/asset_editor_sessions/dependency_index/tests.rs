use crate::ui::workbench::view::ViewInstanceId;

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
