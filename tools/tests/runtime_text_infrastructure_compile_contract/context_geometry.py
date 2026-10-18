import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[3]


class RuntimeTextInfrastructureCompileContractTests(unittest.TestCase):
    def test_runtime_product_roots_bind_text_through_the_core_context(self):
        dynamic_construction = (
            ROOT / "zircon_runtime/src/dynamic_api/session/construction.rs"
        ).read_text(encoding="utf-8")
        dynamic_extract = (
            ROOT / "zircon_runtime/src/dynamic_api/session/ui_extract_cache.rs"
        ).read_text(encoding="utf-8")
        graphics_construction = (
            ROOT
            / "zircon_runtime/src/graphics/runtime_builtin_graphics/host/module_host/create/create_render_framework.rs"
        ).read_text(encoding="utf-8")
        framework_compatibility = (
            ROOT
            / "zircon_runtime/src/graphics/runtime/render_framework/wgpu_render_framework_construction/construct.rs"
        ).read_text(encoding="utf-8")
        renderer_compatibility = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/construct.rs"
        ).read_text(encoding="utf-8")
        renderer_icon_compatibility = (
            ROOT
            / "zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_construct/new_with_icon_source.rs"
        ).read_text(encoding="utf-8")
        dynamic_extract_product = dynamic_extract.split("#[cfg(test)]\nmod tests", 1)[0]

        self.assertIn("let text_context = text_runtime_context_for_core(&core)", dynamic_construction)
        self.assertIn(
            ".load_runtime_ui_surfaces(&core, text_context.clone())",
            dynamic_construction,
        )
        self.assertIn(
            "RuntimeUiExtractCache::new_with_text_context(&text_context)",
            dynamic_construction,
        )
        self.assertLess(
            dynamic_construction.index("let text_context = text_runtime_context_for_core(&core)"),
            dynamic_construction.index("RuntimeUiExtractCache::new_with_text_context"),
        )
        self.assertIn(
            "UiTextMeasureCache::new_with_text_context(text_context)?",
            dynamic_extract_product,
        )
        self.assertIn("let text_context = text_runtime_context_for_core(core)", graphics_construction)
        self.assertIn("let font_collection = text_context.font_collection();", graphics_construction)
        self.assertIn("            font_collection,", graphics_construction)

        for product_source in (
            dynamic_construction,
            dynamic_extract_product,
            graphics_construction,
        ):
            self.assertNotIn("shared_font_collection_service()", product_source)
            self.assertNotIn("UiTextMeasureCache::default()", product_source)
            self.assertNotIn("SharedTextLayoutSession::new()", product_source)

        compatibility_marker = (
            "// Runtime201 standalone compatibility: Core-owned products inject "
            "TextRuntimeContext.\n"
        )
        for source, expected_global_calls in (
            (framework_compatibility, 1),
            (renderer_compatibility, 1),
            (renderer_icon_compatibility, 2),
        ):
            self.assertEqual(
                source.count("shared_font_collection_service()"),
                expected_global_calls,
            )
            self.assertEqual(source.count(compatibility_marker), expected_global_calls)
            self.assertEqual(
                source.count(
                    compatibility_marker + "            shared_font_collection_service(),"
                ),
                expected_global_calls,
            )

    def test_runtime_context_owns_system_font_policy_before_renderer_construction(self):
        context = (ROOT / "zircon_runtime/src/text/context.rs").read_text(
            encoding="utf-8"
        )
        health = (ROOT / "zircon_runtime/src/text/context/health.rs").read_text(
            encoding="utf-8"
        )
        font_collection = (
            ROOT / "zircon_runtime/src/text/font/shared.rs"
        ).read_text(encoding="utf-8")
        text_module = (ROOT / "zircon_runtime/src/text/module.rs").read_text(
            encoding="utf-8"
        )
        builtin_modules = (
            ROOT / "zircon_runtime/src/builtin/runtime_modules/core_modules.rs"
        ).read_text(encoding="utf-8")
        renderer_text = (
            ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub enum TextSystemFontPolicy", context)
        self.assertIn("system_font_policy: TextSystemFontPolicy", context)
        self.assertIn("discovered_system_face_count: usize", context)
        self.assertIn("FontCollectionService::new_with_system_font_policy", context)
        self.assertIn("pub const fn system_font_policy", health)
        self.assertIn("pub const fn discovered_system_face_count", health)
        self.assertIn("pub(crate) fn new_with_system_font_policy", font_collection)
        self.assertIn("database.apply_system_font_policy(policy)", font_collection)
        self.assertIn("pub const fn for_target(target: RuntimeTargetMode)", text_module)
        self.assertIn("TextRuntimeContext::new_with_system_font_policy", text_module)
        self.assertEqual(
            builtin_modules.count("text::TextModule::for_target(target)"),
            2,
        )
        self.assertNotIn("apply_system_font_policy", renderer_text)
        self.assertNotIn("SystemFontPolicy", renderer_text)

    def test_direction_alias_exact_compare_keeps_all_fingerprint_authorities(self):
        source = (
            ROOT / "zircon_runtime/src/text/cache/shaped_cache.rs"
        ).read_text(encoding="utf-8")
        exact_compare = source.split(
            "fn matches_lookup_except_base_direction", 1
        )[1].split("\n    }", 1)[0]

        self.assertIn("self.font_collection == lookup.font_collection", exact_compare)
        self.assertIn(
            "self.unicode_data_snapshot == lookup.unicode_data_snapshot",
            exact_compare,
        )

    def test_vertical_shape_provider_default_fails_closed_without_horizontal_delegation(self):
        source = (ROOT / "zircon_runtime/src/text/shaping/mod.rs").read_text(
            encoding="utf-8"
        )
        default_vertical = source.split(
            "fn shape_vertical_range_with_kerning", 1
        )[1].split("\n    }", 1)[0]

        self.assertIn("TextLayoutError::UnsupportedWritingMode", default_vertical)
        self.assertNotIn("shape_horizontal_range_with_kerning", default_vertical)

    def test_inline_widget_projection_keeps_visual_geometry_finite(self):
        source = (ROOT / "zircon_runtime/src/ui/text/inline_widget.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("let mut main_offset = FiniteGeometryAccumulator::default()", source)
        self.assertIn("main_offset.add(advance)", source)
        self.assertNotIn("*main_offset +=", source)
        self.assertIn("finite_f32_or_geometry(x_candidate, x_exact)", source)
        self.assertGreaterEqual(source.count("finite_sum(["), 3)
        self.assertIn(
            "visual_start_accumulation_recovers_from_finite_advance_overflow",
            source,
        )
        self.assertIn("visual_start_rejects_non_finite_advance_geometry", source)
        self.assertIn(
            "widget_frames_publish_finite_coordinates_for_extreme_finite_inputs",
            source,
        )

    def test_vertical_ui_placement_origin_reuses_finite_geometry_owner(self):
        source = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/vertical.rs"
        ).read_text(encoding="utf-8")
        tests = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/tests/vertical.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("use crate::text::layout_geometry::finite_sum;", source)
        self.assertIn("finite_vertical_placement_y(frame.y, constraints.inset)", source)
        self.assertIn("finite_sum([frame_y, inset])", source)
        self.assertNotIn("frame.y + constraints.inset", source)
        rich_vertical = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_layout_vertical.rs"
        ).read_text(encoding="utf-8")
        rich_horizontal = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_layout.rs"
        ).read_text(encoding="utf-8")
        self.assertIn(
            "super::vertical::finite_vertical_placement_y(frame.y, constraints.inset)",
            rich_vertical,
        )
        self.assertNotIn("frame.y + constraints.inset", rich_vertical)
        self.assertIn("let line_y = finite_sum([frame.y, rich_line.origin.y])", rich_horizontal)
        self.assertIn(
            "let line_height = finite_sum([rich_line.ascent, rich_line.descent])",
            rich_horizontal,
        )
        self.assertIn("unclipped_measured_height.max(finite_sum([", rich_horizontal)
        self.assertIn("line_frame.height,", rich_horizontal)
        self.assertIn("-frame.y,", rich_horizontal)
        self.assertNotIn("frame.y + rich_line.origin.y", rich_horizontal)
        self.assertNotIn("rich_line.ascent + rich_line.descent", rich_horizontal)
        self.assertNotIn("line_frame.bottom() - frame.y", rich_horizontal)
        self.assertIn(
            "vertical_rl_placement_origin_stays_finite_for_extreme_frame_and_inset",
            tests,
        )

    def test_rich_table_physical_frames_publish_finite_absolute_coordinates(self):
        source = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/axes.rs"
        ).read_text(encoding="utf-8")
        cells = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_table/cell_layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "use crate::text::layout_geometry::{finite_f32_or_geometry, finite_sum};",
            source,
        )
        self.assertIn("let right_exact = f64::from(container.x)", source)
        self.assertIn(
            "let x_candidate = container.right() - block_start - block_extent;",
            source,
        )
        self.assertIn("finite_f32_or_geometry(\n                    x_candidate,", source)
        self.assertIn(
            "right_exact - f64::from(block_start) - f64::from(block_extent)",
            source,
        )
        self.assertIn("finite_sum([container.y, consumed_block])", source)
        self.assertNotIn("UiFrame::new(\n                container.x + inline_start", source)
        self.assertNotIn("container.right() - block_start - block_extent,", source)
        self.assertIn(
            "physical_table_frames_keep_extreme_absolute_coordinates_finite",
            source,
        )
        self.assertIn("translate_frame(&mut line.frame, delta_x, delta_y)", cells)
        self.assertIn(
            "translate_frame(&mut line.placement_frame, delta_x, delta_y)", cells
        )
        self.assertIn("translate_frame(&mut text_box.frame, delta_x, delta_y)", cells)
        self.assertIn("frame.x = finite_sum([frame.x, delta_x])", cells)
        self.assertIn("frame.y = finite_sum([frame.y, delta_y])", cells)
        self.assertNotIn("line.translate(delta_x, delta_y)", cells)
        self.assertNotIn("text_box.frame.x +=", cells)
        self.assertIn(
            "table_cell_translation_keeps_extreme_absolute_coordinates_finite",
            cells,
        )

    def test_vertical_layout_keeps_intermediate_geometry_finite(self):
        source = (ROOT / "zircon_runtime/src/text/layout/vertical_layout.rs").read_text(
            encoding="utf-8"
        )
        geometry = (ROOT / "zircon_runtime/src/text/layout_geometry.rs").read_text(
            encoding="utf-8"
        )
        production = source.split("#[cfg(test)]", 1)[0]

        self.assertIn("f64::from(frame_x)", production)
        self.assertIn("let frame_right_exact = f64::from(frame_x)", production)
        self.assertIn("frame_right_overflowed", production)
        self.assertIn("frames.len() as f64 * f64::from(column_advance)", production)
        self.assertIn("finite_f32_or_geometry", production)
        self.assertIn("value.clamp(-(f32::MAX as f64), f32::MAX as f64)", geometry)
        self.assertIn(
            "vertical_layout_keeps_geometry_finite_when_intermediates_overflow_f32",
            (ROOT / "zircon_runtime/src/text/layout/vertical_layout/single_pass_frame_tests.rs").read_text(
                encoding="utf-8"
            ),
        )

    def test_vertical_shaping_keeps_advance_geometry_finite(self):
        source = (ROOT / "zircon_runtime/src/text/shaping/vertical.rs").read_text(
            encoding="utf-8"
        )
        tests = (ROOT / "zircon_runtime/src/text/shaping/vertical/tests.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("let mut cursor_y = FiniteGeometryAccumulator::default()", source)
        self.assertIn("finite_sum", source)
        self.assertIn("cursor_y.add(metrics.advance)", source)
        self.assertIn("f64::from(glyph.offset_x) + f64::from(metrics.offset_x)", source)
        self.assertIn("finite_f32_or_geometry", source)
        self.assertIn(
            "text_vertical_layout_keeps_extreme_advance_geometry_finite",
            tests,
        )

    def test_vertical_orientation_offset_keeps_extreme_advance_finite(self):
        source = (ROOT / "zircon_runtime/src/text/shaping/vertical/orientation.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("let offset_x_exact =", source)
        self.assertIn("offset_x.is_finite()", source)
        self.assertIn("-f32::MAX * 0.5", source)
        self.assertIn(
            "text_vertical_upright_offset_stays_finite_for_extreme_horizontal_advance",
            source,
        )

    def test_direct_vertical_positioning_reuses_finite_advance_guard(self):
        source = (ROOT / "zircon_runtime/src/text/shaping/vertical/direct.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("FiniteGeometryAccumulator", source)
        self.assertIn("let mut cursor_y = FiniteGeometryAccumulator::default()", source)
        self.assertIn("finite_sum", source)
        self.assertIn(
            "cursor_y.add(metrics.advance)", source
        )
        self.assertIn("f64::from(glyph.offset_x) + f64::from(metrics.offset_x)", source)
        self.assertIn("populated_columns as f64 * f64::from(column_width)", source)

    def test_horizontal_shaping_keeps_accumulated_geometry_finite(self):
        horizontal = (ROOT / "zircon_runtime/src/text/shaping/horizontal/mod.rs").read_text(
            encoding="utf-8"
        )
        geometry = (ROOT / "zircon_runtime/src/text/layout_geometry.rs").read_text(
            encoding="utf-8"
        )
        direct = (ROOT / "zircon_runtime/src/text/shaping/horizontal/direct.rs").read_text(
            encoding="utf-8"
        )
        composition = (
            ROOT / "zircon_runtime/src/text/shaping/horizontal/composition.rs"
        ).read_text(encoding="utf-8")
        hard_lines = (
            ROOT / "zircon_runtime/src/text/shaping/cosmic/hard_lines.rs"
        ).read_text(encoding="utf-8")
        cosmic = (ROOT / "zircon_runtime/src/text/shaping/cosmic.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("pub(crate) fn finite_f32_or_geometry", geometry)
        self.assertIn("pub(crate) fn finite_sum", geometry)
        self.assertIn("FiniteGeometryAccumulator", horizontal)
        self.assertIn("let mut cursor = FiniteGeometryAccumulator::default()", horizontal)
        self.assertIn("pub(super) fn position_glyphs", horizontal)
        self.assertIn("position_glyphs(&mut glyphs)", direct)
        self.assertIn("finite_sum(lines.iter().map(|line| line.line_height))", direct)
        self.assertIn("position_glyphs(&mut line.glyphs)", composition)
        self.assertIn("finite_sum(direct.lines.iter().map(|line| line.line_height))", composition)
        self.assertIn("position_glyphs(&mut line.glyphs)", hard_lines)
        self.assertIn("use crate::text::layout_geometry::finite_sum;", cosmic)
        self.assertIn(
            "text_horizontal_positioning_keeps_extreme_advance_geometry_finite",
            (ROOT / "zircon_runtime/src/text/shaping/horizontal/tests.rs").read_text(
                encoding="utf-8"
            ),
        )

    def test_finite_text_geometry_recovery_has_one_module_owner(self):
        geometry = (ROOT / "zircon_runtime/src/text/layout_geometry.rs").read_text(
            encoding="utf-8"
        )
        hit_test = (ROOT / "zircon_runtime/src/ui/text/hit_test.rs").read_text(
            encoding="utf-8"
        )
        input_geometry = (ROOT / "zircon_runtime/src/ui/text/geometry.rs").read_text(
            encoding="utf-8"
        )
        input_geometry_tests = (
            ROOT / "zircon_runtime/src/ui/text/geometry/tests.rs"
        ).read_text(encoding="utf-8")
        line_box = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/line_box.rs"
        ).read_text(encoding="utf-8")
        ellipsis = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/ellipsis.rs"
        ).read_text(encoding="utf-8")
        rich_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_layout.rs"
        ).read_text(encoding="utf-8")
        rich_vertical = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/rich_layout_vertical.rs"
        ).read_text(encoding="utf-8")
        wrapping = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/wrapping.rs"
        ).read_text(encoding="utf-8")
        wrapping_tests = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/wrapping/tests.rs"
        ).read_text(encoding="utf-8")
        advance_index = (
            ROOT / "zircon_runtime/src/text/layout/advance_index.rs"
        ).read_text(encoding="utf-8")
        advance_index_tests = (
            ROOT / "zircon_runtime/src/text/layout/advance_index/tests.rs"
        ).read_text(encoding="utf-8")
        boundary_correction = (
            ROOT / "zircon_runtime/src/text/layout/line_break/boundary_correction.rs"
        ).read_text(encoding="utf-8")
        plain_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/plain_layout.rs"
        ).read_text(encoding="utf-8")
        physical_line_metrics = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/physical_line_metrics.rs"
        ).read_text(encoding="utf-8")
        paragraph_layout = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/paragraph_layout.rs"
        ).read_text(encoding="utf-8")
        paragraph_layout_tests = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/paragraph_layout/tests.rs"
        ).read_text(encoding="utf-8")
        measure = (ROOT / "zircon_runtime/src/text/layout/measure.rs").read_text(
            encoding="utf-8"
        )
        measure_tests = (ROOT / "zircon_runtime/src/text/layout/measure/tests.rs").read_text(
            encoding="utf-8"
        )
        greedy = (ROOT / "zircon_runtime/src/text/layout/line_break/greedy.rs").read_text(
            encoding="utf-8"
        )
        align = (ROOT / "zircon_runtime/src/text/layout/align.rs").read_text(encoding="utf-8")
        rich_materialize = (
            ROOT / "zircon_runtime/src/text/layout/rich/materialize.rs"
        ).read_text(encoding="utf-8")
        rich_advance_index = (
            ROOT / "zircon_runtime/src/text/layout/rich_advance_index.rs"
        ).read_text(encoding="utf-8")
        consumers = [
            "zircon_runtime/src/text/layout/measure.rs",
            "zircon_runtime/src/text/layout/vertical_layout.rs",
            "zircon_runtime/src/text/shaping/horizontal/mod.rs",
            "zircon_runtime/src/text/shaping/vertical.rs",
            "zircon_runtime/src/text/shaping/vertical/orientation.rs",
        ]

        self.assertIn("pub(crate) fn finite_geometry", geometry)
        self.assertIn("pub(crate) fn finite_f32_or_geometry", geometry)
        self.assertIn("pub(crate) fn finite_sum", geometry)
        self.assertIn("pub(crate) struct FiniteGeometryAccumulator", geometry)
        self.assertIn("if self.recovered || !candidate.is_finite()", geometry)
        self.assertIn(
            "finite_geometry_helpers_preserve_normal_f32_and_clamp_overflow", geometry
        )
        self.assertIn(
            "finite_accumulator_preserves_exact_history_after_overflow_recovery",
            geometry,
        )
        self.assertIn("FiniteGeometryAccumulator", hit_test)
        self.assertIn("finite_sum(advances.iter().copied())", hit_test)
        self.assertIn(
            "finite_geometry(f64::from(point.y) - f64::from(line.frame.y))",
            hit_test,
        )
        self.assertIn(
            "finite_geometry(f64::from(point.x) - f64::from(line.frame.x))",
            hit_test,
        )
        self.assertIn("midpoint.add(width * 0.5)", hit_test)
        self.assertIn("midpoint.add(height * 0.5)", hit_test)
        self.assertNotIn("advances.iter().sum::<f32>()", hit_test)
        self.assertIn("use crate::text::layout_geometry::finite_geometry;", input_geometry)
        self.assertIn("f64::from(line.frame.x)", input_geometry)
        self.assertIn("f64::from(line.frame.y)", input_geometry)
        self.assertIn(
            "source_geometry_keeps_extreme_caret_and_selection_frames_finite",
            input_geometry_tests,
        )
        self.assertGreaterEqual(
            line_box.count("finite_sum(natural_advances.iter().copied())"), 2
        )
        self.assertIn("finite_sum(justified_advances.iter().copied())", line_box)
        self.assertIn("finite_f32_or_geometry(candidate, exact)", line_box)
        self.assertIn("f64::from(frame.x) + f64::from(frame.width)", line_box)
        self.assertIn(
            "aligned_x_keeps_extreme_line_box_coordinates_finite", line_box
        )
        self.assertIn("finite_sum(advances.iter().copied())", ellipsis)
        overflow = (ROOT / "zircon_runtime/src/text/layout/overflow.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("use crate::text::layout_geometry::finite_sum;", overflow)
        self.assertGreaterEqual(overflow.count("finite_sum([retained_width,"), 3)
        self.assertGreaterEqual(overflow.count("finite_sum([width, **advance])"), 2)
        self.assertIn(
            "extreme_finite_advances_do_not_trigger_overflow_ellipsis", ellipsis
        )
        self.assertIn(
            "let measured_width = finite_sum(glyph_advances.iter().copied())",
            rich_layout,
        )
        self.assertGreaterEqual(
            rich_vertical.count("finite_sum(advances.iter().copied())"), 2
        )
        self.assertIn("use crate::text::layout_geometry::finite_sum;", wrapping)
        self.assertIn("fn accumulate_finite_advance", wrapping)
        self.assertGreaterEqual(wrapping.count("finite_sum(["), 3)
        self.assertNotIn("*current_advance +=", wrapping)
        self.assertIn(
            "wrapping_advance_accumulation_stays_finite_at_extreme_glyph_widths",
            wrapping_tests,
        )
        self.assertIn(
            "Vec<FiniteGeometryAccumulator>", advance_index
        )
        self.assertIn("prefix.add(finite_non_negative(metric.advance))", advance_index)
        self.assertIn("after_last_prefix.exact() - first_prefix.exact()", advance_index)
        self.assertIn(
            "measured_advance_index_keeps_extreme_prefix_ranges_finite",
            advance_index_tests,
        )
        self.assertIn(
            "use crate::text::layout_geometry::{finite_sum, FiniteGeometryAccumulator};",
            boundary_correction,
        )
        self.assertIn("let raw_advance = finite_sum(", boundary_correction)
        self.assertIn("let mut corrected = FiniteGeometryAccumulator::default()", boundary_correction)
        self.assertNotIn(".sum::<f32>()", boundary_correction)
        self.assertIn(
            "long_line_correction_keeps_extreme_raw_advance_finite",
            (ROOT / "zircon_runtime/src/text/layout/line_break/boundary_correction/tests.rs").read_text(
                encoding="utf-8"
            ),
        )
        self.assertIn("FiniteGeometryAccumulator", plain_layout)
        self.assertIn("y.add(physical_metrics.line_height)", plain_layout)
        self.assertIn("finite_geometry(total_line_count as f64", plain_layout)
        self.assertNotIn("y += physical_metrics.line_height", plain_layout)
        self.assertIn("FiniteGeometryAccumulator", physical_line_metrics)
        for geometry_symbol in (
            "finite_f32_or_geometry",
            "finite_sum",
            "FiniteGeometryAccumulator",
        ):
            self.assertIn(geometry_symbol, physical_line_metrics)
        self.assertIn("let next_height = finite_sum", physical_line_metrics)
        self.assertIn("finite_sum(metrics.iter().map", physical_line_metrics)
        self.assertIn(
            "total_line_height_keeps_extreme_metric_accumulation_finite",
            physical_line_metrics,
        )
        self.assertIn(
            "maximum_line_height_publishes_only_finite_non_negative_geometry",
            physical_line_metrics,
        )
        self.assertIn("FiniteGeometryAccumulator", paragraph_layout)
        self.assertIn("first_indent.add(finite_non_negative_extent", paragraph_layout)
        self.assertIn("first_indent.add(-finite_non_negative_extent", paragraph_layout)
        self.assertIn("finite_sum([level_indent, first_indent])", paragraph_layout)
        self.assertIn("finite_sum([frame.x, inset])", paragraph_layout)
        self.assertNotIn("frame.x + inset", paragraph_layout)
        self.assertNotIn("first_indent +=", paragraph_layout)
        self.assertNotIn("first_indent -=", paragraph_layout)
        self.assertIn(
            "paragraph_override_sweep_keeps_extreme_nested_indent_finite_and_reversible",
            paragraph_layout_tests,
        )
        self.assertIn(
            "paragraph_logical_inset_keeps_extreme_absolute_origin_finite",
            paragraph_layout_tests,
        )
        self.assertIn("widths[index] = finite_sum([", measure)
        self.assertNotIn("widths[index] +=", measure)
        self.assertIn(
            "measured_grapheme_geometry_keeps_multicluster_overlap_finite",
            measure_tests,
        )
        self.assertIn("use crate::text::layout_geometry::finite_sum;", greedy)
        self.assertIn("finite_sum([current_advance, next_advance])", greedy)
        self.assertIn("fn fit_limit(value: f32)", greedy)
        self.assertIn("value.is_infinite()", greedy)
        self.assertIn("use crate::text::layout_geometry::{finite_geometry, finite_sum};", align)
        self.assertIn("finite_geometry(f64::from(extra) - f64::from(assigned))", align)
        self.assertIn("finite_sum([adjusted[index], delta]).max(0.0)", align)
        self.assertIn("finite_sum([assigned, delta])", align)
        self.assertIn("FiniteGeometryAccumulator", rich_materialize)
        self.assertIn("cursor_y.add(line_height)", rich_materialize)
        self.assertIn("cursor_x.add(advance)", rich_materialize)
        self.assertNotIn("cursor_y += line_height", rich_materialize)
        self.assertNotIn("cursor_x += advance", rich_materialize)
        self.assertIn("FiniteGeometryAccumulator", rich_advance_index)
        self.assertIn("corrected.add(-self.index.advance", rich_advance_index)
        self.assertIn("finite_non_negative(corrected.value())", rich_advance_index)
        self.assertIn(
            "rich_boundary_correction_keeps_signed_extreme_span_delta_finite",
            (ROOT / "zircon_runtime/src/text/layout/rich_advance_index/tests.rs").read_text(
                encoding="utf-8"
            ),
        )
        for relative_path in consumers:
            source = (ROOT / relative_path).read_text(encoding="utf-8")
            self.assertNotIn("fn finite_geometry", source, relative_path)
            self.assertNotIn("fn finite_f32_or_geometry", source, relative_path)
            self.assertNotIn("fn finite_sum", source, relative_path)

    def test_shared_cluster_geometry_keeps_accumulated_advance_finite(self):
        source = (ROOT / "zircon_runtime/src/text/cluster_geometry.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("FiniteGeometryAccumulator", source)
        self.assertIn("let mut advance = FiniteGeometryAccumulator::default()", source)
        self.assertIn("advance.add(glyph_advance)", source)
        self.assertIn(
            "multiglyph_cluster_advance_stays_finite_when_f32_accumulation_overflows",
            source,
        )

    def test_tab_layout_keeps_extreme_geometry_finite(self):
        source = (ROOT / "zircon_runtime/src/text/layout/tab.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("tab_interval_exact_width", source)
        self.assertIn("let mut cursor = FiniteGeometryAccumulator::default()", source)
        self.assertIn("cursor.add(resolved_advance)", source)
        self.assertIn("cursor.exact()", source)
        self.assertIn("finite_sum(advances.iter().copied())", source)
        self.assertIn(
            "streaming_tab_layout_keeps_extreme_finite_geometry_bounded", source
        )

    def test_secure_bidi_direction_projection_publishes_finite_line_origin(self):
        source = (
            ROOT
            / "zircon_runtime/src/ui/text/layout_engine/secure_presentation.rs"
        ).read_text(encoding="utf-8")

        self.assertIn(
            "use crate::text::layout_geometry::finite_f32_or_geometry;", source
        )
        self.assertIn(
            "let frame_right_exact = f64::from(frame.x) + f64::from(frame.width);",
            source,
        )
        self.assertIn("finite_f32_or_geometry(x_candidate, x_exact)", source)
        self.assertIn(
            "secure_direction_projection_keeps_extreme_logical_edges_finite", source
        )

    def test_failure_layout_publishes_only_budgeted_finite_metrics(self):
        source = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine/failure_layout.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("let geometry_budget = provider.geometry_budget();", source)
        self.assertIn("fn admitted_failure_extent(", source)
        self.assertIn("geometry_budget.admit_axis_extent(value).is_ok()", source)
        self.assertIn(
            "failure_layout_rejects_non_finite_and_over_budget_metrics", source
        )
        self.assertNotIn("measured_height: line_height.max(0.0)", source)

    def test_resolved_layout_geometry_is_admitted_before_artifact_publication(self):
        layout_engine = (
            ROOT / "zircon_runtime/src/ui/text/layout_engine.rs"
        ).read_text(encoding="utf-8")
        geometry_admission = (
            ROOT
            / "zircon_runtime/src/ui/text/layout_engine/geometry_admission.rs"
        ).read_text(encoding="utf-8")
        geometry_owner = (
            ROOT / "zircon_runtime/src/text/layout_geometry.rs"
        ).read_text(encoding="utf-8")

        admission_call = "admit_resolved_layout_publication("
        artifact_branch = "if !matches!(\n        style.rich_text_format,"
        self.assertIn(admission_call, layout_engine)
        self.assertLess(
            layout_engine.index(admission_call),
            layout_engine.index(artifact_branch),
        )
        self.assertIn("TextLayoutGeometryOwner::ResolvedLayoutPublication", geometry_admission)
        self.assertIn("ResolvedLayoutPublication", geometry_owner)
        self.assertIn(
            "resolved_layout_publication_rejects_geometry_before_artifact_build",
            geometry_admission,
        )
