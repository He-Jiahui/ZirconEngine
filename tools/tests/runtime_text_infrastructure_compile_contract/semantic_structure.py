import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class RuntimeTextInfrastructureCompileContractTests(unittest.TestCase):
    def test_rich_layout_projects_each_shape_once_onto_paragraph_grapheme_ranges(self):
        measure = (ROOT / "zircon_runtime/src/text/layout/measure.rs").read_text(
            encoding="utf-8"
        )
        rich_index = (
            ROOT / "zircon_runtime/src/text/layout/rich_advance_index.rs"
        ).read_text(encoding="utf-8")
        rich_tests = (
            ROOT / "zircon_runtime/src/text/layout/rich_advance_index/tests.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "pub(crate) fn project_shaped_geometry_to_source_ranges",
            measure,
        )
        self.assertIn("paragraph_grapheme_ranges", rich_index)
        self.assertIn(
            "project_shaped_geometry_to_source_ranges",
            rich_index,
        )
        self.assertIn(
            "rich_span_index_keeps_cross_run_combining_grapheme_atomic",
            rich_tests,
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
        self.assertIn(
            "text_size_keeps_multiline_height_finite_when_f32_accumulation_overflows",
            contract_tests,
        )

    def test_measure_text_size_uses_finite_dual_precision_height_accumulation(self):
        measure = (ROOT / "zircon_runtime/src/text/layout/measure.rs").read_text(
            encoding="utf-8"
        )
        contract_tests = (
            ROOT
            / "zircon_runtime/src/text/layout/measure/measured_line_contract_tests.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("let mut height = FiniteGeometryAccumulator::default()", measure)
        self.assertIn("height.add(line_height)", measure)
        self.assertIn("finite_sum(", measure)
        self.assertIn(
            "measured_source_range_width_keeps_extreme_glyph_sum_finite",
            contract_tests,
        )

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
        geometry = (
            ROOT / "zircon_runtime/src/text/glyph_artifact/geometry.rs"
        ).read_text(encoding="utf-8")
        visual_projection = (
            ROOT / "zircon_runtime/src/text/glyph_artifact/visual_projection.rs"
        ).read_text(encoding="utf-8")
        logical_virtual_projection = (
            ROOT
            / "zircon_runtime/src/text/layout/logical_virtual_line/glyph_projection.rs"
        ).read_text(encoding="utf-8")
        logical_virtual_line = (
            ROOT / "zircon_runtime/src/text/layout/logical_virtual_line.rs"
        ).read_text(encoding="utf-8")
        geometry_test_path = (
            ROOT / "zircon_runtime/src/text/glyph_artifact/tests/cluster_geometry.rs"
        )

        self.assertLessEqual(len(artifact_tests.splitlines()), 800)
        self.assertTrue(geometry_test_path.exists())
        geometry_tests = geometry_test_path.read_text(encoding="utf-8")
        self.assertIn("mod cluster_geometry;", artifact_tests)
        self.assertIn("FiniteGeometryAccumulator", geometry)
        self.assertIn(
            "let mut advance = FiniteGeometryAccumulator::default()", geometry
        )
        self.assertIn("advance.add(cluster.advance)", geometry)
        self.assertNotIn(".sum::<f32>()", geometry)
        self.assertIn("use crate::text::layout_geometry::finite_sum;", visual_projection)
        self.assertGreaterEqual(visual_projection.count("finite_sum(["), 3)
        self.assertNotIn("advance += advance.max(0.0)", visual_projection)
        self.assertIn(
            "resolved_source_projection_keeps_multicluster_glyph_advance_finite",
            visual_projection,
        )
        self.assertIn(
            "use crate::text::layout_geometry::finite_sum;",
            logical_virtual_projection,
        )
        self.assertIn("finite_sum([", logical_virtual_projection)
        self.assertNotIn("advance += advance.max(0.0)", logical_virtual_projection)
        self.assertIn(
            "logical_virtual_projection_keeps_multicluster_glyph_advance_finite",
            logical_virtual_line,
        )
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
        self.assertIn(
            "artifact_cluster_geometry_keeps_extreme_prefix_and_caret_geometry_finite",
            geometry_tests,
        )
        virtual_replacement_tests = (
            ROOT / "zircon_runtime/src/text/glyph_artifact/tests/virtual_replacement.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "virtual_replacement_geometry_keeps_extreme_receipt_prefixes_finite",
            virtual_replacement_tests,
        )
        visual_projection_tests = (
            ROOT
            / "zircon_runtime/src/text/glyph_artifact/tests/visual_projection_contract.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "visual_projection_keeps_multicluster_glyph_advance_finite",
            visual_projection_tests,
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
