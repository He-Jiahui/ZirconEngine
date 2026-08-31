from __future__ import annotations

import importlib.util
import math
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "tools/ui_render_segment_evidence.py"
REQUIRED_SOURCE_PATHS = (
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/plan_cache.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/render/record.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/image.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs",
    "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/segment_cache.rs",
)


def _load_tool():
    if not TOOL.is_file():
        return None
    spec = importlib.util.spec_from_file_location("ui_render_segment_evidence", TOOL)
    if spec is None or spec.loader is None:
        return None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _timeline(overrides: dict[str, float] | None = None) -> dict[str, object]:
    counters = {
        "ui.screen_space_ui.frame_prepare_count": 4,
        "ui.screen_space_ui.input_segment_count": 256,
        "ui.screen_space_ui_plan.cache_hit_count": 4,
        "ui.screen_space_ui_plan.build_count": 0,
        "ui.screen_space_ui_plan.command_visit_count": 0,
        "ui.screen_space_ui_plan.command_leaf_count": 2048,
        "ui.screen_space_ui_plan.command_leaf_cache_hit_count": 2048,
        "ui.screen_space_ui_plan.command_leaf_rebuild_count": 0,
        "ui.screen_space_ui_plan.segment_command_visit_count": 0,
        "ui.screen_space_ui_plan.composition_payload_clone_count": 0,
        "ui.screen_space_ui_vertex.plan_reuse_count": 4,
        "ui.screen_space_ui_vertex.hash_count": 0,
        "ui.screen_space_ui_vertex.hash_input_bytes": 0,
        "ui.screen_space_ui_vertex.segment_write_count": 0,
        "ui.screen_space_ui_vertex.segment_write_bytes": 0,
        "ui.screen_space_ui_vertex.segment_buffer_allocation_count": 0,
        "ui.screen_space_ui_image.segment_plan_reuse_count": 256,
        "ui.screen_space_ui_image.batch_visit_count": 0,
        "ui.screen_space_ui_image.texture_dependency_check_count": 0,
        "ui_text.segment_cache.frame_product_reuse_count": 4,
        "ui_text.segment_cache.segment_product_reuse_count": 256,
        "ui_text.segment_cache.text_batch_visit_count": 0,
        "ui_text.segment_cache.glyph_projection_count": 0,
        "ui_text.segment_cache.compatibility_batch_clone_count": 0,
        "ui_text.segment_cache.compatibility_glyph_run_clone_count": 0,
        "ui_text.segment_cache.font_dependency_segment_visit_count": 0,
        "ui_text.segment_cache.font_dependency_asset_visit_count": 0,
        "ui_text.segment_cache.font_asset_ensure_count": 0,
    }
    counters.update(overrides or {})
    return {
        "counters": [
            {"name": name, "value": value} for name, value in counters.items()
        ]
    }


def _source_manifest() -> dict[str, object]:
    return {
        "schema_version": 2,
        "scenario": "render_segment_stable",
        "repository": {
            "git": {
                "revision": "a" * 40,
                "dirty": True,
                "dirty_entry_count": 6,
                "dirty_tree_sha256": "b" * 64,
            },
            "critical_source_files": [
                {
                    "relative_path": path,
                    "sha256": "c" * 64,
                    "byte_length": 1,
                }
                for path in REQUIRED_SOURCE_PATHS
            ],
        },
        "capture": {
            "options": {
                "run_phase": "measured",
                "run_ordinal": 1,
                "measured_run_count": 3,
            }
        },
    }
class UiRenderSegmentEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tool = _load_tool()

    def require_tool(self):
        self.assertIsNotNone(self.tool, f"missing evidence tool: {TOOL}")
        return self.tool

    def test_accepts_complete_stable_frame_reuse_with_zero_downstream_work(self) -> None:
        tool = self.require_tool()
        result = tool.evaluate_stable_render_segment_evidence(_timeline())

        self.assertTrue(result["ready"])
        self.assertEqual(result["blockers"], [])
        self.assertEqual(result["conservation"]["prepared_frames"], 4)
        self.assertEqual(result["conservation"]["input_segments"], 256)

    def test_rejects_missing_counter_instead_of_interpreting_it_as_zero(self) -> None:
        tool = self.require_tool()
        timeline = _timeline()
        timeline["counters"] = [
            counter
            for counter in timeline["counters"]
            if counter["name"]
            != "ui.screen_space_ui_image.texture_dependency_check_count"
        ]

        result = tool.evaluate_stable_render_segment_evidence(timeline)

        self.assertFalse(result["ready"])
        self.assertIn("missing_counter", {item["code"] for item in result["blockers"]})

    def test_rejects_hidden_image_and_font_dependency_sweeps(self) -> None:
        tool = self.require_tool()
        result = tool.evaluate_stable_render_segment_evidence(
            _timeline(
                {
                    "ui.screen_space_ui_image.texture_dependency_check_count": 256,
                    "ui_text.segment_cache.font_dependency_asset_visit_count": 32,
                    "ui_text.segment_cache.font_asset_ensure_count": 32,
                }
            )
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("stable_frame_dependency_work", codes)

    def test_rejects_cache_hits_that_still_build_hash_or_upload(self) -> None:
        tool = self.require_tool()
        result = tool.evaluate_stable_render_segment_evidence(
            _timeline(
                {
                    "ui.screen_space_ui_plan.build_count": 1,
                    "ui.screen_space_ui_vertex.hash_count": 1,
                    "ui.screen_space_ui_vertex.segment_write_count": 1,
                }
            )
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("stable_frame_plan_work", codes)
        self.assertIn("stable_frame_vertex_work", codes)

    def test_rejects_reuse_membership_that_does_not_conserve_frames_or_segments(self) -> None:
        tool = self.require_tool()
        result = tool.evaluate_stable_render_segment_evidence(
            _timeline(
                {
                    "ui.screen_space_ui_plan.cache_hit_count": 3,
                    "ui.screen_space_ui_image.segment_plan_reuse_count": 255,
                }
            )
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("stable_frame_reuse_conservation_failed", codes)
        self.assertIn("stable_segment_reuse_conservation_failed", codes)

    def test_rejects_stable_leaf_hits_that_do_not_conserve_the_leaf_domain(self) -> None:
        tool = self.require_tool()
        result = tool.evaluate_stable_render_segment_evidence(
            _timeline(
                {
                    "ui.screen_space_ui_plan.command_leaf_cache_hit_count": 2047,
                    "ui.screen_space_ui_plan.command_leaf_rebuild_count": 1,
                }
            )
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "stable_command_leaf_reuse_conservation_failed",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_fractional_negative_or_non_finite_counter_values(self) -> None:
        tool = self.require_tool()
        for value in (-1, 0.5, math.nan, math.inf):
            with self.subTest(value=value):
                result = tool.evaluate_stable_render_segment_evidence(
                    _timeline({"ui.screen_space_ui_plan.build_count": value})
                )
                self.assertIn(
                    "invalid_counter_value",
                    {item["code"] for item in result["blockers"]},
                )

    def test_output_artifacts_are_restricted_to_d_e_or_f_drive(self) -> None:
        tool = self.require_tool()

        with self.assertRaises(ValueError):
            tool.validate_output_path(Path("C:/zircon-profiles/evidence.json"))
        self.assertEqual(
            tool.validate_output_path(Path("E:/zircon-profiles/evidence.json")).drive,
            "E:",
        )

    def test_source_manifest_binds_measured_scenario_and_every_renderer_owner(self) -> None:
        tool = self.require_tool()
        validator = getattr(tool, "validate_source_manifest", None)
        self.assertIsNotNone(validator)
        if validator is None:
            return

        self.assertEqual(validator(_source_manifest()), [])

    def test_source_manifest_rejects_missing_owner_or_non_measured_capture(self) -> None:
        tool = self.require_tool()
        validator = getattr(tool, "validate_source_manifest", None)
        self.assertIsNotNone(validator)
        if validator is None:
            return
        missing_owner = _source_manifest()
        missing_owner["repository"]["critical_source_files"].pop()
        wrong_phase = _source_manifest()
        wrong_phase["capture"]["options"]["run_phase"] = "warmup"

        missing_blockers = validator(missing_owner)
        phase_blockers = validator(wrong_phase)

        self.assertIn(
            "missing_critical_source",
            {item["code"] for item in missing_blockers},
        )
        self.assertIn(
            "invalid_capture_contract",
            {item["code"] for item in phase_blockers},
        )


if __name__ == "__main__":
    unittest.main()
