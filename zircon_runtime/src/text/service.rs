#[cfg(test)]
use std::cell::Cell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

use crate::core::framework::text::{
    TextDirection, TextFontRequest, TextLayoutError, TextLayoutService, TextRenderMode,
    TextShapeRequest, TextShapeResult, TextWritingMode,
};

use super::font::{
    FontCollectionService, FontCollectionSnapshot, FontDatabase, shared_font_collection_service,
};
use super::model::TextShapingRequestDiagnostics;
use super::shaping::{
    FallbackTextSpan, ParagraphTextAnalysis, TextShapingCompletion, TextShapingFailure,
    fallback_text_spans, resolve_bidi_base_direction,
    shape_text_with_diagnostics_in_font_collection,
};
use super::{
    BackendShapeRequest, OpenTypeFeature, ShapedGlyphRun, TextRange, TextStyle, VerticalMode,
};

mod projection;

pub(crate) use projection::project_glyph;
use projection::project_shape_result;

// Bound caller-thread shaping during font reload storms; the next frame retries.
const MAX_FONT_GENERATION_SHAPE_ATTEMPTS: usize = 2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextLayoutGenerationRetryReport {
    pub canonical_shape_count: u64,
    pub neutral_projection_count: u64,
    pub neutral_projection_glyph_count: u64,
    pub neutral_projection_bytes: u64,
    pub restart_count: u64,
    pub deferred_count: u64,
}

#[derive(Default)]
struct TextLayoutGenerationRetryMetrics {
    canonical_shape_count: AtomicU64,
    neutral_projection_count: AtomicU64,
    neutral_projection_glyph_count: AtomicU64,
    neutral_projection_bytes: AtomicU64,
    restart_count: AtomicU64,
    deferred_count: AtomicU64,
}

impl TextLayoutGenerationRetryMetrics {
    fn report(&self) -> TextLayoutGenerationRetryReport {
        TextLayoutGenerationRetryReport {
            canonical_shape_count: self.canonical_shape_count.load(Ordering::Relaxed),
            neutral_projection_count: self.neutral_projection_count.load(Ordering::Relaxed),
            neutral_projection_glyph_count: self
                .neutral_projection_glyph_count
                .load(Ordering::Relaxed),
            neutral_projection_bytes: self.neutral_projection_bytes.load(Ordering::Relaxed),
            restart_count: self.restart_count.load(Ordering::Relaxed),
            deferred_count: self.deferred_count.load(Ordering::Relaxed),
        }
    }
}

fn generation_retry_metrics() -> &'static TextLayoutGenerationRetryMetrics {
    static METRICS: OnceLock<TextLayoutGenerationRetryMetrics> = OnceLock::new();
    METRICS.get_or_init(TextLayoutGenerationRetryMetrics::default)
}

#[cfg(test)]
thread_local! {
    static CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT: Cell<u64> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn current_thread_neutral_projection_count() -> u64 {
    CURRENT_THREAD_NEUTRAL_PROJECTION_COUNT.get()
}

pub(crate) fn shared_text_layout_generation_retry_report() -> TextLayoutGenerationRetryReport {
    generation_retry_metrics().report()
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SharedTextLayoutService;

pub fn shared_text_layout_service() -> &'static dyn TextLayoutService {
    static SERVICE: SharedTextLayoutService = SharedTextLayoutService;
    &SERVICE
}

pub(crate) fn fallback_spans_for_request(
    request: TextShapeRequest<'_>,
    font_database: &FontDatabase,
) -> Result<Vec<FallbackTextSpan>, TextLayoutError> {
    let style = backend_style(&request);
    let features = backend_features(&request);
    let backend_request = fallback_backend_request(&request, &style, features.as_slice());
    let canonical_request = backend_request.canonicalized()?;
    let backend_request = canonical_request.request();
    let analysis = ParagraphTextAnalysis::for_snapshot(
        backend_request.text,
        backend_request.explicit_language_script(),
        backend_request.unicode_data_snapshot(),
    );
    fallback_text_spans(
        backend_request.text,
        backend_request,
        font_database,
        &analysis,
    )
    .map_err(|_| TextLayoutError::FontUnavailable)
}

fn fallback_backend_request<'a>(
    request: &'a TextShapeRequest<'a>,
    style: &'a TextStyle,
    features: &'a [OpenTypeFeature],
) -> BackendShapeRequest<'a> {
    BackendShapeRequest::horizontal(
        request.text,
        style,
        request.direction,
        TextRange {
            start: 0,
            end: request.text.len(),
        },
    )
    .with_features(features)
    .with_language(request.language)
}

