use std::collections::HashMap;

use super::super::font_assets::{UiFontAssetCacheStatus, ensure_font_asset_record};
use super::super::*;
use super::support::{RuntimeFontAssetGuard, TextFontProject};
use crate::asset::{AssetManager, AssetUri, ProjectAssetManager};
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord, ResourceState};

const TEST_RASTER_WORKER_COUNT: usize = 1;

#[test]
fn text_font_asset_missing_record_is_cached_for_the_current_resource_revision() {
    let mut text_state = TextRenderState::new(TEST_RASTER_WORKER_COUNT);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    let missing = "res://fonts/late-project-font.font.toml";

    let first =
        ensure_font_asset_record(&mut text_state, &mut font_assets, &asset_manager, missing);
    assert_eq!(first.status, UiFontAssetCacheStatus::Missing);
    assert!(!first.cache_hit);
    assert!(first.record.is_none());

    let second =
        ensure_font_asset_record(&mut text_state, &mut font_assets, &asset_manager, missing);
    assert_eq!(second.status, UiFontAssetCacheStatus::Missing);
    assert!(second.cache_hit);
    assert!(second.record.is_none());
    assert!(!second.loaded);
    assert!(!second.faces_changed);
    assert_eq!(font_assets.len(), 1);
}

#[test]
fn text_font_asset_cache_path_has_no_production_panic_fallbacks() {
    let source = include_str!("../font_assets.rs");

    assert!(!source.contains(".expect("));
    assert!(!source.contains("unreachable!("));
}

#[test]
fn text_font_asset_cache_uses_one_entry_lookup_authority() {
    let source = include_str!("../font_assets.rs");
    let ensure = source
        .split_once("pub(super) fn ensure_font_asset_record")
        .map(|(_, ensure)| ensure)
        .expect("font asset cache owner should remain discoverable");

    assert_eq!(ensure.matches("font_assets.entry(").count(), 1);
    assert!(!ensure.contains("font_assets.get("));
}

#[test]
fn native_family_resolution_only_reads_the_prepared_font_cache() {
    let source = include_str!("../../text.rs");
    let family_resolver = source
        .split_once("fn resolve_family_name(")
        .and_then(|(_, suffix)| suffix.split_once("fn text_bounds("))
        .map(|(resolver, _)| resolver)
        .expect("native family resolver source should remain discoverable");

    assert!(family_resolver.contains("loaded_asset()"));
    assert!(!family_resolver.contains("ensure_font_asset_record"));
    assert!(!family_resolver.contains("resource_cache_identity"));

    let batch_resolver = include_str!("../resolved_batches.rs");
    assert!(
        batch_resolver.contains("text.style.code.then_some(super::DEFAULT_FONT_ASSET)"),
        "code-style native text must refresh its derived default-font dependency"
    );
}

#[test]
fn text_font_asset_error_record_recovers_when_resource_state_changes_at_same_revision() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let fixture = RuntimeFontAssetGuard::new("zircon-ui-text-font-error-recovery");
    let mut text_state = TextRenderState::new(TEST_RASTER_WORKER_COUNT);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    let asset_ref = fixture.asset_ref.as_str();
    let locator = AssetUri::parse(asset_ref).expect("font locator should parse");
    let asset_id = ResourceId::from_locator(&locator);
    let resources = asset_manager.resource_manager();
    let ready_record =
        ResourceRecord::new(asset_id, ResourceKind::Font, locator).with_state(ResourceState::Error);
    let ready_revision = ready_record.revision;
    resources.register_record(ready_record.clone());

    let first =
        ensure_font_asset_record(&mut text_state, &mut font_assets, &asset_manager, asset_ref);
    assert_eq!(first.status, UiFontAssetCacheStatus::Error);
    assert!(!first.cache_hit);
    assert!(first.record.is_none());

    let second =
        ensure_font_asset_record(&mut text_state, &mut font_assets, &asset_manager, asset_ref);
    assert_eq!(second.status, UiFontAssetCacheStatus::Error);
    assert!(second.cache_hit);
    assert!(second.record.is_none());
    assert_eq!(font_assets.len(), 1);

    let ready_asset = fixture.write();
    resources
        .start_reload(asset_id, Vec::new())
        .expect("failed font resource should start recovery");
    resources.register_ready(ready_record, ready_asset);
    assert_eq!(
        resources
            .registry()
            .get(asset_id)
            .expect("recovered font resource should remain registered")
            .revision,
        ready_revision,
        "transient state recovery should not require a metadata revision change"
    );
    let recovered =
        ensure_font_asset_record(&mut text_state, &mut font_assets, &asset_manager, asset_ref);
    assert_eq!(recovered.status, UiFontAssetCacheStatus::Ready);
    assert!(!recovered.cache_hit);
    assert!(recovered.record.is_some());
    assert!(recovered.loaded);

    let cleanup = text_state.remove_font_asset(asset_ref);
    assert!(cleanup.database_changed);
    assert!(cleanup.asset_mapping_changed);
}

