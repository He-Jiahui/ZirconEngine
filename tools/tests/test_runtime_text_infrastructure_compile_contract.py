import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RuntimeTextInfrastructureCompileContractTests(unittest.TestCase):
    def test_rich_cache_telemetry_is_owner_qualified_and_atomically_reset(self):
        cache = (ROOT / "zircon_runtime/src/text/cache/rich_cache.rs").read_text(
            encoding="utf-8"
        )
        parser = (
            ROOT / "zircon_runtime/src/text/rich/parser_registry.rs"
        ).read_text(encoding="utf-8")
        session = (ROOT / "zircon_runtime/src/text/layout_session.rs").read_text(
            encoding="utf-8"
        )
        measure = (ROOT / "zircon_runtime/src/ui/text/measure_cache.rs").read_text(
            encoding="utf-8"
        )
        profile = (
            ROOT / "zircon_runtime/src/ui/surface/render/text_prewarm/profile.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub parser_identity: u64", cache)
        self.assertIn("pub decorator_generation: u64", cache)
        self.assertIn("pub emoji_generation: u64", cache)
        self.assertIn("pub telemetry_saturated: bool", cache)
        self.assertIn("pub(crate) fn take_report", cache)
        self.assertIn("fn reset_interval_counters", cache)
        self.assertNotIn("CompiledRichTextCacheFrameSampler", cache)
        self.assertIn("fn take_compiled_cache_report", parser)
        self.assertIn("fn take_compiled_rich_text_cache_report", session)
        self.assertIn("take_compiled_rich_text_cache_report()", measure)
        self.assertNotIn("compiled_rich_text_cache_sampler", measure)
        self.assertIn('"ui_text.rich_cache.parser_identity"', profile)
        self.assertIn('"ui_text.rich_cache.decorator_generation"', profile)
        self.assertIn('"ui_text.rich_cache.emoji_generation"', profile)
        self.assertIn('"ui_text.rich_cache.counter_saturated"', profile)

    def test_rich_cache_single_flight_contention_is_measurable_before_redesign(self):
        cache = (ROOT / "zircon_runtime/src/text/cache/rich_cache.rs").read_text(
            encoding="utf-8"
        )
        profile = (
            ROOT / "zircon_runtime/src/ui/surface/render/text_prewarm/profile.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub compile_requests_in_flight: usize", cache)
        self.assertIn("pub single_flight_wait_count: u64", cache)
        self.assertIn("pub single_flight_wait_nanos: u64", cache)
        self.assertIn("pub single_flight_wait_max_nanos: u64", cache)
        self.assertIn("Cell::new(false)", cache)
        self.assertIn("initialized_here.set(true)", cache)
        self.assertIn('"ui_text.rich_cache.compile_requests_in_flight"', profile)
        self.assertIn('"ui_text.rich_cache.single_flight_waits"', profile)
        self.assertIn('"ui_text.rich_cache.single_flight_wait_nanos"', profile)
        self.assertIn('"ui_text.rich_cache.single_flight_wait_max_nanos"', profile)

    def test_compiled_rich_dependencies_are_typed_before_render_resource_collection(self):
        dependency = (
            ROOT / "zircon_runtime/src/text/rich/compiled/dependency.rs"
        ).read_text(encoding="utf-8")
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )
        memory = (
            ROOT / "zircon_runtime/src/text/rich/compiled/memory.rs"
        ).read_text(encoding="utf-8")
        text_facade = (ROOT / "zircon_runtime/src/text/mod.rs").read_text(
            encoding="utf-8"
        )
        texture_collection = (
            ROOT / "zircon_runtime/src/graphics/scene/resources/ui_texture.rs"
        ).read_text(encoding="utf-8")
        plan = (
            ROOT
            / "docs/plans/zircon_runtime/text/07/2026-08-30-rich-typed-dependency-closure-foundation.md"
        ).read_text(encoding="utf-8")

        self.assertIn("pub enum RichTextDependency", dependency)
        self.assertIn("ImageTexture(ResourceId)", dependency)
        self.assertIn("pub(super) fn collect", dependency)
        self.assertIn("dependencies: Arc<[RichTextDependency]>", compiled)
        self.assertIn("dependency::collect(&parsed)", compiled)
        self.assertIn("pub fn dependencies(&self) -> &[RichTextDependency]", compiled)
        self.assertNotIn("resource_ids", compiled)
        self.assertIn("compiled.dependencies.len()", memory)
        self.assertIn("RichTextDependency", text_facade)
        self.assertIn("RichTextDependency::ImageTexture", texture_collection)
        self.assertNotIn("rich.resource_ids()", texture_collection)
        self.assertIn(
            "RRT-P1-020_typed_image_dependency_foundation_static_complete", plan
        )
        self.assertIn("O(R + D log D)", plan)

    def test_rich_paint_projection_profile_is_runtime_owned_and_cache_truthful(self):
        projection = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/paint_projection.rs"
        ).read_text(encoding="utf-8")
        render = (
            ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
        ).read_text(encoding="utf-8")
        plan_cache = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("struct ScreenSpaceUiTextPaintProjectionReport", projection)
        self.assertIn("project_transient_paint_elements", projection)
        self.assertIn('"materialize_transient_text_paint"', projection)
        self.assertIn('"ui_text.paint_projection.rich_command_count"', projection)
        self.assertIn('"ui_text.paint_projection.rich_paint_run_count"', projection)
        self.assertIn('"ui_text.paint_projection.rich_run_text_bytes"', projection)
        self.assertIn("mod paint_projection;", render)
        self.assertIn("project_transient_paint_elements(", render)
        self.assertIn("publish_profile_counters", render)
        self.assertIn("paint_projection_report", plan_cache)
        self.assertIn(
            "ScreenSpaceUiTextPaintProjectionReport::default().publish_profile_counters()",
            plan_cache,
        )

    def test_table_layout_work_receipt_is_frame_scoped_and_phase_complete(self):
        work_report = (
            ROOT / "zircon_runtime/src/text/layout_session/table_work.rs"
        ).read_text(encoding="utf-8")
        session = (
            ROOT / "zircon_runtime/src/text/layout_session.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("struct TextTableLayoutWorkReport", work_report)
        self.assertIn("preferred_cell_layout_count", work_report)
        self.assertIn("final_cell_layout_count", work_report)
        self.assertIn("publish_profile_counters", work_report)
        self.assertIn('"rich_table_preferred_cell_layout_count"', work_report)
        self.assertIn('"rich_table_final_cell_layout_count"', work_report)
        self.assertIn("table_layout_work_report: TextTableLayoutWorkReport", session)
        self.assertIn("record_table_layout_attempt", session)
        self.assertIn("record_table_layout_tracks", session)
        self.assertIn("record_table_preferred_cell_layout", table_layout)
        self.assertIn("record_table_final_cell_layout", table_layout)
        self.assertIn("record_table_layout_output", table_layout)

    def test_layout_geometry_budget_is_session_owned_and_reports_rejection_context(self):
        geometry = (
            ROOT / "zircon_runtime/src/text/layout_geometry.rs"
        ).read_text(encoding="utf-8")
        session = (
            ROOT / "zircon_runtime/src/text/layout_session.rs"
        ).read_text(encoding="utf-8")
        diagnostics = (
            ROOT / "zircon_runtime/src/text/layout_session/diagnostics.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct TextLayoutGeometryBudget", geometry)
        self.assertIn("DEFAULT_MAX_EXACT_LAYOUT_EXTENT", geometry)
        self.assertIn("fn checked_add_accumulated", geometry)
        self.assertIn("fn checked_scale_accumulated", geometry)
        self.assertIn("geometry_budget: TextLayoutGeometryBudget", session)
        self.assertIn("new_with_font_collection_and_geometry_budget", session)
        self.assertIn("pub(crate) fn reject_geometry", session)
        self.assertIn("TextLayoutGeometryRejectionReceipt", diagnostics)
        self.assertIn("source_range: Option<(u32, u32)>", diagnostics)
        self.assertIn("work_units: usize", diagnostics)

    def test_rich_table_uses_typed_unbounded_measurement_without_fake_extent(self):
        geometry = (
            ROOT / "zircon_runtime/src/text/layout_geometry.rs"
        ).read_text(encoding="utf-8")
        measurement = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/measurement.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("enum TextLayoutAxisConstraint", geometry)
        self.assertIn("Self::Unbounded => f32::INFINITY", geometry)
        self.assertIn("intrinsic_measurement_frame_with_provider", measurement)
        self.assertIn("bounded_inline_measurement_frame_with_provider", measurement)
        self.assertIn("intrinsic_measurement_frame_with_provider", table_layout)
        self.assertIn("bounded_inline_measurement_frame_with_provider", table_layout)
        self.assertNotIn("MAX_PROVISIONAL_CELL_BLOCK_EXTENT", table_layout)
        self.assertNotIn("provisional_block_extent", table_layout)

    def test_rich_table_track_sizing_rejects_invalid_geometry_without_sanitizing(self):
        sizing = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/sizing.rs"
        ).read_text(encoding="utf-8")
        cell_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("TextLayoutAxisConstraint", sizing)
        self.assertIn("Result<Vec<f32>, TextLayoutGeometryViolation>", sizing)
        self.assertIn("checked_add_accumulated", sizing)
        self.assertNotIn("sanitize_non_negative", sizing)
        self.assertIn("Result<Self, TextLayoutGeometryViolation>", cell_layout)
        self.assertIn("checked_add_accumulated", cell_layout)
        self.assertIn("TextLayoutAxisConstraint::from_request_extent", table_layout)
        self.assertIn("subtract_accumulated", table_layout)

    def test_rich_table_validates_layout_and_aggregate_geometry_before_publication(self):
        geometry_admission = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/geometry_admission.rs"
        ).read_text(encoding="utf-8")
        measurement = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/measurement.rs"
        ).read_text(encoding="utf-8")
        table_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs"
        ).read_text(encoding="utf-8")
        table_geometry = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/geometry.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("validate_resolved_layout_geometry", geometry_admission)
        self.assertIn("validate_resolved_size_geometry", geometry_admission)
        self.assertIn("admit_coordinate", geometry_admission)
        self.assertIn("glyph_advances", geometry_admission)
        self.assertIn("validate_resolved_layout_geometry", measurement)
        self.assertIn("validate_resolved_size_geometry", measurement)
        self.assertIn("checked_scale_accumulated", measurement)
        self.assertIn("TextLayoutGeometryOwner::IntrinsicMeasurement", measurement)
        self.assertIn("validate_resolved_layout_geometry", table_layout)
        self.assertIn("checked_add_accumulated", table_geometry)
        self.assertIn("TextLayoutGeometryOwner::TableAggregate", table_geometry)

    def test_html_subset_authoring_diagnostics_are_bounded_canonical_artifact_data(self):
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        builder = (
            ROOT / "zircon_runtime/src/text/rich/parser/builder.rs"
        ).read_text(encoding="utf-8")
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        html_parser = (
            ROOT / "zircon_runtime/src/text/rich/parser/html.rs"
        ).read_text(encoding="utf-8")
        html_diagnostics = (
            ROOT / "zircon_runtime/src/text/rich/parser/html_diagnostics.rs"
        ).read_text(encoding="utf-8")
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub enum RichTextAuthoringDiagnosticCode", model)
        self.assertIn("pub source_range: (u32, u32)", model)
        self.assertIn("pub authoring_diagnostics: Vec<RichTextAuthoringDiagnostic>", model)
        self.assertIn("pub authoring_diagnostics_truncated: bool", model)
        self.assertIn("pub max_authoring_diagnostics: usize", admission)
        self.assertIn("with_max_authoring_diagnostics", admission)
        self.assertIn("fn push_authoring_diagnostic", builder)
        self.assertIn("authoring_diagnostics_truncated = true", builder)
        self.assertIn("RichTextAuthoringDiagnosticCode::UnsupportedTag", html_parser)
        self.assertIn("RichTextAuthoringDiagnosticCode::UnmatchedClosingTag", html_parser)
        self.assertIn("RichTextAuthoringDiagnosticCode::ImplicitlyClosedTag", html_parser)
        self.assertIn("RichTextAuthoringDiagnosticCode::UnclosedTag", html_parser)
        self.assertIn("mod html_diagnostics;", parser)
        self.assertIn("RichTextAuthoringDiagnosticCode::UnsupportedAttribute", html_diagnostics)
        self.assertIn("RichTextAuthoringDiagnosticCode::MalformedAttribute", html_diagnostics)
        self.assertIn("RichTextAuthoringDiagnosticCode::InvalidAttributeValue", html_diagnostics)
        self.assertIn(
            "RichTextAuthoringDiagnosticCode::UnsupportedStyleProperty", html_diagnostics
        )
        self.assertIn("RichTextAuthoringDiagnosticCode::MalformedTag", html_diagnostics)
        self.assertIn(
            "RichTextAuthoringDiagnosticCode::UnterminatedQuotedAttribute",
            html_diagnostics,
        )
        self.assertIn("RichTextAuthoringDiagnosticCode::MalformedEntity", html_diagnostics)
        self.assertIn("RichTextAuthoringDiagnosticCode::UnrecognizedEntity", html_diagnostics)
        self.assertIn(
            "fn issues(&self) -> HtmlTokenIssues",
            (ROOT / "zircon_runtime/src/text/rich/html_subset.rs").read_text(
                encoding="utf-8"
            ),
        )
        html_subset = (
            ROOT / "zircon_runtime/src/text/rich/html_subset.rs"
        ).read_text(encoding="utf-8")
        self.assertRegex(
            html_subset,
            r"Malformed\s*\{\s*issues: HtmlTokenIssues",
        )
        self.assertIn("fn decode_entities_with_issues", html_subset)
        self.assertIn("pub(super) fn looks_like_tag_candidate", html_subset)
        self.assertIn(
            "pub(super) fn has_unterminated_attribute_quote", html_subset
        )
        compiled_memory = (
            ROOT / "zircon_runtime/src/text/rich/compiled/memory.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("mod memory;", compiled)
        self.assertIn(
            "compiled.parsed.authoring_diagnostics.capacity()", compiled_memory
        )

    def test_rich_bidi_controls_publish_bounded_source_ranged_authoring_evidence(self):
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        html_parser = (
            ROOT / "zircon_runtime/src/text/rich/parser/html.rs"
        ).read_text(encoding="utf-8")
        bidi_diagnostics = (
            ROOT / "zircon_runtime/src/text/rich/parser/bidi_diagnostics.rs"
        ).read_text(encoding="utf-8")
        html_subset = (
            ROOT / "zircon_runtime/src/text/rich/html_subset.rs"
        ).read_text(encoding="utf-8")

        for code in (
            "BidirectionalMark",
            "BidirectionalEmbedding",
            "BidirectionalOverride",
            "BidirectionalIsolate",
        ):
            self.assertIn(code, model)
            self.assertIn(code, bidi_diagnostics)
        self.assertIn("push_source_bidi_control_diagnostics", parser)
        self.assertIn("push_literal_bidi_control_diagnostic", parser)
        self.assertIn("decode_entities_with_issues_observing", html_parser)
        self.assertIn("decode_entities_with_issues_observing", html_subset)
        self.assertIn("RichTextAuthoringRecovery::PreservedAsText", bidi_diagnostics)
        self.assertNotIn("replace_bidi", bidi_diagnostics)
        self.assertNotIn("strip_bidi", bidi_diagnostics)

    def test_rich_bidi_trust_is_typed_cache_identifying_and_fail_closed(self):
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        parser = (
            ROOT / "zircon_runtime/src/text/rich/parser_registry.rs"
        ).read_text(encoding="utf-8")
        cache = (ROOT / "zircon_runtime/src/text/cache/rich_cache.rs").read_text(
            encoding="utf-8"
        )
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )
        builder = (
            ROOT / "zircon_runtime/src/text/rich/parser/builder.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub enum RichTextContentTrust", admission)
        self.assertIn("Untrusted", admission)
        self.assertIn("TrustedAuthoring", admission)
        self.assertIn("pub max_bidi_control_depth: usize", admission)
        self.assertIn("with_max_bidi_control_depth", admission)
        self.assertIn("BidiControlNotAllowed", admission)
        self.assertIn("UnbalancedBidiControl", admission)
        self.assertIn("BidiControlDepthExceeded", admission)
        self.assertIn("pub fn compile_with_content_trust", parser)
        self.assertIn("content_trust: RichTextContentTrust", cache)
        self.assertIn("content_trust: RichTextContentTrust", compiled)
        self.assertIn("finish_bidi_control_admission", builder)

    def test_rich_format_identity_is_versioned_across_api_parser_and_cache(self):
        model = (ROOT / "zircon_runtime/src/text/model/style.rs").read_text(
            encoding="utf-8"
        )
        interface = (
            ROOT / "zircon_runtime_interface/src/ui/surface/render/typography.rs"
        ).read_text(encoding="utf-8")
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        conversion = (
            ROOT / "zircon_runtime/src/graphics/text_transport/conversion.rs"
        ).read_text(encoding="utf-8")
        cache = (ROOT / "zircon_runtime/src/text/cache/rich_cache.rs").read_text(
            encoding="utf-8"
        )
        style_parser = (
            ROOT / "zircon_runtime/src/ui/surface/render/resolve/text_style_parsing.rs"
        ).read_text(encoding="utf-8")

        for source in (model, interface):
            self.assertIn('serde(rename = "markdown_inline_v1")', source)
            self.assertIn("MarkdownInlineV1,", source)
            self.assertIn('serde(rename = "bbcode_v1")', source)
            self.assertIn("BbCodeV1,", source)
            self.assertIn('serde(rename = "html_subset_v1")', source)
            self.assertIn("HtmlSubsetV1,", source)
            self.assertNotIn("    Markdown,", source)
            self.assertNotIn("    BbCode,", source)
            self.assertNotIn("    Html,", source)
        for source in (parser, conversion):
            self.assertIn("RichTextFormat::MarkdownInlineV1", source)
            self.assertIn("RichTextFormat::BbCodeV1", source)
            self.assertIn("RichTextFormat::HtmlSubsetV1", source)
        self.assertIn("format: RichTextFormat,", cache)
        self.assertNotIn("format: u8,", cache)
        self.assertNotIn("rich_text_format_id", cache)
        self.assertIn('["markdown_inline_v1"]', style_parser)
        self.assertIn('["bbcode_v1"]', style_parser)
        self.assertIn('["html_subset_v1"]', style_parser)
        self.assertNotIn('["markdown"]', style_parser)
        self.assertNotIn('["bbcode",', style_parser)
        self.assertNotIn('["html"]', style_parser)

    def test_layout_geometry_overflow_has_stable_typed_diagnostic(self):
        source = (
            ROOT / "zircon_runtime/src/core/framework/text/layout_error.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("GeometryTooLarge,", source)
        self.assertIn(
            'Self::GeometryTooLarge => "ZR-TEXT-LAYOUT-013"', source
        )
        self.assertIn(
            'Self::GeometryTooLarge => "text.layout.geometry_too_large"', source
        )
        self.assertIn("TextLayoutError::GeometryTooLarge,", source)

    def test_glyph_projection_has_one_crate_local_owner(self):
        service = (ROOT / "zircon_runtime/src/text/service.rs").read_text(encoding="utf-8")
        projection = (ROOT / "zircon_runtime/src/text/service/projection.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub(crate) use projection::project_glyph;", service)
        self.assertIn("pub(crate) fn project_glyph(", projection)

    def test_font_fallback_hash_optional_branches_have_unit_type(self):
        source = (ROOT / "zircon_runtime/src/text/font/fallback_cache.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("None => {\n            hasher.update(&[0]);\n        }", source)

    def test_font_family_dedupe_declares_identity_map_type(self):
        source = (ROOT / "zircon_runtime/src/text/font/matching.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "HashMap::<FontFamilyIdentity, usize>::with_capacity(capacity)", source
        )

    def test_sdf_face_cache_callers_use_database_only(self):
        source_paths = [
            ROOT / "zircon_runtime/src/text/sdf/font_bake/async_batch.rs",
            ROOT / "zircon_runtime/src/text/sdf/font_bake/dynamic_batch.rs",
        ]
        for path in source_paths:
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("resolve_faces_for_key_cached(&slot.key, font_database, asset_manager)", source)
        self.assertIn(
            "resolve_faces_for_key_cached(&slot.key, font_database)",
            (ROOT / "zircon_runtime/src/text/sdf/font_bake/async_batch.rs").read_text(
                encoding="utf-8"
            ),
        )

    def test_surface_font_dependency_owner_is_visible_only_to_surface_tree(self):
        owner = (
            ROOT / "zircon_runtime/src/ui/surface/render/font_dependencies.rs"
        ).read_text(encoding="utf-8")
        boundary = (ROOT / "zircon_runtime/src/ui/surface/render/mod.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "pub(in crate::ui::surface) fn text_font_asset_dependencies", owner
        )
        self.assertIn(
            "pub(super) use font_dependencies::text_font_asset_dependencies;", boundary
        )

    def test_paragraph_list_prefix_reuses_checked_source_range_owner(self):
        source = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs"
        ).read_text(encoding="utf-8")
        tests = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/paragraph_layout/tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("match checked_source_range(text, range)", source)
        self.assertNotIn(
            ".and_then(|range| text.get(range.0 as usize..range.1 as usize))", source
        )
        self.assertIn("paragraph_insets_reject_malformed_list_prefix_ranges", tests)

    def test_default_face_visibility_reaches_sdf_without_global_export(self):
        database = (ROOT / "zircon_runtime/src/text/font/database.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "pub(in crate::text) const fn runtime_default_primary_face", database
        )

    def test_cosmic_cache_clones_snapshot_and_shared_revision_is_runtime(self):
        cosmic = (ROOT / "zircon_runtime/src/text/shaping/cosmic/font_system_cache.rs").read_text(
            encoding="utf-8"
        )
        shared = (ROOT / "zircon_runtime/src/text/font/shared.rs").read_text(encoding="utf-8")
        self.assertIn("font_collection: font_collection.clone()", cosmic)
        self.assertIn("pub(crate) fn collection_id(&self)", shared)
        self.assertIn("pub(crate) fn revision(&self)", shared)
        lease = (ROOT / "zircon_runtime/src/text/glyph_artifact.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "pub(crate) fn revision(&self) -> crate::text::font::FontCollectionRevision",
            lease,
        )

    def test_rich_parser_admission_precedes_cache_and_bounds_visible_output(self):
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        registry = (
            ROOT / "zircon_runtime/src/text/rich/parser_registry.rs"
        ).read_text(encoding="utf-8")
        emoji = (
            ROOT / "zircon_runtime/src/text/rich/emoji_shortcode.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("pub struct RichParseBudget", admission)
        self.assertIn("SourceByteBudgetExceeded", admission)
        self.assertIn("OutputByteBudgetExceeded", admission)
        self.assertLess(
            registry.index("self.budget.admit_source(markup.len())?;"),
            registry.index(
                ".compile(markup, format, content_trust, generation, |markup|"
            ),
        )
        self.assertIn("existing_output_bytes", emoji)
        self.assertIn("max_output_bytes", emoji)

    def test_rich_active_tag_stack_has_request_local_depth_admission(self):
        admission = (
            ROOT / "zircon_runtime/src/text/rich/admission.rs"
        ).read_text(encoding="utf-8")
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        active_tags = (
            ROOT / "zircon_runtime/src/text/rich/parser/active_tags.rs"
        ).read_text(encoding="utf-8")
        tests = (
            ROOT / "zircon_runtime/src/text/rich/tests/admission.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub max_active_tag_depth: usize", admission)
        self.assertIn("ActiveTagDepthBudgetExceeded", admission)
        self.assertIn("fn push(", active_tags)
        self.assertIn("Result<(), RichTextParseError>", active_tags)
        self.assertIn("max_active_tag_depth", parser)
        self.assertIn(
            "rich_parser_rejects_active_tag_depth_before_stack_growth", tests
        )

    def test_rich_markup_token_budget_precedes_dispatch(self):
        admission = (
            ROOT / "zircon_runtime/src/text/rich/admission.rs"
        ).read_text(encoding="utf-8")
        builder = (
            ROOT / "zircon_runtime/src/text/rich/parser/builder.rs"
        ).read_text(encoding="utf-8")
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        markdown = (
            ROOT / "zircon_runtime/src/text/rich/parser/markdown.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub max_tokens: usize", admission)
        self.assertIn("TokenBudgetExceeded", admission)
        self.assertIn("fn admit_tokens(", builder)
        self.assertIn("result.admit_tokens(1)?", parser)
        self.assertIn("result.admit_tokens(2)?", markdown)

    def test_rich_attribute_budget_precedes_token_string_allocation(self):
        admission = (
            ROOT / "zircon_runtime/src/text/rich/admission.rs"
        ).read_text(encoding="utf-8")
        bbcode = (ROOT / "zircon_runtime/src/text/rich/bbcode.rs").read_text(
            encoding="utf-8"
        )
        html = (ROOT / "zircon_runtime/src/text/rich/html_subset.rs").read_text(
            encoding="utf-8"
        )
        tests = (
            ROOT / "zircon_runtime/src/text/rich/tests/admission.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub max_attributes_per_token: usize", admission)
        self.assertIn("pub max_attribute_bytes_per_token: usize", admission)
        self.assertIn("AttributeCountBudgetExceeded", admission)
        self.assertIn("AttributeByteBudgetExceeded", admission)
        self.assertIn("tokenizer_budget.admit_attribute(", bbcode)
        self.assertIn("tokenizer_budget.admit_attribute(", html)
        self.assertIn(
            "rich_parser_rejects_attribute_count_and_bytes_before_string_allocation",
            tests,
        )

    def test_rich_token_byte_budget_precedes_tag_name_allocation(self):
        admission = (
            ROOT / "zircon_runtime/src/text/rich/admission.rs"
        ).read_text(encoding="utf-8")
        bbcode = (ROOT / "zircon_runtime/src/text/rich/bbcode.rs").read_text(
            encoding="utf-8"
        )
        html = (ROOT / "zircon_runtime/src/text/rich/html_subset.rs").read_text(
            encoding="utf-8"
        )
        tests = (
            ROOT / "zircon_runtime/src/text/rich/tests/admission.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub max_token_bytes: usize", admission)
        self.assertIn("TokenByteBudgetExceeded", admission)
        self.assertIn("tokenizer_budget.admit_token_bytes", bbcode)
        self.assertIn("tokenizer_budget.admit_token_bytes", html)
        self.assertIn(
            "rich_parser_rejects_oversized_token_before_tag_name_allocation", tests
        )

    def test_rich_compiled_indices_never_use_saturating_identity(self):
        source_paths = [
            ROOT / "zircon_runtime/src/text/rich/parser.rs",
            ROOT / "zircon_runtime/src/text/rich/compiled.rs",
            ROOT / "zircon_runtime/src/ui/text/rich_text.rs",
            ROOT / "zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs",
        ]
        for path in source_paths:
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("unwrap_or(u32::MAX)", source)
            self.assertNotIn("fn to_u32(", source)
        compiled = source_paths[1].read_text(encoding="utf-8")
        self.assertIn("checked_artifact_index(\"visible byte length\"", compiled)
        self.assertIn("table_cell_projection_indices(&parsed,", compiled)

    def test_compiled_rich_text_does_not_materialize_duplicate_cluster_index(self):
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )

        self.assertNotIn("cluster_ranges:", compiled)
        self.assertNotIn("pub fn cluster_ranges", compiled)
        self.assertNotIn(".grapheme_indices(true)", compiled)
        self.assertNotIn("unicode_segmentation::UnicodeSegmentation", compiled)

    def test_rich_table_projection_uses_bounded_interval_owner(self):
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )
        rich_ui = (ROOT / "zircon_runtime/src/ui/text/rich_text.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("struct RichRangeIntervalIndex", compiled)
        self.assertIn("fn collect_intersections", compiled)
        self.assertIn("RichRangeIntervalIndex::new", compiled)
        self.assertIn(".windows(2)", compiled)
        self.assertIn("interval_entry_key", compiled)
        self.assertNotIn("let projected_runs = parsed", compiled)
        self.assertNotIn("let projected_paragraphs = parsed", compiled)
        self.assertNotIn("let projected_tables = parsed", compiled)
        self.assertNotIn("run_indices.sort_unstable()", rich_ui)
        self.assertNotIn("paragraph_indices.sort_unstable()", rich_ui)
        self.assertNotIn("table_indices.sort_unstable()", rich_ui)

    def test_rich_decorator_dispatch_has_one_exact_tag_index(self):
        decorator = (ROOT / "zircon_runtime/src/text/rich/decorator.rs").read_text(
            encoding="utf-8"
        )
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        builder = (ROOT / "zircon_runtime/src/text/rich/parser/builder.rs").read_text(
            encoding="utf-8"
        )
        tests = (ROOT / "zircon_runtime/src/text/rich/tests/admission.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "HashMap<String, Box<dyn RichTextDecorator>>", decorator
        )
        self.assertIn(".get(tag)", decorator)
        self.assertNotIn(".iter()\n            .find", decorator)
        self.assertIn("catch_unwind", decorator)
        self.assertIn("DecoratorPanicked", decorator)
        self.assertIn("DecoratorMetadataBudgetExceeded", decorator)
        self.assertIn("max_decorator_metadata_bytes_per_call", decorator)
        self.assertIn("pub max_retained_run_metadata_bytes: usize", admission)
        self.assertIn("RunMetadataBudgetExceeded", admission)
        self.assertIn("consumed_run_metadata_bytes", builder)
        self.assertIn("rich_parser_isolates_decorator_panics_as_typed_failure", tests)
        self.assertIn("rich_parser_budgets_decorator_and_retained_run_metadata", tests)

    def test_rich_parser_production_api_keeps_one_immutable_artifact_owner(self):
        registry = (
            ROOT / "zircon_runtime/src/text/rich/parser_registry.rs"
        ).read_text(encoding="utf-8")

        self.assertNotIn("    pub fn parse(\n", registry)
        self.assertNotIn(".map(|compiled| compiled.parsed().clone())", registry)
        self.assertIn(
            "#[cfg(test)]\npub(crate) fn parse_rich_text(", registry
        )
        self.assertIn("pub fn compile(", registry)

    def test_rich_parser_identity_and_generations_fail_closed_without_reuse(self):
        parser_registry = (
            ROOT / "zircon_runtime/src/text/rich/parser_registry.rs"
        ).read_text(encoding="utf-8")
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        decorator = (ROOT / "zircon_runtime/src/text/rich/decorator.rs").read_text(
            encoding="utf-8"
        )
        emoji = (
            ROOT / "zircon_runtime/src/text/rich/emoji_shortcode.rs"
        ).read_text(encoding="utf-8")
        adapter = (ROOT / "zircon_runtime/src/ui/text/rich_text.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("fetch_update", parser_registry)
        self.assertNotIn("fetch_add", parser_registry)
        self.assertIn("checked_add", parser_registry)
        self.assertNotIn("wrapping_add", parser_registry)
        self.assertIn("ParserIdentityExhausted", admission)
        self.assertIn("GenerationExhausted", decorator)
        self.assertIn("GenerationExhausted", emoji)
        self.assertLess(
            parser_registry.index("next_decorator_generation()?"),
            parser_registry.index("self.decorators.register(decorator)?"),
        )
        self.assertLess(
            parser_registry.index("next_emoji_generation()?"),
            parser_registry.index("self.emoji_shortcodes.register(name, replacement)?"),
        )
        self.assertIn(
            "RichTextParseError::ParserIdentityExhausted",
            adapter,
        )
        self.assertIn(
            "parser_identity_and_generation_exhaustion_never_reuse_cache_identity",
            parser_registry,
        )

    def test_rich_compiler_cache_is_parser_and_surface_session_owned(self):
        cache = (
            ROOT / "zircon_runtime/src/text/cache/rich_cache.rs"
        ).read_text(encoding="utf-8")
        cache_mod = (ROOT / "zircon_runtime/src/text/cache/mod.rs").read_text(
            encoding="utf-8"
        )
        parser = (
            ROOT / "zircon_runtime/src/text/rich/parser_registry.rs"
        ).read_text(encoding="utf-8")
        session = (ROOT / "zircon_runtime/src/text/layout_session.rs").read_text(
            encoding="utf-8"
        )
        session_tests = (
            ROOT / "zircon_runtime/src/text/layout_session/tests.rs"
        ).read_text(encoding="utf-8")
        adapter = (ROOT / "zircon_runtime/src/ui/text/rich_text.rs").read_text(
            encoding="utf-8"
        )
        ui_text_mod = (ROOT / "zircon_runtime/src/ui/text/mod.rs").read_text(
            encoding="utf-8"
        )
        measure_cache = (
            ROOT / "zircon_runtime/src/ui/text/measure_cache.rs"
        ).read_text(encoding="utf-8")
        measurement = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/measurement.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct CompiledRichTextCacheOwner", cache)
        self.assertNotIn("fn shared_cache()", cache)
        self.assertNotIn("shared_compiled_rich_text_cache_report", cache)
        self.assertNotIn("shared_compiled_rich_text_cache_report", cache_mod)
        self.assertIn("cache: CompiledRichTextCacheOwner", parser)
        self.assertIn("#[cfg(test)]\npub(super) fn shared_builtin_parser", parser)
        self.assertIn("rich_text_parser: Arc<RichTextParser>", session)
        self.assertIn("pub(crate) fn compile_rich_text(", session)
        self.assertIn("pub(crate) fn lookup_compiled_rich_text(", session)
        self.assertNotIn(
            "use crate::text::rich::parser_registry::compile_rich_text",
            adapter,
        )
        self.assertIn(
            "#[cfg(test)]\npub(crate) use rich_text::parse_source_text;",
            ui_text_mod,
        )
        self.assertNotIn(
            "link_at_layout_point, parse_source_text, UiParsedText",
            ui_text_mod,
        )
        self.assertIn("provider.compile_rich_text(", adapter)
        self.assertIn("parse_source_text_with_provider", adapter)
        self.assertIn(
            "take_compiled_rich_text_cache_report()",
            measure_cache,
        )
        self.assertNotIn("CompiledRichTextCacheFrameSampler", measure_cache)
        self.assertIn(
            "use super::super::rich_text::parse_source_text_with_provider;",
            measurement,
        )
        self.assertNotIn(
            "use super::super::rich_text::parse_source_text;",
            measurement,
        )
        self.assertGreaterEqual(
            measurement.count("parse_source_text_with_provider("),
            2,
        )
        self.assertIn(
            "rich_parser_cache_isolated_by_surface_session_owner",
            session_tests,
        )
        self.assertGreaterEqual(parser.count("self.cache.clear();"), 2)
        self.assertIn(
            "provider_generation_publication_retires_cache_without_revoking_last_use_artifacts",
            parser,
        )

    def test_rich_italic_and_features_reach_font_selection_and_shaped_cache(self):
        style = (ROOT / "zircon_runtime/src/text/model/style.rs").read_text(
            encoding="utf-8"
        )
        metrics = (ROOT / "zircon_runtime/src/text/layout/rich/metrics.rs").read_text(
            encoding="utf-8"
        )
        request = (ROOT / "zircon_runtime/src/text/model/shaped_run.rs").read_text(
            encoding="utf-8"
        )
        query = (ROOT / "zircon_runtime/src/text/font/query.rs").read_text(
            encoding="utf-8"
        )
        cache = (ROOT / "zircon_runtime/src/text/cache/shaped_cache.rs").read_text(
            encoding="utf-8"
        )
        cosmic = (ROOT / "zircon_runtime/src/text/shaping/cosmic.rs").read_text(
            encoding="utf-8"
        )
        cosmic_tests = (
            ROOT / "zircon_runtime/src/text/shaping/cosmic/tests.rs"
        ).read_text(encoding="utf-8")
        service = (ROOT / "zircon_runtime/src/text/service.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub italic: bool", style)
        self.assertIn("pub features: Arc<[OpenTypeFeature]>", style)
        self.assertIn("legacy_text_style_defaults_new_shaping_identity_fields", style)
        self.assertIn("BTreeMap", request)
        self.assertIn(
            "open_type_feature_normalization_keeps_one_last_declared_value_per_tag",
            (ROOT / "zircon_runtime/src/text/model/shaped_run/tests.rs").read_text(
                encoding="utf-8"
            ),
        )
        self.assertIn("style.italic = italic;", metrics)
        self.assertIn("style.features = Arc::from(features.as_slice());", metrics)
        self.assertGreaterEqual(request.count("features: style.features.as_ref()"), 2)
        self.assertIn("style: if style.italic", query)
        self.assertIn("pub(crate) font_italic: bool", cache)
        self.assertIn("font_italic: request.style.italic", cache)
        self.assertIn("CosmicStyle::Italic", cosmic)
        self.assertIn("italic: request.font.italic", service)
        self.assertIn("neutral_font_request_projects_italic_to_backend_style", service)
        self.assertIn("attrs_apply_italic_style", cosmic_tests)

    def test_rich_representation_counts_fail_before_owner_vectors_grow(self):
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        builder = (ROOT / "zircon_runtime/src/text/rich/parser/builder.rs").read_text(
            encoding="utf-8"
        )
        table = (ROOT / "zircon_runtime/src/text/rich/bbcode_table.rs").read_text(
            encoding="utf-8"
        )
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        run_alignment = (
            ROOT / "zircon_runtime/src/text/rich/parser/run_alignment.rs"
        ).read_text(encoding="utf-8")
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )
        tests = (ROOT / "zircon_runtime/src/text/rich/tests/admission.rs").read_text(
            encoding="utf-8"
        )

        for field in (
            "pub max_runs: usize",
            "pub max_paragraphs: usize",
            "pub max_tables: usize",
            "pub max_table_cells: usize",
            "pub max_projection_indices: usize",
        ):
            self.assertIn(field, admission)
        self.assertIn("with_representation_limits", admission)
        for error in (
            "RunCountBudgetExceeded",
            "ParagraphCountBudgetExceeded",
            "TableCountBudgetExceeded",
            "TableCellCountBudgetExceeded",
            "ProjectionIndexBudgetExceeded",
        ):
            self.assertIn(error, admission)
        for owner in ("admit_run", "push_paragraph", "push_table"):
            self.assertIn(owner, builder)
        self.assertIn("BbCodeTableState::new", parser)
        self.assertIn("align_runs_to_graphemes_bounded", parser)
        self.assertIn("max_runs", parser)
        self.assertLessEqual(len(parser.splitlines()), 800)
        self.assertLessEqual(len(run_alignment.splitlines()), 800)
        self.assertIn("let mut run_index", run_alignment)
        self.assertIn("RunCountBudgetExceeded", run_alignment)
        self.assertIn("collect_intersections(", compiled)
        self.assertIn(
            "rich_parser_rejects_run_paragraph_table_and_cell_growth_before_publish", tests
        )
        self.assertIn(
            "rich_parser_rejects_projection_index_growth_before_compiled_publish", tests
        )

    def test_rich_block_and_table_depth_fail_typed_without_suppression(self):
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        blocks = (ROOT / "zircon_runtime/src/text/rich/bbcode_blocks.rs").read_text(
            encoding="utf-8"
        )
        tables = (ROOT / "zircon_runtime/src/text/rich/bbcode_table.rs").read_text(
            encoding="utf-8"
        )
        table_children = "\n".join(
            path.read_text(encoding="utf-8")
            for path in (ROOT / "zircon_runtime/src/text/rich/bbcode_table").glob("*.rs")
        )
        tests = (ROOT / "zircon_runtime/src/text/rich/tests/admission.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub max_block_depth: usize", admission)
        self.assertIn("pub max_table_depth: usize", admission)
        self.assertIn("BlockDepthBudgetExceeded", admission)
        self.assertIn("TableDepthBudgetExceeded", admission)
        self.assertNotIn("suppressed_list_depth", blocks)
        self.assertNotIn("suppressed_depth", tables)
        self.assertNotIn("unwrap_or(u16::MAX)", tables)
        self.assertNotIn("unwrap_or(u16::MAX)", table_children)
        self.assertIn(
            "rich_parser_rejects_block_and_table_depth_before_stack_growth", tests
        )

    def test_rich_parse_failure_is_typed_and_not_retained_as_cache_residency(self):
        cache = (ROOT / "zircon_runtime/src/text/cache/rich_cache.rs").read_text(
            encoding="utf-8"
        )
        layout_error = (
            ROOT / "zircon_runtime/src/core/framework/text/layout_error.rs"
        ).read_text(encoding="utf-8")
        adapter = (ROOT / "zircon_runtime/src/ui/text/rich_text.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "OnceLock<Result<Arc<CompiledRichText>, RichTextParseError>>", cache
        )
        self.assertIn(
            ".record_failed(&cell, format, content_trust, generation);", cache
        )
        self.assertIn("RichTextBudgetExceeded", layout_error)
        self.assertIn(".map_err(rich_parse_layout_error)", adapter)
        self.assertIn("RichTextParseError::DecoratorPanicked", adapter)
        self.assertIn("TextLayoutError::LayoutFailed", adapter)
        self.assertIn("_ => TextLayoutError::RichTextBudgetExceeded", adapter)

    def test_rich_icons_use_typed_assets_and_share_layout_paint_geometry(self):
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        decorator = (
            ROOT / "zircon_runtime/src/text/rich/inline_decorators.rs"
        ).read_text(encoding="utf-8")
        metrics = (
            ROOT / "zircon_runtime/src/text/layout/rich/metrics.rs"
        ).read_text(encoding="utf-8")
        vertical = (
            ROOT / "zircon_runtime/src/text/layout/rich_vertical.rs"
        ).read_text(encoding="utf-8")
        dependency = (
            ROOT / "zircon_runtime/src/text/rich/compiled/dependency.rs"
        ).read_text(encoding="utf-8")
        semantics = (
            ROOT / "zircon_runtime/src/text/rich/compiled/semantic_text.rs"
        ).read_text(encoding="utf-8")
        renderer = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/rich_text.rs"
        ).read_text(encoding="utf-8")
        texture_collector = (
            ROOT / "zircon_runtime/src/graphics/scene/resources/ui_texture.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub struct RichIconAssetId(ResourceId);", model)
        icon_model = model.split("Icon {", 1)[1].split("Widget {", 1)[0]
        for field in (
            "asset: RichIconAssetId",
            "size: Vec2",
            "baseline: InlineBaseline",
            "alternative_text: Option<String>",
        ):
            self.assertIn(field, icon_model)
        self.assertNotIn("glyph:", icon_model)
        self.assertNotIn("font:", icon_model)
        self.assertNotIn("DEFAULT_ICON_FONT_FAMILY", decorator)
        self.assertIn("controlled_resource_locator", decorator)
        self.assertIn("InlineObjectRef::Icon { size, baseline, .. }", metrics)
        self.assertIn("InlineObjectRef::Icon { size, .. }", vertical)
        self.assertIn("IconAsset(RichIconAssetId)", dependency)
        self.assertIn("RichTextDependency::IconAsset(*asset)", dependency)
        self.assertIn("InlineObjectRef::Icon", semantics)
        self.assertIn("alternative_text: Some(alternative_text)", semantics)
        icon_paint = renderer.split("InlineObjectRef::Icon { asset, .. } =>", 1)[1]
        icon_paint = icon_paint.split("InlineObjectRef::Widget", 1)[0]
        self.assertIn("ScreenSpaceUiImageBatch", icon_paint)
        self.assertIn("texture: asset.resource_id()", icon_paint)
        self.assertNotIn("push_text_batch", icon_paint)
        self.assertIn(
            "RichTextDependency::IconAsset(asset) => asset.resource_id()",
            texture_collector,
        )

    def test_inline_widget_artifact_owns_local_slot_and_surface_resolves_node(self):
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        decorator = (
            ROOT / "zircon_runtime/src/text/rich/inline_decorators.rs"
        ).read_text(encoding="utf-8")
        projection = (
            ROOT / "zircon_runtime/src/ui/text/inline_widget.rs"
        ).read_text(encoding="utf-8")
        arrangement = (
            ROOT / "zircon_runtime/src/ui/layout/pass/inline_widgets.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub struct RichInlineWidgetSlotId(u64);", model)
        widget_model = model.split("Widget {", 1)[1].split("}\n}", 1)[0]
        self.assertIn("slot: RichInlineWidgetSlotId", widget_model)
        self.assertNotIn("id: u64", widget_model)
        self.assertIn("RichInlineWidgetSlotId::new", decorator)
        self.assertIn("InlineObjectRef::Widget { slot, size }", decorator)
        self.assertNotIn("UiNodeId", projection)
        self.assertIn("pub(crate) slot: RichInlineWidgetSlotId", projection)
        self.assertIn("UiNodeId::new(binding.slot.value())", arrangement)
        self.assertIn("direct_children.contains(&node_id)", arrangement)

    def test_measure_line_contract_tests_have_a_bounded_module_owner(self):
        measure = (ROOT / "zircon_runtime/src/text/layout/measure.rs").read_text(
            encoding="utf-8"
        )
        contract_test_path = (
            ROOT
            / "zircon_runtime/src/text/layout/measure/measured_line_contract_tests.rs"
        )

        self.assertLessEqual(len(measure.splitlines()), 800)
        self.assertTrue(contract_test_path.exists())
        contract_tests = contract_test_path.read_text(encoding="utf-8")
        self.assertIn("mod measured_line_contract_tests;", measure)
        self.assertNotIn("mod measured_line_contract_tests {", measure)
        self.assertIn(
            "measured_line_reuses_one_shape_for_advances_and_face_metrics",
            contract_tests,
        )
        self.assertIn("text_size_sums_the_metrics_of_each_physical_line", contract_tests)

    def test_cosmic_backend_tests_have_a_bounded_module_owner(self):
        cosmic = (ROOT / "zircon_runtime/src/text/shaping/cosmic.rs").read_text(
            encoding="utf-8"
        )
        test_path = ROOT / "zircon_runtime/src/text/shaping/cosmic/tests.rs"

        self.assertLessEqual(len(cosmic.splitlines()), 800)
        self.assertTrue(test_path.exists())
        tests = test_path.read_text(encoding="utf-8")
        self.assertIn("mod tests;", cosmic)
        self.assertNotIn("mod tests {", cosmic)
        self.assertIn("glyph_layout_offsets_are_projected_to_pixels", tests)
        self.assertIn("attrs_apply_normalized_open_type_features", tests)
        self.assertIn("cosmic_rich_line_starts_follow_backend_bidi_paragraphs", tests)

    def test_rich_parser_performance_tests_have_a_bounded_module_owner(self):
        rich_tests = (ROOT / "zircon_runtime/src/text/rich/tests.rs").read_text(
            encoding="utf-8"
        )
        performance_test_path = (
            ROOT / "zircon_runtime/src/text/rich/tests/parser_performance.rs"
        )

        self.assertLessEqual(len(rich_tests.splitlines()), 800)
        self.assertTrue(performance_test_path.exists())
        performance_tests = performance_test_path.read_text(encoding="utf-8")
        self.assertIn("mod parser_performance;", rich_tests)
        self.assertIn(
            "text_rich_unterminated_marker_release_benchmark_evidence",
            performance_tests,
        )
        self.assertIn(
            "text_rich_active_tag_index_release_benchmark_evidence",
            performance_tests,
        )
        self.assertIn("legacy_mismatched_close_scan", performance_tests)

    def test_glyph_artifact_cluster_geometry_tests_have_a_bounded_module_owner(self):
        artifact_tests = (
            ROOT / "zircon_runtime/src/text/glyph_artifact/tests.rs"
        ).read_text(encoding="utf-8")
        geometry_test_path = (
            ROOT / "zircon_runtime/src/text/glyph_artifact/tests/cluster_geometry.rs"
        )

        self.assertLessEqual(len(artifact_tests.splitlines()), 800)
        self.assertTrue(geometry_test_path.exists())
        geometry_tests = geometry_test_path.read_text(encoding="utf-8")
        self.assertIn("mod cluster_geometry;", artifact_tests)
        self.assertIn(
            "artifact_cluster_geometry_snaps_ligature_caret_and_selection_to_whole_glyph",
            geometry_tests,
        )
        self.assertIn(
            "artifact_cluster_geometry_maps_rtl_affinity_to_opposite_visual_edges",
            geometry_tests,
        )
        self.assertIn(
            "artifact_cluster_geometry_rejects_a_stale_font_generation",
            geometry_tests,
        )

    def test_rich_html_parser_has_a_bounded_format_owner(self):
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        html_parser_path = ROOT / "zircon_runtime/src/text/rich/parser/html.rs"

        self.assertLessEqual(len(parser.splitlines()), 800)
        self.assertTrue(html_parser_path.exists())
        html_parser = html_parser_path.read_text(encoding="utf-8")
        self.assertIn("mod html;", parser)
        self.assertIn("html::parse_html(markup, budget, content_trust)", parser)
        self.assertNotIn("fn parse_html(", parser)
        self.assertIn("pub(super) fn parse_html(", html_parser)
        self.assertIn("append_html_text", html_parser)
        self.assertIn("align_runs_to_graphemes_bounded", html_parser)

    def test_ui_render_extract_has_bounded_popup_and_text_prewarm_owners(self):
        extract = (
            ROOT / "zircon_runtime/src/ui/surface/render/extract.rs"
        ).read_text(encoding="utf-8")
        popup_path = (
            ROOT / "zircon_runtime/src/ui/surface/render/extract/popup_anchor.rs"
        )
        prewarm_path = (
            ROOT / "zircon_runtime/src/ui/surface/render/extract/owner_text_prewarm.rs"
        )

        self.assertLessEqual(len(extract.splitlines()), 800)
        self.assertTrue(popup_path.exists())
        self.assertTrue(prewarm_path.exists())
        popup = popup_path.read_text(encoding="utf-8")
        prewarm = prewarm_path.read_text(encoding="utf-8")
        self.assertIn("mod popup_anchor;", extract)
        self.assertIn("mod owner_text_prewarm;", extract)
        self.assertIn("pub(super) fn resolve_popup_anchor_frame", popup)
        self.assertIn("fn pointer_anchor_resolves_from_transient_surface_state", popup)
        self.assertIn("pub(super) struct OwnerTextPrewarmCollection", prewarm)
        self.assertIn("pub(super) fn collect_owner_text_prewarm_requests", prewarm)
        self.assertNotIn("struct OwnerTextPrewarmCollection", extract)

    def test_rich_format_and_link_consumers_follow_typed_hard_cuts(self):
        inline_semantics = (
            ROOT / "zircon_runtime/src/text/rich/tests/inline_semantics.rs"
        ).read_text(encoding="utf-8")
        admission = (
            ROOT / "zircon_runtime/src/text/rich/tests/admission.rs"
        ).read_text(encoding="utf-8")
        inline_widgets = (
            ROOT / "zircon_runtime/src/ui/tests/text_pipeline/inline_widgets.rs"
        ).read_text(encoding="utf-8")
        rich_consumer_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for source_root in (
                "zircon_runtime/src/text",
                "zircon_runtime/src/ui/text",
                "zircon_runtime/src/ui/tests/text_pipeline",
                "zircon_runtime/src/graphics/scene/scene_renderer/ui",
                "zircon_runtime/src/graphics/text_transport",
                "zircon_runtime_interface/src/ui",
            )
            for path in (ROOT / source_root).rglob("*.rs")
        )

        self.assertNotIn(".href", inline_semantics)
        self.assertIn("target.matches_display", inline_semantics)
        self.assertNotIn("href:", admission)
        self.assertIn("target: UiRichLinkTarget::parse", admission)
        self.assertNotIn("[link=", rich_consumer_sources)
        self.assertIn("[url=", rich_consumer_sources)
        self.assertNotIn('rich_text_format = "bbcode"', inline_widgets)
        self.assertIn('rich_text_format = "bbcode_v1"', inline_widgets)
        self.assertIsNone(
            re.search(
                r"\bUiRichTextFormat::(?:BbCode|Markdown|Html|HtmlSubset)\b",
                rich_consumer_sources,
            )
        )
        self.assertIsNone(
            re.search(
                r"\bRichTextFormat::(?:BbCode|Markdown|Html|HtmlSubset)\b",
                rich_consumer_sources,
            )
        )


if __name__ == "__main__":
    unittest.main()
