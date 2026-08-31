import unittest

from tools.runtime_pointer_hover_hot_paths_pressure import run


class RuntimePointerHoverHotPathsPressureTests(unittest.TestCase):
    def test_stable_pointer_route_eliminates_clone_copy_and_allocation_work(self) -> None:
        report = run(event_count=1_000_000)
        retained = report["retained_hover_path"]

        self.assertEqual(retained["legacy_route_clone_count"], 1_000_000)
        self.assertEqual(retained["candidate_route_clone_count"], 0)
        self.assertEqual(retained["legacy_node_copy_count"], 512_000_000)
        self.assertEqual(retained["candidate_node_copy_count"], 0)
        self.assertEqual(retained["candidate_node_comparison_count"], 512_000_000)
        self.assertEqual(retained["legacy_vec_allocations_lower_bound"], 1_000_000)
        self.assertEqual(retained["candidate_vec_allocation_count"], 0)
        self.assertEqual(retained["legacy_payload_bytes_lower_bound"], 4_096_000_000)

    def test_large_hover_diff_replaces_quadratic_comparisons_with_linear_work(self) -> None:
        large = run(event_count=1_000_000)["hover_diff"]["large_path"]

        self.assertEqual(large["legacy_node_comparison_count"], 524_288_000_000)
        self.assertEqual(large["candidate_membership_insert_count"], 1_024_000_000)
        self.assertEqual(large["candidate_membership_lookup_count"], 1_024_000_000)
        self.assertEqual(large["candidate_membership_operation_count"], 2_048_000_000)
        self.assertEqual(large["candidate_membership_allocations"], 1_000_000)
        self.assertAlmostEqual(large["work_reduction_percent"], 99.609375)

    def test_small_hover_diff_keeps_the_zero_membership_allocation_path(self) -> None:
        small = run(event_count=1_000_000)["hover_diff"]["small_path"]

        self.assertEqual(small["route_depth"], 8)
        self.assertEqual(small["legacy_node_comparison_count"], 128_000_000)
        self.assertEqual(small["candidate_node_comparison_count"], 128_000_000)
        self.assertEqual(small["legacy_membership_allocations"], 0)
        self.assertEqual(small["candidate_membership_allocations"], 0)

    def test_model_is_bound_to_exact_current_and_head_sources(self) -> None:
        binding = run(event_count=1)["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "5ffc4945095a6fc734bcbb2e632958026350b760",
        )
        self.assertEqual(
            binding["head_baseline_git_blobs"],
            {
                "zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs": (
                    "a042804a567e258151a66c6635abd2a52c20e0ba"
                ),
                "zircon_runtime/src/ui/surface/surface/event_routing.rs": (
                    "9315b899d3f7cd79e2f6c0b2604e634f0332092b"
                ),
            },
        )
        self.assertEqual(len(binding["source_sha256"]), 3)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_invalid_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(event_count=0)
        with self.assertRaises(ValueError):
            run(event_count=1, route_depth=8)
        with self.assertRaises(ValueError):
            run(event_count=1, small_route_depth=9)
        with self.assertRaises(ValueError):
            run(event_count=1, node_identity_bytes=0)


if __name__ == "__main__":
    unittest.main()
