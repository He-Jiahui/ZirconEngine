import unittest

from tools.runtime_input_manager_metadata_capacity_pressure import run


class RuntimeInputManagerMetadataCapacityPressureTests(unittest.TestCase):
    def test_dispatch_metadata_halves_top_level_passes_and_visits(self) -> None:
        metadata = run()["dispatch_metadata"]

        self.assertEqual(metadata["legacy_top_level_result_pass_count"], 4_096)
        self.assertEqual(metadata["candidate_top_level_result_pass_count"], 2_048)
        self.assertEqual(metadata["legacy_top_level_result_visit_count"], 1_048_576)
        self.assertEqual(metadata["candidate_top_level_result_visit_count"], 524_288)
        self.assertEqual(metadata["top_level_visit_reduction_percent"], 50.0)

    def test_ime_capacity_plans_the_maximum_batch_expansion_once(self) -> None:
        capacity = run()["ime_capacity"]

        self.assertEqual(capacity["input_method_request_count"], 262_144)
        self.assertEqual(capacity["maximum_host_request_value_count"], 786_432)
        self.assertEqual(capacity["legacy_planned_reserve_call_count"], 0)
        self.assertEqual(capacity["candidate_planned_reserve_call_count"], 1_024)
        self.assertEqual(capacity["candidate_planned_slot_count"], 786_432)
        self.assertEqual(capacity["capacity_multiplier"], 3)

    def test_semantic_invariants_are_explicit(self) -> None:
        invariants = run()["invariants"]

        self.assertTrue(invariants["host_request_order_preserved"])
        self.assertTrue(invariants["redraw_short_circuit_preserved"])
        self.assertTrue(invariants["ime_disable_short_circuit_preserved"])
        self.assertTrue(invariants["ime_optional_payload_semantics_preserved"])

    def test_model_is_bound_to_exact_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "5ffc4945095a6fc734bcbb2e632958026350b760",
        )
        self.assertEqual(
            binding["head_baseline_git_blobs"],
            {
                "zircon_runtime/src/ui/dispatch/input_manager/outcome.rs": (
                    "c44c34de8c33e0ba240aaab7c8dfd3970e86ac34"
                ),
                "zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs": (
                    "27a41991c710f91f060d14fc599537320f08373a"
                ),
            },
        )
        self.assertEqual(len(binding["source_sha256"]), 3)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_inputs(self) -> None:
        with self.assertRaises(ValueError):
            run(batch_count=0)
        with self.assertRaises(ValueError):
            run(results_per_batch=0)
        with self.assertRaises(ValueError):
            run(ime_appends_per_sample=0)
        with self.assertRaises(ValueError):
            run(ime_requests_per_append=0)
        with self.assertRaises(ValueError):
            run(max_host_requests_per_request=0)


if __name__ == "__main__":
    unittest.main()
