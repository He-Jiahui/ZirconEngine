from pathlib import Path
import unittest

from tools.runtime_ui_text_prepare_cache_pressure import run


ROOT = Path(__file__).resolve().parents[2]
TEXT = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs"
SEGMENT_CACHE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs"
)
SEGMENT_RUN_INDEX = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache/run_index.rs"
)
RENDER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
PLAN_CACHE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs"
)
RECORD = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs"
AUTO_ROUTE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches/auto_route.rs"
)
RESOLVED_BATCHES = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs"
)
TEXT_RENDER_STATE = ROOT / "zircon_runtime/src/text/render_state.rs"
NATIVE_BITMAP_FRAME = ROOT / "zircon_runtime/src/text/native_bitmap_atlas/frame.rs"
SDF_ATLAS = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs"
SDF_ATLAS_PREPARED_TEXTS = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/prepared_texts.rs"
)
SDF_ATLAS_TEXT_KEYS = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs"
)
SDF_CPU_FRAME = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_cpu_frame.rs"
)
TEXT_PREPARE_REPORT = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs"
)
SDF_RENDER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs"
SDF_COMPILED_FRAME = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/compiled_frame.rs"
)
SDF_DECORATIONS = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/decorations.rs"
)
SDF_VERTICES = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs"
)
SDF_MATERIAL = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs"
)


