use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use super::super::font_assets::{
    EnsuredUiFontAsset, UiFontAssetCache, UiFontAssetCacheStatus, UiFontAssetLoadError,
    ensure_font_asset_record as ensure_font_asset_record_with_claims, font_asset_cache_report,
};
use super::super::*;
use super::support::{RuntimeFontAssetGuard, TextFontProject};
use crate::asset::assets::FontSourceBudgetError;
use crate::asset::{AssetManager, AssetUri, ProjectAssetManager};
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord, ResourceState};
use crate::text::font::FontDatabaseError;
use crate::text::font::{
    FontCollectionService, FontLoadError, FontLoadIoFailure, RuntimeFontAssetClaimScope,
    RuntimeFontAssetClaimUpdateReport,
};

const TEST_RASTER_WORKER_COUNT: usize = 1;

fn test_text_render_state() -> TextRenderState {
    TextRenderState::new_with_font_collection(
        TEST_RASTER_WORKER_COUNT,
        FontCollectionService::new(),
    )
}

struct TestFontAssetClaims {
    active: Vec<std::sync::Arc<str>>,
    scope: RuntimeFontAssetClaimScope,
}

impl TestFontAssetClaims {
    fn new(text_state: &TextRenderState) -> Self {
        Self {
            active: Vec::new(),
            scope: text_state
                .font_collection()
                .runtime_font_asset_claim_scope(),
        }
    }

    fn ensure<'a>(
        &mut self,
        text_state: &mut TextRenderState,
        font_assets: &'a mut UiFontAssetCache,
        asset_manager: &ProjectAssetManager,
        asset_ref: &str,
    ) -> EnsuredUiFontAsset<'a> {
        ensure_font_asset_record_with_claims(
            text_state,
            font_assets,
            asset_manager,
            asset_ref,
            &mut self.active,
            &mut self.scope,
        )
    }

    fn release(&mut self, asset_ref: &str) -> RuntimeFontAssetClaimUpdateReport {
        self.active.retain(|active| active.as_ref() != asset_ref);
        self.scope.replace_shared_claims(&self.active)
    }
}

#[test]
fn text_font_asset_registration_maps_source_read_failure_to_stable_io_cause() {
    let mapped = UiFontAssetLoadError::from_database_error(FontDatabaseError::ReadFailed {
        path: PathBuf::from("missing.ttf"),
        source: io::Error::from(io::ErrorKind::NotFound),
    });

    assert_eq!(
        mapped,
        UiFontAssetLoadError::SourceReadFailed(FontLoadIoFailure::NotFound)
    );
}

#[test]
fn text_font_asset_registration_preserves_source_budget_failure() {
    let source = FontSourceBudgetError::SourceBytes {
        limit_bytes: 64,
        actual_bytes: 65,
    };
    let mapped = UiFontAssetLoadError::from_database_error(FontDatabaseError::SourceBudget {
        path: PathBuf::from("oversized.ttf"),
        source,
    });

    assert_eq!(mapped, UiFontAssetLoadError::SourceBudgetExceeded(source));
}

#[test]
fn text_font_asset_missing_record_is_cached_for_the_current_resource_revision() {
    let mut text_state = test_text_render_state();
    let mut claims = TestFontAssetClaims::new(&text_state);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    let missing = "res://fonts/late-project-font.font.toml";

    let first = claims.ensure(&mut text_state, &mut font_assets, &asset_manager, missing);
    assert_eq!(first.status, UiFontAssetCacheStatus::Missing);
    assert!(!first.cache_hit);
    assert!(first.record.is_none());

    let second = claims.ensure(&mut text_state, &mut font_assets, &asset_manager, missing);
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
    let refresh = source
        .split_once("pub(super) fn refresh_font_asset_records")
        .and_then(|(_, refresh)| refresh.split_once("#[cfg(test)]"))
        .map(|(refresh, _)| refresh)
        .expect("batch font asset cache owner should remain discoverable");

    assert!(refresh.contains("font_assets\n                .get("));
    assert!(!refresh.contains("font_assets.entry("));
}

#[test]
fn native_text_prepare_uses_prepared_glyph_runs_without_a_layout_backend() {
    let source = include_str!("../../text.rs");
    assert!(source.contains("native_bitmap_atlas_glyph_runs"));
    assert!(!source.contains("TextArea"));
    assert!(!source.contains("TextRenderer"));
    assert!(!source.contains("TextAtlas"));
    assert!(!source.contains("layout_runs()"));

    let segment_cache = include_str!("../segment_cache.rs");
    assert!(
        segment_cache.contains("if text.style.code"),
        "code-style native text must refresh its derived default-font dependency"
    );
}

