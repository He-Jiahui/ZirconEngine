import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class RuntimeTextInfrastructureCompileContractTests(unittest.TestCase):
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
        self.assertIn(
            "checked_source_range_to_u32(text, range.start, range.end)", source
        )
        self.assertIn("text.get(range.start..range.end)", source)
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
