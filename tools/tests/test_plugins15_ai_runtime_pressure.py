import unittest
from pathlib import Path

from tools.plugins15_ai_runtime_pressure import run


ROOT = Path(__file__).resolve().parents[2]
SNAPSHOT = ROOT / "zircon_plugins/ai/runtime/src/manager/snapshot.rs"
STATE = ROOT / "zircon_plugins/ai/runtime/src/manager/state.rs"
SCAN = ROOT / "zircon_plugins/ai/runtime/src/perception/scan.rs"
STIMULI = ROOT / "zircon_plugins/ai/runtime/src/perception/stimuli.rs"
BATCH_VALIDATOR = ROOT / "tools/zircon-validation-plugins15-ai-runtime-batch.ps1"


class Plugins15AiRuntimePressureTests(unittest.TestCase):
    def test_compiled_tree_generation_eliminates_deep_catalog_clones(self) -> None:
        compiled = run()["compiled_tree_generation"]

        self.assertEqual(compiled["baseline_top_level_tree_clone_count"], 8_192)
        self.assertEqual(compiled["candidate_top_level_tree_clone_count"], 0)
        self.assertEqual(compiled["baseline_compiled_node_copy_count"], 262_144)
        self.assertEqual(compiled["candidate_compiled_node_copy_count"], 0)
        self.assertEqual(compiled["candidate_arc_handle_clone_count"], 32)

    def test_ordered_stimuli_removes_snapshot_sort_only(self) -> None:
        ordered = run()["ordered_stimuli"]

        self.assertEqual(ordered["baseline_snapshot_sort_count"], 1)
        self.assertEqual(ordered["candidate_snapshot_sort_count"], 0)
        self.assertEqual(ordered["baseline_sort_input_element_count"], 8_192)
        self.assertEqual(ordered["candidate_sort_input_element_count"], 0)
        self.assertEqual(ordered["candidate_cloned_stimulus_count"], 8_192)

    def test_single_pass_sampling_halves_world_projection_work(self) -> None:
        sampling = run()["single_pass_sampling"]

        self.assertEqual(sampling["baseline_world_projection_count"], 2)
        self.assertEqual(sampling["candidate_world_projection_count"], 1)
        self.assertEqual(sampling["baseline_projected_node_record_count"], 8_192)
        self.assertEqual(sampling["candidate_projected_node_record_count"], 4_096)
        self.assertEqual(sampling["candidate_redundant_sample_sort_count"], 0)

    def test_targeted_debug_snapshot_bounds_agent_projection(self) -> None:
        targeted = run()["targeted_debug_snapshot"]

        self.assertEqual(targeted["baseline_agent_projection_count"], 8_192)
        self.assertEqual(targeted["candidate_agent_projection_count"], 256)
        self.assertEqual(targeted["candidate_global_key_union_count"], 0)
        self.assertEqual(targeted["candidate_behavior_tree_catalog_clone_count"], 0)

    def test_compiled_tree_source_keeps_arc_and_release_gate(self) -> None:
        source = STATE.read_text(encoding="utf-8")

        self.assertIn("compiled_behavior_tree_generation: Arc<[CompiledBehaviorTree]>", source)
        self.assertIn("immutable_compiled_tree_generation_release_benchmark_evidence", source)
        self.assertIn("optimized_p95.saturating_mul(10) <= legacy_p95", source)

    def test_ordered_stimuli_source_keeps_sort_free_snapshot(self) -> None:
        source = STIMULI.read_text(encoding="utf-8")
        snapshot = source.split("fn snapshot(", 1)[1].split("fn sense_rank", 1)[0]

        self.assertIn("BTreeMap<StimulusKey, AiPerceptionStimulus>", source)
        self.assertNotIn("sort_by", snapshot)
        self.assertIn("PERF_RESULT plugins15_ordered_perception_stimuli", source)

    def test_sampling_source_keeps_single_projection_collector(self) -> None:
        source = SCAN.read_text(encoding="utf-8")
        collector = source.split("fn collect_perception_samples", 1)[1].split(
            "#[derive(Clone, Copy, Debug)]", 1
        )[0]

        self.assertEqual(collector.count(".node_records()"), 1)
        self.assertNotIn("sort_by", collector)
        self.assertIn("PERF_RESULT plugins15_single_pass_perception_sampling", source)

    def test_targeted_snapshot_source_keeps_bounded_release_gate(self) -> None:
        source = SNAPSHOT.read_text(encoding="utf-8")

        self.assertIn("build_agent_runtime_snapshots", source)
        self.assertIn("PERF_RESULT plugins15_targeted_debug_snapshot", source)
        self.assertIn("optimized_p95.saturating_mul(4) <= legacy_p95", source)

    def test_batch_validator_runs_four_exact_release_benchmarks(self) -> None:
        source = BATCH_VALIDATOR.read_text(encoding="utf-8")

        self.assertEqual(source.count("release_benchmark_evidence\""), 4)
        for argument in (
            '"--locked"',
            '"--release"',
            '"--jobs"',
            '"1"',
            '"--exact"',
            '"--ignored"',
            '"--nocapture"',
            '"--test-threads=1"',
        ):
            self.assertIn(argument, source)
        self.assertIn("PLUGINS15_BATCH_PASS", source)

    def test_release_acceptance_is_explicit_and_pending(self) -> None:
        acceptance = run()["acceptance"]

        self.assertEqual(acceptance["compiled_tree_p95_maximum_legacy_ratio"], 0.10)
        self.assertEqual(acceptance["ordered_stimuli_p95_maximum_legacy_ratio"], 0.75)
        self.assertEqual(acceptance["single_pass_sampling_p95_maximum_legacy_ratio"], 0.75)
        self.assertEqual(acceptance["targeted_debug_snapshot_p95_maximum_legacy_ratio"], 0.25)
        self.assertTrue(acceptance["release_timing_pending"])

    def test_model_is_bound_to_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "ca3ac3cc6ad218d04a5cd469447cea2452441321",
        )
        self.assertEqual(len(binding["source_sha256"]), 6)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_invalid_workloads(self) -> None:
        with self.assertRaises(ValueError):
            run(tree_count=0)
        with self.assertRaises(ValueError):
            run(stimulus_count=0)
        with self.assertRaises(ValueError):
            run(active_agent_count=257, total_agent_count=256)


if __name__ == "__main__":
    unittest.main()
