use std::cell::{Cell, RefCell};
use std::path::Path;

use super::*;
use crate::core::framework::text::{TextGlyphRotation, TextOpenTypeFeature};
use crate::text::font::{
    font_handle_registry_report, force_publish_shared_font_database, resolve_font_face_handle,
    runtime_default_font_database_for_test, shared_font_database_snapshot,
    shared_font_database_test_serial_guard, FontDatabase,
};
use crate::text::{
    ShapedGlyph, ShapedGlyphRotation, TextVerticalGlyphDecisionBasis,
    TextVerticalGlyphFallbackReason, TextVerticalGlyphFeatureSet, TextVerticalGlyphOrientation,
    TextVerticalGlyphSubstitution,
};

struct SharedFontDatabaseRestore(FontDatabase);

impl Drop for SharedFontDatabaseRestore {
    fn drop(&mut self) {
        force_publish_shared_font_database(&self.0);
    }
}

#[test]
fn production_text_layout_service_shapes_through_neutral_contract() {
    let before = shared_text_layout_generation_retry_report();
    let font = TextFontRequest {
        size: 16.0,
        ..TextFontRequest::default()
    };
    let result = shared_text_layout_service()
        .shape(TextShapeRequest::new("Zircon", font))
        .expect("production text service should shape a neutral request");

    assert!(!result.runs.is_empty());
    assert!(result.metrics.width > 0.0);
    assert!(result.runs.iter().any(|run| !run.glyphs.is_empty()));
    let after = shared_text_layout_generation_retry_report();
    let glyph_count = result
        .runs
        .iter()
        .map(|run| run.glyphs.len())
        .sum::<usize>() as u64;
    assert!(after.neutral_projection_count > before.neutral_projection_count);
    assert!(
        after.neutral_projection_glyph_count >= before.neutral_projection_glyph_count + glyph_count
    );
    assert!(after.neutral_projection_bytes > before.neutral_projection_bytes);
}

#[test]
fn packaged_default_shapes_to_resolvable_latin_and_cjk_handles() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (_, original_database) = shared_font_database_snapshot();
    let _restore_database = SharedFontDatabaseRestore(original_database);
    force_publish_shared_font_database(&runtime_default_font_database_for_test());
    let mut request = TextShapeRequest::new("A界", TextFontRequest::default());
    request.language = Some("zh-Hans-CN");

    let shaped = shared_text_layout_service()
        .shape(request)
        .expect("the packaged default composite must shape without system fonts");
    let (_, database) = shared_font_database_snapshot();
    let families = shaped
        .runs
        .iter()
        .flat_map(|run| &run.glyphs)
        .filter(|glyph| glyph.requires_rasterization)
        .map(|glyph| {
            glyph
                .font_face
                .and_then(resolve_font_face_handle)
                .and_then(|face| database.face_family_name(face))
                .expect("every rasterizable packaged glyph must resolve to a live face")
        })
        .collect::<Vec<_>>();

    assert!(
        families.iter().any(|family| family.as_str() == "Fira Mono"),
        "Latin must use the packaged default typeface"
    );
    assert!(
        families
            .iter()
            .any(|family| family.as_str() == "Zircon Noto Sans CJK SC Proof"),
        "zh-Hans must use the packaged composite sub-font"
    );
}

#[test]
fn packaged_unknown_scalar_projects_the_engine_last_resort_face() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (_, original_database) = shared_font_database_snapshot();
    let _restore_database = SharedFontDatabaseRestore(original_database);
    force_publish_shared_font_database(&runtime_default_font_database_for_test());
    let shaped = shared_text_layout_service()
        .shape(TextShapeRequest::new(
            "\u{10FFFF}",
            TextFontRequest::default(),
        ))
        .expect("the packaged last-resort face must keep missing text renderable");
    let glyph = shaped
        .runs
        .iter()
        .flat_map(|run| &run.glyphs)
        .find(|glyph| glyph.requires_rasterization)
        .expect("missing text must publish one rasterizable notdef glyph");
    let (_, database) = shared_font_database_snapshot();

    assert_eq!(glyph.glyph_id, 0);
    assert_eq!(
        glyph.font_face.and_then(resolve_font_face_handle),
        database.runtime_last_resort_face()
    );
}

#[test]
fn production_shape_resolves_auto_direction_from_the_canonical_run() {
    let font = TextFontRequest {
        size: 16.0,
        ..TextFontRequest::default()
    };

    let result = shared_text_layout_service()
        .shape(TextShapeRequest::new("مرحبا", font))
        .expect("automatic RTL request shapes");

    assert_eq!(result.resolved_direction, TextDirection::RightToLeft);
}

