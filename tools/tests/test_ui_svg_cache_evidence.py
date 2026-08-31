from __future__ import annotations

import importlib.util
import math
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "tools/ui_svg_cache_evidence.py"
REQUIRED_SOURCE_PATHS = (
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/cache.rs",
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/svg/pixels.rs",
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/cache.rs",
    "zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/visual_assets/loading/async_loader.rs",
    "zircon_editor/src/ui/retained_host/app/assets/refresh.rs",
    "zircon_editor/src/ui/retained_host/ui_perf.rs",
    "zircon_editor/src/ui/retained_host/ui_perf/counter_catalog.rs",
    "zircon_editor/src/ui/retained_host/host_contract/presenter/gpu/stats.rs",
    "zircon_runtime/crates/zr_rhi/src/ui_surface.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/presentation.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry.rs",
    "zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/shared_image_registry/allocation_ledger.rs",
    "tools/ui-profile-counter-evidence.ps1",
)
DEVICE_IMAGE_BUDGET_BYTES = 64 * 1024 * 1024


def _load_tool():
    if not TOOL.is_file():
        return None
    spec = importlib.util.spec_from_file_location("ui_svg_cache_evidence", TOOL)
    if spec is None or spec.loader is None:
        return None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _timeline(overrides: dict[str, float] | None = None) -> dict[str, object]:
    prefix = "ui.idle_hover."
    counters = {
        f"{prefix}visual_asset_cache_hit_count": 8,
        f"{prefix}visual_asset_cache_miss_count": 0,
        f"{prefix}visual_asset_cache_candidate_build_count": 0,
        f"{prefix}svg_tree_cache_memory_hit_count": 4,
        f"{prefix}svg_tree_cache_miss_count": 0,
        f"{prefix}svg_parse_count": 0,
        f"{prefix}svg_parse_bytes": 0,
        f"{prefix}svg_raster_count": 0,
        f"{prefix}svg_raster_pixels": 0,
        f"{prefix}svg_raster_product_hit_count": 4,
        f"{prefix}svg_raster_product_miss_count": 0,
        f"{prefix}svg_raster_unique_bucket_count": 0,
        f"{prefix}visual_asset_async_stale_discard_count": 0,
        f"{prefix}gpu_image_prepare_cache_hits": 8,
        f"{prefix}gpu_image_prepare_command_visits": 0,
        f"{prefix}gpu_image_upload_writes": 0,
        f"{prefix}gpu_image_shared_upload_writes": 0,
        f"{prefix}gpu_image_cache_key_allocations": 0,
        f"{prefix}gpu_image_device_allocation_count": 4,
        f"{prefix}gpu_image_device_allocation_bytes": 16 * 1024 * 1024,
        f"{prefix}gpu_image_registry_evicted_pinned_bytes": 0,
        f"{prefix}gpu_image_surface_pin_count": 4,
        f"{prefix}gpu_image_in_flight_present_pin_count": 0,
        f"{prefix}gpu_image_eviction_completion_count": 0,
    }
    counters.update(overrides or {})
    return {
        "counters": [
            {"name": name, "value": value} for name, value in counters.items()
        ],
        "spans": [
            {"name": "visual_assets_render_svg_parse", "duration_us": 110},
            {"name": "visual_assets_render_svg_raster", "duration_us": 7},
            {"name": "visual_assets_render_svg_raster", "duration_us": 9},
        ],
    }


