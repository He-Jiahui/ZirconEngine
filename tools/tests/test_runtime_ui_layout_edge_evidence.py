from __future__ import annotations

import importlib.util
import math
from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / "tools/runtime_ui_layout_edge_evidence.py"
SCHEMA = "zircon.runtime.ui_layout_edge_evidence.v1"
REQUIRED_SOURCE_PATHS = (
    "zircon_runtime_interface/src/ui/tree/node/ui_tree.rs",
    "zircon_runtime/src/ui/layout/pass/slot.rs",
    "zircon_runtime/src/ui/surface/virtual_list_prototype_pool.rs",
)


def _load_tool():
    if not TOOL.is_file():
        return None
    spec = importlib.util.spec_from_file_location("runtime_ui_layout_edge_evidence", TOOL)
    if spec is None or spec.loader is None:
        return None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _timeline(
    scenario: str = "child_dependency_patch",
    *,
    overrides: dict[str, int | float] | None = None,
    durations_us: list[int | float] | None = None,
) -> dict[str, object]:
    values: dict[str, int | float] = {
        "operation_count": 4,
        "edge_count": 10_000,
        "legacy_slot_count": 0,
        "retained_flat_slot_count": 0,
        "affected_parent_count": 0,
        "affected_parent_edge_count": 0,
        "changed_child_count": 4,
        "journal_mutation_count": 4,
        "structure_mutation_count": 4,
        "edge_projection_visit_count": 4,
        "parent_child_visit_count": 0,
        "changed_child_visit_count": 4,
        "workspace_slot_visit_count": 0,
        "unrelated_parent_slot_visit_count": 0,
        "missing_edge_global_slot_visit_count": 0,
        "fallback_repair_count": 0,
        "parity_mismatch_count": 0,
        "allocation_count": 4,
        "allocation_bytes": 256,
    }
    if scenario == "full_build":
        values.update(
            operation_count=1,
            changed_child_count=0,
            journal_mutation_count=0,
            structure_mutation_count=0,
            edge_projection_visit_count=10_000,
            parent_child_visit_count=10_000,
            changed_child_visit_count=0,
            allocation_count=32,
            allocation_bytes=262_144,
        )
    elif scenario == "legacy_migration":
        values.update(
            operation_count=1,
            legacy_slot_count=10_000,
            changed_child_count=0,
            journal_mutation_count=0,
            structure_mutation_count=0,
            edge_projection_visit_count=20_000,
            parent_child_visit_count=10_000,
            changed_child_visit_count=0,
            workspace_slot_visit_count=10_000,
            allocation_count=64,
            allocation_bytes=524_288,
        )
    elif scenario == "parent_order_patch":
        values.update(
            affected_parent_count=4,
            affected_parent_edge_count=256,
            changed_child_count=0,
            edge_projection_visit_count=256,
            parent_child_visit_count=256,
            changed_child_visit_count=0,
        )
    values.update(overrides or {})
    counters = [
        {"name": f"ui.layout_edge.{name}", "value": value}
        for name, value in values.items()
    ]
    durations_us = durations_us or [10] * int(values["operation_count"])
    counters.extend(
        {
            "name": "ui.layout_edge.operation_duration_us",
            "value": value,
        }
        for value in durations_us
    )
    return {"counters": counters}


