import unittest

from tools.runtime_path_module_validation_pressure import run


class RuntimePathModuleValidationPressureTests(unittest.TestCase):
    def test_entity_path_plans_capacity_without_an_extra_scan(self) -> None:
        path = run()["entity_path"]

        self.assertEqual(path["baseline_input_scan_count"], 8_192)
        self.assertEqual(path["candidate_input_scan_count"], 8_192)
        self.assertEqual(path["baseline_explicit_reserve_call_count"], 0)
        self.assertEqual(path["candidate_explicit_reserve_call_count"], 8_192)
        self.assertEqual(path["planned_slots_per_path"], 4_095)
        self.assertEqual(path["candidate_planned_segment_slot_count"], 33_546_240)

    def test_entity_path_segment_ownership_work_is_preserved(self) -> None:
        path = run()["entity_path"]

        self.assertEqual(path["baseline_segment_ownership_allocation_count"], 8_388_608)
        self.assertEqual(path["candidate_segment_ownership_allocation_count"], 8_388_608)

    def test_module_field_halves_trim_calls_without_success_allocations(self) -> None:
        module = run()["module_field"]

        self.assertEqual(module["baseline_trim_call_count"], 16_384)
        self.assertEqual(module["candidate_trim_call_count"], 8_192)
        self.assertEqual(module["trim_call_reduction_percent"], 50.0)
        self.assertEqual(module["baseline_success_heap_allocations"], 0)
        self.assertEqual(module["candidate_success_heap_allocations"], 0)

    def test_model_is_bound_to_exact_current_and_head_sources(self) -> None:
        binding = run()["source_binding"]

        self.assertEqual(
            binding["git_revision"],
            "630d66c362013e3b5b72f97362ad56fc54ff6d8c",
        )
        self.assertEqual(
            binding["head_baseline_git_blobs"],
            {
                "zircon_runtime/src/core/framework/scene/entity_path.rs": (
                    "a0c00d7bff922c2d83affadf698c97054dae9bb6"
                ),
                "zircon_runtime/src/plugin/extension_registry/validation/runtime_core.rs": (
                    "99cac76fcb28638c1344122d976221b2dc129475"
                ),
            },
        )
        self.assertEqual(len(binding["source_sha256"]), 3)
        self.assertEqual(len(binding["source_manifest_sha256"]), 64)

    def test_rejects_non_positive_inputs(self) -> None:
        with self.assertRaises(ValueError):
            run(path_checks_per_sample=0)
        with self.assertRaises(ValueError):
            run(path_segments=0)
        with self.assertRaises(ValueError):
            run(segment_bytes=0)
        with self.assertRaises(ValueError):
            run(module_checks_per_sample=0)
        with self.assertRaises(ValueError):
            run(field_padding_bytes=0)


if __name__ == "__main__":
    unittest.main()
