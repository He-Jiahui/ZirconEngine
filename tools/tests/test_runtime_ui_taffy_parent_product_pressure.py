from pathlib import Path
import unittest

from tools.runtime_ui_taffy_parent_product_pressure import (
    SCHEMA,
    SOURCE_GUARDS,
    build_report,
    parent_work,
    pressure_suite,
    validate_output_path,
    validate_source_texts,
)


REPO_ROOT = Path(__file__).resolve().parents[2]


class RuntimeUiTaffyParentProductPressureTests(unittest.TestCase):
    def test_schema_v2_binds_the_retained_order_index(self):
        self.assertEqual(SCHEMA, "zircon.runtime.ui_taffy_parent_product_pressure.v2")
        self.assertIn(
            "zircon_runtime/src/ui/layout/pass/slot.rs",
            SOURCE_GUARDS,
        )

    def test_warm_parent_visit_reuses_order_without_sorting(self):
        report = parent_work(
            event_count=1_000,
            visited_parent_count=1,
            children_per_parent=1_024,
            changed_children_per_parent=1,
        )

        current = report["current_scratch_rebuild"]
        self.assertEqual(current["ordered_child_index_lookup_count"], 1_000)
        self.assertEqual(current["ordered_child_sort_count"], 0)
        self.assertEqual(current["ordered_child_sort_item_count"], 0)
        self.assertEqual(current["taffy_node_create_count"], 1_025_000)

    def test_wide_parent_separates_topology_solve_and_output_work(self):
        report = parent_work(
            event_count=1_000,
            visited_parent_count=1,
            children_per_parent=1_024,
            changed_children_per_parent=1,
        )

        self.assertEqual(
            report["current_scratch_rebuild"]["taffy_node_create_count"],
            1_025_000,
        )
        self.assertEqual(
            report["retained_topology_conservative"]["taffy_node_create_count"],
            0,
        )
        self.assertEqual(
            report["retained_topology_conservative"]["taffy_compute_count"],
            1_000,
        )
        self.assertEqual(
            report["retained_topology_conservative"]["child_layout_read_count"],
            1_024_000,
        )
        self.assertEqual(
            report["retained_delta_patch"]["child_contract_visit_count"], 1_000
        )

    def test_nested_layout_keeps_ancestor_solves_in_the_model(self):
        report = pressure_suite(1_000)["scenarios"][
            "nested_auto_layout_leaf_change"
        ]

        self.assertEqual(
            report["current_scratch_rebuild"]["topology_build_count"], 8_000
        )
        self.assertEqual(
            report["current_scratch_rebuild"]["taffy_node_create_count"], 72_000
        )
        self.assertEqual(
            report["retained_topology_conservative"]["taffy_compute_count"], 8_000
        )
        self.assertEqual(
            report["comparison"]["conservative_compute_count_reduction"], 0
        )

    def test_independent_forest_does_not_model_unrelated_parent_work(self):
        report = pressure_suite(100)["scenarios"][
            "independent_forest_single_parent_change"
        ]

        self.assertEqual(report["unrelated_parent_count"], 10_000)
        self.assertEqual(report["unrelated_parent_visit_count"], 0)
        self.assertEqual(
            report["current_scratch_rebuild"]["parent_product_visit_count"], 100
        )

    def test_rejects_invalid_pressure_inputs(self):
        invalid = (
            dict(
                event_count=0,
                visited_parent_count=1,
                children_per_parent=1,
                changed_children_per_parent=1,
            ),
            dict(
                event_count=1,
                visited_parent_count=0,
                children_per_parent=1,
                changed_children_per_parent=1,
            ),
            dict(
                event_count=1,
                visited_parent_count=1,
                children_per_parent=1,
                changed_children_per_parent=2,
            ),
        )
        for values in invalid:
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    parent_work(**values)

    def test_source_guard_is_fail_closed(self):
        valid_sources = {
            path: "\n".join(tokens) for path, tokens in SOURCE_GUARDS.items()
        }
        self.assertTrue(validate_source_texts(valid_sources)["ready"])

        missing_clear = dict(valid_sources)
        bridge_path = "zircon_runtime/src/ui/layout/taffy_bridge/compute.rs"
        missing_clear[bridge_path] = missing_clear[bridge_path].replace(
            "self.taffy.clear();", ""
        )
        result = validate_source_texts(missing_clear)

        self.assertFalse(result["ready"])
        self.assertIn(
            {
                "code": "source_contract_changed",
                "relative_path": bridge_path,
                "missing_token": "self.taffy.clear();",
            },
            result["blockers"],
        )

    def test_current_source_is_bound_and_ready(self):
        report = build_report(REPO_ROOT, 10)

        self.assertTrue(report["ready"], report["source_binding"])
        revision = report["source_binding"]["git_revision"]
        self.assertEqual(len(revision), 40)
        int(revision, 16)
        self.assertEqual(
            len(report["source_binding"]["critical_source_files"]),
            len(SOURCE_GUARDS),
        )
        self.assertFalse(report["is_product_timing"])

    def test_artifact_output_is_limited_to_controlled_drives(self):
        with self.assertRaises(ValueError):
            validate_output_path(r"C:\zircon-profiles\taffy-parent.json")
        self.assertEqual(
            validate_output_path(r"E:\zircon-profiles\taffy-parent.json").drive.upper(),
            "E:",
        )


if __name__ == "__main__":
    unittest.main()
