#[test]
fn asset_management_header_queries_follow_publication_and_preserve_retained_overview() {
    use super::*;
    use crate::asset::AssetManagementFamilyKind;

    let manager = ProjectAssetManager::default();
    let generation = ProjectAssetManagementGeneration::from_record_sets(
        Some(1),
        Some(
            manager
                .resource_manager()
                .management_generation()
                .identity(),
        ),
        ModelAssetManagementRecordSet::from_records(Vec::new()),
        MeshAssetManagementRecordSet::from_results(vec![
            (
                ResourceId::from_stable_label("management-header:first"),
                Err(MeshValidationError::MissingPositionAttribute),
            ),
            (
                ResourceId::from_stable_label("management-header:second"),
                Err(MeshValidationError::MissingPositionAttribute),
            ),
        ]),
        SceneAssetManagementRecordSet::from_records(Vec::new()),
        SceneEntityManagementRecordSet::from_records(Vec::new()),
        MaterialAssetManagementRecordSet::from_records(Vec::new()),
        ShaderAssetManagementRecordSet::from_records(Vec::new()),
    );
    {
        let _publication = manager.project_generation_write();
        manager.install_asset_management_generation(generation);
    }
    let retained = manager.current_asset_management_generation();
    let expected = manager.asset_management_record_sets().overview();
    assert_eq!(expected.summary.managed_record_count, 2);
    assert_eq!(expected.summary.degraded_record_count, 2);
    assert_eq!(
        expected.family_status_index.degraded,
        vec![AssetManagementFamilyKind::Mesh]
    );
    assert_eq!(retained.overview(), &expected);
    assert_eq!(manager.asset_management_overview(), expected);
    assert_eq!(
        manager.asset_management_family_summaries(),
        expected.families
    );
    assert_eq!(
        manager.asset_management_family_status_index(),
        expected.family_status_index
    );
    assert_eq!(
        manager.asset_management_family_status_view(AssetManagementFamilyStatus::Degraded),
        expected.family_status_view(AssetManagementFamilyStatus::Degraded)
    );
    assert_eq!(
        manager.asset_management_family_issue_index(),
        expected.family_issue_index
    );
    assert_eq!(
        manager.asset_management_family_issue_view(AssetManagementFamilyIssueBucket::WithIssues),
        expected.family_issue_view(AssetManagementFamilyIssueBucket::WithIssues)
    );

    {
        let _publication = manager.project_generation_write();
        manager.install_asset_management_generation(ProjectAssetManagementGeneration::empty());
    }
    assert_eq!(
        manager
            .asset_management_overview()
            .summary
            .managed_record_count,
        0
    );
    assert!(manager
        .asset_management_family_status_index()
        .degraded
        .is_empty());
    assert!(manager
        .asset_management_family_issue_index()
        .with_issues
        .is_empty());
    assert_eq!(retained.overview(), &expected);
}

#[cfg(feature = "profiling")]
use crate::core::runtime::diagnostics::profiling::{
    reset_capture, snapshot, start_capture, test_capture_lock, ProfileCaptureConfig,
    ProfileFrameScope,
};

#[test]
fn asset_management_kind_lookup_reads_the_published_asset_generation() {
    let source = include_str!("../management.rs");
    let kind_lookup = source
        .split("fn asset_ids_by_kind")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn asset_ids_for_management_record_sets")
                .next()
        })
        .expect("read asset management kind lookup");

    assert!(kind_lookup.contains("current_asset_management_generation()"));
    assert!(kind_lookup.contains("ids_by_kind(kind)"));
    assert!(!kind_lookup.contains(".management_generation()"));
    assert!(!kind_lookup.contains("ResourceManagementQuery"));
    assert!(!kind_lookup.contains("scan.next_row()"));
    assert!(!kind_lookup.contains(".registry()"));
    assert!(!kind_lookup.contains("list_resources("));
    assert!(!kind_lookup.contains("ids.sort()"));
    assert!(!kind_lookup.contains("sort_by("));
    assert!(!kind_lookup.contains("sort_by_key("));
    assert!(!kind_lookup.contains("sort_unstable"));
}

#[test]
fn asset_management_aggregate_derives_scene_entities_from_one_scene_projection() {
    let source = include_str!("../management.rs");
    let aggregate = source
        .split("fn build_asset_management_record_sets")
        .nth(1)
        .and_then(|source| {
            source
                .split("pub(crate) fn refresh_asset_management_generation")
                .next()
        })
        .expect("read asset management aggregate implementation");

    assert_eq!(
        aggregate
            .matches("self.scene_asset_management_records_for_ids(ids.scenes)")
            .count(),
        1
    );
    assert!(aggregate.contains("SceneAssetManagementRecord::entity_management_records"));
    assert!(aggregate.contains("SceneAssetManagementRecordSet::from_records(scene_records)"));
    assert!(aggregate.contains("SceneEntityManagementRecordSet::from_records(scene_entities)"));
    assert_eq!(
        aggregate
            .matches("asset_ids_for_management_record_sets(generation)")
            .count(),
        1
    );
    assert!(!aggregate.contains("self.model_asset_management_record_set()"));
    assert!(!aggregate.contains("self.mesh_asset_management_record_set()"));
    assert!(!aggregate.contains("self.material_asset_management_record_set()"));
    assert!(!aggregate.contains("self.shader_asset_management_record_set()"));
}

