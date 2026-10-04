use super::*;
use crate::core::resource::{ResourceId, ResourceKind, ResourceLocator};

fn record_with_state(state: ResourceState) -> ResourceRecord {
    let locator =
        ResourceLocator::parse("res://textures/state-matrix.png").expect("valid test locator");
    ResourceRecord::new(
        ResourceId::from_locator(&locator),
        ResourceKind::Texture,
        locator,
    )
    .with_state(state)
}

#[test]
fn asset_load_state_projection_matches_resource_record_matrix() {
    assert_eq!(
        AssetLoadState::from_resource(None, None, false),
        AssetLoadState::NotLoaded
    );
    assert_eq!(
        AssetLoadState::from_resource(
            Some(&record_with_state(ResourceState::Pending)),
            None,
            false
        ),
        AssetLoadState::Loading
    );
    assert_eq!(
        AssetLoadState::from_resource(Some(&record_with_state(ResourceState::Ready)), None, true),
        AssetLoadState::Loaded
    );
    assert_eq!(
        AssetLoadState::from_resource(Some(&record_with_state(ResourceState::Ready)), None, false),
        AssetLoadState::NotLoaded
    );
    assert_eq!(
        AssetLoadState::from_resource(Some(&record_with_state(ResourceState::Error)), None, true),
        AssetLoadState::Failed
    );
    assert_eq!(
        AssetLoadState::from_resource(
            Some(&record_with_state(ResourceState::Ready)),
            Some(RuntimeResourceState::Error),
            true
        ),
        AssetLoadState::Failed
    );
    assert_eq!(
        AssetLoadState::from_resource(
            Some(&record_with_state(ResourceState::Reloading)),
            None,
            true
        ),
        AssetLoadState::Reloading
    );
    assert_eq!(
        AssetLoadState::from_resource(
            Some(&record_with_state(ResourceState::Ready)),
            Some(RuntimeResourceState::Reloading),
            true
        ),
        AssetLoadState::Reloading
    );
}

#[test]
fn asset_load_states_classification_helpers_cover_tooling_status_rows() {
    let loaded = AssetLoadStates {
        load_state: AssetLoadState::Loaded,
        dependency_load_state: DependencyLoadState::Loaded,
        recursive_dependency_load_state: RecursiveDependencyLoadState::Loaded,
    };
    assert!(loaded.is_loaded_with_dependencies());
    assert!(!loaded.has_not_loaded_state());
    assert!(!loaded.is_loading_class());
    assert!(!loaded.is_failed());

    let reloading_dependency = AssetLoadStates {
        load_state: AssetLoadState::Loaded,
        dependency_load_state: DependencyLoadState::Loaded,
        recursive_dependency_load_state: RecursiveDependencyLoadState::Reloading,
    };
    assert!(reloading_dependency.is_loaded_with_direct_dependencies());
    assert!(!reloading_dependency.is_loaded_with_dependencies());
    assert!(!reloading_dependency.has_not_loaded_state());
    assert!(reloading_dependency.is_loading_class());
    assert!(!reloading_dependency.is_failed());

    let failed_direct_dependency = AssetLoadStates {
        load_state: AssetLoadState::Loaded,
        dependency_load_state: DependencyLoadState::Failed,
        recursive_dependency_load_state: RecursiveDependencyLoadState::Failed,
    };
    assert!(failed_direct_dependency.is_loaded());
    assert!(!failed_direct_dependency.is_loaded_with_direct_dependencies());
    assert!(!failed_direct_dependency.has_not_loaded_state());
    assert!(!failed_direct_dependency.is_loading_class());
    assert!(failed_direct_dependency.is_failed());

    let missing_root = AssetLoadStates {
        load_state: AssetLoadState::NotLoaded,
        dependency_load_state: DependencyLoadState::NotLoaded,
        recursive_dependency_load_state: RecursiveDependencyLoadState::NotLoaded,
    };
    assert!(!missing_root.is_loaded());
    assert!(missing_root.has_not_loaded_state());
    assert!(!missing_root.is_loading_class());
    assert!(!missing_root.is_failed());
}
