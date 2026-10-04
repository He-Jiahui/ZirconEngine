from pathlib import Path
import unittest

from tools.analysis.performance.runtime.runtime_ui_taffy_parent_product_pressure import (
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
    def test_schema_v5_binds_the_retained_order_and_parent_products(self):
        self.assertEqual(SCHEMA, "zircon.runtime.ui_taffy_parent_product_pressure.v5")
        self.assertIn(
            "zircon_runtime/src/ui/layout/pass/slot.rs",
            SOURCE_GUARDS,
        )
        self.assertIn(
            "zircon_runtime/src/ui/layout/pass/engine.rs",
            SOURCE_GUARDS,
        )
        self.assertIn(
            "zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs",
            SOURCE_GUARDS,
        )
        self.assertIn(
            "mutation_source_node_ids",
            SOURCE_GUARDS["zircon_runtime/src/ui/layout/pass/engine.rs"],
        )

    def test_warm_parent_visit_reuses_order_without_sorting(self):
        report = parent_work(
            event_count=1_000,
            visited_parent_count=1,
            children_per_parent=1_024,
            changed_children_per_parent=1,
        )

        baseline = report["pre_retained_scratch_rebuild"]
        self.assertEqual(baseline["ordered_child_index_lookup_count"], 1_000)
        self.assertEqual(baseline["ordered_child_sort_count"], 0)
        self.assertEqual(baseline["ordered_child_sort_item_count"], 0)
        self.assertEqual(baseline["taffy_node_create_count"], 1_025_000)

    def test_wide_parent_separates_topology_solve_and_output_work(self):
        report = parent_work(
            event_count=1_000,
            visited_parent_count=1,
            children_per_parent=1_024,
            changed_children_per_parent=1,
        )

        self.assertEqual(
            report["pre_retained_scratch_rebuild"]["taffy_node_create_count"],
            1_025_000,
        )
        self.assertEqual(
            report["m1_retained_topology"]["taffy_node_create_count"],
            0,
        )
        self.assertEqual(
            report["m1_retained_topology"]["taffy_compute_count"],
            1_000,
        )
        self.assertEqual(
            report["m1_retained_topology"]["child_layout_read_count"],
            1_024_000,
        )
        self.assertEqual(
            report["m2_retained_delta_patch"]["child_contract_visit_count"], 1_000
        )

    def test_nested_layout_keeps_ancestor_solves_in_the_model(self):
        report = pressure_suite(1_000)["scenarios"][
            "nested_auto_layout_leaf_change"
        ]

        self.assertEqual(
            report["pre_retained_scratch_rebuild"]["topology_build_count"], 8_000
        )
        self.assertEqual(
            report["pre_retained_scratch_rebuild"]["taffy_node_create_count"], 72_000
        )
        self.assertEqual(
            report["m1_retained_topology"]["taffy_compute_count"], 8_000
        )
        self.assertEqual(
            report["comparison"]["m3_avoided_compute_count"], 0
        )

    def test_window_resize_reuses_child_contracts_but_not_solve_output(self):
        report = pressure_suite(1_000)["scenarios"][
            "window_resize_all_visible_parents"
        ]

        self.assertEqual(
            report["m1_retained_topology"]["child_contract_visit_count"],
            192_000,
        )
        self.assertEqual(
            report["m2_retained_delta_patch"]["child_contract_visit_count"],
            0,
        )
        self.assertEqual(
            report["current_retained_solve_output_reuse"]["taffy_compute_count"],
            12_000,
        )
        self.assertEqual(
            report["current_retained_solve_output_reuse"]["child_layout_read_count"],
            192_000,
        )

    def test_unchanged_child_contract_reuses_solve_and_publishes_exact_receipt(self):
        report = pressure_suite(1_000)["scenarios"][
            "wide_parent_unchanged_child_contract"
        ]["current_retained_solve_output_reuse"]

        self.assertEqual(report["child_contract_visit_count"], 1_000)
        self.assertEqual(report["child_style_update_count"], 0)
        self.assertEqual(report["taffy_compute_count"], 0)
        self.assertEqual(report["taffy_compute_reuse_count"], 1_000)
        self.assertEqual(report["child_layout_read_count"], 0)
        self.assertEqual(report["published_child_frame_count"], 1_000)

    def test_translation_reuses_solve_but_publishes_every_child(self):
        report = pressure_suite(1_000)["scenarios"]["wide_parent_translation"][
            "current_retained_solve_output_reuse"
        ]

        self.assertEqual(report["taffy_compute_count"], 0)
        self.assertEqual(report["taffy_compute_reuse_count"], 1_000)
        self.assertEqual(report["child_layout_read_count"], 0)
        self.assertEqual(report["published_child_frame_count"], 1_024_000)

    def test_independent_forest_does_not_model_unrelated_parent_work(self):
        report = pressure_suite(100)["scenarios"][
            "independent_forest_single_parent_change"
        ]

        self.assertEqual(report["unrelated_parent_count"], 10_000)
        self.assertEqual(report["unrelated_parent_visit_count"], 0)
        self.assertEqual(
            report["pre_retained_scratch_rebuild"]["parent_product_visit_count"], 100
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

        missing_retained_insert = dict(valid_sources)
        product_path = "zircon_runtime/src/ui/layout/taffy_bridge/product_cache.rs"
        missing_retained_insert[product_path] = missing_retained_insert[product_path].replace(
            "self.products.insert(parent_id, product)", ""
        )
        result = validate_source_texts(missing_retained_insert)

        self.assertFalse(result["ready"])
        self.assertIn(
            {
                "code": "source_contract_changed",
                "relative_path": product_path,
                "missing_token": "self.products.insert(parent_id, product)",
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