#[test]
fn native_fallback_request_defers_auto_direction_to_canonical_shaping() {
    let request = TextShapeRequest::new("مرحبا", TextFontRequest::default());
    let style = backend_style(&request);
    let features = backend_features(&request);

    let backend = fallback_backend_request(&request, &style, features.as_slice());

    assert_eq!(backend.base_direction, TextDirection::Auto);
}

#[test]
fn neutral_font_request_projects_italic_to_backend_style() {
    let request = TextShapeRequest::new(
        "Italic",
        TextFontRequest {
            italic: true,
            ..TextFontRequest::default()
        },
    );

    assert!(backend_style(&request).italic);
}

#[test]
fn service_reports_font_unavailable_instead_of_publishing_synthetic_glyphs() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (_, original_database) = shared_font_database_snapshot();
    let _restore_database = SharedFontDatabaseRestore(original_database);
    force_publish_shared_font_database(&FontDatabase::default());

    let result = shared_text_layout_service().shape(TextShapeRequest::new(
        "requires a real font face",
        TextFontRequest::default(),
    ));

    assert_eq!(result, Err(TextLayoutError::FontUnavailable));
}

#[test]
fn projection_never_requests_rasterization_without_a_font_face() {
    let glyph = ShapedGlyph {
        glyph_id: 1,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange { start: 0, end: 1 },
        visual_range: TextRange { start: 0, end: 1 },
        advance: 8.0,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::LeftToRight,
        bidi_level: 0,
        cluster_flags: Default::default(),
        rotation: ShapedGlyphRotation::None,
        script: Default::default(),
    };

    let projected = project_glyph(&glyph, (None, None));

    assert!(!projected.requires_rasterization);
}

#[test]
fn neutral_projection_retains_the_complete_vertical_cluster_decision() {
    let basis = TextVerticalGlyphDecisionBasis {
        orientation: TextVerticalGlyphOrientation::TransformOrRotate,
        features: TextVerticalGlyphFeatureSet::VertAndVrt2,
        substitution: TextVerticalGlyphSubstitution::Observed,
        fallback_reason: TextVerticalGlyphFallbackReason::None,
    };
    let glyph = ShapedGlyph {
        glyph_id: 12,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange { start: 0, end: 3 },
        visual_range: TextRange { start: 0, end: 3 },
        advance: 20.0,
        x: 10.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::LeftToRight,
        bidi_level: 0,
        cluster_flags: crate::text::ShapedGlyphClusterFlags {
            cluster_start: true,
            vertical_decision: Some(basis),
            ..crate::text::ShapedGlyphClusterFlags::default()
        },
        rotation: ShapedGlyphRotation::None,
        script: Default::default(),
    };
    let collection = crate::core::framework::text::TextFontCollectionHandle::new(1);
    let face = crate::core::framework::text::TextFontFaceHandle::new(collection, 4, 9);
    let instance = crate::core::framework::text::TextFontFaceHandle::new(collection, 8, 9);

    let projected = project_glyph(&glyph, (Some(face), Some(instance)));
    let decision = projected
        .vertical_glyph_decision()
        .expect("cluster-head decision must survive neutral projection");

    assert_eq!(decision.basis, basis);
    assert_eq!(decision.rotation, TextGlyphRotation::None);
    assert_eq!(decision.font_face, Some(face));
    assert_eq!(decision.font_instance, Some(instance));
}

#[test]
fn neutral_request_features_map_to_backend_shape_features() {
    let requested = [
        TextOpenTypeFeature::new(*b"liga", 0),
        TextOpenTypeFeature::new(*b"tnum", 1),
    ];
    let request =
        TextShapeRequest::new("0123", TextFontRequest::default()).with_features(&requested);

    assert_eq!(
        backend_features(&request),
        vec![
            OpenTypeFeature::new(*b"liga", 0),
            OpenTypeFeature::new(*b"tnum", 1),
        ]
    );
}

#[test]
fn neutral_service_features_change_final_ligature_glyph_count() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let (_, original_database) = shared_font_database_snapshot();
    let _restore_database = SharedFontDatabaseRestore(original_database);
    let mut feature_database = FontDatabase::with_default_fallbacks();
    let font_source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    feature_database
        .register_font_file(&font_source, Some("Fira Sans"), 0)
        .expect("register deterministic ligature fixture");
    force_publish_shared_font_database(&feature_database);

    let families = ["Fira Sans"];
    let font = TextFontRequest {
        families: &families,
        size: 24.0,
        ..TextFontRequest::default()
    };
    let requested = [TextOpenTypeFeature::new(*b"liga", 0)];
    let default_shape = shared_text_layout_service()
        .shape(TextShapeRequest::new("fi", font))
        .expect("default ligature request should shape");
    let disabled_ligature_shape = shared_text_layout_service()
        .shape(TextShapeRequest::new("fi", font).with_features(&requested))
        .expect("feature-bearing ligature request should shape");
    let default_glyph_count = default_shape
        .runs
        .iter()
        .map(|run| run.glyphs.len())
        .sum::<usize>();
    let disabled_ligature_glyph_count = disabled_ligature_shape
        .runs
        .iter()
        .map(|run| run.glyphs.len())
        .sum::<usize>();

    assert!(
        disabled_ligature_glyph_count > default_glyph_count,
        "liga=0 must reach SharedTextLayoutService::shape and suppress the Fira Sans fi ligature: default={default_glyph_count}, disabled={disabled_ligature_glyph_count}"
    );
}

