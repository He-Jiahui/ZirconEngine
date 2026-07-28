use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::font_assets::{
    LoadedUiFontAsset, UiFontAssetCacheStatus, effective_text_render_mode, ensure_font_asset_record,
};
use super::resolved_batches::ResolvedScreenSpaceUiTextBatches;
use super::*;
use crate::asset::project::{ProjectManifest, ProjectPaths};
use crate::asset::{AssetManager, AssetUri, FontAsset, ProjectAssetManager};
use crate::core::resource::{ResourceId, ResourceKind, ResourceRecord, ResourceState};
use zircon_runtime_interface::project::RelPath;
use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextWritingMode};

mod prepare_report;

#[cfg(target_os = "windows")]
#[test]
fn screen_space_ui_font_initialization_discovers_system_faces_from_empty_snapshot() {
    let mut font_system = FontSystem::new();
    let mut font_database = FontDatabase::with_default_fallbacks();

    let discovered = initialize_screen_space_ui_font_system(&mut font_system, &mut font_database);

    assert!(discovered > 0);
    assert!(
        font_database
            .match_face(&crate::text::FontQuery::single_family("Segoe UI"))
            .is_some()
    );
    assert!(font_system.db().faces().next().is_some());
}

#[test]
fn text_backend_routing_keeps_explicit_native_out_of_sdf_atlas_batches() {
    let native = text_batch("Normal", UiTextRenderMode::Native);
    let sdf = text_batch("Signed", UiTextRenderMode::Sdf);

    let routed = ResolvedScreenSpaceUiTextBatches::from_explicit_batches(&[native], &[sdf]);

    assert_eq!(routed.native_texts().len(), 1);
    assert_eq!(routed.native_texts()[0].text, "Normal");
    assert_eq!(routed.sdf_texts().len(), 1);
    assert_eq!(routed.sdf_texts()[0].text, "Signed");
}

#[test]
fn text_backend_routing_respects_auto_font_mode_without_crossing_backends() {
    let mut routed = ResolvedScreenSpaceUiTextBatches::default();

    routed.push_resolved_auto_text(
        text_batch("NormalAuto", UiTextRenderMode::Auto),
        UiTextRenderMode::Native,
    );
    routed.push_resolved_auto_text(
        text_batch("SdfAuto", UiTextRenderMode::Auto),
        UiTextRenderMode::Sdf,
    );

    assert_eq!(routed.native_texts().len(), 1);
    assert_eq!(routed.native_texts()[0].text, "NormalAuto");
    assert_eq!(routed.sdf_texts().len(), 1);
    assert_eq!(routed.sdf_texts()[0].text, "SdfAuto");
}

#[test]
fn text_batch_resolution_invalidates_existing_renderer_after_shared_font_publish() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let mut reader = TextRenderState::new(0);
    let mut writer = TextRenderState::new(0);
    let previous_family = reader
        .font_database()
        .default_ui_family_for_test()
        .map(str::to_owned);
    assert!(writer.set_default_ui_family("Shared Screen-Space Refresh Family"));

    let asset_manager = ProjectAssetManager::default();
    let mut font_assets = HashMap::new();
    let resolved = super::resolved_batches::resolve_text_batches(
        &mut reader,
        &mut font_assets,
        &asset_manager,
        &[],
        &[],
        &[],
    );

    assert!(resolved.font_faces_changed());
    assert!(
        !reader.refresh_shared_font_database(),
        "batch resolution must consume the pending shared generation"
    );

    let _ = writer.set_default_ui_family_asset(previous_family.as_deref());
    assert!(reader.refresh_shared_font_database());
}

#[test]
fn text_font_asset_missing_record_is_cached_for_the_current_resource_revision() {
    let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
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
    let source = include_str!("font_assets.rs");

    assert!(!source.contains(".expect("));
    assert!(!source.contains("unreachable!("));
}

