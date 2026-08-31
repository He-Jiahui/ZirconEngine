import unittest

from tools.ui_measure_validity_locality_pressure import run


class RuntimeUiMeasureValidityLocalityPerformanceContractTests(unittest.TestCase):
    def test_invalid_parent_does_not_force_a_valid_clean_subtree(self) -> None:
        result = run(
            clean_subtree_node_count=10_000,
            update_count=10_000,
            required_measured_node_count=2,
            root_direct_child_count=2,
        )

        self.assertEqual(result["retired_forced_measured_node_work"], 100_020_000)
        self.assertEqual(result["local_validity_measured_node_work"], 20_000)
        self.assertEqual(result["local_validity_probe_node_work"], 30_000)
        self.assertEqual(result["eliminated_measured_node_work"], 100_000_000)
        self.assertEqual(result["measured_node_work_reduction_ratio"], 5_001)


if __name__ == "__main__":
    unittest.main()