impl TextLayoutService for SharedTextLayoutService {
    fn resolve_render_mode(&self, request: &TextFontRequest<'_>) -> TextRenderMode {
        match request.render_mode {
            TextRenderMode::Auto => TextRenderMode::Native,
            mode => mode,
        }
    }

    fn resolve_direction(&self, text: &str, requested: TextDirection) -> TextDirection {
        resolve_bidi_base_direction(text, requested)
    }

    fn shape(&self, request: TextShapeRequest<'_>) -> Result<TextShapeResult, TextLayoutError> {
        shape_text_request_in_font_collection(request, &shared_font_collection_service())
    }
}

pub(crate) fn shape_text_request_in_font_collection(
    request: TextShapeRequest<'_>,
    font_collection: &Arc<FontCollectionService>,
) -> Result<TextShapeResult, TextLayoutError> {
    let style = backend_style(&request);
    let features = backend_features(&request);
    let source_range = TextRange {
        start: 0,
        end: request.text.len(),
    };
    let backend_request = match request.writing_mode {
        TextWritingMode::HorizontalTopToBottom => {
            BackendShapeRequest::horizontal(request.text, &style, request.direction, source_range)
                .with_kerning(request.include_kerning)
        }
        TextWritingMode::VerticalRightToLeft => BackendShapeRequest::vertical(
            request.text,
            &style,
            request.direction,
            source_range,
            VerticalMode::Mixed,
        )
        .with_kerning(request.include_kerning),
    }
    .with_features(features.as_slice())
    .with_language(request.language);
    shape_backend_request_at_stable_generation_in_font_collection(
        backend_request,
        font_collection,
        |shaped, font_collection, _, _| {
            let resolved_direction = shaped.direction;
            project_shape_result(shaped, resolved_direction, font_collection)
        },
    )
    .map_err(TextShapingFailure::into_error)
}

pub(super) fn shape_backend_request_at_stable_generation<Projected>(
    request: BackendShapeRequest<'_>,
    project: impl FnMut(
        ShapedGlyphRun,
        &FontCollectionSnapshot,
        u64,
        TextShapingRequestDiagnostics,
    ) -> Projected,
) -> Result<Projected, TextShapingFailure> {
    let font_collection = shared_font_collection_service();
    shape_backend_request_at_stable_generation_in_font_collection(
        request,
        &font_collection,
        project,
    )
}

pub(super) fn shape_backend_request_at_stable_generation_in_font_collection<Projected>(
    request: BackendShapeRequest<'_>,
    font_collection: &Arc<FontCollectionService>,
    project: impl FnMut(
        ShapedGlyphRun,
        &FontCollectionSnapshot,
        u64,
        TextShapingRequestDiagnostics,
    ) -> Projected,
) -> Result<Projected, TextShapingFailure> {
    let canonical_request = request.canonicalized().map_err(TextShapingFailure::from)?;
    let request = canonical_request.request();
    validate_backend_shape_request(&request).map_err(TextShapingFailure::from)?;
    shape_for_stable_font_generation(
        || {
            let snapshot = font_collection.collection_snapshot();
            (snapshot.generation(), snapshot)
        },
        || font_collection.generation(),
        |snapshot: &FontCollectionSnapshot| {
            shape_text_with_diagnostics_in_font_collection(request, snapshot)
        },
        project,
    )
}

