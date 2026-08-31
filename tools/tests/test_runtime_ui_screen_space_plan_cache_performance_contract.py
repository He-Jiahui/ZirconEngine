import unittest
from pathlib import Path

from tools.runtime_ui_screen_space_plan_cache_pressure import run


ROOT = Path(__file__).resolve().parents[2]
RENDER = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs"
PLAN_CACHE = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs"
)
BACKGROUND = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/background.rs"
)
RECORD = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs"
RENDERER = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs"
)
CONSTRUCT = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/construct.rs"
RUST_TESTS = ROOT / "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests.rs"
RUST_PLAN_CACHE_TESTS = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/plan_cache.rs"
)
RUST_BACKGROUND_TESTS = ROOT / (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/background.rs"
)
PROFILE_MANIFEST = ROOT / "tools/profile-capture-manifest.ps1"


class RuntimeUiScreenSpacePlanCachePerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.render = RENDER.read_text(encoding="utf-8")
        cls.plan_cache = PLAN_CACHE.read_text(encoding="utf-8")
        cls.background = BACKGROUND.read_text(encoding="utf-8")
        cls.record = RECORD.read_text(encoding="utf-8")
        cls.renderer = RENDERER.read_text(encoding="utf-8")
        cls.construct = CONSTRUCT.read_text(encoding="utf-8")
        cls.rust_tests = RUST_TESTS.read_text(encoding="utf-8")
        cls.rust_plan_cache_tests = RUST_PLAN_CACHE_TESTS.read_text(encoding="utf-8")
        cls.rust_background_tests = RUST_BACKGROUND_TESTS.read_text(encoding="utf-8")

    def test_renderer_retains_one_complete_planned_ui_authority(self) -> None:
        self.assertIn("struct ScreenSpaceUiPlanCache", self.plan_cache)
        self.assertIn(
            "cached_plan: Option<Arc<PreparedScreenSpaceUi>>", self.plan_cache
        )
        self.assertIn("plan_cache: ScreenSpaceUiPlanCache", self.renderer)
        self.assertIn("plan_cache: Default::default()", self.construct)
        self.assertIn(
            "render_segments: Arc<[Arc<PlannedScreenSpaceUi>]>", self.render
        )

    def test_prepared_plan_retains_render_segments_without_flat_payload_clones(self) -> None:
        self.assertIn("append_non_render_payload_cloned", self.render)
        self.assertNotIn("combined.append_cloned(segment)", self.render)
        self.assertIn(
            "vertex_segments: Vec<ScreenSpaceUiVertexSegmentBuffer>", self.renderer
        )
        self.assertIn("screen_space_ui_vertex_segment_plan_reused(", self.record)
        self.assertIn("for (segment, vertex_segment)", self.record)
        self.assertNotIn("let vertices = prepared.vertices.as_slice();", self.record)

    def test_cache_key_uses_submission_identity_and_all_planner_inputs(self) -> None:
        self.assertIn(
            "Arc::ptr_eq(&self.submission, submission)", self.plan_cache
        )
        self.assertIn("viewport_size: UVec2", self.plan_cache)
        self.assertIn("framebuffer_background_bits", self.plan_cache)
        self.assertIn("font_revision: FontCollectionRevision", self.plan_cache)
        self.assertIn("self.font_revision == font_revision", self.plan_cache)

    def test_changed_submission_reuses_prefix_dependent_segment_plans(self) -> None:
        self.assertIn("segment_entries: Vec<ScreenSpaceUiSegmentPlanCacheEntry>", self.plan_cache)
        self.assertIn("incoming_background_generation", self.plan_cache)
        self.assertIn("outgoing_background_generation", self.plan_cache)
        self.assertIn("background_effects", self.plan_cache)
        self.assertIn("entry.matches(", self.plan_cache)
        self.assertIn("command_segment,", self.plan_cache)
        self.assertIn("entry.background_effects.as_ref() == background_effects.as_ref()", self.plan_cache)
        self.assertIn("ui.screen_space_ui_plan.segment_cache_hit_count", self.plan_cache)
        self.assertIn("ui.screen_space_ui_plan.segment_command_visit_count", self.plan_cache)
        self.assertIn("ui.screen_space_ui_plan.composition_payload_clone_count", self.plan_cache)
        self.assertIn(
            "screen_space_ui_plan_cache_reuses_unchanged_segments_across_submission_wrappers",
            self.rust_plan_cache_tests,
        )
        self.assertIn(
            "screen_space_ui_plan_cache_preserves_suffix_when_changed_segment_has_same_background_effects",
            self.rust_plan_cache_tests,
        )
        self.assertIn(
            "screen_space_ui_plan_cache_invalidates_suffix_when_background_effects_change",
            self.rust_plan_cache_tests,
        )
        self.assertIn(
            "screen_space_ui_plan_cache_invalidates_route_and_projection_domains",
            self.rust_plan_cache_tests,
        )
        self.assertIn(
            "screen_space_ui_segment_plan_composition_retains_cached_prefix_and_suffix",
            self.rust_plan_cache_tests,
        )

    def test_changed_surface_reuses_immutable_command_leaf_plans(self) -> None:
        frame_extract = (
            ROOT / "zircon_runtime_interface/src/ui/surface/render/frame_extract.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("pub fn command_segments(", frame_extract)
        self.assertIn("pub fn segments(", frame_extract)
        self.assertIn(
            "impl ExactSizeIterator<Item = &Arc<[UiRenderCommand]>>",
            frame_extract,
        )
        self.assertIn("UiRenderFrameCommandSegmentsIter::new", frame_extract)
        self.assertIn(
            "for command_segment in segment.extract().command_segments()",
            self.plan_cache,
        )
        self.assertIn("commands: Arc<[UiRenderCommand]>", self.plan_cache)
        self.assertIn("Arc::ptr_eq(&self.commands, command_segment)", self.plan_cache)
        self.assertIn(
            "screen_space_ui_plan_cache_reuses_unchanged_command_segments_within_surface",
            self.rust_plan_cache_tests,
        )

    def test_exact_and_all_leaf_hits_publish_the_same_leaf_observability(self) -> None:
        exact_hit = self.plan_cache.split(
            "if self.key.as_ref().is_some_and(|key| {", 1
        )[1].split("let planner_inputs_match", 1)[0]
        all_leaf_hit = self.plan_cache.split("if all_segments_reused {", 1)[1].split(
            "let (cached_plan, composition_payload_clone_count)", 1
        )[0]

        for branch in (exact_hit, all_leaf_hit):
            self.assertIn(
                "record_screen_space_ui_plan_full_reuse_profile(", branch
            )

        helper = self.plan_cache.split(
            "fn record_screen_space_ui_plan_full_reuse_profile", 1
        )[1].split("impl ScreenSpaceUiPlanCacheKey", 1)[0]
        for counter in (
            '"ui.screen_space_ui_plan.command_leaf_cache_hit_count"',
            '"ui.screen_space_ui_plan.command_leaf_count"',
            '"ui.screen_space_ui_plan.command_leaf_rebuild_count"',
            '"ui.screen_space_ui_plan.segment_command_visit_count"',
        ):
            self.assertIn(
                counter,
                helper,
            )

    def test_pressure_model_is_bound_to_current_zircon_and_reference_sources(self) -> None:
        result = run()
        binding = result["source_binding"]

        self.assertEqual(len(binding["implementation"]), 5)
        self.assertEqual(len(binding["primary_reference"]), 1)
        self.assertEqual(len(binding["secondary_reference"]), 1)
        for source in (
            *binding["implementation"],
            *binding["primary_reference"],
            *binding["secondary_reference"],
        ):
            self.assertGreater(source["bytes"], 0)
            self.assertEqual(len(source["sha256"]), 64)

    def test_record_path_uses_the_renderer_owned_cache(self) -> None:
        self.assertIn("self.plan_cache.prepare_with_font_revision(", self.record)
        self.assertNotIn("prepare_screen_space_ui(frame", self.record)
        self.assertIn("screen_space_ui_plan.cache_hit_count", self.plan_cache)
        self.assertIn("screen_space_ui_plan.build_count", self.plan_cache)
        self.assertIn("screen_space_ui_plan.command_visit_count", self.plan_cache)

    def test_background_lookup_walks_one_reverse_paint_effect_log(self) -> None:
        self.assertIn("effects: Vec<ScreenSpaceUiBackgroundEffect>", self.background)
        self.assertIn("for effect in self.effects.iter().rev()", self.background)
        self.assertNotIn("candidates: Vec<", self.background)
        self.assertNotIn("blockers: Vec<", self.background)
        self.assertIn("ui.screen_space_ui_background.query_count", self.render)
        self.assertIn("ui.screen_space_ui_background.effect_visit_count", self.render)
        self.assertIn("ui.screen_space_ui_background.max_effect_visit_count", self.render)
        self.assertIn("record_counter_batch(", self.render)
        self.assertIn(
            "screen_space_ui_background_lookup_stops_at_newest_relevant_effect",
            self.rust_background_tests,
        )

    def test_stable_prepared_plan_skips_vertex_payload_hashing(self) -> None:
        self.assertIn(
            "vertex_buffer_plan: Option<Weak<PreparedScreenSpaceUi>>", self.renderer
        )
        self.assertIn("vertex_buffer_plan: None", self.construct)
        self.assertIn("screen_space_ui_vertex_plan_reused(", self.record)
        reuse_check = self.record.index("screen_space_ui_vertex_plan_reused(")
        payload_hash = self.record.index("blake3::hash(vertex_bytes)")
        self.assertLess(reuse_check, payload_hash)
        self.assertIn(
            "self.vertex_buffer_plan = Some(Arc::downgrade(prepared))", self.record
        )
        self.assertIn("current.as_ptr(), Arc::as_ptr(next)", self.record)
        self.assertIn("self.vertex_buffer_plan = None", self.record)
        self.assertIn(
            "screen_space_ui_vertex_plan_reuse_requires_exact_plan_identity",
            self.rust_plan_cache_tests,
        )

    def test_text_prepare_report_has_one_retained_authority(self) -> None:
        self.assertNotIn("last_text_prepare_report:", self.renderer)
        self.assertIn("text_prepare_report_valid: bool", self.renderer)
        self.assertIn("text_prepare_report_valid: false", self.construct)
        self.assertNotIn(
            "self.last_text_prepare_report = self.text_system.prepare_report()",
            self.record,
        )
        self.assertIn("self.text_prepare_report_valid = false", self.record)
        self.assertIn("self.text_prepare_report_valid = true", self.record)
        self.assertIn(
            "if self.text_prepare_report_valid",
            self.record,
        )
        self.assertIn("self.text_system.prepare_report()", self.record)

    def test_rust_regressions_cover_reuse_and_each_invalidation_domain(self) -> None:
        self.assertIn(
            "screen_space_ui_plan_cache_reuses_stable_submission_identity",
            self.rust_plan_cache_tests,
        )
        self.assertIn(
            "screen_space_ui_plan_cache_invalidates_each_planner_input",
            self.rust_plan_cache_tests,
        )

    def test_pressure_model_counts_only_generation_builds(self) -> None:
        result = run(
            frame_count=4_096,
            plan_build_count=64,
            commands_per_submission=32_768,
            text_batches_per_submission=4_096,
            modeled_text_bytes_per_batch=48,
            vertices_per_plan=196_608,
            modeled_vertex_bytes=24,
        )

        self.assertEqual(
            result["retired_per_frame_planning"]["command_visits"],
            134_217_728,
        )
        self.assertEqual(
            result["retained_generation_planning"]["command_visits"],
            2_097_152,
        )
        self.assertEqual(
            result["retained_generation_planning"]["cache_hit_count"],
            4_032,
        )
        self.assertEqual(
            result["retained_segment_planning_non_background_change"][
                "command_visits"
            ],
            65_024,
        )
        self.assertEqual(
            result["retained_segment_planning_non_background_change"][
                "segment_cache_hit_count"
            ],
            3_969,
        )
        self.assertEqual(
            result["retained_segment_planning_background_suffix_change"][
                "command_visits"
            ],
            2_097_152,
        )
        self.assertEqual(result["inputs"]["background_dependent_suffix_count"], 63)
        self.assertEqual(
            result["retained_segment_planning_non_background_change"][
                "composition_payload_clone_count"
            ],
            0,
        )
        self.assertEqual(
            result["retained_segment_planning_non_background_change"][
                "flat_composition_payload_clone_count"
            ],
            15_269_888,
        )
        self.assertEqual(
            result["retained_segment_planning_non_background_change"][
                "retained_render_segment_reference_count"
            ],
            4_096,
        )
        self.assertEqual(
            result["retained_segment_planning_non_background_change"][
                "vertex_hash_pass_count"
            ],
            127,
        )
        self.assertEqual(
            result["retained_segment_planning_non_background_change"][
                "modeled_vertex_hash_input_bytes"
            ],
            9_363_456,
        )
        self.assertEqual(result["inputs"]["draws_per_plan"], 32_768)
        self.assertEqual(result["inputs"]["post_text_draws_per_plan"], 4_096)
        self.assertEqual(result["inputs"]["image_batches_per_plan"], 1_024)
        self.assertEqual(
            result["delta"][
                "segment_non_background_avoided_command_visits_vs_whole_plan_cache"
            ],
            2_032_128,
        )
        self.assertAlmostEqual(
            result["delta"][
                "segment_non_background_command_visit_reduction_ratio_vs_whole_plan_cache"
            ],
            32.25196850393701,
        )
        self.assertEqual(
            result["delta"]["segment_consumer_avoided_composition_payload_clones"],
            15_269_888,
        )
        self.assertEqual(
            result["retained_command_leaf_planning_non_background_change"][
                "command_visits"
            ],
            36_800,
        )
        self.assertEqual(
            result["retained_command_leaf_planning_non_background_change"][
                "command_leaf_cache_hit_count"
            ],
            32_193,
        )
        self.assertEqual(
            result["retained_command_leaf_planning_non_background_change"][
                "modeled_vertex_hash_input_bytes"
            ],
            5_299_200,
        )
        self.assertGreater(
            result["delta"][
                "command_leaf_non_background_command_visit_reduction_ratio_vs_surface_segment"
            ],
            1.7,
        )
        self.assertEqual(
            result["delta"][
                "segment_non_background_avoided_vertex_hash_input_bytes_vs_whole_plan_cache"
            ],
            292_626_432,
        )
        self.assertAlmostEqual(
            result["delta"][
                "segment_non_background_vertex_hash_input_reduction_ratio_vs_whole_plan_cache"
            ],
            32.25196850393701,
        )
        self.assertEqual(result["delta"]["avoided_command_visits"], 132_120_576)
        self.assertEqual(result["delta"]["command_visit_reduction_ratio"], 64.0)
        self.assertEqual(
            result["delta"]["avoided_modeled_planner_text_payload_clone_bytes"],
            792_723_456,
        )
        self.assertIn(
            "per-frame downstream text-system batch clones",
            result["interpretation"]["excluded"],
        )
        self.assertEqual(
            result["delta"]["avoided_modeled_vertex_hash_input_bytes"],
            19_025_362_944,
        )
        self.assertEqual(result["delta"]["avoided_vertex_hash_passes"], 4_032)
        self.assertEqual(result["delta"]["vertex_hash_input_reduction_ratio"], 64.0)
        self.assertEqual(
            result["retired_per_frame_planning"][
                "renderer_text_prepare_report_snapshot_clone_count"
            ],
            4_096,
        )
        self.assertEqual(
            result["retained_generation_planning"][
                "renderer_text_prepare_report_snapshot_clone_count"
            ],
            0,
        )
        self.assertEqual(
            result["delta"][
                "avoided_renderer_text_prepare_report_snapshot_clones"
            ],
            4_096,
        )
        self.assertIn(
            "consumer-requested text prepare report clones",
            result["interpretation"]["excluded"],
        )
        self.assertIn(
            "actual GPU writes skipped by equal segment payload hashes",
            result["interpretation"]["excluded"],
        )

    def test_profile_manifest_binds_the_plan_authority(self) -> None:
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")
        for relative_path in (
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/background.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/resolved_layout.rs",
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/screen_space_ui_renderer.rs",
        ):
            self.assertIn(f'"{relative_path}"', manifest)


if __name__ == "__main__":
    unittest.main()
