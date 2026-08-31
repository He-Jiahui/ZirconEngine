import unittest

from tools.editor_template_command_fragment_cache_pressure import pressure_report


class EditorTemplateCommandFragmentCachePressureTests(unittest.TestCase):
    def test_warm_cache_only_materializes_changed_fragment_commands(self):
        report = pressure_report(4096, 12, 1, 4, 10_000)

        self.assertEqual(
            report["current_region_rebuild"]["candidate_node_visits"], 49_152
        )
        self.assertEqual(
            report["current_region_rebuild"]["command_materializations"],
            196_608,
        )
        self.assertEqual(
            report["warm_retained_fragments"]["fragment_lookups"], 49_152
        )
        self.assertEqual(
            report["warm_retained_fragments"]["changed_node_rebuilds"], 4096
        )
        self.assertEqual(
            report["warm_retained_fragments"]["command_materializations"],
            16_384,
        )
        self.assertEqual(
            report["delta"]["eliminated_command_materializations"], 180_224
        )
        self.assertEqual(
            report["delta"]["command_materialization_reduction_ratio"], 12.0
        )
        self.assertEqual(report["delta"]["candidate_node_visit_reduction"], 0)
        self.assertFalse(report["is_product_timing"])

    def test_role_split_only_materializes_changed_interaction_commands(self):
        report = pressure_report(4096, 12, 1, 4, 10_000, 1)

        self.assertEqual(
            report["warm_retained_fragments"]["changed_fragment_rebuilds"], 4096
        )
        self.assertEqual(
            report["warm_retained_fragments"]["command_materializations"], 4096
        )
        self.assertEqual(
            report["delta"]["eliminated_command_materializations"], 192_512
        )
        self.assertEqual(
            report["delta"]["command_materialization_reduction_ratio"], 48.0
        )

    def test_all_changed_candidates_have_no_modeled_materialization_gain(self):
        report = pressure_report(2, 3, 3, 5, 100)

        self.assertEqual(
            report["delta"]["eliminated_command_materializations"], 0
        )
        self.assertEqual(
            report["delta"]["command_materialization_reduction_ratio"], 1.0
        )

    def test_rejects_invalid_or_inconsistent_inputs(self):
        for values in (
            (0, 1, 1, 1, 1),
            (1, 0, 1, 1, 1),
            (1, 1, 0, 1, 1),
            (1, 1, 1, 0, 1),
            (1, 1, 1, 1, 0),
            (1, 2, 3, 1, 10),
            (1, 11, 1, 1, 10),
            (1, 1, 1, 4, 10, 0),
            (1, 1, 1, 4, 10, 5),
        ):
            with self.subTest(values=values):
                with self.assertRaises(ValueError):
                    pressure_report(*values)

if __name__ == "__main__":
    unittest.main()