fn shape_for_stable_font_generation<Snapshot, Shaped, Projected>(
    mut snapshot: impl FnMut() -> (u64, Snapshot),
    mut generation: impl FnMut() -> u64,
    mut shape: impl FnMut(&Snapshot) -> Result<TextShapingCompletion<Shaped>, TextShapingFailure>,
    mut project: impl FnMut(Shaped, &Snapshot, u64, TextShapingRequestDiagnostics) -> Projected,
) -> Result<Projected, TextShapingFailure> {
    let metrics = generation_retry_metrics();
    let mut request_diagnostics = TextShapingRequestDiagnostics::EMPTY;
    for _ in 0..MAX_FONT_GENERATION_SHAPE_ATTEMPTS {
        let (shape_generation, font_snapshot) = snapshot();
        let completion = match shape(&font_snapshot) {
            Ok(completion) => completion,
            Err(failure) => {
                let mut failure_diagnostics = failure.request_diagnostics();
                failure_diagnostics.shaping_attempt_count =
                    failure_diagnostics.shaping_attempt_count.saturating_add(1);
                request_diagnostics.merge(failure_diagnostics);
                return Err(failure.replace_request_diagnostics(request_diagnostics));
            }
        };
        let (shaped, mut attempt_diagnostics) = completion.into_parts();
        attempt_diagnostics.shaping_attempt_count =
            attempt_diagnostics.shaping_attempt_count.saturating_add(1);
        request_diagnostics.merge(attempt_diagnostics);
        metrics
            .canonical_shape_count
            .fetch_add(1, Ordering::Relaxed);
        if shape_generation != generation() {
            metrics.restart_count.fetch_add(1, Ordering::Relaxed);
            request_diagnostics.font_generation_restart_count = request_diagnostics
                .font_generation_restart_count
                .saturating_add(1);
            continue;
        }
        let projected = project(
            shaped,
            &font_snapshot,
            shape_generation,
            request_diagnostics,
        );
        if shape_generation == generation() {
            return Ok(projected);
        }
        metrics.restart_count.fetch_add(1, Ordering::Relaxed);
        request_diagnostics.font_generation_restart_count = request_diagnostics
            .font_generation_restart_count
            .saturating_add(1);
    }
    metrics.deferred_count.fetch_add(1, Ordering::Relaxed);
    Err(TextShapingFailure::font_generation_changed().with_request_diagnostics(request_diagnostics))
}

fn validate_backend_shape_request(
    request: &BackendShapeRequest<'_>,
) -> Result<(), TextLayoutError> {
    if !request.style.font_size.is_finite() || request.style.font_size <= 0.0 {
        return Err(TextLayoutError::InvalidFontSize);
    }
    Ok(())
}

fn backend_features(request: &TextShapeRequest<'_>) -> Vec<OpenTypeFeature> {
    // Keep framework DTOs independent from the implementation-owned, normalized cache key type.
    request
        .features
        .iter()
        .map(|feature| OpenTypeFeature::new(feature.tag, feature.value))
        .collect()
}

fn backend_style(request: &TextShapeRequest<'_>) -> TextStyle {
    TextStyle {
        font: request.font.asset.map(str::to_string),
        font_family: request
            .font
            .families
            .first()
            .map(|family| (*family).to_string()),
        language: request.language.map(str::to_string),
        font_weight: request.font.weight,
        italic: request.font.italic,
        font_size: request.font.size,
        line_height: request.line_height,
        tab_size: request.tab_size,
        ..TextStyle::default()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::path::Path;

    use super::*;
    use crate::core::framework::text::{TextGlyphRotation, TextOpenTypeFeature};
    use crate::text::font::{
        FontDatabase, font_handle_registry_report, force_publish_shared_font_database,
        resolve_font_face_handle, runtime_default_font_database_for_test,
        shared_font_database_snapshot, shared_font_database_test_serial_guard,
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
            after.neutral_projection_glyph_count
                >= before.neutral_projection_glyph_count + glyph_count
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
        let mut generations = [10_u64, 11, 12, 13].into_iter();
        let mut shaped_count = 0;

        let result = shape_for_stable_font_generation(
            || (generations.next().expect("generation snapshot"), ()),
            || generations.next().expect("generation probe"),
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
}
