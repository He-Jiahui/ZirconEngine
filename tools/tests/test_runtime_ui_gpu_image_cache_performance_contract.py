from pathlib import Path
import unittest

from tools.runtime_ui_image_prepare_cache_pressure import run


ROOT = Path(__file__).resolve().parents[2]
IMAGE = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs"
IMAGE_TESTS = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/image/tests.rs"
RENDER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
PLAN_CACHE = (
    ROOT
    / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs"
)
RECORD = (
    ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs"
)


class RuntimeUiGpuImageCachePerformanceContract(unittest.TestCase):
    def test_gpu_image_bindings_survive_short_idle_gaps_with_a_bound(self) -> None:
        source = IMAGE.read_text(encoding="utf-8")

        self.assertIn("SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_IDLE_EPOCHS", source)
        self.assertIn("SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_MAX_ENTRIES", source)
        self.assertIn("binding_cache_epoch_is_recent", source)
        self.assertIn("retain_prepare_epoch", source)
        self.assertIn("binding_cache_epoch_is_recent", source)
        self.assertIn("while self.bindings.len() > SCREEN_SPACE_UI_IMAGE_BINDING_CACHE_MAX_ENTRIES", source)
        self.assertIn(
            "binding_cache_entry_is_trimmable",
            source,
        )

    def test_empty_frame_reset_does_not_drop_the_gpu_binding_cache(self) -> None:
        source = IMAGE.read_text(encoding="utf-8")
        clear_frame_state = source.split("pub(super) fn clear_frame_state", 1)[1].split(
            "pub(super) fn prepare", 1
        )[0]

        self.assertNotIn("self.image_bindings.clear()", clear_frame_state)
        self.assertIn("self.image_bindings.retain_prepare_epoch", clear_frame_state)

    def test_texture_resolution_cache_is_retained_by_generation_identity(self) -> None:
        source = IMAGE.read_text(encoding="utf-8")
        prepare = source.split("pub(super) fn prepare", 1)[1].split(
            "fn prepare_batch", 1
        )[0]

        for identity in (
            "management_generation: Option<ResourceManagementGenerationIdentity>",
            "readiness_generation: Option<ResourceReadinessGenerationIdentity>",
            "frame_prepare_epoch: Option<u64>",
        ):
            self.assertIn(identity, source)
        for unchanged_identity in (
            "self.management_generation == management_generation",
            "self.readiness_generation == readiness_generation",
            "self.frame_prepare_epoch == frame_prepare_epoch",
        ):
            self.assertIn(unchanged_identity, source)
        self.assertIn("prepared_textures.begin_prepare(", prepare)
        self.assertNotIn("prepared_textures.clear()", prepare)

    def test_rust_regressions_cover_generation_reuse_and_invalidation(self) -> None:
        rust_tests = IMAGE_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "screen_space_ui_image_resolution_cache_reuses_one_management_generation",
            rust_tests,
        )
        self.assertIn(
            "screen_space_ui_image_resolution_cache_invalidates_a_new_management_generation",
            rust_tests,
        )
        self.assertIn(
            "screen_space_ui_image_resolution_cache_does_not_retain_missing_authority",
            rust_tests,
        )

    def test_image_geometry_and_draws_remain_segment_local(self) -> None:
        image_source = IMAGE.read_text(encoding="utf-8")
        render_source = RENDER.read_text(encoding="utf-8")
        plan_cache_source = PLAN_CACHE.read_text(encoding="utf-8")
        record_source = RECORD.read_text(encoding="utf-8")
        prepared_definition = render_source.split(
            "pub(super) struct PreparedScreenSpaceUi {", 1
        )[1].split("}", 1)[0]
        append_payload = render_source.split(
            "fn append_non_render_payload_cloned", 1
        )[1].split("\n    }", 1)[0]

        self.assertNotIn("images:", prepared_definition)
        self.assertNotIn("segment.images.iter().cloned()", append_payload)
        self.assertNotIn("images: combined.images", plan_cache_source)
        self.assertIn("pub(super) fn image_batches(&self)", render_source)
        self.assertIn("ScreenSpaceUiImageSegmentCache", image_source)
        self.assertIn("Option<Weak<PlannedScreenSpaceUi>>", image_source)
        self.assertIn("render_segments: &[Arc<PlannedScreenSpaceUi>]", image_source)
        self.assertIn("&prepared.render_segments", record_source)
        self.assertNotIn("let prepared_images =", record_source)
        self.assertIn("self.image_system.render(&mut pass);", record_source)

    def test_segment_reuse_still_validates_unique_gpu_texture_dependencies(self) -> None:
        image_source = IMAGE.read_text(encoding="utf-8")

        self.assertIn("ScreenSpaceUiImageTextureDependency", image_source)
        self.assertIn("requested: ResourceId", image_source)
        self.assertIn("texture: Option<Arc<GpuTextureResource>>", image_source)
        self.assertIn("refresh_segment_dependencies", image_source)
        self.assertIn("Arc::ptr_eq", image_source)
        self.assertIn("screen_space_ui_image_segment_plan_reused", image_source)

    def test_stable_dependency_skips_the_requested_identity_map_lookup(self) -> None:
        image_source = IMAGE.read_text(encoding="utf-8")
        refresh = image_source.split("fn refresh_segment_dependencies", 1)[1].split(
            "pub(super) fn render", 1
        )[0]

        self.assertIn("resolution_is_current", image_source)
        self.assertIn("resolved_texture_id_for", image_source)
        self.assertIn("texture_resolution_generation_changed", refresh)
        self.assertIn(
            "streamer.ui_texture_ref(dependency.resolved_texture_id)", refresh
        )

    def test_rust_regressions_cover_segment_identity_and_viewport_invalidation(self) -> None:
        rust_tests = IMAGE_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "screen_space_ui_image_segment_cache_reuses_exact_plan_and_viewport",
            rust_tests,
        )
        self.assertIn(
            "screen_space_ui_image_segment_cache_invalidates_plan_or_viewport_change",
            rust_tests,
        )
        self.assertIn(
            "screen_space_ui_image_segment_geometry_deduplicates_texture_dependencies",
            rust_tests,
        )
        self.assertIn(
            "screen_space_ui_image_texture_dependency_uses_exact_gpu_identity",
            rust_tests,
        )

    def test_pressure_model_bounds_registry_scans_to_generation_changes(self) -> None:
        result = run(
            frame_count=4_096,
            image_batches_per_frame=1_024,
            unique_texture_count=64,
            unresolved_unique_texture_count=8,
            registry_record_count=16_384,
            management_generation_count=4,
        )

        self.assertEqual(
            result["per_frame_resolution_cache"]["registry_record_visits"],
            2_415_919_104,
        )
        self.assertEqual(
            result["generation_retained_resolution_cache"][
                "registry_record_visits"
            ],
            2_359_296,
        )
        self.assertEqual(
            result["delta"]["registry_record_visit_reduction_ratio"], 1_024.0
        )
        self.assertEqual(
            result["generation_retained_resolution_cache"][
                "image_texture_cache_lookups"
            ],
            4_194_304,
        )
        self.assertIn(
            "image vertex materialization",
            result["interpretation"]["included"],
        )

    def test_pressure_model_bounds_geometry_work_to_segment_changes(self) -> None:
        result = run(
            frame_count=4_096,
            image_batches_per_frame=1_024,
            unique_texture_count=64,
            unresolved_unique_texture_count=8,
            registry_record_count=16_384,
            management_generation_count=4,
            segment_count=64,
            segment_plan_change_count=32,
        )

        self.assertEqual(
            result["full_frame_image_prepare"]["image_batch_visits"],
            4_194_304,
        )
        self.assertEqual(
            result["segment_retained_image_prepare"]["image_batch_visits"],
            1_536,
        )
        self.assertEqual(
            result["segment_retained_image_prepare"][
                "unique_texture_dependency_checks"
            ],
            262_144,
        )
        self.assertEqual(
            result["segment_retained_image_prepare"][
                "requested_identity_cache_lookups"
            ],
            256,
        )
        self.assertEqual(
            result["segment_retained_image_prepare"]["gpu_texture_map_lookups"],
            262_144,
        )
        self.assertEqual(
            result["delta"]["image_batch_visit_reduction_ratio"],
            2_730.666667,
        )
        self.assertIn(
            "additional per-segment vertex-buffer binds",
            result["interpretation"]["excluded"],
        )


if __name__ == "__main__":
    unittest.main()
