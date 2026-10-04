use super::{
    shader_artifact_identity_is_current, ShaderSourcePreparationFailure,
    ShaderSourcePreparationTraversal, MAX_SHADER_DEPENDENCY_DEPTH,
};
use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceRecord,
};

#[test]
fn shader_source_preparation_traversal_deduplicates_completed_shared_descendants() {
    let root = ResourceId::from_stable_label("shader-root");
    let left = ResourceId::from_stable_label("shader-left");
    let right = ResourceId::from_stable_label("shader-right");
    let shared = ResourceId::from_stable_label("shader-shared");
    let mut traversal = ShaderSourcePreparationTraversal::default();

    assert!(traversal.enter(root));
    assert!(traversal.enter(left));
    assert!(traversal.enter(shared));
    traversal.finish(shared, true);
    traversal.finish(left, true);
    assert!(traversal.enter(right));
    assert!(
        !traversal.enter(shared),
        "a shared completed descendant must not be revisited through a second edge"
    );
    traversal.finish(right, true);
    traversal.finish(root, true);
}

#[test]
fn shader_source_preparation_rejects_active_cycle_with_stable_path() {
    let root = ResourceId::from_stable_label("shader-cycle-root");
    let child = ResourceId::from_stable_label("shader-cycle-child");
    let mut traversal = ShaderSourcePreparationTraversal::default();
    assert_eq!(traversal.try_enter(root, 0), Ok(true));
    assert_eq!(traversal.try_enter(child, 1), Ok(true));
    assert_eq!(
        traversal.try_enter(root, 2),
        Err(ShaderSourcePreparationFailure::Cycle {
            path: vec![root, child, root]
        })
    );
}

#[test]
fn shader_source_preparation_rejects_depth_budget_before_publication() {
    let root = ResourceId::from_stable_label("shader-depth-root");
    let mut traversal = ShaderSourcePreparationTraversal::default();
    assert_eq!(
        traversal.try_enter(root, MAX_SHADER_DEPENDENCY_DEPTH + 1),
        Err(ShaderSourcePreparationFailure::DepthBudget {
            limit: MAX_SHADER_DEPENDENCY_DEPTH
        })
    );
    assert!(traversal.staged.is_empty());
}

#[test]
fn shader_source_preparation_rejects_node_and_source_budgets() {
    let mut traversal = ShaderSourcePreparationTraversal::default();
    for index in 0..super::MAX_SHADER_DEPENDENCY_NODES {
        let shader_id = ResourceId::from_stable_label(&format!("shader-node-{index}"));
        assert_eq!(traversal.try_enter(shader_id, 0), Ok(true));
        traversal.finish(shader_id, true);
    }
    assert!(matches!(
        traversal.try_enter(ResourceId::from_stable_label("shader-node-overflow"), 0),
        Err(ShaderSourcePreparationFailure::NodeBudget { .. })
    ));
    assert!(matches!(
        traversal.add_source_bytes(super::MAX_SHADER_SOURCE_BYTES + 1),
        Err(ShaderSourcePreparationFailure::SourceBudget { .. })
    ));
}

#[test]
fn shader_source_preparation_discards_staged_sources_after_graph_failure() {
    let root = ResourceId::from_stable_label("shader-staged-root");
    let mut traversal = ShaderSourcePreparationTraversal::default();
    assert_eq!(traversal.try_enter(root, 0), Ok(true));
    traversal.finish(root, false);
    traversal.discard();
    assert!(traversal.staged.is_empty());
    assert!(traversal.completed.is_empty());
}

#[test]
fn shader_artifact_identity_requires_root_and_transitive_publications() {
    let resources = ResourceManager::new();
    let include_locator = ResourceLocator::parse("res://shaders/identity-include.zshader").unwrap();
    let include = ResourceRecord::new(
        ResourceId::from_locator(&include_locator),
        ResourceKind::Shader,
        include_locator,
    )
    .with_source_hash("include-v1");
    let root_locator = ResourceLocator::parse("res://shaders/identity-root.zshader").unwrap();
    let root = ResourceRecord::new(
        ResourceId::from_locator(&root_locator),
        ResourceKind::Shader,
        root_locator.clone(),
    )
    .with_dependency_ids(vec![include.id]);
    resources.register_ready(include.clone(), ()).unwrap();
    resources.register_ready(root.clone(), ()).unwrap();
    let original = resources
        .readiness_generation()
        .row_identity(root.id)
        .unwrap();
    let revision = original.row().record.revision;

    assert!(shader_artifact_identity_is_current(
        revision,
        &original,
        revision,
        &original.clone()
    ));
    assert!(!shader_artifact_identity_is_current(
        revision,
        &original,
        revision + 1,
        &original
    ));

    let unrelated_locator = ResourceLocator::parse("res://shaders/unrelated.zshader").unwrap();
    resources
        .register_ready(
            ResourceRecord::new(
                ResourceId::from_locator(&unrelated_locator),
                ResourceKind::Shader,
                unrelated_locator,
            ),
            (),
        )
        .unwrap();
    let unchanged = resources
        .readiness_generation()
        .row_identity(root.id)
        .unwrap();
    assert!(shader_artifact_identity_is_current(
        revision, &original, revision, &unchanged
    ));

    resources
        .register_ready(include.clone().with_source_hash("include-v2"), ())
        .unwrap();
    let changed = resources
        .readiness_generation()
        .row_identity(root.id)
        .unwrap();
    assert_eq!(changed.row().record.revision, revision);
    assert!(!shader_artifact_identity_is_current(
        revision, &original, revision, &changed
    ));

    let other_manager = ResourceManager::new();
    other_manager.register_ready(include, ()).unwrap();
    other_manager.register_ready(root.clone(), ()).unwrap();
    let other = other_manager
        .readiness_generation()
        .row_identity(root.id)
        .unwrap();
    assert!(!shader_artifact_identity_is_current(
        revision, &original, revision, &other
    ));

    resources.remove_by_locator(&root_locator).unwrap();
    assert!(resources
        .readiness_generation()
        .row_identity(root.id)
        .is_none());
    resources.register_ready(root.clone(), ()).unwrap();
    let readded = resources
        .readiness_generation()
        .row_identity(root.id)
        .unwrap();
    assert!(!shader_artifact_identity_is_current(
        revision, &original, revision, &readded
    ));
}

#[test]
fn shader_rebuild_pairs_source_with_its_atomic_resource_revision() {
    let production = include_str!("../resource_streamer_ensure_shader_source.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("shader preparation test boundary");

    assert!(production.contains("load_shader_asset_snapshot(shader_id)"));
    assert!(production.contains("let revision = shader.revision();"));
    assert!(!production.contains(
        "let revision = self.resource_revision(shader_id)?;\n        let dependency_identity"
    ));
}

#[test]
fn shader_cache_fast_path_walks_dependencies_before_accepting_root() {
    let production = include_str!("../resource_streamer_ensure_shader_source.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("shader preparation test boundary");
    let dependency_snapshot = production
        .find("let import_dependencies = cache_shader")
        .expect("root dependency snapshot before cache fast path");
    let cache_identity = production
        .find("shader_artifact_identity_is_current")
        .expect("shader cache identity fast path");
    assert!(dependency_snapshot < cache_identity);
    let cache_path = &production[cache_identity..];
    assert!(cache_path.contains("for dependency in &import_dependencies"));
    assert!(cache_path.contains("ensure_shader_dependency_sources("));
}