#[test]
fn text_font_asset_cache_uses_one_entry_lookup_authority() {
    let source = include_str!("font_assets.rs");
    let ensure = source
        .split_once("pub(super) fn ensure_font_asset_record")
        .map(|(_, ensure)| ensure)
        .expect("font asset cache owner should remain discoverable");

    assert_eq!(ensure.matches("font_assets.entry(").count(), 1);
    assert!(!ensure.contains("font_assets.get("));
}

#[test]
fn native_family_resolution_only_reads_the_prepared_font_cache() {
    let source = include_str!("../text.rs");
    let family_resolver = source
        .split_once("fn resolve_family_name(")
        .and_then(|(_, suffix)| suffix.split_once("fn text_bounds("))
        .map(|(resolver, _)| resolver)
        .expect("native family resolver source should remain discoverable");

    assert!(family_resolver.contains("loaded_asset()"));
    assert!(!family_resolver.contains("ensure_font_asset_record"));
    assert!(!family_resolver.contains("resource_cache_identity"));

    let batch_resolver = include_str!("resolved_batches.rs");
    assert!(
        batch_resolver.contains("text.style.code.then_some(super::DEFAULT_FONT_ASSET)"),
        "code-style native text must refresh its derived default-font dependency"
    );
}

#[test]
fn text_font_asset_error_record_recovers_when_resource_state_changes_at_same_revision() {
    let _shared_font_database = crate::text::font::shared_font_database_test_serial_guard();
    let fixture = RuntimeFontAssetGuard::new("zircon-ui-text-font-error-recovery");
    let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
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
    let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
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
    let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
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
    let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
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
    let mut text_state = TextRenderState::new(NATIVE_BITMAP_ATLAS_RASTER_WORKER_COUNT);
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

#[test]
fn text_font_refresh_recomputes_internal_vertical_advances() {
    let mut text = text_batch("AB", UiTextRenderMode::Sdf);
    text.writing_mode = UiTextWritingMode::VerticalRl;
    text.glyph_advances = vec![999.0, 999.0];

    super::super::render::text_advances::refresh_screen_space_text_batch_glyphs(&mut text);

    assert_eq!(text.glyph_advances.len(), 2);
    assert!(text.glyph_advances.iter().all(|advance| *advance < 999.0));
}

#[test]
fn text_font_refresh_preserves_resolved_layout_vertical_advances() {
    let mut text = text_batch("AB", UiTextRenderMode::Sdf);
    text.writing_mode = UiTextWritingMode::VerticalRl;
    text.source_range = Some(UiTextRange { start: 0, end: 2 });
    text.glyph_advances = vec![11.0, 13.0];

    super::super::render::text_advances::refresh_screen_space_text_batch_glyphs(&mut text);

    assert_eq!(text.glyph_advances, vec![11.0, 13.0]);
}

#[test]
fn auto_text_mode_uses_font_asset_default_when_present() {
    let resolved = effective_text_render_mode(
        UiTextRenderMode::Auto,
        Some(&LoadedUiFontAsset {
            family: Some("Studio Mono".to_string()),
            render_mode: Some(UiTextRenderMode::Sdf),
            composite_font: None,
        }),
    );

    assert_eq!(resolved, UiTextRenderMode::Sdf);
}

#[test]
fn explicit_text_mode_overrides_font_asset_default() {
    let resolved = effective_text_render_mode(
        UiTextRenderMode::Native,
        Some(&LoadedUiFontAsset {
            family: Some("Studio Mono".to_string()),
            render_mode: Some(UiTextRenderMode::Sdf),
            composite_font: None,
        }),
    );

    assert_eq!(resolved, UiTextRenderMode::Native);
}

#[test]
fn auto_text_mode_falls_back_to_native_without_font_asset_default() {
    let resolved = effective_text_render_mode(UiTextRenderMode::Auto, None);

    assert_eq!(resolved, UiTextRenderMode::Native);
}

#[test]
fn native_text_align_maps_start_end_through_text_direction() {
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::LeftToRight),
        NativeTextAlign::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::LeftToRight),
        NativeTextAlign::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::Start, UiTextDirection::RightToLeft),
        NativeTextAlign::Right
    );
    assert_eq!(
        native_text_align(UiTextAlign::End, UiTextDirection::RightToLeft),
        NativeTextAlign::Left
    );
    assert_eq!(
        native_text_align(UiTextAlign::Justify, UiTextDirection::LeftToRight),
        NativeTextAlign::Justified
    );
}

