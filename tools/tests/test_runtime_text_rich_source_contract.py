import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RuntimeTextRichSourceContractTests(unittest.TestCase):
    def test_validator_checks_structural_and_utf8_invariants(self):
        source = (ROOT / "zircon_runtime/src/text/layout/rich_source.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("validate_rich_text_layout_source", source)
        self.assertIn("source.run(index).ok_or(TextLayoutError::LayoutFailed)?", source)
        self.assertIn("run.source_index == u32::MAX", source)
        self.assertIn("run.source_index <= previous", source)
        self.assertIn("run.byte_range.0 < previous_end", source)
        self.assertIn("run.byte_range.1 <= run.byte_range.0", source)
        self.assertIn("source.text().get(start..end).is_none()", source)

    def test_validator_is_before_rich_index_source_spans(self):
        source = (ROOT / "zircon_runtime/src/text/layout/rich_advance_index.rs").read_text(
            encoding="utf-8"
        )
        source_spans = source.index("fn source_spans")
        validation = source.index("for_each_validated_rich_run(source", source_spans)
        self.assertGreater(validation, source_spans)
        self.assertNotIn("validate_rich_text_layout_source(source)?;", source)
        self.assertIn("if run_start < cursor || run_end <= cursor", source)
        self.assertIn(") -> Result<Vec<SourceSpan<'a>>, TextLayoutError>", source)
        self.assertIn("source.run(index).ok_or(TextLayoutError::LayoutFailed)?", (ROOT / "zircon_runtime/src/text/layout/rich_source.rs").read_text(encoding="utf-8"))

    def test_rich_line_range_extraction_fails_closed_on_invalid_hard_line_slice(self):
        source = (ROOT / "zircon_runtime/src/text/layout/rich.rs").read_text(
            encoding="utf-8"
        )
        self.assertGreaterEqual(source.count("TextLayoutError::LayoutFailed"), 2)
        self.assertNotIn(
            "let Some(text) = source.text().get(start..end) else {\n            continue;",
            source,
        )
        self.assertNotIn("unwrap_or(usize::MAX)", source)
        self.assertNotIn("unwrap_or(u32::MAX)", source)
        self.assertIn("checked_source_range(source.text(), forced_range)", source)
        self.assertIn(
            "pub(crate) fn rich_forced_line_ranges(text: &str) -> TextLayoutOutcome<Vec<(u32, u32)>>",
            source,
        )
        self.assertIn("hard_line_count(text)", source)
        self.assertIn("visit_hard_lines(text", source)
        vertical = (ROOT / "zircon_runtime/src/text/layout/rich_vertical.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("TextLayoutError::LayoutFailed", vertical)
        self.assertNotIn(
            "let Some(text) = source.text().get(start..end) else {\n            continue;",
            vertical,
        )

    def test_rich_item_projection_does_not_publish_zero_advance_success(self):
        source = (ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_layout.rs").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("TextShapingOutcome::Ready(Vec::new())", source)
        self.assertIn("TextShapingOutcome::failed(TextLayoutError::LayoutFailed)", source)
        vertical = (ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_layout_vertical.rs").read_text(
            encoding="utf-8"
        )
        self.assertGreaterEqual(
            vertical.count("TextShapingOutcome::failed(TextLayoutError::LayoutFailed)"), 2
        )

    def test_rich_table_ranges_are_checked_without_clamping_legal_empty_cells(self):
        source = (ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("fn checked_table_source_range", source)
        self.assertIn("let mut previous_cell_end = table_range.start", source)
        self.assertIn("if range.start < previous_cell_end", source)
        self.assertIn("if table_start < cursor", source)
        self.assertIn("if start > end", source)
        self.assertIn("source_start", source)
        self.assertNotIn("saturating_sub(parsed.source_offset())", source)
        self.assertNotIn("unwrap_or_default()\n                .saturating_sub", source)

    def test_rich_table_segment_projection_rejects_invalid_utf8_ranges(self):
        source = (ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/source_slice.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("parsed.project_range(range, parent_table_depth)", source)
        self.assertIn("Err(error) => return TextShapingOutcome::failed(error)", source)

    def test_ui_projection_rejects_invalid_indices_instead_of_silently_dropping_them(self):
        source = (ROOT / "zircon_runtime/src/ui/text/rich_text.rs").read_text(
            encoding="utf-8"
        )
        tests = (ROOT / "zircon_runtime/src/ui/text/rich_text/tests.rs").read_text(
            encoding="utf-8"
        )

        self.assertNotIn(".map(|index| index as u32)", source)
        self.assertNotIn(
            ".filter_map(|run| u32::try_from(run.rich_run_index).ok())", source
        )
        self.assertIn("fn checked_projection_index", source)
        self.assertIn(
            "rich_ui_projection_rejects_invalid_compiled_indices_instead_of_dropping_them",
            tests,
        )

        owner = (ROOT / "zircon_runtime/src/ui/text/rich_text.rs").read_text(encoding="utf-8")
        self.assertIn(") -> Result<Self, TextLayoutError>", owner)
        self.assertIn("return Err(TextLayoutError::LayoutFailed);", owner)
        self.assertIn("local_range.start > local_range.end", owner)
        self.assertIn("!self.text().is_char_boundary(local_range.start)", owner)
        self.assertNotIn("local_range.start.min(self.text().len())", owner)
        self.assertNotIn("local_range.end.min(self.text().len()).max(start)", owner)

    def test_rich_table_delimiter_trimming_reuses_hard_line_owner(self):
        source = (ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("use crate::text::is_hard_line_separator;", source)
        self.assertIn("is_hard_line_separator(character)", source)
        self.assertIn("-> Result<std::ops::Range<usize>, TextLayoutError>", source)
        self.assertNotIn("text.as_bytes()[start] == b'\\n'", source)
        self.assertNotIn("text.as_bytes()[end - 1] == b'\\n'", source)
        self.assertNotIn("start.min(text.len())", source)
        self.assertNotIn("end.min(text.len()).max(start)", source)

    def test_vertical_rich_column_ranges_fail_closed_on_numeric_and_slice_errors(self):
        source = (ROOT / "zircon_runtime/src/text/layout/rich_vertical.rs").read_text(
            encoding="utf-8"
        )
        owner = (ROOT / "zircon_runtime/src/text/layout/rich_source.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("fn checked_source_range", owner)
        self.assertIn("fn checked_source_range_to_u32", owner)
        self.assertIn("checked_source_range", source)
        self.assertIn("checked_source_range_to_u32", source)
        self.assertIn("fn push_checked_range", source)
        self.assertIn("checked_source_range(source.text(), source_range)?", source)
        self.assertNotIn("usize::try_from(source_range.0).unwrap_or(usize::MAX)", source)
        self.assertNotIn("usize::try_from(range.0).unwrap_or(chunk_start)", source)
        self.assertNotIn("u32::try_from(value).unwrap_or(u32::MAX)", source)

    def test_rich_source_validation_and_projection_share_one_run_pass(self):
        owner = (ROOT / "zircon_runtime/src/text/layout/rich_source.rs").read_text(
            encoding="utf-8"
        )
        index = (ROOT / "zircon_runtime/src/text/layout/rich_advance_index.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("for_each_validated_rich_run(source, |_, _, _| Ok(()))", owner)
        self.assertIn("for_each_validated_rich_run(source, |run, run_start, run_end|", index)
        self.assertEqual(index.count("for_each_validated_rich_run(source"), 1)

    def test_secondary_consumers_reuse_the_same_validator(self):
        artifact = (ROOT / "zircon_runtime/src/text/glyph_artifact/rich.rs").read_text(
            encoding="utf-8"
        )
        measure_cache = (ROOT / "zircon_runtime/src/ui/text/measure_cache.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("resolved_text_spans(source", artifact)
        self.assertIn(") -> Result<Vec<ResolvedRichTextSpan>, TextLayoutError>",
                      (ROOT / "zircon_runtime/src/text/layout/rich_advance_index.rs").read_text(encoding="utf-8"))
        self.assertIn("resolved_text_spans(&parsed, &base_style)\n                    .ok()?", measure_cache)

    def test_rich_advance_index_preserves_deferred_generation_outcome(self):
        source = (ROOT / "zircon_runtime/src/text/layout/rich_advance_index.rs").read_text(
            encoding="utf-8"
        )
        self.assertGreaterEqual(source.count("TextShapingOutcome::Deferred(error)"), 4)
        self.assertIn("TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error)", source)
        self.assertNotIn("measured_grapheme_geometry_with_provider(text, style, provider).into_result()", source)
        self.assertIn("if start > end || source.get(start..end).is_none()", source)
        self.assertNotIn("map_or(TextDirection::Auto", source)

    def test_measurement_projection_rejects_malformed_shaped_ranges(self):
        source = (ROOT / "zircon_runtime/src/text/layout/measure.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("fn validate_shaped_geometry_source", source)
        self.assertIn("validate_shaped_geometry_source(shaped, text)?", source)
        self.assertIn("TextShapingOutcome::from_result(measured_grapheme_geometry_from_shaped", source)
        self.assertNotIn("let start = start.min(source_text.len())", source)
        self.assertNotIn("let end = end.min(source_text.len()).max(start)", source)
        self.assertIn("!run.source_text.is_char_boundary(relative_start)", source)
        self.assertIn("line.source_range.start < previous_line_end", source)
        self.assertIn("measured_grapheme_geometry_rejects_non_boundary_glyph_ranges", 
                      (ROOT / "zircon_runtime/src/text/layout/measure/tests.rs").read_text(encoding="utf-8"))

    def test_gap_fill_semantics_are_documented_and_regressed(self):
        source = (ROOT / "zircon_runtime/src/text/layout/rich_source.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("Gaps are allowed", source)
        self.assertIn("source_contract_accepts_empty_and_partially_covered_text", source)
        plan = (ROOT / "docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md").read_text(
            encoding="utf-8"
        )
        self.assertIn("rich_source_contract_fail_closed_static_implemented", plan)
        self.assertIn("base_style_gap_fill_preserved", plan)

    def test_accessibility_semantics_reuse_current_compiled_rich_generation(self):
        projection = (
            ROOT / "zircon_runtime/src/text/semantic_projection.rs"
        ).read_text(encoding="utf-8")
        accessibility = (
            ROOT / "zircon_runtime/src/ui/accessibility/semantic_text.rs"
        ).read_text(encoding="utf-8")
        resolution = (
            ROOT / "zircon_runtime/src/ui/accessibility/extract/resolution.rs"
        ).read_text(encoding="utf-8")
        surface = (ROOT / "zircon_runtime/src/ui/surface/surface.rs").read_text(
            encoding="utf-8"
        )
        plan = (
            ROOT
            / "docs/plans/zircon_runtime/text/07/2026-08-30-rich-accessibility-semantic-projection-review.md"
        ).read_text(encoding="utf-8")

        self.assertIn("struct RichSemanticProjection", projection)
        self.assertIn("compiled.source_markup() == source_markup", projection)
        self.assertIn("compiled.format() == format", projection)
        self.assertIn(
            "self.compiled.generation() == other.compiled.generation()", projection
        )
        self.assertNotIn("self.compiled == other.compiled", projection)
        self.assertNotIn("RichTextParser", accessibility)
        self.assertNotIn("parse_source_text", accessibility)
        self.assertIn("current_render_commands_for_node(node_id)", accessibility)
        self.assertIn("semantic_text::own_text(surface, node)", resolution)
        self.assertNotIn("name::own_text(node.template_metadata.as_ref())", resolution)
        self.assertIn("fn current_render_commands_for_node", surface)
        self.assertIn(
            "RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete",
            plan,
        )

    def test_hidden_rich_relation_semantics_use_the_surface_text_owner(self):
        projection = (
            ROOT / "zircon_runtime/src/text/semantic_projection.rs"
        ).read_text(encoding="utf-8")
        measure_cache = (
            ROOT / "zircon_runtime/src/ui/text/measure_cache.rs"
        ).read_text(encoding="utf-8")
        surface = (ROOT / "zircon_runtime/src/ui/surface/surface.rs").read_text(
            encoding="utf-8"
        )
        accessibility = (
            ROOT / "zircon_runtime/src/ui/accessibility/semantic_text.rs"
        ).read_text(encoding="utf-8")
        tests = (
            ROOT / "zircon_runtime/src/ui/tests/accessibility/naming_relations.rs"
        ).read_text(encoding="utf-8")
        plan = (
            ROOT
            / "docs/plans/zircon_runtime/text/07/2026-08-30-rich-visibility-independent-semantic-owner-review.md"
        ).read_text(encoding="utf-8")

        self.assertIn("from_compiled_rich_semantic_projection", projection)
        self.assertIn("compile_rich_semantic_projection", measure_cache)
        self.assertIn(".compile_rich_text(source_markup, format)", measure_cache)
        self.assertIn("fn compile_rich_semantic_projection", surface)
        self.assertIn("surface.compile_rich_semantic_projection", accessibility)
        self.assertIn("current_render_commands_for_node(node_id)", accessibility)
        self.assertNotIn("RichTextParser", accessibility)
        self.assertNotIn("parse_source_text", accessibility)
        self.assertIn(
            "hidden_rich_relation_target_uses_surface_text_owner_without_render_command",
            tests,
        )
        self.assertIn(
            "surface.current_render_commands_for_node(id(2)).is_none()", tests
        )
        self.assertIn(
            "RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete",
            plan,
        )
        self.assertIn(
            "RRT-P1-040_typed_children_and_managed_validation_pending", plan
        )

    def test_list_semantics_are_parser_owned_typed_metadata(self):
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        blocks = (ROOT / "zircon_runtime/src/text/rich/bbcode_blocks.rs").read_text(
            encoding="utf-8"
        )
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        projection = (ROOT / "zircon_runtime/src/ui/text/rich_text.rs").read_text(
            encoding="utf-8"
        )
        tests = (ROOT / "zircon_runtime/src/text/rich/tests/block.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub enum RichListItemKind", model)
        self.assertIn("pub enum RichOrderedListMarker", model)
        self.assertIn("pub struct RichListItem", model)
        self.assertIn("pub list_item: Option<RichListItem>", model)
        self.assertNotIn("pub list_prefix: Option", model)
        self.assertIn("RichListItemKind::Ordered", blocks)
        self.assertIn("paragraph.list_item = Some", parser)
        self.assertIn(".list_item", projection)
        self.assertIn("RichOrderedListMarker::AlphaUpper", tests)
        self.assertIn("vec![1, 2, 1]", tests)

    def test_inline_image_semantics_are_compiled_once_under_a_dedicated_budget(self):
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        html = (ROOT / "zircon_runtime/src/text/rich/html_subset.rs").read_text(
            encoding="utf-8"
        )
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )
        semantic_builder = (
            ROOT / "zircon_runtime/src/text/rich/compiled/semantic_text.rs"
        ).read_text(encoding="utf-8")
        admission = (ROOT / "zircon_runtime/src/text/rich/admission.rs").read_text(
            encoding="utf-8"
        )
        projection = (
            ROOT / "zircon_runtime/src/text/semantic_projection.rs"
        ).read_text(encoding="utf-8")
        metadata = (ROOT / "zircon_runtime/src/text/rich/decorator.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("alternative_text: Option<String>", model)
        self.assertIn("tooltip: Option<String>", model)
        self.assertIn('attribute(attributes, "alt")', html)
        self.assertIn('attribute(attributes, "title")', html)
        self.assertIn('"alt", "title"', html)
        self.assertIn("pub max_semantic_text_bytes: usize", admission)
        self.assertIn("SemanticTextByteBudgetExceeded", admission)
        self.assertIn("semantic_text: Arc<str>", compiled)
        self.assertIn("semantic_text::semantic_text_for_inline_runs", compiled)
        self.assertIn("fn semantic_text_for_inline_runs", semantic_builder)
        self.assertIn("max_semantic_text_bytes", compiled)
        self.assertIn("self.compiled.semantic_text()", projection)
        self.assertIn("alternative_text", metadata)
        self.assertIn("tooltip", metadata)

    def test_rich_link_target_is_validated_once_and_remains_typed_to_the_host(self):
        target = (
            ROOT / "zircon_runtime_interface/src/ui/text/rich_link_target.rs"
        ).read_text(encoding="utf-8")
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        html = (ROOT / "zircon_runtime/src/text/rich/html_subset.rs").read_text(
            encoding="utf-8"
        )
        hit = (
            ROOT / "zircon_runtime/src/ui/text/rich_text/link_hit.rs"
        ).read_text(encoding="utf-8")
        effect = (
            ROOT / "zircon_runtime_interface/src/ui/dispatch/input/effect.rs"
        ).read_text(encoding="utf-8")
        result = (
            ROOT / "zircon_runtime_interface/src/ui/dispatch/input/result.rs"
        ).read_text(encoding="utf-8")
        application = (
            ROOT / "zircon_runtime/src/ui/surface/input/effect/link.rs"
        ).read_text(encoding="utf-8")
        plan = (
            ROOT
            / "docs/plans/zircon_runtime/text/07/2026-08-30-rich-link-target-owner-hard-cut.md"
        ).read_text(encoding="utf-8")

        self.assertIn("pub struct UiRichLinkTarget", target)
        self.assertIn("locator: Arc<ResourceLocator>", target)
        self.assertIn("pub fn parse", target)
        self.assertIn("impl<'de> Deserialize<'de> for UiRichLinkTarget", target)
        self.assertIn("ResourceScheme::Builtin", target)
        self.assertNotIn("pub locator: ResourceLocator", target)
        self.assertIn("pub target: UiRichLinkTarget", model)
        self.assertIn('#[serde(rename = "href")]', model)
        self.assertNotIn("pub href: String", model)
        self.assertIn("UiRichLinkTarget::parse", html)
        self.assertNotIn("href.to_string()", html)
        self.assertIn("pub(crate) target: UiRichLinkTarget", hit)
        self.assertIn("link_target: UiRichLinkTarget", effect)
        self.assertIn("link_target: UiRichLinkTarget", result)
        self.assertIn('#[serde(rename = "href")]', effect)
        self.assertIn('#[serde(rename = "href")]', result)
        self.assertNotIn("fn rich_link_target_is_valid", application)
        self.assertNotIn("ResourceLocator::parse", application)
        self.assertNotIn("split_once", application)
        self.assertIn(
            "RRT-P1-030_typed_link_target_foundation_static_complete", plan
        )
        self.assertIn("No timing, allocation, RSS, power", plan)

    def test_rich_link_tooltip_is_shared_from_parser_to_hit_projection(self):
        model = (ROOT / "zircon_runtime/src/text/model/rich.rs").read_text(
            encoding="utf-8"
        )
        html = (ROOT / "zircon_runtime/src/text/rich/html_subset.rs").read_text(
            encoding="utf-8"
        )
        parser = (ROOT / "zircon_runtime/src/text/rich/parser.rs").read_text(
            encoding="utf-8"
        )
        metadata = (ROOT / "zircon_runtime/src/text/rich/decorator.rs").read_text(
            encoding="utf-8"
        )
        compiled = (ROOT / "zircon_runtime/src/text/rich/compiled.rs").read_text(
            encoding="utf-8"
        )
        compiled_memory = (
            ROOT / "zircon_runtime/src/text/rich/compiled/memory.rs"
        ).read_text(encoding="utf-8")
        hit = (
            ROOT / "zircon_runtime/src/ui/text/rich_text/link_hit.rs"
        ).read_text(encoding="utf-8")
        tests = (ROOT / "zircon_runtime/src/text/rich/tests.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("pub tooltip: Option<Arc<str>>", model)
        self.assertIn("#[serde(default, skip_serializing_if = \"Option::is_none\")]", model)
        self.assertIn("impl LinkRef", model)
        self.assertIn(
            "self.tooltip.as_ref().map_or(0, |tooltip| tooltip.len())", model
        )
        self.assertIn('matches_ascii_case(attribute, &["href", "title"])', html)
        self.assertIn("pub(super) fn bbcode_link(", html)
        self.assertIn('bbcode_attribute_value(attributes, "title")', html)
        self.assertIn("html_subset::bbcode_link(value.as_deref(), &attributes", parser)
        self.assertIn("link.retained_heap_bytes()", metadata)
        self.assertIn("mod memory;", compiled)
        self.assertIn("memory::calculate_estimated_bytes(self)", compiled)
        self.assertIn("LinkRef::retained_heap_bytes", compiled_memory)
        self.assertIn("pub(crate) tooltip: Option<Arc<str>>", hit)
        self.assertIn("tooltip: link.tooltip.clone()", hit)
        self.assertIn("text_rich_html_and_bbcode_links_preserve_shared_tooltips", tests)


if __name__ == "__main__":
    unittest.main()