#[test]
fn service_projects_a_run_with_one_font_handle_batch() {
    let _shared_font_database = shared_font_database_test_serial_guard();
    let before = font_handle_registry_report();
    let font = TextFontRequest {
        size: 16.0,
        ..TextFontRequest::default()
    };

    let result = shared_text_layout_service()
        .shape(TextShapeRequest::new("Batch projection", font))
        .expect("production text service should project a shaped run");
    let after = font_handle_registry_report();
    let glyph_count = result
        .runs
        .iter()
        .map(|run| run.glyphs.len())
        .sum::<usize>();

    assert!(glyph_count > 1);
    assert_eq!(
        after.registration_batch_count,
        before.registration_batch_count + 1
    );
    assert_eq!(
        after.registration_lock_acquire_count,
        before.registration_lock_acquire_count + 1
    );
    assert!(
        after.registration_unique_pair_count - before.registration_unique_pair_count
            <= glyph_count as u64
    );
}

#[test]
fn generation_retry_is_bounded_and_defers_after_the_budget() {
    let before = shared_text_layout_generation_retry_report();
    let generations = RefCell::new([10_u64, 11, 12, 13].into_iter());
    let mut shaped_count = 0;

    let result = shape_for_stable_font_generation(
        || {
            (
                generations
                    .borrow_mut()
                    .next()
                    .expect("generation snapshot"),
                (),
            )
        },
        || generations.borrow_mut().next().expect("generation probe"),
        |_| {
            shaped_count += 1;
            Ok(TextShapingCompletion::new(
                shaped_count,
                TextShapingRequestDiagnostics::EMPTY,
            ))
        },
        |shaped_count, _, _, _| shaped_count,
    );
    let after = shared_text_layout_generation_retry_report();

    let failure = result.expect_err("generation churn must defer after the retry budget");
    assert_eq!(failure.error(), &TextLayoutError::FontGenerationChanged);
    assert_eq!(
        failure.request_diagnostics().shaping_attempt_count,
        MAX_FONT_GENERATION_SHAPE_ATTEMPTS as u64
    );
    assert_eq!(
        failure.request_diagnostics().font_generation_restart_count,
        MAX_FONT_GENERATION_SHAPE_ATTEMPTS as u64
    );
    assert_eq!(shaped_count, MAX_FONT_GENERATION_SHAPE_ATTEMPTS);
    assert!(
        after.canonical_shape_count
            >= before.canonical_shape_count + MAX_FONT_GENERATION_SHAPE_ATTEMPTS as u64
    );
    assert!(after.restart_count >= before.restart_count + 2);
    assert!(after.deferred_count >= before.deferred_count + 1);
}

#[test]
fn generation_retry_restarts_when_projection_observes_a_font_publish() {
    let generation = Cell::new(10_u64);
    let shape_count = Cell::new(0_u64);
    let projection_count = Cell::new(0_u64);
    let projected_attempt_count = Cell::new(0_u64);
    let projected_restart_count = Cell::new(0_u64);

    let result = shape_for_stable_font_generation(
        || (generation.get(), ()),
        || generation.get(),
        |_| {
            shape_count.set(shape_count.get() + 1);
            Ok(TextShapingCompletion::new(
                shape_count.get(),
                TextShapingRequestDiagnostics::EMPTY,
            ))
        },
        |shaped_count, _, _, diagnostics| {
            projection_count.set(projection_count.get() + 1);
            projected_attempt_count.set(diagnostics.shaping_attempt_count);
            projected_restart_count.set(diagnostics.font_generation_restart_count);
            if projection_count.get() == 1 {
                generation.set(11);
            }
            shaped_count
        },
    );

    assert_eq!(result, Ok(2));
    assert_eq!(shape_count.get(), 2);
    assert_eq!(projection_count.get(), 2);
    assert_eq!(projected_attempt_count.get(), 2);
    assert_eq!(projected_restart_count.get(), 1);
}
