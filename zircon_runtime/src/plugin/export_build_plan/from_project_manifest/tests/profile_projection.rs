use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ExportProfile, ExportTargetPlatform, RuntimeProfileId};

use super::{
    begin_profile_projection_build_observation, ExportProfileSelectionProjection,
    SelectedProfileFeatureIds,
};

#[test]
fn preallocated_profile_feature_indexes_preserve_contract() {
    let feature_ids = vec![
        "rendering.forward".to_string(),
        "deferred.fast".to_string(),
        "deferred".to_string(),
    ];
    let selected = SelectedProfileFeatureIds::from_feature_ids(&feature_ids);

    assert_eq!(selected.qualified.len(), 2);
    assert!(selected.qualified.contains("rendering.forward"));
    assert!(selected.qualified.contains("deferred.fast"));
    assert_eq!(selected.short.len(), 1);
    assert!(selected.short.contains("deferred"));
}

#[test]
fn profile_selection_projection_build_and_lookup_counts_scale_linearly() {
    for plugin_count in [1, 100, 1_000] {
        for features_per_plugin in [1, 10, 100] {
            let plugin_ids = (0..plugin_count)
                .map(|index| format!("plugin_{index}"))
                .collect::<Vec<_>>();
            let mut profile = ExportProfile::new(
                "linear-profile",
                RuntimeTargetMode::ClientRuntime,
                ExportTargetPlatform::Windows,
                RuntimeProfileId::Client3d,
            )
            .with_selected_plugins(plugin_ids.iter().cloned());
            for plugin_id in &plugin_ids {
                profile = profile.with_feature_selection(
                    plugin_id.clone(),
                    (0..features_per_plugin).map(|index| format!("{plugin_id}.feature_{index}")),
                );
            }

            begin_profile_projection_build_observation();
            let projection = ExportProfileSelectionProjection::new(&profile);
            for plugin_id in &plugin_ids {
                assert!(projection.selects_plugin(plugin_id));
                for feature_index in 0..features_per_plugin {
                    assert!(projection.selects_feature(
                        plugin_id,
                        &format!("{plugin_id}.feature_{feature_index}")
                    ));
                }
            }

            let feature_count = plugin_count * features_per_plugin;
            let metrics = projection.metrics();
            assert_eq!(metrics.projection_builds, 1);
            assert_eq!(metrics.selected_plugin_rows_indexed, plugin_count);
            assert_eq!(metrics.selected_feature_owner_rows_indexed, plugin_count);
            assert_eq!(metrics.selected_feature_rows_indexed, feature_count);
            assert_eq!(metrics.lookup_probes, plugin_count + feature_count);
        }
    }
}

#[test]
fn feature_projection_search_does_not_allocate_normalized_ids() {
    let source = include_str!("../profile_projection.rs");
    let allocating_helper = ["normalize_", "profile_feature_id"].concat();
    assert!(
        !source.contains(&allocating_helper),
        "feature matching should compare qualified and short ids without formatting a String"
    );
}

#[test]
fn feature_projection_matches_short_and_qualified_ids_without_normalizing() {
    let profile = ExportProfile::new(
        "feature-matching",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client3d,
    )
    .with_feature_selection(
        "rendering",
        [
            "rendering.forward".to_string(),
            "deferred.fast".to_string(),
            "deferred".to_string(),
        ],
    );
    let projection = ExportProfileSelectionProjection::new(&profile);

    assert!(projection.selects_feature("rendering", "rendering.deferred"));
    assert!(projection.selects_feature("rendering", "rendering.forward"));
    assert!(!projection.selects_feature("rendering", "rendering.deferred.fast"));
    assert!(!projection.selects_feature("rendering", "rendering.shadow"));
    assert!(!projection.selects_feature("rendering", "other.deferred"));
}
