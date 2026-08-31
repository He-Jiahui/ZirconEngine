from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RHI_SURFACE = ROOT / "zircon_runtime/crates/zr_rhi/src/ui_surface.rs"
WGPU_BATCHING = ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs"
WGPU_SURFACE = ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs"
WGPU_PRESENTATION = (
    ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs"
)
WGPU_RENDER_PASS = (
    ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/render_pass.rs"
)
WGPU_GEOMETRY = ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/geometry.rs"
WGPU_PIPELINE = ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/pipeline.rs"
WGPU_RETAINED_CACHE = (
    ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/retained_cache.rs"
)
WGPU_IMAGE_CACHE = (
    ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/image_cache.rs"
)
WGPU_TEXT = ROOT / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/text.rs"
WGPU_MATERIAL_SHADER = (
    ROOT
    / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shaders/ui_material.wgsl"
)
BOUNDS_INDEX = (
    ROOT
    / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/bounds_index.rs"
)
SCALE_TESTS = (
    ROOT
    / "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching/tests/scale_and_cache.rs"
)


class RuntimeUiRhiDamageCandidateIndexPerformanceContractTests(unittest.TestCase):
    def test_visibility_counter_records_real_command_visits(self) -> None:
        source = RHI_SURFACE.read_text(encoding="utf-8")

        self.assertIn("stats.record_command_visit();", source)
        self.assertIn("command_visibility_scan_count.saturating_add(1)", source)
        self.assertNotRegex(source, r"command_visibility_scan_count:\s*1")

    def test_versioned_damage_cache_uses_retained_candidates(self) -> None:
        source = WGPU_BATCHING.read_text(encoding="utf-8")

        self.assertIn("command_damage_index: BoundsIndex", source)
        self.assertIn("query_sorted_into(damage, candidate_storage)", source)
        self.assertNotIn("Some(draw_list.stats())", source)

    def test_bounds_index_keeps_linear_pooled_storage_without_query_scan(self) -> None:
        source = BOUNDS_INDEX.read_text(encoding="utf-8")

        self.assertIn("crossing_by_start: Vec<usize>", source)
        self.assertIn("crossing_by_end: Vec<usize>", source)
        self.assertIn("fn query_sorted_into", source)
        self.assertIn("candidates.clear()", source)
        self.assertNotIn("fn query_sorted(", source)
        self.assertIn("Vec::with_capacity(bounds.len())", source)
        self.assertNotIn(".iter().find", source)
        self.assertNotIn("for bounds in &self.bounds", source)

    def test_compiled_plan_indexes_draw_ops_and_native_record_uses_candidates(self) -> None:
        batching = WGPU_BATCHING.read_text(encoding="utf-8")
        presentation = WGPU_PRESENTATION.read_text(encoding="utf-8")
        render_pass = WGPU_RENDER_PASS.read_text(encoding="utf-8")

        self.assertIn("draw_op_damage_index: BoundsIndex", batching)
        self.assertIn("fn draw_ops_intersecting", batching)
        self.assertIn("DrawOpSelection", batching)
        self.assertNotIn("Vec<&DrawOp>", batching)
        self.assertIn("record_draw_plan_to_view", presentation)
        self.assertNotIn("record_draw_ops_to_view", presentation)
        self.assertIn(
            "draw_plan.draw_ops_intersecting(scissor_bounds, damage_draw_op_candidates)",
            " ".join(render_pass.split()),
        )
        self.assertIn("damage_draw_op_candidates: Vec<usize>", WGPU_SURFACE.read_text(encoding="utf-8"))
        self.assertIn("command_damage_candidates: Vec<usize>", batching)

    def test_draw_op_scale_regression_requires_one_of_ten_thousand(self) -> None:
        source = SCALE_TESTS.read_text(encoding="utf-8")

        self.assertIn("compiled_plan_damage_query_prunes_ten_thousand_draw_ops", source)
        self.assertIn("compiled_plan_damage_query_preserves_mixed_op_order", source)
        self.assertIn("compiled_plan_damage_query_reuses_candidate_storage", source)
        self.assertIn("compiled_command_damage_query_reuses_candidate_storage", source)
        self.assertIn("assert_eq!(plan.ops.len(), item_count as usize);", source)
        self.assertIn("assert_eq!(candidates.len(), 1);", source)

    def test_scale_regression_requires_one_candidate_from_ten_thousand(self) -> None:
        source = SCALE_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "versioned_damage_cache_queries_only_spatial_command_candidates", source
        )
        self.assertIn("let item_count = 10_000_u32;", source)
        self.assertIn("cached_stats.command_visibility_scan_count, 1", source)
        self.assertIn(
            "versioned_damage_cache_indexes_clipped_sparse_source_rows", source
        )

    def test_first_damage_build_reports_both_visibility_projections(self) -> None:
        source = SCALE_TESTS.read_text(encoding="utf-8")
        geometry = WGPU_GEOMETRY.read_text(encoding="utf-8")

        self.assertIn("first_stats.command_visibility_scan_count", source)
        self.assertIn("u64::from(item_count).saturating_mul(2)", source)
        self.assertIn(
            ".saturating_add(damage_stats.command_visibility_scan_count)", geometry
        )
        self.assertIn(
            "damage_stats.command_visibility_scan_count = total_visibility_scans",
            geometry,
        )

    def test_damage_patch_clears_the_scissor_before_candidate_replay(self) -> None:
        pipeline = WGPU_PIPELINE.read_text(encoding="utf-8")
        render_pass = WGPU_RENDER_PASS.read_text(encoding="utf-8")
        shader = WGPU_MATERIAL_SHADER.read_text(encoding="utf-8")
        retained_cache = WGPU_RETAINED_CACHE.read_text(encoding="utf-8")

        self.assertIn("create_damage_clear_pipeline", pipeline)
        self.assertIn("damage_clear_vs_main", shader)
        self.assertIn("damage_clear_fs_main", shader)
        self.assertIn("record_damage_clear", render_pass)
        self.assertEqual(render_pass.count("record_damage_clear(&mut pass"), 3)
        self.assertIn("pass.draw(0..3, 0..1);", render_pass)
        self.assertIn("stats.record_clear_draw();", render_pass)
        self.assertIn("damage_clear_draw_count", render_pass)
        self.assertIn("content_draw_calls", render_pass)
        self.assertNotIn(
            "draw_ops.is_empty() && !draw_plan.ops.is_empty()", render_pass
        )
        self.assertIn(
            "retained_cache_empty_damage_patch_clears_removed_pixels",
            retained_cache,
        )

    def test_complete_image_generation_skips_resource_prepare_and_confirmation_scans(
        self,
    ) -> None:
        image_cache = WGPU_IMAGE_CACHE.read_text(encoding="utf-8")
        presentation = WGPU_PRESENTATION.read_text(encoding="utf-8")

        self.assertIn("prepared_generation: Option<u64>", image_cache)
        self.assertIn("prepared_source_count: u64", image_cache)
        self.assertIn("reusable_image_prepare_generation", image_cache)
        self.assertIn("committed_image_prepare_generation", image_cache)
        self.assertRegex(image_cache, r"image_resources\s*\.is_empty\(\)")
        self.assertIn("had_staged_resources", image_cache)
        self.assertIn("external_images_present", image_cache)
        self.assertIn("!external_images_present", image_cache)
        self.assertIn("external_images.is_some()", image_cache)
        self.assertIn("all_sources_resident", image_cache)
        self.assertLess(
            image_cache.index("if self.prepared_generation == Some(generation)"),
            image_cache.index("'source: for ("),
        )
        provider_confirmation = presentation.split(
            "if let Some(provider) = self.external_images.as_deref()", 1
        )[1]
        self.assertLess(
            provider_confirmation.index(
                "for source_index in self.image_cache.resolved_external_source_indices()"
            ),
            provider_confirmation.index("provider.confirm_resident"),
        )

    def test_text_generation_cache_hit_reads_cached_renderer_count(self) -> None:
        source = WGPU_TEXT.read_text(encoding="utf-8")
        cache_hit = source.split("if self.batch_cache_key == Some(cache_key)", 1)[1]
        cache_hit = cache_hit.split("self.viewport.update", 1)[0]

        self.assertIn("prepared_renderer_count: u64", source)
        self.assertIn("self.prepared_renderer_count", cache_hit)
        self.assertNotIn("self.batches.iter()", " ".join(cache_hit.split()))

    def test_external_image_confirmation_reuses_prepare_acceptance_indices(self) -> None:
        image_cache = WGPU_IMAGE_CACHE.read_text(encoding="utf-8")
        presentation = WGPU_PRESENTATION.read_text(encoding="utf-8")
        confirmation = presentation.split(
            "if let Some(provider) = self.external_images.as_deref()", 1
        )[1]
        confirmation = confirmation.split("let mut batch_stats", 1)[0]

        self.assertIn("resolved_external_source_indices: Vec<usize>", image_cache)
        self.assertIn("self.resolved_external_source_indices.clear()", image_cache)
        self.assertIn("self.resolved_external_source_indices.push(source_index)", image_cache)
        self.assertIn("resolved_external_source_indices()", confirmation)
        self.assertNotIn("provider.resolve", confirmation)
        self.assertNotIn("for source in &draw_plan.image_upload_sources", confirmation)


if __name__ == "__main__":
    unittest.main()