#[test]
fn text_default_font_revision_refreshes_family_and_composite_projection() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let project = TextFontProject::new("zircon-ui-text-default-font-refresh");
    project.write_default_font_asset(Some("First UI Family"), Some("First Composite Family"));
    let mut text_state = TextRenderState::new(TEST_RASTER_WORKER_COUNT);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("first default font revision should publish");

    let first = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        DEFAULT_FONT_ASSET,
    );
    assert_eq!(first.status, UiFontAssetCacheStatus::Ready);
    assert_eq!(
        text_state.font_database().default_ui_family_for_test(),
        Some("First UI Family")
    );
    assert_eq!(
        text_state
            .font_database()
            .project_composite_font_for_test()
            .map(|composite| composite.default_family.as_str()),
        Some("First Composite Family")
    );

    project.write_default_font_asset(Some("Second UI Family"), Some("Second Composite Family"));
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("second default font revision should publish");
    let second = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        DEFAULT_FONT_ASSET,
    );
    assert_eq!(second.status, UiFontAssetCacheStatus::Ready);
    assert!(!second.cache_hit);
    assert_eq!(
        text_state.font_database().default_ui_family_for_test(),
        Some("Second UI Family")
    );
    assert_eq!(
        text_state
            .font_database()
            .project_composite_font_for_test()
            .map(|composite| composite.default_family.as_str()),
        Some("Second Composite Family")
    );

    project.write_default_font_asset(None, None);
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("family-less default font revision should publish");
    let cleared = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        DEFAULT_FONT_ASSET,
    );
    assert_eq!(cleared.status, UiFontAssetCacheStatus::Ready);
    assert!(!cleared.cache_hit);
    assert_eq!(
        text_state.font_database().default_ui_family_for_test(),
        Some("Fira Sans"),
        "a family-less project manifest should publish the family parsed from its source face"
    );
    assert_eq!(
        text_state.font_database().project_composite_font_for_test(),
        None
    );

    project.remove_named_font_asset("default");
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("default font removal revision should publish");
    let removed = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        DEFAULT_FONT_ASSET,
    );
    assert_eq!(removed.status, UiFontAssetCacheStatus::Missing);
    assert!(!removed.cache_hit);
    assert!(removed.faces_changed);
    assert_eq!(
        text_state.font_database().default_ui_family_for_test(),
        None
    );
    assert_eq!(
        text_state.font_database().project_composite_font_for_test(),
        None
    );
}

#[test]
fn text_font_asset_negative_cache_recovers_after_project_revision_is_published() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let project = TextFontProject::new("zircon-ui-text-font-negative-recovery");
    let mut text_state = TextRenderState::new(TEST_RASTER_WORKER_COUNT);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("empty project should open");

    let missing = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::FONT_REF,
    );
    assert_eq!(missing.status, UiFontAssetCacheStatus::Missing);
    assert!(!missing.cache_hit);

    project.write_font_asset();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("project font revision should publish");

    let recovered = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::FONT_REF,
    );
    assert_eq!(recovered.status, UiFontAssetCacheStatus::Ready);
    assert!(!recovered.cache_hit);
    assert!(recovered.record.is_some());
    assert!(recovered.loaded);
    assert!(recovered.faces_changed);

    let cached = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::FONT_REF,
    );
    assert_eq!(cached.status, UiFontAssetCacheStatus::Ready);
    assert!(cached.cache_hit);
    assert!(cached.record.is_some());
    assert!(!cached.loaded);
    assert!(!cached.faces_changed);
    let cleanup = text_state.remove_font_asset(TextFontProject::FONT_REF);
    assert!(cleanup.asset_mapping_changed);
}