#[test]
fn native_text_area_placement_snaps_fractional_origin_to_device_pixels() {
    let mut text = text_batch("editor base.zui", UiTextRenderMode::Native);
    text.frame = UiFrame::new(12.49, 7.51, 120.0, 20.0);
    text.clip_frame = Some(UiFrame::new(12.2, 7.2, 80.0, 20.0));

    let placement = native_text_area_placement(crate::core::math::UVec2::new(200, 80), &text);

    assert_eq!(placement.left, 12.0);
    assert_eq!(placement.top, 8.0);
    assert_eq!(placement.bounds.left, 12);
    assert_eq!(placement.bounds.top, 7);
    assert_eq!(placement.bounds.right, 93);
    assert_eq!(placement.bounds.bottom, 28);
}

#[test]
fn native_text_area_placement_drops_non_finite_origin_values() {
    let mut text = text_batch("folder-open.svg", UiTextRenderMode::Native);
    text.frame = UiFrame::new(f32::NAN, f32::INFINITY, 120.0, 20.0);

    let placement = native_text_area_placement(crate::core::math::UVec2::new(200, 80), &text);

    assert_eq!(placement.left, 0.0);
    assert_eq!(placement.top, 0.0);
    assert_eq!(placement.bounds.left, 0);
    assert_eq!(placement.bounds.top, 0);
}

fn text_batch(text: &str, mode: UiTextRenderMode) -> ScreenSpaceUiTextBatch {
    ScreenSpaceUiTextBatch {
        text: text.to_string(),
        frame: UiFrame::new(0.0, 0.0, 128.0, 24.0),
        clip_frame: None,
        source_range: None,
        glyph_advances: Vec::new(),
        shaped_glyphs: Vec::new(),
        layout_error: None,
        color: [1.0, 1.0, 1.0, 1.0],
        background_color: None,
        font: Some("res://fonts/default.font.toml".to_string()),
        font_family: Some("Zircon Sans".to_string()),
        language: None,
        font_weight: UiResolvedStyle::DEFAULT_FONT_WEIGHT,
        font_size: 16.0,
        line_height: 20.0,
        text_align: UiTextAlign::Left,
        text_direction: UiTextDirection::LeftToRight,
        writing_mode: UiTextWritingMode::HorizontalTb,
        wrap: UiTextWrap::None,
        style: Default::default(),
        distance_field_mode: match mode {
            UiTextRenderMode::Msdf => crate::text::sdf::SdfMode::Msdf,
            UiTextRenderMode::Mtsdf => crate::text::sdf::SdfMode::Mtsdf,
            UiTextRenderMode::Auto | UiTextRenderMode::Native | UiTextRenderMode::Sdf => {
                crate::text::sdf::SdfMode::Sdf
            }
        },
        text_effects: Default::default(),
        text_decorations: Default::default(),
        text_decoration_baseline: None,
        clip_transform: None,
    }
}

struct TextFontProject {
    root: PathBuf,
    font_root: PathBuf,
}

impl TextFontProject {
    const FONT_REF: &'static str = "res://fonts/late.font.toml";
    const SHARED_FIRST_REF: &'static str = "res://fonts/shared-first.font.toml";
    const SHARED_SECOND_REF: &'static str = "res://fonts/shared-second.font.toml";

