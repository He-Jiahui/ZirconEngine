from __future__ import annotations

import importlib.util
import math
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "tools/runtime_ui_surface_input_evidence.py"
REQUIRED_SOURCE_PATHS = (
    "zircon_runtime/src/dynamic_api/session/runtime_ui.rs",
    "zircon_runtime/src/ui/surface/frame_hit_test.rs",
    "zircon_runtime/src/ui/surface/surface/event_routing.rs",
)


def _load_tool():
    if not TOOL.is_file():
        return None
    spec = importlib.util.spec_from_file_location(
        "runtime_ui_surface_input_evidence", TOOL
    )
    if spec is None or spec.loader is None:
        return None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _counter(name: str, values: list[int | float]) -> list[dict[str, object]]:
    return [{"name": name, "value": value} for value in values]


def _timeline(
    route_class: str = "pointer_uncaptured",
    *,
    candidates: list[int] | None = None,
    dispatched: list[int] | None = None,
    clones: list[int] | None = None,
    rebuilds: list[int] | None = None,
    text_syncs: list[int] | None = None,
    allocations: list[int] | None = None,
    latencies_us: list[int] | None = None,
    present_latencies_us: list[int] | None = None,
    overrides: dict[str, int | float] | None = None,
) -> dict[str, object]:
    candidates = candidates or [1, 2, 1, 2]
    event_count = len(candidates)
    dispatched = dispatched or list(candidates)
    clones = clones or [max(value - 1, 0) for value in dispatched]
    rebuilds = rebuilds or [0] * event_count
    text_syncs = text_syncs or list(dispatched)
    allocations = allocations or list(clones)
    latencies_us = latencies_us or [100, 110, 105, 108]
    present_latencies_us = present_latencies_us or [5000, 5200, 5100, 5150]
    aggregates = {
        "ui.surface_set.input.event_count": event_count,
        "ui.surface_set.input.directory_query_count": event_count,
        "ui.surface_set.input.capture_direct_route_count": 0,
        "ui.surface_set.input.focus_direct_route_count": 0,
        "ui.surface_set.input.unrouted_reject_count": 0,
        "ui.surface_set.input.tree_scan_count": 0,
        "ui.surface_set.input.render_command_scan_count": 0,
        "ui.surface_set.input.publication_patch_count": 0,
        "ui.surface_set.input.publication_full_rebuild_count": 0,
    }
    if route_class == "pointer_captured":
        candidates = [0] * event_count
        dispatched = [1] * event_count
        clones = [0] * event_count
        text_syncs = [1] * event_count
        allocations = [0] * event_count
        aggregates["ui.surface_set.input.directory_query_count"] = 0
        aggregates["ui.surface_set.input.capture_direct_route_count"] = event_count
    elif route_class == "focused":
        candidates = [0] * event_count
        dispatched = [1] * event_count
        clones = [0] * event_count
        text_syncs = [1] * event_count
        allocations = [0] * event_count
        aggregates["ui.surface_set.input.directory_query_count"] = 0
        aggregates["ui.surface_set.input.focus_direct_route_count"] = event_count
    elif route_class == "unrouted":
        candidates = [0] * event_count
        dispatched = [0] * event_count
        clones = [0] * event_count
        text_syncs = [0] * event_count
        allocations = [0] * event_count
        aggregates["ui.surface_set.input.directory_query_count"] = 0
        aggregates["ui.surface_set.input.unrouted_reject_count"] = event_count
    aggregates.update(overrides or {})

    counters: list[dict[str, object]] = [
        {"name": name, "value": value} for name, value in aggregates.items()
    ]
    for name, values in (
        ("ui.surface_set.input.candidate_surface_count", candidates),
        ("ui.surface_set.input.dispatched_surface_count", dispatched),
        ("ui.surface_set.input.event_clone_count", clones),
        ("ui.surface_set.input.event_rebuild_count", rebuilds),
        ("ui.surface_set.input.text_owner_sync_count", text_syncs),
        ("ui.surface_set.input.warm_path_allocation_count", allocations),
        ("ui.surface_set.input.input_to_damage_us", latencies_us),
        ("ui.surface_set.input.input_to_present_us", present_latencies_us),
    ):
        counters.extend(_counter(name, values))
    return {"counters": counters}