#[test]
fn text_font_asset_ready_to_missing_retires_faces_and_publishes_generation() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let project = TextFontProject::new("zircon-ui-text-font-ready-to-missing");
    project.write_font_asset();
    let mut text_state = TextRenderState::new(TEST_RASTER_WORKER_COUNT);
    let _ = text_state.remove_font_asset(TextFontProject::FONT_REF);
    let initial_face_count = text_state.face_count();
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("font project should open");

    let ready = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::FONT_REF,
    );
    assert_eq!(ready.status, UiFontAssetCacheStatus::Ready);
    assert!(ready.faces_changed);
    assert_eq!(text_state.face_count(), initial_face_count + 1);
    let ready_generation = crate::text::font::shared_font_database_generation();

    project.remove_font_asset();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("font removal revision should publish");
    let missing = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::FONT_REF,
    );

    assert_eq!(missing.status, UiFontAssetCacheStatus::Missing);
    assert!(!missing.cache_hit);
    assert!(!missing.loaded);
    assert!(missing.record.is_none());
    assert!(missing.faces_changed);
    assert_eq!(text_state.face_count(), initial_face_count);
    assert_eq!(
        crate::text::font::shared_font_database_generation(),
        ready_generation + 1,
        "retiring the last owner must publish exactly one database generation"
    );

    let cached = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::FONT_REF,
    );
    assert_eq!(cached.status, UiFontAssetCacheStatus::Missing);
    assert!(cached.cache_hit);
    assert!(!cached.faces_changed);
}

#[test]
fn text_font_asset_shared_face_owner_mapping_changes_trigger_invalidation() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let project = TextFontProject::new("zircon-ui-text-font-shared-owner-mapping");
    project.write_shared_font_source();
    project.write_shared_font_manifest("shared-second");
    let mut text_state = TextRenderState::new(TEST_RASTER_WORKER_COUNT);
    let initial_face_count = text_state.face_count();
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("shared font project should open");

    let missing_first = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::SHARED_FIRST_REF,
    );
    assert_eq!(missing_first.status, UiFontAssetCacheStatus::Missing);
    assert!(!missing_first.faces_changed);

    let ready_second = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::SHARED_SECOND_REF,
    );
    assert_eq!(ready_second.status, UiFontAssetCacheStatus::Ready);
    assert!(ready_second.faces_changed);
    assert_eq!(
        text_state.face_count(),
        initial_face_count + 1,
        "the first ready owner should register one shared physical face"
    );

    project.write_shared_font_manifest("shared-first");
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("first shared owner revision should publish");
    let ready_first = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::SHARED_FIRST_REF,
    );
    assert_eq!(ready_first.status, UiFontAssetCacheStatus::Ready);
    assert!(ready_first.faces_changed);
    assert_eq!(
        text_state.face_count(),
        initial_face_count + 1,
        "attaching an asset owner to an existing face must not duplicate the face"
    );

    project.remove_shared_font_manifest("shared-first");
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("first shared owner removal should publish");
    let missing_first = ensure_font_asset_record(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::SHARED_FIRST_REF,
    );
    assert_eq!(missing_first.status, UiFontAssetCacheStatus::Missing);
    assert!(missing_first.faces_changed);
    assert_eq!(
        text_state.face_count(),
        initial_face_count + 1,
        "the remaining asset owner must keep the shared face active"
    );

    let removed_second = text_state.remove_font_asset(TextFontProject::SHARED_SECOND_REF);
    assert!(removed_second.database_changed);
    assert!(removed_second.asset_mapping_changed);
    assert_eq!(
        text_state.face_count(),
        initial_face_count,
        "removing the last owner should retire the shared physical face"
    );
}