    fn new(prefix: &str) -> Self {
        let unique = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        let paths = ProjectPaths::from_root(&root).expect("project paths should build");
        paths
            .ensure_layout(&[RelPath::project_assets()])
            .expect("project layout should exist");
        ProjectManifest::new(
            "UI Text Font Cache",
            AssetUri::parse("res://empty.scene.toml").expect("startup uri should parse"),
            1,
        )
        .save(paths.manifest_path())
        .expect("project manifest should save");
        let font_root = paths.asset_root(&RelPath::project_assets()).join("fonts");
        fs::create_dir_all(&font_root).expect("font directory should exist");
        Self { root, font_root }
    }

    fn write_font_asset(&self) {
        self.write_named_font_asset("late", Some("Late UI Font"), None);
    }

    fn write_default_font_asset(&self, family: Option<&str>, composite_family: Option<&str>) {
        self.write_named_font_asset("default", family, composite_family);
    }

    fn write_shared_font_source(&self) {
        fs::copy(default_font_path(), self.font_root.join("shared.ttf"))
            .expect("shared font fixture should copy");
    }

    fn write_shared_font_manifest(&self, stem: &str) {
        let manifest = "source = \"shared.ttf\"\nfamily = \"Shared UI Font\"\n";
        fs::write(self.font_root.join(format!("{stem}.font.toml")), manifest)
            .expect("shared font manifest should write");
    }

    fn remove_shared_font_manifest(&self, stem: &str) {
        fs::remove_file(self.font_root.join(format!("{stem}.font.toml")))
            .expect("shared font manifest should be removed");
    }

    fn write_named_font_asset(
        &self,
        stem: &str,
        family: Option<&str>,
        composite_family: Option<&str>,
    ) {
        let font_file = format!("{stem}.ttf");
        fs::copy(default_font_path(), self.font_root.join(&font_file))
            .expect("font fixture should copy");
        let mut manifest = format!("source = {font_file:?}\nrender_mode = \"sdf\"\n");
        if let Some(family) = family {
            manifest.push_str(&format!("family = {family:?}\n"));
        }
        if let Some(composite_family) = composite_family {
            manifest.push_str(&format!(
                "\n[composite_font]\ndefault_family = {composite_family:?}\n"
            ));
        }
        fs::write(self.font_root.join(format!("{stem}.font.toml")), manifest)
            .expect("font manifest should write");
    }

    fn remove_font_asset(&self) {
        self.remove_named_font_asset("late");
    }

    fn remove_named_font_asset(&self, stem: &str) {
        fs::remove_file(self.font_root.join(format!("{stem}.font.toml")))
            .expect("remove font manifest");
        fs::remove_file(self.font_root.join(format!("{stem}.ttf"))).expect("remove font source");
    }
}

impl Drop for TextFontProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn default_font_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join("FiraSans-Regular.ttf")
}

struct RuntimeFontAssetGuard {
    asset_ref: String,
    manifest_path: PathBuf,
    source_path: PathBuf,
}

impl RuntimeFontAssetGuard {
    fn new(prefix: &str) -> Self {
        let unique = format!(
            "{prefix}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after unix epoch")
                .as_nanos()
        );
        let font_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("fonts");
        Self {
            asset_ref: format!("res://fonts/{unique}.font.toml"),
            manifest_path: font_root.join(format!("{unique}.font.toml")),
            source_path: font_root.join(format!("{unique}.ttf")),
        }
    }

    fn write(&self) -> FontAsset {
        fs::copy(default_font_path(), &self.source_path).expect("font fixture should copy");
        let source_name = self
            .source_path
            .file_name()
            .and_then(|name| name.to_str())
            .expect("font source name should be utf-8");
        let manifest = format!("source = {source_name:?}\nfamily = \"Recovered UI Font\"\n");
        fs::write(&self.manifest_path, &manifest).expect("font manifest should write");
        FontAsset::from_toml_str(&manifest).expect("font manifest should parse")
    }
}

impl Drop for RuntimeFontAssetGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.manifest_path);
        let _ = fs::remove_file(&self.source_path);
    }
}
