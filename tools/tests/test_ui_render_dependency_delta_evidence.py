import importlib.util
import math
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
TOOL_PATH = REPO_ROOT / "tools" / "ui_render_dependency_delta_evidence.py"


def _load_tool():
    spec = importlib.util.spec_from_file_location(
        "ui_render_dependency_delta_evidence", TOOL_PATH
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {TOOL_PATH}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def _timeline(overrides=None):
    counters = {
        "ui.screen_space_ui.prepared_frame_count": 1,
        "ui.screen_space_ui.input_segment_count": 64,
        "ui.screen_space_ui.changed_segment_count": 1,
        "ui.screen_space_ui.changed_segment_command_count": 16,
        "ui.screen_space_ui.segment_delta_full_fallback_count": 0,
        "ui.screen_space_ui_plan.segment_cache_hit_count": 511,
        "ui.screen_space_ui_plan.command_leaf_count": 512,
        "ui.screen_space_ui_plan.command_leaf_cache_hit_count": 511,
        "ui.screen_space_ui_plan.command_leaf_rebuild_count": 1,
        "ui.screen_space_ui_plan.segment_command_visit_count": 16,
        "ui.screen_space_ui_image.segment_plan_reuse_count": 63,
        "ui.screen_space_ui_image.changed_texture_dependency_count": 2,
        "ui.screen_space_ui_image.texture_dependency_check_count": 2,
        "ui.screen_space_ui_image.binding_lookup_count": 2,
        "ui.screen_space_ui_image.binding_retention_scan_count": 0,
        "ui_text.segment_cache.segment_product_reuse_count": 63,
        "ui_text.segment_cache.changed_text_batch_count": 3,
        "ui_text.segment_cache.text_batch_visit_count": 3,
        "ui_text.segment_cache.changed_glyph_count": 24,
        "ui_text.segment_cache.glyph_projection_count": 24,
        "ui_text.segment_cache.changed_dependency_segment_count": 1,
        "ui_text.segment_cache.changed_dependency_count": 4,
        "ui_text.segment_cache.frame_dependency_segment_visit_count": 1,
        "ui_text.segment_cache.frame_dependency_entry_visit_count": 4,
    }
    counters.update(overrides or {})
    return {"counters": [{"name": name, "value": value} for name, value in counters.items()]}


def _source_manifest():
    return {
        "schema_version": 2,
        "scenario": "render_segment_delta",
        "capture": {
            "options": {
                "run_phase": "measured",
                "run_ordinal": 1,
                "measured_run_count": 3,
                "warmup_complete": True,
            }
        },
        "repository": {
            "git": {"revision": "a" * 40, "dirty_paths": []},
            "critical_source_files": [
                {"relative_path": path, "sha256": "b" * 64}
                for path in tool.CRITICAL_SOURCE_FILES
            ],
        },
    }


tool = _load_tool()


class RenderDependencyDeltaEvidenceTests(unittest.TestCase):
    def test_accepts_one_segment_delta_with_exact_local_work(self):
        result = tool.evaluate_render_dependency_delta_evidence(_timeline())

        self.assertTrue(result["ready"])
        self.assertEqual(result["summary"]["expected_reused_surface_segments"], 63)
        self.assertEqual(result["summary"]["expected_reused_command_leaves"], 511)
        self.assertEqual(result["blockers"], [])

    def test_rejects_missing_counter_instead_of_treating_it_as_zero(self):
        timeline = _timeline()
        timeline["counters"] = [
            counter
            for counter in timeline["counters"]
            if counter["name"]
            != "ui.screen_space_ui_image.binding_retention_scan_count"
        ]

        result = tool.evaluate_render_dependency_delta_evidence(timeline)

        self.assertFalse(result["ready"])
        self.assertIn("missing_counter", {item["code"] for item in result["blockers"]})

    def test_rejects_segment_reuse_conservation_mismatch(self):
        result = tool.evaluate_render_dependency_delta_evidence(
            _timeline(
                {
                    "ui.screen_space_ui_plan.segment_cache_hit_count": 62,
                    "ui_text.segment_cache.segment_product_reuse_count": 64,
                }
            )
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "surface_segment_reuse_conservation_failed",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_command_leaf_reuse_conservation_mismatch(self):
        result = tool.evaluate_render_dependency_delta_evidence(
            _timeline(
                {
                    "ui.screen_space_ui_plan.command_leaf_cache_hit_count": 510,
                    "ui.screen_space_ui_plan.command_leaf_rebuild_count": 2,
                }
            )
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "command_leaf_reuse_conservation_failed",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_frame_wide_image_dependency_and_binding_work(self):
        result = tool.evaluate_render_dependency_delta_evidence(
            _timeline(
                {
                    "ui.screen_space_ui_image.texture_dependency_check_count": 128,
                    "ui.screen_space_ui_image.binding_lookup_count": 128,
                    "ui.screen_space_ui_image.binding_retention_scan_count": 512,
                }
            )
        )

        self.assertFalse(result["ready"])
        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("image_dependency_delta_failed", codes)
        self.assertIn("binding_global_scan_detected", codes)

    def test_rejects_frame_wide_text_dependency_recomposition(self):
        result = tool.evaluate_render_dependency_delta_evidence(
            _timeline(
                {
                    "ui_text.segment_cache.frame_dependency_segment_visit_count": 64,
                    "ui_text.segment_cache.frame_dependency_entry_visit_count": 256,
                }
            )
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "text_dependency_delta_failed",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_untyped_full_fallback(self):
        result = tool.evaluate_render_dependency_delta_evidence(
            _timeline({"ui.screen_space_ui.segment_delta_full_fallback_count": 1})
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "unexpected_full_fallback",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_fractional_negative_or_non_finite_values(self):
        for invalid in (-1, 0.5, math.inf, math.nan):
            with self.subTest(invalid=invalid):
                result = tool.evaluate_render_dependency_delta_evidence(
                    _timeline({"ui.screen_space_ui.changed_segment_count": invalid})
                )
                self.assertFalse(result["ready"])
                self.assertIn(
                    "invalid_counter_value",
                    {item["code"] for item in result["blockers"]},
                )

    def test_output_artifacts_are_restricted_to_d_e_or_f_drive(self):
        for accepted in (Path("D:/profiles/out.json"), Path("E:/out.json"), Path("F:/out.json")):
            with self.subTest(path=accepted):
                self.assertEqual(tool.validate_output_path(accepted), accepted)

        with self.assertRaises(ValueError):
            tool.validate_output_path(Path("C:/temp/out.json"))

    def test_source_manifest_binds_measured_scenario_and_renderer_owners(self):
        self.assertEqual(tool.validate_source_manifest(_source_manifest()), [])

        missing_owner = _source_manifest()
        missing_owner["repository"]["critical_source_files"].pop()
        wrong_phase = _source_manifest()
        wrong_phase["capture"]["options"]["run_phase"] = "warmup"
        invalid_hash = _source_manifest()
        invalid_hash["repository"]["critical_source_files"][0]["sha256"] = "not-a-hash"

        self.assertIn(
            "missing_critical_source",
            {item["code"] for item in tool.validate_source_manifest(missing_owner)},
        )
        self.assertIn(
            "capture_not_measured",
            {item["code"] for item in tool.validate_source_manifest(wrong_phase)},
        )
        self.assertIn(
            "invalid_critical_source_hash",
            {item["code"] for item in tool.validate_source_manifest(invalid_hash)},
        )


if __name__ == "__main__":
    unittest.main()
