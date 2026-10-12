use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginFeatureSelection;

use super::owner_feature_selection_is_complete;

fn complete_selection() -> ProjectPluginFeatureSelection {
    ProjectPluginFeatureSelection::new("rendering.deferred")
        .with_runtime_crate("zircon_plugin_rendering_deferred_runtime")
        .with_editor_crate("zircon_plugin_rendering_deferred_editor")
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
}

#[test]
fn completed_owner_feature_selection_skips_catalog_projection_requirements() {
    assert!(owner_feature_selection_is_complete(
        &complete_selection(),
        None
    ));
}

#[test]
fn provider_requirement_prevents_incomplete_fast_path() {
    let selection = complete_selection();
    assert!(!owner_feature_selection_is_complete(
        &selection,
        Some("rendering_deferred_provider")
    ));

    let selection = selection.with_provider_package_id("custom_provider");
    assert!(owner_feature_selection_is_complete(
        &selection,
        Some("rendering_deferred_provider")
    ));
}
