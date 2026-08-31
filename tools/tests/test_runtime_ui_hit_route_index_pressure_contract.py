import unittest

from tools.runtime_ui_hit_route_index_pressure import run


class RuntimeUiHitRouteIndexPressureContract(unittest.TestCase):
    def test_model_counts_publication_patch_cow_and_route_payload_separately(self) -> None:
        result = run()

        self.assertEqual(
            result["retired_per_entry_routes"]["combined_work_units"],
            692_060_160,
        )
        self.assertEqual(
            result["retained_route_index"]["combined_work_units"],
            4_096_000,
        )
        self.assertEqual(
            result["retained_route_index"][
                "semantic_input_work_including_snapshot_cow"
            ],
            1_081_344,
        )
        self.assertEqual(
            result["retained_route_index"]["noop_input_route_table_clone_count"],
            0,
        )
        self.assertEqual(
            result["retired_per_entry_routes"]["route_only_payload_bytes"],
            25_460_736,
        )
        self.assertEqual(
            result["retained_route_index"]["route_only_payload_bytes"],
            262_144,
        )
        self.assertEqual(
            result["delta"]["route_only_payload_reduction_ratio"],
            97.125,
        )
        self.assertGreater(result["delta"]["work_reduction_ratio"], 160.0)

    def test_model_rejects_impossible_or_non_positive_inputs(self) -> None:
        with self.assertRaises(ValueError):
            run(node_count=0)
        with self.assertRaises(ValueError):
            run(node_count=100, hit_entry_count=101)
        with self.assertRaises(ValueError):
            run(node_count=100, chain_depth=101)


if __name__ == "__main__":
    unittest.main()