def _source_manifest(scenario: str) -> dict[str, object]:
    return {
        "schema_version": 2,
        "scenario": f"runtime_ui_layout_edge_{scenario}",
        "repository": {
            "git": {
                "revision": "a" * 40,
                "dirty": True,
                "dirty_entry_count": 3,
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


class RuntimeUiLayoutEdgeEvidenceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tool = _load_tool()

    def require_tool(self):
        self.assertIsNotNone(self.tool, f"missing evidence tool: {TOOL}")
        return self.tool

    def test_accepts_parent_owned_full_build_and_legacy_migration(self) -> None:
        tool = self.require_tool()
        full = tool.evaluate_layout_edge_run(_timeline("full_build"), "full_build")
        migration = tool.evaluate_layout_edge_run(
            _timeline("legacy_migration"), "legacy_migration"
        )

        self.assertTrue(full["ready"])
        self.assertTrue(migration["ready"])
        self.assertEqual(10_000, full["metrics"]["operation_visit_count"])
        self.assertEqual(30_000, migration["metrics"]["operation_visit_count"])

    def test_accepts_exact_child_and_parent_order_patches(self) -> None:
        tool = self.require_tool()
        child = tool.evaluate_layout_edge_run(
            _timeline("child_dependency_patch"), "child_dependency_patch"
        )
        order = tool.evaluate_layout_edge_run(
            _timeline("parent_order_patch"), "parent_order_patch"
        )

        self.assertTrue(child["ready"])
        self.assertTrue(order["ready"])
        self.assertEqual(8, child["metrics"]["operation_visit_count"])
        self.assertEqual(512, order["metrics"]["operation_visit_count"])

    def test_rejects_missing_counter_or_duration_sample(self) -> None:
        tool = self.require_tool()
        timeline = _timeline()
        timeline["counters"] = [
            counter
            for counter in timeline["counters"]
            if counter["name"] != "ui.layout_edge.workspace_slot_visit_count"
        ]
        for index, counter in enumerate(timeline["counters"]):
            if counter["name"] == "ui.layout_edge.operation_duration_us":
                del timeline["counters"][index]
                break

        result = tool.evaluate_layout_edge_run(timeline, "child_dependency_patch")

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("missing_counter", codes)
        self.assertIn("sample_count_mismatch", codes)

    def test_rejects_workspace_missing_edge_unrelated_or_fallback_repair_visits(self) -> None:
        result = self.require_tool().evaluate_layout_edge_run(
            _timeline(
                overrides={
                    "workspace_slot_visit_count": 10_000,
                    "unrelated_parent_slot_visit_count": 8_000,
                    "missing_edge_global_slot_visit_count": 100_000_000,
                    "fallback_repair_count": 1,
                }
            ),
            "child_dependency_patch",
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("runtime_flat_slot_scan_detected", codes)
        self.assertIn("unrelated_parent_visit_detected", codes)
        self.assertIn("defensive_repair_detected", codes)

    def test_rejects_child_patch_that_visits_parent_or_more_than_changed_children(self) -> None:
        result = self.require_tool().evaluate_layout_edge_run(
            _timeline(
                overrides={
                    "edge_projection_visit_count": 8,
                    "parent_child_visit_count": 10_000,
                    "changed_child_visit_count": 8,
                }
            ),
            "child_dependency_patch",
        )

        self.assertIn(
            "child_patch_locality_failed",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_parent_order_patch_beyond_affected_parent_edges(self) -> None:
        result = self.require_tool().evaluate_layout_edge_run(
            _timeline(
                "parent_order_patch",
                overrides={
                    "edge_projection_visit_count": 257,
                    "parent_child_visit_count": 10_000,
                },
            ),
            "parent_order_patch",
        )

        self.assertIn(
            "parent_order_locality_failed",
            {item["code"] for item in result["blockers"]},
        )

    def test_rejects_retained_legacy_authority_or_journal_mismatch(self) -> None:
        result = self.require_tool().evaluate_layout_edge_run(
            _timeline(
                overrides={
                    "retained_flat_slot_count": 10_000,
                    "journal_mutation_count": 3,
                    "structure_mutation_count": 4,
                }
            ),
            "child_dependency_patch",
        )

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("retained_flat_slot_authority", codes)
        self.assertIn("mutation_journal_conservation_failed", codes)

    def test_rejects_parity_mismatch_or_invalid_counter_value(self) -> None:
        tool = self.require_tool()
        mismatch = tool.evaluate_layout_edge_run(
            _timeline(overrides={"parity_mismatch_count": 1}),
            "child_dependency_patch",
        )
        self.assertIn("layout_parity_failed", {item["code"] for item in mismatch["blockers"]})

        for value in (-1, 0.5, math.nan, math.inf):
            with self.subTest(value=value):
                invalid = tool.evaluate_layout_edge_run(
                    _timeline(overrides={"changed_child_visit_count": value}),
                    "child_dependency_patch",
                )
                self.assertIn(
                    "invalid_counter_value",
                    {item["code"] for item in invalid["blockers"]},
                )

    def test_scaling_keeps_local_work_independent_of_unrelated_slots(self) -> None:
        tool = self.require_tool()
        runs = []
        for scenario, operation_visits in (
            ("child_dependency_patch", 2),
            ("parent_order_patch", 128),
        ):
            for unrelated_slots in (64, 1_000, 10_000):
                runs.append(
                    _scale_run(scenario, unrelated_slots, operation_visits, 10, 256)
                )

        result = tool.evaluate_unrelated_slot_scaling(runs)

        self.assertTrue(result["ready"])
        self.assertEqual([], result["blockers"])

    def test_scaling_rejects_missing_size_or_slot_dependent_local_work(self) -> None:
        tool = self.require_tool()
        runs = []
        for scenario, base_visits in (
            ("child_dependency_patch", 2),
            ("parent_order_patch", 128),
        ):
            for unrelated_slots in (64, 10_000):
                multiplier = 100 if unrelated_slots == 10_000 else 1
                runs.append(
                    _scale_run(
                        scenario,
                        unrelated_slots,
                        base_visits * multiplier,
                        10 * multiplier,
                        256 * multiplier,
                    )
                )

        result = tool.evaluate_unrelated_slot_scaling(runs)

        codes = {item["code"] for item in result["blockers"]}
        self.assertIn("missing_unrelated_slot_scale_run", codes)
        self.assertIn("local_work_scales_with_unrelated_slots", codes)
        self.assertIn("local_latency_scales_with_unrelated_slots", codes)
        self.assertIn("local_allocation_scales_with_unrelated_slots", codes)

    def test_source_manifest_and_tool_binding_are_independent(self) -> None:
        tool = self.require_tool()
        manifest = _source_manifest("child_dependency_patch")

        self.assertEqual(
            [], tool.validate_source_manifest(manifest, "child_dependency_patch")
        )
        manifest["repository"]["critical_source_files"][0]["sha256"] = ""
        self.assertIn(
            "invalid_source_fingerprint",
            {
                item["code"]
                for item in tool.validate_source_manifest(
                    manifest, "child_dependency_patch"
                )
            },
        )
        self.assertRegex(tool.tool_binding()["sha256"], r"^[0-9A-F]{64}$")

    def test_diagnostic_without_source_manifest_and_c_drive_output_fail_closed(self) -> None:
        tool = self.require_tool()
        result = tool.build_layout_edge_report(
            _timeline(), "child_dependency_patch", None
        )

        self.assertFalse(result["ready"])
        self.assertIn(
            "missing_source_manifest", {item["code"] for item in result["blockers"]}
        )
        with self.assertRaises(ValueError):
            tool.validate_output_path(Path("C:/zircon-profiles/layout-edge.json"))
        self.assertEqual(
            "E:",
            tool.validate_output_path(
                Path("E:/zircon-profiles/layout-edge.json")
            ).drive,
        )


def _scale_run(
    scenario: str,
    unrelated_slot_count: int,
    operation_visits: int,
    duration_p95_us: int,
    allocation_bytes: int,
) -> dict[str, object]:
    return {
        "schema": SCHEMA,
        "ready": True,
        "scenario": scenario,
        "unrelated_slot_count": unrelated_slot_count,
        "metrics": {
            "operation_visit_count": operation_visits,
            "operation_duration_p95_us": duration_p95_us,
            "allocation_bytes": allocation_bytes,
        },
    }


if __name__ == "__main__":
    unittest.main()