class RuntimeUiTextSegmentCachePerformanceContract(unittest.TestCase):
    def test_pressure_model_separates_immutable_and_readiness_work(self) -> None:
        result = run()

        self.assertEqual(
            result["flat_frame_text_prepare"]["text_batch_visits"], 4_194_304
        )
        self.assertEqual(
            result["flat_frame_text_prepare"]["glyph_instance_projections"],
            100_663_296,
        )
        self.assertEqual(
            result["segment_retained_text_prepare"]["segment_rebuilds"], 288
        )
        self.assertEqual(
            result["segment_retained_text_prepare"]["text_batch_visits"], 4_608
        )
        self.assertEqual(
            result["segment_retained_text_prepare"][
                "active_glyph_dependency_checks"
            ],
            8_388_608,
        )
        self.assertEqual(
            result["delta"]["text_batch_visit_reduction_ratio"], 910.222222
        )
        self.assertIn("async raster worker time", result["interpretation"]["excluded"])

    def test_prepared_frame_does_not_flatten_segment_text_payloads(self) -> None:
        render_source = RENDER.read_text(encoding="utf-8")
        plan_cache_source = PLAN_CACHE.read_text(encoding="utf-8")
        prepared_definition = render_source.split(
            "pub(super) struct PreparedScreenSpaceUi {", 1
        )[1].split("}", 1)[0]
        append_payload = render_source.split(
            "fn append_non_render_payload_cloned", 1
        )[1].split("\n    }", 1)[0]

        self.assertNotIn("auto_texts:", prepared_definition)
        self.assertNotIn("native_texts:", prepared_definition)
        self.assertNotIn("sdf_texts:", prepared_definition)
        self.assertNotIn("segment.auto_texts.iter().cloned()", append_payload)
        self.assertNotIn("segment.native_texts.iter().cloned()", append_payload)
        self.assertNotIn("segment.sdf_texts.iter().cloned()", append_payload)
        self.assertNotIn("auto_texts: combined.auto_texts", plan_cache_source)
        self.assertNotIn("native_texts: combined.native_texts", plan_cache_source)
        self.assertNotIn("sdf_texts: combined.sdf_texts", plan_cache_source)

    def test_text_prepare_consumes_retained_segment_identity(self) -> None:
        text_source = TEXT.read_text(encoding="utf-8")
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        record_source = RECORD.read_text(encoding="utf-8")

        self.assertIn("mod segment_cache;", text_source)
        self.assertIn("ScreenSpaceUiTextSegmentCache", segment_source)
        self.assertIn("frame_segments: Vec<Weak<PlannedScreenSpaceUi>>", segment_source)
        self.assertIn("render_segments: &[Arc<PlannedScreenSpaceUi>]", text_source)
        self.assertIn("&prepared.render_segments", record_source)
        self.assertNotIn("&prepared.auto_texts", record_source)
        self.assertNotIn("&prepared.native_texts", record_source)
        self.assertNotIn("&prepared.sdf_texts", record_source)

    def test_async_atlas_progress_is_not_hidden_by_segment_reuse(self) -> None:
        text_source = TEXT.read_text(encoding="utf-8")
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")

        self.assertIn("begin_sdf_generation_frame", text_source)
        self.assertIn("active_native_glyph_dependency", segment_source)
        self.assertIn("font_revision: FontCollectionRevision", segment_source)
        self.assertIn("segment_plan_reused", segment_source)

    def test_cached_frame_pins_auto_routes_without_stable_frame_text_scans(self) -> None:
        auto_route_source = AUTO_ROUTE.read_text(encoding="utf-8")
        resolved_source = RESOLVED_BATCHES.read_text(encoding="utf-8")
        text_source = TEXT.read_text(encoding="utf-8")
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        record_source = RECORD.read_text(encoding="utf-8")
        prepare_frame_product = text_source.split(
            "let frame_product = self.segment_cache.prepare_frame_product", 1
        )[0]

        self.assertIn("active_routes: HashSet", auto_route_source)
        self.assertIn(
            "self.active_routes.contains(&recency.identity)", auto_route_source
        )
        self.assertIn("auto_router.replace_active_routes", segment_source)
        self.assertNotIn("replace_active_routes", prepare_frame_product)
        self.assertIn("self.auto_raster_router.clear_active_routes();", text_source)
        self.assertIn("self.text_system.clear_frame_state();", record_source)

    def test_local_segment_product_owns_resolution_and_projection_work(self) -> None:
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        build_segment_product = segment_source.split(
            "fn build_segment_product", 1
        )[1].split("fn segment_product_entry_reused", 1)[0]
        prepare_frame_product = segment_source.split(
            "pub(super) fn prepare_frame_product", 1
        )[1].split("pub(super) fn invalidate_frame_product", 1)[0]

        self.assertIn("segment_product_entries:", segment_source)
        self.assertIn("segment_products: Arc<[Arc<", segment_source)
        self.assertIn(
            "resolve_text_batches_after_font_dependencies", build_segment_product
        )
        self.assertIn("native_bitmap_atlas_glyph_runs", build_segment_product)
        self.assertIn("segment_product_entry_reused", prepare_frame_product)
        self.assertIn("current.viewport_size == viewport_size", segment_source)
        self.assertIn("current.font_revision == font_revision", segment_source)
        self.assertIn("compatibility_batch_clone_count", segment_source)
        self.assertIn("compatibility_glyph_run_clone_count", segment_source)

    def test_native_atlas_borrows_segment_glyph_runs_without_frame_flattening(self) -> None:
        text_source = TEXT.read_text(encoding="utf-8")
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        render_state_source = TEXT_RENDER_STATE.read_text(encoding="utf-8")
        atlas_frame_source = NATIVE_BITMAP_FRAME.read_text(encoding="utf-8")
        frame_definition = segment_source.split(
            "pub(super) struct ScreenSpaceUiTextFrameProduct {", 1
        )[1].split("}", 1)[0]

        self.assertNotIn("native_glyph_runs:", frame_definition)
        self.assertIn(
            "flat_map(|product| product.native_glyph_runs())", text_source
        )
        self.assertIn(
            "generated_glyph_run_projection.glyph_runs.iter()", text_source
        )
        self.assertIn("compatibility_glyph_run_clone_count = 0", segment_source)
        self.assertIn("glyph_runs: GlyphRuns", render_state_source)
        self.assertIn("glyph_runs: GlyphRuns", atlas_frame_source)
        self.assertIn(
            "GlyphRuns: Clone + IntoIterator<Item = &'a NativeBitmapAtlasGlyphRun>",
            render_state_source,
        )
        self.assertIn(
            "GlyphRuns: Clone + IntoIterator<Item = &'a NativeBitmapAtlasGlyphRun>",
            atlas_frame_source,
        )

    def test_native_atlas_queries_readiness_once_per_unique_raster_key(self) -> None:
        atlas_frame_source = NATIVE_BITMAP_FRAME.read_text(encoding="utf-8")
        readiness_pass, ordered_geometry_pass = atlas_frame_source.split(
            "for glyph_run in glyph_runs {", 1
        )

        self.assertIn("for glyph_run in glyph_runs.clone()", readiness_pass)
        self.assertIn("readiness_by_raster_key", readiness_pass)
        self.assertIn("GlyphRasterKey,", atlas_frame_source)
        self.assertIn("source_cache.cached_image", readiness_pass)
        self.assertIn("source_cache.approximate_cached_image", readiness_pass)
        self.assertNotIn("source_cache.cached_image", ordered_geometry_pass)
        self.assertNotIn("source_cache.approximate_cached_image", ordered_geometry_pass)
        self.assertIn(
            '"ui_text.native_raster_plan.glyph_instance_visit_count"',
            readiness_pass,
        )
        self.assertIn(
            '"ui_text.native_raster_plan.unique_raster_dependency_count"',
            readiness_pass,
        )

    def test_sdf_stable_frame_uses_typed_generation_without_freezing_readiness(self) -> None:
        text_source = TEXT.read_text(encoding="utf-8")
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        atlas_source = SDF_ATLAS.read_text(encoding="utf-8")
        cpu_source = SDF_CPU_FRAME.read_text(encoding="utf-8")
        render_source = SDF_RENDER.read_text(encoding="utf-8")
        compiled_source = SDF_COMPILED_FRAME.read_text(encoding="utf-8")

        self.assertIn("ScreenSpaceUiTextFrameProductGeneration", text_source)
        self.assertIn("frame_product_generation_counter", segment_source)
        self.assertIn("prepare_retained", atlas_source)
        self.assertIn("prepare_retained", cpu_source)
        self.assertIn("prepare_retained", render_source)
        self.assertIn("retained_frame_generation", compiled_source)
        self.assertIn("self.viewport_size == viewport_size", compiled_source)
        self.assertIn("atlas_upload.mode == SdfAtlasUploadMode::None", render_source)
        self.assertIn("record_generation_failures", text_source)
        self.assertIn("needs_sdf_cpu_rebuild", text_source)
        self.assertIn(
            "if let Some(resolved_texts) = fallback_resolved_texts.as_ref()",
            text_source,
        )
        self.assertIn("prepare_retained_segments", text_source)

    def test_frame_product_publishes_segment_local_global_run_spans(self) -> None:
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        run_index_source = SEGMENT_RUN_INDEX.read_text(encoding="utf-8")

        self.assertIn("mod run_index;", segment_source)
        self.assertIn("ScreenSpaceUiTextFrameRunIndex", segment_source)
        self.assertIn("run_index: ScreenSpaceUiTextFrameRunIndex", segment_source)
        self.assertIn("from_segment_run_counts", segment_source)
        self.assertIn("resolved_texts.native_texts().len()", segment_source)
        self.assertIn("resolved_texts.sdf_texts().len()", segment_source)
        self.assertIn("native_run_base", run_index_source)
        self.assertIn("sdf_run_base", run_index_source)
        self.assertIn("run_index_segment_count", segment_source)
        self.assertIn("sdf_run_index_run_count", segment_source)
        text_source = TEXT.read_text(encoding="utf-8")
        self.assertIn("frame_product.sdf_run_count()", text_source)
        self.assertIn("frame_product.native_run_count()", text_source)
        self.assertIn("self.sdf_atlas.plan().runs.len()", text_source)
        self.assertIn("sdf_cpu_runs.len()", text_source)
        self.assertIn("native_decoration_metrics.len()", text_source)

    def test_sdf_atlas_and_cpu_prepare_consume_borrowed_segment_text_streams(self) -> None:
        text_source = TEXT.read_text(encoding="utf-8")
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        atlas_source = SDF_ATLAS.read_text(encoding="utf-8")
        prepared_source = SDF_ATLAS_PREPARED_TEXTS.read_text(encoding="utf-8")
        key_source = SDF_ATLAS_TEXT_KEYS.read_text(encoding="utf-8")
        cpu_source = SDF_CPU_FRAME.read_text(encoding="utf-8")
        render_state_source = TEXT_RENDER_STATE.read_text(encoding="utf-8")

        self.assertIn("pub(super) fn sdf_text_segments", segment_source)
        self.assertIn("pub(super) fn native_text_segments", segment_source)
        self.assertIn("prepare_retained_segments", atlas_source)
        self.assertIn("prepare_retained_segments", cpu_source)
        self.assertIn("frame_product.sdf_text_segments()", text_source)
        self.assertIn("frame_product.native_text_segments()", text_source)
        self.assertIn("fn matches_iter", prepared_source)
        self.assertIn("fn replace_iter", prepared_source)
        self.assertIn("collect_sdf_atlas_text_keys_iter", key_source)
        self.assertIn("prepare_sdf_runs_cpu_iter_into", render_state_source)
        self.assertIn("prepare_sdf_decoration_metrics_iter_into", render_state_source)
        self.assertNotIn("ScreenSpaceUiTextSegmentProduct", atlas_source)

    def test_sdf_renderer_consumes_borrowed_segment_streams_on_the_product_path(self) -> None:
        text_source = TEXT.read_text(encoding="utf-8")
        render_source = SDF_RENDER.read_text(encoding="utf-8")
        compiled_source = SDF_COMPILED_FRAME.read_text(encoding="utf-8")
        decorations_source = SDF_DECORATIONS.read_text(encoding="utf-8")
        vertices_source = SDF_VERTICES.read_text(encoding="utf-8")
        material_source = SDF_MATERIAL.read_text(encoding="utf-8")

        self.assertIn("prepare_retained_segments", render_source)
        self.assertIn("frame_product.sdf_text_segments()", text_source)
        self.assertIn("frame_product.native_text_segments()", text_source)
        self.assertIn("frame_product.sdf_run_count()", text_source)
        self.assertIn("matches_iter", compiled_source)
        self.assertIn("replace_iter", compiled_source)
        self.assertIn("build_text_decoration_vertices_iter", decorations_source)
        self.assertIn("build_sdf_vertex_plan_iter", vertices_source)
        self.assertIn("rebuild_iter", material_source)
        self.assertNotIn("Vec<&ScreenSpaceUiTextBatch>", render_source)
        self.assertNotIn("texts.clone().count()", render_source)

    def test_frame_product_materializes_flat_texts_only_for_fallback(self) -> None:
        text_source = TEXT.read_text(encoding="utf-8")
        segment_source = SEGMENT_CACHE.read_text(encoding="utf-8")
        report_source = TEXT_PREPARE_REPORT.read_text(encoding="utf-8")
        frame_definition = segment_source.split(
            "pub(super) struct ScreenSpaceUiTextFrameProduct {", 1
        )[1].split("}", 1)[0]
        fallback_branch = text_source.split("if needs_sdf_fallback {", 1)[1].split(
            "} else {", 1
        )[0]

        self.assertNotIn("resolved_texts:", frame_definition)
        self.assertIn("resolved_report:", frame_definition)
        self.assertIn("materialize_resolved_texts", segment_source)
        self.assertIn("frame_product.materialize_resolved_texts()", fallback_branch)
        self.assertIn("ScreenSpaceUiResolvedTextReport", report_source)
        self.assertIn("frame_product.resolved_report()", text_source)
        self.assertIn("self.native.prepare_retained", text_source)
        self.assertNotIn("let base_resolved_texts", text_source)


if __name__ == "__main__":
    unittest.main()