def _source_manifest(route_class: str) -> dict[str, object]:
    return {
        "schema_version": 2,
        "scenario": f"runtime_ui_{route_class}",
        "repository": {
            "git": {
                "revision": "a" * 40,
                "dirty": True,
                "dirty_entry_count": 4,
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


class RuntimeUiSurfaceInputEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tool = _load_tool()

    def require_tool(self):
        self.assertIsNotNone(self.tool, f"missing evidence tool: {TOOL}")
        return self.tool

    def test_accepts_uncaptured_pointer_with_bounded_candidate_fallthrough(self) -> None:
        result = self.require_tool().evaluate_input_run(
            _timeline(), "pointer_uncaptured", 64
        )

        self.assertTrue(result["ready"])
        self.assertEqual([], result["blockers"])
        self.assertEqual(2, result["metrics"]["candidate_surface_p95"])
        self.assertEqual(6, result["metrics"]["dispatched_surface_count"])

    def test_accepts_capture_focus_and_unrouted_direct_routes(self) -> None:
        tool = self.require_tool()

        captured = tool.evaluate_input_run(
            _timeline("pointer_captured"), "pointer_captured", 64
        )
        focused = tool.evaluate_input_run(
            _timeline("focused"), "focused", 64
        )
        unrouted = tool.evaluate_input_run(
            _timeline("unrouted"), "unrouted", 64
        )

        self.assertTrue(captured["ready"])
        self.assertTrue(focused["ready"])
        self.assertTrue(unrouted["ready"])

    def test_rejects_missing_per_event_sample_instead_of_using_totals(self) -> None:
        tool = self.require_tool()
        timeline = _timeline()
        removed = False
        counters = []
        for counter in timeline["counters"]:
            if (
                not removed
                and counter["name"]
                == "ui.surface_set.input.candidate_surface_count"
            ):
                removed = True
                continue
            counters.append(counter)
        timeline["counters"] = counters

        result = tool.evaluate_input_run(timeline, "pointer_uncaptured", 64)

        self.assertIn(
            "sample_count_mismatch", {item["code"] for item in result["blockers"]}
        )

    def test_rejects_event_time_rebuild_tree_or_render_scan(self) -> None:
        result = self.require_tool().evaluate_input_run(
            _timeline(
                rebuilds=[0, 1, 0, 0],
                overrides={
                    "ui.surface_set.input.tree_scan_count": 64,
                    "ui.surface_set.input.render_command_scan_count": 8,
                },
            ),
            "pointer_uncaptured",
            64,
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("event_time_rebuild_detected", codes)
        self.assertIn("event_time_global_scan_detected", codes)

    def test_rejects_candidate_p95_above_two_or_dispatch_beyond_candidate(self) -> None:
        result = self.require_tool().evaluate_input_run(
            _timeline(candidates=[1, 3, 3, 4], dispatched=[1, 3, 4, 4]),
            "pointer_uncaptured",
            64,
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("candidate_surface_p95_exceeded", codes)
        self.assertIn("dispatch_conservation_failed", codes)

    def test_rejects_candidate_count_above_surface_count(self) -> None:
        result = self.require_tool().evaluate_input_run(
            _timeline(candidates=[2, 2, 2, 2]), "pointer_uncaptured", 1
        )

        self.assertIn(
            "candidate_count_exceeds_surface_count",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_input_to_damage_or_present_p95_over_budget(self) -> None:
        result = self.require_tool().evaluate_input_run(
            _timeline(
                latencies_us=[900, 1000, 1001, 1200],
                present_latencies_us=[8000, 9000, 9001, 12000],
            ),
            "pointer_uncaptured",
            64,
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("input_to_damage_p95_exceeded", codes)
        self.assertIn("input_to_present_p95_exceeded", codes)

    def test_rejects_direct_route_fanout_clone_or_warm_allocation(self) -> None:
        timeline = _timeline("focused")
        for counter in timeline["counters"]:
            if counter["name"] == "ui.surface_set.input.dispatched_surface_count":
                counter["value"] = 2
                break
        for counter in timeline["counters"]:
            if counter["name"] == "ui.surface_set.input.event_clone_count":
                counter["value"] = 1
                break
        for counter in timeline["counters"]:
            if counter["name"] == "ui.surface_set.input.warm_path_allocation_count":
                counter["value"] = 1
                break

        result = self.require_tool().evaluate_input_run(timeline, "focused", 64)

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("direct_route_fanout_detected", codes)
        self.assertIn("direct_route_clone_detected", codes)
        self.assertIn("unexpected_warm_path_allocation", codes)

    def test_rejects_missing_fractional_negative_or_non_finite_counter(self) -> None:
        tool = self.require_tool()
        for value in (-1, 0.5, math.nan, math.inf):
            with self.subTest(value=value):
                timeline = _timeline()
                for counter in timeline["counters"]:
                    if counter["name"] == "ui.surface_set.input.event_rebuild_count":
                        counter["value"] = value
                        break
                result = tool.evaluate_input_run(
                    timeline, "pointer_uncaptured", 64
                )
                self.assertIn(
                    "invalid_counter_value",
                    {item["code"] for item in result["blockers"]},
                )

    def test_source_manifest_requires_measured_owner_fingerprints(self) -> None:
        tool = self.require_tool()
        manifest = _source_manifest("pointer_uncaptured")

        self.assertEqual(
            [], tool.validate_source_manifest(manifest, "pointer_uncaptured")
        )

        manifest["repository"]["critical_source_files"][0]["sha256"] = ""
        blockers = tool.validate_source_manifest(manifest, "pointer_uncaptured")
        self.assertIn(
            "invalid_source_fingerprint", {item["code"] for item in blockers}
        )

    def test_diagnostic_report_without_source_manifest_is_never_accepted(self) -> None:
        result = self.require_tool().build_input_report(
            _timeline(), "pointer_uncaptured", 64, None
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "missing_source_manifest", {item["code"] for item in result["blockers"]}
        )

    def test_tool_binding_fingerprints_the_analyzer_separately(self) -> None:
        binding = self.require_tool().tool_binding()

        self.assertEqual(str(TOOL), binding["path"])
        self.assertRegex(binding["sha256"], r"^[0-9A-F]{64}$")
        self.assertGreater(binding["byte_length"], 0)

    def test_surface_scaling_accepts_bounded_focus_and_pointer_p95(self) -> None:
        tool = self.require_tool()
        runs = []
        for surface_count, focus_p95, pointer_p95 in (
            (1, 100, 120),
            (4, 102, 124),
            (16, 104, 128),
            (64, 105, 132),
        ):
            runs.extend(
                [
                    _scaling_run("focused", surface_count, focus_p95, 0),
                    _scaling_run("pointer_uncaptured", surface_count, pointer_p95, 1),
                ]
            )

        result = tool.evaluate_surface_scaling(runs)

        self.assertTrue(result["ready"])
        self.assertEqual([], result["blockers"])

    def test_surface_scaling_rejects_missing_count_or_superlinear_latency(self) -> None:
        tool = self.require_tool()
        runs = []
        for surface_count in (1, 4, 64):
            runs.extend(
                [
                    _scaling_run(
                        "focused", surface_count, 100 if surface_count == 1 else 120, 0
                    ),
                    _scaling_run(
                        "pointer_uncaptured",
                        surface_count,
                        100 if surface_count == 1 else 130,
                        1,
                    ),
                ]
            )

        result = tool.evaluate_surface_scaling(runs)

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("missing_surface_scale_run", codes)
        self.assertIn("focused_scaling_regression", codes)
        self.assertIn("pointer_scaling_regression", codes)

    def test_surface_scaling_rejects_noncanonical_candidate_or_wrong_schema(self) -> None:
        tool = self.require_tool()
        runs = []
        for surface_count in (1, 4, 16, 64):
            runs.extend(
                [
                    _scaling_run("focused", surface_count, 100, 0),
                    _scaling_run("pointer_uncaptured", surface_count, 100, 1),
                ]
            )
        runs[0]["schema"] = "fabricated"
        runs[-1]["metrics"]["candidate_surface_p95"] = 2

        result = tool.evaluate_surface_scaling(runs)

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("invalid_scaling_run", codes)
        self.assertIn("noncanonical_pointer_candidate_set", codes)

    def test_output_artifacts_are_restricted_to_d_e_or_f_drive(self) -> None:
        tool = self.require_tool()
        with self.assertRaises(ValueError):
            tool.validate_output_path(Path("C:/zircon-profiles/runtime-ui-input.json"))
        self.assertEqual(
            "E:",
            tool.validate_output_path(
                Path("E:/zircon-profiles/runtime-ui-input.json")
            ).drive,
        )


def _scaling_run(
    route_class: str, surface_count: int, p95_us: int, candidate_p95: int
) -> dict[str, object]:
    return {
        "schema": "zircon.runtime.ui_surface_input_evidence.v1",
        "ready": True,
        "route_class": route_class,
        "surface_count": surface_count,
        "metrics": {
            "input_to_damage_p95_us": p95_us,
            "input_to_present_p95_us": p95_us * 10,
            "candidate_surface_p95": candidate_p95,
        },
    }


if __name__ == "__main__":
    unittest.main()
