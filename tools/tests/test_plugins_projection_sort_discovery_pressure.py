import unittest

from tools.plugins_projection_sort_discovery_pressure import run


class PluginsProjectionSortDiscoveryPressureTests(unittest.TestCase):
    def test_world_sync_presizes_four_outputs_and_moves_nested_payloads(self) -> None:
        projection = run()["world_sync_projection"]

        self.assertEqual(projection["baseline_snapshot_capture_count"], 1)
        self.assertEqual(projection["candidate_snapshot_capture_count"], 1)
        self.assertEqual(projection["candidate_capacity_count_row_visit_count"], 65_536)
        self.assertEqual(projection["candidate_presized_output_vector_count"], 4)
        self.assertEqual(
            projection["baseline_modeled_nested_payload_clone_count"], 196_608
        )
        self.assertEqual(projection["candidate_modeled_nested_payload_clone_count"], 0)
        self.assertEqual(
            projection["candidate_modeled_nested_payload_move_count"], 196_608
        )

    def test_cached_sort_evaluates_the_key_once_per_pending_update(self) -> None:
        pending_sort = run()["pending_update_sort"]

        self.assertEqual(pending_sort["modeled_sort_comparison_count"], 10_240)
        self.assertEqual(pending_sort["baseline_sort_key_evaluation_count"], 20_480)
        self.assertEqual(pending_sort["candidate_sort_key_evaluation_count"], 1_024)
        self.assertEqual(pending_sort["key_evaluation_reduction_percent"], 95.0)
        self.assertEqual(pending_sort["baseline_expensive_graph_query_count"], 81_920)
        self.assertEqual(pending_sort["candidate_expensive_graph_query_count"], 4_096)

    def test_discovery_input_allocation_count_is_exact_for_six_clones(self) -> None:
        discovery = run()["discovery_input"]

        self.assertEqual(discovery["baseline_path_owner_allocation_count"], 1_835_008)
        self.assertEqual(discovery["candidate_path_owner_allocation_count"], 524_288)
        self.assertAlmostEqual(
            discovery["path_owner_allocation_reduction_percent"], 71.4285714286
        )
        self.assertEqual(
            discovery["baseline_deep_path_clone_allocation_count"], 1_572_864
        )
        self.assertEqual(discovery["candidate_deep_path_clone_allocation_count"], 0)
        self.assertEqual(discovery["candidate_shared_handle_clone_count"], 1_572_864)

    def test_historical_release_evidence_retains_exact_acceptance_data(self) -> None:
        evidence = run()["historical_release_evidence"]

        self.assertEqual(evidence["world_sync"]["p50_reduction_percent"], 90.66)
        self.assertEqual(evidence["world_sync"]["p95_reduction_percent"], 83.62)
        self.assertEqual(
            evidence["world_sync"]["allocation_reduction_percent"], 99.998474
        )
        self.assertEqual(evidence["pending_sort"]["p50_reduction_percent"], 95.3360)
        self.assertEqual(evidence["pending_sort"]["p95_reduction_percent"], 96.3322)
        self.assertEqual(
            evidence["pending_sort"]["allocation_reduction_percent"], 95.157692
        )
        self.assertEqual(evidence["discovery_input"]["p50_reduction_percent"], 60.772)
        self.assertEqual(evidence["discovery_input"]["p95_reduction_percent"], 59.934)
        self.assertEqual(
            evidence["discovery_input"]["allocation_reduction_percent"], 71.429
        )

    def test_checksums_remain_bound_to_each_historical_native_model(self) -> None:
        evidence = run()["historical_release_evidence"]

        self.assertEqual(evidence["world_sync"]["checksum"], 6_649_329_941_810_118_656)
        self.assertEqual(
            evidence["pending_sort"]["checksum"], 1_123_984_918_402_528_105
        )
        self.assertEqual(
            evidence["discovery_input"]["checksum"], 10_711_012_688_504_291_325
        )

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 4)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(physics_nodes=0)
        with self.assertRaises(ValueError):
            run(pending_updates=0)
        with self.assertRaises(ValueError):
            run(modeled_sort_comparisons=0)
        with self.assertRaises(ValueError):
            run(discovery_inputs=0)
        with self.assertRaises(ValueError):
            run(clones_per_input=0)


if __name__ == "__main__":
    unittest.main()