#[test]
fn management_records_read_the_project_asset_generation_snapshot() {
    let source = include_str!("../management.rs");
    let records = source
        .split("impl ProjectAssetManager")
        .nth(1)
        .and_then(|source| source.split("fn build_asset_management_record_sets").next())
        .expect("read management accessors");

    for accessor in [
        ".model_records()",
        ".scene_records()",
        ".scene_entity_records()",
        ".material_records()",
        ".shader_records()",
    ] {
        assert!(
            records.contains(accessor),
            "missing snapshot accessor {accessor}"
        );
    }
    assert!(!records.contains("self.registry()"));
    assert!(!records.contains("list_resources("));
}

#[test]
fn refresh_skips_unchanged_resource_generations() {
    let source = include_str!("../management.rs");
    let refresh = source
        .split("pub(crate) fn refresh_asset_management_generation")
        .nth(1)
        .and_then(|source| source.split("pub fn asset_management_record_sets").next())
        .expect("read asset management refresh owner");

    assert!(refresh.contains("is_for_generations"));
    assert!(refresh.contains("return;"));
    assert!(refresh.contains("management_generation()"));
}

#[test]
fn astra_m10_management_scan_uses_the_captured_publication() {
    use crate::asset::AssetUri;
    use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord};

    let manager = super::ProjectAssetManager::default();
    let captured = manager.resource_manager().management_generation();
    let existing_models = manager
        .asset_ids_for_management_record_sets(&captured)
        .models;
    let uri = AssetUri::parse("res://astra-management/model.glb").unwrap();
    let id = ResourceId::from_locator(&uri);
    assert!(!existing_models.contains(&id));
    manager
        .resource_manager()
        .register_lazy_records(vec![ResourceRecord::new(id, ResourceKind::Model, uri)])
        .unwrap();
    let published = manager.resource_manager().management_generation();

    assert_ne!(captured.identity(), published.identity());
    assert_eq!(
        manager
            .asset_ids_for_management_record_sets(&captured)
            .models,
        existing_models
    );
    let mut expected = existing_models;
    expected.push(id);
    expected.sort_unstable();
    let mut actual = manager
        .asset_ids_for_management_record_sets(&published)
        .models;
    actual.sort_unstable();
    assert_eq!(actual, expected);
}

#[test]
fn refresh_clears_the_asset_projection_when_no_project_is_active() {
    let source = include_str!("../management.rs");
    let refresh = source
        .split("pub(crate) fn refresh_asset_management_generation")
        .nth(1)
        .and_then(|source| source.split("pub fn asset_management_record_sets").next())
        .expect("read asset management refresh owner");

    let clear = refresh
        .find("has_project_generation()")
        .expect("refresh must inspect published project identity");
    let empty = refresh
        .find("ProjectAssetManagementGeneration::empty()")
        .expect("refresh must install empty closed-project projection");
    assert!(clear < empty);
}

#[cfg(feature = "profiling")]
#[test]
fn asset_management_record_sets_reuse_the_published_projection_in_the_active_frame() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "asset-management-kind-query".to_owned();
    config.max_frames = 4;
    config.max_spans = 16;
    config.max_counters = 32;
    start_capture(config);

    {
        let _frame = ProfileFrameScope::enter("runtime", "asset_management");
        let manager = super::ProjectAssetManager::default();
        let _records = manager.asset_management_record_sets();
    }

    let profile = snapshot();
    reset_capture();

    assert!(profile
        .counters
        .iter()
        .all(|counter| !counter.name.starts_with("resource_management.")));
    assert!(profile
        .spans
        .iter()
        .all(|span| span.category != "resource_management"));
}

#[cfg(feature = "profiling")]
#[test]
fn asset_management_record_sets_do_not_emit_without_an_active_profile_frame() {
    let _guard = test_capture_lock();
    let mut config = ProfileCaptureConfig::default();
    config.session_id = "asset-management-no-frame".to_owned();
    config.max_spans = 16;
    config.max_counters = 32;
    start_capture(config);

    let manager = super::ProjectAssetManager::default();
    let _records = manager.asset_management_record_sets();

    let profile = snapshot();
    reset_capture();

    assert!(profile
        .counters
        .iter()
        .all(|counter| !counter.name.starts_with("resource_management.")));
    assert!(profile
        .spans
        .iter()
        .all(|span| span.category != "resource_management"));
}