def _source_manifest() -> dict[str, object]:
    return {
        "schema_version": 2,
        "scenario": "idle_hover",
        "repository": {
            "git": {
                "revision": "a" * 40,
                "dirty": True,
                "dirty_entry_count": 5,
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


class UiSvgCacheEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tool = _load_tool()

    def require_tool(self):
        self.assertIsNotNone(self.tool, f"missing evidence tool: {TOOL}")
        return self.tool

    def test_accepts_explicit_zero_work_with_positive_retained_hits(self) -> None:
        result = self.require_tool().evaluate_stable_svg_cache_evidence(
            _timeline(), "idle_hover"
        )

        self.assertTrue(result["ready"])
        self.assertEqual([], result["blockers"])
        self.assertEqual(8, result["retained_hits"]["visual_asset"])
        self.assertEqual(4, result["retained_hits"]["raster_product"])

    def test_missing_parse_or_raster_counter_is_not_interpreted_as_zero(self) -> None:
        tool = self.require_tool()
        timeline = _timeline()
        timeline["counters"] = [
            counter
            for counter in timeline["counters"]
            if counter["name"]
            not in {
                "ui.idle_hover.svg_parse_count",
                "ui.idle_hover.svg_raster_count",
            }
        ]

        result = tool.evaluate_stable_svg_cache_evidence(timeline, "idle_hover")

        missing = {
            blocker["counter"]
            for blocker in result["blockers"]
            if blocker["code"] == "missing_counter"
        }
        self.assertEqual(
            {"ui.idle_hover.svg_parse_count", "ui.idle_hover.svg_raster_count"},
            missing,
        )

    def test_rejects_cpu_or_gpu_work_even_when_retained_hits_are_positive(self) -> None:
        result = self.require_tool().evaluate_stable_svg_cache_evidence(
            _timeline(
                {
                    "ui.idle_hover.svg_parse_count": 1,
                    "ui.idle_hover.svg_raster_count": 2,
                    "ui.idle_hover.visual_asset_async_stale_discard_count": 1,
                    "ui.idle_hover.gpu_image_upload_writes": 1,
                }
            ),
            "idle_hover",
        )

        work = {
            blocker["counter"]
            for blocker in result["blockers"]
            if blocker["code"] == "stable_svg_work_detected"
        }
        self.assertEqual(
            {
                "ui.idle_hover.svg_parse_count",
                "ui.idle_hover.svg_raster_count",
                "ui.idle_hover.visual_asset_async_stale_discard_count",
                "ui.idle_hover.gpu_image_upload_writes",
            },
            work,
        )

    def test_missing_async_stale_discard_counter_is_not_interpreted_as_zero(self) -> None:
        timeline = _timeline()
        timeline["counters"] = [
            counter
            for counter in timeline["counters"]
            if counter["name"]
            != "ui.idle_hover.visual_asset_async_stale_discard_count"
        ]

        result = self.require_tool().evaluate_stable_svg_cache_evidence(
            timeline, "idle_hover"
        )

        self.assertIn(
            "ui.idle_hover.visual_asset_async_stale_discard_count",
            {
                blocker["counter"]
                for blocker in result["blockers"]
                if blocker["code"] == "missing_counter"
            },
        )

    def test_device_residency_gauges_are_required_and_budget_bounded(self) -> None:
        tool = self.require_tool()
        missing = _timeline()
        missing["counters"] = [
            counter
            for counter in missing["counters"]
            if counter["name"]
            != "ui.idle_hover.gpu_image_device_allocation_bytes"
        ]
        over_budget = _timeline(
            {
                "ui.idle_hover.gpu_image_device_allocation_bytes": (
                    DEVICE_IMAGE_BUDGET_BYTES + 1
                )
            }
        )
        invalid_pinned = _timeline(
            {
                "ui.idle_hover.gpu_image_device_allocation_bytes": 8,
                "ui.idle_hover.gpu_image_registry_evicted_pinned_bytes": 9,
            }
        )

        missing_result = tool.evaluate_stable_svg_cache_evidence(
            missing, "idle_hover"
        )
        budget_result = tool.evaluate_stable_svg_cache_evidence(
            over_budget, "idle_hover"
        )
        pinned_result = tool.evaluate_stable_svg_cache_evidence(
            invalid_pinned, "idle_hover"
        )

        self.assertIn(
            "ui.idle_hover.gpu_image_device_allocation_bytes",
            {
                blocker["counter"]
                for blocker in missing_result["blockers"]
                if blocker["code"] == "missing_counter"
            },
        )
        self.assertIn(
            "device_allocation_budget_exceeded",
            {blocker["code"] for blocker in budget_result["blockers"]},
        )
        self.assertIn(
            "invalid_device_allocation_relationship",
            {blocker["code"] for blocker in pinned_result["blockers"]},
        )

    def test_quiescent_residency_requires_gpu_and_evicted_pins_to_drain(self) -> None:
        tool = self.require_tool()
        timeline = _timeline(
            {
                "ui.idle_hover.gpu_image_registry_evicted_pinned_bytes": 4_096,
                "ui.idle_hover.gpu_image_in_flight_present_pin_count": 2,
            }
        )

        result = tool.evaluate_stable_svg_cache_evidence(
            timeline, "idle_hover", require_quiescent=True
        )

        self.assertEqual(
            {
                "ui.idle_hover.gpu_image_registry_evicted_pinned_bytes",
                "ui.idle_hover.gpu_image_in_flight_present_pin_count",
            },
            {
                blocker["counter"]
                for blocker in result["blockers"]
                if blocker["code"] == "device_residency_not_quiescent"
            },
        )

    def test_rejects_missing_retained_hits_and_invalid_counter_values(self) -> None:
        tool = self.require_tool()
        no_hits = tool.evaluate_stable_svg_cache_evidence(
            _timeline({"ui.idle_hover.svg_raster_product_hit_count": 0}),
            "idle_hover",
        )
        invalid = tool.evaluate_stable_svg_cache_evidence(
            _timeline({"ui.idle_hover.svg_parse_count": math.nan}),
            "idle_hover",
        )

        self.assertIn(
            "missing_retained_hit", {item["code"] for item in no_hits["blockers"]}
        )
        self.assertIn(
            "invalid_counter_value", {item["code"] for item in invalid["blockers"]}
        )

    def test_capture_wide_spans_are_diagnostic_not_scenario_attribution(self) -> None:
        result = self.require_tool().evaluate_stable_svg_cache_evidence(
            _timeline(), "idle_hover"
        )

        self.assertEqual(
            {"count": 1, "duration_us": 110},
            result["capture_wide_spans"]["svg_parse"],
        )
        self.assertEqual(
            {"count": 2, "duration_us": 16},
            result["capture_wide_spans"]["svg_raster"],
        )
        self.assertEqual(
            "diagnostic_only_not_scenario_attribution",
            result["capture_wide_spans"]["authority"],
        )

    def test_source_manifest_requires_measured_phase_and_every_owner(self) -> None:
        tool = self.require_tool()
        self.assertEqual([], tool.validate_source_manifest(_source_manifest(), "idle_hover"))

        missing_owner = _source_manifest()
        missing_owner["repository"]["critical_source_files"].pop()
        wrong_phase = _source_manifest()
        wrong_phase["capture"]["options"]["run_phase"] = "warmup"

        self.assertIn(
            "missing_critical_source",
            {
                item["code"]
                for item in tool.validate_source_manifest(
                    missing_owner, "idle_hover"
                )
            },
        )
        self.assertIn(
            "invalid_capture_contract",
            {
                item["code"]
                for item in tool.validate_source_manifest(wrong_phase, "idle_hover")
            },
        )

    def test_source_manifest_rejects_an_unfingerprinted_required_owner(self) -> None:
        tool = self.require_tool()
        manifest = _source_manifest()
        manifest["repository"]["critical_source_files"][0]["sha256"] = ""
        manifest["repository"]["critical_source_files"][0]["byte_length"] = 0

        blockers = tool.validate_source_manifest(manifest, "idle_hover")

        self.assertIn("invalid_source_fingerprint", {item["code"] for item in blockers})

    def test_diagnostic_report_without_source_manifest_cannot_be_accepted(self) -> None:
        result = self.require_tool().build_svg_cache_report(
            _timeline(), "idle_hover", None
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "missing_source_manifest", {item["code"] for item in result["blockers"]}
        )

    def test_output_artifacts_are_restricted_to_d_e_or_f_drive(self) -> None:
        tool = self.require_tool()

        with self.assertRaises(ValueError):
            tool.validate_output_path(Path("C:/zircon-profiles/svg-cache.json"))
        self.assertEqual(
            "E:",
            tool.validate_output_path(
                Path("E:/zircon-profiles/svg-cache.json")
            ).drive,
        )


if __name__ == "__main__":
    unittest.main()
