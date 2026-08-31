import unittest

from tools.plugins14_tiled_plan_pressure import run


class Plugins14TiledPlanPressureTests(unittest.TestCase):
    def test_shared_plan_reduces_reference_count_pairs_by_seventy_five_percent(self) -> None:
        work = run()["work"]

        self.assertEqual(work["baseline_arc_refcount_pair_count"], 800_000)
        self.assertEqual(work["candidate_arc_refcount_pair_count"], 200_000)
        self.assertEqual(work["arc_refcount_pair_reduction_percent"], 75.0)

    def test_shared_plan_reduces_modeled_atomic_operations_by_seventy_five_percent(
        self,
    ) -> None:
        work = run()["work"]

        self.assertEqual(work["baseline_modeled_atomic_rmw_count"], 1_600_000)
        self.assertEqual(work["candidate_modeled_atomic_rmw_count"], 400_000)
        self.assertEqual(work["atomic_rmw_reduction_percent"], 75.0)

    def test_plan_observation_and_zero_copy_harvest_are_preserved(self) -> None:
        work = run()["work"]

        self.assertEqual(work["baseline_plan_payload_observation_count"], 200_000)
        self.assertEqual(work["candidate_plan_payload_observation_count"], 200_000)
        self.assertEqual(work["baseline_completed_plan_copy_count"], 0)
        self.assertEqual(work["candidate_completed_plan_copy_count"], 0)

    def test_release_acceptance_requires_twenty_one_alternating_pairs(self) -> None:
        result = run()

        self.assertEqual(result["inputs"]["sample_pairs"], 21)
        self.assertEqual(result["acceptance"]["sample_order"], "alternating")
        self.assertEqual(result["acceptance"]["percentile_method"], "nearest_rank")
        self.assertEqual(
            result["acceptance"]["candidate_p95_maximum_legacy_ratio"], 0.8
        )
        self.assertTrue(result["acceptance"]["release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 2)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(clones_per_sample=0)
        with self.assertRaises(ValueError):
            run(sample_pairs=0)


if __name__ == "__main__":
    unittest.main()
