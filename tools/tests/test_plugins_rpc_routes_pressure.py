import unittest

from tools.plugins_rpc_routes_pressure import run


class PluginsRpcRoutesPressureTests(unittest.TestCase):
    def test_rpc_expiration_eliminates_id_buffer_and_second_hash_pass(self) -> None:
        rpc = run()["rpc_expiration"]

        self.assertEqual(rpc["baseline_pending_table_scan_count"], 131_072)
        self.assertEqual(rpc["candidate_pending_table_scan_count"], 131_072)
        self.assertEqual(rpc["baseline_expired_id_materialization_count"], 32_768)
        self.assertEqual(rpc["candidate_expired_id_materialization_count"], 0)
        self.assertEqual(rpc["baseline_second_pass_hash_removal_count"], 32_768)
        self.assertEqual(rpc["candidate_second_pass_hash_removal_count"], 0)
        self.assertEqual(rpc["baseline_temporary_collection_count"], 2)
        self.assertEqual(rpc["candidate_temporary_collection_count"], 1)

    def test_rpc_expiration_preserves_report_cardinality(self) -> None:
        rpc = run()["rpc_expiration"]

        self.assertEqual(rpc["baseline_report_write_count"], 32_768)
        self.assertEqual(rpc["candidate_report_write_count"], 32_768)
        self.assertEqual(rpc["temporary_collection_reduction_percent"], 50.0)

    def test_route_expansion_eliminates_all_modeled_route_row_copies(self) -> None:
        routes = run()["route_expansion"]

        self.assertEqual(routes["baseline_shared_cache_route_clone_count"], 2_048)
        self.assertEqual(
            routes["baseline_shared_cache_route_row_copy_count"], 524_288
        )
        self.assertEqual(
            routes["baseline_cache_insertion_route_row_copy_count"], 526_592
        )
        self.assertEqual(routes["baseline_total_route_row_copy_count"], 1_050_880)
        self.assertEqual(routes["candidate_total_route_row_copy_count"], 0)

    def test_route_expansion_preserves_cache_entries_and_plans_gain_capacity(self) -> None:
        routes = run()["route_expansion"]

        self.assertEqual(routes["baseline_cache_insert_count"], 2_305)
        self.assertEqual(routes["candidate_cache_insert_count"], 2_305)
        self.assertEqual(routes["candidate_gain_reserve_call_count"], 2_304)
        self.assertEqual(routes["candidate_planned_gain_slot_count"], 526_592)

    def test_historical_release_evidence_keeps_exact_checksums_and_reductions(self) -> None:
        evidence = run()["historical_release_evidence"]

        self.assertEqual(evidence["rpc"]["checksum"], 8_727_815_200_911_380_074)
        self.assertEqual(evidence["rpc"]["p50_reduction_percent"], 60.6253)
        self.assertEqual(evidence["rpc"]["p95_reduction_percent"], 52.1047)
        self.assertEqual(evidence["rpc"]["allocation_reduction_percent"], 50.0)
        self.assertEqual(
            evidence["routes"]["checksum"], 13_349_105_238_628_374_174
        )
        self.assertEqual(evidence["routes"]["p50_reduction_percent"], 41.16)
        self.assertEqual(evidence["routes"]["p95_reduction_percent"], 36.67)
        self.assertEqual(
            evidence["routes"]["allocation_reduction_percent"], 81.742794
        )

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 3)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_invalid_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(pending_requests=0)
        with self.assertRaises(ValueError):
            run(expired_requests=0)
        with self.assertRaises(ValueError):
            run(source_tracks=0)
        with self.assertRaises(ValueError):
            run(downstream_tracks=0)
        with self.assertRaises(ValueError):
            run(pending_requests=4, expired_requests=5)


if __name__ == "__main__":
    unittest.main()