#[test]
fn text_font_asset_error_record_recovers_when_resource_state_changes_at_same_revision() {
    let fixture = RuntimeFontAssetGuard::new("zircon-ui-text-font-error-recovery");
    let mut text_state = test_text_render_state();
    let mut claims = TestFontAssetClaims::new(&text_state);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    let asset_ref = fixture.asset_ref.as_str();
    let locator = AssetUri::parse(asset_ref).expect("font locator should parse");
    let asset_id = ResourceId::from_locator(&locator);
    let resources = asset_manager.resource_manager();
    let ready_record =
        ResourceRecord::new(asset_id, ResourceKind::Font, locator).with_state(ResourceState::Error);
    let ready_revision = ready_record.revision;
    resources.register_record(ready_record.clone()).unwrap();

    let first = claims.ensure(&mut text_state, &mut font_assets, &asset_manager, asset_ref);
    assert_eq!(first.status, UiFontAssetCacheStatus::Error);
    assert!(!first.cache_hit);
    assert!(first.record.is_none());
    assert!(matches!(
        first.failure,
        Some(UiFontAssetLoadError::Source(
            FontLoadError::ManifestReadFailed(FontLoadIoFailure::NotFound)
        ))
    ));
    let error_report = font_asset_cache_report(&font_assets);
    assert_eq!(error_report.error_count, 1);
    assert_eq!(error_report.source_not_found_count, 1);
    assert_eq!(error_report.ready_count, 0);
    assert_eq!(error_report.missing_count, 0);

    let second = claims.ensure(&mut text_state, &mut font_assets, &asset_manager, asset_ref);
    assert_eq!(second.status, UiFontAssetCacheStatus::Error);
    assert!(second.cache_hit);
    assert!(second.record.is_none());
    assert!(matches!(
        second.failure,
        Some(UiFontAssetLoadError::Source(
            FontLoadError::ManifestReadFailed(FontLoadIoFailure::NotFound)
        ))
    ));
    assert_eq!(font_assets.len(), 1);

    let ready_asset = fixture.write();
    resources
        .start_reload(asset_id, Vec::new())
        .expect("failed font resource should start recovery");
    resources.register_ready(ready_record, ready_asset).unwrap();
    assert_eq!(
        resources
            .registry()
            .get(asset_id)
            .expect("recovered font resource should remain registered")
            .revision,
        ready_revision,
        "transient state recovery should not require a metadata revision change"
    );
    let recovered = claims.ensure(&mut text_state, &mut font_assets, &asset_manager, asset_ref);
    assert_eq!(recovered.status, UiFontAssetCacheStatus::Ready);
    assert!(!recovered.cache_hit);
    assert!(recovered.record.is_some());
    assert!(recovered.loaded);
    assert!(recovered.failure.is_none());
    let ready_report = font_asset_cache_report(&font_assets);
    assert_eq!(ready_report.ready_count, 1);
    assert_eq!(ready_report.error_count, 0);

    let cleanup = claims.release(asset_ref);
    assert!(cleanup.font_inputs_changed);
    assert!(text_state.refresh_font_collection());
}

#[test]
fn text_default_font_revision_refreshes_family_and_composite_projection() {
    let project = TextFontProject::new("zircon-ui-text-default-font-refresh");
    project.write_default_font_asset(Some("First UI Family"), Some("First Composite Family"));
    let mut text_state = test_text_render_state();
    let mut claims = TestFontAssetClaims::new(&text_state);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("first default font revision should publish");

    let first = claims.ensure(
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
    let second = claims.ensure(
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
    let cleared = claims.ensure(
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
    let removed = claims.ensure(
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
        Some("Fira Mono"),
        "removing the project default must restore the packaged Runtime family"
    );
    assert_eq!(
        text_state.font_database().project_composite_font_for_test(),
        None
    );
}

#[test]
fn text_font_asset_negative_cache_recovers_after_project_revision_is_published() {
    let project = TextFontProject::new("zircon-ui-text-font-negative-recovery");
    let mut text_state = test_text_render_state();
    let mut claims = TestFontAssetClaims::new(&text_state);
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("empty project should open");

    let missing = claims.ensure(
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

    let recovered = claims.ensure(
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

    let cached = claims.ensure(
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
    let cleanup = claims.release(TextFontProject::FONT_REF);
    assert!(cleanup.font_inputs_changed);
    assert!(text_state.refresh_font_collection());
}

#[test]
fn text_font_asset_ready_to_missing_retires_faces_and_publishes_generation() {
    let project = TextFontProject::new("zircon-ui-text-font-ready-to-missing");
    project.write_font_asset();
    let mut text_state = test_text_render_state();
    let mut claims = TestFontAssetClaims::new(&text_state);
    let initial_face_count = text_state.face_count();
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("font project should open");

    let ready = claims.ensure(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::FONT_REF,
    );
    assert_eq!(ready.status, UiFontAssetCacheStatus::Ready);
    assert!(ready.faces_changed);
    assert_eq!(text_state.face_count(), initial_face_count + 1);
    let ready_generation = text_state.font_collection_revision().generation();

    project.remove_font_asset();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("font removal revision should publish");
    let missing = claims.ensure(
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
        text_state.font_collection_revision().generation(),
        ready_generation + 1,
        "retiring the last owner must publish exactly one database generation"
    );

    let cached = claims.ensure(
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
    let project = TextFontProject::new("zircon-ui-text-font-shared-owner-mapping");
    project.write_shared_font_source();
    project.write_shared_font_manifest("shared-second");
    let mut text_state = test_text_render_state();
    let mut claims = TestFontAssetClaims::new(&text_state);
    let initial_face_count = text_state.face_count();
    let mut font_assets = HashMap::new();
    let asset_manager = ProjectAssetManager::default();
    asset_manager
        .open_project(project.root.to_string_lossy().as_ref())
        .expect("shared font project should open");

    let missing_first = claims.ensure(
        &mut text_state,
        &mut font_assets,
        &asset_manager,
        TextFontProject::SHARED_FIRST_REF,
    );
    assert_eq!(missing_first.status, UiFontAssetCacheStatus::Missing);
    assert!(!missing_first.faces_changed);

    let ready_second = claims.ensure(
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
    let ready_first = claims.ensure(
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
    let missing_first = claims.ensure(
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

    let removed_second = claims.release(TextFontProject::SHARED_SECOND_REF);
    assert!(removed_second.font_inputs_changed);
    assert!(text_state.refresh_font_collection());
    assert_eq!(
        text_state.face_count(),
        initial_face_count,
        "removing the last owner should retire the shared physical face"
    );
}
